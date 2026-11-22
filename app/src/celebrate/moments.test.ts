import { beforeAll, describe, expect, it } from 'vitest';
import { FixtureClient } from '../test/fixtureEngine';
import { visible_events_json } from '../wasm-pkg/legatum.js';
import type { Event, Hex, PlayerView, RuinReward } from '../types';
import { momentsForTurn, planMoments, ruinText, type Moment } from './moments';
import { soundHooksForMoments } from './feedback';
import { celebrationFrame } from './mapEffect';
import { feedbackForTurn } from '../feel/turnFeedback';
import { contextFromViews, formatEvent, formatEvents } from '../format';
import { tileIndex } from '../hex';
import { empireOfPlayer } from '../empires';
import { withV2 } from '../test/vpV2Fixture';

const visibleEvents = (events: Event[], before: PlayerView, after: PlayerView): Event[] =>
  JSON.parse(visible_events_json(JSON.stringify(events), JSON.stringify(before), JSON.stringify(after)));

let view: PlayerView;
beforeAll(async () => {
  const client = new FixtureClient();
  await client.newGame(await client.standardSetup('42', 'India', 'Byzantium', 'Ada', 'Bo'));
  client.endTurn({});
  view = client.view(0);
  view.economy.bankruptcy = null;
  view.units = view.units.filter((u) => u.owner === 0);
  for (const c of view.cities) if (c.detail) c.detail.yields.total.food = c.population * 2;
});
const gold: RuinReward = { kind: 'Gold', amount: 40, balance: 80, affordable: null };
const find = (reward: RuinReward = gold, owner = 0): Event => ({ type: 'RuinExplored', unit: 1, owner, at: { q: 2, r: 2 }, reward });
const map = (events: Event[], before = view) => momentsForTurn(events, 0, before, view);

describe('VP v2 moments', () => {
  it('announces a technology reveal once with nearby and owned explored deposits', () => {
    const after = structuredClone(view);
    const city = after.cities.find((c) => c.owner === 0)!;
    const index = tileIndex(city.pos, after.width, after.height);
    after.tiles[index] = { ...after.tiles[index], resource: 'Iron', owner: 0, explored: true };
    const events = [
      { type: 'TechResearched', player: 0, tech: 'IronWorking' },
      { type: 'ResourcesRevealed', player: 0, resource: 'Iron', city: city.id, sources: 1, supply: 2 },
      { type: 'ResourcesRevealed', player: 0, resource: 'Iron', city: null, sources: 0, supply: 0 },
    ] as Event[];
    const cards = momentsForTurn(events, 0, view, after).filter((m) => m.title === 'Iron revealed');
    expect(cards).toHaveLength(1);
    expect(cards[0].effect).toBe('1 deposit near you, 1 inside your borders');
    expect(momentsForTurn(events.slice(1), 0, view, after).some((m) => m.title === 'Iron revealed')).toBe(false);
  });
  const run = (events: Event[]) => momentsForTurn(events, 0, view, withV2(view, { era: 2, scored: [2] }));

  it('entering an era is your held era card that lists each open Age card; scoring one is a major +1 VP moment', () => {
    const [entered] = run([{ type: 'EraEntered', player: 0, era: 2, first: false }]);
    expect(entered).toMatchObject({ tier: 'major', title: 'The Classical era', kicker: 'A new era', effect: 'A new era begins' });
    // A rival's entry is a notable card for a teammate only; elsewhere it is the report line and the era notice.
    expect(run([{ type: 'EraEntered', player: 2, era: 2, first: true }])).toEqual([]);
    const [scored] = run([{ type: 'EraObjectiveScored', player: 0, era: 2, slot: 1, objective: 'Gain 4 cities' }]);
    expect(scored).toMatchObject({ tier: 'major', title: 'The Classical Age', effect: '+1 VP, kept for good', kicker: 'Age card scored', emblem: 'milestone' });
    expect(scored.deltas).toEqual([{ counter: 'vp', amount: 1 }]);
    // A rival's score is a report line, not your card.
    expect(run([{ type: 'EraObjectiveScored', player: 2, era: 2, slot: 1, objective: 'Gain 4 cities' }])).toEqual([]);
  });

  it('names the Age you scored while only one Age has begun, and counts Ages once there are several', () => {
    const after = withV2(view, { era: 2, scored: [2] });
    after.eras = after.eras!.filter((e) => e.era === 2);
    after.eras[0].objectives = [{ slot: 0, family: 'Fight', text: 'Breach: capture a walled city', target: 1, objective: { kind: 'Breach' } }];
    const score = (state: PlayerView) => momentsForTurn([{ type: 'EraObjectiveScored', player: 0, era: 2, slot: 0, objective: 'Breach: capture a walled city' }], 0, view, state)[0];
    expect(score(after).details).toEqual(['Scored with Breach', 'Classical Age scored']);
    const more = structuredClone(after);
    more.eras = [...more.eras!, { ...structuredClone(more.eras![0]), era: 3, name: 'Medieval', scored_by: [] }];
    expect(score(more).details?.[1]).toBe('1 of 2 Ages scored');
  });

  it('the era-up card says what changes: the era\'s units and buildings, and how your cities now look', () => {
    const entering: Event[] = [{ type: 'EraEntered', player: 0, era: 2, first: true }];
    const [entered] = run(entering);
    const card = entered.eraUp!;
    expect(card.first).toBe(true);
    // What this era's technologies open for this empire: each item once, units before buildings.
    expect(new Set(card.brings.map((b) => b.key)).size).toBe(card.brings.length);
    const kinds = card.brings.map((b) => b.kind);
    expect(kinds).toEqual([...kinds].sort((a, b) => ['Unit', 'Building', 'Improvement', 'Project'].indexOf(a) - ['Unit', 'Building', 'Improvement', 'Project'].indexOf(b)));
    // The board change: your cities from the Ancient to the Classical style, at your largest city's size.
    expect(card.cities.from).toEqual({ era: 0, name: 'Ancient' });
    expect(card.cities.to).toEqual({ era: 1, name: 'Classical' });
    expect(['hamlet', 'town', 'city', 'metropolis']).toContain(card.cities.tier);
    // Entering the first era has nothing to change from.
    expect(momentsForTurn([{ type: 'EraEntered', player: 0, era: 1, first: false }], 0, view, view)[0].eraUp!.cities.from).toBeNull();
    // Your own era-up takes the organic india sting (the legendary hook), not the tech pluck a plain tech earns.
    expect(soundHooksForMoments([entered], entering)).toEqual({ techComplete: 'fanfare.legendary' });
    // A teammate's or a rival's entry never takes it over.
    expect(soundHooksForMoments(run([{ type: 'EraEntered', player: 2, era: 2, first: true }]), [{ type: 'EraEntered', player: 2, era: 2, first: true }])).toEqual({});
  });

});

