// @vitest-environment happy-dom
import { act, createElement } from 'react';
import { createRoot } from 'react-dom/client';
import { afterEach, expect, it, vi } from 'vitest';
import type { CityDetail, CityView, PlayerView } from '../types';
import { TileCard } from './TileCard';
import { tileIndex } from '../hex';
import { blockerText } from '../map/settleAdvice';

afterEach(() => { document.body.innerHTML = ''; vi.unstubAllGlobals(); });
const view = (faction: string) => ({
  me: 0, width: 4, height: 4,
  tiles: Array.from({ length: 16 }, () => ({ explored: true, visible: true, terrain: 'Hills', ruin: false, owner: null })),
  units: [], cities: [], players: [{ id: 0, team: 0, faction, name: 'You' }],
}) as unknown as PlayerView;

async function card(faction: string) {
  vi.stubGlobal('IS_REACT_ACT_ENVIRONMENT', true);
  const host = document.createElement('div');
  document.body.append(host);
  const root = createRoot(host);
  await act(async () => root.render(createElement(TileCard, { view: view(faction), hex: { q: 1, r: 1 } })));
  const text = host.querySelector('.tilecard')!.textContent!;
  await act(async () => root.unmount());
  return text;
}

it('shows base terrain without an obsolete Celts hill modifier', async () => {
  const celts = await card('Celts');
  expect(celts).toContain('Hills');
  expect(celts).toContain('defence');
  expect(celts).not.toContain('Celts terrain rule');
  expect(celts).not.toContain('No yield');
  const other = await card('Switzerland');
  expect(other).not.toContain('terrain rule');
});

it('explains a striped claim and a saved tile awaiting an own-city handoff', async () => {
  const v = view('Sparta');
  v.width = 12;
  v.height = 12;
  v.tiles = Array.from({ length: 144 }, () => ({ explored: true, visible: true, terrain: 'Hills', ruin: false, owner: null })) as PlayerView['tiles'];
  const hex = { q: 7, r: 5 };
  v.tiles[tileIndex(hex, v.width, v.height)].owner = 0;
  const source = { id: 11, owner: 0, name: 'Shizuoka', pos: { q: 3, r: 3 }, detail: { territory: [hex] } } as unknown as CityView;
  const recipient = { id: 27, owner: 0, name: 'Sparta Colony 11', pos: { q: 8, r: 5 }, detail: { territory: [] } } as unknown as CityView;
  v.cities = [source, recipient];
  vi.stubGlobal('IS_REACT_ACT_ENVIRONMENT', true);
  const host = document.createElement('div'); document.body.append(host);
  const root = createRoot(host);
  await act(async () => root.render(createElement(TileCard, { view: v, hex })));
  expect(host.querySelector('[data-testid="tile-work-status"]')?.textContent).toContain('Sparta Colony 11 will take it over and work it after the next turn');
  v.cities = [source];
  await act(async () => root.render(createElement(TileCard, { view: v, hex })));
  expect(host.querySelector('[data-testid="tile-work-status"]')?.textContent).toContain('Striped: Shizuoka owns this tile beyond its 3-tile work radius. It gives no yield.');
  await act(async () => root.unmount());
});

it('explains a hidden-city spacing blocker on the tile card without naming its city or owner', async () => {
  const v = view('India');
  const hex = { q: 1, r: 1 };
  v.city_gap_blocked = [hex];
  const note = blockerText(v, 'CityTooClose');
  expect(note).toBe('Too close to another city to found one here');
  vi.stubGlobal('IS_REACT_ACT_ENVIRONMENT', true);
  const host = document.createElement('div'); document.body.append(host);
  const root = createRoot(host);
  await act(async () => root.render(createElement(TileCard, { view: v, hex, note })));
  expect(host.querySelector('[data-testid="tile-note"]')?.textContent).toBe(note);
  expect(host.textContent).not.toMatch(/Rival|City \d/);
  await act(async () => root.unmount());
});

