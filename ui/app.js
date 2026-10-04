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
  code: '<path d="M8 7l-5 5 5 5M16 7l5 5-5 5M14 4l-4 16"/>',
  chat: '<path d="M4 5h16v11H9l-5 4z"/>',
  music: '<path d="M9 18V5l11-2v13"/><circle cx="6" cy="18" r="3"/><circle cx="17" cy="16" r="3"/>',
  next: '<path d="M5 5v14l10-7z" fill="currentColor"/><path d="M19 5v14" stroke-width="2.4"/>',
  playpause: '<path d="M4 5v14l9-7z" fill="currentColor"/><path d="M16 5v14M20 5v14" stroke-width="2.2"/>',
  smile: '<circle cx="12" cy="12" r="9"/><path d="M8.5 14.5a4.5 4.5 0 0 0 7 0M9 9.5h.01M15 9.5h.01" stroke-width="2.2"/>',
  globe: '<circle cx="12" cy="12" r="9"/><path d="M3 12h18M12 3a14 14 0 0 1 0 18M12 3a14 14 0 0 0 0 18"/>',
  grip: '<path d="M9 6h.01M15 6h.01M9 12h.01M15 12h.01M9 18h.01M15 18h.01" stroke-width="3"/>',
  left: '<path d="M15 6l-6 6 6 6"/>',
  right: '<path d="M9 6l6 6-6 6"/>',
  trash: '<path d="M4 7h16M9 7V4h6v3M6 7l1 13h10l1-13"/>',
  pin: '<path d="M9 4h6l-1 6 3 3H7l3-3zM12 13v7"/>',
  lock: '<rect x="5" y="11" width="14" height="10" rx="2"/><path d="M8 11V8a4 4 0 0 1 8 0v3"/>',
};
const svg = (name, size = 22, sw = 1.8) =>
  `<svg width="${size}" height="${size}" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="${sw}" stroke-linecap="round" stroke-linejoin="round">${ICON[name]}</svg>`;