describe('truthful ruin moments', () => {
  it('uses the owner faction for production, captured buildings and ruin purchases', () => {
    const after = structuredClone(view);
    after.players.find((p) => p.id === 0)!.faction = 'Athens';
    const city = after.cities.find((c) => c.owner === 0)!;
    city.detail!.buildings = ['Library'];
    const item = { kind: 'Building' as const, building: 'Library' as const };
    const events: Event[] = [
      { type: 'ProductionCompleted', city: city.id, owner: 0, item },
      { type: 'CityCaptured', city: city.id, from: 2, to: 0 },
      find({ ...gold, affordable: { city: city.id, item, cost: 80 } }),
    ];
    const moments = momentsForTurn(events, 0, view, after);
    expect(moments.find((m) => m.eventIndex === 0)?.title).toBe('Lyceum');
    expect(moments.find((m) => m.eventIndex === 1)?.details.join(' ')).toContain('Buildings kept: Lyceum');
    expect(moments.find((m) => m.eventIndex === 2)?.details.join(' ')).toContain('Made affordable: Lyceum');
  });
  it('ordinary finds are notable and get their own Discoveries report with exact numbers', () => {
    const [m] = map([find()]);
    expect(m.tier).toBe('notable'); expect(m.effect).toBe('+40 Gold');
    expect(m.at).toEqual({ q: 2, r: 2 }); expect(m.deltas).toEqual([{ counter: 'gold', amount: 40 }]);
    expect(formatEvent(find(), contextFromViews(null, view))).toMatchObject({ category: 'discoveries', details: ruinText(gold, () => 'Home').details });
  });
  it('tiers an actual Buy crossing, population level, rare Scout and completing Science as major', () => {
    const rewards: RuinReward[] = [
      { ...gold, affordable: { city: 1, item: { kind: 'Unit', unit: 'Scout' }, cost: 60 } },
      { kind: 'Population', city: 1, amount: 1, population: 4, yields: { food: 2, production: 0, science: 1, gold: 0 } },
      { kind: 'FreeScout', unit: 9 },
      { kind: 'Science', amount: 25, research: { tech: 'BronzeWorking', cost: 40, progress_before: 20, income: 2, turns_saved: 10, completes: true } },
    ];
    for (const reward of rewards) expect(map([find(reward)])[0].tier).toBe('major');
    expect(map([find(rewards[0])])[0].details.join(' ')).toContain('60 Gold');
  });
  it('does not infer a pivotal ruin from unrelated income or a completed tech elsewhere in the turn', () => {
    const science: RuinReward = { kind: 'Science', amount: 25, research: { tech: 'BronzeWorking', cost: 40, progress_before: 40, income: 5, turns_saved: 0, completes: false } };
    expect(map([find(science), { type: 'TechResearched', player: 0, tech: 'BronzeWorking' }]).find((m) => m.eventIndex === 0)?.tier).toBe('notable');
  });
  it('cannot turn even a visible rival ruin into a reward or disclose its private numbers', () => {
    const rival = find(gold, 2);
    expect(map([rival])).toEqual([]);
    expect(visibleEvents([rival], view, view)).toEqual([]);
  });
  it('partner cards are smaller, have no private-counter flight and no haptic', () => {
    const partner = find(gold, 1), moments = map([partner]);
    expect(moments[0]).toMatchObject({ partner: true, deltas: [] });
    expect(feedbackForTurn([partner, { type: 'EraObjectiveScored', player: 1, era: 2, slot: 1, objective: 'Gain 4 cities' }], view)).toEqual(['turnResolved']);
    expect(soundHooksForMoments(moments, [partner])).toEqual({});
  });
  it('is pure, orders by tier, caps at three and retains every collapsed event for the report', () => {
    const events = [find(), find({ kind: 'FreeScout', unit: 9 }), find(), find(), find()];
    const original = structuredClone(events);
    const moments = map(events), plan = planMoments(moments);
    expect(events).toEqual(original);
    expect(plan.shown.map((m) => m.eventIndex)).toEqual([1, 0, 2]);
    expect(plan.collapsed.map((m) => m.eventIndex)).toEqual([3, 4]);
    expect(planMoments([])).toEqual({ shown: [], collapsed: [] });
  });
  it('deduplicates discovery haptics and attaches the highest applicable sound tier to the same call', () => {
    const events = [find(), find({ kind: 'FreeScout', unit: 9 }), find()];
    expect(feedbackForTurn(events, view)).toEqual(['turnResolved', 'discovery']);
    expect(soundHooksForMoments(map(events), events)).toEqual({ discovery: 'sting.major' });
  });
  it('keeps lore free of fabricated numerical effects', () => {
    expect(ruinText(gold, () => 'Home').lore).not.toMatch(/\d/);
  });
});

