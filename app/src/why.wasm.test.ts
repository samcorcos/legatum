// Coach questions through the real engine: playing a turn with a divergent order returns a question,
// asking changes neither the state hash nor the game, and the check's cost per End Turn is measured.
import { existsSync, readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { beforeAll, describe, expect, it } from 'vitest';
import { WasmClient } from './wasmClient';
import { buildSetup, newLobby, setDifficulty, setSlot } from './lobby';
import type { GameSetup, Order } from './types';

const available = existsSync(fileURLToPath(new URL('./wasm-pkg/legatum.js', import.meta.url)));
let makeCore: () => WasmClient;

function setup(mapSize: 'Small' | 'Standard'): GameSetup {
  let l = setDifficulty({ ...newLobby('solo', '77'), mapSize }, 'Hard');
  if (mapSize === 'Small') for (const i of [3, 4, 5, 6, 7]) l = setSlot(l, i, { kind: 'closed' });
  return buildSetup(l);
}

describe.skipIf(!available)('Coach questions in the engine', () => {
  beforeAll(async () => {
    const path = './wasm-pkg/legatum.js';
    const mod = await import(/* @vite-ignore */ path);
    await mod.default({ module_or_path: readFileSync(fileURLToPath(new URL('./wasm-pkg/legatum_bg.wasm', import.meta.url))) });
    makeCore = () => new WasmClient(mod.Game);
  });

  it('a turn that declares a war Hard would not declare gets one question, and asking leaves the game alone', async () => {
    const a = makeCore();
    await a.newGame(setup('Small'));
    const none = { keys: [] as string[], last: null };
    const hash = a.hash();
    // Playing Hard's own orders asks nothing; asking changes nothing.
    const hard = a.whyReview(0, [], none).hard_orders;
    expect(a.whyReview(0, hard, none).question).toBeNull();
    expect(a.hash()).toBe(hash);
    const declare: Order = { type: "DeclareWar", against: 1, breach: false } as unknown as Order;
    const r = a.whyReview(0, [...hard, declare], none);
    expect(a.hash()).toBe(hash);
    expect(r.question).not.toBeNull();
    expect(r.question!.category).toBe('war_peace');
    expect(r.question!.text).toContain('you declared war');
    // One question per turn at most, whatever qualified.
    expect(Array.isArray(r.candidates)).toBe(true);
    // The same fact is not asked again.
    expect(a.whyReview(0, [...hard, declare], { keys: [r.question!.key], last: null }).question).toBeNull();
    // The turn then plays.
    expect(a.endTurn({ 0: [] }).turn).toBe(1);
  });

  it('adds well under a second to End Turn at about turn 100 of a Standard game', async () => {
    const watched = makeCore();
    await watched.newGame(setup('Standard'));
    const asked = { keys: [] as string[], last: null };
    const base: number[] = [];
    const extra: number[] = [];
    // He plays Hard's own orders, so the game is a real one: a live seat with cities and armies.
    let mine: Order[] = [];
    for (let t = 0; t < 100; t++) {
      const t0 = performance.now();
      const r = watched.whyReview(0, mine, asked);
      const t1 = performance.now();
      mine = r.hard_orders;
      watched.endTurn({ 0: mine });
      const t2 = performance.now();
      if (t >= 90) { extra.push(t1 - t0); base.push(t2 - t1); }
    }
    const v = watched.view(0);
    const mine0 = v.cities.filter((c) => c.owner === 0).length;
    console.log(`WHYMS game: turn ${v.turn}, ${mine0} cities of his, ${v.cities.length} known`);
    expect(mine0).toBeGreaterThan(2);
    const mean = (v: number[]) => v.reduce((x, y) => x + y, 0) / v.length;
    console.log(`WHYMS turn 91-100: review ${mean(extra).toFixed(0)} ms, endTurn ${mean(base).toFixed(0)} ms (max review ${Math.max(...extra).toFixed(0)} ms)`);
    expect(mean(extra)).toBeLessThan(1000);
  }, 600_000);
});
