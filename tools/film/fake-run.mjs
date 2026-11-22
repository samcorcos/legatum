#!/usr/bin/env node
// Stand-in evolution run: writes a small run log in the exact docs/archive/ai-evolution-log.md contract,
// built from REAL simulated games, so the dashboard panel, the morning report and the film can be
// developed before tools/evolve exists. Nothing here evolves: each "generation" is a real
// legatum-sim batch whose AI difficulty steps Easy -> Normal -> Hard (with a Proxy-capable
// simulator) or whose brain steps random -> mixed -> scripted (without). run.json says so.
//
//   node tools/film/fake-run.mjs [--gens 21] [--games 8] [--coach-every 5] [--seed 880001]
//        [--sim path/to/legatum-sim] [--run-id YYYY-MM-DD-HHMM] [--out docs/sim-results]
//
// Resumable: finished batches are cached under ~/LegatumData/evolve-standin/<run-id>/.

import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { spawnSync, execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";
import { readArchive, snapshotAt } from "./lib/archive.mjs";
import { classifyGame, ARCHETYPE_ORDER } from "./lib/standin-archetypes.mjs";

const here = path.dirname(fileURLToPath(import.meta.url));
const root = path.resolve(here, "../..");

const args = Object.fromEntries(
  process.argv.slice(2).reduce((acc, a, i, all) => (a.startsWith("--") ? [...acc, [a.slice(2), all[i + 1]?.startsWith("--") || all[i + 1] == null ? "1" : all[i + 1]]] : acc), []),
);
const GENS = Number(args.gens || 21);
const GAMES = Number(args.games || 8);
const COACH_EVERY = Number(args["coach-every"] || 5);
// Seeds are u64: BigInt arithmetic, decimal strings on the wire (Number loses precision past 2^53).
if (args.seed && !/^\d+$/.test(args.seed)) throw new Error(`--seed must be a decimal u64, got ${args.seed}`);
const SEED = BigInt(args.seed || 880001);
if (SEED > 18446744073709551615n) throw new Error("--seed exceeds u64");
const seedAt = (offset) => ((SEED + BigInt(offset)) % 18446744073709551616n).toString();
const SPEED = "standard";
const sim = path.resolve(args.sim || process.env.LEGATUM_SIM || path.join(root, "target/release/legatum-sim"));
const pad = (n) => String(n).padStart(2, "0");
const now = new Date();
const runId = args["run-id"] || `${now.getFullYear()}-${pad(now.getMonth() + 1)}-${pad(now.getDate())}-${pad(now.getHours())}${pad(now.getMinutes())}`;
const runDir = path.resolve(args.out || path.join(root, "docs/sim-results"), `evolve-${runId}`);
const dataRoot = path.join(os.homedir(), "LegatumData", "evolve-standin");
const archiveDir = path.join(dataRoot, "archive");
const work = path.join(dataRoot, runId);
const analysisDir = path.join(work, "analysis");
for (const d of [runDir, path.join(runDir, "checkpoints"), path.join(runDir, "coach"), archiveDir, work, analysisDir]) fs.mkdirSync(d, { recursive: true });

const help = spawnSync(sim, ["--help"], { encoding: "utf8" });
if (help.status !== 0) throw new Error(`cannot run ${sim}: build it with cargo build --release -p legatum-sim`);
const proxyMode = help.stdout.includes("--proxy");
const TIERS = proxyMode ? ["easy", "normal", "hard"] : ["random", "mixed", "scripted"];
const tierOf = (gen) => Math.min(2, Math.floor((gen * 3) / GENS));
const stageArgs = (tier) => (proxyMode ? ["--proxy", "0", "--rotate-proxy", "--difficulty", TIERS[tier]] : ["--brain", TIERS[tier]]);

const round = (x, d = 3) => (x == null || !Number.isFinite(x) ? null : Math.round(x * 10 ** d) / 10 ** d);
const mean = (xs) => (xs.length ? xs.reduce((a, b) => a + b, 0) / xs.length : null);
const quantile = (xs, q) => {
  if (!xs.length) return null;
  const s = [...xs].sort((a, b) => a - b), i = (s.length - 1) * q, lo = Math.floor(i);
  return s[lo] + (s[Math.min(lo + 1, s.length - 1)] - s[lo]) * (i - lo);
};
const entropy = (counts) => {
  const total = Object.values(counts).reduce((a, b) => a + b, 0);
  if (!total) return 0;
  const h = -Object.values(counts).filter(Boolean).reduce((a, c) => a + (c / total) * Math.log2(c / total), 0);
  return h / Math.log2(ARCHETYPE_ORDER.length);
};
const writeAtomic = (file, text) => {
  fs.writeFileSync(`${file}.tmp`, text);
  fs.renameSync(`${file}.tmp`, file);
};
const appendLine = (file, obj) => fs.appendFileSync(file, JSON.stringify(obj) + "\n");
const genSpan = (bs) => (bs.length === 1 ? `generation ${bs[0].name.slice(1)}` : `generations ${bs[0].name.slice(1)}–${bs.at(-1).name.slice(1)}`);
const log = (...m) => console.error(`[fake-run ${new Date().toISOString().slice(11, 19)}]`, ...m);

// ---------- batches ----------

function readIndex() {
  const file = path.join(archiveDir, "index.jsonl");
  if (!fs.existsSync(file)) return [];
  return fs.readFileSync(file, "utf8").split("\n").filter(Boolean).flatMap((l) => {
    try { return [JSON.parse(l)]; } catch { return []; }
  });
}

/** Run (or reuse) one batch; returns its games with archive paths, telemetry-derived seats and classes. */
function batch(name, tier, seed, games) {
  const dir = path.join(work, "batches", name);
  const done = path.join(dir, "batch.json");
  if (fs.existsSync(done)) return JSON.parse(fs.readFileSync(done, "utf8"));
  fs.mkdirSync(dir, { recursive: true });
  const started = new Date().toISOString();
  const t0 = Date.now();
  const cmd = [...stageArgs(tier), "--games", String(games), "--seed", String(seed), "--archive", "--archive-dir", archiveDir, "--label", `standin-${name}`, "--out-dir", dir];
  log(`${name}: ${path.basename(sim)} ${cmd.join(" ")}`);
  const r = spawnSync(sim, cmd, { encoding: "utf8", stdio: ["ignore", "ignore", "pipe"] });
  if (r.status !== 0) throw new Error(`${name} failed: ${r.stderr.slice(-2000)}`);
  const wall_s = (Date.now() - t0) / 1000;
  const reportFile = fs.readdirSync(dir).find((f) => f.endsWith(".json") && f !== "batch.json");
  const report = JSON.parse(fs.readFileSync(path.join(dir, reportFile), "utf8"));
  const index = readIndex().filter((m) => m.created_at >= started.slice(0, 19));
  const out = report.games.map((g) => {
    const meta = index.filter((m) => m.seed === String(g.seed)).at(-1);
    if (!meta) throw new Error(`${name}: seed ${g.seed} missing from archive index`);
    const [y, mo] = meta.created_at.split("-");
    const archive_path = path.join(archiveDir, y, mo, `${meta.game_id}.tape.json.zst`);
    const rec = readArchive(archive_path);
    const classes = classifyGame(rec.telemetry, g.turns);
    const snap150 = snapshotAt(rec.telemetry, 150);
    return {
      game_id: meta.game_id,
      archive_path,
      seed: String(g.seed),
      turns: g.turns,
      outcome: g.outcome,
      winners: g.winner_players,
      eliminations: g.eliminations,
      proxy_seat: g.players.find((p) => p.brain === "proxy")?.id ?? null,
      reached_150: g.turns >= 150,
      players: g.players.map((p) => ({
        id: p.id,
        faction: p.faction,
        brain: p.brain,
        difficulty: p.difficulty,
        vp: p.vp,
        alive: p.alive,
        cities: p.cities,
        alive_t150: snap150 ? !!snap150.players.find((x) => x.player === p.id)?.alive : p.alive,
        cities_t150: snap150 ? snap150.players.find((x) => x.player === p.id)?.cities ?? 0 : null,
        ...classes[p.id],
      })),
    };
  });
  // Batch-level incident rates from the detector (proxy runs always analyze; others may not).
  const rates = {};
  for (const r of report.ai_analysis?.rates || []) {
    if (r.attacker_faction != null) continue;
    const x = (rates[r.behavior] ||= { incidents: 0, opportunities: 0 });
    x.incidents += r.incidents;
    x.opportunities += r.opportunities;
  }
  const result = { name, tier, tier_name: TIERS[tier], seed, wall_s, command: [path.basename(sim), ...cmd], games: out, incident_rates: rates };
  writeAtomic(done, JSON.stringify(result));
  return result;
}

// ---------- per-generation metrics ----------

function candidates(b) {
  return b.games.flatMap((g) => {
    const maxVp = Math.max(1, ...g.players.map((p) => p.vp));
    return g.players.filter((p) => p.id !== g.proxy_seat).map((p) => ({
      id: `${b.name}-s${g.seed}-p${p.id}`,
      game: g,
      player: p,
      fitness: round(p.vp / maxVp + (g.winners.includes(p.id) ? 0.25 : 0)),
      won: g.winners.includes(p.id),
    }));
  });
}
const shareOf = (cands) => {
  const counts = Object.fromEntries(ARCHETYPE_ORDER.map((a) => [a, 0]));
  for (const c of cands) counts[c.player.archetype]++;
  return Object.fromEntries(ARCHETYPE_ORDER.map((a) => [a, round(counts[a] / Math.max(1, cands.length))]));
};
const proxyWinRate = (games) => (proxyMode ? round(games.filter((g) => g.winners.includes(g.proxy_seat)).length / Math.max(1, games.length)) : null);

function genRecord(gen, b) {
  const cands = candidates(b);
  const fit = cands.map((c) => c.fitness);
  const best = cands.reduce((a, c) => (c.fitness > a.fitness ? c : a), cands[0]);
  const aiWins = b.games.filter((g) => g.winners.some((w) => w !== g.proxy_seat)).length;
  const byFaction = {};
  for (const c of cands) {
    const f = (byFaction[c.player.faction] ||= { w: 0, n: 0 });
    f.n++;
    if (c.won) f.w++;
  }
  const smoothed = Object.values(byFaction).map((f) => (f.w + 1) / (f.n + 2));
  const winnerArch = {};
  for (const g of b.games) for (const w of g.winners) {
    const a = g.players.find((p) => p.id === w).archetype;
    winnerArch[a] = (winnerArch[a] || 0) + 1;
  }
  const elites = {};
  for (const c of cands) {
    const a = c.player.archetype;
    if (!elites[a] || c.fitness > elites[a].fitness) elites[a] = { id: c.id, fitness: c.fitness };
  }
  return {
    gen,
    t: new Date().toISOString(),
    wall_s: round(b.wall_s, 1),
    best: {
      id: best.id,
      fitness: best.fitness,
      proxy_win_rate: proxyWinRate(b.games),
      league_win_rate: round(aiWins / b.games.length),
      faction_ratio: round(Math.max(...smoothed) / Math.min(...smoothed), 2),
      archetype: best.player.archetype,
    },
    mean_fitness: round(mean(fit)),
    fitness_p10: round(quantile(fit, 0.1)),
    fitness_p90: round(quantile(fit, 0.9)),
    sigma: null,
    archetype_share: shareOf(cands),
    winner_entropy: round(entropy(winnerArch)),
    elites,
    genome_best: { difficulty_tier: b.tier },
    gene_mean: { difficulty_tier: b.tier },
    gene_spread: { difficulty_tier: 0 },
  };
}

// ---------- checkpoints and the stand-in coach ----------

function benchmark(batches) {
  const games = batches.flatMap((b) => b.games);
  const cands = batches.flatMap(candidates);
  const rate = (key) => {
    const acc = {};
    for (const c of cands) {
      const x = (acc[key(c)] ||= { w: 0, n: 0 });
      x.n++;
      if (c.won) x.w++;
    }
    return Object.fromEntries(Object.entries(acc).sort().map(([k, v]) => [k, round(v.w / v.n)]));
  };
  const present = ARCHETYPE_ORDER.filter((a) => cands.some((c) => c.player.archetype === a));
  const ahead = present.map(() => present.map(() => 0));
  const pairs = present.map(() => present.map(() => 0));
  for (const g of games) {
    const ai = g.players.filter((p) => p.id !== g.proxy_seat);
    for (const p of ai) for (const q of ai) {
      if (p === q) continue;
      const i = present.indexOf(p.archetype), j = present.indexOf(q.archetype);
      if (i === j) continue;
      pairs[i][j]++;
      ahead[i][j] += p.vp > q.vp ? 1 : p.vp === q.vp ? 0.5 : 0;
    }
  }
  const t150 = games.filter((g) => g.reached_150);
  const aiAt150 = t150.flatMap((g) => g.players.filter((p) => p.id !== g.proxy_seat));
  return {
    proxy_win_rate: proxyWinRate(games),
    ai_survival_t150: aiAt150.length ? round(aiAt150.filter((p) => p.alive_t150).length / aiAt150.length) : null,
    faction_win_rates: rate((c) => c.player.faction),
    archetype_win_rates: rate((c) => c.player.archetype),
    matchup_matrix: {
      archetypes: present,
      ahead_rate: ahead.map((row, i) => row.map((v, j) => (pairs[i][j] ? round(v / pairs[i][j]) : null))),
      pairs,
      definition: "ahead_rate[i][j]: in games where archetypes i and j both appear among the AI seats, how often i finishes with more VP than j (ties count half)",
    },
    sample: { gens: batches.map((b) => Number(b.name.slice(1))).filter(Number.isFinite), game_ids: games.map((g) => g.game_id) },
  };
}

const describe = (g, p) => `${p.faction} (seat ${p.id + 1}, ${p.difficulty}) at turn ${p.class_turn}: ${p.f_cities} cities, army ${p.f_army}, science ${p.f_science}, gold ${p.f_gold}; classified ${p.archetype}`;

function showcaseTypical(b, why) {
  const cands = candidates(b);
  const best = cands.reduce((a, c) => (c.fitness > a.fitness ? c : a), cands[0]);
  return showcaseFor(best.game, best.player, why ?? `Generation ${Number(b.name.slice(1))}'s best candidate (${best.id}, ${best.player.archetype}, fitness ${best.fitness}) against the Proxy`);
}
function showcaseFor(g, p, why) {
  const turns = [{ turn: p.class_turn, player: p.id, note: describe(g, p) }];
  const firstOut = [...(g.eliminations || [])].sort((a, b) => a.turn - b.turn)[0];
  if (firstOut) {
    const q = g.players.find((x) => x.id === firstOut.player);
    turns.push({ turn: firstOut.turn, player: q.id, note: `${q.faction} (seat ${q.id + 1}${q.id === g.proxy_seat ? ", the Proxy" : ""}) is eliminated` });
  }
  const w = g.players.find((x) => g.winners.includes(x.id));
  if (w) turns.push({ turn: g.turns, player: w.id, note: `${w.faction} (seat ${w.id + 1}${w.id === g.proxy_seat ? ", the Proxy" : ""}) wins ${({ vp: "on victory points", elimination: "by conquest", "turn-limit": "at the turn limit" })[g.outcome] || `(${g.outcome})`} with ${w.vp} VP` });
  turns.sort((a, b) => a.turn - b.turn);
  return { game_id: g.game_id, archive_path: g.archive_path, seed: g.seed, why, highlight_turns: turns };
}

function incidentEvidence(game, behavior, limit = 2) {
  const out = path.join(analysisDir, `${game.game_id}.incidents.json`);
  if (!fs.existsSync(out)) {
    const r = spawnSync(sim, ["--replay", game.archive_path, "--analyze", "--out", analysisDir], { encoding: "utf8" });
    if (r.status !== 0) { log(`detector failed on ${game.game_id}: ${r.stderr.slice(-400)}`); return []; }
  }
  const report = JSON.parse(fs.readFileSync(out, "utf8"));
  const seen = new Set();
  return report.incidents
    .filter((i) => i.behavior === behavior && i.player !== game.proxy_seat)
    .filter((i) => (seen.has(i.player) ? false : seen.add(i.player)))
    .slice(0, limit)
    .map((i) => ({ game_id: game.game_id, turn: i.turn, player: i.player, detail: `${i.faction}: ${i.evidence}${i.consecutive_turns > 1 ? ` (${i.consecutive_turns} turns in a row)` : ""} [${i.id}]` }));
}

const BEHAVIOR_WORDS = {
  unanswered_attack: "units that take hits without answering",
  scout_not_fleeing: "scouts that stay next to enemies",
  garrison_under_ranged_fire: "garrisons that sit under ranged fire",
  idle_military_threatened_city: "healthy units idling near a threatened city",
  repeated_bounce: "moves that bounce turn after turn",
  movement_oscillation: "units pacing back and forth",
  gold_hoarded_under_attack: "gold hoarded while under attack",
  settler_wandering: "settlers wandering without founding",
  research_idle: "empty research queues",
  production_idle: "cities with nothing in production",
};

function coach(gen, cur, prev, prevLabel) {
  const pct = (x) => Math.round(x * 100);
  const obs = [];
  const shareNow = shareOf(cur.flatMap(candidates)), shareBefore = shareOf(prev.flatMap(candidates));
  const leader = ARCHETYPE_ORDER.reduce((a, k) => (shareNow[k] > shareNow[a] ? k : a), ARCHETYPE_ORDER[0]);
  for (const a of ARCHETYPE_ORDER) {
    const before = shareBefore[a], after = shareNow[a], d = after - before;
    if (Math.abs(d) < 0.08) continue;
    const kind = d > 0 ? (a === leader && after >= 0.35 ? "dominant" : "emerged") : after < 0.05 ? "disappeared" : "countered";
    if (kind === "countered") continue; // a fall without a known cause is not a counter
    const games = (d > 0 ? cur : prev).flatMap((b) => b.games);
    const evidence = games.flatMap((g) => g.players.filter((p) => p.id !== g.proxy_seat && p.archetype === a).map((p) => ({ game_id: g.game_id, turn: p.class_turn, player: p.id, detail: describe(g, p) }))).slice(0, 3);
    obs.push({ kind, behavior: `${a} play`, archetype: a, evidence, metric_change: { name: `${a} share of candidates`, before: before, after: after }, confidence: Math.abs(d) >= 0.15 ? "medium" : "low", _mag: Math.abs(d) });
  }
  const sumRates = (bs) => {
    const acc = {};
    for (const b of bs) for (const [k, v] of Object.entries(b.incident_rates)) {
      const x = (acc[k] ||= { incidents: 0, opportunities: 0 });
      x.incidents += v.incidents;
      x.opportunities += v.opportunities;
    }
    return acc;
  };
  const rNow = sumRates(cur), rBefore = sumRates(prev);
  for (const k of Object.keys(BEHAVIOR_WORDS)) {
    const a = rBefore[k], b = rNow[k];
    if (!a || !b || a.opportunities < 150 || b.opportunities < 150) continue;
    const before = a.incidents / a.opportunities, after = b.incidents / b.opportunities;
    const rel = before ? (after - before) / before : after ? 1 : 0;
    if (Math.abs(after - before) < 0.01 || Math.abs(rel) < 0.3) continue;
    const kind = after < before ? "disappeared" : "weakness";
    const src = (after < before ? prev : cur).flatMap((bb) => bb.games);
    const evidence = src.slice(0, 3).flatMap((g) => incidentEvidence(g, k)).slice(0, 2);
    obs.push({ kind, behavior: k, evidence, metric_change: { name: `${k} incidents per opportunity`, before: round(before), after: round(after) }, confidence: Math.min(a.opportunities, b.opportunities) >= 1000 ? "medium" : "low", _mag: Math.abs(rel) / 3 });
  }
  obs.sort((x, y) => y._mag - x._mag);
  const observations = obs.slice(0, 5).map(({ _mag, ...o }) => o);
  const pNow = proxyWinRate(cur.flatMap((b) => b.games)), pBefore = proxyWinRate(prev.flatMap((b) => b.games));
  const top = observations[0];
  const topText = !top ? "no behavior changed by enough to report"
    : top.archetype ? `${top.archetype} play went from ${pct(top.metric_change.before)}% to ${pct(top.metric_change.after)}% of candidates`
    : `${BEHAVIOR_WORDS[top.behavior]} went from ${(top.metric_change.before * 100).toFixed(1)}% to ${(top.metric_change.after * 100).toFixed(1)}% of opportunities`;
  const proxyText = pNow == null ? "" : ` The Proxy won ${pct(pNow)}% of games in ${genSpan(cur)} (${pct(pBefore)}% in ${prevLabel}).`;
  const summary = `Stand-in coach (rules over the incident detector and the stand-in archetype classifier, not an LLM). Compared with ${prevLabel}, ${topText}.${proxyText}`;
  return {
    gen,
    observations,
    ...(shareNow[leader] >= 0.4 ? { dominant_strategy: { archetype: leader, share: shareNow[leader], counter_proposal: null } } : {}),
    proposals: [],
    summary,
  };
}

// ---------- main ----------

const files = {
  run: path.join(runDir, "run.json"),
  gens: path.join(runDir, "generations.jsonl"),
  events: path.join(runDir, "events.jsonl"),
};
const existingGens = fs.existsSync(files.gens) ? fs.readFileSync(files.gens, "utf8").split("\n").filter(Boolean).map((l) => JSON.parse(l)) : [];
const doneGens = new Set(existingGens.map((g) => g.gen));
if (!fs.existsSync(files.gens)) fs.writeFileSync(files.gens, "");
if (!fs.existsSync(files.events)) fs.writeFileSync(files.events, "");

const simRevision = (() => {
  try { return execFileSync("git", ["-C", root, "rev-parse", "--short", "HEAD"], { encoding: "utf8" }).trim(); } catch { return null; }
})();
const runJson = {
  run_id: runId,
  started_at: now.toISOString(),
  status: "running",
  standin: {
    note: "STAND-IN RUN, not an evolution. Each generation is one real legatum-sim batch; nothing is optimized. The AI steps through fixed tiers so the dashboard, report and film have real games to show. Archetypes come from a stand-in telemetry classifier (tools/film/lib/standin-archetypes.mjs) until strategy-diversity-metrics lands, and the coach is a rule over the incident detector, not an LLM.",
    generator: "tools/film/fake-run.mjs",
    simulator: path.relative(root, sim).startsWith("..") ? sim : path.relative(root, sim),
    proxy_benchmark: proxyMode,
    stages: [0, 1, 2].map((t) => {
      const gens = [...Array(GENS).keys()].filter((g) => tierOf(g) === t);
      return { tier: t, name: proxyMode ? `${TIERS[t]} AIs vs the Proxy` : `${TIERS[t]} brains`, from_gen: gens[0], to_gen: gens.at(-1) };
    }),
    definitions: {
      candidate: proxyMode ? "one AI seat in one game (7 per game; the Proxy seat is excluded)" : "one seat in one game",
      fitness: "final VP / the game's top VP, plus 0.25 if that seat's team won",
      "best.proxy_win_rate": "share of the generation's games the Proxy won",
      "best.league_win_rate": "share of the generation's games won by an AI seat (there is no league in a stand-in)",
      "best.faction_ratio": "best / worst AI faction win rate in the generation, Laplace-smoothed ((wins+1)/(seats+2))",
      winner_entropy: "Shannon entropy of the winners' archetypes, normalized 0-1 over the archetype set",
      "benchmark window": "each checkpoint's benchmark pools its generation and the two before it (see benchmark.sample)",
    },
    archive_dir: archiveDir,
  },
  config: {
    population: GAMES * (proxyMode ? 7 : 8),
    games_per_candidate: 1,
    speed: SPEED,
    seeds: { first: SEED.toString(), per_generation: GAMES, rule: "generation g plays seeds first + g*per_generation ..., the same seeds for every candidate in it" },
    coach_every: COACH_EVERY,
    genes: [{ name: "difficulty_tier", min: 0, max: 2, default: 1, description: proxyMode ? "Stand-in gene: the AI difficulty (0 Easy, 1 Normal, 2 Hard). The real run tunes decision parameters." : "Stand-in gene: 0 random, 1 mixed, 2 scripted brains." }],
  },
  baseline: null,
  source_revision: simRevision,
};
if (fs.existsSync(files.run)) Object.assign(runJson, { ...JSON.parse(fs.readFileSync(files.run, "utf8")), status: "running" });
writeAtomic(files.run, JSON.stringify(runJson, null, 2) + "\n");

const batches = [];
let bestSoFar = -Infinity, leader = null, leaderStreak = 0, pendingLeader = null;
const everSeen = new Set(), extinct = new Set(), zeroStreak = {};
const events = fs.readFileSync(files.events, "utf8").split("\n").filter(Boolean).map((l) => JSON.parse(l));
const emit = (e) => {
  if (events.some((x) => x.gen === e.gen && x.kind === e.kind && x.detail === e.detail)) return;
  events.push(e);
  appendLine(files.events, e);
};
let prevCheckpoint = 0;

for (let gen = 0; gen < GENS; gen++) {
  const b = batch(`g${gen}`, tierOf(gen), seedAt(gen * GAMES), GAMES);
  batches.push(b);
  const rec = existingGens.find((g) => g.gen === gen) || genRecord(gen, b);
  if (!doneGens.has(gen)) appendLine(files.gens, rec);
  if (gen === 0) {
    const t150 = b.games.filter((g) => g.reached_150);
    runJson.baseline = {
      proxy_win_rate: rec.best.proxy_win_rate,
      ai_cities_t150: t150.length ? round(mean(t150.map((g) => g.players.filter((p) => p.id !== g.proxy_seat).reduce((a, p) => a + p.cities_t150, 0))), 1) : null,
      archetype_entropy: round(entropy(Object.fromEntries(Object.entries(rec.archetype_share).map(([k, v]) => [k, v * 1000])))),
    };
    writeAtomic(files.run, JSON.stringify(runJson, null, 2) + "\n");
  }
  // events
  const t = rec.t;
  if (rec.best.fitness > bestSoFar) {
    if (gen > 0) {
      const g = b.games.find((x) => rec.best.id.includes(`-s${x.seed}-`));
      emit({ gen, t, kind: "new_best", detail: `New best fitness ${rec.best.fitness} (was ${bestSoFar}): ${rec.best.id}, ${rec.best.archetype}`, game_id: g?.game_id });
    }
    bestSoFar = rec.best.fitness;
  }
  const share = rec.archetype_share;
  const top = ARCHETYPE_ORDER.reduce((a, k) => (share[k] > share[a] ? k : a), ARCHETYPE_ORDER[0]);
  if (top === pendingLeader) leaderStreak++;
  else { pendingLeader = top; leaderStreak = 1; }
  if (leader == null && gen === 0) leader = top;
  else if (top !== leader && leaderStreak >= 2 && share[top] >= 0.25) {
    emit({ gen, t, kind: "archetype_takeover", detail: `${top} became the most common strategy (${Math.round(share[top] * 100)}% of candidates in generation ${gen}), taking over from ${leader}` });
    leader = top;
  }
  for (const a of ARCHETYPE_ORDER) {
    if (share[a] >= 0.05) { everSeen.add(a); extinct.delete(a); }
    zeroStreak[a] = share[a] === 0 ? (zeroStreak[a] || 0) + 1 : 0;
    if (everSeen.has(a) && !extinct.has(a) && zeroStreak[a] >= 2) {
      emit({ gen, t, kind: "archetype_extinct", detail: `No candidate played ${a} in generations ${gen - 1} and ${gen}` });
      extinct.add(a);
    }
  }
  for (const g of b.games) if (g.outcome === "stalled") emit({ gen, t, kind: "stall", detail: `Game hit the safety cap without a winner`, game_id: g.game_id });

  const isCheckpoint = gen === 0 || (gen % COACH_EVERY === 0) || gen === GENS - 1;
  if (!isCheckpoint) continue;
  const window = batches.slice(Math.max(0, gen - 2), gen + 1);
  const cp = { gen, genome: rec.genome_best, benchmark: benchmark(window), showcase: [showcaseTypical(b)] };
  if (gen > 0) {
    const prevWindow = batches.slice(Math.max(0, prevCheckpoint - 2), prevCheckpoint + 1);
    const prevLabel = genSpan(prevWindow);
    const c = coach(gen, window, prevWindow, prevLabel);
    // A second showcase: a game that shows the behavior the coach saw emerge, if any.
    const rise = c.observations.find((o) => o.kind !== "disappeared" && o.kind !== "weakness" && o.evidence.length);
    if (rise) {
      const ev = rise.evidence[0];
      const g = window.flatMap((x) => x.games).find((x) => x.game_id === ev.game_id);
      cp.showcase.push(showcaseFor(g, g.players.find((p) => p.id === ev.player), `Shows ${rise.behavior}, which went from ${Math.round(rise.metric_change.before * 100)}% to ${Math.round(rise.metric_change.after * 100)}% of candidates since ${prevLabel}`));
    }
    writeAtomic(path.join(runDir, "coach", `${gen}.json`), JSON.stringify(c, null, 2) + "\n");
    prevCheckpoint = gen;
  }
  if (gen === GENS - 1 && gen > 0) {
    // Before/after on the same map: replay generation 0's typical seed with the final tier.
    const base = showcaseTypical(batches[0]);
    const i = batches[0].games.findIndex((g) => g.game_id === base.game_id);
    const re = batch("rematch", tierOf(gen), seedAt(0), i + 1);
    const g = re.games[i];
    const star = g.players.filter((p) => p.id !== g.proxy_seat).sort((a, b) => b.vp - a.vp)[0];
    cp.showcase.push(showcaseFor(g, star, `Same seed (${g.seed}) and map as generation 0's typical game ${base.game_id}, replayed by generation ${gen}'s AIs`));
  }
  writeAtomic(path.join(runDir, "checkpoints", `${gen}.json`), JSON.stringify(cp, null, 2) + "\n");
  log(`checkpoint ${gen} written`);
}

runJson.status = "finished";
runJson.finished_at = new Date().toISOString();
writeAtomic(files.run, JSON.stringify(runJson, null, 2) + "\n");

// A short digest; the morning report (tools/film/report.mjs) is the full write-up.
const gens = fs.readFileSync(files.gens, "utf8").split("\n").filter(Boolean).map((l) => JSON.parse(l));
const day = runJson.finished_at.slice(0, 10);
const lines = [
  `# Evolution digest ${runId} (stand-in)`,
  "",
  runJson.standin.note,
  "",
  `- Generations: ${gens.length} (${gens[0].gen}–${gens.at(-1).gen}), ${GAMES} games each, ${SPEED} speed.`,
  proxyMode ? `- Proxy win rate: ${gens[0].best.proxy_win_rate} at generation 0, ${gens.at(-1).best.proxy_win_rate} at generation ${gens.at(-1).gen}.` : "- No Proxy benchmark in this simulator build.",
  `- Best fitness: ${gens[0].best.fitness} at generation 0, ${Math.max(...gens.map((g) => g.best.fitness))} at best.`,
  `- Events: ${events.length} (${[...new Set(events.map((e) => e.kind))].join(", ") || "none"}).`,
  "",
  "See report.html in this directory for the full morning report.",
];
writeAtomic(path.join(runDir, `digest-${day}.md`), lines.join("\n") + "\n");
log(`finished: ${runDir}`);
console.log(runDir);
