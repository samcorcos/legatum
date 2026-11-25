// @vitest-environment happy-dom
import { act } from 'react';
import { createRoot, type Root } from 'react-dom/client';
import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import { CityPanel } from './CityPanel';
import { signed } from './bits';
import { TileCard } from './TileCard';
import { ForecastCard } from './ForecastCard';
import { FixtureClient } from '../test/fixtureEngine';
import { useGame } from '../store';
import { neighbors, tileIndex } from '../hex';
import { FIRST_GROWTH, TECHS, greatWorkRules } from '../data';
import { TILE_RELEASE_RULE } from './cityTerritoryText';
import type { BattleForecast, Faction } from '../types';

let container: HTMLDivElement;
let root: Root;
beforeEach(() => {
  vi.stubGlobal('IS_REACT_ACT_ENVIRONMENT', true);
  container = document.body.appendChild(document.createElement('div'));
  root = createRoot(container);
  useGame.setState(useGame.getInitialState(), true);
});
afterEach(() => {
  act(() => root.unmount());
  container.remove();
  vi.unstubAllGlobals();
});

async function renderCity(faction: Faction = 'Switzerland', gold?: number) {
  const client = new FixtureClient();
  await client.newGame(await client.standardSetup('42', faction, 'India', 'Ada', 'Bo'));
  client.endTurn({});
  const state = client.debugState();
  const city = state.cities.find((c) => c.owner === 0)!;
  city.population = 1;
  for (const h of neighbors(city.pos)) state.terrain[tileIndex(h, state.width, state.height)] = 'Grassland';
  state.terrain[tileIndex(neighbors(city.pos)[0], state.width, state.height)] = 'Hills';
  if (gold !== undefined) state.players[0].gold = gold;
  const view = client.view(0);
  const c = view.cities.find((c) => c.id === city.id)!;
  useGame.setState({ client, views: [view], humans: [0], orders: [[]], errors: [[]], seat: 0 });
  act(() => root.render(<CityPanel city={c} view={view} />));
  return { client, city: c, view };
}

it('shows Raze loot, confirms the order, and explains protected cities', async () => {
  const { city, view } = await renderCity();
  const edit = vi.fn((change: (orders: any[]) => any[]) => {
    useGame.setState({ orders: [change(useGame.getState().orders[0])] });
  });
  act(() => useGame.setState({ editOrders: edit as any }));
  city.population = 3;
  city.detail!.captured = true;
  city.detail!.raze_blocker = undefined;
  act(() => root.render(<CityPanel city={city} view={view} />));
  const raze = container.querySelector<HTMLButtonElement>('[data-testid="city-raze"] button')!;
  expect(container.querySelector('[data-testid="city-raze"]')!.textContent).toContain('30 Gold');
  act(() => raze.click());
  expect(useGame.getState().orders[0]).toEqual([]);
  expect(container.querySelector('[data-testid="raze-confirm"]')).not.toBeNull();
  act(() => [...container.querySelectorAll('button')].find((b) => b.textContent === 'Confirm Raze')!.click());
  expect(useGame.getState().orders[0]).toEqual([{ type: 'Raze', city: city.id }]);
  act(() => [...container.querySelectorAll('button')].find((b) => b.textContent?.includes('Undo'))!.click());
  city.detail!.raze_blocker = 'Berea can be razed from the turn after its capture';
  act(() => root.render(<CityPanel city={city} view={view} />));
  expect(container.textContent).toContain('can be razed from the turn after its capture');
  expect(container.querySelector<HTMLButtonElement>('[data-testid="city-raze"] button')!.disabled).toBe(true);
});

it('shows public identity but no remembered city health or population under fog', async () => {
  const { city, view } = await renderCity();
  const fogged = { ...city, id: 90, owner: 2, name: 'Waypost Seven', visible: false,
    population: 7, hp: 17, strength: 99, walls: true, detail: null };
  const i = tileIndex(city.pos, view.width, view.height);
  view.tiles[i] = { ...view.tiles[i], visible: false, owner: 2 };
  view.cities = [fogged];
  act(() => root.render(<CityPanel city={fogged} view={view} />));
  expect(container.textContent).toContain('Waypost Seven');
  expect(container.querySelector('.cp-meta')).toBeNull();
  expect(container.querySelector('[data-testid="city-fire"]')).toBeNull();
  act(() => root.render(<TileCard view={view} hex={city.pos} />));
  expect(container.textContent).toContain('Waypost Seven · out of sight');
  expect(container.textContent).not.toContain('pop 7');
  expect(container.textContent).not.toContain('strength 99');
  expect(container.querySelector('[data-testid="city-fire"]')).toBeNull();
});

it('queues a nested building upgrade after its prerequisite', async () => {
  const { city, view } = await renderCity('Switzerland');
  const granary = { kind: 'Building', building: 'Granary' } as const;
  const mill = { kind: 'Building', building: 'Mill' } as const;
  city.detail!.unavailable!.push({ item: mill, cost: 180, reason: 'Requires Granary', prerequisite_building: 'Granary', production_banked: 0 });
  act(() => root.render(<CityPanel city={city} view={view} />));
  expect(container.querySelector('[data-item="b:Mill"]')).toBeNull();
  expect(container.textContent).toMatch(/Leads to: Mill \(180\s*\)/);
  const queueUpgrade = container.querySelector<HTMLButtonElement>('.upgrade-queue')!;
  expect(queueUpgrade.getAttribute('aria-label')).toMatch(/^Queue Mill after /);
  act(() => queueUpgrade.click());
  expect(useGame.getState().orders[0]).toEqual([{ type: 'SetProductionQueue', city: city.id, queue: [granary, mill] }]);
});

/** The claim phrase's tooltip text ("Ring N. ... Claims the nearest ring first ..."): pinned open by a click. */
function claimRuleText(): string {
  if (!document.querySelector('[data-testid="claim-rule"]')) {
    const phrase = [...container.querySelectorAll<HTMLElement>('[data-testid="city-growth"] .tip-trigger')].find((el) => el.textContent?.startsWith('claims'))!;
    act(() => phrase.click());
  }
  return document.querySelector('[data-testid="claim-rule"]')!.textContent ?? '';
}

