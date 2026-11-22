// @vitest-environment happy-dom
import { expect, it } from 'vitest';
import { claimSummary, growthProgress, productionProgress, territoryCounts } from './cityProgress';
import { bannerMeters } from './map/render';
import { FixtureClient } from './test/fixtureEngine';
import { progressFixture } from './test/progressFixture';
import type { CityDetail, CityView, PlayerView, ProductionOption } from './types';

function fixtureView() {
  const { fixture, capital, second } = progressFixture();
  const client = new FixtureClient();
  client.load(JSON.stringify(fixture));
  const view = client.view(0);
  const city = (id: number) => view.cities.find((c) => c.id === id)!;
  return { view, capital: city(capital), second: city(second) };
}

it('reads growth and starvation from the core forecast, with what the next population unlocks', () => {
  const { capital, second } = fixtureView();
  expect(growthProgress(capital)).toMatchObject({ state: 'growing', turns: 1, unlocks: [{ kind: 'Unit', unit: 'Settler' }] });
  expect(growthProgress(second)).toMatchObject({ state: 'starving', turns: second.detail!.turns_to_starve, unlocks: [] });
});

it('names the tile the next growth claims, and says when it only blocks rivals', () => {
  const { view, capital } = fixtureView();
  const claim = claimSummary(capital, view)!;
  expect(claim.tile).toEqual(capital.detail!.next_claim!.tile);
  expect(claim.text).toMatch(/^claims [A-Z][a-z]+ (N|NE|E|SE|S|SW|W|NW)$/);
  expect(claim.effect).toMatch(/^[+-]\d+ \w+/);
  expect(claim.worked).toBe(true);
  // Beyond the work radius the tile is claimed but not worked: it only blocks rivals.
  const far = structuredClone(capital);
  far.detail!.next_claim = { ...far.detail!.next_claim!, worked: false };
  expect(claimSummary(far, view)!.effect).toBe('no yield · blocks rivals');
  expect(claimSummary(far, view)!.yields).toEqual({ food: 0, production: 0, gold: 0, science: 0 });
  far.detail!.next_claim = null;
  expect(claimSummary(far, view)).toBeNull();
});

it('previews a far claim being worked by the nearest own city after the handoff', () => {
  const { view, capital } = fixtureView();
  const source = structuredClone(capital);
  source.pos = { q: 13, r: 14 };
  source.detail!.next_claim = {
    tile: { q: 12, r: 11 }, yields: { food: 2, production: 0, gold: 0, science: 0 }, worked: false,
  };
  const recipient = { ...structuredClone(capital), id: 27, name: 'Sparta Colony 11', pos: { q: 12, r: 10 } };
  const claim = claimSummary(source, { ...view, cities: [source, recipient] })!;
  expect(claim.handoffTo).toBe('Sparta Colony 11');
  expect(claim.effect).toBe('worked by Sparta Colony 11 after Ready');
  expect(claim.yields).toEqual({ food: 0, production: 0, gold: 0, science: 0 });
});

it('counts tiles worked against tiles owned (the centre belongs to neither count)', () => {
  const { capital } = fixtureView();
  const counts = territoryCounts(capital.detail!);
  expect(counts.worked).toBe(capital.detail!.yields.worked.length);
  expect(counts.owned).toBe(capital.detail!.territory.length - 1);
  expect(counts.owned).toBeGreaterThanOrEqual(counts.worked);
});

it('banners carry growth and production meters only for your own cities', () => {
  const { view, capital, second } = fixtureView();
  expect(bannerMeters(capital)).toEqual([
    { kind: 'growth', fraction: expect.any(Number), text: '1', bad: false, paused: false },
    { kind: 'production', fraction: expect.any(Number), text: String(capital.detail!.turns_to_complete), bad: false, paused: false, icon: 'building-granary' },
  ]);
  expect(bannerMeters(second)[0]).toMatchObject({ kind: 'growth', bad: true, text: String(second.detail!.turns_to_starve) });
  expect(bannerMeters(second).find((m) => m.kind === 'production')).toMatchObject({ icon: 'unit-scout' });
  const rival = view.cities.find((c) => c.owner !== view.me);
  if (rival) expect(bannerMeters(rival)).toEqual([]);
});

it('a city told what to build this turn shows that item on its banner at once, not the red Build prompt', () => {
  const { capital } = fixtureView();
  const idle = { ...capital, detail: { ...capital.detail!, producing: null, production_queue: [] } };
  expect(bannerMeters(idle).find((m) => m.kind === 'production')).toMatchObject({ text: 'Build', bad: true });
  const option = idle.detail.options[0];
  const chosen = bannerMeters(idle, undefined, null, option.item).find((m) => m.kind === 'production')!;
  expect(chosen.text).toBe(String(option.turns));
  expect(chosen.bad).toBeUndefined();
  expect(chosen.icon).toBeTruthy();
  // An option with no turn count (Egypt's Grow, or no output) shows a dash, never "-1".
  const noForecast = { ...idle, detail: { ...idle.detail, options: [{ ...option, turns: -1 }] } };
  expect(bannerMeters(noForecast, undefined, null, option.item).find((m) => m.kind === 'production')!.text).toBe('–');
  // Re-choosing what the city already builds keeps the live meter.
  expect(bannerMeters(capital, undefined, null, capital.detail!.producing)).toEqual(bannerMeters(capital));
});

