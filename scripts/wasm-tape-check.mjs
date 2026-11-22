#!/usr/bin/env node
// Cross-target determinism check: replay native sim tapes in WASM (Node) and compare the
// per-turn state hashes with the ones the native run recorded. Game.replay_tape also checks
// the initial hash and replays from a save/load at every turn boundary, including the final state.
//
//   wasm-pack build crates/wasm --target nodejs --out-dir ../../target/wasm-node
//   cargo run --release -p legatum-sim -- --games 20 --label tapes --tape-out target/tapes
//   node scripts/wasm-tape-check.mjs target/tapes [target/wasm-node]
//
// Exit code 0 and "PASS" when every tape matches; 1 and "FAIL" otherwise.

import { createRequire } from "node:module";
import { readdirSync, readFileSync, existsSync } from "node:fs";
import { join, resolve } from "node:path";

const [tapeDir, pkgArg] = process.argv.slice(2);
if (!tapeDir) {
  console.error("usage: node scripts/wasm-tape-check.mjs <tape-dir> [wasm-node-pkg-dir]");
  process.exit(2);
}
const pkgDir = resolve(pkgArg ?? "target/wasm-node");
const entry = join(pkgDir, "legatum_wasm.js");
if (!existsSync(entry)) {
  console.error(`no nodejs wasm build at ${entry}`);
  console.error("build it: wasm-pack build crates/wasm --target nodejs --out-dir ../../target/wasm-node");
  process.exit(2);
}
const { Game } = createRequire(import.meta.url)(entry);

const files = readdirSync(tapeDir).filter((f) => f.endsWith(".json")).sort();
if (files.length === 0) {
  console.error(`no .json tapes in ${tapeDir}`);
  process.exit(2);
}

let failed = 0;
for (const file of files) {
  const text = readFileSync(join(tapeDir, file), "utf8");
  // Hashes are decimal strings in the tape, so JSON.parse keeps them exact.
  const expected = JSON.parse(text).turns.map((t) => t.hash);
  let got;
  try {
    got = JSON.parse(Game.replay_tape(text));
  } catch (e) {
    failed++;
    console.log(`FAIL  ${file}: replay threw: ${e?.message ?? e}`);
    continue;
  }
  const n = Math.max(expected.length, got.length);
  let diverged = -1;
  for (let i = 0; i < n; i++) {
    if (expected[i] !== got[i]) {
      diverged = i;
      break;
    }
  }
  if (diverged === -1) {
    console.log(`ok    ${file} (${expected.length} turns)`);
  } else {
    failed++;
    console.log(
      `FAIL  ${file}: first divergence after turn ${diverged + 1}: native ${expected[diverged]} vs wasm ${got[diverged]}`,
    );
  }
}

if (failed === 0) {
  console.log(`PASS: ${files.length} tapes, WASM hashes match native (initial state, every turn, and save/load)`);
} else {
  console.log(`FAIL: ${failed} of ${files.length} tapes diverged`);
  process.exit(1);
}
