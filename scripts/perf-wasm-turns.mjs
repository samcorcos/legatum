#!/usr/bin/env node
// The WASM side of `crates/wasm/examples/turn_profile.rs`: replays the same 8-empire game (the human
// seat's orders come from the native run's orders file) through the shipped WASM API in Node (V8,
// the engine family Chromium uses) and times what the app waits on each turn.
//
//   wasm-pack build crates/wasm --target nodejs --release --out-dir ../../target/wasm-node
//   node scripts/perf-wasm-turns.mjs target/wasm-node SEED ORDERS_FILE > wasm-SEED.csv
//   tools/perf/summarize_turns.py wasm-*.csv
//
// CSV: turn,end_turn_ms,view_ms,view_kb,ai_ms,step_ms,parse_ms,wasm_mb (ai/step blank: WASM has no split).
import { createRequire } from 'node:module';
import { readFileSync } from 'node:fs';
import { resolve, join } from 'node:path';

const [pkgArg, seed, ordersFile] = process.argv.slice(2);
const require = createRequire(import.meta.url);
const mod = require(resolve(pkgArg, 'legatum_wasm.js'));
const { Game } = mod;
const FACTIONS = ['Switzerland', 'India', 'Byzantium', 'Germany', 'Celts', 'Mongols', 'Sparta', 'Athens'];
const seats = [...Array(8).keys()].map((i) => ({
  name: i === 0 ? 'Me' : `AI ${i}`,
  faction: FACTIONS[(i + Number(seed)) % 8],
  team: i,
  is_human: i === 0,
  difficulty: 'Hard',
}));
const setup = { seed: String(seed), width: 48, height: 32, turn_limit: null, barbarians: true, players: seats };
const orders = readFileSync(ordersFile, 'utf8').split('\n').filter(Boolean);
const game = new Game(JSON.stringify(setup));
const wasmMb = () => {
  const m = mod.__wasm?.memory;
  return m ? m.buffer.byteLength / 1048576 : NaN;
};
console.log('turn,end_turn_ms,view_ms,view_kb,ai_ms,step_ms,parse_ms,wasm_mb');
orders.forEach((human, i) => {
  let t = performance.now();
  game.end_turn(human);
  const end = performance.now() - t;
  t = performance.now();
  const view = game.view(0);
  const viewMs = performance.now() - t;
  t = performance.now();
  JSON.parse(view);
  const parse = performance.now() - t;
  console.log(`${i + 1},${end.toFixed(2)},${viewMs.toFixed(2)},${(view.length / 1024).toFixed(1)},,,${parse.toFixed(2)},${wasmMb().toFixed(1)}`);
});
