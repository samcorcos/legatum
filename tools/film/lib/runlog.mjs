// Reader for the evolution run log (docs/archive/ai-evolution-log.md). Shared by the dashboard's Evolution
// panel, the morning report and the film. It only reads the log, tolerates a run that is still
// being written (a torn last line is skipped) and never looks at evolution internals.
import fs from "node:fs";
import path from "node:path";
import zlib from "node:zlib";

// One color per strategy archetype, the same in the panel, the report and the film.
export const ARCHETYPE_COLORS = {
  Rush: "#e0674f",
  Wide: "#7cc07a",
  Science: "#a58be0",
  Economic: "#e3b64f",
  Turtle: "#5ec4c0",
  Wonder: "#5ec4c0",
  Diplomatic: "#e69ac8",
  Balanced: "#8f9aab",
};
const SPARE = ["#d9a066", "#9ccf5a", "#e07fa0", "#7fd6e0", "#c9c26b", "#b58f6d"];
export function archetypeColor(name) {
  if (ARCHETYPE_COLORS[name]) return ARCHETYPE_COLORS[name];
  let h = 0;
  for (const c of String(name)) h = (h * 31 + c.charCodeAt(0)) >>> 0;
  return SPARE[h % SPARE.length];
}

const readText = (file) => {
  if (fs.existsSync(file)) return fs.readFileSync(file, "utf8");
  if (fs.existsSync(`${file}.gz`)) return zlib.gunzipSync(fs.readFileSync(`${file}.gz`)).toString("utf8");
  return null;
};
const readJson = (file) => {
  const t = readText(file);
  if (t == null) return null;
  try { return JSON.parse(t); } catch { return null; }
};
export const parseJsonl = (text) => (text || "").split("\n").flatMap((l) => {
  if (!l.trim()) return [];
  try { return [JSON.parse(l)]; } catch { return []; } // a line still being written
});

function readGenDir(dir) {
  if (!fs.existsSync(dir)) return [];
  const gens = new Set();
  for (const f of fs.readdirSync(dir)) {
    const m = /^(\d+)\.json(\.gz)?$/.exec(f);
    if (m) gens.add(Number(m[1]));
  }
  return [...gens].sort((a, b) => a - b).map((g) => readJson(path.join(dir, `${g}.json`))).filter(Boolean);
}

/** List run directories (newest first) under docs/sim-results. */
export function listRuns(resultsDir) {
  if (!fs.existsSync(resultsDir)) return [];
  return fs.readdirSync(resultsDir)
    .filter((f) => /^evolve-/.test(f) && fs.existsSync(path.join(resultsDir, f, "run.json")))
    .sort()
    .reverse()
    .map((f) => path.join(resultsDir, f));
}

/** A cheap signature that changes whenever any log file changes (for caches). */
export function runSignature(dir) {
  const parts = [];
  const walk = (d) => {
    for (const f of fs.readdirSync(d)) {
      const p = path.join(d, f);
      const st = fs.statSync(p);
      if (st.isDirectory()) walk(p);
      else parts.push(`${path.relative(dir, p)}:${st.size}:${Math.floor(st.mtimeMs)}`);
    }
  };
  walk(dir);
  return parts.sort().join("|");
}

export function loadRun(dir) {
  const run = readJson(path.join(dir, "run.json"));
  if (!run) throw new Error(`${dir}: no readable run.json`);
  const generations = parseJsonl(readText(path.join(dir, "generations.jsonl"))).sort((a, b) => a.gen - b.gen);
  const events = parseJsonl(readText(path.join(dir, "events.jsonl"))).sort((a, b) => a.gen - b.gen);
  const checkpoints = readGenDir(path.join(dir, "checkpoints"));
  const coach = readGenDir(path.join(dir, "coach"));
  const digests = fs.readdirSync(dir).filter((f) => /^digest-.*\.md$/.test(f)).sort();
  return { dir, id: run.run_id || path.basename(dir).replace(/^evolve-/, ""), run, generations, events, checkpoints, coach, digests, standin: !!run.standin };
}

/** Archetype names in a stable order: by total share across the run, largest first. */
export function archetypesOf(generations) {
  const total = {};
  for (const g of generations) for (const [k, v] of Object.entries(g.archetype_share || {})) total[k] = (total[k] || 0) + v;
  return Object.keys(total).sort((a, b) => total[b] - total[a] || a.localeCompare(b));
}

const leaderOf = (share) => Object.entries(share || {}).reduce((a, [k, v]) => (v > a[1] ? [k, v] : a), [null, -1])[0];

/**
 * Eras: spans of generations where the most common archetype (on a 3-generation rolling mean of
 * the shares) stays the same. Runs shorter than `minLen` merge into the previous era, so one noisy
 * generation never becomes a chapter.
 */
