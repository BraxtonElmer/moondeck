//! Just enough IMAP over TLS to read new message headers and a text snippet, read-only.

use std::{
    io::{BufRead, BufReader, Read, Write},
    net::TcpStream,
    sync::Arc,
    time::Duration,
};

use rustls::{pki_types::ServerName, ClientConfig, ClientConnection, RootCertStore, StreamOwned};

type Res<T> = Result<T, String>;

fn err(e: impl std::fmt::Display) -> String {
    e.to_string()
}

pub struct Imap {
    io: BufReader<StreamOwned<ClientConnection, TcpStream>>,
    tag: u32,
}

/// One untagged response line, with any `{n}` literals it carried.
struct Line {
    text: String,
    literals: Vec<Vec<u8>>,
}

pub struct Message {
    pub uid: u32,
    pub header: String,
    pub body: Vec<u8>,
}

impl Imap {
    pub fn connect(host: &str) -> Res<Self> {
        let roots = RootCertStore { roots: webpki_roots::TLS_SERVER_ROOTS.to_vec() };
        let config = ClientConfig::builder_with_provider(Arc::new(rustls::crypto::ring::default_provider()))
            .with_safe_default_protocol_versions()
            .map_err(err)?
            .with_root_certificates(roots)
            .with_no_client_auth();
        let name = ServerName::try_from(host.to_string()).map_err(err)?;
        let conn = ClientConnection::new(Arc::new(config), name).map_err(err)?;
        let tcp = TcpStream::connect((host, 993)).map_err(err)?;
        tcp.set_read_timeout(Some(Duration::from_secs(30))).map_err(err)?;
        tcp.set_write_timeout(Some(Duration::from_secs(30))).map_err(err)?;
        let mut imap = Self { io: BufReader::new(StreamOwned::new(conn, tcp)), tag: 0 };
        imap.read_raw_line()?; // server greeting
        Ok(imap)
    }

    fn read_raw_line(&mut self) -> Res<String> {
        let mut buf = Vec::new();
        self.io.read_until(b'\n', &mut buf).map_err(err)?;
        if buf.is_empty() {
            return Err("server closed the connection".into());
        }
        Ok(String::from_utf8_lossy(&buf).into_owned())
    }

    fn command(&mut self, cmd: &str) -> Res<Vec<Line>> {
        self.tag += 1;
        let tag = format!("m{}", self.tag);
        let stream = self.io.get_mut();
        stream.write_all(format!("{tag} {cmd}\r\n").as_bytes()).map_err(err)?;
        stream.flush().map_err(err)?;

        let mut lines = Vec::new();
        loop {
            let mut line = Line { text: String::new(), literals: Vec::new() };
            loop {
                let raw = self.read_raw_line()?;
                let trimmed = raw.trim_end();
                if let Some(n) = literal_len(trimmed) {
                    line.text.push_str(trimmed);
                    let mut data = vec![0; n];
                    self.io.read_exact(&mut data).map_err(err)?;
                    line.literals.push(data);
                    continue;
                }
                line.text.push_str(trimmed);
                break;
            }
            if let Some(rest) = line.text.strip_prefix(&format!("{tag} ")) {
                return if rest.starts_with("OK") { Ok(lines) } else { Err(rest.to_string()) };
            }
            lines.push(line);
        }
    }

    pub fn login(&mut self, user: &str, pass: &str) -> Res<()> {
        self.command(&format!("LOGIN {} {}", quote(user), quote(pass)))
            .map(|_| ())
            .map_err(|e| format!("login failed: {e}"))
    }

    /// Read-only select, so nothing is ever marked as read.
    pub fn examine(&mut self, mailbox: &str) -> Res<()> {
        self.command(&format!("EXAMINE {}", quote(mailbox))).map(|_| ())
    }

    /// UIDs of messages received on or after `since` ("4-Oct-2026").
    pub fn uids_since(&mut self, since: &str) -> Res<Vec<u32>> {
        let lines = self.command(&format!("UID SEARCH SINCE {since}"))?;
        Ok(lines
            .iter()
            .filter_map(|l| l.text.strip_prefix("* SEARCH"))
            .flat_map(|rest| rest.split_whitespace().filter_map(|n| n.parse().ok()).collect::<Vec<u32>>())
            .collect())
    }

    /// Headers plus the first ~2 KB of the first body part, without setting \Seen.
    pub fn fetch(&mut self, uids: &[u32]) -> Res<Vec<Message>> {
        if uids.is_empty() {
            return Ok(Vec::new());
        }
        let set = uids.iter().map(u32::to_string).collect::<Vec<_>>().join(",");
        let lines = self.command(&format!(
            "UID FETCH {set} (UID BODY.PEEK[HEADER.FIELDS (FROM SUBJECT DATE)] BODY.PEEK[1]<0.2000>)"
        ))?;
        Ok(lines
            .into_iter()
            .filter(|l| l.text.contains("FETCH"))
            .filter_map(|mut l| {
                let uid = l.text.split("UID ").nth(1)?.split(|c: char| !c.is_ascii_digit()).next()?.parse().ok()?;
                let mut lits = l.literals.drain(..);
                let header = String::from_utf8_lossy(&lits.next().unwrap_or_default()).into_owned();
                let body = lits.next().unwrap_or_default();
                Some(Message { uid, header, body })
            })
            .collect())
    }

    pub fn logout(mut self) {
        let _ = self.command("LOGOUT");
    }
}

fn literal_len(line: &str) -> Option<usize> {
    let body = line.strip_suffix('}')?;
    let open = body.rfind('{')?;
    body[open + 1..].trim_end_matches('+').parse().ok()
}

fn quote(s: &str) -> String {
    format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\""))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn literals() {
        assert_eq!(literal_len("* 1 FETCH (UID 5 BODY[1]<0> {123}"), Some(123));
        assert_eq!(literal_len("* OK done"), None);
    }

    #[test]
    fn quoting() {
        assert_eq!(quote(r#"pa"ss\word"#), r#""pa\"ss\\word""#);
    }
}
