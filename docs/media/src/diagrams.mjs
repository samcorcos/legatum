// Builds the README explainers as self-contained SVGs, one light and one dark per diagram.
//   node docs/media/src/diagrams.mjs
// Each SVG carries its own opaque panel (so it reads on either GitHub page colour), uses only system fonts (an <img> SVG
// loads no web fonts), and animates with CSS that switches off under prefers-reduced-motion (the finished state stands).
import { writeFileSync, mkdirSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const out = join(dirname(fileURLToPath(import.meta.url)), "../diagrams");
mkdirSync(out, { recursive: true });

const PAL = {
  light: { card: "#f2ead8", chip: "#f7f1e3", ink: "#2b2419", ink2: "#5b503f", rule: "#bcac8e", track: "#dfd2b5", accent: "#9a3322", gilt: "#8f6f1c", good: "#3d6427", wash: "rgba(154,51,34,.13)", seats: ["#2f6f9a", "#a8741a", "#4a7a3a", "#7a4a8a"], hex: "rgba(43,36,25,.055)" },
  dark: { card: "#1b1712", chip: "#241f17", ink: "#efe5cc", ink2: "#b9ab8c", rule: "#4c4333", track: "#2e281e", accent: "#e57a5f", gilt: "#d6b25c", good: "#93c46e", wash: "rgba(229,122,95,.16)", seats: ["#6fb3d6", "#e0a84a", "#8fc476", "#c392d4"], hex: "rgba(239,229,204,.05)" },
};
const SERIF = `'Cormorant Garamond','Iowan Old Style','Palatino Linotype',Palatino,Georgia,serif`;
const SANS = `'Avenir Next','Segoe UI',system-ui,-apple-system,Helvetica,Arial,sans-serif`;
const esc = (s) => s.replace(/&/g, "&amp;").replace(/</g, "&lt;");

/** Wrap a diagram body in the shared card: double ink rule, faint hex-grid paper, title and description for screen readers. */
function shell({ w, h, title, desc, body, css = "" }, p) {
  const hexW = 46, hexH = 40;
  return `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 ${w} ${h}" width="${w}" height="${h}" role="img" aria-labelledby="t d">
<title id="t">${esc(title)}</title><desc id="d">${esc(desc)}</desc>
<style>
text{font-family:${SANS};fill:${p.ink}}
.serif{font-family:${SERIF}}
.kick{font-size:12px;letter-spacing:.24em;fill:${p.accent};font-weight:700}
.h{font-size:30px;font-weight:600}
.lab{font-size:11px;letter-spacing:.18em;fill:${p.ink2};font-weight:700}
.sm{font-size:13px;fill:${p.ink2}}
.md{font-size:14px}
.bd{font-weight:700}
.c{text-anchor:middle}
${css}
@media (prefers-reduced-motion:reduce){*{animation:none!important}}
</style>
<defs><pattern id="hx" width="${hexW}" height="${hexH * 2}" patternUnits="userSpaceOnUse"><path d="M0 ${hexH * 0.5}l${hexW * 0.25} -${hexH * 0.5}h${hexW * 0.5}l${hexW * 0.25} ${hexH * 0.5}l-${hexW * 0.25} ${hexH * 0.5}h-${hexW * 0.5}z M${hexW * 0.5} ${hexH * 1.5}l${hexW * 0.25} -${hexH * 0.5}h${hexW * 0.5}" fill="none" stroke="${p.hex}" stroke-width="1"/></pattern>
<marker id="ar" viewBox="0 0 10 10" refX="8" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path d="M1 1.5L8.5 5L1 8.5z" fill="${p.ink2}"/></marker></defs>
<rect x="1" y="1" width="${w - 2}" height="${h - 2}" rx="14" fill="${p.card}" stroke="${p.ink}" stroke-width="2"/>
<rect x="1" y="1" width="${w - 2}" height="${h - 2}" rx="14" fill="url(#hx)"/>
<rect x="8" y="8" width="${w - 16}" height="${h - 16}" rx="9" fill="none" stroke="${p.rule}" stroke-width="1"/>
${body}
</svg>
`;
}

/** CSS keyframes for one element over a loop of `total` seconds: points are [seconds, value]; the first and last repeat so the loop is seamless. */
function anim(id, prop, points, total, extra = "") {
  const pct = (t) => `${((t / total) * 100).toFixed(2)}%`;
  const frames = points.map(([t, v]) => `${pct(t)}{${prop}:${v}}`).join("");
  return `.${id}{animation:${id} ${total}s linear infinite;${extra}}@keyframes ${id}{${frames}}`;
}
const star = (cx, cy, r, fill) => `<path d="M${cx} ${cy - r}L${cx + r * 0.24} ${cy - r * 0.24}L${cx + r} ${cy}L${cx + r * 0.24} ${cy + r * 0.24}L${cx} ${cy + r}L${cx - r * 0.24} ${cy + r * 0.24}L${cx - r} ${cy}L${cx - r * 0.24} ${cy - r * 0.24}Z" fill="${fill}"/>`;
const box = (x, y, w, h, p, o = {}) => `<rect x="${x}" y="${y}" width="${w}" height="${h}" rx="${o.rx ?? 9}" fill="${o.fill ?? p.chip}" stroke="${o.stroke ?? p.ink}" stroke-width="${o.sw ?? 1.5}"${o.dash ? ` stroke-dasharray="${o.dash}"` : ""}/>`;
const text = (x, y, s, cls = "", extra = "") => `<text x="${x}" y="${y}" class="${cls}" ${extra}>${esc(s)}</text>`;
const arrow = (d, p, o = {}) => `<path d="${d}" fill="none" stroke="${o.color ?? p.ink2}" stroke-width="${o.sw ?? 1.6}" marker-end="url(#ar)"${o.both ? ' marker-start="url(#ar)"' : ""}${o.dash ? ` stroke-dasharray="${o.dash}"` : ""}/>`;

// ---------------------------------------------------------------------------------------------------------------------
// 1. Simultaneous turns (animated)
function simultaneous(p) {
  const W = 960, H = 480, T = 10;
  const lanes = [
    { name: "You", done: 2.7, start: 0.4 },
    { name: "Your partner", done: 3.5, start: 0.5 },
    { name: "AI · Sparta", done: 1.3, start: 0.3 },
    { name: "AI · Mongols", done: 1.9, start: 0.3 },
  ];
  const stages = ["Surprise wars begin", "Ranged fire", "Movement and melee, in rounds", "Cities are founded", "Growth, Gold, research, borders", "Scoring, then the next turn"];
  const stageAt = (i) => 4.3 + i * 0.8;
  let css = "", body = "";
  body += text(40, 50, "SIMULTANEOUS TURNS", "kick");
  body += text(40, 90, "Everyone plans at once. The engine resolves it in fixed stages.", "serif h");
  body += text(40, 134, "1   PLAN, ALL AT THE SAME TIME", "lab");
  lanes.forEach((l, i) => {
    const y = 156 + i * 58, col = p.seats[i];
    body += `<circle cx="58" cy="${y + 20}" r="16" fill="${p.card}" stroke="${col}" stroke-width="2.5"/>${star(58, y + 20, 8.5, col)}`;
    body += text(86, y + 14, l.name, "md bd");
    body += `<rect x="86" y="${y + 24}" width="262" height="9" rx="4.5" fill="${p.track}"/>`;
    body += `<rect class="bar${i}" x="86" y="${y + 24}" width="262" height="9" rx="4.5" fill="${col}" style="transform-box:fill-box;transform-origin:left"/>`;
    body += `<path class="ok${i}" d="M364 ${y + 27}l5 5l10 -11" fill="none" stroke="${p.good}" stroke-width="3" stroke-linecap="round" stroke-linejoin="round"/>`;
    css += anim(`bar${i}`, "transform", [[0, "scaleX(0)"], [l.start, "scaleX(0)"], [l.done, "scaleX(1)"], [9.4, "scaleX(1)"], [9.8, "scaleX(0)"], [T, "scaleX(0)"]], T);
    css += anim(`ok${i}`, "opacity", [[0, 0], [l.done, 0], [l.done + 0.25, 1], [9.4, 1], [9.8, 0], [T, 0]], T);
  });
  // The gate: closed until the last order is in.
  body += `<rect x="412" y="148" width="8" height="228" rx="4" fill="${p.track}"/><rect class="gate" x="412" y="148" width="8" height="228" rx="4" fill="${p.accent}"/>`;
  body += text(416, 398, "Everyone", "sm c") + text(416, 414, "is ready", "sm c");
  css += anim("gate", "opacity", [[0, 0], [3.6, 0], [3.9, 1], [9.4, 1], [9.8, 0], [T, 0]], T);
  body += `<g class="flow">${arrow("M428 262H492", p, { color: p.accent, sw: 2.4 })}</g>`;
  css += anim("flow", "opacity", [[0, 0], [3.9, 0], [4.2, 1], [9.4, 1], [9.8, 0], [T, 0]], T);
  // The engine.
  body += box(500, 120, 422, 300, p);
  body += text(522, 150, "2   THE ENGINE RESOLVES THE TURN", "lab");
  body += text(522, 170, "legatum-core, written in Rust: same orders in, same state out", "sm");
  stages.forEach((s, i) => {
    const y = 186 + i * 38, t0 = stageAt(i);
    body += `<rect class="hi${i}" x="512" y="${y}" width="398" height="30" rx="6" fill="${p.wash}"/><rect class="hb${i}" x="512" y="${y}" width="4" height="30" rx="2" fill="${p.accent}"/>`;
    body += `<circle cx="540" cy="${y + 15}" r="11" fill="none" stroke="${p.ink2}" stroke-width="1.4"/>${text(540, y + 19.5, String(i + 1), "c bd", 'style="font-size:12px"')}`;
    body += text(564, y + 20, s, "md");
    body += `<path class="dn${i}" d="M884 ${y + 15}l5 5l10 -11" fill="none" stroke="${p.good}" stroke-width="2.6" stroke-linecap="round" stroke-linejoin="round"/>`;
    css += anim(`hi${i}`, "opacity", [[0, 0], [t0, 0], [t0 + 0.12, 1], [t0 + 0.72, 1], [t0 + 0.84, 0], [T, 0]], T);
    css += anim(`hb${i}`, "opacity", [[0, 0], [t0, 0], [t0 + 0.12, 1], [t0 + 0.72, 1], [t0 + 0.84, 0], [T, 0]], T);
    css += anim(`dn${i}`, "opacity", [[0, 0], [t0 + 0.7, 0], [t0 + 0.85, 1], [9.4, 1], [9.8, 0], [T, 0]], T);
  });
  body += text(480, 452, "No active player, no waiting for a slow one, and any game can be replayed exactly.", "sm c");
  // Reduced motion shows the finished state: no highlight wash, every stage ticked.
  css += stages.map((_, i) => `.hi${i},.hb${i}{opacity:0}`).join("") ;
  return shell({ w: W, h: H, title: "How a simultaneous turn resolves", desc: "Every empire, human or AI, plans its orders at the same time. When everyone is ready, the Rust engine resolves the turn in six fixed stages and returns the same next state to every player.", body, css }, p);
}

// ---------------------------------------------------------------------------------------------------------------------
// 2. Architecture
function architecture(p) {
  const W = 980, H = 560;
  let b = "";
  b += text(40, 50, "ARCHITECTURE", "kick");
  b += text(40, 90, "One Rust engine. Everything else drives it.", "serif h");
  // Front ends.
  const fe = [
    { x: 40, w: 440, t: "The game in your browser", s: "TypeScript UI · 2D hex renderer · phone to desktop", n: "engine runs in the page as WebAssembly" },
    { x: 500, w: 440, t: "The online host", s: "Two humans on a team and six AI in one game", n: "runs in the host's tab, through a relay" },
  ];
  fe.forEach((f) => { b += box(f.x, 116, f.w, 84, p) + text(f.x + 20, 148, f.t, "serif bd", 'style="font-size:22px"') + text(f.x + 20, 170, f.s, "sm") + text(f.x + 20, 188, f.n, "sm", `style="fill:${p.gilt};font-weight:600"`); });
  // Core.
  b += box(40, 244, 900, 120, p, { sw: 2.4, fill: p.card });
  b += `<rect x="46" y="250" width="888" height="108" rx="6" fill="none" stroke="${p.rule}"/>`;
  b += text(66, 282, "legatum-core", "serif bd", 'style="font-size:28px"') + text(66, 304, "game state + every order + a seed  →  the next state", "md");
  const chips = ["deterministic", "integer maths only", "no rendering code", "one engine, no second copy of the rules"];
  let cx = 66;
  chips.forEach((c) => { const w = c.length * 7.2 + 28; b += `<rect x="${cx}" y="318" width="${w}" height="32" rx="16" fill="${p.chip}" stroke="${p.ink2}" stroke-width="1.2"/>` + text(cx + w / 2, 339, c, "c md"); cx += w + 12; });
  // Arrows front ends <-> core.
  b += arrow("M260 202V242", p, { both: true }) + arrow("M720 202V242", p, { both: true });
  b += text(274, 228, "orders in, view out", "sm");
  b += text(734, 228, "orders in, view out", "sm");
  // Bottom row.
  const bt = [
    { x: 40, t: "Headless runner", s: "Rust CLI, parallel across every core", n: "thousands of seeded games per batch" },
    { x: 350, t: "The climb", s: "Parameter files, spot-instance batches", n: "scored on human milestones" },
    { x: 660, t: "The AI", s: "Easy, Normal and Hard", n: "sees only the view a player sees" },
  ];
  bt.forEach((c) => { b += box(c.x, 430, 280, 98, p) + text(c.x + 18, 462, c.t, "serif bd", 'style="font-size:22px"') + text(c.x + 18, 484, c.s, "sm") + text(c.x + 18, 504, c.n, "sm", `style="fill:${p.gilt};font-weight:600"`); });
  b += arrow("M180 428V366", p, { both: true }) + arrow("M800 428V366", p, { both: true });
  b += arrow("M322 479H346", p, { color: p.gilt, sw: 2.2 }) + arrow("M632 479H656", p, { color: p.gilt, sw: 2.2 });
  b += text(198, 404, "plays whole games", "sm") + text(818, 404, "orders in, view out", "sm");
  b += text(334, 420, "results", "sm c", 'style="font-size:11px"') + text(644, 420, "tuned", "sm c", 'style="font-size:11px"');
  return shell({ w: W, h: H, title: "Legatum architecture", desc: "A deterministic Rust engine, legatum-core, takes game state, orders and a seed and returns the next state. The browser game and the online host (a player's tab) run it as WebAssembly; the headless runner runs it natively. The AI sees only a player's view, and the climb tunes the AI's parameters from runner results.", body: b }, p);
}

// ---------------------------------------------------------------------------------------------------------------------
// 3. The AI does not cheat
function noCheat(p) {
  const W = 960, H = 520;
  let b = "";
  b += text(40, 50, "THE AI DOES NOT CHEAT", "kick");
  b += text(40, 90, "It sees what you see. It gets harder by deciding better.", "serif h");
  // Flow: truth -> fog -> two views -> same engine.
  b += box(40, 132, 190, 130, p) + text(135, 168, "The true game", "serif bd c", 'style="font-size:21px"') + text(135, 192, "every tile, every unit,", "sm c") + text(135, 210, "every treasury", "sm c") + text(135, 240, "only the engine knows", "sm c", `style="fill:${p.gilt};font-weight:600"`);
  b += arrow("M232 197H286", p, { sw: 2.2 });
  b += `<path d="M290 160H410L370 197V232H330V197Z" fill="${p.wash}" stroke="${p.accent}" stroke-width="2" stroke-linejoin="round"/>` + text(350, 150, "FOG OF WAR", "lab c", `style="fill:${p.accent}"`) + text(350, 258, "one shared filter", "sm c");
  b += arrow("M412 180H470", p, { sw: 2.2 }) + arrow("M412 214H470", p, { sw: 2.2 });
  const views = [
    { y: 124, t: "Your view", c: p.seats[0] },
    { y: 200, t: "An AI empire’s view", c: p.seats[2] },
  ];
  views.forEach((v) => {
    b += box(474, v.y, 262, 64, p, { stroke: v.c, sw: 2 }) + text(492, v.y + 26, v.t, "md bd") + text(492, v.y + 47, "explored tiles · units in sight", "sm", 'style="font-size:12px"');
  });
  b += arrow("M738 156H786", p) + arrow("M738 232H786", p);
  b += box(790, 132, 130, 132, p, { sw: 2 }) + text(855, 170, "Same rules", "serif bd c", 'style="font-size:21px"') + text(855, 196, "same engine,", "sm c") + text(855, 214, "same orders,", "sm c") + text(855, 232, "same limits", "sm c");
  // The ladder.
  b += text(40, 316, "WHAT “HARDER” MEANS", "lab");
  const rungs = [["Easy", "simple choices"], ["Normal", "solid choices"], ["Hard", "plans and commits to a strategy"]];
  rungs.forEach(([t, s], i) => {
    const x = 40 + i * 301, h = 62 + i * 14;
    b += `<rect x="${x}" y="${420 - h}" width="281" height="${h}" rx="7" fill="${p.chip}" stroke="${p.ink}" stroke-width="1.5"/>`;
    b += text(x + 14, 420 - h + 26, t, "md bd") + text(x + 14, 420 - h + 46, s, "sm", 'style="font-size:12px"');
  });
  b += text(40, 454, "More thinking, never more resources: no hidden bonuses, no map hack, no extra Gold.", "md bd");
  b += text(40, 480, "It is tuned by hill-climbing against milestones that people reached in real games.", "sm");
  return shell({ w: W, h: H, title: "The AI sees what a player sees", desc: "The true game state is filtered through fog of war into a player's view. The AI receives exactly the same kind of view and plays under the same rules; harder levels think more, they do not get resource bonuses or hidden information.", body: b }, p);
}

const diagrams = { "simultaneous-turns": simultaneous, architecture, "ai-no-cheating": noCheat };
for (const [name, fn] of Object.entries(diagrams)) {
  for (const mode of ["light", "dark"]) writeFileSync(join(out, `${name}-${mode}.svg`), fn(PAL[mode]));
  console.log(`wrote ${name}-light.svg and ${name}-dark.svg`);
}
