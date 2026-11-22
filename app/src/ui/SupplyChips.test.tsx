// @vitest-environment happy-dom
// The top-bar strategic supply chip and tooltip (pillar 5: no hidden math). "Iron 5 / 6" is in use over
// supplied; a shortage is never colour alone and the tooltip names every deposit, user, held build and idle user.
import { renderToStaticMarkup } from 'react-dom/server';
import { expect, it } from 'vitest';
import type { SupplyView } from '../types';
import { SupplyIcon, SupplyTip, supplyFraction, supplySummary } from './SupplyChips';

const iron = (over: Partial<SupplyView> = {}): SupplyView => ({
  resource: 'Iron', revealed: true, revealed_by: 'IronWorking',
  supplied: { total: 6, parts: [['Iron near Lugano, with a Mine', 3], ['Iron near Bern', 3]] },
  used: { total: 5, parts: [['Forge in Lugano', 1], ['Pikeman ×4', 4]] },
  idle: [], held: [], spare: 1, ...over,
});

it('reads spare over supplied and says what is spare', () => {
  expect(supplyFraction(iron())).toBe('1/6');
  expect(supplySummary(iron())).toBe('Iron: 5 of 6 at work, 1 to spare');
});

it('says how many buildings or units are idle when short, and never calls demand "in use" or a negative spare', () => {
  const short = iron({ supplied: { total: 6, parts: [['Iron near Bern', 6]] }, used: { total: 8, parts: [['Pikeman ×8', 8]] }, idle: ['Pikeman', 'Pikeman'], spare: -2 });
  expect(supplySummary(short)).toBe('Iron: needs 8, supplies 6, so 2 buildings or units are idle');
  expect(supplySummary({ ...short, idle: ['Pikeman'] })).toContain('1 building or unit is idle');
  const html = renderToStaticMarkup(<SupplyTip s={short} />);
  expect(html).toContain('Needs 8, supplies 6');
  expect(html).toContain('Short by 2');
  expect(html).toContain('Spare');
  expect(html).not.toMatch(/in use|−2/);
});

it('the tooltip header reads "2 of 2 Iron at work, 2 idle"', () => {
  const short = iron({ supplied: { total: 2, parts: [['Iron near Bern', 2]] }, used: { total: 4, parts: [['Pikeman ×4', 4]] }, idle: ['Pikeman', 'Pikeman'], spare: -2 });
  const title = document.createElement('div');
  title.innerHTML = renderToStaticMarkup(<SupplyTip s={short} />);
  expect(title.querySelector('.bd-title')!.textContent).toBe('2 of 2 Iron at work, 2 idle');
});

it('with no deposit says to bring one inside the borders', () => {
  const none = iron({ supplied: { total: 0, parts: [] }, used: { total: 0, parts: [] }, spare: 0 });
  expect(supplySummary(none)).toBe('Iron: no deposit inside your borders yet');
  const tip = document.createElement('div');
  tip.innerHTML = renderToStaticMarkup(<SupplyTip s={none} />);
  expect(tip.textContent).toContain('No Iron deposit inside your borders yet. Bring an Iron deposit inside your borders, or build a Mine on one for +1 Iron.');
});

it('an unrevealed resource says what reveals it', () => {
  expect(supplySummary(iron({ revealed: false }))).toBe('Iron: revealed by Iron Working');
});

it('the tooltip lists deposits, what needs them, held builds and idle buildings or units', () => {
  const html = renderToStaticMarkup(<SupplyTip s={iron({
    used: { total: 7, parts: [['Forge in Lugano', 1], ['Pikeman ×6', 6]] }, idle: ['Pikeman', 'Forge in Bern'], held: ['Forge in Bern'], spare: -2,
  })} />);
  for (const text of ['Iron near Lugano, with a Mine', 'Iron near Bern', 'Forge in Lugano', 'Pikeman ×6', '1 held for the Forge in Bern', 'Idle, newest first:', 'Short of Iron: 2 buildings or units are idle', '25% weaker', 'Short by 2', 'Spare']) {
    expect(html).toContain(text);
  }
  expect(html.replace(/<[^>]*>/g, '')).not.toMatch(/stockpile|stored|a turn|\busers?\b/i);
});

