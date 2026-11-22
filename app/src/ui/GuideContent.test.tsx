// @vitest-environment happy-dom
import { renderToStaticMarkup } from 'react-dom/server';
import { createRoot } from 'react-dom/client';
import { act } from 'react';
import { expect, it } from 'vitest';
import { DECADENCE_FOOD_RULE, RULES, UNIT_NAME, UNIT_STATS, WORK_RADIUS, TILE_GOLD_BASE, TILE_GOLD_PER_TILE, decadencePercent, tileRingPercent } from '../data';
import { TREATY_DEFINITION, allianceTerms } from '../treaties';
import { eraResource } from '../resources';
import { useGame } from '../store';
import type { PlayerView } from '../types';
import { GUIDE_SECTIONS, GuideContent } from './GuideContent';

it('explains Peace, Alliance and War with Gold costs from the core snapshot', () => {
  const start = renderToStaticMarkup(<GuideContent section="start" />);
  const turns = renderToStaticMarkup(<GuideContent section="turns" />);
  const cities = renderToStaticMarkup(<GuideContent section="cities" />);
  const plainTurns = turns.replace(/<[^>]*>/g, '');
  const terms = allianceTerms();
  expect(start).toContain('Other teams start at Peace, with closed borders');
  expect(plainTurns).toContain(`Each empire pays ${terms.upkeep} Gold per owned city each turn`);
  expect(turns).toContain(`For its first ${terms.protectedTurns} turns an Alliance is protected: leaving inside that window costs ${terms.breachCost} Gold and is a betrayal`);
  expect(turns).toContain('Declaring War on an ally is always a betrayal');
  expect(turns).toContain(`A betrayed team won’t ally with the betrayer for ${terms.betrayalTurns} turns`);
  expect(turns).toContain(`starts a ${RULES.treaty.guaranteeTurns}-turn truce`);
  expect(cities).toContain('After you have researched every technology');
  expect(start + turns).not.toMatch(/Every other team is at war|no diplomacy|Pact|trust|exit notice/);
});

it('defines Peace, Alliance and War in one line each, before the detailed rules', () => {
  const turns = renderToStaticMarkup(<GuideContent section="turns" />);
  const ladder = turns.indexOf('guide-ladder');
  expect(ladder).toBeGreaterThan(turns.indexOf('Diplomacy: Peace, Alliance or War'));
  expect(ladder).toBeLessThan(turns.indexOf('Peace keeps borders closed'));
  for (const kind of ['Peace', 'Alliance', 'War'] as const) expect(turns).toContain(`<b>${kind}</b> ${TREATY_DEFINITION[kind]}`);
});

it('keeps surprise-war attacks this turn distinct from next-turn alliance betrayal', () => {
  const turns = renderToStaticMarkup(<GuideContent section="turns" />);
  expect(turns).toContain('Declaring War while at Peace is a surprise war. It takes effect at once, so an attack ordered with it resolves the same turn.');
  expect(turns).toContain('Declaring War on an ally is always a betrayal. It takes effect next turn');
});

it('explains population as territory and the way cities claim land', () => {
  const cities = renderToStaticMarkup(<GuideContent section="cities" />);
  expect(cities).toContain('Population is territory');
  expect(cities).toContain('filling each ring');
  expect(cities).toContain(`only works tiles within ${WORK_RADIUS} tiles`);
  expect(cities).toContain('Code of Laws');
  expect(cities).toContain('Select a city, choose Buy a tile, then click a lit tile on the map');
  expect(cities).toContain('in any order');
  expect(cities).toContain(`A tile costs ${TILE_GOLD_BASE} Gold, plus ${TILE_GOLD_PER_TILE} Gold for each tile the city has gained`);
  expect(cities).toContain(`Tiles in the second ring cost ${tileRingPercent(2)}% of that, and tiles in the third ring ${tileRingPercent(3)}%`);
  expect(cities).toContain('shared with buying units and buildings');
  expect(cities).not.toContain('Once you know Code of Laws, the price');
  expect(cities).not.toContain('border growth');
});