describe('barbarian camp clearing', () => {
  const campAt: Hex = { q: 5, r: 5 };
  const cleared = (owner = 0, gold = 50): Event => ({ type: 'BarbarianCampCleared', camp: 1, at: campAt, by: owner, unit: 3, gold });

  it('celebrates like a ruin: Gold as a delta, the place named, Show on map', () => {
    const [m] = map([cleared()]);
    expect(m).toMatchObject({ tier: 'notable', title: 'The raiders’ camp falls', effect: '+50 Gold', at: campAt });
    expect(m.deltas).toEqual([{ counter: 'gold', amount: 50 }]);
    expect(m.details[0]).toMatch(/^Cleared /);
    expect(m.lore).not.toMatch(/\d/);
  });

  it('every clearer, Byzantium too, sees only Gold, in the card and the report line', () => {
    const events = [cleared()];
    const after = structuredClone(view);
    after.players.find((p) => p.id === 0)!.faction = 'Byzantium';
    for (const [m] of [map(events), momentsForTurn(events, 0, view, after)]) expect(m.effect).toBe('+50 Gold');
    const lines = formatEvents(events, contextFromViews(view, after));
    expect(lines).toHaveLength(1);
    expect(lines[0].text).toContain('+50 Gold');
    expect(lines[0].text).not.toContain('Science');
  });

  it('borders growing over an empty camp pays nothing and earns no celebration', () => {
    const dispersed: Event = { type: 'BarbarianCampCleared', camp: 1, at: campAt, by: 0, unit: null, gold: 0 };
    expect(map([dispersed])).toEqual([]);
  });

  it('a partner clearing a camp is a smaller card with no private counter flight', () => {
    const [m] = map([cleared(1)]);
    expect(m).toMatchObject({ partner: true, deltas: [] });
  });
});

it('reuses every tier for city growth/founding, research and Great Works', () => {
  const city = view.cities.find((c) => c.owner === 0)!;
  const events: Event[] = [
    { type: 'CityGrew', owner: 0, city: city.id, population: 3 },
    { type: 'CityFounded', owner: 0, city: 99, name: 'New Home', at: city.pos },
    { type: 'ProductionCompleted', city: city.id, owner: 0, item: { kind: 'Building', building: 'RoyalArsenal' } },
  ];
  expect(map(events).map((m) => m.tier)).toEqual(['legendary', 'notable', 'minor']);
  const before = structuredClone(view); before.players[0].cities = 0;
  expect(map([events[1]], before)[0].tier).toBe('major');
  const research: Event = { type: 'TechResearched', player: 0, tech: 'BronzeWorking' };
  // Every technology you finish earns the unlock card, the first of its era or not.
  for (const known of [[], ['Irrigation']] as const) {
    before.research.known = [...known];
    const [card] = map([research], before);
    expect(card.tier).toBe('major');
    expect(card.unlock?.tech).toBe('BronzeWorking');
    expect(card.lore).toBe(card.unlock?.quote);
  }
  // A partner's technology stays a small card with no unlock card of its own.
  const partner = momentsForTurn([{ type: 'TechResearched', player: 1, tech: 'BronzeWorking' }], 0, before, { ...view, players: view.players.map((p) => ({ ...p, team: 0 })) })[0];
  expect(partner.tier).toBe('notable'); expect(partner.unlock).toBeUndefined();
});

it('has identical reduced-motion spatial frames at all times, with no particles or flights', () => {
  expect(celebrationFrame(20, true)).toEqual(celebrationFrame(500, true));
  expect(celebrationFrame(200, true)).toMatchObject({ particles: false, flight: null, radius: 1 });
  expect(celebrationFrame(100, false)).not.toEqual(celebrationFrame(500, false));
});

it('sorts unsorted caller moments before applying the cap', () => {
  const moments = ['minor', 'major', 'notable', 'legendary'].map((tier, eventIndex) => ({ ...map([find()])[0], tier, eventIndex }) as Moment);
  expect(planMoments(moments).shown.map((m) => m.tier)).toEqual(['legendary', 'major', 'notable']);
});