it('states the resource users in the current era', () => {
  const classical = renderToStaticMarkup(<SupplyTip s={iron()} era={1} />);
  const medieval = renderToStaticMarkup(<SupplyTip s={iron()} era={2} />);
  expect(classical).toContain('Used for this era: Phalangites, Composite Archers, Cataphracts, Forges and Siege');
  expect(medieval).toContain('Used for this era: Pikemen, Crossbowmen, Knights, Universities, Forges and Siege');
});

it('the top bar chip shows painted art from 24 px up, so Iron and Coal read apart without their names', () => {
  const iron = renderToStaticMarkup(<SupplyIcon resource="Iron" size={24} />);
  const coal = renderToStaticMarkup(<SupplyIcon resource="Coal" size={24} />);
  for (const [markup, name] of [[iron, 'Iron'], [coal, 'Coal']]) {
    expect(markup).toContain('supply-art');
    expect(markup).toContain('width="24"');
    expect(markup).toContain(`alt="${name}"`);
  }
  expect(iron).not.toBe(coal.replace('Coal', 'Iron'));
});

it('uses painted medallions in small chips too, including the new late resources', () => {
  for (const resource of ['Iron', 'Niter', 'Oil', 'Silicon']) {
    const chip = renderToStaticMarkup(<SupplyIcon resource={resource} size={16} />);
    expect(chip).toContain('supply-art');
    expect(chip).toContain(`alt="${resource}"`);
  }
  expect(renderToStaticMarkup(<SupplyTip s={iron()} />)).toContain('amt-iron');
});

it('each idle entry with a place offers Show on map; one without a place does not', () => {
  const html = renderToStaticMarkup(<SupplyTip s={iron({ used: { total: 7, parts: [['Pikeman ×7', 7]] }, idle: ['Pikeman', 'Forge in Bern'], spare: -1 })}
    idleEntries={[{ label: 'Pikeman', hex: null }, { label: 'Forge in Bern', hex: { q: 1, r: 2 } }]} />);
  expect(html.match(/Show on map/g)).toHaveLength(1);
  expect(html).toContain('Forge in Bern</span><button');
});

// ---- Each era is defined by its resource: the era chip, the missing alert, who holds the deposits ----
import type { PlayerView, Resource } from '../types';
import { SupplyChips, chipSupplies, holdersText, isMissing, missingText, supplyWanted } from './SupplyChips';
import { depositHolders, eraResource, idleEntries, lackedResources, listEmpires, needText, unitNeed } from '../resources';
import { hexAt } from '../hex';
import { placeName } from '../empires';

const supplyOf = (resource: Resource, over: Partial<SupplyView> = {}): SupplyView => ({
  resource, revealed: true, revealed_by: null, supplied: { total: 0, parts: [] }, used: { total: 0, parts: [] }, idle: [], held: [], spare: 0, ...over,
});

/** A view of seat 0 in `era` (0-based), with a city, these supplies, and tiles that hold deposits for the holder lines. */
function eraView(era: number, supply: SupplyView[], tiles: { resource: Resource; owner: number | null }[] = [], units: Partial<PlayerView['units'][number]>[] = [], cities = [{ id: 1, owner: 0, name: 'Astra', pos: { q: 0, r: 0 } }]): PlayerView {
  return {
    me: 0, width: 8, height: 8, research: { era, known: [] },
    players: [
      { id: 0, name: 'Ada', faction: 'India', team: 0, era: era + 1, alive: true },
      { id: 1, name: 'AI 2', faction: 'Byzantium', team: 1, era: 3, alive: true },
      { id: 2, name: 'AI 3', faction: 'Celts', team: 2, era: 3, alive: true },
    ],
    supply, units, cities,
    tiles: tiles.map((t) => ({ explored: true, visible: true, terrain: 'Plains', resource: t.resource, owner: t.owner })),
  } as unknown as PlayerView;
}

it('the era table gives each era its resource', () => {
  expect(([0, 1, 2, 3, 4, 5, 6, 7, 8] as const).map(eraResource)).toEqual([null, 'Iron', 'Iron', 'Niter', 'Coal', 'Oil', 'Oil', 'Oil', 'Silicon']);
});

it('there is no supply chip before the first city', () => {
  expect(chipSupplies(eraView(0, [supplyOf('Iron')], [], [], []))).toEqual([]);
  expect(chipSupplies(eraView(0, [supplyOf('Iron')])).map((s) => s.resource)).toEqual(['Iron']);
});