it('shows the next claim on the territory bar, and automatic priorities without focus controls', async () => {
  const { city } = await renderCity();
  expect(claimRuleText()).toMatch(/^Ring [1-3]\. .*Claims the nearest ring first, then the best tile for your focus \(Balanced, no active build\)/);
  const next = city.detail!.next_claim!.tile;
  // Named by terrain and direction from the city, never raw coordinates, with what it adds.
  expect(container.querySelector('[data-testid="city-growth"]')!.textContent).toMatch(/Grows to 2 · claims \w+ (N|NE|E|SE|S|SW|W|NW) · unlocks Settler/);
  // The ring lives in the tooltip, never on the one-line label.
  expect(container.querySelector('[data-testid="city-growth"] .pbar-label')!.textContent).not.toMatch(/ring/);
  // What it adds sits under the bar, as icons with counts, beside the numbers.
  expect(container.querySelector('[data-testid="city-growth"] .pbar-foot-left')!.textContent).toMatch(/^Next tile\+\d+/);
  expect(container.querySelector('[data-testid="city-growth"] .pbar-foot-right')!.textContent).toMatch(/^\d+\/\d+ · \d+ turns?$/);
  expect(container.textContent).not.toContain(`q ${next.q}, r ${next.r}`);
  // The border meter is gone: territory is counted as tiles worked and owned.
  expect(container.querySelector('[data-testid="city-borders"]')).toBeNull();
  expect(container.querySelector('[data-testid="city-territory"]')!.textContent).toMatch(/^\d+ worked · \d+ owned$/);
  act(() => container.querySelector<HTMLElement>('.cp-economy summary')!.click());
  act(() => container.querySelector<HTMLElement>('.cp-territory .tip-trigger')!.click());
  expect(document.querySelector('.tip-bubble:not([hidden])')?.textContent).toContain(TILE_RELEASE_RULE);
  const labels = [...container.querySelectorAll('button')].map((b) => b.textContent);
  expect(labels).not.toContain('Balanced');
  expect(labels).not.toContain('Growth');
  expect(labels).not.toContain('Production');
});

it('previews Sparta shared kill healing against military targets', async () => {
  const { view } = await renderCity();
  const unit = view.units.find((u) => u.owner === 0 && u.kind === 'Spearman')!;
  view.players.find((p) => p.id === 0)!.faction = 'Sparta';
  const enemy = { ...unit, id: 91, owner: 255, pos: neighbors(unit.pos)[0] };
  view.units = [unit, enemy];
  const forecast: BattleForecast = { attacker_strength: { total: 20, parts: [] }, defender_strength: { total: 10, parts: [] },
    win_chance: 100, loss_chance: 0, expected_damage_to_attacker: 0, expected_damage_to_defender: 20,
    attacker_hp: 40, defender_hp: 20, ranged: false };
  act(() => root.render(<ForecastCard forecast={forecast} ranged={false} view={view} unit={unit} target={enemy.pos} />));
  expect(container.textContent).toContain('A kill heals your surviving attackers +100 health, shared');
});

it('previews build-derived yields and explains the Settler population gate', async () => {
  const { city } = await renderCity();
  const spearman = { kind: 'Unit', unit: 'Spearman' } as const;
  const option = city.detail!.options.find((o) => o.item.kind === 'Unit' && o.item.unit === 'Spearman')!;
  // Population is territory: every owned tile is worked whatever the build, so the build changes
  // only which tile the next growth claims, never today's yields.
  expect(option.yields.total).toEqual(city.detail!.yields.total);
  act(() => useGame.setState({ orders: [[{ type: 'SetProduction', city: city.id, item: spearman }]] }));
  expect(claimRuleText()).toContain('(Production, from building Spearman)');
  expect(claimRuleText()).toContain('Preview after Ready');
  expect(container.querySelectorAll('.cp-yield b')[1].textContent).toBe(String(option.yields.total.production));
  expect(container.querySelectorAll('.cp-yield b')[0].textContent).toBe(signed(option.yields.total.food - 2 * city.population));
  const summary = container.querySelector('.cp-economy > summary')!;
  // The closed Tiles summary stays compact while the city's yields remain visible.
  expect(summary.textContent).toContain('Tiles');
  expect([...document.querySelectorAll('[role="tooltip"]')].some((t) => t.textContent?.includes('8 × Production'))).toBe(true);
  // At population 1 a Settler can't be chosen: its card is greyed out with the reason and the growth forecast.
  const settler = container.querySelector('.production-choice[data-item="u:Settler"]')!;
  expect(settler.getAttribute('data-unavailable')).toBe('true');
  expect(settler.querySelector('.prod-unavailable')!.textContent).toMatch(new RegExp(`^Needs 2 population · ${city.name} grows to 2 in \\d+ turns?$`));
  act(() => useGame.setState({ orders: [[]] }));
  expect(claimRuleText()).toContain('(Balanced, no active build)');
  expect(claimRuleText()).not.toContain('Preview after Ready');
});

it('drops the claim from the territory bar when there is no free tile', async () => {
  const { city, view } = await renderCity();
  city.detail!.next_claim = null;
  act(() => root.render(<CityPanel city={city} view={view} />));
  expect(container.querySelector('[data-testid="city-growth"]')!.textContent).not.toContain('claims');
});

it('says a claim beyond the work radius only blocks rivals', async () => {
  const { city, view } = await renderCity();
  city.detail!.next_claim = { ...city.detail!.next_claim!, worked: false };
  act(() => root.render(<CityPanel city={city} view={view} />));
  expect(container.querySelector('[data-testid="city-growth"]')!.textContent).toMatch(/claims \w+ \w+/);
  expect(claimRuleText()).toMatch(/^Ring \d\. Beyond 3 tiles from the city: claimed but not worked/);
  expect(container.querySelector('[data-testid="city-growth"] .pbar-foot-left')!.textContent).toBe('Next tile · no yield · blocks rivals');
});