it('never counts minor report-only moments in the overflow chip', () => {
  const moments = ['major', 'notable', 'notable', 'notable', 'minor', 'minor'].map((tier, eventIndex) => ({ ...map([find()])[0], tier, eventIndex }) as Moment);
  expect(planMoments(moments).collapsed.map((m) => m.tier)).toEqual(['notable']);
});


it('keeps a lost faction goal on its consequential feedback channel instead of a triumphant tier hook', () => {
  const events: Event[] = [{ type: 'FactionGoalLost', player: 0, faction: 'Switzerland', vp: 2 }];
  expect(soundHooksForMoments(map(events), events)).toEqual({});
  expect(feedbackForTurn(events, view)).toContain('vpLost');
});

it('a captured city is a major card with what you gain; a lost city is a sober loss card', () => {
  const city = view.cities.find((c) => c.owner === 0)!;
  const rival = view.players.find((p) => p.team !== view.team)!;
  const taken: Event = { type: 'CityCaptured', city: city.id, from: rival.id, to: 0 };
  const [gain] = map([taken]);
  expect(gain).toMatchObject({ tier: 'major', title: city.name, effect: `Captured from ${empireOfPlayer(view, rival.id)}`, kicker: 'A city taken', at: city.pos });
  expect(gain.loss).toBeUndefined();
  expect(gain.details[0]).toBe(`Population ${city.population}.`);
  expect(soundHooksForMoments([gain], [taken])).toEqual({ cityCaptured: 'sting.major' });
  const lost: Event = { type: 'CityCaptured', city: city.id, from: 0, to: rival.id };
  const [loss] = map([lost]);
  expect(loss).toMatchObject({ tier: 'major', effect: `Taken by ${empireOfPlayer(view, rival.id)}`, kicker: 'A city is lost', loss: true });
  // No triumphant hook: the consequential cityLost feedback stays in charge.
  expect(soundHooksForMoments([loss], [lost])).toEqual({});
  // Captures between two other empires are not ours to celebrate.
  expect(map([{ type: 'CityCaptured', city: city.id, from: rival.id, to: rival.id + 1 }])).toEqual([]);
});

it('an original-capital capture is one Tier A point card for the captor and every rival', () => {
  const city = view.cities.find((c) => c.owner === 0)!;
  const rival = view.players.find((p) => p.team !== view.team)!;
  const events: Event[] = [
    { type: 'CapitalTaken', captor: rival.id, capital: city.id, name: city.name },
    { type: 'CityCaptured', city: city.id, from: 0, to: rival.id },
  ];
  const after = structuredClone(view);
  after.cities = after.cities.map((c) => c.id === city.id ? { ...c, owner: rival.id } : c);
  const rivalCard = momentsForTurn(events, 0, view, after).find((m) => m.owner === rival.id);
  expect(rivalCard).toMatchObject({ tier: 'major', title: city.name, kicker: `Original capital taken from ${empireOfPlayer(after, 0)}`, effect: `${empireOfPlayer(after, rival.id)} scores 1 VP while holding it`, loss: true, seal: 1, frame: view.players.find((p) => p.id === 0)!.faction });
  expect(rivalCard!.details[0]).toBe('The point is lost if anyone takes the city or it is razed.');
  const rivalAfter = structuredClone(after);
  rivalAfter.team = rival.team;
  const ownCards = momentsForTurn(events, rival.id, view, rivalAfter).filter((m) => m.owner === rival.id);
  expect(ownCards).toHaveLength(1);
  expect(ownCards[0].deltas).toEqual([{ counter: 'vp', amount: 1 }]);
  expect(ownCards[0]).toMatchObject({ tier: 'legendary', effect: '+1 VP while you hold it', lore: `${city.name}’s gates are yours.`, seal: 1 });
  expect(ownCards[0].loss).toBeUndefined();
});

it('a capital taken with a world wonder tells both point gains on one card', () => {
  const city = view.cities.find((c) => c.owner === 0)!;
  const rival = view.players.find((p) => p.team !== view.team)!;
  const after = structuredClone(view);
  const tile = tileIndex(city.pos, after.width, after.height);
  after.tiles[tile] = { ...after.tiles[tile], landmark: { building: 'Pyramids', city: city.id, of: rival.id } };
  const events: Event[] = [
    { type: 'CapitalTaken', captor: 0, capital: city.id, name: city.name },
    { type: 'CityCaptured', city: city.id, from: rival.id, to: 0 },
    { type: 'GreatWorkCompleted', player: 0, of: rival.id, building: 'Pyramids', vp: 2, again: true },
  ];
  const cards = momentsForTurn(events, 0, view, after);
  expect(cards).toHaveLength(1);
  expect(cards[0].effect).toContain('+2 held VP from Pyramids');
  expect(cards[0].deltas).toEqual([{ counter: 'vp', amount: 3 }]);
});

