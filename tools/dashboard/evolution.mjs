// Evolution panel data. Reads only the run log (docs/archive/ai-evolution-log.md) through tools/film's
// reader. Everything here runs in the dashboard's 15-second background refresh; requests only
// read the cached result. Showcase and evidence games are exported to the replay viewer in the
// same background pass (a few per cycle), so "watch" links open the existing Replays section.
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { execFile } from "node:child_process";
import { promisify } from "node:util";
import { listRuns, loadRun, runSignature, eras, matrixOf, archetypesOf, archetypeColor } from "../film/lib/runlog.mjs";
import { readArchive } from "../film/lib/archive.mjs";

const execFileAsync = promisify(execFile);
const MAX_POINTS = 400;
const EXPORTS_PER_CYCLE = 3;
export const safeRunId = (id) => /^[A-Za-z0-9_.-]+$/.test(id || "") && !id.includes("..");
export const replayFileFor = (gameId, seed) => `evo-${String(gameId).replace(/[^A-Za-z0-9_-]/g, "-")}-seed-${seed}.json`;

function downsample(gens) {
  if (gens.length <= MAX_POINTS) return gens;
  const step = gens.length / MAX_POINTS;
  const out = [];
  for (let i = 0; i < MAX_POINTS; i++) out.push(gens[Math.floor(i * step)]);
  if (out.at(-1) !== gens.at(-1)) out.push(gens.at(-1));
  return out;
}