it('the era comes from the research view, 0-based, not the public player era', () => {
  const view = eraView(2, [supplyOf('Iron'), supplyOf('Niter')]);
  view.players[0].era = 4; // a lagging or ahead public value must not matter
  expect(chipSupplies(view).map((s) => s.resource)).toEqual(['Iron', 'Niter']);
});

it('the top bar shows revealed resources and the current era resource as a research target', () => {
  const all = ['Iron', 'Niter', 'Coal', 'Oil', 'Silicon'].map((r) => supplyOf(r as Resource, { revealed: r === 'Iron' || r === 'Niter', supplied: { total: 2, parts: [[`${r} near Bern`, 2]] }, used: { total: 1, parts: [] }, spare: 1 }));
  expect(chipSupplies(eraView(3, all)).map((s) => s.resource)).toEqual(['Iron', 'Niter']);
  // A short Iron keeps its chip; only the current era's hidden resource joins revealed chips.
  const short = all.map((s) => (s.resource === 'Iron' ? { ...s, idle: ['Pikeman'], used: { total: 3, parts: [] }, spare: -1 } : s));
  expect(chipSupplies(eraView(3, short)).map((s) => s.resource)).toEqual(['Iron', 'Niter']);
  expect(chipSupplies(eraView(4, short)).map((s) => s.resource)).toEqual(['Iron', 'Niter', 'Coal']);
  const hiddenCoal = short.map((s) => s.resource === 'Niter' ? { ...s, revealed: false } : s);
  expect(chipSupplies(eraView(4, hiddenCoal)).map((s) => s.resource)).toEqual(['Iron', 'Coal']);
  const html = renderToStaticMarkup(<SupplyChips view={eraView(4, hiddenCoal)} />);
  const root = document.createElement('div');
  root.innerHTML = html;
  expect(root.querySelector('.supply-coal')?.textContent).toBe('Coal: revealed by Industrialization');
  expect(renderToStaticMarkup(<SupplyTip s={hiddenCoal.find((s) => s.resource === 'Coal')!} era={4} />)).toContain('Riflemen, Field Cannons, Cavalry and Factories');
  expect(html).toContain('supply-unrevealed');
  expect(html).not.toContain('supply-coal supply-short');
});

it('an Industrial empire gives Iron, Niter, Coal and Oil separate chips with spare over total', () => {
  const supplies = [
    supplyOf('Iron', { supplied: { total: 9, parts: [] }, used: { total: 8, parts: [] }, held: ['University in Pylos'], spare: 0 }),
    supplyOf('Niter', { supplied: { total: 3, parts: [] }, used: { total: 1, parts: [] }, spare: 2 }),
    supplyOf('Coal', { supplied: { total: 4, parts: [] }, used: { total: 5, parts: [] }, idle: ['Industrial Infantry'], spare: -1 }),
    supplyOf('Oil', { supplied: { total: 2, parts: [] }, used: { total: 1, parts: [] }, spare: 1 }),
    supplyOf('Silicon', { revealed: false }),
  ];
  const view = eraView(4, supplies);
  expect(chipSupplies(view).map((s) => s.resource)).toEqual(['Iron', 'Niter', 'Coal', 'Oil']);
  expect(supplies.map((s) => supplyWanted(view, s))).toEqual([true, true, true, true, false]);
  const root = document.createElement('div');
  root.innerHTML = renderToStaticMarkup(<SupplyChips view={view} />);
  expect(root.querySelectorAll('.supply-stat')).toHaveLength(4);
  expect(root.querySelector('.supply-iron')?.classList.contains('supply-short')).toBe(false);
  expect(root.querySelector('.supply-coal')?.classList.contains('supply-era')).toBe(true);
  expect(root.querySelector('.supply-coal')?.classList.contains('supply-short')).toBe(true);
  expect(root.querySelector('[data-testid="supply-Iron"]')?.textContent).toBe('0/9');
  expect(root.querySelector('[data-testid="supply-Niter"]')?.textContent).toBe('2/3');
  expect(root.querySelector('[data-testid="supply-Coal"]')?.textContent).toBe('0/4');
  expect(root.querySelector('[data-testid="supply-Oil"]')?.textContent).toBe('1/2');
  expect(root.textContent).not.toContain('Your other supplies');
});