it('uses the generated Library, Agoge and resource-specific Deep Mine rules', () => {
  const cities = renderToStaticMarkup(<GuideContent section="cities" />);
  expect(RULES.factionBuildingStats.Athens.Laboratory.yields.production).toBe(0);
  expect(RULES.factionBuildingStats.Athens.Laboratory.yields.science).toBeGreaterThan(0);
  expect(cities).toContain(`Writing (${RULES.buildingNames.Laboratory} for Science)`);
  expect(cities).toContain(`Sparta’s ${RULES.buildingNames.WarShrine}`);
  const mine = RULES.improvements.forResource;
  expect(cities).toContain(`+${mine.Copper.upgradeYields.gold} Gold on Copper`);
  expect(cities).toContain(`+${mine.Iron.upgradeYields.production} Production on Iron`);
  expect(cities).toContain(`+${mine.Coal.upgradeYields.production} Production on Coal`);
  expect(RULES.techCardEffects.Athens.IronWorking).toContain(`Mines upgrade: Deep Mine (+${mine.Copper.upgradeYields.gold} Gold on Copper, +${mine.Iron.upgradeYields.production} Production on Iron and Coal)`);
});

it('teaches the exported era curve, pre-Modern class bonus, Keshig upgrade and barbarian XP cap', () => {
  const combat = renderToStaticMarkup(<GuideContent section="combat" />);
  expect(combat).toContain(RULES.eraCurve.stepPercent.map((p) => `${p}%`).join(', '));
  expect(combat).toContain(`before the ${RULES.eraNames[RULES.triangleEndsAtEraTier]} era`);
  expect(combat).toContain(`+${RULES.classBonusPercent}% of its strength`);
  expect(combat).toContain('a counter one era behind is reliable only in the early eras');
  expect(combat).not.toMatch(/two fights in three|class bonus is larger than one era|bites the same in every era/);
  expect(combat).toContain(`next upgrade is a ${UNIT_NAME[UNIT_STATS.Cataphract.upgradesTo!]}`);
  expect(combat).toContain(`first promotion, at ${RULES.promotion.xp[0]} XP`);
  expect(combat).toContain('A kill earns extra experience');
  expect(combat).not.toContain('A kill doubles the experience');
});

it('names every scoring category and the current-era research queue controls', () => {
  const victory = renderToStaticMarkup(<GuideContent section="victory" />);
  const controls = renderToStaticMarkup(<GuideContent section="controls" />);
  expect(victory).toContain('members’ faction goals, scored Age cards, sacked capitals (Celts only), held rival original capitals, and held world wonders');
  expect(victory).toContain('You lose points when you lose a held goal, capital or wonder');
  expect(controls).toContain('Click a current-era technology to research it now, moving it to the front while keeping the rest of the queue in order');
  expect(controls).toContain('Shift+click adds a technology to the end of the queue');
  expect(controls).not.toMatch(/missing prerequisites first|drops the rest/);
});

it('locates city totals, next-tile yields, and city comparison', () => {
  const cities = renderToStaticMarkup(<GuideContent section="cities" />);
  expect(cities).toContain('Food, Production, Gold and Science per turn are always shown');
  expect(cities).toContain('“Next tile” under the bar gives that tile’s yield');
  expect(cities).toContain('Compare cities (C) opens the table');
});

it('starts with the capital build and says units cannot attack empires at Peace (G8)', () => {
  const turns = renderToStaticMarkup(<GuideContent section="turns" />);
  expect(turns).toContain('Choose what it builds on turn 1');
  expect(turns).toContain('can’t attack another empire’s units or cities until one side declares War');
});

it('makes the full sound credits available inside the guide', () => {
  const credits = renderToStaticMarkup(<GuideContent section="credits" />);
  expect(credits).toContain('<summary>Full sound credits</summary>');
  expect(credits).not.toContain('audio/CREDITS.md');
  expect(credits).toContain('<summary>Open-source licences</summary>');
  expect(credits).toContain('Legatum is not affiliated with or endorsed by Take-Two Interactive or Firaxis Games.');
});

