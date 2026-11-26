// @vitest-environment happy-dom
// Byzantium's Buy a city against the real engine: the view's price reaches the city panel, Make an offer queues
// `BuyCity`, the turn starts the offer, and the seller reads the countdown and what it can do.
import { existsSync, readFileSync } from 'node:fs';
import { fileURLToPath, URL as NodeURL } from 'node:url';
import { act } from 'react';
import { createRoot, type Root } from 'react-dom/client';
import { afterEach, beforeAll, beforeEach, describe, expect, it, vi } from 'vitest';
import { WasmClient } from '../wasmClient';
import { coreFixture } from './coreFixture';
import { buyCityState, PYLOS_ESCROW } from './buyCityState';
import { useGame } from '../store';
import { CityBuy, buyIsPrimary } from '../ui/CityBuy';
import { contextFromViews, formatEvents } from '../format';
import { offerChip, SELLER_OPTIONS } from '../buyCity';
import type { PlayerView } from '../types';

const available = existsSync(fileURLToPath(new NodeURL('../wasm-pkg/legatum.js', import.meta.url)));
let makeCore: () => WasmClient;
let container: HTMLDivElement, root: Root;

describe.skipIf(!available)('Buy a city', () => {
  beforeAll(async () => {
    const mod = await import(/* @vite-ignore */ '../wasm-pkg/legatum.js');
    await mod.default({ module_or_path: readFileSync(fileURLToPath(new NodeURL('../wasm-pkg/legatum_bg.wasm', import.meta.url))) });
    makeCore = () => new WasmClient(mod.Game);
  });
  beforeEach(() => {
    vi.stubGlobal('IS_REACT_ACT_ENVIRONMENT', true);
    container = document.body.appendChild(document.createElement('div')); root = createRoot(container);
    useGame.setState(useGame.getInitialState(), true);
  });
  afterEach(() => { act(() => root.unmount()); container.remove(); vi.unstubAllGlobals(); });

  const seat = (view: PlayerView) => useGame.setState({ views: [view], humans: [0], orders: [[]], errors: [[]], ready: [false], seat: 0 });

  it('Byzantium sees the price with every line, queues the offer, and the turn starts it in public', () => {
    const fx = buyCityState('buyer');
    const core = makeCore(); core.load(coreFixture(fx.state));
    const v = core.view(0);
    const buy = v.city_buys!.find((b) => b.city === fx.pylos && b.buyer === 0)!;
    expect(buy.blocker ?? null).toBeNull();
    expect(buy.breakdown.parts.reduce((n, [, x]) => n + x, 0)).toBe(buy.price);
    expect(buy.breakdown.parts.map(([l]) => l).join(' | ')).not.toMatch(/armed|Base price/);
    // Its capital is never for sale, and the fall is public.
    expect(v.byzantine_capitals).toEqual([{ player: 0, capital: fx.constantinople }]);

    seat(v);
    const pylos = v.cities.find((c) => c.id === fx.pylos)!;
    expect(buyIsPrimary(v, pylos, [])).toBe(true);
    act(() => root.render(<CityBuy view={v} city={pylos} />));
    const button = container.querySelector<HTMLButtonElement>('[data-testid=buy-city-offer]')!;
    expect(button.textContent).toBe(`Make an offer (${buy.price} Gold)`);
    expect(button.className).toContain('primary');
    expect(container.querySelector('.bd')!.textContent).toContain('Starting price');
    act(() => button.click());
    expect(useGame.getState().orders[0]).toEqual([{ type: 'BuyCity', city: fx.pylos }]);
    expect(container.querySelector('[data-testid=buy-city-queued]')!.textContent).toContain('Queued: your offer for Pylos');
    expect(buyIsPrimary(v, pylos, useGame.getState().orders[0])).toBe(false);

    const before1 = core.view(1);
    const result = core.endTurn({ 0: [{ type: 'BuyCity', city: fx.pylos }] });
    const made = result.events.find((e) => e.type === 'CityOfferMade');
    expect(made).toMatchObject({ city: fx.pylos, buyer: 0, seller: 1, escrow: buy.price });
    const after0 = core.view(0), after1 = core.view(1);
    expect(after0.city_offers).toEqual([expect.objectContaining({ city: fx.pylos, buyer: 0, seller: 1, escrow: buy.price, turns_left: 3 })]);
    expect(offerChip(after1, fx.pylos)).toBe('Being bought: 3 turns');
    expect(offerChip(after0, fx.pylos)).toBe('Being bought: 3 turns');
    const sellerLines = formatEvents(result.events, contextFromViews(before1, after1)).map((l) => l.text);
    expect(sellerLines.some((t) => t.startsWith('Byzantium is buying Pylos') && t.endsWith(SELLER_OPTIONS))).toBe(true);
  });

  it('the seller sees the countdown, the Gold offered and what it can do on its own city', () => {
    const fx = buyCityState('seller');
    const core = makeCore(); core.load(coreFixture(fx.state));
    const v = core.view(0);
    expect(v.city_offers).toEqual([{ city: fx.pylos, buyer: 1, seller: 0, escrow: PYLOS_ESCROW, ends_turn: 14, turns_left: 3, price_now: expect.any(Number) }]);
    seat(v);
    const pylos = v.cities.find((c) => c.id === fx.pylos)!;
    act(() => root.render(<CityBuy view={v} city={pylos} />));
    expect(container.querySelector('[data-testid=city-offer]')!.textContent).toContain('Byzantium is buying Pylos: it changes hands at the end of turn 14');
    expect(container.querySelector('[data-testid=city-offer-options]')!.textContent).toBe(SELLER_OPTIONS);
    expect(container.querySelector('[data-testid=buy-city-offer]')).toBeNull();
  });
});