it('names every unit owner in the tile card and shows the matching faction emblem', async () => {
  const v = view('Mongols');
  v.players.push({ id: 1, team: 1, faction: 'Byzantium', name: 'Rival' } as never);
  v.units = [
    { id: 1, owner: 0, kind: 'Scout', pos: { q: 1, r: 1 }, military: true, hp: 100, strength: 5, fortify: 0 } as never,
    { id: 2, owner: 1, kind: 'Scout', pos: { q: 1, r: 1 }, military: true, hp: 100, strength: 5, fortify: 0 } as never,
    { id: 3, owner: 255, kind: 'Scout', pos: { q: 1, r: 1 }, military: true, hp: 100, strength: 5, fortify: 0 } as never,
  ];
  const host = document.createElement('div'); document.body.append(host);
  vi.stubGlobal('IS_REACT_ACT_ENVIRONMENT', true);
  const root = createRoot(host);
  await act(async () => root.render(createElement(TileCard, { view: v, hex: { q: 1, r: 1 } })));
  const rows = [...host.querySelectorAll('.tilecard-unit')];
  expect(rows.map((row) => row.textContent)).toEqual([
    expect.stringContaining('Your Scout · Ancient era · 100 HP · strength 5'),
    expect.stringContaining('Byzantine Scout · Ancient era · 100 HP · strength 5'),
    expect.stringContaining('Barbarian Scout · Ancient era · 100 HP · strength 5'),
  ]);
  expect(rows[0].querySelector('.faction-mark')?.getAttribute('aria-label')).toBe('Mongols');
  expect(rows[1].querySelector('.faction-mark')?.getAttribute('aria-label')).toBe('Byzantium');
  expect(rows[2].querySelector('.faction-mark')).toBeNull();
  await act(async () => root.unmount());
});

async function wonderCard(patch: (v: PlayerView) => void, hex = { q: 1, r: 1 }) {
  const v = view('India');
  v.tiles[tileIndex({ q: 1, r: 1 }, v.width, v.height)].wonder = 'GrandCanyon';
  patch(v);
  vi.stubGlobal('IS_REACT_ACT_ENVIRONMENT', true);
  const host = document.createElement('div'); document.body.append(host);
  const root = createRoot(host);
  await act(async () => root.render(createElement(TileCard, { view: v, hex })));
  const out = { text: host.textContent!, host, root };
  return out;
}

it('shows a painted natural wonder, its bonus in plain words and the open discovery reward', async () => {
  const { text, host, root } = await wonderCard(() => {});
  expect(text).toContain('Grand Canyon');
  expect(text).toContain('The city that owns this tile gets +25% Science');
  expect(text).toContain("Unclaimed: grow a city's borders over it to own it.");
  expect(text).toContain('Discovery reward: 50 Science, paid once to the first empire to see it.');
  expect(host.querySelector('.tilecard-lead img')?.getAttribute('src')).toContain('grand-canyon-a-512.png');
  // The wonder leads: the lead block comes before the terrain line.
  expect(host.querySelector('.tilecard')!.firstElementChild!.classList.contains('tilecard-wonder-lead')).toBe(true);
  await act(async () => root.unmount());
});

it('says who holds the wonder, and what it adds to your own city', async () => {
  const mine = await wonderCard((v) => {
    v.tiles[tileIndex({ q: 1, r: 1 }, v.width, v.height)].owner = 0;
    v.tiles[tileIndex({ q: 1, r: 1 }, v.width, v.height)].wonder_found = true;
    v.cities = [{ id: 1, owner: 0, name: 'Kyoto', pos: { q: 2, r: 2 }, visible: true, detail: { territory: [{ q: 1, r: 1 }], yields: { total: { food: 0, production: 0, gold: 0, science: 15 }, sources: [{ label: 'Grand Canyon (+25%)', yields: { food: 0, production: 0, gold: 0, science: 3 } }] } } } as never];
  });
  expect(mine.text).toContain('Held by your city Kyoto: +3 Science each turn.');
  expect(mine.text).toContain('Discovery reward: already paid');
  await act(async () => mine.root.unmount());
  const theirs = await wonderCard((v) => {
    v.tiles[tileIndex({ q: 1, r: 1 }, v.width, v.height)].owner = 1;
    v.players.push({ id: 1, team: 1, faction: 'Byzantium', name: 'Rival' } as never);
  });
  expect(theirs.text).toMatch(/Held by .+\./);
  expect(theirs.text).not.toContain('Held by your city');
  await act(async () => theirs.root.unmount());
});

