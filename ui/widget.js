"use strict";

// A pinned widget window. Which one is decided by the window label: widget-next, widget-timer, widget-notes.
// Timer, focus and notes share localStorage with the overlay, so both stay in sync.

const tauri = window.__TAURI__;
const invoke = tauri?.core?.invoke ?? (async () => null);
const kind = (tauri?.window?.getCurrentWindow?.().label ?? "widget-" + (location.hash.slice(1) || "timer")).replace("widget-", "");
const $ = (id) => document.getElementById(id);
const esc = (s) => String(s).replace(/[&<>"']/g, (c) => `&#${c.charCodeAt(0)};`);
const store = {
  get(k, d) { try { return JSON.parse(localStorage.getItem(k)) ?? d; } catch { return d; } },
  set(k, v) { try { localStorage.setItem(k, JSON.stringify(v)); } catch {} },
};
const fmt = (d) => d.toTimeString().slice(0, 5);
const TITLES = { next: "Up next", timer: "Timer", notes: "Notes" };

document.body.classList.add("w-" + kind);

async function syncAccent() {
  const hex = await invoke("accent_color").catch(() => null);
  if (!hex) return;
  const [r, g, b] = [1, 3, 5].map((i) => parseInt(hex.slice(i, i + 2), 16));
  document.documentElement.style.setProperty("--accent", hex);
  document.documentElement.style.setProperty("--accent-ink", (0.299 * r + 0.587 * g + 0.114 * b) / 255 > 0.5 ? "#111" : "#fff");
}

function frame(body) {
  $("w").innerHTML = `
    <header data-tauri-drag-region><span data-tauri-drag-region>${TITLES[kind] ?? kind}</span>
      <button class="wclose" data-act="close" aria-label="Unpin ${TITLES[kind]}">×</button></header>
    <div class="wbody">${body}</div>`;
}

// ---------- up next ----------
let next = null;
async function loadNext() {
  const [goals, tasks] = await Promise.all([invoke("goals_today").catch(() => []), invoke("tasks_open").catch(() => [])]);
  const now = Date.now() / 1000;
  const items = [
    ...(goals ?? []).filter((g) => g.scheduled && !g.done && !g.skipped && !g.snoozed)
      .map((g) => ({ kind: "goal", id: g.id, name: g.name, due_in: g.due_in, sub: `${g.source} · ${g.detail}` })),
    ...(tasks ?? []).filter((t) => t.due != null)
      .map((t) => ({ kind: "task", id: t.id, name: t.title, due_in: t.due - now, sub: t.detail })),
  ].sort((a, b) => (a.due_in ?? 1e9) - (b.due_in ?? 1e9));
  next = { item: items[0] ?? null, more: Math.max(0, items.length - 1) };
  render();
}
function left(secs) {
  if (secs == null) return "today";
  if (secs < 0) return "overdue";
  const m = Math.round(secs / 60);
  return m >= 60 ? `${Math.floor(m / 60)}h ${m % 60}m left` : `${m}m left`;
}

// ---------- timer ----------
function timerState() {
  const end = store.get("timerEnd", null);
  const ms = end ? Math.max(0, end - Date.now()) : 0;
  return {
    end,
    running: ms > 0,
    text: ms > 0 ? `${Math.floor(ms / 60000)}:${String(Math.floor((ms % 60000) / 1000)).padStart(2, "0")}` : end ? "Time’s up" : "0:00",
  };
}

function render() {
  if (kind === "next") {
    if (!next) return frame(`<p class="wmuted">Loading…</p>`);
    const it = next.item;
    frame(it
      ? `<b class="wtitle">${esc(it.name)}</b>
         <span class="wmuted">${esc(it.sub)}</span>
         <div class="wrow"><span class="${it.due_in != null && it.due_in < 0 ? "late" : "wmuted"}">${left(it.due_in)}${next.more ? ` · ${next.more} more` : ""}</span>
         <button class="wbtn primary" data-act="done">Done</button></div>`
      : `<b class="wtitle">All clear</b><span class="wmuted">Nothing due right now.</span>`);
  } else if (kind === "timer") {
    const t = timerState();
    frame(`<div class="wbig${t.end && !t.running ? " up" : ""}">${t.text}</div>
      <div class="wrow">${[5, 15, 25, 50].map((m) => `<button class="wbtn" data-act="timer" data-arg="${m}">${m}m</button>`).join("")}
      ${t.end ? `<button class="wbtn" data-act="stop" aria-label="Stop timer">■</button>` : ""}</div>`);
  } else if (kind === "notes") {
    const notes = store.get("notes", []);
    frame(`<div class="wlist">${notes.map((n, i) => `<div class="wnote"><span>${esc(n)}</span><button data-act="rmnote" data-arg="${i}" aria-label="Delete note">×</button></div>`).join("") || `<p class="wmuted">No notes yet.</p>`}</div>
      <input id="wnote" placeholder="Jot something… (Enter)" aria-label="New note">`);
  }
}

const ACTIONS = {
  close: () => invoke("close_widget", { kind }),
  done: () => {
    const it = next?.item;
    if (!it) return;
    (it.kind === "goal" ? invoke("mark_goal", { id: it.id, kind: "done" }) : invoke("set_task_status", { id: it.id, status: "done" })).then(loadNext);
  },
  timer: (m) => { store.set("timerEnd", Date.now() + Number(m) * 60000); render(); },
  stop: () => { store.set("timerEnd", null); render(); },
  rmnote: (i) => { const n = store.get("notes", []); n.splice(Number(i), 1); store.set("notes", n); render(); },
};

document.addEventListener("click", (e) => {
  const el = e.target.closest("[data-act]");
  if (el) ACTIONS[el.dataset.act]?.(el.dataset.arg);
});
document.addEventListener("keydown", (e) => {
  if (e.key === "Enter" && e.target.id === "wnote" && e.target.value.trim()) {
    store.set("notes", [e.target.value.trim(), ...store.get("notes", [])].slice(0, 12));
    render();
    $("wnote")?.focus();
  }
});
// Another window changed timer or notes.
window.addEventListener("storage", (e) => {
  if ((kind === "notes" && e.key === "notes") || (kind === "timer" && e.key === "timerEnd")) render();
});

if (kind === "timer") setInterval(render, 1000);
if (kind === "next") { loadNext(); setInterval(loadNext, 60000); window.addEventListener("focus", loadNext); }
syncAccent();
render();