it('a city frozen by the occupation lock shows Held on its banner, not the red Build prompt', () => {
  const { capital } = fixtureView();
  const idle = { ...capital, occupation_lock: 1, detail: { ...capital.detail!, producing: null, production_queue: [] } };
  expect(bannerMeters(idle).find((m) => m.kind === 'production')).toMatchObject({ text: 'Held', paused: true, icon: 'fortify' });
  expect(bannerMeters({ ...idle, occupation_lock: null }).find((m) => m.kind === 'production')).toMatchObject({ text: 'Build', bad: true });
});

/** A minimal own city with detail, for production-meter states the fixture doesn't cover. */
function cityWithDetail(detail: Partial<CityDetail>): { view: PlayerView; city: CityView } {
  const d: CityDetail = {
    yields: { total: { food: 0, production: 0, gold: 0, science: 0 }, sources: [], focus: 'Balanced', focus_reason: '', worked: [] },
    food_stored: 10, growth_threshold: 20, turns_to_grow: 5,
    producing: null, production_queue: [], production_stored: 0, production_cost: null, turns_to_complete: null,
    production_stall: null, idle_production_cap: 30, idle_production_waste: 0,
    buy_cost: null, buildings: [], options: [], territory: [], next_claim: null, resistance: 0,
    ...detail,
  } as CityDetail;
  const city = { id: 1, owner: 0, name: 'Test', pos: { q: 0, r: 0 }, population: 1, hp: 100, max_hp: 100, strength: 10, is_capital: true, walls: false, visible: true, detail: d } as CityView;
  const view = { me: 0, width: 1, height: 1, tiles: [], players: [{ id: 0, faction: 'India' }], cities: [city] } as unknown as PlayerView;
  return { view, city };
}

it('production meter shows a distinct "choose" state for an idle city', () => {
  const { city } = cityWithDetail({ producing: null, production_queue: [] });
  expect(productionProgress(city)).toMatchObject({ item: null, state: 'idle' });
  expect(bannerMeters(city).find((m) => m.kind === 'production')).toEqual({ kind: 'production', fraction: 0, text: 'Build', bad: true, icon: null });
});

it('an automated idle city shows its focus on the banner, not the red "Build" prompt', () => {
  const { city } = cityWithDetail({ producing: null, production_queue: [] });
  expect(bannerMeters(city, undefined, 'Science').find((m) => m.kind === 'production'))
    .toEqual({ kind: 'production', fraction: 0, text: 'Auto', paused: true, icon: 'yield-science', inkIcon: true });
});

it('an automated city that is building carries the Auto mark inside its meter strip', () => {
  const { city } = cityWithDetail({ producing: { kind: 'Unit', unit: 'Scout' }, production_cost: 60, production_stored: 10, turns_to_complete: 3 });
  expect(bannerMeters(city, undefined, 'Gold').find((m) => m.kind === 'production')).toMatchObject({ icon: 'unit-scout', auto: { label: 'Auto', icon: 'yield-gold' } });
  expect(bannerMeters(city).find((m) => m.kind === 'production')!.auto).toBeUndefined();
});

it('production meter shows a paused, grey state while waiting on a queued item', () => {
  const settler = { kind: 'Unit' as const, unit: 'Settler' as const };
  const option: ProductionOption = {
    item: settler, cost: 45, turns: 9, production_stored: 5, production_banked: 0, buy_cost: null,
    yields: { total: { food: 0, production: 0, gold: 0, science: 0 }, sources: [], focus: 'Balanced', focus_reason: '', worked: [] },
  };
  const { city } = cityWithDetail({ producing: null, production_queue: [settler], production_stored: 5, options: [option] });
  const progress = productionProgress(city)!;
  expect(progress).toMatchObject({ item: settler, state: 'waiting', stored: 5, needed: 45, turns: null });
  expect(bannerMeters(city).find((m) => m.kind === 'production')).toMatchObject({ kind: 'production', bad: false, paused: true, text: '–', icon: 'unit-settler' });
});

it('production meter flags a stall in red, with the stall reason for the tooltip', () => {
  const settler = { kind: 'Unit' as const, unit: 'Settler' as const };
  const { city } = cityWithDetail({
    producing: settler, production_stored: 10, production_cost: 45, turns_to_complete: null,
    production_stall: { kind: 'Population', required: 2, available: 1 },
  });
  const progress = productionProgress(city)!;
  expect(progress).toMatchObject({ item: settler, state: 'stalled', turns: null, stallText: 'Build stalled: Settler needs population 2 (1 after growth next turn). Its progress stays banked; grow the city or choose another build.' });
  expect(bannerMeters(city).find((m) => m.kind === 'production')).toMatchObject({ kind: 'production', bad: true, paused: false, text: '–', icon: 'unit-settler' });
});
