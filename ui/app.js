"use strict";

const tauri = window.__TAURI__;
const invoke = tauri?.core?.invoke ?? (async () => {});
const $ = (id) => document.getElementById(id);

// ---------- icons ----------
const ICON = {
  search: '<circle cx="11" cy="11" r="7"/><path d="M20 20l-4-4"/>',
  sliders: '<path d="M4 7h10M18 7h2M4 17h4M12 17h8"/><circle cx="16" cy="7" r="2"/><circle cx="10" cy="17" r="2"/>',
  clock: '<circle cx="12" cy="12" r="9"/><path d="M12 7v5l3 2"/>',
  check: '<path d="M5 12l5 5L20 7"/>',
  flame: '<path d="M12 3c1 4 5 5.5 5 10a5 5 0 0 1-10 0c0-3 2-4 2.5-6.5C11 8 12 9 12.5 9.5 13 7 12.5 5 12 3z"/>',
  plus: '<path d="M12 5v14M5 12h14"/>',
  pencil: '<path d="M4 20h4L19 9l-4-4L4 16z"/>',
  sun: '<circle cx="12" cy="12" r="4"/><path d="M12 2v2M12 20v2M4.9 4.9l1.4 1.4M17.7 17.7l1.4 1.4M2 12h2M20 12h2M4.9 19.1l1.4-1.4M17.7 6.3l1.4-1.4"/>',
  folder: '<path d="M3 7a2 2 0 0 1 2-2h4l2 2h8a2 2 0 0 1 2 2v8a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z"/>',
  brief: '<rect x="3" y="7" width="18" height="13" rx="2"/><path d="M9 7V5h6v2"/>',
  cube: '<path d="M12 3l8 4.5v9L12 21l-8-4.5v-9z"/><path d="M12 12l8-4.5M12 12v9M12 12L4 7.5"/>',
  pad: '<rect x="2.5" y="7" width="19" height="11" rx="5.5"/><path d="M7 11v3M5.5 12.5h3"/><circle cx="16" cy="11.5" r="0.8"/><circle cx="18" cy="13.5" r="0.8"/>',
  target: '<circle cx="12" cy="12" r="8"/><circle cx="12" cy="12" r="3"/>',
  timer: '<circle cx="12" cy="13" r="8"/><path d="M12 9v4l2.5 2M9 2h6"/>',
  mic: '<rect x="9" y="3" width="6" height="11" rx="3"/><path d="M5 11a7 7 0 0 0 14 0M12 18v3"/>',
  micoff: '<rect x="9" y="3" width="6" height="11" rx="3"/><path d="M5 11a7 7 0 0 0 14 0M12 18v3M4 4l16 16"/>',
  play: '<path d="M8 5v14l11-7z" fill="currentColor"/>',
  pause: '<path d="M8 5v14M16 5v14" stroke-width="2.6"/>',
  vol: '<path d="M4 9h4l5-4v14l-5-4H4z"/><path d="M16.5 8.5a5 5 0 0 1 0 7M19 6a8.5 8.5 0 0 1 0 12"/>',
  shot: '<path d="M3 8V5a2 2 0 0 1 2-2h3M16 3h3a2 2 0 0 1 2 2v3M21 16v3a2 2 0 0 1-2 2h-3M8 21H5a2 2 0 0 1-2-2v-3"/><circle cx="12" cy="12" r="3"/>',
  clip: '<rect x="6" y="4" width="12" height="17" rx="2"/><path d="M9 4V3h6v1M9 10h6M9 14h4"/>',
  picker: '<path d="M17 2l5 5-3 3-5-5zM14 7l-9 9v3h3l9-9"/>',
  lock: '<rect x="5" y="11" width="14" height="10" rx="2"/><path d="M8 11V8a4 4 0 0 1 8 0v3"/>',
};
const svg = (name, size = 22, sw = 1.8) =>
  `<svg width="${size}" height="${size}" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="${sw}" stroke-linecap="round" stroke-linejoin="round">${ICON[name]}</svg>`;