it('a resource leads its tile with a big medallion, short benefit lines, then the status, then the terrain', async () => {
  const v = view('India');
  const i = tileIndex({ q: 1, r: 1 }, v.width, v.height);
  (v.tiles[i] as { terrain: string }).terrain = 'Forest';
  (v.tiles[i] as { resource?: string }).resource = 'Iron';
  (v as unknown as { research: { known: string[] } }).research = { known: [] };
  vi.stubGlobal('IS_REACT_ACT_ENVIRONMENT', true);
  const host = document.createElement('div'); document.body.append(host);
  const root = createRoot(host);
  await act(async () => root.render(createElement(TileCard, { view: v, hex: { q: 1, r: 1 } })));
  const card = host.querySelector('.tilecard')!;
  expect(card.firstElementChild!.getAttribute('data-testid')).toBe('tile-resource');
  expect(card.querySelector('.tilecard-lead img')?.getAttribute('width')).toBe('46');
  expect(card.querySelector('.tilecard-lead-name')!.textContent).toBe('Iron');
  expect(card.querySelector('.tilecard-lead-kind')!.textContent).toBe('Strategic resource');
  const benefits = [...card.querySelectorAll('.tilecard-benefits li')].map((li) => li.textContent);
  expect(benefits).toEqual(['+2 Production with a Mine', 'Supplies 2 Iron inside your borders, 3 with a Mine']);
  expect(card.querySelector('[data-testid="tile-improvement"]')!.textContent).toBe('Unclaimed: bring it inside your borders');
  // Order: lead, benefits, status, then the terrain name.
  const order = [...card.children].map((c) => c.className.split(' ')[0]);
  expect(order.indexOf('tilecard-lead')).toBeLessThan(order.indexOf('resource-line'));
  expect(order.indexOf('resource-line')).toBeLessThan(order.indexOf('tilecard-terrain'));
  expect(card.querySelector('.tilecard-terrain')!.textContent).toBe('Forest');
  await act(async () => root.unmount());
});

/** A minimal own or foreign city at (1,1), for the tile card's production line. */
function cardWithCity(owner: number, detail: CityDetail | null) {
  const city = {
    id: 1, owner, name: 'Outpost', pos: { q: 1, r: 1 }, population: 1, hp: 100, max_hp: 100, strength: 10,
    is_capital: true, walls: false, visible: true, detail,
  } as CityView;
  const view = {
    me: 0, width: 4, height: 4,
    tiles: Array.from({ length: 16 }, () => ({ explored: true, visible: true, terrain: 'Hills', ruin: false, owner: null })),
    units: [], cities: [city], players: [{ id: 0, team: 0, faction: 'India', name: 'You' }, { id: 1, team: 1, faction: 'Byzantium', name: 'Rival' }],
  } as unknown as PlayerView;
  return { view, city };
}

async function render(view: PlayerView, hex = { q: 1, r: 1 }) {
  vi.stubGlobal('IS_REACT_ACT_ENVIRONMENT', true);
  const host = document.createElement('div');
  document.body.append(host);
  const root = createRoot(host);
  await act(async () => root.render(createElement(TileCard, { view, hex })));
  const text = host.querySelector('.tilecard')!.textContent!;
  await act(async () => root.unmount());
  return text;
}

const baseDetail: CityDetail = {
  yields: { total: { food: 0, production: 0, gold: 0, science: 0 }, sources: [], focus: 'Balanced', focus_reason: '', worked: [] },
  food_stored: 0, growth_threshold: 20, turns_to_grow: 5,
  producing: { kind: 'Unit', unit: 'Settler' }, production_queue: [], production_stored: 12, production_cost: 45, turns_to_complete: 4,
  production_stall: null, idle_production_cap: 30, idle_production_waste: 0,
  buy_cost: null, buildings: [], options: [], unavailable: [], production_banked: [], territory: [], next_claim: null,
  tile_base_price: 40, buyable_tiles: [],
  resistance: 0,
};

it('explains that a claimed tile beyond the city work radius blocks rivals but yields nothing', async () => {
  const v = view('India');
  v.width = 6;
  v.height = 6;
  v.tiles = Array.from({ length: 36 }, () => ({ explored: true, visible: true, terrain: 'Hills', ruin: false, owner: null })) as never;
  const hex = { q: 0, r: 0 };
  v.tiles[tileIndex(hex, v.width, v.height)].owner = 0;
  v.cities = [{ id: 1, owner: 0, name: 'Outpost', pos: { q: 3, r: 5 }, detail: { territory: [hex] } } as never];
  const text = await render(v, hex);
  expect(text).toContain('Striped: Outpost owns this tile beyond its 3-tile work radius. It gives no yield. It blocks rivals.');
});

it('names the item, stored Production and turns left for your own city', async () => {
  const { view } = cardWithCity(0, baseDetail);
  const text = await render(view);
  expect(text).toContain('Producing Settler: 12/45, 4 turns');
});

it('never shows production for a city you do not own (pillar 6: no hidden info)', async () => {
  // A foreign city's detail is null (the core only sends full detail for your own cities), but
  // the row must stay hidden even if a detail happened to be present.
  const { view } = cardWithCity(1, null);
  const text = await render(view);
  expect(text).not.toContain('Producing');
  expect(text).not.toContain('No production chosen');
  const { view: leaked } = cardWithCity(1, baseDetail);
  expect(await render(leaked)).not.toContain('Producing');
});