it('a sacked city gives the raider a report-backed card and the victim a sober loss card', () => {
  const city = view.cities.find((c) => c.owner === 0)!;
  const victim = view.players.find((p) => p.id === 0)!;
  const raider = view.players.find((p) => p.team !== view.team)!;
  const event: Event = { type: 'CityRazed', city: city.id, from: victim.id, by: raider.id, at: city.pos,
    sack_gold: 180, camp: { q: city.pos.q + 1, r: city.pos.r } };
  const after = structuredClone(view);
  after.cities = after.cities.filter((c) => c.id !== city.id);
  const [loss] = momentsForTurn([event], victim.id, view, after);
  expect(loss).toMatchObject({ tier: 'major', owner: victim.id, title: city.name, at: city.pos,
    kicker: 'A city is lost', effect: `Sacked by ${empireOfPlayer(view, raider.id)}`,
    lore: 'Nothing is left to retake.', loss: true, emblem: 'captured',
    details: ['The city is destroyed and cannot be retaken.'] });

  const raiderAfter = structuredClone(after);
  raiderAfter.team = raider.team;
  const [sack] = momentsForTurn([event], raider.id, view, raiderAfter);
  expect(sack).toMatchObject({ tier: 'major', owner: raider.id, title: city.name, at: city.pos,
    kicker: 'A city sacked', effect: `Sacked from ${empireOfPlayer(view, victim.id)}`, emblem: 'captured' });
  expect(sack.details).toEqual(['Gold taken: 180.', 'A warband camp was raised among the ruins.']);

  const thirdParty = { ...raider, id: 99, name: 'Third party', team: 99 };
  const thirdAfter = structuredClone(after);
  thirdAfter.players.push(thirdParty);
  thirdAfter.team = thirdParty.team;
  expect(momentsForTurn([event], thirdParty.id, view, thirdAfter)).toEqual([]);
});

it('an ordered raze is named as razing, not a fresh loss for the former owner', () => {
  const city = view.cities.find((c) => c.owner === 0)!;
  const former = view.players.find((p) => p.id === 0)!;
  const holder = view.players.find((p) => p.team !== view.team)!;
  const event: Event = { type: 'CityRazed', city: city.id, from: former.id, by: holder.id,
    at: city.pos, sack_gold: 30, ordered: true };
  const after = structuredClone(view);
  after.cities = after.cities.filter((c) => c.id !== city.id);
  expect(momentsForTurn([event], former.id, view, after)[0]).toMatchObject({ kicker: 'A former city razed', loss: false });
  after.team = holder.team;
  expect(momentsForTurn([event], holder.id, view, after)[0]).toMatchObject({ kicker: 'A city razed', effect: 'Razed by order', details: ['Gold gained: 30.'] });
});

describe('growth and border milestones', () => {
  it('a growth that unlocks the Settler earns a card naming it; other growth stays a light slip', () => {
    const after = structuredClone(view);
    const city = after.cities.find((c) => c.owner === 0)!;
    city.population = 2;
    const grew: Event = { type: 'CityGrew', owner: 0, city: city.id, population: 2 };
    const [card] = momentsForTurn([grew], 0, view, after);
    expect(card).toMatchObject({ tier: 'notable', title: `${city.name} grew to 2`, effect: 'Unlocks Settler', at: city.pos });
    expect(formatEvent(grew, contextFromViews(view, after))).toMatchObject({ text: `${city.name} grew to 2 population · unlocks Settler`, tone: 'good', weight: 2 });
    city.population = 3;
    const plain: Event = { ...grew, population: 3 };
    expect(momentsForTurn([plain], 0, view, after)[0]).toMatchObject({ tier: 'minor', effect: 'Population 3' });
    expect(formatEvent(plain, contextFromViews(view, after)).text).toBe(`${city.name} grew to 3 population`);
  });

  it('a claimed border tile names its terrain and what it adds', () => {
    const after = structuredClone(view);
    const city = after.cities.find((c) => c.owner === 0)!;
    const tile = city.detail!.next_claim!.tile;
    const i = tileIndex(tile, after.width, after.height);
    after.tiles[i] = { ...after.tiles[i], terrain: 'Hills', resource: null };
    const claimed: Event = { type: 'TileClaimed', city: city.id, owner: 0, tile };
    const line = formatEvent(claimed, contextFromViews(view, after));
    expect(line.text).toMatch(new RegExp(`^${city.name} claimed a Hills tile \\(\\+\\d+ (Food|Production)`));
    expect(line.tone).toBe('good');
    const [slip] = momentsForTurn([claimed], 0, view, after);
    expect(slip).toMatchObject({ tier: 'minor', at: tile });
    expect(slip.effect).toMatch(/^Hills \+/);
    // A rival's claim is not ours to celebrate.
    expect(momentsForTurn([{ ...claimed, owner: 3 }], 0, view, after)).toEqual([]);
  });
});

