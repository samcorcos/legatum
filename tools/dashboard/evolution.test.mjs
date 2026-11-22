// Evolution panel API: reads a fixture run log and serves the film with byte ranges (Safari).
import assert from "node:assert/strict";
import { after, before, test } from "node:test";
import { Readable, Writable } from "node:stream";
import { once } from "node:events";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { createServer } from "./server.mjs";
import { createEvolution } from "./evolution.mjs";

let tmp, server;
const gen = (g, share, pwr) => ({ gen: g, t: `2026-09-27T00:0${g}:00Z`, wall_s: 1, best: { id: `c${g}`, fitness: 0.5 + g / 10, proxy_win_rate: pwr, league_win_rate: 0.5, faction_ratio: 1.2, archetype: "Rush" }, mean_fitness: 0.4, fitness_p10: 0.1, fitness_p90: 0.8, sigma: 0.3, archetype_share: share, winner_entropy: 0.7, elites: {}, genome_best: {}, gene_mean: {}, gene_spread: {} });

before(async () => {
  tmp = fs.mkdtempSync(path.join(os.tmpdir(), "legatum-evo-test-"));
  const run = path.join(tmp, "results", "evolve-2026-09-27-0100");
  fs.mkdirSync(path.join(run, "checkpoints"), { recursive: true });
  fs.mkdirSync(path.join(run, "coach"), { recursive: true });
  fs.writeFileSync(path.join(run, "run.json"), JSON.stringify({ run_id: "2026-09-27-0100", started_at: "2026-09-27T00:00:00Z", status: "running", config: { population: 16, games_per_candidate: 200, speed: "standard", coach_every: 2, genes: [] }, baseline: { proxy_win_rate: 0.6 } }));
  fs.writeFileSync(path.join(run, "generations.jsonl"), [gen(0, { Rush: 0.7, Wide: 0.3 }, 0.6), gen(1, { Rush: 0.5, Wide: 0.5 }, 0.5), gen(2, { Rush: 0.2, Wide: 0.8 }, 0.3)].map((g) => JSON.stringify(g)).join("\n") + "\n{\"gen\": 3, \"t");
  fs.writeFileSync(path.join(run, "events.jsonl"), JSON.stringify({ gen: 2, t: "x", kind: "archetype_takeover", detail: "Wide took over" }) + "\n");
  fs.writeFileSync(path.join(run, "checkpoints", "2.json"), JSON.stringify({ gen: 2, genome: {}, benchmark: { proxy_win_rate: 0.3, ai_survival_t150: 0.8, faction_win_rates: {}, archetype_win_rates: {}, matchup_matrix: { Rush: { Wide: 0.4 }, Wide: { Rush: 0.6 } } }, showcase: [{ game_id: "g1", archive_path: "/nonexistent/g1.tape.json.zst", seed: "5", why: "typical", highlight_turns: [{ turn: 10, player: 1, note: "n" }] }] }));
  fs.writeFileSync(path.join(run, "coach", "2.json"), JSON.stringify({ gen: 2, observations: [{ kind: "emerged", behavior: "Wide play", archetype: "Wide", evidence: [{ game_id: "g1", turn: 10, player: 1, detail: "d" }], metric_change: { name: "Wide share", before: 0.3, after: 0.8 }, confidence: "high" }], proposals: [], summary: "Wide spread." }));
  fs.mkdirSync(path.join(tmp, "films"));
  fs.writeFileSync(path.join(tmp, "films", "2026-09-27-0100.mp4"), Buffer.alloc(1000, 7));
  const evo = createEvolution({ root: tmp, replayDir: path.join(tmp, "replays"), resultsDir: path.join(tmp, "results"), filmsDir: path.join(tmp, "films"), sim: path.join(tmp, "no-sim") });
  await evo.refresh();
  server = createServer({ replayDir: path.join(tmp, "replays"), evo });
});
after(() => { if (tmp) fs.rmSync(tmp, { recursive: true, force: true }); });

async function get(url, headers = {}) {
  const req = Readable.from([]);
  req.url = url; req.method = "GET"; req.headers = headers;
  let status = 0, head = {};
  const chunks = [];
  const res = new Writable({ write(chunk, _e, done) { chunks.push(Buffer.from(chunk)); done(); } });
  res.writeHead = (code, fields = {}) => { status = code; head = fields; };
  const finished = once(res, "finish");
  server.emit("request", req, res);
  await finished;
  return { status, headers: head, body: Buffer.concat(chunks) };
}

test("the panel API summarizes the newest run and skips a torn last line", async () => {
  const r = await get("/api/evolution");
  assert.equal(r.status, 200);
  const { runs, run } = JSON.parse(r.body);
  assert.deepEqual(runs.map((x) => x.id), ["2026-09-27-0100"]);
  assert.equal(run.gens.length, 3);
  assert.equal(run.latest.gen, 2);
  assert.deepEqual(run.archetypes.map((a) => a.name).sort(), ["Rush", "Wide"]);
  assert.deepEqual(run.matrix.names, ["Rush", "Wide"]);
  assert.equal(run.matrix.rate[0][1], 0.4);
  const coach = run.timeline.find((t) => t.type === "coach");
  assert.equal(coach.observations[0].evidence[0].watch, null); // no simulator, no replay yet
  assert.ok(run.film.url.startsWith("/films/2026-09-27-0100.mp4"));
  assert.ok(run.eras.length >= 1);
});

test("the film is served with byte ranges and unsafe names are refused", async () => {
  const r = await get("/films/2026-09-27-0100.mp4", { range: "bytes=10-19" });
  assert.equal(r.status, 206);
  assert.equal(r.headers["content-range"], "bytes 10-19/1000");
  assert.equal(r.body.length, 10);
  assert.equal((await get("/films/2026-09-27-0100.mp4")).status, 200);
  assert.equal((await get("/films/..%2Fsecret.mp4")).status, 404);
  assert.equal((await get("/evolution/../report.html")).status, 404);
});
