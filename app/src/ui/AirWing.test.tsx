// @vitest-environment happy-dom
import { existsSync, readFileSync } from 'node:fs';
import { fileURLToPath, URL as NodeURL } from 'node:url';
import { act } from 'react';
import { createRoot, type Root } from 'react-dom/client';
import { afterEach, beforeAll, beforeEach, describe, expect, it, vi } from 'vitest';
import { WasmClient } from '../wasmClient';
import { useGame } from '../store';
import { AirWing } from './AirWing';
import { coreFixture } from '../test/coreFixture';
import { siegeFixture } from '../test/siegeFixture';

const available = existsSync(fileURLToPath(new NodeURL('../wasm-pkg/legatum.js', import.meta.url)));
let client: WasmClient;
let container: HTMLDivElement;
let root: Root;

describe.skipIf(!available)('the air wing of a city, on the Rust engine', () => {
  beforeAll(async () => {
    const path = '../wasm-pkg/legatum.js';
    const mod = await import(/* @vite-ignore */ path);
    await mod.default({ module_or_path: readFileSync(fileURLToPath(new NodeURL('../wasm-pkg/legatum_bg.wasm', import.meta.url))) });
    client = new WasmClient(mod.Game);
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

  function render(shape?: (view: ReturnType<WasmClient['view']>) => void, planeId = 900) {
    const s = siegeFixture({ fresh: true });
    const aircraft = JSON.stringify([{ id: 900, kind: 'Bomber', city: s.own, hp: 100 }, { id: 901, kind: 'Fighter', city: s.own, hp: 80 }]);
    const save = coreFixture(s.fixture).replace(/"next_unit_id":\d+/, '"next_unit_id":901');
    client.load(save.replace(/^\{/, `{"aircraft":${aircraft},`));
    const view = client.view(0);
    shape?.(view);
    const city = view.cities.find((c) => c.id === s.own)!;
    useGame.setState({ client, humans: [0], views: [view], seat: 0, orders: [[]], errors: [[]], ready: [false], air: { plane: null, what: null, hover: null }, airHeld: {} });
    act(() => root.render(<AirWing city={city} view={view} planeId={planeId} />));
    return { view, city };
  }
  const button = (text: RegExp, within: ParentNode = container) => [...within.querySelectorAll<HTMLButtonElement>('button')].find((b) => text.test(b.textContent ?? ''))!;
  const card = (plane: number) => container.querySelector<HTMLElement>(`[data-plane="${plane}"]`)!;
  /** Nobody to strike: the fixture's enemy Soldiers and cities are removed from the view. */
  const quiet = (view: { units: { owner: number }[]; cities: { owner: number }[] }) => { (view as { units: unknown[] }).units = view.units.filter((u) => u.owner === 0); (view as { cities: unknown[] }).cities = view.cities.filter((c) => c.owner === 0); };

  it('shows one selected aircraft in the side card and nothing for a city without it', () => {
    render();
    expect(container.querySelectorAll('[data-plane]')).toHaveLength(1);
    expect([...container.querySelectorAll('.up-stat')].map((stat) => stat.textContent)).toEqual(expect.arrayContaining(['Hangar2/2', 'Range8']));
    expect(container.textContent).toContain('Bomber');
    expect(container.textContent).toContain('Uses 1 Oil');
    expect(container.querySelector('.up-kind')?.textContent).toBe('Bomber');
    const view = useGame.getState().views![0]!;
    act(() => root.render(<AirWing city={{ ...view.cities[0], id: 4242 }} view={view} planeId={900} />));
    expect(container.querySelector('[data-testid="air-wing"]')).toBeNull();
  });

  it('shows Oil shortage on the aircraft card', () => {
    render((view) => { view.aircraft![0].unsupplied = 'Oil'; });
    expect(card(900).textContent).toContain('No Oil supply: strikes and interceptions are 25% weaker');
    const view = useGame.getState().views![0]!;
    view.aircraft![0].unsupplied = null;
  });

  it('strikes: the Bomber lists what is in range with its forecast, and picking one queues one order', () => {
    const { view } = render();
    act(() => button(/^Strike/, card(900)).click());
    expect(useGame.getState().air).toMatchObject({ plane: 900, what: 'strike' });
    const rows = [...container.querySelectorAll<HTMLButtonElement>('.air-target')];
    expect(rows.length).toBeGreaterThan(0);
    expect(rows[0].textContent).toMatch(/Destroys it|about −\d+ HP/);
    const spearman = rows.find((r) => /^Spearman · Byzantium\d+ tiles/.test(r.textContent ?? ''));
    expect(spearman).toBeDefined();
    act(() => spearman!.click());
    const orders = useGame.getState().orders[0];
    expect(orders).toHaveLength(1);
    expect(orders[0]).toMatchObject({ type: 'AirStrike', plane: 900 });
    expect(client.validate(0, orders)).toEqual([]);
    // The pending order replaces the buttons and can be cancelled.
    expect(container.textContent).toContain('Strikes the Byzantium Spearman');
    expect(card(900).textContent).toContain('Striking this turn');
    act(() => button(/Cancel order/).click());
    expect(useGame.getState().orders[0]).toEqual([]);
    expect(view.aircraft).toHaveLength(2);
  });

  it('recon: the button opens a map picker, a click on a lit tile queues one order that names its target, and the order can be cancelled', () => {
    render();
    const recon = button(/^Recon/, card(900));
    expect(recon.textContent).toContain('A');
    expect(recon.getAttribute('aria-keyshortcuts')).toBe('A');
    act(() => recon.click());
    expect(useGame.getState().air).toMatchObject({ plane: 900, what: 'recon' });
    expect(container.querySelector('[data-testid="recon-hint"]')?.textContent).toContain('lit tile');
    const view = useGame.getState().views![0]!;
    const city = view.cities.find((c) => c.id === useGame.getState().views![0]!.aircraft![0].city)!;
    const target = { q: city.pos.q + 5, r: city.pos.r };
    act(() => useGame.getState().editOrders((ord) => [...ord, { type: 'AirRecon', plane: 900, target }], 'commit'));
    const orders = useGame.getState().orders[0];
    expect(client.validate(0, orders)).toEqual([]);
    expect(card(900).textContent).toContain('Scouting this turn');
    expect(card(900).textContent).toContain('Scouts the tile 5 tiles');
    expect(card(900).textContent).toContain('next turn only');
    act(() => button(/Cancel order/).click());
    expect(useGame.getState().orders[0]).toEqual([]);
  });

  it('recon says why it is unavailable', () => {
    render((view) => { (view as { finished: boolean }).finished = true; });
    const recon = button(/^Recon/, card(900));
    expect(recon.disabled).toBe(true);
    expect(card(900).textContent).toContain('Recon unavailable: the game is over.');
  });

  it('a Fighter with an enemy in reach asks: strike, or stay on guard to intercept (the job is on the card)', () => {
    render(undefined, 901);
    const fighter = card(901);
    expect(fighter.getAttribute('data-air-state')).toBe('ready');
    expect(fighter.textContent).toMatch(/Ready: \d+ targets? within 5 tiles/);
    expect(fighter.textContent).toContain('stay on guard to intercept');
    expect([...fighter.querySelectorAll('button')].map((b) => b.textContent?.replace(/\s+/g, ' ').trim())).toEqual(['Strike…S', 'Recon…A', 'Stay on guardF', 'Rebase…R']);
    // Staying on guard answers it for this turn.
    act(() => button(/Stay on guard/, fighter).click());
    expect(card(901).getAttribute('data-air-state')).toBe('guard');
    expect(card(901).textContent).toContain('On guard: covers 4 tiles around Waypost Seven');
    expect(card(901).textContent).toContain('On guard: it intercepts enemy air strikes');
    // A strike order says it used the turn.
    act(() => button(/^Strike/, card(901)).click());
    const row = container.querySelector<HTMLButtonElement>('.air-target')!;
    act(() => row.click());
    expect(useGame.getState().orders[0][0]).toMatchObject({ type: 'AirStrike', plane: 901 });
    expect(card(901).textContent).toContain('Striking this turn, so it will not intercept');
  });

  it('a Fighter with nothing in reach still offers Stay on guard and Wait for target', () => {
    render(quiet, 901);
    const fighter = card(901);
    expect(fighter.getAttribute('data-air-state')).toBe('guard');
    // Unanswered, so the status says it is awaiting orders (red), not that it is settled.
    expect(fighter.querySelector('.up-status')?.textContent).toBe('Awaiting orders · on guard over 4 tiles around Waypost Seven');
    expect(fighter.textContent).toContain('No enemy unit or city is visible within 5 tiles');
    expect(fighter.textContent).not.toContain('or rebase it');
    // The fixture has one city, so Rebase explains why it is unavailable.
    expect([...fighter.querySelectorAll('button')].map((b) => b.textContent?.replace(/\s+/g, ' ').trim())).toEqual(['Recon…A', 'Stay on guardF', 'Skip this turnSpace', 'Rebase…R']);
    expect(button(/^Rebase/, fighter).disabled).toBe(true);
    expect(fighter.textContent).toContain('Rebase unavailable: no other city');
    act(() => button(/^Stay on guard/, fighter).click());
    expect(card(901).textContent).toContain('Waiting on guard');
    expect([...card(901).querySelectorAll('button')].map((b) => b.textContent?.replace(/\s+/g, ' ').trim())).toEqual(['Recon…A', 'WakeF', 'Rebase…R']);
    expect(useGame.getState().airHeld[0].asleep).toEqual([901]);
  });

  it('a Bomber with no target says why', () => {
    render(quiet);
    const bomber = card(900);
    expect(bomber.getAttribute('data-air-state')).toBe('idle');
    expect(bomber.querySelector('.up-status')?.textContent).toBe('Awaiting orders · no targets within 8 tiles');
    expect(bomber.textContent).toContain('Wait for a target, or skip this turn.');
    expect(bomber.textContent).not.toMatch(/rebase it/i);
    expect(bomber.textContent).toContain('No enemy unit or city is visible within 8 tiles of Waypost Seven');
    expect([...bomber.querySelectorAll('button')].map((b) => b.textContent?.replace(/\s+/g, ' ').trim())).toEqual(['Recon…A', 'Wait for targetF', 'Skip this turnSpace', 'Rebase…R']);
    // A card with nothing to strike still shows the next action: Wait is the one primary (8.9.6 UI review).
    expect(button(/^Wait for target/, bomber).classList.contains('primary')).toBe(true);
  });

  it('a Bomber with targets is ready: Strike is the primary, Wait sets it aside for the turn', () => {
    render();
    const bomber = card(900);
    expect(bomber.textContent).toMatch(/Ready: \d+ targets? within 8 tiles/);
    expect(button(/^Strike/, bomber).classList.contains('primary')).toBe(true);
    // Hold and Skip end in the same order when a target is in reach, so the Bomber's card shows only Skip.
    expect([...bomber.querySelectorAll('button')].map((b) => b.textContent?.replace(/\s+/g, ' ').trim())).toEqual(['Strike…S', 'Recon…A', 'Skip this turnSpace', 'Rebase…R']);
    act(() => button(/^Skip this turn/, bomber).click());
    expect(card(900).getAttribute('data-air-state')).toBe('idle');
    expect(card(900).textContent).toContain('Holding this turn');
    // The answer stays on the card as the active button, like a fortified unit's Fortify.
    const skip = button(/^Skip this turn/, card(900));
    expect(skip.getAttribute('aria-pressed')).toBe('true');
    expect(skip.querySelector('.action-check')).not.toBeNull();
  });

  it('a Fighter with only an at-Peace rival in reach keeps Strike as the primary and says why there is no target', () => {
    render((view) => {
      const enemy = view.players.find((p) => p.id !== view.me)!;
      view.diplomacy = { ...view.diplomacy, legacy: false, relations: [{ a: view.team, b: enemy.team, state: 'Peace' }] } as typeof view.diplomacy;
    }, 901);
    const fighter = card(901);
    expect(fighter.getAttribute('data-air-state')).toBe('guard');
    expect(button(/^Strike/, fighter).classList.contains('primary')).toBe(true);
    expect(fighter.querySelector('.up-hints')?.textContent).toMatch(/not at War with you: declare War to strike/);
    expect(fighter.querySelector('.up-hints')?.textContent).toContain('Stay on guard to intercept enemy air strikes');
    expect(fighter.querySelector('.up-hints')?.textContent).not.toContain('Nothing to strike');
    // While a picker is the decision, nothing is primary but the button that opened it.
    act(() => useGame.getState().set({ air: { plane: 901, what: 'rebase', hover: null } }));
    expect(container.querySelector('.act.primary')).toBeNull();
  });

  it('shows Strength from the engine data, and a held Fighter keeps Stay on guard as its active button', () => {
    render(undefined, 901);
    const stats = [...container.querySelectorAll('.up-stat')].map((stat) => stat.textContent);
    expect(stats.find((t) => t?.startsWith('Strength'))).toMatch(/^Strength\d+$/);
    act(() => button(/^Stay on guard/, card(901)).click());
    const guard = button(/^Stay on guard/, card(901));
    expect(guard.getAttribute('aria-pressed')).toBe('true');
    expect(guard.querySelector('.action-check')).not.toBeNull();
    expect(button(/^Skip this turn/, card(901))).toBeUndefined();
  });

  it('offers the Stealth upgrade as an Improve row with its Gold price, and says what is short', () => {
    render((view) => { view.aircraft![0].upgrade_cost = 300; view.aircraft![0].upgrade_options = [{ via: 'StealthBomber', to: 'StealthBomber', cost: 300 }]; view.economy.gold = 100; });
    const row = container.querySelector<HTMLButtonElement>('.improve-row button')!;
    expect(row.textContent).toContain('Upgrade to Stealth Bomber');
    expect(container.querySelector('.improve-row')?.textContent).toContain('300');
    expect(row.disabled).toBe(true);
    expect(container.querySelector('.improve-reason')?.textContent).toContain('Not enough Gold: the upgrade costs 300 and you have 100.');
    expect(row.getAttribute('aria-describedby')).toBe(container.querySelector('.improve-reason')?.id);
    act(() => root.unmount());
    root = createRoot(container);
    render((view) => { view.aircraft![0].upgrade_cost = 300; view.aircraft![0].upgrade_options = [{ via: 'StealthBomber', to: 'StealthBomber', cost: 300 }]; view.economy.gold = 1000; view.supply = [{ resource: 'Silicon', spare: 1 } as never]; });
    const ready = container.querySelector<HTMLButtonElement>('.improve-row button')!;
    expect(ready.disabled).toBe(false);
    act(() => ready.click());
    expect(useGame.getState().orders[0][0]).toMatchObject({ type: 'UpgradeAircraft', plane: 900 });
  });

  it('disables an aircraft upgrade when a queued city purchase leaves too little Gold', () => {
    const { view, city } = render((v) => { v.aircraft![0].upgrade_cost = 300; v.aircraft![0].upgrade_options = [{ via: 'StealthBomber', to: 'StealthBomber', cost: 300 }]; v.supply = [{ resource: 'Silicon', spare: 1 } as never]; });
    const option = city.detail!.options.find((o) => o.buy_cost !== null && o.buy_cost > 0)!;
    view.economy.gold = option.buy_cost! + 250;
    const orders = [{ type: 'BuyItem' as const, city: city.id, item: option.item }];
    act(() => useGame.setState({ orders: [orders], errors: [[]] }));
    act(() => root.render(<AirWing city={city} view={view} planeId={900} />));
    const upgrade = button(/Upgrade to Stealth Bomber/);
    expect(upgrade.disabled).toBe(true);
    expect(container.querySelector('.improve-reason')?.textContent).toContain('250 left after your queued orders');
  });

  it('a refused rebase returns to needs orders and explains full reachable hangars', () => {
    const { view, city } = render();
    const destination = { ...city, id: 77, name: 'Emain Macha', pos: { q: city.pos.q + 1, r: city.pos.r } };
    view.cities.push(destination);
    view.aircraft![1].city = destination.id;
    const refused = "Emain Macha's hangar is full (2 aircraft)";
    act(() => useGame.setState({
      orders: [[{ type: 'Rebase', plane: 901, to: destination.id }, { type: 'Rebase', plane: 900, to: destination.id }]],
      errors: [[{ index: 1, message: refused }]],
    }));
    expect(card(900).getAttribute('data-air-state')).toBe('ready');
    expect(card(900).querySelector('.up-status-why')?.textContent).toBe(refused);
    expect(card(900).textContent).not.toContain('Rebasing this turn');
    expect(button(/^Rebase/, card(900)).disabled).toBe(true);
    expect(card(900).textContent).toContain('Rebase unavailable: every other city hangar is full.');
    // The status names the refused order, not the plane's accepted state.
    expect(card(900).querySelector('.up-status')?.textContent).toBe('Rebasing to Emain Macha · won’t resolve');
  });
});