it('has a Buy a tile button that opens placement, then shows the queued tile with its cost chip and a Cancel', async () => {
  const { client, city: before } = await renderCity();
  client.debugState().players[0].gold = 500;
  const view = client.view(0), city = view.cities.find((c) => c.id === before.id)!, d = city.detail!;
  const edit = vi.fn((change: (orders: any[]) => any[]) => useGame.setState({ orders: [change(useGame.getState().orders[0])] }));
  act(() => useGame.setState({ views: [view], editOrders: edit as any }));
  act(() => root.render(<CityPanel city={city} view={view} />));
  const first = d.buyable_tiles[0];
  const control = () => container.querySelector<HTMLElement>('[data-testid="city-buy-tile"]')!;
  const button = control() as HTMLButtonElement;
  expect(button.tagName).toBe('BUTTON');
  expect(button.textContent).toBe(`Buy a tile · from ${first.price} Gold`);
  expect(button.disabled).toBe(false);
  expect(useGame.getState().place).toBeNull();
  act(() => button.click());
  expect(useGame.getState().place).toMatchObject({ city: city.id, building: null, intent: 'tile' });
  expect(container.querySelector('[data-testid="city-placement"]')!.textContent).toContain(`Click a lit tile to buy it for ${city.name}`);
  act(() => useGame.getState().set({ place: null }));
  act(() => useGame.setState({ orders: [[{ type: 'BuyTile', city: city.id, tile: first.tile }]] }));
  expect(control().textContent).toContain('Buying ');
  expect(control().querySelector('.cost-chip')!.textContent).toContain(`−${first.price}`);
  const cancel = [...control().querySelectorAll('button')].find((b) => b.textContent === 'Cancel')!;
  act(() => cancel.click());
  expect(useGame.getState().orders[0]).toEqual([]);
  expect(control().tagName).toBe('BUTTON');
  expect(control().textContent).toContain('Buy a tile · from');
});

it('disables Buy a tile and says why in place: short of Gold, or a core blocker', async () => {
  const { client, city: before } = await renderCity();
  client.debugState().players[0].gold = 5;
  const view = client.view(0), city = view.cities.find((c) => c.id === before.id)!;
  act(() => useGame.setState({ views: [view] }));
  act(() => root.render(<CityPanel city={city} view={view} />));
  const button = () => container.querySelector<HTMLButtonElement>('button[data-testid="city-buy-tile"]')!;
  expect(button().disabled).toBe(true);
  const reason = () => container.querySelector('.city-borders-reason')?.textContent;
  expect(button().textContent).toBe('Buy a tile');
  expect(reason()).toMatch(/^Need \d+ more Gold$/);
  const blocked = { ...city, detail: { ...city.detail!, buy_tile_blocker: `${city.name} already bought something this turn` } };
  act(() => root.render(<CityPanel city={blocked} view={view} />));
  expect(button().textContent).toBe('Buy a tile');
  expect(reason()).toBe('Already bought this turn');
  const empty = { ...city, detail: { ...city.detail!, buyable_tiles: [] } };
  act(() => root.render(<CityPanel city={empty} view={view} />));
  expect(reason()).toBe('No free tile in reach');
  const wording = { ...city, detail: { ...city.detail!, buy_tile_blocker: `${city.name} has no free tile to buy within 3 tiles of the city.` } };
  act(() => root.render(<CityPanel city={wording} view={view} />));
  expect(reason()).toBe('No free tile in reach');
  const odd = { ...city, detail: { ...city.detail!, buy_tile_blocker: 'Something unmapped and rather long that must wrap and not be clipped' } };
  act(() => root.render(<CityPanel city={odd} view={view} />));
  expect(reason()).toBe('Something unmapped and rather long that must wrap and not be clipped');
});

it('shows starvation in red with the turns until a population is lost, from the core forecast', async () => {
  const { city, view } = await renderCity();
  Object.assign(city, { population: 3 });
  Object.assign(city.detail!, { turns_to_grow: null, turns_to_starve: 4 });
  act(() => root.render(<CityPanel city={city} view={view} />));
  const growth = container.querySelector('[data-testid="city-growth"]')!;
  expect(growth.classList.contains('bad')).toBe(true);
  expect(growth.textContent).toContain('Starving');
  expect(growth.textContent).toContain('−1 population in 4');
  act(() => growth.closest<HTMLElement>('.tip-trigger')!.click());
  expect(document.querySelector('.tip-bubble:not([hidden])')?.textContent).toContain('Build a Farm or a Granary to add Food.');
});

it('shows a damaged city healing 20 a turn, and says why when an enemy is adjacent', async () => {
  const { city, view } = await renderCity();
  expect(container.querySelector('[data-testid="city-heal"]')).toBeNull();
  Object.assign(city, { hp: 110 });
  Object.assign(city.detail!, { turns_to_heal: 5, heal_blocked: false });
  act(() => root.render(<CityPanel city={city} view={view} />));
  const heal = container.querySelector('[data-testid="city-heal"]')!;
  expect(heal.textContent).toContain('Healing');
  expect(heal.textContent).toContain('Healing · +20 a turn · full in 5 turns');
  expect(heal.querySelector('[role="progressbar"]')?.getAttribute('aria-valuenow')).toBe('110');
  expect(heal.classList.contains('paused')).toBe(false);
  Object.assign(city.detail!, { heal_blocked: true });
  act(() => root.render(<CityPanel city={city} view={view} />));
  expect(container.querySelector('[data-testid="city-heal"]')!.textContent).toContain('Not healing: an enemy military unit is adjacent');
  expect(container.querySelector('[data-testid="city-heal"]')!.classList.contains('paused')).toBe(true);
});

