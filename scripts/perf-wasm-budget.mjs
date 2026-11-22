#!/usr/bin/env node
// The WASM side of `crates/wasm/examples/ai_budget.rs`: the same all-AI 8-empire game through the
// shipped WASM API in Node (V8), timing the whole lobby turn and each AI seat's planning.
// Needs a build with the `seat-timing` feature (tools/perf/ai-budget.sh makes it):
//   wasm-pack build crates/wasm --target nodejs --profile wasm-release --out-dir ../../target/wasm-budget -- --features seat-timing
//   node scripts/perf-wasm-budget.mjs target/wasm-budget SEED TURNS hard|search > wasm-SEED.csv
// CSV columns match the native example: turn,end_turn_ms,view_ms,hash,seat0_ms..seat7_ms,searched_seats.
import { createRequire } from 'node:module';
import { resolve } from 'node:path';

const [pkgArg, seed, turnsArg, mode] = process.argv.slice(2);
const require = createRequire(import.meta.url);
const { Game } = require(resolve(pkgArg, 'legatum_wasm.js'));
const FACTIONS = ['Switzerland', 'India', 'Byzantium', 'Germany', 'Celts', 'Mongols', 'Sparta', 'Athens'];
const search = mode === 'search';
const seats = [...Array(8).keys()].map((i) => ({
  name: `AI ${i}`,
  faction: FACTIONS[(i + Number(seed)) % 8],
  team: i,
  is_human: false,
  difficulty: 'Hard',
  search,
}));
const setup = { seed: String(seed), width: 48, height: 32, turn_limit: null, barbarians: true, players: seats };
const game = new Game(JSON.stringify(setup));
console.log(`turn,end_turn_ms,view_ms,hash,${seats.map((_, i) => `seat${i}_ms`).join(',')},searched_seats`);
for (let t = 1; t <= Number(turnsArg); t++) {
  let t0 = performance.now();
  let report;
  try {
    report = JSON.parse(game.end_turn(''));
  } catch {
    break;
  }
  const end = performance.now() - t0;
  t0 = performance.now();
  game.view(0);
  const view = performance.now() - t0;
  const ms = Array(8).fill('');
  let mask = 0;
  for (const [seat, m, searched] of JSON.parse(game.seat_timings())) {
    ms[seat] = m.toFixed(2);
    if (searched) mask |= 1 << seat;
  }
  console.log(`${t},${end.toFixed(2)},${view.toFixed(2)},${report.hash},${ms.join(',')},${mask}`);
  if (report.finished) break;
}
