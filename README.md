# Moondeck

A glass overlay for Windows that checks whether you actually did the thing.

Most reminder apps work on time. Moondeck works on proof: it watches signals
(apps you used, commits you pushed, game dailies, files you touched) and only
nudges you when the proof is missing.

`Alt + Space` opens the overlay. `Esc` or a click on empty space sends it back to the tray.

## Layout

- **Top** – clock, goals done today, next deadline, command bar
- **Left** – Today / Timeline / Streaks
- **Right** – what's up next, snooze, quick notes
- **Bottom** – the deck: launch workspaces, focus, timer, media, tools

## Build

Tauri v2 + Rust. The frontend is plain HTML/CSS/JS in `ui/` with no build step.

```
cd src-tauri
cargo run
```

Release build: `cargo build --release`.