export function eras(generations, { minLen } = {}) {
  if (!generations.length) return [];
  const names = archetypesOf(generations);
  const min = minLen ?? Math.max(2, Math.floor(generations.length / 12));
  const smooth = generations.map((g, i) => {
    const win = generations.slice(Math.max(0, i - 1), i + 2);
    return Object.fromEntries(names.map((n) => [n, win.reduce((a, w) => a + (w.archetype_share?.[n] || 0), 0) / win.length]));
  });
  let spans = [];
  generations.forEach((g, i) => {
    const lead = leaderOf(smooth[i]);
    const last = spans.at(-1);
    if (last && last.dominant === lead) { last.to = g.gen; last.idx.push(i); }
    else spans.push({ dominant: lead, from: g.gen, to: g.gen, idx: [i] });
  });
  // Merge short spans into their predecessor (or successor for the first one).
  let changed = true;
  while (changed && spans.length > 1) {
    changed = false;
    for (let i = 0; i < spans.length; i++) {
      if (spans[i].idx.length >= min) continue;
      const into = i > 0 ? spans[i - 1] : spans[i + 1];
      into.idx = i > 0 ? [...into.idx, ...spans[i].idx] : [...spans[i].idx, ...into.idx];
      into.from = generations[into.idx[0]].gen;
      into.to = generations[into.idx.at(-1)].gen;
      spans.splice(i, 1);
      changed = true;
      break;
    }
  }
  // Re-join neighbours that now share a leader.
  spans = spans.reduce((acc, s) => {
    const last = acc.at(-1);
    if (last && last.dominant === s.dominant) { last.idx.push(...s.idx); last.to = s.to; }
    else acc.push(s);
    return acc;
  }, []);
  return spans.map((s) => {
    const gens = s.idx.map((i) => generations[i]);
    const meanShare = Object.fromEntries(names.map((n) => [n, gens.reduce((a, g) => a + (g.archetype_share?.[n] || 0), 0) / gens.length]));
    const bestGen = gens.reduce((a, g) => ((g.best?.fitness ?? -Infinity) > (a.best?.fitness ?? -Infinity) ? g : a), gens[0]);
    const pw = gens.map((g) => g.best?.proxy_win_rate).filter((x) => x != null);
    return {
      from: s.from,
      to: s.to,
      dominant: s.dominant,
      dominant_share: meanShare[s.dominant],
      mean_share: meanShare,
      best: { gen: bestGen.gen, id: bestGen.best?.id, fitness: bestGen.best?.fitness },
      proxy_win_rate: pw.length ? { first: pw[0], last: pw.at(-1), mean: pw.reduce((a, b) => a + b, 0) / pw.length } : null,
    };
  });
}

/** Everything a reader might cite, in one object. Every number carries its generation or game id. */
export function facts(data) {
  const { run, generations, events, checkpoints, coach } = data;
  const first = generations[0], last = generations.at(-1);
  const cpFirst = checkpoints[0], cpLast = checkpoints.at(-1);
  return {
    run_id: data.id,
    status: run.status,
    standin: run.standin || null,
    started_at: run.started_at,
    finished_at: run.finished_at || null,
    config: run.config,
    baseline: run.baseline,
    generations: {
      count: generations.length,
      first: first?.gen ?? null,
      last: last?.gen ?? null,
      series: generations.map((g) => ({
        gen: g.gen,
        best_fitness: g.best?.fitness ?? null,
        best_id: g.best?.id ?? null,
        best_archetype: g.best?.archetype ?? null,
        proxy_win_rate: g.best?.proxy_win_rate ?? null,
        mean_fitness: g.mean_fitness ?? null,
        winner_entropy: g.winner_entropy ?? null,
        archetype_share: g.archetype_share || {},
      })),
    },
    archetypes: archetypesOf(generations),
    eras: eras(generations),
    events: events.map((e) => ({ gen: e.gen, kind: e.kind, detail: e.detail, game_id: e.game_id ?? null })),
    coach: coach.map((c) => ({
      gen: c.gen,
      summary: c.summary,
      dominant_strategy: c.dominant_strategy || null,
      observations: (c.observations || []).map((o) => ({ kind: o.kind, behavior: o.behavior, archetype: o.archetype ?? null, metric_change: o.metric_change || null, confidence: o.confidence ?? null, evidence: (o.evidence || []).slice(0, 3) })),
      proposals: (c.proposals || []).length,
    })),
    checkpoints: checkpoints.map((c) => ({
      gen: c.gen,
      benchmark: c.benchmark ? {
        proxy_win_rate: c.benchmark.proxy_win_rate ?? null,
        ai_survival_t150: c.benchmark.ai_survival_t150 ?? null,
        faction_win_rates: c.benchmark.faction_win_rates || {},
        archetype_win_rates: c.benchmark.archetype_win_rates || {},
        sample_games: c.benchmark.sample?.game_ids?.length ?? null,
      } : null,
      showcase: (c.showcase || []).map((s) => ({ game_id: s.game_id, seed: s.seed, why: s.why, highlight_turns: s.highlight_turns || [] })),
    })),
    before_after: cpFirst && cpLast && cpFirst !== cpLast ? {
      before_gen: cpFirst.gen,
      after_gen: cpLast.gen,
      proxy_win_rate: [cpFirst.benchmark?.proxy_win_rate ?? null, cpLast.benchmark?.proxy_win_rate ?? null],
      ai_survival_t150: [cpFirst.benchmark?.ai_survival_t150 ?? null, cpLast.benchmark?.ai_survival_t150 ?? null],
    } : null,
  };
}

/** Normalize a matchup matrix: the stand-in's {archetypes, ahead_rate} or a nested {A: {B: rate}}. */
export function matrixOf(m) {
  if (!m) return null;
  if (Array.isArray(m.archetypes) && Array.isArray(m.ahead_rate)) return { names: m.archetypes, rate: m.ahead_rate, pairs: m.pairs || null, definition: m.definition || null };
  const names = Object.keys(m).filter((k) => m[k] && typeof m[k] === "object");
  if (!names.length) return null;
  return { names, rate: names.map((a) => names.map((b) => (a === b ? null : m[a][b] ?? null))), pairs: null, definition: null };
}