it('names what the next population unlocks on the growth bar, with a tooltip explaining why', async () => {
  const { city } = await renderCity();
  // Population 1: the core lists the Settler at 2.
  expect(city.detail!.population_unlocks).toContainEqual({ population: 2, item: { kind: 'Unit', unit: 'Settler' } });
  const growth = container.querySelector('[data-testid="city-growth"]')!;
  expect(growth.textContent).toMatch(/^Grows to 2 · claims .* · unlocks Settler/);
  expect(growth.querySelector('[role="progressbar"]')!.getAttribute('aria-valuemax')).toBe(String(city.detail!.growth_threshold));
  // The unlock note on the label itself (distinct from the growth-math breakdown that wraps the
  // whole bar) explains the rule generically: a Settler costs population, so it needs a bigger city.
  const unlockTip = [...growth.querySelectorAll<HTMLElement>('.tip-trigger')].find((el) => el.textContent?.startsWith('unlocks'))!;
  act(() => unlockTip.click());
  expect(document.querySelector('[aria-label="Breakdown"]')?.textContent)
    .toContain('Settlers cost 1 population, so a city must reach size 2 before it can build one.');
});

it('counts the tiles it works against the tiles it owns', async () => {
  const { city } = await renderCity();
  const counts = container.querySelector('[data-testid="city-territory"]')!;
  expect(counts.textContent).toBe(`${city.detail!.yields.worked.length} worked · ${city.detail!.territory.length - 1} owned`);
});

it('shows Standard growth tooltip constants', async () => {
  const { city, view } = await renderCity();
  act(() => root.render(<CityPanel city={city} view={view} />));
  // The growth breakdown is rich: it opens pinned on click rather than as a hover bubble.
  const growth = [...container.querySelectorAll<HTMLElement>('.tip-trigger')].find((el) => el.querySelector('[data-testid="city-growth"]'))!;
  act(() => growth.click());
  expect(document.querySelector('[aria-label="Breakdown"]')?.textContent).toContain(`The first growth costs ${FIRST_GROWTH} Food. After that, each growth costs 30 Food plus 16 for every population above 1`);
  expect(document.querySelector('[aria-label="Breakdown"]')?.textContent).not.toContain('Nutrient Synthesis');
});

it('shows every Buy price as a spend chip on the button, and the chosen buy against the treasury (G16)', async () => {
  const { city, view } = await renderCity();
  const buy = container.querySelector<HTMLButtonElement>('.production-choice[data-item="u:Spearman"] .btn-buy')!;
  const price = Number(buy.getAttribute('aria-label')!.match(/for (\d+) Gold/)![1]);
  expect(buy.querySelector('.cost-chip .sr-only')!.textContent).toBe(`Costs ${price} Gold`);
  act(() => {
    useGame.setState({ orders: [[{ type: 'BuyItem', city: city.id, item: { kind: 'Unit', unit: 'Spearman' } }]] });
    root.render(<CityPanel city={city} view={view} />);
  });
  const chosen = container.querySelector('.production-choice[data-item="u:Spearman"] .btn-buy.active .cost-chip .sr-only')!;
  // This fixture's treasury is short: the chosen card shows the spend against the treasury.
  const after = view.economy.gold - price;
  expect(chosen.textContent).toBe(`Costs ${price} Gold (${view.economy.gold} → ${after < 0 ? `−${-after}, debt` : after})`);
});

it('a Buy names the city in a notice with Undo, and Undo takes the purchase back (misclick guard)', async () => {
  const { city } = await renderCity('Switzerland', 5000);
  const buy = container.querySelector<HTMLButtonElement>('.production-choice[data-item="u:Spearman"] .btn-buy')!;
  act(() => buy.click());
  const s = useGame.getState();
  expect(s.orders[0]).toContainEqual({ type: 'BuyItem', city: city.id, item: { kind: 'Unit', unit: 'Spearman' } });
  expect(s.notice?.text).toMatch(new RegExp(`^Buying Spearman in ${city.name} for \\d+ Gold\\.$`));
  const { undoNotice } = await import('../notice');
  act(() => undoNotice());
  expect(useGame.getState().orders[0].some((o) => o.type === 'BuyItem')).toBe(false);
});

it.each([false, true])('disables production and Buy after the game ends, including a pending purchase (%s)', async (buying) => {
  const { city, view } = await renderCity();
  city.detail!.producing = city.detail!.options[0].item;
  city.detail!.buy_cost = 0;
  const edit = vi.fn();
  act(() => {
    useGame.setState({ editOrders: edit, orders: [buying ? [{ type: 'BuyItem', city: city.id, item: { kind: 'Unit', unit: 'Spearman' } }] : []] });
    root.render(<CityPanel city={city} view={{ ...view, finished: true }} />);
  });
  const controls = [...container.querySelectorAll<HTMLButtonElement>('.prod, .btn-buy')];
  expect(controls.length).toBeGreaterThan(1);
  controls.forEach((button) => {
    expect(button.disabled).toBe(true);
    act(() => button.click());
  });
  expect(edit).not.toHaveBeenCalled();
});

it('explains a building once: the tile-focus context appears a single time, only changed yields get sources (A14)', async () => {
  const client = new FixtureClient();
  await client.newGame(await client.standardSetup('42', 'Switzerland', 'India', 'Ada', 'Bo'));
  client.endTurn({});
  client.debugState().players[0].techs.push('BronzeWorking');
  const view = client.view(0);
  const city = view.cities.find((c) => c.owner === 0)!;
  useGame.setState({ client, views: [view], humans: [0], orders: [[]], errors: [[]], seat: 0 });
  act(() => root.render(<CityPanel city={city} view={view} />));
  const row = [...container.querySelectorAll<HTMLElement>('.prod-list .tip-trigger')].find((el) => el.textContent?.includes('Workshop'))!;
  const info = row.querySelector<HTMLButtonElement>('[data-tip-info]');
  expect(info, 'a building with a forecast offers its breakdown from an ⓘ').not.toBeNull();
  act(() => info!.click());
  const pinned = document.querySelector<HTMLElement>('[aria-label="Breakdown"]')!;
  expect(pinned.textContent!.match(/This city works every owned tile within its work radius/g)).toHaveLength(1);
  expect(pinned.textContent).toContain('After completion');
  expect(pinned.querySelectorAll('.bd-title').length).toBeLessThanOrEqual(3);
});


