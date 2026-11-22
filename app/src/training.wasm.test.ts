// Training mode through the real engine: the setup option is stored in the save and reloads, the
// AI seats' battles are searched in WASM slices, and the same seed and orders give the same game.
import { existsSync, readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { beforeAll, describe, expect, it, vi } from 'vitest';
import { WasmClient } from './wasmClient';
import { TRAINING_DEV_KEY, buildSetup, newLobby, setDifficulty, setSlot } from './lobby';
import type { GameSetup } from './types';

/** A save without its archive, whose metadata carries the recording time. */
function state(save: string): string {
  const { archive: _archive, ...rest } = JSON.parse(save);
  return JSON.stringify(rest);
}

const available = existsSync(fileURLToPath(new URL('./wasm-pkg/legatum.js', import.meta.url)));
let makeCore: () => WasmClient;

/** A small solo game from the lobby: you plus two AIs at the given level. */
function lobbySetup(level: 'Hard' | 'Training'): GameSetup {
  // Training is a research tool: the lobby model writes it only with the developer switch on.
  vi.stubGlobal('localStorage', { getItem: (key: string) => key === TRAINING_DEV_KEY ? '1' : null, setItem: () => undefined, removeItem: () => undefined });
  try { return lobbySetupWithSwitch(level); } finally { vi.unstubAllGlobals(); }
}

function lobbySetupWithSwitch(level: 'Hard' | 'Training'): GameSetup {
  let l = setDifficulty(newLobby('solo', '77'), level);
  // Keep the battle-producing seats fixed as the selectable faction pool grows.
  l = setSlot(setSlot(l, 1, { faction: 'Aztecs' }), 2, { faction: 'Germany' });
  l = setSlot(setSlot({ ...l, mapSize: 'Small' }, 3, { kind: 'closed' }), 4, { kind: 'closed' });
  for (const i of [5, 6, 7]) l = setSlot(l, i, { kind: 'closed' });
  // Keep the established battle-search fixture stable as new factions enter the random pool.
  l = setSlot(setSlot(l, 1, { faction: 'Aztecs' }), 2, { faction: 'Germany' });
  return buildSetup(l);
}

describe.skipIf(!available)('Training in the engine', () => {
  beforeAll(async () => {
    const path = './wasm-pkg/legatum.js';
    const mod = await import(/* @vite-ignore */ path);
    await mod.default({ module_or_path: readFileSync(fileURLToPath(new URL('./wasm-pkg/legatum_bg.wasm', import.meta.url))) });
    makeCore = () => new WasmClient(mod.Game);
  });

  it('is stored in the save, reloads, and replays the same game from the same seed', async () => {
    const setup = lobbySetup('Training');
    expect(setup.players.filter((p) => p.search)).toHaveLength(2);
    const a = makeCore();
    await a.newGame(setup);
    const b = makeCore();
    await b.newGame(setup);
    for (let t = 0; t < 12; t++) {
      a.endTurn({});
      b.endTurn({});
    }
    expect(state(a.save())).toBe(state(b.save()));
    const saved = JSON.parse(a.save());
    expect(saved.setup.players.map((p: { search?: boolean }) => Boolean(p.search))).toEqual(setup.players.map((p) => Boolean(p.search)));
    // Reload, then both games play on identically.
    const c = makeCore();
    c.load(a.save());
    for (let t = 0; t < 6; t++) {
      a.endTurn({});
      c.endTurn({});
    }
    expect(state(c.save())).toBe(state(a.save()));
  });

  it('searches the AI seats\' battles in slices, plays the result and logs it', async () => {
    // Start the AI seats at war so this tests the search path even if their
    // peacetime strategies change and they do not meet in 120 turns.
    const setup = { ...lobbySetup('Training'), legacy_all_war: true };
    const a = makeCore();
    await a.newGame(setup);
    const b = makeCore();
    await b.newGame(setup);
    let searched = 0;
    const lines: string[] = [];
    // A short search per turn; the same budget on both copies keeps them identical.
    const train = (g: WasmClient) => {
      const p = g.trainingBegin(0);
      if (!p || p.battles.length === 0) return 0;
      for (let i = 0; i < 60; i++) if (g.trainingStep(4, 0)?.done) break;
      const done = g.trainingFinish(0);
      lines.push(...(done?.lines ?? []));
      return done?.progress.battles.length ?? 0;
    };
    // Hard's purchase decisions can delay the first AI battle; match the Rust
    // training check's 400-turn window while keeping the exact replay assertion.
    for (let t = 0; t < 400 && searched === 0; t++) {
      searched += train(a);
      train(b);
      a.endTurn({});
      b.endTurn({});
    }
    expect(searched).toBeGreaterThan(0);
    expect(state(a.save())).toBe(state(b.save()));
    const first = JSON.parse(lines[0]);
    expect(first.type).toBe('decision');
    expect(first.profile.length).toBeGreaterThan(0);
    expect(a.trainingGameId()).toMatch(/^[\w-]+$/);
  }, 120_000);

  it('plain Hard is unchanged: its setup and saves carry no search field', async () => {
    const setup = lobbySetup('Hard');
    expect(JSON.stringify(setup)).not.toContain('search');
    const g = makeCore();
    await g.newGame(setup);
    expect(g.save()).not.toContain('"search"');
  });
});
