import { existsSync, readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { beforeAll, expect, it } from 'vitest';
import { WasmClient } from '../wasmClient';
import { heldBackCatalog, scenarioCatalog } from './scenarios';

const available = existsSync(fileURLToPath(new URL('../wasm-pkg/legatum.js', import.meta.url)));
type GameClass = ConstructorParameters<typeof WasmClient>[0];
let Game: GameClass;
beforeAll(async () => {
  if (!available) return;
  const pkg = '../wasm-pkg/legatum.js'; const mod = await import(/* @vite-ignore */ pkg);
  await mod.default({ module_or_path: readFileSync(fileURLToPath(new URL('../wasm-pkg/legatum_bg.wasm', import.meta.url))) });
  Game = mod.Game;
});

it.skipIf(!available).each([...scenarioCatalog(), ...heldBackCatalog()].map((e) => [e.file.id, e] as const))('%s loads in the engine with the seat as the only human', (_, entry) => {
  const client = new WasmClient(Game);
  client.loadScenario(entry.text);
  const status = client.scenarioStatus();
  expect(status.outcome).toBe('Playing');
  expect(status.turns_left).toBe(entry.file.objective.turns);
  expect(status.end_turn).toBe(status.start_turn + entry.file.objective.turns);
  expect(status.score_lines.length).toBe(entry.file.objective.score.length);
  const view = client.view(entry.file.seat);
  expect(view.players.filter((p) => p.is_human).map((p) => p.id)).toEqual([entry.file.seat]);
  expect(view.players.find((p) => p.id === entry.file.seat)?.faction).toBe(entry.file.faction);
  // One turn with no orders resolves and counts down the window.
  client.endTurn({ [entry.file.seat]: [] });
  expect(client.scenarioStatus().turns_left).toBe(entry.file.objective.turns - 1);
  // The objective survives the engine-error recovery path (load from a save).
  client.load(client.save());
  expect(client.scenarioStatus().turn).toBe(status.turn + 1);
});

it.skipIf(!available)('a decided scenario plays no further turns', () => {
  const entry = scenarioCatalog().find((e) => e.file.id === 'war-sparta-nicaea')!;
  const client = new WasmClient(Game);
  const file = JSON.parse(entry.text);
  file.objective.turns = 1;
  client.loadScenario(JSON.stringify(file));
  client.endTurn({ [file.seat]: [] });
  const status = client.scenarioStatus();
  expect(status.outcome).toBe('Lost');
  expect(status.reason).toMatch(/^Time ran out/);
  expect(() => client.endTurn({ [file.seat]: [] })).toThrow(/scenario is over/);
});

it.skipIf(!available)('a tampered state is refused', () => {
  const entry = scenarioCatalog()[0];
  const file = JSON.parse(entry.text);
  file.state.hash = '1';
  expect(() => new WasmClient(Game).loadScenario(JSON.stringify(file))).toThrow(/hash/);
});