it('Build replaces the current item; Queue and Shift-click append without spending earned Production', async () => {
  const { client, city } = await renderCity();
  const before = client.debugState().cities[0].prod;
  const button = (label: string) => container.querySelector<HTMLButtonElement>(`[aria-label="${label}"]`)!;
  act(() => button('Build Spearman').click());
  act(() => button('Queue Scout').click());
  act(() => button('Build Archer').click());
  expect(useGame.getState().orders[0]).toEqual([{ type: 'SetProductionQueue', city: city.id, queue: [
    { kind: 'Unit', unit: 'Archer' }, { kind: 'Unit', unit: 'Scout' },
  ] }]);
  act(() => button('Build Spearman').dispatchEvent(new MouseEvent('click', { bubbles: true, shiftKey: true })));
  expect(useGame.getState().orders[0][0]).toMatchObject({ queue: [
    { kind: 'Unit', unit: 'Archer' }, { kind: 'Unit', unit: 'Scout' }, { kind: 'Unit', unit: 'Spearman' },
  ] });
  expect(client.debugState().cities[0].prod).toBe(before);
});

it('memoizes speculative validation and keeps the city head to its Gold line', async () => {
  const { client, city, view } = await renderCity();
  const validate = vi.spyOn(client, 'validate');
  act(() => root.render(<CityPanel city={city} view={view} />));
  expect(validate).not.toHaveBeenCalled();
  const status = container.querySelector('.purchase-status')!.textContent!;
  expect(status).toMatch(/\d+ Gold \([+−]\d+\/turn\) to spend/);
  expect(status).not.toMatch(/need more Gold|Settler needs population 2/);
  expect(container.querySelectorAll('.production-choice .buy-need, .production-choice .hint')).toHaveLength(0);
});

it('keeps worked and owned counts in the closed economy disclosure and growth as a bar', async () => {
  const { city } = await renderCity();
  const disclosure = container.querySelector<HTMLDetailsElement>('.cp-economy')!;
  expect(disclosure.open).toBe(false);
  expect(disclosure.querySelector('summary')!.textContent).toBe(`Tiles ${city.detail!.yields.worked.length} worked · ${city.detail!.territory.length - 1} owned`);
  // The counts sit inside the disclosure; Growth (the territory bar) stays outside it as a bar.
  expect(disclosure.querySelector('[data-testid="city-territory"]')).not.toBeNull();
  expect(disclosure.querySelector('[data-testid="city-growth"]')).toBeNull();
  // Growth sits outside the disclosure, as a bar with turns.
  expect(container.querySelector('[data-testid="city-growth"]')!.textContent).toContain(`${city.detail!.turns_to_grow} turn`);
});

it('shows all four city yields outside Tiles, keeps their source tips, and opens city comparison', async () => {
  const { city } = await renderCity();
  const disclosure = container.querySelector<HTMLDetailsElement>('.cp-economy')!;
  const row = container.querySelector<HTMLElement>('[data-testid="city-yields"]')!;
  expect(disclosure.open).toBe(false);
  expect(disclosure.contains(row)).toBe(false);
  expect(row.getAttribute('aria-label')).toBe('City yields per turn');
  expect([...row.querySelectorAll('.cp-yield-l')].map((el) => el.textContent)).toEqual(['Food', 'Production', 'Gold', 'Science']);
  const totals = city.detail!.yields.total;
  const net = totals.food - 2 * city.population;
  expect([...row.querySelectorAll('.cp-yield b')].map((el) => el.textContent)).toEqual(
    [net > 0 ? `+${net}` : net < 0 ? `−${-net}` : '0', ...(['production', 'gold', 'science'] as const).map((k) => String(totals[k]))]);
  expect(row.querySelectorAll('.tip-trigger')).toHaveLength(4);
  expect(container.querySelector('[data-testid="city-growth"] .pbar-foot-left')!.textContent).toContain('Next tile');
  act(() => container.querySelector<HTMLButtonElement>('.cp-compare')!.click());
  expect(useGame.getState().citiesOpen).toBe(true);
});

it('shows the actual Production per turn beside stored progress', async () => {
  const { city, view } = await renderCity();
  const option = city.detail!.options.find((o) => o.item.kind === 'Unit' && o.item.unit === 'Spearman')!;
  Object.assign(city.detail!, { producing: option.item, production_stored: 10, production_cost: option.cost, turns_to_complete: 5 });
  act(() => root.render(<CityPanel city={city} view={view} />));
  expect(container.querySelector('.cp-now-turns')!.textContent).toContain(`10/${option.cost} stored · +${city.detail!.yields.total.production} Production/turn · 5 turns`);
});

it('names the real reason a waiting queue head waits, and keeps it when choosing another build', async () => {
  const { city, view } = await renderCity();
  const spearman = { kind: 'Unit', unit: 'Spearman' } as const;
  const cost = city.detail!.options.find((o) => o.item.kind === 'Unit' && o.item.unit === 'Spearman')!.cost;
  // A queued unit with no free spawn tile: the only waiting state besides a resource shortage.
  Object.assign(city.detail!, { producing: null, production_queue: [spearman], production_stored: 28, idle_production_cap: cost, idle_production_waste: 0 });
  act(() => root.render(<CityPanel city={city} view={view} />));
  expect(container.querySelector('.cp-now-name')!.textContent).toBe('Spearman');
  expect(container.querySelector('.cp-now-tag')!.textContent).toBe('waiting for room to appear');
  // Production is held up to the waiting item's cost, and the panel shows that cap.
  expect(container.querySelector('.cp-now-turns')!.textContent).toBe(`28/${cost} held`);
  // The waiting item is shown once, in Now producing: the queue below lists only what follows it.
  expect(container.querySelector('.queue-name')).toBeNull();
  const warning = container.querySelector('.production-warning')!.textContent!;
  expect(warning).toContain(`Spearman is waiting: no free tile in or beside ${city.name} for the new unit`);
  expect(warning).toContain(`Production is held for it meanwhile (28/${cost})`);
  expect(warning).not.toContain('Queued items are blocked');
  act(() => container.querySelector<HTMLButtonElement>('[aria-label="Build Archer"]')!.click());
  expect(useGame.getState().orders[0]).toEqual([{ type: 'SetProductionQueue', city: city.id, queue: [
    { kind: 'Unit', unit: 'Archer' }, spearman,
  ] }]);
  expect(container.querySelector('.production-warning')).toBeNull();
});

