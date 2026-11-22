// @vitest-environment happy-dom
import { existsSync, readFileSync } from 'node:fs';
import { fileURLToPath, URL as NodeURL } from 'node:url';
import { act } from 'react';
import { createRoot, type Root } from 'react-dom/client';
import { afterEach, beforeAll, beforeEach, describe, expect, it, vi } from 'vitest';
import { WasmClient } from '../wasmClient';
import { useGame } from '../store';
import { CityPanel } from './CityPanel';
import type { Faction } from '../types';
import { withoutArchive } from '../test/saveFixture';

const available = existsSync(fileURLToPath(new NodeURL('../wasm-pkg/legatum.js', import.meta.url)));
let makeClient: () => WasmClient;
let container: HTMLDivElement;
let root: Root;

describe.skipIf(!available)('production controls on the Rust engine', () => {
  beforeAll(async () => {
    const path = '../wasm-pkg/legatum.js';
    const mod = await import(/* @vite-ignore */ path);
    await mod.default({ module_or_path: readFileSync(fileURLToPath(new NodeURL('../wasm-pkg/legatum_bg.wasm', import.meta.url))) });
    makeClient = () => new WasmClient(mod.Game);
  });
  beforeEach(() => {
    vi.stubGlobal('IS_REACT_ACT_ENVIRONMENT', true);
    useGame.setState(useGame.getInitialState(), true);
    container = document.body.appendChild(document.createElement('div'));
    root = createRoot(container);
  });
  afterEach(() => {
    act(() => root.unmount());
    container.remove();
    vi.unstubAllGlobals();
  });

  async function renderCity(stored = 30, gold = 1_000, faction: Faction = 'India', purchased = 0, building: 'Granary' | 'Market' = 'Granary') {
    const client = makeClient();
    const setup = await client.standardSetup('42', faction, 'Byzantium', 'Ada', 'Bo');
    setup.players = setup.players.slice(0, 2);
    setup.players[1].team = 1;
    await client.newGame(setup);
    client.endTurn({});
    // Only one city exists. Change its fixture fields in the opaque save text, preserving
    // all u64 bitset words exactly (JSON.parse/stringify would round them).
    const granary = { kind: 'Building', building } as const;
    const save = withoutArchive(client.save())
      .replace(/"production_stored":\d+/, `"production_stored":${stored}`)
      .replace('"producing":null', `"producing":${JSON.stringify(granary)}`)
      .replace('"techs":[]', '"techs":["Irrigation","Currency"]')
      .replace(/"gold":\d+/, `"gold":${gold}`)
      .replace('"bought":null', `"bought":${purchased ? JSON.stringify({ item: granary, cost: stored, progress: purchased }) : 'null'}`);
    client.load(withoutArchive(save));
    const view = client.view(0);
    const city = view.cities.find((c) => c.owner === 0)!;
    expect(city.detail!.production_stored).toBe(stored);
    expect(view.economy.gold).toBe(gold);
    useGame.setState({ client, humans: [0], views: [view], seat: 0, orders: [[]], errors: [[]], ready: [false] });
    act(() => root.render(<CityPanel city={city} view={view} />));
    return { client, city, view };
  }

  function button(label: string) {
    return [...container.querySelectorAll<HTMLButtonElement>('button')].find((b) => b.getAttribute('aria-label') === label)!;
  }

  it('buys a separate Spearman and preserves the chosen building and earned progress', async () => {
    const { client, city } = await renderCity(10, 1_000, 'India', 0, 'Market');
    act(() => button('Buy Spearman for 160 Gold').click());
    const soldier = { kind: 'Unit', unit: 'Spearman' } as const;
    const orders = useGame.getState().orders[0];
    expect(orders).toEqual([{ type: 'BuyItem', city: city.id, item: soldier }]);
    expect(container.textContent).toContain('160 Gold');
    expect(container.querySelector('.purchase-status')?.textContent).toContain('to spend');
    expect(client.validate(0, orders)).toEqual([]);
    const result = client.endTurn({ 0: orders });
    expect(result.events).toContainEqual({ type: 'ItemBought', city: city.id, owner: 0, item: soldier, gold: 160 });
    expect(result.events).toContainEqual({ type: 'ProductionCompleted', city: city.id, owner: 0, item: soldier });
    const detail = client.view(0).cities.find((c) => c.id === city.id)!.detail!;
    expect(detail.producing).toEqual({ kind: 'Building', building: 'Market' });
    expect(detail.production_stored).toBeGreaterThan(10);
  });

  it('queues, reorders and removes with numbered controls and retains the saved plan', async () => {
    const { client, city } = await renderCity(10);
    act(() => button('Queue Spearman').click());
    expect(button('Build Spearman').classList.contains('pending')).toBe(true);
    expect(button('Build Spearman').textContent).toContain('Queued');
    act(() => button('Queue Scout').click());
    expect(container.querySelectorAll('[data-testid="production-queue"] li')).toHaveLength(2);
    act(() => button('Move Scout up').click());
    act(() => button('Remove Spearman from queue').click());
    const orders = useGame.getState().orders[0];
    expect(orders).toEqual([{ type: 'SetProductionQueue', city: city.id, queue: [
      { kind: 'Building', building: 'Granary' }, { kind: 'Unit', unit: 'Scout' },
    ] }]);
    expect(client.validate(0, orders)).toEqual([]);
    client.endTurn({ 0: orders });
    client.load(client.save());
    expect(client.view(0).cities.find((c) => c.id === city.id)!.detail!.production_queue).toEqual([{ kind: 'Unit', unit: 'Scout' }]);
  });

  it('never submits speculative purchase orders while rendering an online city', async () => {
    const { client, city, view } = await renderCity();
    const validate = vi.fn(() => []);
    const remote = Object.assign(Object.create(client), { kind: 'remote', validate });
    act(() => {
      useGame.setState({ client: remote });
      root.render(<CityPanel city={city} view={view} />);
    });
    expect(button('Buy Spearman for 160 Gold').disabled).toBe(false);
    expect(validate).not.toHaveBeenCalled();
    expect(useGame.getState().orders[0]).toEqual([]);
  });

  it('updates online affordability reasons after an earlier purchase reserves gold', async () => {
    const { client, city, view } = await renderCity(0, 130);
    const validate = vi.fn(() => []);
    const remote = Object.assign(Object.create(client), { kind: 'remote', validate });
    view.cities.push({ ...city, id: city.id + 1 });
    act(() => {
      useGame.setState({ client: remote, orders: [[{ type: 'BuyItem', city: city.id + 1, item: { kind: 'Unit', unit: 'Scout' } }]] });
      root.render(<CityPanel city={city} view={view} />);
    });
    const buy = button('Buy Spearman for 160 Gold');
    expect(buy.disabled).toBe(true);
    expect(container.querySelector('.purchase-status')?.textContent).toContain('to spend');
    expect(buy.getAttribute('aria-describedby')).toBe('purchase-status-' + city.id);
    expect(document.body.textContent).toContain('Not enough Gold: costs 160 Gold; you have 10');
    expect(validate).not.toHaveBeenCalled();
  });

  it('keeps the current stall visible when only the tail of a queue changes', async () => {
    const { city, view } = await renderCity();
    city.detail!.producing = { kind: 'Unit', unit: 'Spearman' };
    city.detail!.production_stall = { kind: 'SpawnBlocked' };
    city.detail!.turns_to_complete = null;
    act(() => root.render(<CityPanel city={city} view={view} />));
    act(() => button('Queue Scout').click());
    expect(container.textContent).toContain('Stalled');
    expect(container.textContent).toContain('no free spawn tile');
  });

  it.each([
    { stored: 30, gold: 160, faction: 'India' as const, cost: 160, disabled: false },
    { stored: 30, gold: 159, faction: 'India' as const, cost: 160, disabled: true },
    { stored: 30, gold: 140, faction: 'Switzerland' as const, cost: 140, disabled: false },
    { stored: 50, gold: 0, faction: 'India' as const, cost: 160, disabled: true },
  ])('quotes the full $cost with stored=$stored, gold=$gold, faction=$faction', async ({ stored, gold, faction, cost, disabled }) => {
    const { client } = await renderCity(stored, gold, faction);
    const buy = button(`Buy Spearman for ${cost} Gold`);
    expect(buy.disabled).toBe(disabled);
    if (!disabled) {
      act(() => buy.click());
      const orders = useGame.getState().orders[0];
      expect(client.validate(0, orders)).toEqual([]);
      expect(client.endTurn({ 0: orders }).events).toContainEqual(expect.objectContaining({ type: 'ItemBought', gold: cost }));
    } else {
      expect(container.querySelector('.purchase-status')?.textContent).toContain('to spend');
      expect(buy.getAttribute('aria-describedby')).toMatch(/^purchase-status-/);
    }
  });
});
