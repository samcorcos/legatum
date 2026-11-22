// The model against the real core: every figure comes from the view, none is computed in TypeScript.
import { existsSync, readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { beforeAll, describe, expect, it } from 'vitest';
import { WasmClient } from './wasmClient';
import { cityRows } from './citiesOverview';
import { coreFixture } from './test/coreFixture';
import { citiesFixture } from './test/citiesFixture';
import { COSTLY_LISTED_COST, showsWhereToBuild, whereSummary, whereToBuild } from './whereToBuild';
import type { ProductionItem } from './types';

const available = existsSync(fileURLToPath(new URL('./wasm-pkg/legatum.js', import.meta.url)));
let client: WasmClient;
beforeAll(async () => {
  if (!available) return;
  const path = './wasm-pkg/legatum.js'; const mod = await import(/* @vite-ignore */ path);
  await mod.default({ module_or_path: readFileSync(fileURLToPath(new URL('./wasm-pkg/legatum_bg.wasm', import.meta.url))) });
  client = new WasmClient(mod.Game);
});

const GL: ProductionItem = { kind: 'Building', building: 'RelayUplink' };
const FACTORY: ProductionItem = { kind: 'Building', building: 'Autoworks' };

describe.skipIf(!available)('the Great Library across five cities', () => {
  it("uses each city's own turns from the core, counts banked progress, and names the fastest", () => {
    client.load(coreFixture(citiesFixture()));
    const view = client.view(0);
    const w = whereToBuild(view, GL, []);
    expect(w.sites).toHaveLength(5);
    for (const s of w.sites.filter((x) => x.state === 'ready')) {
      const option = view.cities.find((c) => c.id === s.cityId)!.detail!.options
        .filter((o) => o.item.kind === 'Wonder' && o.item.building === 'RelayUplink')
        .sort((a, b) => a.turns - b.turns || b.production_stored - a.production_stored)[0]!;
      expect(s.turns).toBe(option.turns > 0 ? option.turns : null);
      expect(s.stored).toBe(option.production_stored);
    }
    const zurich = w.sites.find((s) => s.name === 'Zurich')!;
    expect(zurich.state).toBe('ready');
    expect(zurich.stored).toBeGreaterThanOrEqual(120);
    const turns = w.sites.filter((s) => s.state === 'ready' && s.turns !== null).map((s) => s.turns as number);
    expect(w.fastest?.turns).toBe(Math.min(...turns));
    expect(showsWhereToBuild(w)).toBe(true);
  });

  it('blocks a city with no Library and says so in plain words', () => {
    client.load(coreFixture(citiesFixture()));
    const w = whereToBuild(client.view(0), GL, []);
    for (const name of ['Basel', 'Geneva', 'Lugano']) expect(w.sites.find((s) => s.name === name)).toMatchObject({ state: 'blocked', reason: 'Needs a Library first' });
  });

  it('shows the same list for a technology not yet researched, from the research forecast', () => {
    const f = citiesFixture();
    f.players[0].techs = f.players[0].techs.filter((t) => t !== 'Philosophy');
    client.load(coreFixture(f));
    const w = whereToBuild(client.view(0), GL, []);
    expect(w.researched).toBe(false);
    expect(w.sites.every((s) => s.state === 'later' || s.state === 'blocked')).toBe(true);
    expect(whereSummary(w).text).toMatch(/^Best city for this once researched: \w+, done in \d+ turns?\./);
  });

  it('says where the one Great Work is already being built', () => {
    const f = citiesFixture();
    const bern = f.cities.find((c) => c.name === 'Bern')!;
    bern.producing = { kind: 'Building', building: 'RelayUplink' };
    client.load(coreFixture(f));
    const w = whereToBuild(client.view(0), GL, []);
    expect(w.sites.find((s) => s.name === 'Bern')).toMatchObject({ state: 'building' });
    expect(w.sites.find((s) => s.name === 'Zurich')).toMatchObject({ state: 'elsewhere', reason: 'Already building in Bern; one per empire' });
    expect(whereSummary(w).kind).toBe('building');
  });

  it('compares a costly building too, and the overview reads the same view', () => {
    client.load(coreFixture(citiesFixture()));
    const view = client.view(0);
    const plant = whereToBuild(view, FACTORY, []);
    expect(plant.listedCost).toBeGreaterThanOrEqual(COSTLY_LISTED_COST);
    expect(plant.sites.every((site) => site.state === 'later' || site.state === 'blocked')).toBe(true);
    expect(showsWhereToBuild(plant)).toBe(true);
    const rows = cityRows(view, []);
    expect(rows).toHaveLength(5);
    expect(rows.filter((r) => r.idle).map((r) => r.name)).toEqual(['Geneva']);
    const bern = rows.find((r) => r.name === 'Bern')!;
    expect(bern.production).toBe(view.cities.find((c) => c.name === 'Bern')!.detail!.yields.total.production);
  });
});