it('a waiting item short of a strategic resource says how long, and that it starts by itself', async () => {
  const { city, view } = await renderCity();
  const archer = city.detail!.options.find((o) => o.item.kind === 'Unit' && o.item.unit === 'Archer')!;
  archer.resource_blocker = 'Needs 10 Iron: you have 4, +2 per turn → 3 turns';
  Object.assign(city.detail!, { producing: null, production_queue: [archer.item], production_stored: 12, idle_production_cap: archer.cost, idle_production_waste: 0 });
  act(() => root.render(<CityPanel city={city} view={view} />));
  expect(container.querySelector('.cp-now-tag')!.textContent).toBe('waiting for Iron');
  expect(container.querySelector('.production-warning')!.textContent).toBe(
    `Archer is waiting: Needs 10 Iron: you have 4, +2 per turn → 3 turns. It starts by itself once the resource is there. Production is held for it meanwhile (12/${archer.cost}); a different build started now would use it instead.`);
});

it('greys out an unavailable Settler: clicking Build or + Queue adds no order and says why on the card', async () => {
  const { city } = await renderCity();
  const edit = vi.fn();
  act(() => useGame.setState({ editOrders: edit }));
  const card = container.querySelector('.production-choice[data-item="u:Settler"]')!;
  const build = card.querySelector<HTMLButtonElement>('[aria-label="Build Settler"]')!;
  // Looks disabled before hover: aria-disabled, no Build chevron, Buy disabled.
  expect(build.getAttribute('aria-disabled')).toBe('true');
  expect(card.querySelector('.prod-go')!.textContent).toBe('Not yet');
  expect(card.querySelector<HTMLButtonElement>('.btn-buy')!.disabled).toBe(true);
  act(() => build.click());
  act(() => card.querySelector<HTMLButtonElement>('[aria-label="Queue Settler"]')!.click());
  expect(edit).not.toHaveBeenCalled();
  const turns = city.detail!.turns_to_grow!;
  expect(card.querySelector('.production-notice.bad')!.textContent).toBe(
    `Can't build Settler yet: needs 2 population. ${city.name} grows to 2 in ${turns} turn${turns === 1 ? '' : 's'}.`);
});

it('hides faction exclusions in one rule note while population blockers remain Not yet', async () => {
  const { city, view } = await renderCity('Mongols');
  // These historical reason strings intentionally differ from the current core wording. The
  // permanent label should follow faction eligibility, not a display string comparison.
  city.detail!.unavailable.push(
    { item: { kind: 'Building', building: 'RiceStorehouse' }, cost: 60, reason: 'Mongols keeps its cities open and cannot build the Rice Storehouse', production_banked: 0 },
  );
  act(() => root.render(<CityPanel city={city} view={view} />));
  for (const key of ['b:RiceStorehouse']) {
    expect(container.querySelector(`.production-choice[data-item="${key}"]`)).toBeNull();
  }
  expect([...container.querySelectorAll('.prod-faction-rule')].filter((line) => line.textContent?.includes('Mongols keeps its cities open'))).toHaveLength(1);
  const settler = container.querySelector('.production-choice[data-item="u:Settler"]')!;
  expect(settler.querySelector('.prod-go')!.textContent).toBe('Not yet');
  expect(settler.querySelector('.prod-unavailable')!.textContent).toMatch(/Needs 2 population/);
});

it('says a non-growing city must work more Food before a Settler', async () => {
  const { city, view } = await renderCity();
  city.detail!.turns_to_grow = null;
  city.detail!.yields.total.food = 2;
  act(() => root.render(<CityPanel city={city} view={view} />));
  expect(container.querySelector('.production-choice[data-item="u:Settler"] .prod-unavailable')!.textContent)
    .toBe(`Needs 2 population · ${city.name} is not growing: work more Food or build something else first`);
});

it('shows banked progress on cards and in the city, and what stays banked when switching', async () => {
  const { city, view } = await renderCity();
  const d = city.detail!;
  const walls = { kind: 'Building', building: 'Granary' } as const;
  const granary = d.options.find((o) => o.item.kind === 'Building' && o.item.building === 'Granary')!;
  const archer = d.options.find((o) => o.item.kind === 'Unit' && o.item.unit === 'Archer')!;
  const spearman = d.options.find((o) => o.item.kind === 'Unit' && o.item.unit === 'Spearman')!;
  // Building a Spearman (12 so far); the Granary has 20 banked from an earlier switch.
  Object.assign(d, { producing: spearman.item, production_stored: 12, production_cost: spearman.cost, production_banked: [{ item: walls, progress: 20 }] });
  Object.assign(granary, { production_banked: 20, production_stored: 20 });
  Object.assign(archer, { production_banked: 0, production_stored: 0 });
  act(() => root.render(<CityPanel city={city} view={view} />));
  const cost = (key: string) => container.querySelector(`.production-choice[data-item="${key}"] .prod-cost`)!.textContent;
  expect(cost('u:Spearman')).toBe(`12/${spearman.cost} so far`);
  expect(cost('b:Granary')).toBe(`20/${granary.cost} banked`);
  expect(cost('u:Archer')).toContain(String(archer.cost));
  expect(container.querySelector('[data-testid="city-banked"]')!.textContent).toBe(`Banked Granary 20/${granary.cost}`);
  expect(container.querySelector('[data-testid="production-switch"]')).toBeNull();
  act(() => container.querySelector<HTMLButtonElement>('[aria-label="Build Archer"]')!.click());
  expect(container.querySelector('[data-testid="production-switch"]')!.textContent)
    .toBe(`${city.name} keeps the 12 Production already put into the Spearman; Archer starts at 0/${archer.cost}.`);
  expect(cost('u:Spearman')).toBe(`12/${spearman.cost} banked`);
  act(() => container.querySelector<HTMLButtonElement>('[aria-label="Build Granary"]')!.click());
  expect(container.querySelector('[data-testid="production-switch"]')!.textContent)
    .toBe(`${city.name} keeps the 12 Production already put into the Spearman; Granary resumes at 20/${granary.cost}.`);
  // The resumed item leaves the Banked line while it is the pending build.
  expect(container.querySelector('[data-testid="city-banked"]')).toBeNull();
  // The rule is one hover away on the picker title.
  expect(document.body.textContent).toContain('Its progress stays banked with it in this city and resumes when you build it again');
});