const esc = (s) => String(s).replace(/[&<>"']/g, (c) => `&#${c.charCodeAt(0)};`);

// ---------- data ----------
// Shown only in a plain browser preview, where there's no engine to ask.
const SAMPLE_TASKS = [
  { id: 1, created: Date.now() / 1000 - 3600, source: "notification", app: "Teams", title: "Send the Q4 slides to Priya", detail: "Priya · via Teams", due: Date.now() / 1000 + 2 * 3600 },
  { id: 2, created: Date.now() / 1000 - 7200, source: "notification", app: "WhatsApp", title: "Pay rent", detail: "Landlord · via WhatsApp", due: null },
];
const SAMPLE_GOALS = [
  { id: "commit", name: "Commit to moondeck", source: "git", detail: "No commit yet", due: "22:00", due_in: 3 * 3600 + 48 * 60, pct: 0, streak: 3, history: [true, false, true, true, true, false], action: { label: "Open project", open: [] } },
  { id: "code", name: "Code for 2 hours", source: "Window time", detail: "52 of 120 min", due: "23:00", due_in: 4 * 3600, pct: 43, streak: 1, history: [false, false, true, null, null, true] },
  { id: "read", name: "Read 30 min", source: "Manual", detail: "Checked by you", due: null, due_in: null, pct: 100, done: true, by_hand: true, manual: true, streak: 12, history: [true, true, true, true, true, true] },
].map((g) => ({ done: false, by_hand: false, skipped: false, manual: false, scheduled: true, snoozed: null, action: null, ...g }));
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

// Used in a plain browser preview; the app reads deck.json.
const SAMPLE_DECK = {
  pages: [
    { name: "Launch", tiles: [
      { label: "Start my day", icon: "sun", run: { type: "routine", targets: [], focus: 50 } },
      { label: "VS Code", icon: "code", run: { type: "open", targets: ["Visual Studio Code"] } },
      { label: "Discord", icon: "chat", run: { type: "open", targets: ["Discord"] } },
      { label: "Spotify", icon: "music", run: { type: "open", targets: ["Spotify"] } },
      { label: "Docs", icon: "globe", run: { type: "open", targets: ["https://docs.rs"] } },
    ] },
    { name: "Controls", tiles: ["focus:target:Focus", "timer:timer:Timer", "mic:mic:Mic", "playpause:playpause:Play/Pause", "next:next:Next", "volume:vol:Volume"]
      .map((x) => { const [action, icon, label] = x.split(":"); return { label, icon, run: { type: "builtin", action } }; }) },
    { name: "Tools", tiles: ["snip:shot:Snip", "clipboard:clip:Clipboard", "emoji:smile:Emoji", "lock:lock:Lock PC"]
      .map((x) => { const [action, icon, label] = x.split(":"); return { label, icon, run: { type: "builtin", action } }; }) },
  ],
};
const BUILTINS = [
  ["focus", "target", "Focus"], ["timer", "timer", "Timer"], ["mic", "mic", "Mic"], ["playpause", "playpause", "Play/Pause"],
  ["next", "next", "Next"], ["volume", "vol", "Volume"], ["snip", "shot", "Snip"], ["clipboard", "clip", "Clipboard"],
  ["emoji", "smile", "Emoji"], ["lock", "lock", "Lock PC"],
];

const store = {
  get(k, d) { try { return JSON.parse(localStorage.getItem(k)) ?? d; } catch { return d; } },
  set(k, v) { try { localStorage.setItem(k, JSON.stringify(v)); } catch {} },
};

const S = {
  goals: tauri ? null : SAMPLE_GOALS, tasks: tauri ? null : SAMPLE_TASKS, tab: "today", snoozeOpen: false, cmdOpen: false,
  focusUntil: store.get("focusUntil", null), micMuted: false,
  notes: store.get("notes", ["Ask Client X about the API keys"]),
  page: Number(store.get("page", 0)) || 0, pop: null, volume: 50,
  editDeck: false, editTile: null, addKind: "apps", apps: null, appQuery: "", icons: {},
  deckCfg: tauri ? null : SAMPLE_DECK,
  timerEnd: store.get("timerEnd", null), toast: "",
  timeline: tauri ? null : SAMPLE_TIMELINE,
  sheet: null,
};

// ---------- derived ----------
const GOAL_COLORS = { git: "#7048e8", "Window time": "#4263eb", Manual: "#2f9e44" };
function left(secs) {
  if (secs == null) return "today";
  if (secs < 0) return "overdue";
  const m = Math.round(secs / 60);
  return m >= 60 ? `${Math.floor(m / 60)}h ${m % 60}m left` : `${m}m left`;
}
function goals() {
  return (S.goals ?? []).filter((g) => g.scheduled).map((g) => {
    const missing = !g.done && g.pct === 0 && !g.manual;
    return {
      ...g,
      missing,
      color: GOAL_COLORS[g.source] ?? "var(--accent)",
      dueText: g.due ?? "Any time",
      left: left(g.due_in),
      kind: g.manual ? "manual" : missing ? "miss" : "prog",
      tag: g.snoozed ? "Snoozed · " + g.snoozed : g.manual ? "Your call" : missing ? "Missing proof" : "In progress",
      sub: g.skipped ? "Skipped today" : g.done ? (g.by_hand ? "Checked by you" : `Proof · ${g.source} · ${g.detail}`) : `${g.source} · ${g.detail}`,
    };
  });
}
function tasks() {
  const now = Date.now() / 1000;
  return (S.tasks ?? []).map((t) => {
    const due_in = t.due == null ? null : t.due - now;
    return {
      ...t,
      isTask: true,
      name: t.title,
      due_in,
      due: t.due == null ? null : fmt(new Date(t.due * 1000)),
      dueText: t.due == null ? "No date" : whenText(t.due),
      left: left(due_in),
      kind: due_in != null && due_in < 0 ? "miss" : "prog",
      tag: due_in != null && due_in < 0 ? "Overdue" : "From " + t.app,
      color: "var(--accent)",
      sub: t.detail,
      source: t.app,
      detail: t.detail,
    };
  });
}
// "Today 17:00", "Tomorrow 09:30", "Fri 9 Oct 23:59"
function whenText(ts) {
  const d = new Date(ts * 1000);
  const days = Math.round((new Date(d).setHours(0, 0, 0, 0) - new Date().setHours(0, 0, 0, 0)) / 86400000);
  const day = days === 0 ? "Today" : days === 1 ? "Tomorrow" : d.toLocaleDateString(undefined, { weekday: "short", day: "numeric", month: "short" });
  return `${day} ${fmt(d)}`;
}
function queue(gs) {
  const byDue = (a, b) => (a.due_in ?? 1e9) - (b.due_in ?? 1e9);
  const active = gs.filter((g) => !g.done && !g.skipped).concat(tasks().filter((t) => t.due_in != null));
  return active.filter((g) => !g.snoozed).sort(byDue).concat(active.filter((g) => g.snoozed).sort(byDue));
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
  const next = q[0] ? `${q[0].name} · ${q[0].left.replace(" left", "")}` : gs.length ? "All clear" : "No goals yet";
  $("top").innerHTML = `
    <span class="clock">${fmt(now)}</span>
    <span class="day">${now.toLocaleDateString(undefined, { weekday: "short" })}</span>
    <span class="segs">${gs.map((g) => `<span class="${g.done ? "on" : ""}"></span>`).join("")}</span>
    <span class="count">${doneCount} of ${gs.length}</span>
    <span class="spacer"></span>
    <span class="due">${svg("clock", 14, 2.2)}${esc(next)}</span>
    <button class="search round primary" data-act="cmd" aria-label="Search and run commands">${svg("search", 18, 2.2)}</button>`;
}

// Commands are built from whatever is live: goals, deck tiles, tools.
function commands() {
  const gs = goals();
  return [
    ...(S.deckCfg?.pages ?? []).flatMap((p, pi) => p.tiles
      .filter((t) => t.run?.type !== "builtin")
      .map((t) => [t.run?.type === "routine" ? t.label : `Open ${t.label}`, p.name, `tile:${pi}:${p.tiles.indexOf(t)}`])),
    ...gs.filter((g) => !g.done).map((g) => [`Mark “${g.name}” done`, "Goal", `toggle:${g.id}`]),
    [`Start focus · ${focusMinutes()} min`, "Focus", "focus"],
    ...[5, 15, 25, 50].map((m) => [`Start timer · ${m} min`, "Timer", `timer:${m}`]),
    ["Snip a screenshot", "Tool", "shell:screenshot"],
    ["Clipboard history", "Tool", "shell:clipboard"],
    ["Lock PC", "Tool", "shell:lock"],
    ["Pin Up next widget", "Widget", "pin:next"],
    ["Pin Timer widget", "Widget", "pin:timer"],
    ["Pin Notes widget", "Widget", "pin:notes"],
    ["Edit goals", "Settings", "editgoals"],
    ["Edit deck", "Settings", "edit"],
  ];
}
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
  const hits = commands().filter(([label]) => label.toLowerCase().includes(q.toLowerCase()));
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
  const inboxCount = S.tasks?.length ? ` <small class="badge">${S.tasks.length}</small>` : "";
  const tabs = [["today", "Today"], ["inbox", "Inbox" + inboxCount], ["timeline", "Timeline"], ["streaks", "Streaks"]]
    .map(([id, l]) => `<button class="${S.tab === id ? "on" : ""}" data-act="tab" data-arg="${id}">${l}</button>`).join("");
  let body = "";
  if (S.tab === "today") {
    const empty = !S.goals ? `<p class="foot">Loading…</p>` : !gs.length ? `<p class="foot">No goals for today.</p>` : "";
    body = `<div class="list">${empty}${gs.map((g) => `
      <div class="goal row${g.done ? " done" : ""}">
        <button class="tick" data-act="toggle" data-arg="${g.id}" aria-label="${g.done ? "Undo" : "Mark done"}: ${esc(g.name)}">${tickIcon(g)}</button>
        <div><span class="name">${esc(g.name)}</span><span class="sub">${esc(g.sub)}</span></div>
        <div class="meta"><span>${esc(g.dueText)}</span><span class="streak">${svg("flame", 12, 2.2)}${g.streak}</span></div>
      </div>`).join("")}
      <span class="spacer"></span>
      <button class="ghost" data-act="editgoals">+ New goal</button>
      <p class="foot">Tap a circle to mark done yourself</p></div>`;
  } else if (S.tab === "inbox") {
    const ts = tasks();
    const rows = !S.tasks ? `<p class="foot">Loading…</p>`
      : !ts.length ? `<p class="foot">Nothing detected yet. Tasks and deadlines from your notifications and email show up here.</p>
        <button class="ghost" data-act="settings">Choose sources</button>`
      : ts.map((t) => `
      <div class="goal row">
        <button class="tick" data-act="taskdone" data-arg="${t.id}" aria-label="Done: ${esc(t.title)}"><svg width="24" height="24" viewBox="0 0 24 24"><circle cx="12" cy="12" r="9" fill="none" class="ring-track" stroke-width="2.4"/></svg></button>
        <div><span class="name">${esc(t.title)}</span><span class="sub">${esc(t.detail)}</span></div>
        <div class="meta"><span class="${t.due_in != null && t.due_in < 0 ? "late" : ""}">${esc(t.dueText)}</span><button class="x" data-act="taskdismiss" data-arg="${t.id}" aria-label="Dismiss: ${esc(t.title)}">Dismiss</button></div>
      </div>`).join("");
    body = `<div class="list">${rows}<span class="spacer"></span><p class="foot">Found by local rules${S.aiOn ? " + AI" : ""} · nothing leaves your PC${S.aiOn ? " except to your AI provider" : ""}</p></div>`;
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
      const days = g.history.map((b) => `<span class="${b ? "on" : b === null ? "rest" : ""}"></span>`).join("") + `<span class="${g.done ? "on" : "today"}"></span>`;
      return `<div class="streakrow"><div><b>${esc(g.name)}</b><span>${g.streak} ${g.streak === 1 ? "day" : "days"}</span></div><span class="dots">${days}</span></div>`;
    }).join("")}<p class="foot">Last 7 days · today on the right</p></div>`;
  }
  $("left").innerHTML = `<div class="tabs">${tabs}</div>${body}`;
}

function renderRight(q) {
  const c = q[0];
  let next;
  if (c) {
    const action = c.isTask ? null : c.action?.label ?? (c.manual ? "Mark done" : null);
    next = `<div class="next">
      <span class="tag ${c.kind}">${esc(c.tag)}</span>
      <div><h2>${esc(c.name)}</h2><p>${esc(c.isTask ? c.detail : c.source + " · " + c.detail)} · <b>${esc(c.due ? c.left + " · " + c.due : "today")}</b></p></div>
      ${action ? `<button class="pill primary" data-act="primary">${esc(action)}</button>` : ""}
      <div class="pair">
        <button class="pill" data-act="done">Done</button>
        ${c.isTask ? `<button class="pill" data-act="taskdismiss" data-arg="${c.id}">Dismiss</button>` : `<button class="pill${S.snoozeOpen ? " on" : ""}" data-act="snoozemenu">Later ▾</button>`}
      </div>
      ${S.snoozeOpen && !c.isTask ? `<div class="grid2">${["15 min", "1 hour", "Tonight", "Skip today"].map((l) => `<button data-act="snooze" data-arg="${l}">${l}</button>`).join("")}</div>` : ""}
    </div>`;
  } else {
    next = `<div class="clear"><h2>All clear</h2><p>${S.goals?.length ? "Every goal has proof today." : "Add a goal to get started."}</p></div>`;
  }
  const then = q.slice(1, 3);
  $("right").innerHTML = `
    <div class="labelrow"><span class="label">Up next</span><button class="pinbtn" data-act="pin" data-arg="next" aria-label="Pin Up next as a widget">${svg("pin", 14, 2)}</button></div>
    ${next}
    ${then.length ? `<div class="then"><small>Then</small>${then.map((g) => `<div><i style="background:${g.missing ? "var(--red)" : g.color}"></i><b>${esc(g.name)}</b><small>${esc(g.snoozed ? "Snoozed · " + g.snoozed : g.isTask ? g.dueText : g.due ? "Due " + g.due : "Today")}</small></div>`).join("")}</div>` : ""}
    <div class="notes">
      <div class="labelrow"><span class="label">Quick note</span><button class="pinbtn" data-act="pin" data-arg="notes" aria-label="Pin notes as a widget">${svg("pin", 14, 2)}</button></div>
      ${S.notes.slice(0, 3).map((n) => `<div class="note">${esc(n)}</div>`).join("")}
      <div class="noteform"><input id="note" type="text" placeholder="Jot something…" aria-label="Quick note"><button class="primary" data-act="note" aria-label="Add note">${svg("plus", 16, 2.4)}</button></div>
    </div>`;
}

// ---------- deck ----------
const page = () => S.deckCfg?.pages?.[S.page] ?? S.deckCfg?.pages?.[0];
const focusMinutes = () => {
  for (const p of S.deckCfg?.pages ?? []) for (const t of p.tiles) if (t.run?.type === "routine" && t.run.focus) return t.run.focus;
  return 50;
};
const isUrl = (x) => /^[a-z]+:\/\//i.test(x ?? "");

function loadIcon(target) {
  if (!tauri || !target || isUrl(target) || target in S.icons) return;
  S.icons[target] = null;
  invoke("app_icon", { target }).then((url) => { if (url) { S.icons[target] = url; renderDeck(timer()); renderPop(timer()); } });
}

function iconHtml(tile) {
  const first = tile.run?.targets?.[0];
  if (tile.icon === "app" || !ICON[tile.icon]) {
    if (isUrl(first)) return svg("globe");
    loadIcon(first);
    const img = S.icons[first];
    if (img) return `<img src="${img}" alt="">`;
    if (!ICON[tile.icon]) return `<b class="letter">${esc((tile.label || "?").slice(0, 1).toUpperCase())}</b>`;
  }
  return svg(tile.icon);
}

/// Live look of a tile: builtins reflect state (mic muted, timer running...).
function tileView(tile, t) {
  const v = { label: tile.label, cls: "", icon: iconHtml(tile) };
  const a = tile.run?.type === "builtin" ? tile.run.action : null;
  const focusOn = S.focusUntil && S.focusUntil > Date.now();
  if (a === "focus") { v.label = focusOn ? "Until " + fmt(new Date(S.focusUntil)) : `Focus ${focusMinutes()}m`; v.cls = focusOn ? "accent" : "soft"; }
  if (a === "timer") { v.label = t.running ? t.text : tile.label; v.cls = S.pop === "timer" ? "active" : t.running ? "soft" : ""; }
  if (a === "mic") { v.label = S.micMuted ? "Muted" : "Mic on"; v.icon = svg(S.micMuted ? "micoff" : "mic"); v.cls = S.micMuted ? "warn" : ""; }
  if (a === "volume") { v.label = `Vol ${S.volume}%`; v.cls = S.pop === "volume" ? "active" : ""; }
  if (tile.run?.type === "routine") v.cls = "soft";
  if (S.editDeck && S.pop === "tile" && S.editTile === tile) v.cls += " active";
  return v;
}

function renderDeck(t) {
  const el = $("deck");
  const cfg = S.deckCfg;
  if (!cfg) { el.innerHTML = `<p class="foot" style="flex:1">Loading deck…</p>`; return; }
  if (S.page >= cfg.pages.length) S.page = 0;
  el.classList.toggle("editing", S.editDeck);
  const pages = cfg.pages.map((p, i) => `<button class="${S.page === i ? "on" : ""}" data-act="page" data-arg="${i}">${esc(p.name)}</button>`).join("")
    + (S.editDeck ? `<button data-act="addpage" aria-label="Add page">+ Page</button>` : "");
  const ts = page()?.tiles ?? [];
  el.innerHTML = `
    <div class="pages">${pages}</div>
    <div class="tiles" id="tiles">${ts.map((tile, i) => {
      const v = tileView(tile, t);
      return `<button class="tile ${v.cls}" data-act="tile" data-arg="${S.page}:${i}" data-i="${i}" ${S.editDeck ? 'draggable="true"' : ""} aria-label="${esc(v.label)}">
        ${i < 9 && !S.editDeck ? `<kbd class="num">${i + 1}</kbd>` : ""}<span class="ico">${v.icon}</span>${esc(v.label)}
      </button>`;
    }).join("")}${S.editDeck ? `<button class="tile add${S.pop === "add" ? " active" : ""}" data-act="addtile" aria-label="Add a tile"><span class="ico">${svg("plus")}</span>Add</button>` : ""}</div>
    <button class="editbtn round${S.editDeck ? " on" : ""}" data-act="editdeck" aria-label="${S.editDeck ? "Done editing" : "Edit deck"}">${svg(S.editDeck ? "check" : "pencil", 18, 2)}</button>`;
}

function saveDeck() {
  if (tauri) invoke("save_deck", { deck: S.deckCfg }).catch((e) => toast("Couldn't save deck: " + e));
}
function addTile(tile) {
  page().tiles.push(tile);
  saveDeck();
  toast(`Added ${tile.label}`);
}

/// Run a tile: open things, run a routine, or a built-in control.
function runTile(tile) {
  const run = tile.run ?? {};
  if (run.type === "open") {
    invoke("open_targets", { targets: run.targets ?? [] }).then(() => toast("Opening " + tile.label), (e) => toast("Couldn't open " + e));
  } else if (run.type === "routine") {
    if (run.targets?.length) invoke("open_targets", { targets: run.targets }).catch((e) => toast("Couldn't open " + e));
    if (run.focus) S.focusUntil = Date.now() + run.focus * 60000;
    toast(`${tile.label} · ${run.targets?.length ?? 0} opened${run.focus ? ` · focus ${run.focus}m` : ""}`);
  } else if (run.type === "builtin") {
    const a = run.action;
    if (a === "timer" || a === "volume") ACTIONS.pop(a);
    else if (a === "playpause" || a === "next" || a === "prev") ACTIONS.media(a);
    else if (a === "snip") ACTIONS.shell("screenshot");
    else if (a === "clipboard" || a === "emoji" || a === "lock") ACTIONS.shell(a);
    else ACTIONS[a]?.();
  }
}

// Drag to reorder while editing.
let dragFrom = null;
document.addEventListener("dragstart", (e) => {
  const tile = e.target.closest?.(".tile[data-i]");
  if (!tile || !S.editDeck) return;
  dragFrom = Number(tile.dataset.i);
  e.dataTransfer.effectAllowed = "move";
});
document.addEventListener("dragover", (e) => {
  if (dragFrom != null && e.target.closest?.("#tiles")) e.preventDefault();
});
document.addEventListener("drop", (e) => {
  const over = e.target.closest?.(".tile[data-i]");
  if (dragFrom == null || !over) return;
  e.preventDefault();
  const tiles = page().tiles;
  const [moved] = tiles.splice(dragFrom, 1);
  tiles.splice(Number(over.dataset.i), 0, moved);
  dragFrom = null;
  saveDeck();
  renderDeck(timer());
});
document.addEventListener("dragend", () => (dragFrom = null));

function renderAddPop(el) {
  const kinds = [["apps", "App"], ["web", "Website"], ["path", "File / folder"], ["routine", "Routine"], ["builtin", "Control"]];
  let body = "";
  if (S.addKind === "apps") {
    if (!S.apps) { if (tauri) invoke("list_apps").then((a) => { S.apps = a; renderPop(timer()); }); else S.apps = ["Visual Studio Code", "Discord", "Spotify", "Firefox", "Notepad", "Steam", "OBS Studio"]; }
    const q = S.appQuery.toLowerCase();
    const hits = (S.apps ?? []).filter((a) => a.toLowerCase().includes(q)).slice(0, 7);
    hits.forEach(loadIcon);
    body = `<label class="field"><input id="appq" value="${esc(S.appQuery)}" placeholder="Search installed apps" autocomplete="off"></label>
      <div class="picklist">${!S.apps ? `<p class="foot">Loading apps…</p>` : hits.map((a) => `<button class="row" data-act="addapp" data-arg="${esc(a)}"><span class="ico sm">${S.icons[a] ? `<img src="${S.icons[a]}" alt="">` : svg("folder", 16)}</span>${esc(a)}</button>`).join("") || `<p class="foot">No match</p>`}</div>`;
  } else if (S.addKind === "web") {
    body = `<label class="field">Label<input id="add-label" placeholder="GitHub"></label>
      <label class="field">URL<input id="add-target" placeholder="https://github.com"></label>
      <button class="pill primary" data-act="addcustom" data-arg="web">Add website</button>`;
  } else if (S.addKind === "path") {
    body = `<label class="field">Label<input id="add-label" placeholder="Client X"></label>
      <label class="field">Path<input id="add-target" placeholder="C:\\Users\\you\\Projects\\client-x"></label>
      <button class="pill primary" data-act="addcustom" data-arg="path">Add</button>`;
  } else if (S.addKind === "routine") {
    body = `<label class="field">Label<input id="add-label" placeholder="Start my day"></label>
      <label class="field">Open, one per line (apps, folders, URLs)<textarea id="add-target" rows="3" placeholder="Visual Studio Code&#10;Spotify&#10;https://mail.google.com"></textarea></label>
      <label class="field">Focus minutes (0 for none)<input id="add-focus" type="number" min="0" value="50"></label>
      <button class="pill primary" data-act="addcustom" data-arg="routine">Add routine</button>`;
  } else {
    body = `<div class="picklist grid">${BUILTINS.map(([a, icon, label]) => `<button class="row" data-act="addbuiltin" data-arg="${a}"><span class="ico sm">${svg(icon, 16)}</span>${label}</button>`).join("")}</div>`;
  }
  el.innerHTML = `<header>Add a tile <small>to ${esc(page()?.name ?? "")}</small></header>
    <div class="tabs small">${kinds.map(([k, l]) => `<button class="${S.addKind === k ? "on" : ""}" data-act="addkind" data-arg="${k}">${l}</button>`).join("")}</div>${body}`;
  $("appq")?.focus();
}

function renderTilePop(el) {
  const tile = S.editTile;
  const i = page().tiles.indexOf(tile);
  const targets = tile.run?.targets;
  el.innerHTML = `<header>Edit tile</header>
    <label class="field">Label<input id="edit-label" value="${esc(tile.label)}"></label>
    ${targets ? `<label class="field">Opens, one per line<textarea id="edit-targets" rows="3">${esc(targets.join("\n"))}</textarea></label>` : ""}
    ${tile.run?.type === "routine" ? `<label class="field">Focus minutes<input id="edit-focus" type="number" min="0" value="${tile.run.focus ?? 0}"></label>` : ""}
    <div class="pair">
      <button class="pill" data-act="movetile" data-arg="-1" ${i === 0 ? "disabled" : ""} aria-label="Move left">${svg("left", 16, 2.2)}</button>
      <button class="pill" data-act="movetile" data-arg="1" ${i === page().tiles.length - 1 ? "disabled" : ""} aria-label="Move right">${svg("right", 16, 2.2)}</button>
      <button class="pill danger" data-act="removetile">${svg("trash", 16, 2)} Remove</button>
    </div>
    <button class="pill primary" data-act="savetile">Save</button>`;
}

function renderPagePop(el) {
  const p = page();
  el.innerHTML = `<header>Edit page</header>
    <label class="field">Name<input id="page-name" value="${esc(p.name)}"></label>
    <div class="pair">
      <button class="pill" data-act="movepage" data-arg="-1" ${S.page === 0 ? "disabled" : ""}>Move up</button>
      <button class="pill" data-act="movepage" data-arg="1" ${S.page === S.deckCfg.pages.length - 1 ? "disabled" : ""}>Move down</button>
    </div>
    <div class="pair">
      <button class="pill danger" data-act="removepage" ${S.deckCfg.pages.length < 2 ? "disabled" : ""}>${svg("trash", 16, 2)} Delete page</button>
      <button class="pill primary" data-act="savepage">Save</button>
    </div>`;
}

function renderPop(t) {
  const el = $("pop");
  el.hidden = !S.pop;
  el.classList.toggle("wide", S.pop === "add");
  if (S.pop === "add") return renderAddPop(el);
  if (S.pop === "tile" && S.editTile) return renderTilePop(el);
  if (S.pop === "page") return renderPagePop(el);
  if (S.pop === "volume") {
    el.innerHTML = `<header>Volume <span id="volval">${S.volume}%</span></header>
      <input id="vol" type="range" min="0" max="100" value="${S.volume}" aria-label="Volume">
      <p>Default output device</p>`;
  } else if (S.pop === "timer") {
    el.innerHTML = `<header>Timer <span id="timertext">${t.text}</span></header>
      <button class="pinbtn inpop" data-act="pin" data-arg="timer">${svg("pin", 14, 2)} Pin as widget</button>
      <div class="grid4">${[5, 15, 25, 50].map((m) => `<button data-act="timer" data-arg="${m}">${m}m</button>`).join("")}</div>
      ${t.running ? `<button class="pill primary" data-act="timerstop">Stop timer</button>` : ""}
      <p>Timers count as proof for manual goals like “Read 30 min”.</p>`;
  }
}

// ---------- settings ----------
const PROVIDERS = [
  ["off", "Off"], ["groq", "Groq"], ["gemini", "Gemini"], ["custom", "Custom"],
];
const PRESET_HINT = {
  groq: ["https://api.groq.com/openai/v1", "llama-3.3-70b-versatile"],
  gemini: ["https://generativelanguage.googleapis.com/v1beta/openai", "gemini-2.5-flash"],
  custom: ["http://localhost:11434/v1", "llama3.2"],
};
const sw = (key, on) => `<label class="switch"><input type="checkbox" data-set="${key}"${on ? " checked" : ""}><span></span></label>`;
const keyStatus = (name) => S.sheet.secrets[name] ? `<span class="status ok">Saved in Windows Credential Manager</span>` : `<span class="status">Not set</span>`;

function renderSheet() {
  const el = $("sheet");
  el.hidden = !S.sheet;
  if (!S.sheet) return;
  const { settings: st, data_dir } = S.sheet;
  const ai = st.ai;
  const [hintUrl, hintModel] = PRESET_HINT[ai.provider] ?? ["", ""];
  el.innerHTML = `
    <header><h2>Settings</h2><button class="close round" data-act="settings" aria-label="Close settings">${svg("plus", 16, 2.4).replace("<svg", '<svg style="transform:rotate(45deg)"')}</button></header>

    <section>
      <h3>General</h3>
      <div class="set-row"><div>Start with Windows<small>Runs quietly in the tray, Alt + Space to open.</small></div>${sw("autostart", st.autostart)}</div>
    </section>

    <section>
      <h3>AI (optional)</h3>
      <p class="status">Used to pull tasks and deadlines out of messages. Local rules work without it.</p>
      <div class="tabs">${PROVIDERS.map(([id, l]) => `<button class="${ai.provider === id ? "on" : ""}" data-act="provider" data-arg="${id}">${l}</button>`).join("")}</div>
      ${ai.provider === "off" ? "" : `
      <div class="fields2">
        <label class="field">Base URL<input data-set="ai.base_url" value="${esc(ai.base_url)}" placeholder="${hintUrl}"></label>
        <label class="field">Model<input data-set="ai.model" value="${esc(ai.model)}" placeholder="${hintModel}"></label>
      </div>
      <div class="keyrow">
        <label class="field">API key${ai.provider === "custom" ? " (blank for local servers)" : ""}<input id="key-ai" type="password" autocomplete="off" placeholder="${S.sheet.secrets.ai ? "•••••••• (saved)" : "Paste key"}"></label>
        <button class="btn" data-act="savekey" data-arg="ai">Save</button>
        <button class="btn" data-act="clearkey" data-arg="ai">Clear</button>
      </div>
      <div class="set-row">${keyStatus("ai")}<button class="btn primary" data-act="aitest">Test</button></div>
      ${S.sheet.test ? `<span class="status ${S.sheet.test.ok ? "ok" : "bad"}">${esc(S.sheet.test.text)}</span>` : ""}`}
    </section>

    <section>
      <h3>Sources</h3>
      <div class="set-row"><div>Windows notifications<small>Reads Windows' notification history on this PC to spot tasks and deadlines.</small></div>${sw("notifications", st.notifications)}</div>
      <div class="set-row"><div>Gmail<small>IMAP with an app password (Google Account › Security › App passwords).</small></div>${sw("gmail.enabled", st.gmail.enabled)}</div>
      ${st.gmail.enabled ? `
      <label class="field">Gmail address<input data-set="gmail.address" value="${esc(st.gmail.address)}" placeholder="you@gmail.com"></label>
      <div class="keyrow">
        <label class="field">App password<input id="key-gmail" type="password" autocomplete="off" placeholder="${S.sheet.secrets.gmail ? "•••••••• (saved)" : "16-character app password"}"></label>
        <button class="btn" data-act="savekey" data-arg="gmail">Save</button>
        <button class="btn" data-act="clearkey" data-arg="gmail">Clear</button>
      </div>${keyStatus("gmail")}` : ""}
      <div class="set-row"><div>Outlook / Microsoft 365<small>Signs in with your Microsoft account using your own Azure app registration.</small></div>${sw("outlook.enabled", st.outlook.enabled)}</div>
      ${st.outlook.enabled ? `
      <label class="field">Application (client) ID<input data-set="outlook.client_id" value="${esc(st.outlook.client_id)}" placeholder="00000000-0000-0000-0000-000000000000"></label>
      <span class="status">Azure portal › App registrations › New. Allow public client flows, add the Mail.Read permission.</span>
      ${S.sheet.secrets.outlook
        ? `<div class="set-row"><span class="status ok">${esc(st.outlook.account || "Signed in")}</span><button class="btn" data-act="outlookout">Sign out</button></div>`
        : S.sheet.signin
          ? `<div class="set-row"><span class="status">Enter <b class="code">${esc(S.sheet.signin.user_code)}</b> at ${esc(S.sheet.signin.verification_uri)} (opened in your browser)</span></div>`
          : `<div class="set-row"><span class="status">Not signed in</span><button class="btn primary" data-act="outlookin">Sign in</button></div>`}` : ""}
      ${st.gmail.enabled || st.outlook.enabled ? `<div class="set-row"><span class="status${S.sheet.mail ? (S.sheet.mail.ok ? " ok" : " bad") : ""}">${esc(S.sheet.mail?.text ?? "Checks every 5 minutes")}</span><button class="btn" data-act="mailcheck">Check now</button></div>` : ""}
    </section>

    <section>
      <h3>Files</h3>
      <div class="links">
        <button class="btn" data-act="editgoals">Edit goals</button>
        <button class="btn" data-act="edit">Edit deck</button>
        <button class="btn" data-act="datadir">Open data folder</button>
      </div>
      <span class="status">${esc(data_dir)}</span>
    </section>`;
}

function setPath(obj, path, value) {
  const keys = path.split(".");
  const last = keys.pop();
  keys.reduce((o, k) => o[k], obj)[last] = value;
}
function saveSettings() {
  if (!S.sheet) return;
  invoke("save_settings", { settings: S.sheet.settings }).catch((e) => toast("Couldn't save: " + e));
}
async function openSettings() {
  const view = tauri ? await invoke("get_settings").catch((e) => { toast(String(e)); return null; })
    : { settings: { autostart: false, notifications: false, gmail: { enabled: false, address: "" }, outlook: { enabled: false, client_id: "", account: "" }, ai: { provider: "off", base_url: "", model: "" } }, secrets: {}, data_dir: "%APPDATA%\\dev.braxtonelmer.moondeck" };
  if (!view) return;
  S.sheet = view;
  S.pop = null; S.cmdOpen = false;
  render();
}

function renderToast() {
  const el = $("toast");
  el.hidden = !S.toast || !!S.pop;
  el.innerHTML = `${svg("check", 14, 3)}${esc(S.toast)}`;
}

function render() {
  store.set("timerEnd", S.timerEnd);
  store.set("focusUntil", S.focusUntil);
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
  renderSheet();
  $("settings").innerHTML = svg("sliders", 18, 2);
  $("hint").innerHTML = S.editDeck
    ? "Drag to reorder · click a tile to edit"
    : "<kbd>1–9</kbd> tiles · <kbd>[ ]</kbd> pages · <kbd>/</kbd> search · <kbd>Esc</kbd> to tray";
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
  S.notes = [text, ...S.notes].slice(0, 12);
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
  toggle: (id) => {
    const g = S.goals?.find((x) => x.id === id);
    if (g) markGoal(id, g.done ? "undo" : "done");
  },
  done: () => {
    const c = queue(goals())[0];
    S.snoozeOpen = false;
    if (c?.isTask) setTask(c.id, "done");
    else if (c) markGoal(c.id, "done");
  },
  taskdone: (id) => setTask(Number(id), "done"),
  taskdismiss: (id) => setTask(Number(id), "dismissed"),
  primary: () => {
    const c = queue(goals())[0];
    if (!c) return;
    if (c.action?.open?.length) invoke("open_targets", { targets: c.action.open }).catch((e) => toast("Couldn't open " + e));
    else if (c.manual) markGoal(c.id, "done");
  },
  settings: () => { if (S.sheet) S.sheet = null; else openSettings(); },
  provider: (id) => { S.sheet.settings.ai.provider = id; S.sheet.test = null; saveSettings(); },
  savekey: (name) => {
    const input = $("key-" + name);
    const value = input?.value ?? "";
    if (!value.trim()) return toast("Paste a key first");
    invoke("set_secret", { name, value }).then(() => { S.sheet.secrets[name] = true; toast("Saved"); render(); }, (e) => toast(String(e)));
  },
  clearkey: (name) => {
    invoke("set_secret", { name, value: "" }).then(() => { S.sheet.secrets[name] = false; toast("Cleared"); render(); });
  },
  aitest: () => {
    S.sheet.test = { ok: true, text: "Testing…" };
    invoke("ai_test").then(
      (reply) => { S.sheet.test = { ok: true, text: "Connected · model replied “" + reply.slice(0, 40) + "”" }; render(); },
      (e) => { S.sheet.test = { ok: false, text: String(e) }; render(); },
    );
  },
  datadir: () => invoke("open_data_dir"),
  mailcheck: () => {
    S.sheet.mail = { ok: true, text: "Checking…" };
    invoke("mail_check").then(
      (text) => { S.sheet.mail = { ok: true, text }; render(); loadTasks(); },
      (e) => { S.sheet.mail = { ok: false, text: String(e) }; render(); },
    );
  },
  outlookin: () => {
    invoke("outlook_sign_in").then(
      (code) => { S.sheet.signin = code; render(); },
      (e) => { S.sheet.mail = { ok: false, text: String(e) }; render(); },
    );
  },
  outlookout: () => invoke("outlook_sign_out").then(openSettings),
  editgoals: () => { invoke("edit_goals").catch(() => {}); toast("Opened goals.json · changes show next time"); },
  snoozemenu: () => (S.snoozeOpen = !S.snoozeOpen),
  snooze: (label) => {
    const c = queue(goals())[0];
    S.snoozeOpen = false;
    if (c) invoke("snooze_goal", { id: c.id, label }).then(loadGoals);
  },
  note: addNote,
  page: (i) => {
    i = Number(i);
    if (S.editDeck && S.page === i) { S.pop = S.pop === "page" ? null : "page"; return; }
    S.page = i; S.pop = null; store.set("page", i);
  },
  tile: (arg) => {
    const [p, i] = arg.split(":").map(Number);
    const tile = S.deckCfg?.pages?.[p]?.tiles?.[i];
    if (!tile) return;
    if (S.editDeck) { S.editTile = tile; S.pop = "tile"; return; }
    runTile(tile);
  },
  editdeck: () => { S.editDeck = !S.editDeck; S.pop = null; S.editTile = null; },
  edit: () => { S.editDeck = true; S.pop = "add"; },
  addtile: () => { S.pop = S.pop === "add" ? null : "add"; S.appQuery = ""; },
  addkind: (k) => (S.addKind = k),
  addapp: (name) => addTile({ label: name.replace(/^Visual Studio Code$/, "VS Code"), icon: "app", run: { type: "open", targets: [name] } }),
  addbuiltin: (a) => { const [, icon, label] = BUILTINS.find((b) => b[0] === a); addTile({ label, icon, run: { type: "builtin", action: a } }); },
  addcustom: (kind) => {
    const label = $("add-label").value.trim();
    const raw = $("add-target").value.trim();
    if (!raw) return toast("Fill in what it should open");
    if (kind === "routine") {
      const targets = raw.split("\n").map((x) => x.trim()).filter(Boolean);
      addTile({ label: label || "Routine", icon: "sun", run: { type: "routine", targets, focus: Number($("add-focus").value) || 0 } });
    } else {
      const target = kind === "web" && !isUrl(raw) ? "https://" + raw : raw;
      addTile({ label: label || target.replace(/^https?:\/\//, "").split(/[\/\\]/).filter(Boolean).pop(), icon: kind === "web" ? "globe" : "app", run: { type: "open", targets: [target] } });
    }
    S.pop = null;
  },
  savetile: () => {
    const tile = S.editTile;
    tile.label = $("edit-label").value.trim() || tile.label;
    if ($("edit-targets")) tile.run.targets = $("edit-targets").value.split("\n").map((x) => x.trim()).filter(Boolean);
    if ($("edit-focus")) tile.run.focus = Number($("edit-focus").value) || 0;
    saveDeck(); S.pop = null; S.editTile = null;
  },
  movetile: (d) => {
    const tiles = page().tiles;
    const i = tiles.indexOf(S.editTile), j = i + Number(d);
    if (j < 0 || j >= tiles.length) return;
    [tiles[i], tiles[j]] = [tiles[j], tiles[i]];
    saveDeck();
  },
  removetile: () => {
    const tiles = page().tiles;
    tiles.splice(tiles.indexOf(S.editTile), 1);
    saveDeck(); S.pop = null; S.editTile = null;
  },
  addpage: () => {
    S.deckCfg.pages.push({ name: "Page " + (S.deckCfg.pages.length + 1), tiles: [] });
    S.page = S.deckCfg.pages.length - 1; S.pop = "page"; saveDeck();
  },
  savepage: () => { page().name = $("page-name").value.trim() || page().name; saveDeck(); S.pop = null; },
  movepage: (d) => {
    const ps = S.deckCfg.pages, i = S.page, j = i + Number(d);
    if (j < 0 || j >= ps.length) return;
    [ps[i], ps[j]] = [ps[j], ps[i]];
    S.page = j; saveDeck();
  },
  removepage: () => {
    if (S.deckCfg.pages.length < 2) return;
    S.deckCfg.pages.splice(S.page, 1);
    S.page = Math.max(0, S.page - 1); S.pop = null; saveDeck();
  },
  pop: (key) => { S.pop = S.pop === key ? null : key; S.toast = ""; },
  focus: () => (S.focusUntil = S.focusUntil && S.focusUntil > Date.now() ? null : Date.now() + focusMinutes() * 60000),
  mic: () => {
    S.micMuted = !S.micMuted;
    invoke("set_mic_muted", { muted: S.micMuted }).catch(() => { S.micMuted = !S.micMuted; toast("No microphone found"); render(); });
  },
  media: (action) => invoke("media", { action }),
  shell: (action) => invoke("shell_action", { action }),
  timer: (m) => (S.timerEnd = Date.now() + Number(m) * 60000),
  timerstop: () => (S.timerEnd = null),
  toast: (text) => toast(text),
  pin: (kind) => {
    if (!tauri) return toast("Widgets open in the app");
    invoke("pin_widget", { kind }).then(() => toast("Pinned · drag it anywhere"), (e) => toast(String(e)));
  },
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
    if (S.sheet) { S.sheet = null; render(); return; }
    if (S.pop || S.cmdOpen || S.snoozeOpen) { S.pop = null; S.cmdOpen = false; S.snoozeOpen = false; render(); }
    else hide();
  }
});

document.addEventListener("keydown", (e) => {
  if (e.key === "Escape") {
    if (S.sheet) { S.sheet = null; render(); return; }
    if (S.pop || S.cmdOpen || S.snoozeOpen) { S.pop = null; S.cmdOpen = false; S.snoozeOpen = false; render(); }
    else hide();
  } else if (e.key === "Enter" && e.target.id === "note") {
    addNote(); render(); $("note").focus();
  } else if (e.target.id === "cmdq" && (e.key === "ArrowDown" || e.key === "ArrowUp")) {
    e.preventDefault();
    const rows = [...$("cmdlist").querySelectorAll(".row")];
    const i = rows.findIndex((r) => r.classList.contains("sel"));
    const j = Math.max(0, Math.min(rows.length - 1, i + (e.key === "ArrowDown" ? 1 : -1)));
    rows.forEach((r, k) => r.classList.toggle("sel", k === j));
    rows[j]?.scrollIntoView({ block: "nearest" });
  } else if (e.key === "Enter" && e.target.id === "cmdq") {
    ($("cmdlist").querySelector(".row.sel") ?? $("cmdlist").querySelector(".row"))?.click();
  } else if (!e.target.closest?.("input, textarea, select") && !e.ctrlKey && !e.altKey && !e.metaKey) {
    // Shortcuts when not typing: 1-9 run tiles, [ ] switch pages, / or Ctrl+K search.
    const pages = S.deckCfg?.pages ?? [];
    if (/^[1-9]$/.test(e.key) && !S.editDeck) {
      const tile = page()?.tiles?.[Number(e.key) - 1];
      if (tile) { runTile(tile); render(); }
    } else if ((e.key === "]" || e.key === "PageDown") && pages.length) {
      ACTIONS.page((S.page + 1) % pages.length); render();
    } else if ((e.key === "[" || e.key === "PageUp") && pages.length) {
      ACTIONS.page((S.page - 1 + pages.length) % pages.length); render();
    } else if (e.key === "/") {
      e.preventDefault(); S.cmdOpen = true; render();
    }
  } else if (e.ctrlKey && e.key.toLowerCase() === "k") {
    e.preventDefault(); S.cmdOpen = !S.cmdOpen; render();
  }
});

document.addEventListener("input", (e) => {
  if (e.target.id === "vol") {
    S.volume = Number(e.target.value);
    $("volval").textContent = S.volume + "%";
    invoke("set_volume", { level: S.volume });
  } else if (e.target.id === "cmdq") {
    filterCmd(e.target.value);
  } else if (e.target.id === "appq") {
    S.appQuery = e.target.value;
    const pos = e.target.selectionStart;
    renderPop(timer());
    $("appq").setSelectionRange(pos, pos);
  }
});
document.addEventListener("change", (e) => {
  const key = e.target.dataset?.set;
  if (key && S.sheet) {
    setPath(S.sheet.settings, key, e.target.type === "checkbox" ? e.target.checked : e.target.value.trim());
    saveSettings();
    if (e.target.type === "checkbox") render();
    return;
  } if (e.target.id === "vol") renderDeck(timer()); });

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

async function loadGoals() {
  if (!tauri) return;
  const gs = await invoke("goals_today").catch((e) => { toast(String(e)); return null; });
  if (gs) { S.goals = gs; render(); }
}
function markGoal(id, kind) {
  if (!tauri) {
    const g = S.goals.find((x) => x.id === id);
    if (g) { g.done = kind === "done"; g.by_hand = g.done; g.pct = g.done ? 100 : 0; }
    return;
  }
  invoke("mark_goal", { id, kind }).then(loadGoals);
}

async function loadTasks() {
  if (!tauri) return;
  const ts = await invoke("tasks_open").catch(() => null);
  if (ts) { S.tasks = ts; render(); }
}
function setTask(id, status) {
  S.tasks = (S.tasks ?? []).filter((t) => t.id !== id);
  if (tauri) invoke("set_task_status", { id, status }).then(loadTasks);
}

async function loadSystem() {
  if (!tauri) return;
  const [cfg, audio] = await Promise.all([
    invoke("deck_config").catch((e) => { toast(String(e)); return null; }),
    invoke("audio_state").catch(() => null),
  ]);
  if (cfg) S.deckCfg = cfg;
  else if (!S.deckCfg) S.deckCfg = { pages: [] };
  if (audio) { S.volume = audio.volume; S.micMuted = audio.mic_muted; }
  invoke("get_settings").then((v) => { S.aiOn = v.settings.ai.provider !== "off"; }, () => {});
  renderDeck(timer());
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
  loadSystem();
  loadGoals();
  loadTasks();
  S.pop = null; S.cmdOpen = false; S.snoozeOpen = false; S.sheet = null; S.editDeck = false; S.editTile = null;
  render();
  enter();
});
tauri?.event?.listen("overlay:hide", hide);
// Widgets share timer and notes through localStorage.
window.addEventListener("storage", (e) => {
  if (e.key === "notes") S.notes = store.get("notes", []);
  else if (e.key === "timerEnd") S.timerEnd = store.get("timerEnd", null);
  else return;
  render();
});
tauri?.event?.listen("outlook:done", (e) => {
  if (!S.sheet) return;
  S.sheet.signin = null;
  S.sheet.mail = e.payload;
  openSettings().then(() => { if (S.sheet) { S.sheet.mail = e.payload; render(); } });
});

render();
syncAccent();
loadTimeline();
loadSystem();
loadGoals();
loadTasks();
enter();