it('the era resource is missing when none is supplied and something waits on it', () => {
  const view = eraView(3, [], [], [{ id: 1, owner: 0, kind: 'Pikeman', upgrade_need: 'Niter', upgrade_supply_blocker: 'No Niter to spare' }]);
  view.research.known.push('Gunpowder');
  expect(isMissing(view, supplyOf('Niter'))).toBe(true);
  expect(isMissing(eraView(3, []), supplyOf('Niter'))).toBe(false);
  expect(isMissing(eraView(3, []), supplyOf('Niter', { held: ['Bank in Bern'] }))).toBe(true);
  expect(isMissing(eraView(3, []), supplyOf('Niter', { used: { total: 1, parts: [] }, idle: ['Bank in Bern'] }))).toBe(true);
  expect(isMissing(eraView(3, []), supplyOf('Niter', { supplied: { total: 2, parts: [] }, held: ['Bank in Bern'] }))).toBe(false);
  expect(isMissing(eraView(3, []), supplyOf('Niter', { revealed: false, held: ['Bank in Bern'] }))).toBe(false);
});

it('keeps a revealed 0/0 supply neutral when only future upgrades could use it', () => {
  const view = eraView(4, [supplyOf('Niter'), supplyOf('Oil')], [], [
    { id: 1, owner: 0, kind: 'Pikeman', upgrade_need: 'Niter', upgrade_supply_blocker: 'No Niter to spare' },
    { id: 2, owner: 0, kind: 'Cavalry', upgrade_need: 'Oil', upgrade_supply_blocker: 'No Oil to spare' },
  ]);
  const container = document.createElement('div');
  container.innerHTML = renderToStaticMarkup(<SupplyChips view={view} />);
  for (const resource of ['niter', 'oil']) {
    const chip = container.querySelector(`.supply-${resource}`);
    expect(chip).not.toBeNull();
    expect(chip?.classList.contains('supply-short')).toBe(false);
    expect(chip?.querySelector('.supply-alert')).toBeNull();
  }
  view.research.known.push('Gunpowder', 'Combustion');
  expect(isMissing(view, supplyOf('Niter'))).toBe(true);
  expect(isMissing(view, supplyOf('Oil'))).toBe(true);
  container.innerHTML = renderToStaticMarkup(<SupplyChips view={view} />);
  expect(container.querySelector('.supply-niter')?.classList.contains('supply-short')).toBe(true);
  expect(container.querySelector('.supply-oil')?.classList.contains('supply-short')).toBe(true);
});

it('the missing alert names who needs it and who holds the deposits you have seen', () => {
  expect(missingText('Niter')).toBe('Missing Niter: your gunpowder units and Banks need it.');
  expect(missingText(eraResource(8)!)).toBe(`Missing ${eraResource(8)}: your Future units need it.`);
  const view = eraView(3, [], [{ resource: 'Niter', owner: 1 }, { resource: 'Niter', owner: 1 }, { resource: 'Niter', owner: 2 }, { resource: 'Niter', owner: null }, { resource: 'Iron', owner: 1 }, { resource: 'Niter', owner: 0 }]);
  const survey = depositHolders(view, 'Niter');
  expect(survey).toMatchObject({ empires: ['Byzantium', 'Celts'], held: 3, unclaimed: 1, nearestUnclaimed: hexAt(3, 8) });
  expect(survey.nearest).not.toBeNull();
  const place = placeName(view, hexAt(3, 8));
  expect(place).toMatch(/ of Astra$/);
  expect(holdersText(view, 'Niter')).toBe(`One unclaimed Niter deposit lies ${place}. Byzantium and the Celts hold 3 other Niter deposits.`);
  expect(holdersText(eraView(3, [], [{ resource: 'Niter', owner: 2 }]), 'Niter')).toBe('The Celts hold 1 Niter deposit you have seen.');
  expect(holdersText(eraView(3, [], [{ resource: 'Niter', owner: 1 }]), 'Niter')).toBe('Byzantium holds 1 Niter deposit you have seen.');
  expect(holdersText(eraView(3, [], [{ resource: 'Niter', owner: null }, { resource: 'Niter', owner: null }]), 'Niter')).toMatch(/^2 unclaimed Niter deposits lie; the nearest is .*Astra\.$/);
  expect(holdersText(eraView(3, [], [{ resource: 'Niter', owner: null }]), 'Niter')).toMatch(/^One unclaimed Niter deposit lies .*Astra\.$/);
  expect(holdersText(eraView(3, []), 'Niter')).toBe('You have seen no Niter deposit yet. Explore for one.');
  expect(listEmpires(['Celts', 'Sparta', 'Mongols'])).toBe('the Celts, Sparta and the Mongols');
});