it('counts repeats on the Queue button and the queue list, inserting no line and stacking no badges', async () => {
  const { city } = await renderCity();
  vi.useFakeTimers();
  try {
    const queue = () => container.querySelector<HTMLButtonElement>('[aria-label="Queue Scout"]')!;
    const card = () => container.querySelector('.production-choice[data-item="u:Scout"]')!;
    expect(container.querySelector('[data-testid="production-queue"]')!.textContent).toContain('add from the list below');
    act(() => queue().click());
    expect(queue().textContent).toBe(' Queued #1');
    expect(card().querySelector('.production-notice')).toBeNull();
    act(() => queue().click());
    act(() => queue().click());
    expect(queue().textContent).toBe(' Queued ×3');
    expect(card().querySelectorAll('.queue-badge')).toHaveLength(1);
    expect(card().querySelector('.queue-badge')!.textContent).toBe('×3');
    expect(container.querySelectorAll('[data-testid="production-queue"] li')).toHaveLength(2);
    act(() => { vi.advanceTimersByTime(1600); });
    expect(queue().textContent).toBe('+ Queue');
  } finally { vi.useRealTimers(); }
  void city;
});

it('asks for a new build when the city banked an item it can no longer make', async () => {
  const { city, view } = await renderCity();
  const d = city.detail!;
  d.production_banked = [{ item: { kind: 'Unit', unit: 'Settler' }, progress: 38 }];
  d.unavailable = d.unavailable.map((u) => u.item.kind === 'Unit' && u.item.unit === 'Settler' ? { ...u, production_banked: 38 } : u);
  act(() => root.render(<CityPanel city={city} view={view} />));
  const turns = d.turns_to_grow!;
  expect(container.querySelector('[data-testid="production-choose"]')!.textContent)
    .toBe(`Choose a new build. ${city.name} keeps the 38 Production already put into the Settler until ${city.name} grows to 2 in ${turns} turn${turns === 1 ? '' : 's'}.`);
  expect(container.querySelector('.production-choice[data-item="u:Settler"] .prod-cost')!.textContent).toMatch(/^38\/\d+ banked$/);
  // Stated once: the choose message names the parked Settler, so the Banked line leaves it out.
  expect(container.querySelector('[data-testid="city-banked"]')).toBeNull();
});

it('lists the Improvements group by tile and lights the hovered tile on the map', async () => {
  const client = new FixtureClient();
  await client.newGame(await client.standardSetup('42', 'Switzerland', 'India', 'Ada', 'Bo'));
  client.endTurn({});
  const state = client.debugState();
  const city = state.cities.find((c) => c.owner === 0)!;
  const wheat = neighbors(city.pos)[0], iron = neighbors(city.pos)[2];
  state.resource = state.terrain.map(() => null);
  state.resource[tileIndex(wheat, state.width, state.height)] = 'Wheat';
  state.resource[tileIndex(iron, state.width, state.height)] = 'Iron';
  const view = client.view(0);
  const c = view.cities.find((c) => c.id === city.id)!;
  useGame.setState({ client, views: [view], humans: [0], orders: [[]], errors: [[]], seat: 0 });
  act(() => root.render(<CityPanel city={c} view={view} />));
  const group = container.querySelector('[data-testid="improvements-group"]')!;
  // Only the Wheat: the Iron is hidden until IronWorking, so it can't be targeted.
  expect(group.querySelectorAll('.production-choice').length).toBe(1);
  expect(group.textContent).toMatch(/Farm, Wheat1\u00a0tile\u00a0(N|NE|E|SE|S|SW|W|NW)/);
  expect(group.textContent).toContain('+1 Food when built');
  // The card's own line says what it adds, not a generic sentence (user, 2026-11-01).
  expect(group.querySelector('.prod-effect')!.getAttribute('title')).toBe('+1 Food');
  expect(group.textContent).toContain('Later: Irrigated Farm with Crop Rotation, +1 Food');
  expect(group.textContent).not.toContain('Food · unlocks');
  expect(group.textContent).toContain('No upkeep');
  expect(group.textContent).toContain('1 resource tile unimproved');
  expect(group.textContent).not.toMatch(/\(-?\d+, ?-?\d+\)/);
  // Hovering the row sets the map highlight, leaving clears it.
  const row = group.querySelector<HTMLElement>('.production-choice')!;
  act(() => { row.dispatchEvent(new Event('pointerover', { bubbles: true })); row.dispatchEvent(new PointerEvent('pointerenter')); });
  act(() => { row.dispatchEvent(new FocusEvent('focusin', { bubbles: true })); });
  expect(useGame.getState().improve.hover).toEqual(wheat);
});

async function greatWorkCity(faction: 'Switzerland' | 'India') {
  const client = new FixtureClient();
  await client.newGame(await client.standardSetup('42', faction, 'Byzantium', 'Ada', 'Bo'));
  client.endTurn({});
  const state = client.debugState();
  const city = state.cities.find((c) => c.owner === 0)!;
  city.population = 8;
  city.buildings = ['Factory'];
  state.players[0].techs = [...TECHS];
  state.players[0].gold = 100_000;
  const view = client.view(0);
  const c = view.cities.find((x) => x.id === city.id)!;
  useGame.setState({ client, views: [view], humans: [0], orders: [[]], errors: [[]], seat: 0 });
  act(() => root.render(<CityPanel city={c} view={view} />));
  return { client, city: c };
}