describe('reward emblems and gain lists', () => {
  const at: Hex = { q: 5, r: 5 };
  const city = () => view.cities.find((c) => c.owner === 0)!;
  it('a cleared camp wears the camp emblem and lists each gain with its icon', () => {
    const [m] = map([{ type: 'BarbarianCampCleared', camp: 1, at, by: 0, unit: 3, gold: 50 }]);
    expect(m.emblem).toBe('camp');
    expect(m.gains).toEqual([{ icon: 'yield-gold', text: '+50 Gold' }]);
  });
  it('every ruin reward wears the ruin emblem and names its gain', () => {
    const rewards: RuinReward[] = [gold, { kind: 'Science', amount: 30, research: null }, { kind: 'FreeScout', unit: 1 },
      { kind: 'Population', amount: 1, city: city().id, population: 2, yields: { food: 1, production: 0, gold: 0, science: 0 } }];
    for (const reward of rewards) {
      const [m] = map([find(reward)]);
      expect(m.emblem).toBe('ruin');
      expect(m.gains?.length).toBe(1);
    }
    expect(map([find()])[0].gains).toEqual([{ icon: 'yield-gold', text: '+40 Gold' }]);
  });

  it('shows Athens normal ruin reward plus its Science bonus', () => {
    const text = ruinText(gold, () => 'Home', 'Athens');
    expect(text.effect).toContain('+40 Gold · +20 Science (Athens)');
    expect(text.details).toContain('+20 Science (Athens).');
    expect(text.gains).toContainEqual({ icon: 'yield-science', text: '+20 Science (Athens)' });
  });
  it('each other reward kind maps to its own emblem; the faction goal keeps the crest', () => {
    const c = city();
    const kinds: [Event, string | undefined][] = [
      [{ type: 'CityCaptured', city: c.id, from: 2, to: 0 }, 'captured'],
      [{ type: 'CityFounded', city: c.id, owner: 0, name: c.name, at: c.pos }, 'founded'],
      [{ type: 'EraObjectiveScored', player: 0, era: 2, slot: 0, objective: 'Learn 9 technologies' }, 'milestone'],
      [{ type: 'ProductionCompleted', city: c.id, owner: 0, item: { kind: 'Building', building: 'RoyalArsenal' } }, 'greatwork'],
      [{ type: 'FactionGoalCompleted', player: 0, faction: 'India', vp: 3 }, undefined],
    ];
    // A Live goal regained, and one lost, are report lines: no celebration moment.
    expect(map([{ type: 'FactionGoalCompleted', player: 0, faction: 'Switzerland', vp: 2, again: true }])).toEqual([]);
    expect(map([{ type: 'FactionGoalLost', player: 0, faction: 'Switzerland', vp: 2 }])).toEqual([]);
    expect(map([{ type: 'FactionGoalCompleted', player: 0, faction: 'Celts', vp: 2, again: true }])).toEqual([]);
    for (const [event, emblem] of kinds) expect(map([event])[0]?.emblem, event.type).toBe(emblem);
  });

  it('a generic goal card shows the faction’s own goal building where one is painted', () => {
    expect(map([{ type: 'FactionGoalCompleted', player: 0, faction: 'Athens', vp: 2 }])[0]).toMatchObject({ title: 'An inheritance fulfilled', building: 'Library' });
  });

  describe('India’s Imperial Diwan', () => {
    it('holding the Diwan the first time is a legendary card and regaining it is a report line', () => {
      expect(map([{ type: 'FactionGoalCompleted', player: 0, faction: 'India', vp: 2 }]).map((m) => [m.tier, m.title, m.building])).toEqual([['legendary', 'The Imperial Diwan', 'ImperialDiwan']]);
      expect(map([{ type: 'FactionGoalCompleted', player: 0, faction: 'India', vp: 2, again: true }])).toEqual([]);
      expect(map([{ type: 'FactionGoalLost', player: 0, faction: 'India', vp: 2 }])).toEqual([]);
    });
  });

  describe('Sparta’s and the Celts’ goals', () => {
    it('the last retirement is the goal’s legendary card, earlier ones are major cards, and a rival’s retirement is no moment of yours', () => {
      const retired = (n: number): Event => ({ type: 'VeteranRetired', player: 0, unit: 3, kind: 'Spearman', city: 1, retired: n, of: 3 });
      expect(map([retired(1)]).map((m) => [m.tier, m.title, m.effect])).toEqual([['major', 'A veteran retires with honour', '1 of 3 retired']]);
      expect(map([retired(3), { type: 'FactionGoalCompleted', player: 0, faction: 'Sparta', vp: 2 }]).map((m) => [m.tier, m.title, m.building])).toEqual([['legendary', 'The veterans are honoured', 'Agoge']]);
    });
    it('the Celts’ first call is legendary, a regained goal is a report line, a lost camp is a sober major card', () => {
      expect(map([{ type: 'FactionGoalCompleted', player: 0, faction: 'Celts', vp: 2 }]).map((m) => [m.tier, m.title])).toEqual([['legendary', 'The warbands gather']]);
      const lost = map([{ type: 'FactionGoalLost', player: 0, faction: 'Celts', vp: 2 }]);
      expect(lost.map((m) => [m.tier, m.title, m.loss])).toEqual([['major', 'A warband camp is cleared', true]]);
      // With the clearing in view, the card names the empire and the city, and wears the camp emblem.
      const named = map([{ type: 'BarbarianCampCleared', camp: 1, at: { q: 1, r: 1 }, by: 2, unit: null, gold: 0 }, { type: 'FactionGoalLost', player: 0, faction: 'Celts', vp: 2 }]);
      expect(named[0].effect).toMatch(/^−2 VP · .+ cleared the camp near /);
      expect(named[0].emblem).toBe('camp');
    });
  });

  describe('Byzantium’s held goal (the Heir)', () => {
    const lost = (player: number): Event => ({ type: 'FactionGoalLost', player, faction: 'Byzantium', vp: 2 });
    const got = (player: number, again?: boolean): Event => ({ type: 'FactionGoalCompleted', player, faction: 'Byzantium', vp: 2, ...(again ? { again } : {}) });
    it('capturing him is a major card with +2 VP; the loser’s card is a sober loss', () => {
      const mine = map([got(0), lost(2)]);
      expect(mine).toHaveLength(1);
      expect(mine[0]).toMatchObject({ tier: 'major', title: 'The Heir is taken', deltas: [{ counter: 'vp', amount: 2 }] });
      const theirs = map([got(2), lost(1)]);
      expect(theirs).toHaveLength(1);
      expect(theirs[0]).toMatchObject({ tier: 'major', loss: true, partner: true, owner: 1 });
      expect(theirs[0].effect).toContain('−2 VP');
    });
    it('training him is the legendary moment; retaking him is a major card; losing him alone says nobody scores', () => {
      expect(map([got(1)])[0]).toMatchObject({ tier: 'legendary', title: 'Bo · The Heir lives' });
      expect(map([got(1, true), lost(2)])[0]).toMatchObject({ tier: 'major', title: 'Bo · The Heir is retaken' });
      const alone = map([lost(1)])[0];
      expect(alone).toMatchObject({ title: 'Bo · The Heir is lost', loss: true });
      expect(alone.details.join(' ')).toContain('Nobody scores');
    });
    it('a rival holding him with no stake of ours shows no card', () => {
      expect(map([got(2), lost(3)])).toEqual([]);
    });
  });
});

