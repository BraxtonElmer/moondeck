//! Installed apps (from the Start menu) and their real icons, for the deck's app picker.

use std::{
    collections::{BTreeMap, HashMap},
    ffi::c_void,
    path::{Path, PathBuf},
    sync::{Mutex, OnceLock},
};

use base64::Engine;
use windows::{
    core::PCWSTR,
    Win32::{
        Graphics::Gdi::{
            CreateCompatibleDC, DeleteDC, DeleteObject, GetDIBits, GetObjectW, BITMAP, BITMAPINFO, BITMAPINFOHEADER,
            BI_RGB, DIB_RGB_COLORS,
        },
        Storage::FileSystem::FILE_FLAGS_AND_ATTRIBUTES,
        UI::{
            Shell::{SHGetFileInfoW, SHFILEINFOW, SHGFI_ICON, SHGFI_LARGEICON},
            WindowsAndMessaging::{DestroyIcon, GetIconInfo, ICONINFO},
        },
    },
};

const SKIP: &[&str] = &["uninstall", "help", "readme", "documentation", "website", "release notes", "faq", "manual", "license"];

fn start_menu_roots() -> Vec<PathBuf> {
    [("APPDATA", r"Microsoft\Windows\Start Menu\Programs"), ("ProgramData", r"Microsoft\Windows\Start Menu\Programs")]
        .iter()
        .filter_map(|(var, sub)| std::env::var(var).ok().map(|p| PathBuf::from(p).join(sub)))
        .collect()
}

fn collect(dir: &Path, depth: u8, out: &mut BTreeMap<String, String>) {
    let Ok(entries) = std::fs::read_dir(dir) else { return };
    for e in entries.flatten() {
        let path = e.path();
        if path.is_dir() {
            if depth > 0 {
                collect(&path, depth - 1, out);
            }
        } else if path.extension().is_some_and(|x| x.eq_ignore_ascii_case("lnk")) {
            let Some(name) = path.file_stem().map(|s| s.to_string_lossy().into_owned()) else { continue };
            let lower = name.to_lowercase();
            if !SKIP.iter().any(|s| lower.contains(s)) {
                out.entry(lower).or_insert(name);
            }
        }
    }
}

/// Start menu app names, sorted, without uninstallers and help links.
pub fn list() -> Vec<String> {
    let mut found = BTreeMap::new();
    for root in start_menu_roots() {
        collect(&root, 3, &mut found);
    }
    found.into_values().collect()
}

fn cache() -> &'static Mutex<HashMap<String, Option<String>>> {
    static CACHE: OnceLock<Mutex<HashMap<String, Option<String>>>> = OnceLock::new();
    CACHE.get_or_init(Default::default)
}

/// The icon Windows shows for `path` (exe, shortcut, file or folder), as a PNG data URL.
pub fn icon_data_url(path: &str) -> Option<String> {
    if let Some(hit) = cache().lock().unwrap().get(path) {
        return hit.clone();
    }
    let url = unsafe { icon_png(path) }
        .map(|png| format!("data:image/png;base64,{}", base64::engine::general_purpose::STANDARD.encode(png)));
    cache().lock().unwrap().insert(path.to_string(), url.clone());
    url
}

unsafe fn icon_png(path: &str) -> Option<Vec<u8>> {
    let wide: Vec<u16> = path.encode_utf16().chain(std::iter::once(0)).collect();
    let mut info = SHFILEINFOW::default();
    let ok = SHGetFileInfoW(
        PCWSTR(wide.as_ptr()),
        FILE_FLAGS_AND_ATTRIBUTES(0),
        Some(&mut info),
        std::mem::size_of::<SHFILEINFOW>() as u32,
        SHGFI_ICON | SHGFI_LARGEICON,
    );
    if ok == 0 || info.hIcon.is_invalid() {
        return None;
    }

    let mut ii = ICONINFO::default();
    let got = GetIconInfo(info.hIcon, &mut ii).is_ok();
    let result = got.then(|| {
        let mut bm = BITMAP::default();
        GetObjectW(ii.hbmColor.into(), std::mem::size_of::<BITMAP>() as i32, Some(&mut bm as *mut _ as *mut c_void));
        let (w, h) = (bm.bmWidth, bm.bmHeight);
        if w <= 0 || h <= 0 {
            return None;
        }
        let mut bmi = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: w,
                biHeight: -h, // top-down rows
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB.0,
                ..Default::default()
            },
            ..Default::default()
        };
        let mut px = vec![0u8; (w * h * 4) as usize];
        let dc = CreateCompatibleDC(None);
        let rows = GetDIBits(dc, ii.hbmColor, 0, h as u32, Some(px.as_mut_ptr() as *mut c_void), &mut bmi, DIB_RGB_COLORS);
        let _ = DeleteDC(dc);
        if rows == 0 {
            return None;
        }
        // BGRA -> RGBA. Old-style icons carry no alpha at all; treat them as opaque.
        let has_alpha = px.chunks(4).any(|p| p[3] != 0);
        for p in px.chunks_mut(4) {
            p.swap(0, 2);
            if !has_alpha {
                p[3] = 255;
            }
        }
        encode_png(&px, w as u32, h as u32)
    });

    if !ii.hbmColor.is_invalid() {
        let _ = DeleteObject(ii.hbmColor.into());
    }
    if !ii.hbmMask.is_invalid() {
        let _ = DeleteObject(ii.hbmMask.into());
    }
    let _ = DestroyIcon(info.hIcon);
    result.flatten()
}

fn encode_png(rgba: &[u8], w: u32, h: u32) -> Option<Vec<u8>> {
    let mut out = Vec::new();
    {
        let mut enc = png::Encoder::new(&mut out, w, h);
        enc.set_color(png::ColorType::Rgba);
        enc.set_depth(png::BitDepth::Eight);
        let mut writer = enc.write_header().ok()?;
        writer.write_image_data(rgba).ok()?;
    }
    Some(out)
}

#[cfg(test)]
mod tests {
    #[test]
    #[ignore] // depends on this machine's Start menu
    fn lists_apps_and_icons() {
        let apps = super::list();
        println!("{} apps, e.g. {:?}", apps.len(), &apps[..apps.len().min(8)]);
        assert!(!apps.is_empty());
        let notepad = std::env::var("WINDIR").unwrap() + r"\System32\notepad.exe";
        let url = super::icon_data_url(&notepad).expect("notepad icon");
        assert!(url.starts_with("data:image/png;base64,"));
    }
}