const esc = (s) => String(s).replace(/[&<>"']/g, (c) => `&#${c.charCodeAt(0)};`);

// ---------- data ----------
// Sample goals until real signal sources are wired in.
const GOALS = [
  { id: "genshin", name: "Genshin dailies", src: "HoYoLAB", detail: "3 of 4 commissions", due: "19:00", left: "48m left", short: "Genshin · 48m", pct: 75, color: "#f08c00", streak: 5, tag: "Almost there", title: "1 commission left", action: "Launch game" },
  { id: "side", name: "Side-project commit", src: "git", detail: "No commit yet", due: "22:00", left: "3h 48m left", short: "Side-project · 3h 48m", pct: 0, color: "#e03131", streak: 3, tag: "Missing proof", title: "Commit to side-project", action: "Open workspace" },
  { id: "blender", name: "Blender practice", src: "Window time", detail: "20 of 45 min", due: "23:00", left: "25m to go", short: "Blender · 25m", pct: 44, color: "#4263eb", streak: 1, tag: "In progress", title: "25 more minutes in Blender", action: "Open Blender" },
  { id: "read", name: "Read 30 min", src: "Manual", detail: "Not checked yet", due: "Any time", left: "today", short: "Read · today", pct: 0, color: "#2f9e44", streak: 12, tag: "Manual goal", title: "Read for 30 minutes", action: "Start 30m timer", manual: true },
];
// Shown only in a plain browser preview, where there's no tracker to ask.
const SAMPLE_TIMELINE = {
  focused: 15480, away: 3300,
  entries: [
    [16, 40, "Visual Studio Code", "client-x", 3900], [16, 10, "Discord", "Server", 720], [15, 2, "Genshin Impact", "", 1860],
    [13, 40, "Blender", "donut.blend", 1200], [12, 30, "Away", "idle", 3300, true], [11, 5, "Firefox", "docs.rs", 2280],
    [9, 10, "Visual Studio Code", "moondeck", 6120],
  ].map(([h, m, app, what, secs, away]) => ({ start: new Date().setHours(h, m, 0) / 1000, app, what, secs, away: !!away })),
};
const PALETTE = ["#4263eb", "#7048e8", "#f08c00", "#e64980", "#0ca678", "#1098ad", "#d6336c", "#74b816"];
const appColor = (app) => PALETTE[[...app].reduce((h, c) => (h * 31 + c.charCodeAt(0)) >>> 0, 7) % PALETTE.length];
const dur = (secs) => {
  const m = Math.round(secs / 60);
  return m >= 60 ? `${Math.floor(m / 60)}h ${String(m % 60).padStart(2, "0")}m` : `${m}m`;
};

const HISTORY = { read: [1, 1, 1, 1, 1, 1], genshin: [0, 1, 1, 1, 1, 1], side: [1, 0, 1, 1, 1, 0], blender: [0, 0, 1, 0, 0, 1] };
const CLIPS = [
  ["cargo add tauri-plugin-autostart", "Terminal · 2m ago"],
  ["#7048E8", "Color picker · 14m ago"],
  ["https://docs.rs/windows/latest/windows/", "Firefox · 40m ago"],
  ["Ask Client X about the API keys", "Quick note · 1h ago"],
];

const store = {
  get(k, d) { try { return JSON.parse(localStorage.getItem(k)) ?? d; } catch { return d; } },
  set(k, v) { try { localStorage.setItem(k, JSON.stringify(v)); } catch {} },
};

const S = {
  done: { read: true }, snoozed: {}, tab: "today", snoozeOpen: false, cmdOpen: false,
  focusUntil: null, micMuted: false, playing: false, editing: false,
  notes: store.get("notes", ["Ask Client X about the API keys"]),
  deck: store.get("deck", "launch"), pop: null, volume: 60, output: "Speakers",
  timerEnd: null, toast: "",
  timeline: tauri ? null : SAMPLE_TIMELINE,
};

// ---------- derived ----------
function goals() {
  return GOALS.map((g) => {
    const done = !!S.done[g.id];
    return { ...g, done, missing: !done && g.pct === 0, streak: g.streak + (done ? 1 : 0) };
  });
}
function queue(gs) {
  const active = gs.filter((g) => !g.done);
  return active.filter((g) => !S.snoozed[g.id]).concat(active.filter((g) => S.snoozed[g.id]));
}
const fmt = (d) => d.toTimeString().slice(0, 5);
function timer() {
  const left = S.timerEnd ? Math.max(0, S.timerEnd - Date.now()) : 0;
  const running = left > 0;
  const text = running
    ? `${Math.floor(left / 60000)}:${String(Math.floor((left % 60000) / 1000)).padStart(2, "0")}`
    : S.timerEnd ? "Time’s up" : "0:00";
  return { running, text };
}

// ---------- render ----------
function renderTop(gs, q) {
  const now = new Date();
  const doneCount = gs.filter((g) => g.done).length;
  $("top").innerHTML = `
    <span class="clock">${fmt(now)}</span>
    <span class="day">${now.toLocaleDateString(undefined, { weekday: "short" })}</span>
    <span class="segs">${gs.map((g) => `<span class="${g.done ? "on" : ""}"></span>`).join("")}</span>
    <span class="count">${doneCount} of ${gs.length}</span>
    <span class="spacer"></span>
    <span class="due">${svg("clock", 14, 2.2)}${esc(q[0] ? q[0].short : "All clear")}</span>
    <button class="search round primary" data-act="cmd" aria-label="Search and run commands">${svg("search", 18, 2.2)}</button>`;
}

const COMMANDS = [
  ["Open Side project workspace", "Workspace", "open:Side project"],
  ["Start focus · 50 min", "Focus", "focus"],
  ["Start timer · 25 min", "Timer", "timer:25"],
  ["Mark “Read 30 min” done", "Goal", "toggle:read"],
  ["Take screenshot", "Tool", "toast:Screenshot copied to clipboard"],
];
function renderCmd() {
  const el = $("cmd");
  el.hidden = !S.cmdOpen;
  if (!S.cmdOpen) return;
  el.innerHTML = `
    <label>${svg("search", 16, 2.2)}<input id="cmdq" type="text" placeholder="Open, start, add a goal, jot a note…" aria-label="Command"></label>
    <div id="cmdlist"></div>`;
  filterCmd("");
  $("cmdq").focus();
}
function filterCmd(q) {
  const hits = COMMANDS.filter(([label]) => label.toLowerCase().includes(q.toLowerCase()));
  $("cmdlist").innerHTML = hits
    .map(([label, hint, act], i) => `<button class="row${i === 0 ? " sel" : ""}" data-act="run" data-arg="${esc(act)}"><span>${esc(label)}</span><small>${hint}</small></button>`)
    .join("");
}

function tickIcon(g) {
  if (g.done) return `<span class="check">${svg("check", 13, 3)}</span>`;
  if (g.missing) return `<svg width="24" height="24" viewBox="0 0 24 24"><circle cx="12" cy="12" r="9" fill="none" style="stroke:var(--red)" stroke-width="2.4" stroke-dasharray="3 3.1"/></svg>`;
  return `<svg width="24" height="24" viewBox="0 0 24 24"><circle cx="12" cy="12" r="9" fill="none" class="ring-track" stroke-width="3"/><circle cx="12" cy="12" r="9" fill="none" stroke="${g.color}" stroke-width="3" stroke-linecap="round" stroke-dasharray="${(g.pct * 0.5655).toFixed(1)} 57" transform="rotate(-90 12 12)"/></svg>`;
}
function renderLeft(gs) {
  const tabs = [["today", "Today"], ["timeline", "Timeline"], ["streaks", "Streaks"]]
    .map(([id, l]) => `<button class="${S.tab === id ? "on" : ""}" data-act="tab" data-arg="${id}">${l}</button>`).join("");
  let body = "";
  if (S.tab === "today") {
    body = `<div class="list">${gs.map((g) => `
      <div class="goal row${g.done ? " done" : ""}">
        <button class="tick" data-act="toggle" data-arg="${g.id}" aria-label="${g.done ? "Undo" : "Mark done"}: ${esc(g.name)}">${tickIcon(g)}</button>
        <div><span class="name">${esc(g.name)}</span><span class="sub">${esc(g.done ? (g.manual ? "Checked by you" : "Proof received · " + g.src) : g.src + " · " + g.detail)}</span></div>
        <div class="meta"><span>${esc(g.due)}</span><span class="streak">${svg("flame", 12, 2.2)}${g.streak}</span></div>
      </div>`).join("")}
      <span class="spacer"></span>
      <button class="ghost" data-act="toast" data-arg="Goal editor is coming next">+ New goal</button>
      <p class="foot">Tap a circle to mark done yourself</p></div>`;
  } else if (S.tab === "timeline") {
    const tl = S.timeline;
    const max = Math.max(1, ...(tl?.entries ?? []).map((e) => e.secs));
    const rows = !tl ? `<p class="foot">Loading…</p>`
      : !tl.entries.length ? `<p class="foot">Nothing tracked yet today. Keep Moondeck running and this fills in.</p>`
      : tl.entries.map((e) => `
      <div class="tl"><time>${fmt(new Date(e.start * 1000))}</time><div><b>${esc(e.app)}${e.what ? ` <span>· ${esc(e.what)}</span>` : ""}</b><i style="width:${Math.max(4, (e.secs / max) * 100)}%;background:${e.away ? "var(--track)" : appColor(e.app)}"></i></div><em>${dur(e.secs)}</em></div>`).join("");
    body = `<div class="list">${rows}
      <span class="spacer"></span>
      <div class="total"><span>Focused today</span><b>${tl ? dur(tl.focused) : "–"}</b></div></div>`;
  } else {
    body = `<div class="list">${gs.map((g) => {
      const days = HISTORY[g.id].map((b) => `<span class="${b ? "on" : ""}"></span>`).join("") + `<span class="${g.done ? "on" : "today"}"></span>`;
      return `<div class="streakrow"><div><b>${esc(g.name)}</b><span>${g.streak} days</span></div><span class="dots">${days}</span></div>`;
    }).join("")}<p class="foot">Last 7 days · today on the right</p></div>`;
  }
  $("left").innerHTML = `<div class="tabs">${tabs}</div>${body}`;
}

function renderRight(q) {
  const c = q[0];
  let next;
  if (c) {
    const kind = c.manual ? "manual" : c.pct === 0 ? "miss" : "prog";
    next = `<div class="next">
      <span class="tag ${kind}">${esc(S.snoozed[c.id] ? "Snoozed · " + S.snoozed[c.id] : c.tag)}</span>
      <div><h2>${esc(c.title)}</h2><p>${esc(c.src)} · due ${esc(c.due)} · <b>${esc(c.left)}</b></p></div>
      <button class="pill primary" data-act="primary">${esc(c.action)}</button>
      <div class="pair">
        <button class="pill" data-act="done">Done</button>
        <button class="pill${S.snoozeOpen ? " on" : ""}" data-act="snoozemenu">Later ▾</button>
      </div>
      ${S.snoozeOpen ? `<div class="grid2">${["15 min", "1 hour", "Tonight", "Skip today"].map((l) => `<button data-act="snooze" data-arg="${l}">${l}</button>`).join("")}</div>` : ""}
    </div>`;
  } else {
    next = `<div class="clear"><h2>All clear</h2><p>Every goal has proof today.</p></div>`;
  }
  const then = q.slice(1, 3);
  $("right").innerHTML = `
    <span class="label">Up next</span>
    ${next}
    ${then.length ? `<div class="then"><small>Then</small>${then.map((g) => `<div><i style="background:${g.missing ? "var(--red)" : g.color}"></i><b>${esc(g.name)}</b><small>${esc(S.snoozed[g.id] ? "Snoozed · " + S.snoozed[g.id] : "Due " + g.due)}</small></div>`).join("")}</div>` : ""}
    <div class="notes">
      <span class="label">Quick note</span>
      ${S.notes.map((n) => `<div class="note">${esc(n)}</div>`).join("")}
      <div class="noteform"><input id="note" type="text" placeholder="Jot something…" aria-label="Quick note"><button class="primary" data-act="note" aria-label="Add note">${svg("plus", 16, 2.4)}</button></div>
    </div>`;
}

function tiles(t) {
  const focusOn = S.focusUntil && S.focusUntil > Date.now();
  const add = { label: "Add tile", icon: "plus", cls: "add", act: "toast", arg: "Pick an app, folder, URL or script to pin" };
  const pages = {
    launch: [
      { label: "Start my day", icon: "sun", cls: "soft", act: "startday" },
      { label: "Side project", icon: "folder", cls: "accent", dot: true, act: "open", arg: "Side project" },
      { label: "Client X", icon: "brief", act: "open", arg: "Client X workspace" },
      { label: "Blender", icon: "cube", act: "open", arg: "Blender · donut.blend" },
      { label: "Genshin", icon: "pad", act: "open", arg: "Genshin Impact" },
      add,
    ],
    controls: [
      { label: focusOn ? "Until " + fmt(new Date(S.focusUntil)) : "Focus 50m", icon: "target", cls: focusOn ? "accent" : "soft", act: "focus" },
      { label: t.running ? t.text : "Timer", icon: "timer", cls: S.pop === "timer" ? "active" : t.running ? "soft" : "", act: "pop", arg: "timer" },
      { label: S.micMuted ? "Muted" : "Mic on", icon: S.micMuted ? "micoff" : "mic", cls: S.micMuted ? "warn" : "", act: "mic" },
      { label: S.playing ? "Pause" : "Play", icon: S.playing ? "pause" : "play", act: "play" },
      { label: `Vol ${S.volume}%`, icon: "vol", cls: S.pop === "volume" ? "active" : "", act: "pop", arg: "volume" },
    ],
    tools: [
      { label: "Screenshot", icon: "shot", act: "toast", arg: "Screenshot copied to clipboard" },
      { label: "Clipboard", icon: "clip", cls: S.pop === "clip" ? "active" : "", act: "pop", arg: "clip" },
      { label: "Pick color", icon: "picker", act: "toast", arg: "#7048E8 copied" },
      { label: "Lock PC", icon: "lock", act: "toast", arg: "Would lock Windows (Win + L)" },
      add,
    ],
  };
  return pages[S.deck];
}
function renderDeck(t) {
  const pages = [["launch", "Launch"], ["controls", "Controls"], ["tools", "Tools"]]
    .map(([id, l]) => `<button class="${S.deck === id ? "on" : ""}" data-act="deck" data-arg="${id}">${l}</button>`).join("");
  const el = $("deck");
  el.classList.toggle("editing", S.editing);
  el.innerHTML = `
    <div class="pages">${pages}</div>
    <div class="tiles">${tiles(t).map((x) => `
      <button class="tile ${x.cls || ""}" data-act="${x.act}" data-arg="${esc(x.arg || "")}" aria-label="${esc(x.label)}">
        <span class="ico">${svg(x.icon)}</span>${x.dot ? '<span class="dot"></span>' : ""}${esc(x.label)}
      </button>`).join("")}</div>
    <button class="editbtn round${S.editing ? " on" : ""}" data-act="edit" aria-label="Edit deck">${svg("pencil", 18, 2)}</button>`;
}

function renderPop(t) {
  const el = $("pop");
  el.hidden = !S.pop;
  if (S.pop === "volume") {
    el.innerHTML = `<header>Volume <span id="volval">${S.volume}%</span></header>
      <input id="vol" type="range" min="0" max="100" value="${S.volume}" aria-label="Volume">
      ${["Speakers", "Headphones"].map((n) => `<button class="out row${S.output === n ? " sel" : ""}" data-act="output" data-arg="${n}"><span>${n}</span><small>${S.output === n ? "Active" : ""}</small></button>`).join("")}`;
  } else if (S.pop === "timer") {
    el.innerHTML = `<header>Timer <span id="timertext">${t.text}</span></header>
      <div class="grid4">${[5, 15, 25, 50].map((m) => `<button data-act="timer" data-arg="${m}">${m}m</button>`).join("")}</div>
      ${t.running ? `<button class="pill primary" data-act="timerstop">Stop timer</button>` : ""}
      <p>Timers count as proof for manual goals like “Read 30 min”.</p>`;
  } else if (S.pop === "clip") {
    el.innerHTML = `<header>Clipboard</header>${CLIPS.map(([text, meta]) => `<button class="clip row" data-act="toast" data-arg="Copied again: ${esc(text.length > 28 ? text.slice(0, 28) + "…" : text)}"><b>${esc(text)}</b><small>${meta}</small></button>`).join("")}`;
  }
}

function renderToast() {
  const el = $("toast");
  el.hidden = !S.toast || !!S.pop;
  el.innerHTML = `${svg("check", 14, 3)}${esc(S.toast)}`;
}

function render() {
  const gs = goals();
  const q = queue(gs);
  const t = timer();
  renderTop(gs, q);
  renderCmd();
  renderLeft(gs);
  renderRight(q);
  renderDeck(t);
  renderPop(t);
  renderToast();
  $("settings").innerHTML = svg("sliders", 18, 2);
  $("hint").innerHTML = S.editing
    ? "Editing deck · drag tiles to reorder, click one to change it"
    : "<kbd>Alt + Space</kbd> opens · <kbd>Esc</kbd> back to tray";
}

// ---------- actions ----------
let toastTimer;
function toast(text) {
  S.toast = text;
  S.pop = null;
  clearTimeout(toastTimer);
  toastTimer = setTimeout(() => { S.toast = ""; renderToast(); }, 2600);
}
function addNote() {
  const input = $("note");
  const text = input.value.trim();
  if (!text) return;
  S.notes = [text, ...S.notes].slice(0, 3);
  store.set("notes", S.notes);
}
// ---------- open / close ----------
const EXIT_MS = 200;
let closing = false;
function enter() {
  closing = false;
  const body = document.body;
  if (!body.classList.contains("out")) return;
  // Force a style flush so the "out" state is committed before transitioning in.
  void body.offsetWidth;
  body.classList.remove("out");
}
function hide() {
  if (closing) return;
  closing = true;
  document.body.classList.add("out");
  setTimeout(() => invoke("hide_overlay"), EXIT_MS);
}

const ACTIONS = {
  cmd: () => (S.cmdOpen = !S.cmdOpen),
  run: (arg) => { S.cmdOpen = false; const [a, ...rest] = arg.split(":"); ACTIONS[a]?.(rest.join(":")); },
  tab: (id) => { S.tab = id; if (id === "timeline") loadTimeline(); },
  toggle: (id) => (S.done[id] = !S.done[id]),
  done: () => { const c = queue(goals())[0]; if (c) S.done[c.id] = true; S.snoozeOpen = false; },
  primary: () => { const c = queue(goals())[0]; if (c) toast(c.action + "…"); },
  snoozemenu: () => (S.snoozeOpen = !S.snoozeOpen),
  snooze: (label) => { const c = queue(goals())[0]; if (c) S.snoozed[c.id] = label; S.snoozeOpen = false; },
  note: addNote,
  deck: (id) => { S.deck = id; S.pop = null; store.set("deck", id); },
  edit: () => { S.editing = !S.editing; S.pop = null; },
  pop: (key) => { S.pop = S.pop === key ? null : key; S.toast = ""; },
  open: (name) => toast("Opened " + name),
  startday: () => { S.focusUntil = Date.now() + 50 * 60000; toast("Day started · workspace open · focus 50m on"); },
  focus: () => (S.focusUntil = S.focusUntil && S.focusUntil > Date.now() ? null : Date.now() + 50 * 60000),
  mic: () => (S.micMuted = !S.micMuted),
  play: () => (S.playing = !S.playing),
  output: (n) => (S.output = n),
  timer: (m) => (S.timerEnd = Date.now() + Number(m) * 60000),
  timerstop: () => (S.timerEnd = null),
  toast: (text) => toast(text),
};

document.addEventListener("click", (e) => {
  const el = e.target.closest("[data-act]");
  if (el) {
    ACTIONS[el.dataset.act]?.(el.dataset.arg);
    render();
    return;
  }
  // Clicking empty space closes whatever is open, then the overlay itself.
  if (e.target.id === "stage") {
    if (S.pop || S.cmdOpen || S.snoozeOpen) { S.pop = null; S.cmdOpen = false; S.snoozeOpen = false; render(); }
    else hide();
  }
});

document.addEventListener("keydown", (e) => {
  if (e.key === "Escape") {
    if (S.pop || S.cmdOpen || S.snoozeOpen) { S.pop = null; S.cmdOpen = false; S.snoozeOpen = false; render(); }
    else hide();
  } else if (e.key === "Enter" && e.target.id === "note") {
    addNote(); render(); $("note").focus();
  } else if (e.key === "Enter" && e.target.id === "cmdq") {
    $("cmdlist").querySelector(".row")?.click();
  }
});

document.addEventListener("input", (e) => {
  if (e.target.id === "vol") {
    S.volume = Number(e.target.value);
    $("volval").textContent = S.volume + "%";
  } else if (e.target.id === "cmdq") {
    filterCmd(e.target.value);
  }
});
document.addEventListener("change", (e) => { if (e.target.id === "vol") renderDeck(timer()); });

// Clock + timer: cheap partial updates once a second.
setInterval(() => {
  const gs = goals();
  renderTop(gs, queue(gs));
  if (S.timerEnd) {
    const t = timer();
    if (S.pop === "timer") $("timertext").textContent = t.text;
    else renderDeck(t);
  }
}, 1000);

// Accent follows the Windows accent color.
async function syncAccent() {
  const hex = await invoke("accent_color").catch(() => null);
  if (!hex) return;
  const [r, g, b] = [1, 3, 5].map((i) => parseInt(hex.slice(i, i + 2), 16));
  const light = (0.299 * r + 0.587 * g + 0.114 * b) / 255 > 0.5;
  document.documentElement.style.setProperty("--accent", hex);
  document.documentElement.style.setProperty("--accent-ink", light ? "#111" : "#fff");
}

async function loadTimeline() {
  if (!tauri) return;
  const tl = await invoke("timeline").catch(() => null);
  if (!tl) return;
  S.timeline = tl;
  if (S.tab === "timeline") renderLeft(goals());
}

// Fresh state every time the overlay is summoned.
tauri?.event?.listen("overlay:shown", () => {
  syncAccent();
  loadTimeline();
  S.pop = null; S.cmdOpen = false; S.snoozeOpen = false;
  render();
  enter();
});
tauri?.event?.listen("overlay:hide", hide);

render();
syncAccent();
loadTimeline();
enter();
