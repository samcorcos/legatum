// Truthfulness gate for the film script (pillar 5). Every number the narration or an on-screen
// caption states must be found in the run's facts (tools/film/lib/runlog.mjs facts()), with the
// kind of number it claims to be: a generation must be a logged generation, a turn a turn the log
// mentions, a percentage a logged rate. Sentences with an unsupported number are dropped.
//
// Deliberate limits: number WORDS below ten ("two strategies") are treated as ordinary language;
// the storyboard prompt asks for digits for every quantitative claim, so a digit is a claim.

const WORDS = { ten: 10, eleven: 11, twelve: 12, thirteen: 13, fourteen: 14, fifteen: 15, sixteen: 16, seventeen: 17, eighteen: 18, nineteen: 19, twenty: 20, thirty: 30, forty: 40, fifty: 50, sixty: 60, seventy: 70, eighty: 80, ninety: 90, hundred: 100 };
const GAME_ID = /\b(?:native|sim|lan|human)-[A-Za-z0-9-]{6,}\b/g;

/** Gather every number and id in the facts, sorted by the kind of claim it can support. */
export function collectFacts(facts) {
  const values = [], percents = [], gens = new Set(), turns = new Set(), seeds = new Set(), gameIds = new Set();
  const addPercentForms = (v, key) => { if (v >= 0 && v <= 1) percents.push(v * 100); if (/percent|pct/.test(key)) percents.push(v); };
  const scanString = (s, key) => {
    for (const id of s.match(GAME_ID) || []) gameIds.add(id);
    const re = /(\d+(?:\.\d+)?)\s*(%|percent)?/g;
    let m;
    while ((m = re.exec(s))) {
      const v = Number(m[1]);
      const before = s.slice(Math.max(0, m.index - 14), m.index).toLowerCase();
      if (m[2]) percents.push(v);
      else values.push(v);
      if (/turns?\s*$/.test(before) || /^t$/.test(before.slice(-1))) turns.add(v);
      if (/generations?\s*$|gens?\s*$|[–-]\s*$/.test(before)) gens.add(v);
      if (/seed\s*\(?$/.test(before)) seeds.add(v);
    }
    if (key === "game_id") gameIds.add(s);
  };
  const walk = (x, key = "", parent = "") => {
    if (x == null) return;
    if (typeof x === "number") {
      values.push(x);
      addPercentForms(x, key);
      if (/^(gen|from|to|before_gen|after_gen|from_gen|to_gen|first|last)$/.test(key)) gens.add(x);
      if (key === "turn" || key === "turns") turns.add(x);
      if (key === "seed") seeds.add(x);
      return;
    }
    if (typeof x === "string") {
      if (key === "seed" && /^\d+$/.test(x)) seeds.add(Number(x));
      if (/^(game_id|best_id|id)$/.test(key)) gameIds.add(x);
      scanString(x, key);
      return;
    }
    if (Array.isArray(x)) return x.forEach((v) => walk(v, key, parent));
    for (const [k, v] of Object.entries(x)) {
      const t = /_t(\d+)$/.exec(k); // ai_survival_t150: "turn 150" is part of the metric's name
      if (t) turns.add(Number(t[1]));
      walk(v, k, key);
    }
  };
  walk(facts);
  // A generation claim must name a logged generation, not any number that looks like one
  // (a run id such as 2026-09-27-2330 is not generation 2330).
  const logged = new Set((facts.generations?.series || []).map((g) => g.gen));
  const genClaims = logged.size ? logged : gens;
  return { values, percents, gens: genClaims, turns, seeds, gameIds };
}

const decimals = (s) => (s.includes(".") ? s.split(".")[1].length : 0);
const near = (list, v, tol) => list.some((f) => Math.abs(f - v) <= tol + 1e-9);

/** Find the numeric claims in one sentence: [{raw, value, kind, index}]. */
export function claimsIn(sentence) {
  const out = [];
  const s = sentence.replace(/(\d),(\d{3})\b/g, "$1$2"); // 1,200 -> 1200
  const re = /(\d+(?:\.\d+)?)(\s*(?:%|percent|per cent))?|\b(ten|eleven|twelve|thirteen|fourteen|fifteen|sixteen|seventeen|eighteen|nineteen|twenty|thirty|forty|fifty|sixty|seventy|eighty|ninety|hundred)\b(\s*(?:%|percent|per cent))?/gi;
  let m, lastKind = null, lastEnd = -1;
  while ((m = re.exec(s))) {
    const raw = m[0];
    const value = m[1] != null ? Number(m[1]) : WORDS[m[3].toLowerCase()];
    const isPct = !!(m[2] || m[4]);
    const before = s.slice(Math.max(0, m.index - 16), m.index).toLowerCase();
    const after = s.slice(m.index + raw.length, m.index + raw.length + 12).toLowerCase();
    let kind = isPct ? "percent" : "number";
    if (!isPct) {
      if (/generations?\s+$|\bgens?\s+$|\bgen\.?\s*$/.test(before)) kind = "gen";
      else if (/turns?\s+$|\bt$/.test(before)) kind = "turn";
      else if (/seed\s+$/.test(before)) kind = "seed";
      else if (/^\s*(?:[–—-]|to|and|through)\s*$/.test(s.slice(lastEnd, m.index).toLowerCase()) && (lastKind === "gen" || lastKind === "turn")) kind = lastKind;
      else if (/^\s*(?:th|st|nd|rd)?\s*generation/.test(after)) kind = "gen";
    }
    out.push({ raw: raw.trim(), value, kind, decimals: m[1] ? decimals(m[1]) : 0, index: m.index });
    lastKind = kind;
    lastEnd = m.index + raw.length;
  }
  return out;
}

/** Check one sentence. Returns { ok, unsupported: [reason] }. */
export function checkSentence(sentence, known) {
  const unsupported = [];
  for (const id of sentence.match(GAME_ID) || []) if (!known.gameIds.has(id)) unsupported.push(`unknown game id ${id}`);
  const idSpans = [...sentence.matchAll(GAME_ID)].map((m) => [m.index, m.index + m[0].length]);
  for (const c of claimsIn(sentence)) {
    if (idSpans.some(([a, b]) => c.index >= a && c.index < b)) continue;
    const tol = c.decimals ? 0.5 * 10 ** -c.decimals : 0.5;
    let ok;
    if (c.kind === "gen") ok = known.gens.has(c.value);
    else if (c.kind === "turn") ok = known.turns.has(c.value);
    else if (c.kind === "seed") ok = known.seeds.has(c.value);
    else if (c.kind === "percent") ok = near(known.percents, c.value, tol);
    else ok = near(known.values, c.value, c.decimals ? tol : 0);
    if (!ok) unsupported.push(`${c.kind} ${c.raw} is not in the run log`);
  }
  return { ok: unsupported.length === 0, unsupported };
}

export const splitSentences = (text) => String(text || "").replace(/\s+/g, " ").trim().split(/(?<=[.!?])\s+(?=[A-Z0-9"“])/).filter(Boolean);

const SCENE_TYPES = new Set(["title", "text", "population", "graph", "map", "zoom", "versus", "matrix", "stat", "end"]);
const VISUAL = new Set(["population", "graph", "map", "zoom", "versus", "matrix", "stat", "title", "end"]);

/**
 * Validate a whole script against the facts. Drops unsupported sentences and on-screen text,
 * repairs or drops scenes whose parameters point at things the log does not have, and returns
 * { script, report } where report lists every removal (it is saved beside the film).
 */
export function validateScript(script, facts, { mapGames = new Set() } = {}) {
  const known = collectFacts(facts);
  const report = [];
  const gensList = facts.generations.series.map((g) => g.gen);
  const clampGen = (g) => (g == null ? null : gensList.reduce((a, b) => (Math.abs(b - g) < Math.abs(a - g) ? b : a), gensList[0]));
  const scenes = [];
  (script.scenes || []).forEach((sc, i) => {
    const where = `scene ${i + 1} (${sc.type})`;
    if (!SCENE_TYPES.has(sc.type)) { report.push(`${where}: unknown scene type, dropped`); return; }
    const p = { ...(sc.params || {}) };
    const narration = [];
    for (const s of (sc.narration || []).flatMap(splitSentences)) {
      const r = checkSentence(s, known);
      if (r.ok) narration.push(s);
      else report.push(`${where}: dropped "${s}" (${r.unsupported.join("; ")})`);
    }
    for (const key of ["heading", "body", "caption", "kicker", "label_a", "label_b"]) {
      if (typeof p[key] !== "string" || !p[key]) continue;
      // Lines stay lines (a card's body is laid out by line); sentences are checked one by one.
      p[key] = p[key].split("\n").map((line) => splitSentences(line).filter((s) => {
        const r = checkSentence(s, known);
        if (!r.ok) report.push(`${where}: removed on-screen ${key} "${s}" (${r.unsupported.join("; ")})`);
        return r.ok;
      }).join(" ")).filter(Boolean).join("\n");
    }
    if (p.gen_from != null && !known.gens.has(p.gen_from)) { report.push(`${where}: gen_from ${p.gen_from} not logged, using ${clampGen(p.gen_from)}`); p.gen_from = clampGen(p.gen_from); }
    if (p.gen_to != null && !known.gens.has(p.gen_to)) { report.push(`${where}: gen_to ${p.gen_to} not logged, using ${clampGen(p.gen_to)}`); p.gen_to = clampGen(p.gen_to); }
    if (["map", "zoom", "versus"].includes(sc.type)) {
      for (const k of sc.type === "versus" ? ["game_id", "game_id_b"] : ["game_id"]) {
        if (!mapGames.has(p[k])) { report.push(`${where}: ${k} ${p[k]} is not a showcase tape, scene dropped`); return; }
      }
    }
    if (!narration.length && !VISUAL.has(sc.type) && !(p.heading || p.body)) { report.push(`${where}: nothing left to say or show, dropped`); return; }
    scenes.push({ ...sc, params: p, narration });
  });
  return { script: { ...script, scenes }, report };
}