export function createEvolution({ root, replayDir, resultsDir = path.join(root, "docs/sim-results"), filmsDir = process.env.LEGATUM_FILMS || path.join(os.homedir(), "LegatumData", "films"), sim = process.env.LEGATUM_SIM || path.join(root, "target/release/legatum-sim"), cacheDir = path.join(root, ".progress", "evolution-cache") } = {}) {
  const cache = new Map(); // run id -> { sig, value }
  let list = [];
  const archivePaths = new Map(); // game id -> archive file
  const failed = new Set();

  const replayIndex = () => {
    try { return new Set(JSON.parse(fs.readFileSync(path.join(replayDir, "index.json"), "utf8")).map((r) => r.file)); } catch { return new Set(); }
  };

  function findArchive(gameId, roots) {
    if (archivePaths.has(gameId)) return archivePaths.get(gameId);
    for (const r of roots) {
      if (!fs.existsSync(r)) continue;
      for (const y of fs.readdirSync(r).filter((d) => /^\d{4}$/.test(d))) for (const m of fs.readdirSync(path.join(r, y)).filter((d) => /^\d{2}$/.test(d))) {
        for (const ext of [".tape.json.zst", ".tape.json.gz", ".tape.json"]) {
          const f = path.join(r, y, m, gameId + ext);
          if (fs.existsSync(f)) { archivePaths.set(gameId, f); return f; }
        }
      }
    }
    return null;
  }

  function build(data, exported) {
    const { run, generations, events, checkpoints, coach } = data;
    const names = archetypesOf(generations);
    const showcaseById = new Map();
    for (const c of checkpoints) for (const s of c.showcase || []) showcaseById.set(s.game_id, s);
    const seedOf = (id) => showcaseById.get(id)?.seed ?? data._seeds?.get(id) ?? null;
    const watch = (gameId, turn) => {
      if (!gameId) return null;
      const seed = seedOf(gameId);
      const file = seed != null ? replayFileFor(gameId, seed) : null;
      return file && exported.has(file) ? `/#replay=${encodeURIComponent(file)}&turn=${turn ?? 1}` : null;
    };
    const timeline = [
      ...coach.map((c) => ({
        type: "coach", gen: c.gen, summary: c.summary || "",
        dominant_strategy: c.dominant_strategy || null,
        proposals: (c.proposals || []).length,
        observations: (c.observations || []).map((o) => ({
          kind: o.kind, behavior: o.behavior, archetype: o.archetype || null, confidence: o.confidence ?? null,
          metric_change: o.metric_change || null,
          evidence: (o.evidence || []).map((e) => ({ ...e, watch: watch(e.game_id, e.turn) })),
        })),
      })),
      ...events.map((e) => ({ type: "event", gen: e.gen, t: e.t, kind: e.kind, detail: e.detail, game_id: e.game_id || null, watch: watch(e.game_id, 1) })),
      ...checkpoints.map((c) => ({
        type: "checkpoint", gen: c.gen,
        proxy_win_rate: c.benchmark?.proxy_win_rate ?? null,
        ai_survival_t150: c.benchmark?.ai_survival_t150 ?? null,
        showcase: (c.showcase || []).map((s) => ({
          game_id: s.game_id, seed: s.seed, why: s.why,
          watch: watch(s.game_id, s.highlight_turns?.[0]?.turn ?? 1),
          highlights: (s.highlight_turns || []).map((h) => ({ ...h, watch: watch(s.game_id, h.turn) })),
        })),
      })),
    ].sort((a, b) => b.gen - a.gen || ["coach", "checkpoint", "event"].indexOf(a.type) - ["coach", "checkpoint", "event"].indexOf(b.type));
    const lastCp = checkpoints.at(-1), firstCp = checkpoints[0];
    const filmBase = path.join(filmsDir, data.id);
    const film = fs.existsSync(`${filmBase}.mp4`) ? {
      url: `/films/${encodeURIComponent(data.id)}.mp4?v=${Math.floor(fs.statSync(`${filmBase}.mp4`).mtimeMs)}`,
      vtt: fs.existsSync(`${filmBase}.vtt`) ? `/films/${encodeURIComponent(data.id)}.vtt` : null,
      srt: fs.existsSync(`${filmBase}.srt`) ? `/films/${encodeURIComponent(data.id)}.srt` : null,
      poster: fs.existsSync(`${filmBase}.jpg`) ? `/films/${encodeURIComponent(data.id)}.jpg` : null,
      story: fs.existsSync(`${filmBase}.story.md`) ? `/films/${encodeURIComponent(data.id)}.story.md` : null,
      made_at: fs.statSync(`${filmBase}.mp4`).mtime.toISOString(),
    } : null;
    const report = path.join(data.dir, "report.html");
    return {
      id: data.id,
      dir: path.relative(root, data.dir),
      status: run.status,
      started_at: run.started_at,
      finished_at: run.finished_at || null,
      standin: run.standin ? { note: run.standin.note, stages: run.standin.stages || [] } : null,
      config: { population: run.config?.population, speed: run.config?.speed, coach_every: run.config?.coach_every, games_per_candidate: run.config?.games_per_candidate, genes: (run.config?.genes || []).length },
      baseline: run.baseline || null,
      archetypes: names.map((n) => ({ name: n, color: archetypeColor(n) })),
      gens: downsample(generations).map((g) => ({ gen: g.gen, best: g.best?.fitness ?? null, pwr: g.best?.proxy_win_rate ?? null, mean: g.mean_fitness ?? null, p10: g.fitness_p10 ?? null, p90: g.fitness_p90 ?? null, share: g.archetype_share || {} })),
      latest: generations.at(-1) ? { gen: generations.at(-1).gen, t: generations.at(-1).t, best: generations.at(-1).best, mean_fitness: generations.at(-1).mean_fitness, winner_entropy: generations.at(-1).winner_entropy } : null,
      eras: eras(generations),
      timeline,
      matrix: lastCp?.benchmark?.matchup_matrix ? { gen: lastCp.gen, ...matrixOf(lastCp.benchmark.matchup_matrix) } : null,
      benchmark: lastCp?.benchmark ? { gen: lastCp.gen, before_gen: firstCp !== lastCp ? firstCp.gen : null, before: firstCp !== lastCp ? { proxy_win_rate: firstCp.benchmark?.proxy_win_rate ?? null, ai_survival_t150: firstCp.benchmark?.ai_survival_t150 ?? null } : null, proxy_win_rate: lastCp.benchmark.proxy_win_rate ?? null, ai_survival_t150: lastCp.benchmark.ai_survival_t150 ?? null } : null,
      report: fs.existsSync(report) ? `/evolution/${encodeURIComponent(data.id)}/report.html?v=${Math.floor(fs.statSync(report).mtimeMs)}` : null,
      film,
      watch_note: fs.existsSync(sim) ? null : "Build legatum-sim (cargo build --release -p legatum-sim) to turn on watch links.",
    };
  }

  // Export referenced games to the replay viewer, a few per refresh.
  async function exportReplays(data, exported) {
    if (!fs.existsSync(sim)) return false;
    const wanted = [];
    const roots = new Set([process.env.LEGATUM_ARCHIVE || path.join(os.homedir(), "LegatumData", "archive")]);
    for (const c of [...data.checkpoints].reverse()) for (const s of c.showcase || []) {
      if (s.archive_path) {
        archivePaths.set(s.game_id, s.archive_path);
        roots.add(path.dirname(path.dirname(path.dirname(s.archive_path))));
      }
      wanted.push(s.game_id);
    }
    for (const c of [...data.coach].reverse()) for (const o of c.observations || []) for (const e of o.evidence || []) wanted.push(e.game_id);
    for (const e of [...data.events].reverse()) if (e.game_id) wanted.push(e.game_id);
    data._seeds ||= new Map();
    let did = 0, changed = false;
    for (const id of [...new Set(wanted)]) {
      if (did >= EXPORTS_PER_CYCLE || failed.has(id)) continue;
      const file = findArchive(id, [...roots]);
      if (!file) continue;
      let seed = data._seeds.get(id);
      if (seed != null && exported.has(replayFileFor(id, seed))) continue;
      did++;
      try {
        const rec = readArchive(file);
        seed = String(rec.metadata.seed);
        data._seeds.set(id, seed);
        if (exported.has(replayFileFor(id, seed))) continue;
        fs.mkdirSync(path.join(cacheDir, "tapes"), { recursive: true });
        const tape = path.join(cacheDir, "tapes", `${id}.json`);
        fs.writeFileSync(tape, JSON.stringify(rec.tape));
        const label = `evo-${id.replace(/[^A-Za-z0-9_-]/g, "-")}`;
        await execFileAsync(sim, ["--replay-export", tape, "--out", replayDir, "--label", label], { timeout: 120e3 });
        exported.add(replayFileFor(id, seed));
        changed = true;
      } catch (e) {
        failed.add(id);
        console.error("evolution replay export", id, e.message);
      }
    }
    return changed;
  }

  let seedsById = new Map();
  async function refresh() {
    list = listRuns(resultsDir).slice(0, 6);
    const exported = replayIndex();
    for (const dir of list) {
      const id = path.basename(dir).replace(/^evolve-/, "");
      let sig;
      try { sig = runSignature(dir); } catch { continue; }
      const prev = cache.get(id);
      const films = ["mp4", "vtt", "jpg", "story.md"].map((x) => { try { return Math.floor(fs.statSync(path.join(filmsDir, `${id}.${x}`)).mtimeMs); } catch { return 0; } }).join(",");
      const key = `${sig}#${films}#${exported.size}`;
      if (prev && prev.key === key && dir !== list[0]) continue;
      try {
        const data = prev?.data && prev.sig === sig ? prev.data : loadRun(dir);
        data._seeds = seedsById;
        if (dir === list[0]) await exportReplays(data, exported);
        cache.set(id, { sig, key: `${sig}#${films}#${exported.size}`, data, value: build(data, exported) });
      } catch (e) {
        console.error("evolution run", dir, e.message);
      }
    }
  }

  function state(runId) {
    const ids = list.map((d) => path.basename(d).replace(/^evolve-/, ""));
    const id = runId && ids.includes(runId) ? runId : ids[0];
    return { runs: ids.map((i) => ({ id: i, standin: !!cache.get(i)?.value?.standin, status: cache.get(i)?.value?.status ?? null })), run: id ? cache.get(id)?.value ?? null : null };
  }

  return { refresh, state, filmsDir, resultsDir };
}
