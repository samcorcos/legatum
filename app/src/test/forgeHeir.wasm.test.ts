import { existsSync, readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { beforeAll, describe, expect, it } from 'vitest';
import { RULES, buildingStatsFor } from '../data';
import { WasmClient } from '../wasmClient';
import { coreFixture } from './coreFixture';
import { forgeHeirFixture } from './forgeHeirFixture';

const available = existsSync(fileURLToPath(new URL('../wasm-pkg/legatum.js', import.meta.url)));
let makeCore: () => WasmClient;

describe.skipIf(!available)('Germany Foundry and Byzantium Heir goals in the core', () => {
  beforeAll(async () => {
    const mod = await import(/* @vite-ignore */ '../wasm-pkg/legatum.js');
    await mod.default({ module_or_path: readFileSync(fileURLToPath(new URL('../wasm-pkg/legatum_bg.wasm', import.meta.url))) });
    makeCore = () => new WasmClient(mod.Game);
  });

  it('exports a live two-point Germany goal and the Ruhr Foundry yield', () => {
    expect(RULES.factionGoals.Germany).toMatchObject({ kind: 'Live', target: 5, label: 'Ruhr Foundries', vp: 2 });
    expect(buildingStatsFor('Germany', 'Forge').yields.production).toBe(6);
  });

  it('counts five German Foundries', () => {
    const { fixture, capital } = forgeHeirFixture('Germany');
    const core = makeCore();
    core.load(coreFixture(fixture));
    expect(core.view(0).faction_goal).toMatchObject({ value: 4, target: 5, done: false, label: '4 of 5 Ruhr Foundries' });
    fixture.cities.find((c) => c.id === capital)!.buildings = ['Forge'];
    core.load(coreFixture(fixture));
    expect(core.view(0).faction_goal).toMatchObject({ value: 5, target: 5 });
  });
});