describe('the Mongols’ Yam Relay Station moments', () => {
  const lost = (player: number): Event => ({ type: 'FactionGoalLost', player, faction: 'Mongols', vp: 2 });
  const got = (player: number, again?: boolean): Event => ({ type: 'FactionGoalCompleted', player, faction: 'Mongols', vp: 2, ...(again ? { again } : {}) });
  it('building it is the legendary moment, rebuilding a major card', () => {
    expect(map([got(0)])[0]).toMatchObject({ tier: 'legendary', title: 'The Yam Network', deltas: [{ counter: 'vp', amount: 2 }], building: 'YamRelayStation' });
    expect(map([got(0, true)])[0]).toMatchObject({ tier: 'major', title: 'The Yam Relay Station stands again' });
  });
  it('capturing it is a major card for the captor and a sober loss for the loser', () => {
    const mine = map([got(0), lost(2)]);
    expect(mine).toHaveLength(1);
    expect(mine[0]).toMatchObject({ tier: 'major', title: 'The Yam Relay Station is taken', emblem: 'hall' });
    expect(mine[0].details.join(' ')).toContain('cross all of your land');
    const theirs = map([got(2), lost(1)]);
    expect(theirs).toHaveLength(1);
    expect(theirs[0]).toMatchObject({ tier: 'major', loss: true, partner: true, owner: 1 });
    expect(theirs[0].effect).toContain('−2 VP');
  });
  it('a sacked Station is a loss that says it can be built again; a rival-to-rival hand-over shows nothing', () => {
    const razed = map([{ type: 'CityRazed', city: 1, from: 0, by: 3, at: { q: 2, r: 2 } }, lost(0)]);
    expect(razed.find((m) => m.loss)).toMatchObject({ title: 'The Yam Relay Station is sacked', building: 'YamRelayStation' });
    expect(razed.find((m) => m.loss)!.details.join(' ')).toContain('built again');
    expect(map([got(2), lost(3)])).toEqual([]);
  });
  it('a captured Station city has one card, not the Station’s and a plain capture', () => {
    const after = structuredClone(view);
    const city = after.cities.find((c) => c.owner === 0)!;
    after.courier_halls = [{ of: 2, holder: 0, city: city.id }];
    const moments = momentsForTurn([{ type: 'CityCaptured', city: city.id, from: 2, to: 0 }, lost(2), got(0)], 0, view, after);
    expect(moments.map((m) => m.title)).toEqual(['The Yam Relay Station is taken']);
  });
});

it('Flower Wars creates a named population popup at the receiving city', () => {
  const city = view.cities.find((c) => c.owner === 0)!;
  const moments = map([{ type: 'FlowerWars', city: city.id, owner: 0, population: city.population + 1 }]);
  expect(moments).toContainEqual(expect.objectContaining({
    tier: 'notable', kicker: 'Flower Wars', at: city.pos, effect: `Population ${city.population} → ${city.population + 1}`,
    title: `${city.name} gains 1 population`, deltas: [{ counter: 'population', amount: 1 }], emblem: 'growth',
  }));
});

