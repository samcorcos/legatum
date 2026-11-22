// Exercise the actual HTTP handler without opening a port (works in restricted sandboxes).
import assert from "node:assert/strict";
import { after, before, test } from "node:test";
import { Readable, Writable } from "node:stream";
import { once } from "node:events";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { createServer } from "./server.mjs";

const fixture = {
  version: 1,
  header: {
    label: "test", seed: "18446744073709551615", width: 2, height: 2,
    terrain: [0, 0, 0, 0], terrain_kinds: ["Plains"],
    players: [
      { id: 0, name: "Dawn", faction: "Japan", team: 0, color_index: 0 },
      { id: 1, name: "Dusk", faction: "Byzantium", team: 1, color_index: 1 },
    ],
    winner: null, outcome: "unfinished", turns: 1, targets: { 0: 10, 1: 10 },
  },
  frames: [{
    turn: 1, owners: [[0, 1], [-1, 2], [1, 1]],
    cities: [{ id: 0, owner: 0, name: "Dawn", pos: [0, 0], pop: 1, capital: true }],
    units: [{ pos: [1, 1], owner: 1, kinds: { Soldier: 1 }, strength: 20 }],
    vp: [
      { player: 0, total: 1, sources: { "Largest Economy": 1 }, alive: true },
      { player: 1, total: 0, sources: {}, alive: true },
    ],
    titles: { LargestEconomy: 0, StrongestArmy: null },
    events: [{ turn: 1, text: "Dawn founded Dawn" }],
  }],
};
const archive = [{
  file: "test-replay.json", label: fixture.header.label, seed: fixture.header.seed,
  turns: 1, winner: null, outcome: "unfinished", why_interesting: "Tiny test fixture",
}];
let replayDir, server;

before(() => {
  replayDir = fs.mkdtempSync(path.join(os.tmpdir(), "legatum-dashboard-test-"));
  fs.writeFileSync(path.join(replayDir, "index.json"), JSON.stringify(archive));
  fs.writeFileSync(path.join(replayDir, archive[0].file), JSON.stringify(fixture));
  server = createServer({ replayDir });
});
after(() => {
  if (replayDir) fs.rmSync(replayDir, { recursive: true, force: true });
});

async function get(url) {
  const req = Readable.from([]);
  req.url = url; req.method = "GET";
  let status = 0, headers = {}, body = "";
  const res = new Writable({ write(chunk, _encoding, done) { body += chunk; done(); } });
  res.writeHead = (code, fields = {}) => { status = code; headers = fields; };
  const finished = once(res, "finish");
  server.emit("request", req, res);
  await finished;
  return { status, headers, body };
}

test("dashboard and isolated replay assets are served with correct content types", async () => {
  for (const [route, type] of [["/", "text/html"], ["/replay.js", "text/javascript"], ["/replay.css", "text/css"], ["/favicon.svg", "image/svg+xml"], ["/brand.css", "text/css"], ["/wordmark.svg", "image/svg+xml"]]) {
    const res = await get(route);
    assert.equal(res.status, 200);
    assert.ok(res.headers["content-type"].startsWith(type));
    if (route === "/brand.css") {
      assert.ok(res.body.includes("data:font/woff2;base64,"));
      assert.ok(res.body.includes("'Bodoni Moda'"));
      assert.ok(res.body.includes("'Josefin Sans'"));
      assert.ok(!res.body.includes("fonts.googleapis.com"));
    }
    if (route === "/") {
      for (const id of ["replays", "replay-map", "replay-game", "sim", "inbox", "chart", "agents"]) assert.ok(res.body.includes(`id="${id}"`));
    }
  }
});

test("archive and every replay parse; frames, owners, positions, scores, size and events agree", async () => {
  const response = await get("/replays/index.json");
  assert.equal(response.status, 200);
  assert.ok(response.headers["content-type"].startsWith("application/json"));
  const index = JSON.parse(response.body);
  assert.deepEqual(index, archive);
  for (const entry of index) {
    const response = await get(`/replays/${entry.file}`);
    assert.equal(response.status, 200);
    assert.ok(Buffer.byteLength(response.body) < 3_000_000, entry.file);
    const replay = JSON.parse(response.body);
    assert.deepEqual(replay, fixture);
    const { version, header: h, frames } = replay;
    assert.equal(version, 1);
    assert.equal(h.seed, entry.seed);
    assert.equal(h.winner, entry.winner);
    assert.equal(h.outcome, entry.outcome);
    assert.equal(h.turns, entry.turns);
    assert.equal(frames.length, h.turns);
    assert.equal(h.terrain.length, h.width * h.height);
    const players = new Set(h.players.map((p) => p.id));
    for (const [i, frame] of frames.entries()) {
      assert.equal(frame.turn, i + 1);
      assert.equal(frame.owners.reduce((sum, [owner, count]) => {
        assert.ok(owner === -1 || players.has(owner));
        assert.ok(Number.isInteger(count) && count > 0);
        return sum + count;
      }, 0), h.width * h.height);
      for (const item of [...frame.cities, ...frame.units]) {
        assert.ok(players.has(item.owner));
        const [q, r] = item.pos, col = q + (r - (r & 1)) / 2;
        assert.ok(col >= 0 && col < h.width && r >= 0 && r < h.height);
      }
      assert.equal(new Set(frame.units.map((a) => `${a.owner}:${a.pos}`)).size, frame.units.length);
      for (const vp of frame.vp) assert.equal(vp.total, Object.values(vp.sources).reduce((a, b) => a + b, 0));
      for (const owner of Object.values(frame.titles)) assert.ok(owner === null || players.has(owner));
      assert.ok(frame.events.every((e) => e.turn === frame.turn && typeof e.text === "string"));
    }
  }
});

test("replay routes reject traversal, encoded separators, nested paths and non-JSON files", async () => {
  for (const route of ["/replays/../Cargo.toml", "/replays/%2e%2e%2fCargo.toml", "/replays/%2Findex.json", "/replays/sub/index.json", "/replays/index.json/extra", "/replays/index.json%00", "/replays/index.tmp", "/replays/..\\Cargo.toml", "/replays/missing.json"]) {
    assert.equal((await get(route)).status, 404, route);
  }
  const name = `replay-test-symlink-${process.pid}.json`;
  const link = path.join(replayDir, name);
  try {
    fs.symlinkSync(fileURLToPath(new URL("../../Cargo.toml", import.meta.url)), link);
    assert.equal((await get(`/replays/${name}`)).status, 404);
  } finally { fs.rmSync(link, { force: true }); }
});