it('explains map resources, reveals, supply and idle buildings and units (G11)', () => {
  const cities = renderToStaticMarkup(<GuideContent section="cities" />);
  expect(cities).toContain('Resources: strategic supply begins with Iron');
  expect(cities).toContain('The Ancient era has no resource requirements.');
  // The era table: era, resource, what uses it, and what reveals it.
  const text = (html: string) => html.replace(/<[^>]*>/g, '').trim();
  const rows = [...cities.matchAll(/<tr>(.*?)<\/tr>/g)].filter((m) => m[1].startsWith('<td')).map((m) => [...m[1].matchAll(/<td(?: [^>]*)?>(.*?)<\/td>/g)].map((c) => text(c[1])));
  expect(rows.map((r) => `${r[0]} ${r[1]}`)).toEqual(['Ancient None', 'Classical Iron', 'Medieval Iron', 'Renaissance Niter', 'Industrial Coal', 'Machine Age Oil', 'World War Oil', 'Modern Oil', `Future ${eraResource(8)}`]);
  expect(rows[3]).toEqual(['Renaissance', 'Niter', 'Musketmen, Arquebusiers, Cuirassiers and Banks', 'Gunpowder']);
  expect(rows[0][3]).toBe('—');
  expect(rows[1][3]).toBe('Iron Working');
  expect(rows[5][2]).toContain('Biplanes');
  expect(rows[6][2]).toContain('Fighters, Bombers and Research Labs');
  expect(renderToStaticMarkup(<GuideContent section="combat" />)).toContain('Each uses 1 Oil supply while it exists');
  expect(cities).toContain('Walls, the Granary and Mill, Roadworks, world wonders and faction landmarks');
  expect(cities).toContain('A unit that upgrades switches to the final unit’s resource');
  expect(cities).toContain('supplies 2 of its resource, whether or not you work it. Improved, it supplies 3 (a Mine, Saltpetre Works, Oil Well or Quartz Mine)');
  expect(cities).toContain('the newest go idle until supply returns');
  expect(cities).toContain('an idle unit fights 25% weaker');
  expect(cities).not.toMatch(/stockpile|refunded/i);
  expect(cities).toContain('Every start has a Copper deposit two tiles away and an Iron deposit three to five tiles away');
});

it('explains the Mongol Medieval-era Walls and Siege gate', () => {
  const combat = renderToStaticMarkup(<GuideContent section="combat" />);
  expect(combat).not.toContain('unless they capture');
  expect(combat).toContain('Before the Medieval era, Mongols cannot build Walls, Castles or Star Forts');
  expect(combat).toContain('From the Medieval era they build both.');
});

it('explains Germany’s Forge replacement and double supply', () => {
  const cities = renderToStaticMarkup(<GuideContent section="cities" />);
  expect(cities).toContain('Ruhr Foundry replaces the Forge');
  expect(cities).toContain('German deposits supply twice as much');
  expect(cities).not.toContain('less move until Industrialization');
});

it('explains that a VP win means holding the target, and what resets it', () => {
  const victory = renderToStaticMarkup(<GuideContent section="victory" />);
  expect(victory).toContain('Winning · hold the target');
  expect(victory).toContain('only after it <b>holds</b> its target');
  expect(victory).toContain('10 turns in a row');
  expect(victory).toContain('goes back to zero');
  expect(victory).toContain('The last surviving team also wins');
  expect(victory).not.toMatch(/reach their target together/);
});

it('explains Mercenary Companies from the generated numbers', () => {
  const c = RULES.company;
  const turns = renderToStaticMarkup(<GuideContent section="turns" />);
  expect(turns).toContain('Mercenary Companies (Byzantium)');
  expect(turns).toContain(`The price is ${c.hirePercent}% of that version’s Production cost, in Gold, and all of it goes to the seller`);
  expect(turns).toContain(`serves ${c.turns} turns, then goes home`);
  expect(turns).not.toMatch(/tribute|Bought peace/i);
  expect(turns).not.toContain('Hire an army');
  expect(turns).toContain('Allies defend each other, and join wars by choice.');
  expect(turns).toContain('A Peace offer is between two teams');
  expect(turns).toContain('Making Peace on your own is not a betrayal');
  expect(turns).toContain('Thin levies (Byzantium)');
  expect(turns).toContain('at most one military unit per city, plus one more, not counting Scouts');
  expect(turns).not.toContain('120%');
});