describe('world wonder moments', () => {
  const city = () => view.cities.find((c) => c.owner === 0)!;
  const rival = () => view.players.find((p) => p.team !== view.team)!;
  const withWonder = (cityId: number, of: number) => {
    const after = structuredClone(view);
    const c = after.cities.find((x) => x.id === cityId)!;
    const index = tileIndex({ q: c.pos.q + 1, r: c.pos.r }, after.width, after.height);
    after.tiles[index] = { ...after.tiles[index], landmark: { building: 'Pyramids', city: cityId, of } };
    return after;
  };

  it('a completed wonder carries its VP, its yield and that only one exists', () => {
    const c = city();
    const [card] = momentsForTurn([
      { type: 'ProductionCompleted', city: c.id, owner: 0, item: { kind: 'Wonder', building: 'Pyramids', tile: { q: c.pos.q + 1, r: c.pos.r } } },
      { type: 'GreatWorkCompleted', player: 0, of: 0, building: 'Pyramids', vp: 2, again: false },
    ], 0, view, view);
    expect(card).toMatchObject({ tier: 'legendary', title: 'Pyramids', kicker: 'World wonder built', place: c.name, emblem: 'greatwork', deltas: [{ counter: 'vp', amount: 2 }] });
    expect(card.effect).toBe(`+2 VP while you hold it · +1 Production in ${c.name} · only one in the world`);
    expect(card.details).toEqual(['Only one in the world: no other empire can build it.']);
  });

  it('capturing a wonder city is one card naming the wonder and the VP gained', () => {
    const c = city(), r = rival();
    const moments = momentsForTurn([
      { type: 'CityCaptured', city: c.id, from: r.id, to: 0 },
      { type: 'GreatWorkLost', player: r.id, of: r.id, building: 'Pyramids', vp: 2 },
      { type: 'GreatWorkCompleted', player: 0, of: r.id, building: 'Pyramids', vp: 2, again: true },
    ], 0, view, withWonder(c.id, r.id));
    expect(moments).toHaveLength(1);
    expect(moments[0]).toMatchObject({ tier: 'legendary', title: 'Pyramids', kicker: 'World wonder taken', place: c.name, effect: `+2 VP · captured from ${empireOfPlayer(view, r.id)}`, deltas: [{ counter: 'vp', amount: 2 }] });
    expect(moments[0].loss).toBeFalsy();
  });

  it('a wonder taken with a city names that city even when its tile is out of sight', () => {
    const c = city(), r = rival();
    const [card] = momentsForTurn([
      { type: 'CityCaptured', city: c.id, from: r.id, to: 0 },
      { type: 'GreatWorkCompleted', player: 0, of: r.id, building: 'Pyramids', vp: 2, again: true },
    ], 0, view, view);
    expect(card).toMatchObject({ kicker: 'World wonder taken', title: 'Pyramids', place: c.name });
  });

  it('losing a wonder city is one loss card with the signed VP', () => {
    const c = city(), r = rival();
    const moments = momentsForTurn([
      { type: 'CityCaptured', city: c.id, from: 0, to: r.id },
      { type: 'GreatWorkLost', player: 0, of: 0, building: 'Pyramids', vp: 2 },
      { type: 'GreatWorkCompleted', player: r.id, of: 0, building: 'Pyramids', vp: 2, again: true },
    ], 0, view, withWonder(c.id, 0));
    expect(moments).toHaveLength(1);
    expect(moments[0]).toMatchObject({ tier: 'major', loss: true, title: 'Pyramids', kicker: 'World wonder lost', place: c.name, effect: `−2 VP · taken by ${empireOfPlayer(view, r.id)}`, deltas: [{ counter: 'vp', amount: -2 }] });
  });

  it('a wonder destroyed with a razed city is a loss card that says it cannot be built again', () => {
    const c = city();
    const [loss] = momentsForTurn([
      { type: 'CityRazed', city: c.id, from: 0, by: rival().id, at: c.pos, ordered: true },
      { type: 'GreatWorkLost', player: 0, of: 0, building: 'Pyramids', vp: 2 },
    ], 0, view, view).filter((m) => m.title === 'Pyramids');
    expect(loss).toMatchObject({ tier: 'major', loss: true, kicker: 'World wonder destroyed', effect: '−2 VP · destroyed with its city' });
    expect(loss.details).toContain('It can never be built again.');
  });
});

describe('goal city moments', () => {
  it('a rival starting the Royal Tomb wears the tomb mark and says where it will stand', () => {
    const after = structuredClone(view);
    const me = after.players.find((p) => p.id === 0)!;
    const rival = after.players.find((p) => p.team !== me.team)!;
    rival.faction = 'Egypt';
    after.tomb_sites = [{ of: rival.id, holder: rival.id, city: 99, at: { q: 3, r: 3 }, name: 'Thebes' }];
    const [card] = momentsForTurn([{ type: 'TombStarted', player: rival.id, turns: 6 }], 0, view, after);
    expect(card).toMatchObject({ goalMark: 'goal-royal-tomb', at: { q: 3, r: 3 } });
    expect(card.effect).toMatch(/is building its Royal Tomb in Thebes: 2 VP to whoever holds the city\. 9 turns of work left\.$/);
  });

  it('the owner’s own Colonia Victrix card wears the laurel', () => {
    const after = structuredClone(view);
    after.players[0].faction = 'Rome';
    const [card] = momentsForTurn([{ type: 'FactionGoalCompleted', player: 0, faction: 'Rome', vp: 2 }], 0, view, after);
    expect(card).toMatchObject({ tier: 'legendary', title: 'Colonia Victrix', goalMark: 'goal-colonia' });
  });
});