it('shows the idle "choose" state and a stall reason', async () => {
  const { view: idle } = cardWithCity(0, { ...baseDetail, producing: null, production_queue: [] });
  expect(await render(idle)).toContain('No production chosen');
  const { view: stalled } = cardWithCity(0, { ...baseDetail, production_stall: { kind: 'SpawnBlocked' } });
  expect(await render(stalled)).toContain('Build stalled: no free spawn tile in or beside the city. Move a blocking unit to make room.');
});

it('a dormant strategic resource says its fact once and names the city that builds it', async () => {
  const { view } = cardWithCity(0, { ...baseDetail, territory: [{ q: 2, r: 1 }] });
  const city = view.cities[0];
  city.pos = { q: 3, r: 3 }; city.name = 'Astra';
  const i = 1 * 4 + 2;
  (view.tiles[i] as { owner: number | null; resource?: string }).owner = 0;
  (view.tiles[i] as { resource?: string }).resource = 'Iron';
  (view as unknown as { research: { known: string[] } }).research = { known: ['IronWorking'] };
  city.detail = { ...baseDetail, territory: [{ q: 2, r: 1 }] };
  vi.stubGlobal('IS_REACT_ACT_ENVIRONMENT', true);
  const host = document.createElement('div');
  document.body.append(host);
  const root = createRoot(host);
  await act(async () => root.render(createElement(TileCard, { view, hex: { q: 2, r: 1 } })));
  const text = host.querySelector('.tilecard')!.textContent!;
  await act(async () => root.unmount());
  expect(text).toContain('+2 Production with a Mine');
  expect(text).toContain("Its Production bonus is dormant: build the Mine from Astra's production list");
  expect(text).not.toContain('already counts');
  expect(text).not.toContain('In use');
  expect(text.match(/Supplies 2 Iron/g)).toHaveLength(1);
});

it('a strategic deposit in another empire’s land names its holder, and says when you lack the resource', async () => {
  const v = view('India');
  v.players.push({ id: 1, team: 1, faction: 'Mongols', name: 'Rival' } as never);
  const i = tileIndex({ q: 1, r: 1 }, v.width, v.height);
  Object.assign(v.tiles[i], { terrain: 'Plains', resource: 'Niter', owner: 1 });
  (v as unknown as { research: { known: string[] } }).research = { known: [] };
  (v as unknown as { supply: unknown[] }).supply = [{ resource: 'Niter', revealed: true, supplied: { total: 0, parts: [] }, used: { total: 0, parts: [] }, spare: 0 }];
  vi.stubGlobal('IS_REACT_ACT_ENVIRONMENT', true);
  const host = document.createElement('div'); document.body.append(host);
  const root = createRoot(host);
  await act(async () => root.render(createElement(TileCard, { view: v, hex: { q: 1, r: 1 } })));
  const status = host.querySelector('[data-testid="tile-improvement"]')!.textContent;
  expect(status).toBe('The Mongols hold this deposit. You have no Niter yet: take it or find another.');
  await act(async () => root.unmount());
});

it('reads the supply a deposit gives the viewer, by the viewer\'s faction, not the tile owner\'s', async () => {
  const run = async (viewer: string, owner: string) => {
    const v = view(viewer);
    v.players = [{ id: 0, team: 0, faction: viewer, name: 'You' }, { id: 1, team: 1, faction: owner, name: 'Them' }] as unknown as PlayerView['players'];
    v.research = { known: ['IronWorking'] } as unknown as PlayerView['research'];
    const hex = { q: 1, r: 1 }, i = tileIndex(hex, v.width, v.height);
    v.tiles[i] = { ...v.tiles[i], resource: 'Iron', owner: 1 } as PlayerView['tiles'][number];
    vi.stubGlobal('IS_REACT_ACT_ENVIRONMENT', true);
    const host = document.createElement('div'); document.body.append(host);
    const root = createRoot(host);
    await act(async () => root.render(createElement(TileCard, { view: v, hex })));
    const text = host.querySelector('.tilecard')!.textContent!;
    await act(async () => root.unmount());
    return text;
  };
  // A rival looking at a German deposit: the supply is the viewer's (2, 3 with a Mine), not Germany's doubled 4 and 6.
  const rival = await run('Mongols', 'Germany');
  expect(rival).toContain('Supplies 2 Iron inside your borders, 3 with a Mine');
  expect(rival).not.toContain('Supplies 4 Iron');
  // Germany looking at a rival's deposit: its own doubled numbers.
  const german = await run('Germany', 'Mongols');
  expect(german).toContain('Supplies 4 Iron inside your borders, 6 with a Mine');
});