it.each(['Switzerland', 'India'] as const)('%s sees world wonders as Production only', async (faction) => {
  await greatWorkCity(faction);
  const section = container.querySelector('.cp-great-works')!;
  expect(section.querySelector(':scope > summary')).toBeNull();
  expect(section.querySelector('.cp-great-works-head')?.textContent).toContain('World wonders');
  expect(section.querySelectorAll('.production-choice')).toHaveLength(7);
  expect(section.querySelectorAll('.hint')).toHaveLength(1);
  expect(greatWorkRules(faction)).not.toContain('World wonders must be built with Production');
  expect(section.textContent).toContain('Wonders are built with Production only');
  expect(section.querySelectorAll('.btn-buy')).toHaveLength(0);
  expect(section.querySelectorAll('.prod-buy-why')).toHaveLength(0);
  expect(section.querySelectorAll('.queue-add').length).toBe(section.querySelectorAll('.production-choice:not(.wonder-locked)').length);
});

it('keeps all seven wonders visible with a reason when their technology is unknown', async () => {
  await renderCity('India');
  const section = container.querySelector('.cp-great-works')!;
  expect(section.querySelectorAll('.production-choice')).toHaveLength(7);
  expect(section.querySelectorAll('.wonder-locked')).toHaveLength(7);
  expect(section.querySelector('[data-wonder="Pyramids"] .prod-unavailable')?.textContent).toContain('Needs Masonry');
});

it('shows Walls as Production only in the build list and disables its Buy control with the reason', async () => {
  const { client, city } = await renderCity('India');
  const state = client.debugState();
  state.players[0].techs.push('Masonry');
  const view = client.view(0);
  const updated = view.cities.find((c) => c.id === city.id)!;
  act(() => useGame.setState({ views: [view] }));
  act(() => root.render(<CityPanel city={updated} view={view} />));

  const card = container.querySelector('.production-choice[data-item="b:Walls"]')!;
  const buy = card.querySelector<HTMLButtonElement>('.btn-buy')!;
  expect(buy.disabled).toBe(true);
  expect(card.querySelector('.production-refusal')?.textContent).toBe("Walls must be built with Production; they can't be bought.");
  expect(container.querySelector('.purchase-status')?.textContent).not.toContain("Walls must be built with Production; they can't be bought.");
  expect(card.querySelector<HTMLButtonElement>('.prod')?.disabled).toBe(false);
  act(() => buy.click());
  expect(useGame.getState().orders[0]).toEqual([]);
});

it('the Raze confirm of a city holding a world wonder lists what burns, with the VP', async () => {
  const { city, view } = await renderCity();
  city.detail!.captured = true;
  city.detail!.raze_blocker = undefined;
  city.detail!.buildings = ['Granary', 'Workshop'];
  const site = tileIndex(neighbors(city.pos)[1], view.width, view.height);
  view.tiles[site] = { ...view.tiles[site], landmark: { building: 'Pyramids', city: city.id, of: 0 } };
  act(() => root.render(<CityPanel city={city} view={view} />));
  const idle = container.querySelector('[data-testid="city-raze"]')!.textContent!;
  expect(idle).toContain('destroys the Pyramids (−2 VP)');
  act(() => container.querySelector<HTMLButtonElement>('[data-testid="city-raze"] button')!.click());
  const confirm = container.querySelector('[data-testid="raze-confirm"]')!;
  expect([...confirm.querySelectorAll('li')].map((li) => li.textContent)).toEqual(['the Pyramids (−2 VP)', '2 buildings: Granary, Workshop']);
  const confirmButton = [...confirm.querySelectorAll('button')].find((b) => b.textContent === 'Confirm Raze')!;
  expect(confirmButton.className).toContain('danger');
  expect(confirmButton.className).not.toBe([...confirm.querySelectorAll('button')].find((b) => b.textContent === 'Cancel')!.className);
});

it('stops the active build in one click, starting the next queued item', async () => {
  const { city, view } = await renderCity();
  const d = city.detail!;
  const first = d.options[0].item, second = d.options[1].item;
  useGame.setState({ orders: [[{ type: 'SetProductionQueue', city: city.id, queue: [first, second] }]] });
  act(() => root.render(<CityPanel city={city} view={view} />));
  const stop = container.querySelector<HTMLButtonElement>('[data-testid="production-stop"]')!;
  expect(stop.getAttribute('title')).toBeNull();
  expect(stop.closest('.tip-trigger')).not.toBeNull();
  act(() => stop.click());
  const set = useGame.getState().orders[0].find((o) => o.type === 'SetProductionQueue');
  expect(set).toEqual({ type: 'SetProductionQueue', city: city.id, queue: [second] });
});

it('leads the unit list with the best unit the city can build, the one short of Iron beneath it', async () => {
  const { city, view } = await renderCity();
  const spear = city.detail!.options.find((o) => o.item.kind === 'Unit' && o.item.unit === 'Spearman')!;
  const pike = { ...spear, item: { kind: 'Unit' as const, unit: 'Pikeman' as const }, cost: spear.cost * 2, resource_blocker: 'Needs 1 Iron: you have none yet.', supply_need: 'Iron' as const };
  city.detail!.options.push(pike);
  act(() => root.render(<CityPanel city={city} view={view} />));
  const card = container.querySelector('.production-choice[data-item="u:Spearman"]')!;
  expect(card.closest('details')).toBeNull();
  const better = card.parentElement!.querySelector('.unit-better[data-better="u:Pikeman"]')!;
  expect(better.textContent).toContain('Reisläufer needs Iron'); // the Swiss Pikeman
  expect(container.querySelector('.production-choice[data-item="u:Pikeman"]')).toBeNull();
  // Nothing queued: it can't be queued yet, so the button is off.
  expect(better.querySelector<HTMLButtonElement>('[aria-label="Queue Reisläufer"]')!.disabled).toBe(true);
  // Queued behind a build, its queue row says what it waits for instead of a turn count.
  Object.assign(city.detail!, { producing: spear.item, production_queue: [pike.item] });
  act(() => root.render(<CityPanel city={city} view={view} />));
  expect(container.querySelector('[data-testid="production-queue"] li .queue-turns')!.textContent).toBe('needs Iron');
});