it('the chip shows the alert mark and the tooltip leads with the missing line when the era resource is missing', () => {
  const view = eraView(3, [supplyOf('Niter', { used: { total: 1, parts: [['Bank in Bern', 1]] }, idle: ['Bank in Bern'], spare: -1 })], [{ resource: 'Niter', owner: 1 }]);
  const html = renderToStaticMarkup(<SupplyChips view={view} />);
  expect(html).toContain('supply-missing');
  expect(html).toContain('Missing Niter');
  expect(html).toContain('data-testid="supply-Niter"');
  expect(html).toContain('supply-era');
  const calm = renderToStaticMarkup(<SupplyChips view={eraView(3, [supplyOf('Niter', { supplied: { total: 2, parts: [['Niter near Bern', 2]] }, used: { total: 1, parts: [['Bank in Bern', 1]] }, spare: 1 })])} />);
  expect(calm).not.toContain('supply-missing');
  expect(calm).not.toContain('supply-short');
});

it('lacked resources are the revealed ones with none supplied, or short', () => {
  const lacked = lackedResources([supplyOf('Iron'), supplyOf('Niter', { revealed: false }), supplyOf('Coal', { supplied: { total: 2, parts: [] }, idle: ['Factory in Bern'] })]);
  expect([...lacked].sort()).toEqual(['Coal', 'Iron']);
});

it('the missing tooltip is titled "Missing Niter", not "0 of 0 Niter at work", and offers Show on map to the nearest deposit', () => {
  const s = supplyOf('Niter', { used: { total: 1, parts: [['Bank in Bern', 1]] }, idle: ['Bank in Bern'], spare: -1 });
  const html = renderToStaticMarkup(<SupplyTip s={s} missing={{ users: 'Your gunpowder units and Banks need it.', holders: 'Byzantium holds 1 Niter deposit you have seen.', target: { q: 3, r: 1 } }} />);
  expect(html).toContain('Missing Niter');
  expect(html).not.toMatch(/0 of 0|at work/);
  expect(html).toContain('Your gunpowder units and Banks need it.');
  expect(html).toContain('Byzantium holds 1 Niter deposit you have seen.');
  expect(html).toContain('Show on map');
  expect(renderToStaticMarkup(<SupplyTip s={s} missing={{ users: 'x', holders: 'y', target: null }} />)).not.toContain('Show on map');
});

it('deposit labels name the resource’s own works', () => {
  const oil = supplyOf('Oil', { supplied: { total: 3, parts: [['Oil near Lugano, with an Oil Well', 3]] } });
  expect(renderToStaticMarkup(<SupplyTip s={oil} />)).toContain('with an Oil Well');
  const empty = document.createElement('div');
  empty.innerHTML = renderToStaticMarkup(<SupplyTip s={supplyOf('Oil')} />);
  expect(empty.textContent).toContain('build an Oil Well on one for +1 Oil');
});

it('the Oil shortage names aircraft and routes an idle plane to its hangar city', () => {
  const oil = supplyOf('Oil', { used: { total: 1, parts: [['Bomber', 1]] }, idle: ['Bomber'], spare: -1 });
  const view = eraView(5, [oil]);
  view.aircraft = [{ id: 900, owner: 0, kind: 'Bomber', city: 1, hp: 100, unsupplied: 'Oil' }];
  expect(missingText('Oil')).toContain('aircraft');
  expect(idleEntries(view, oil)).toEqual([{ label: 'Bomber', hex: view.cities[0].pos }]);
  expect(renderToStaticMarkup(<SupplyTip s={oil} idleEntries={idleEntries(view, oil)} />)).toContain('Show on map');
});

it('the build card reads aircraft Oil from the generated core rules', () => {
  expect(unitNeed('Fighter')).toBe('Oil');
  expect(unitNeed('Bomber')).toBe('Oil');
  expect(needText(unitNeed('Bomber')!)).toBe('Uses 1 Oil');
});
