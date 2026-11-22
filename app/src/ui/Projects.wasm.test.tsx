// @vitest-environment happy-dom
import { existsSync, readFileSync } from 'node:fs';
import { fileURLToPath, URL as NodeURL } from 'node:url';
import { act } from 'react';
import { createRoot } from 'react-dom/client';
import { beforeAll, describe, expect, it, vi } from 'vitest';
import { WasmClient } from '../wasmClient';
import { useGame } from '../store';
import { BUILDING_NAME, BUILDING_STATS, GREAT_WORKS, TECHS, greatWorkRequirement } from '../data';
import type { BuildingKind, CityId } from '../types';
import { CityPanel } from './CityPanel';
import { VpTrack } from './VpTrack';

const available = existsSync(fileURLToPath(new NodeURL('../wasm-pkg/legatum.js', import.meta.url)));
let makeClient: () => WasmClient;

describe.skipIf(!available)('Bicycle victory routes through the WASM app boundary', () => {
  beforeAll(async () => {
    const path = '../wasm-pkg/legatum.js';
    const mod = await import(/* @vite-ignore */ path);
    await mod.default({ module_or_path: readFileSync(fileURLToPath(new NodeURL('../wasm-pkg/legatum_bg.wasm', import.meta.url))) });
    makeClient = () => new WasmClient(mod.Game);
  });

  it.each(GREAT_WORKS)('renders, queues, completes and saves %s using core quotes', async (building) => {
    vi.stubGlobal('IS_REACT_ACT_ENVIRONMENT', true);
    useGame.setState(useGame.getInitialState(), true);
    const client = makeClient();
    const setup = await client.standardSetup('42', 'Germany', 'Byzantium', 'Ada', 'Bo');
    setup.players = setup.players.slice(0, 2); setup.players[1].team = 1;
    await client.newGame(setup);
    client.endTurn({});
    // Protect every potentially inexact integer token while arranging the late-game fixture.
    const protectedSave = client.save().replace(/([:[,]\s*)(\d{16,})(?=\s*[,}\]])/g, '$1"u64:$2"');
    const saved = JSON.parse(protectedSave);
    delete saved.archive; // Synthetic fixture edits have no matching pre-edit order tape.
    saved.players[0].techs = [...TECHS];
    saved.units = {};
    const ids = Object.keys(saved.cities).map(Number);
    for (const id of ids) {
      const c = saved.cities[id];
      c.owner = 0; c.population = 8; c.food_stored = 100; c.production_stored = 0;
      c.buildings = [greatWorkRequirement(building)?.[0] ?? 'Granary', 'Arcology'];
    }
    const cost = BUILDING_STATS[building].cost;
    saved.cities[ids[0]].production_stored = cost - 1;
    for (const tile of saved.map.tiles) if (tile.owner === 1) tile.owner = 0;
    client.load(JSON.stringify(saved).replace(/"u64:(\d+)"/g, '$1'));
    const view = client.view(0), city = view.cities.find((c) => c.id === ids[0])!;
    const unlock = view.research.unlocks.find((u) => u.item.kind === 'Building' && u.item.building === building)!;
    expect(unlock.cost_breakdown).toEqual({ total: cost, parts: [['Listed cost', cost]] });
    const option = city.detail!.options.find((o) => o.item.kind === 'Wonder' && o.item.building === building)!;
    expect(option).toMatchObject({ cost, production_stored: cost - 1, buy_cost: null, turns: 1 });
    const item = option.item;
    useGame.setState({ client, humans: [0], views: [view], seat: 0, orders: [[]], errors: [[]], ready: [false], vpExpanded: true });
    const container = document.body.appendChild(document.createElement('div'));
    const root = createRoot(container);
    try {
      act(() => root.render(<><CityPanel city={city} view={view} /><VpTrack /></>));
      expect(container.querySelector('.vp-row.mine')).not.toBeNull();
      const button = [...container.querySelectorAll<HTMLButtonElement>('.cp-great-works button')].find((b) => b.textContent!.startsWith(BUILDING_NAME[building]))!;
      expect(button.disabled).toBe(false);
      // Build opens the placement; a lit tile clicked on the map makes it the order.
      act(() => button.click());
      expect(useGame.getState().place).toMatchObject({ city: city.id, building, intent: 'build' });
      expect(container.querySelector('[data-testid="city-placement"]')?.textContent).toContain(`Choose a tile for ${BUILDING_NAME[building]}: click a highlighted tile near ${city.name}`);
      expect(container.querySelector('.cp-great-works')).toBeNull();
      act(() => useGame.getState().set({ place: { ...useGame.getState().place!, pick: (item as Extract<typeof item, { kind: 'Wonder' }>).tile } }));
      expect(container.querySelector('[data-testid="city-placement"]')).toBeNull();
      expect(container.querySelector('.cp-great-works')).not.toBeNull();
      const orders = useGame.getState().orders[0];
      expect(orders).toEqual([{ type: 'SetProductionQueue', city: city.id, queue: [item] }]);
      expect(client.validate(0, [...orders, { type: 'BuyItem', city: city.id, item }])).toHaveLength(1);
      const result = client.endTurn({0: orders, 1: []});
      expect(result.events).toContainEqual({ type: 'ProductionCompleted', city: city.id, owner: 0, item });
      client.load(client.save());
      const after = client.view(0);
      expect(after.great_works![0].completed).toBe(building as BuildingKind);
      expect(after.players[0].vp_sources).toContainEqual([{ kind: 'GreatWork', of: 0 }, 2]);
      expect(client.validate(0, [{ type: 'SetProduction', city: ids[1] as CityId, item }])).toHaveLength(1);
    } finally {
      act(() => root.unmount()); container.remove(); vi.unstubAllGlobals();
    }
  });
});