it('hides decadence by default and explains the opted-in rule from the core mirror', () => {
  const defaultVictory = renderToStaticMarkup(<GuideContent section="victory" />);
  expect(defaultVictory).not.toContain('Decadence');
  const original = useGame.getState().views;
  const originalSeat = useGame.getState().seat;
  useGame.setState({ seat: 0, views: [{ rule_experiments: { decadence_slope: 45 } } as PlayerView] });
  const host = document.body.appendChild(document.createElement('div'));
  const root = createRoot(host);
  act(() => root.render(<GuideContent section="victory" />));
  const victory = host.innerHTML.replace(/&#x27;/g, "'").replace(/&amp;/g, '&');
  act(() => root.unmount());
  host.remove();
  useGame.setState({ views: original, seat: originalSeat });
  expect(victory).toContain('<b>Decadence.</b>');
  for (const w of ['slows growth (smaller families)', 'Production (sloth)', 'Gold (luxury)', 'Science (complacency)']) expect(victory).toContain(w);
  expect(victory).toContain(`${decadencePercent(30, { decadenceSlope: 45 })}% at 30% ahead, ${decadencePercent(60, { decadenceSlope: 45 })}% at 60% ahead, ${decadencePercent(100, { decadenceSlope: 45 })}% at double the average and ${decadencePercent(200, { decadenceSlope: 45 })}% at triple`);
  expect(victory).toContain('The rate is in proportion to your lead, with no maximum: twice as far ahead, twice the rate.');
  expect(victory).not.toContain('never reaches');
  expect(victory).toContain('The same rate is taken from income that is not a city yield, such as Switzerland’s interest');
  expect(victory).toContain(DECADENCE_FOOD_RULE);
  expect(victory).toContain('a unit deserts each turn, and the treasury forecast warns you before it happens');
  expect(victory).not.toMatch(/share of your Gold income/);
});

it('always shows the occupation lock, and the World Ages entry only when that option is on', () => {
  const render = (experiments?: Record<string, unknown>, section: 'combat' | 'victory' = 'combat') => {
    const original = useGame.getState().views;
    const originalSeat = useGame.getState().seat;
    useGame.setState({ seat: 0, views: [{ rule_experiments: experiments } as PlayerView] });
    const host = document.body.appendChild(document.createElement('div'));
    const root = createRoot(host);
    act(() => root.render(<GuideContent section={section} />));
    const html = host.innerHTML.replace(/&#x27;/g, "'").replace(/&amp;/g, '&');
    act(() => root.unmount());
    host.remove();
    useGame.setState({ views: original, seat: originalSeat });
    return html;
  };
  const on = render();
  expect(on).toContain('Occupation lock.');
  expect(on).toContain('holding it down');
  expect(on).toContain('it cannot move or disband');
  expect(on).toContain('it builds nothing and buys nothing');
  expect(on).toContain('as soon as you make Peace with the empire you took the city from');
  expect(on).not.toMatch(/at the defence ratio|scaled by the ratio|added up/);
  const ages = render(undefined, 'victory');
  expect(ages).toContain('World Ages · ');
  expect(ages).toContain('per Age, kept for good');
});

it('words Switzerland’s tolls and interest the way the faction sheet does, with no maximum', () => {
  const guide = GUIDE_SECTIONS.map((s) => renderToStaticMarkup(<GuideContent section={s.id} />)).join(' ');
  expect(guide).toContain(`Each city earns +1 Gold for every ${RULES.swissTollPopulation} population. Every ${RULES.swissInterestThreshold} Gold in your treasury earns +1 Gold a turn in interest.`);
  expect(guide).not.toMatch(/complete pair|up to 5 Gold interest|one per 100 Gold held/);
});

it('explains India’s closer city spacing and nearby-city yield bonus in the faction guide', () => {
  const guide = renderToStaticMarkup(<GuideContent section="factions" />);
  const plainGuide = guide.replace(/<[^>]*>/g, '');
  const cityGuide = renderToStaticMarkup(<GuideContent section="cities" />);
  expect(guide).toContain('Cities can be founded 3 tiles apart, closer than anyone else.');
  expect(guide).toContain('A city gains 7% Food and Production for every other Indian city within 5 tiles.');
  expect(plainGuide).toContain('Corruption cuts all four yields by 10% per tile beyond 4 from the capital.');
  expect(cityGuide).toContain('New cities must be at least four tiles from every existing city, allies’ cities included. India can found them three tiles apart.');
});
