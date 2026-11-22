#!/usr/bin/env node
// Cross-target determinism check for Hard+Search games. The sim tapes (wasm-tape-check.mjs) replay
// recorded orders, so they never run an AI; this plays whole games through the app's own entry
// point on both targets and compares the state hash after every turn: native `Game` against the
// WASM build (Node), the second one also saved and reloaded halfway.
//
//   wasm-pack build crates/wasm --target nodejs --release --out-dir ../../target/wasm-node
//   node scripts/wasm-search-check.mjs [target/wasm-node]
//
// Exit code 0 and "PASS" when every game matches; 1 and "FAIL" otherwise.
import { createRequire } from "node:module";
import { execFileSync } from "node:child_process";
import { existsSync, mkdtempSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";

const pkgDir = resolve(process.argv[2] ?? "target/wasm-node");
const entry = join(pkgDir, "legatum_wasm.js");
if (!existsSync(entry)) {
  console.error(`no nodejs wasm build at ${entry}`);
  process.exit(2);
}
const { Game } = createRequire(import.meta.url)(entry);

const FACTIONS = ["Switzerland", "India", "Byzantium", "Germany", "Celts", "Mongols", "Sparta", "Athens"];
const seat = (i, human, search) => ({
  name: human ? "Me" : `AI ${i}`,
  faction: FACTIONS[i % 8],
  team: i,
  is_human: human,
  difficulty: "Hard",
  ...(search ? { search: true } : {}),
});
// A duel (full budget) and an eight-player game with an idle human (the reduced, staggered budget).
const games = [
  { name: "duel, seat 1 Hard+Search", turns: 150, reloadAt: 130, setup: { seed: "7100003", width: 40, height: 26, speed: "Standard", turn_limit: null, barbarians: true, players: [seat(0, false, false), seat(1, false, true)] } },
  { name: "8 players, 7 Hard+Search", turns: 70, reloadAt: 45, setup: { seed: "12345", width: 48, height: 32, speed: "Standard", turn_limit: null, barbarians: true, players: [seat(0, true, false), ...[1, 2, 3, 4, 5, 6, 7].map((i) => seat(i, false, true))] } },
];

const dir = mkdtempSync(join(tmpdir(), "wasm-search-"));
let failed = 0;
for (const g of games) {
  const file = join(dir, "setup.json");
  writeFileSync(file, JSON.stringify(g.setup));
  const native = execFileSync(
    "cargo",
    ["run", "--release", "-q", "-p", "legatum-wasm", "--example", "search_hashes", "--", file, String(g.turns)],
    { encoding: "utf8", maxBuffer: 1 << 24 },
  ).trim().split("\n");

  let game = new Game(JSON.stringify(g.setup));
  const wasm = [game.hash()];
  for (let t = 0; t < g.turns; t++) {
    try {
      game.end_turn("{}");
    } catch {
      break;
    }
    if (t + 1 === g.reloadAt) game = Game.load(game.save());
    wasm.push(game.hash());
  }
  const n = Math.max(native.length, wasm.length);
  let diverged = -1;
  for (let i = 0; i < n; i++) {
    if (native[i] !== wasm[i]) {
      diverged = i;
      break;
    }
  }
  if (diverged === -1) console.log(`ok    ${g.name} (${native.length - 1} turns, reloaded at ${g.reloadAt})`);
  else {
    failed++;
    console.log(`FAIL  ${g.name}: first divergence after turn ${diverged}: native ${native[diverged]} vs wasm ${wasm[diverged]}`);
  }
}
if (failed === 0) console.log(`PASS: ${games.length} Hard+Search games, native and WASM hashes match every turn`);
else {
  console.log(`FAIL: ${failed} of ${games.length} games diverged`);
  process.exit(1);
}
