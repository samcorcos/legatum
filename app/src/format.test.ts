import { describe, expect, it } from 'vitest';
import { FixtureClient } from './test/fixtureEngine';
import { tileIndex } from './hex';
import type { Event, Faction, PlayerView, ProductionItem } from './types';
import { BUILDING_KINDS, TECHS, UNIT_KINDS } from './data';
import { withV2 } from './test/vpV2Fixture';
import {
  contextFromViews,
  filterLines,
  formatBreakdown,
  formatEvent,
  formatEvents,
  MOVE_STOPPED_PREFIX,
  PEACE_BLOCK_PREFIX,
  OUTMATCHED_PREFIX,
  TARGET_GONE,
  groupLines,
  movementPaths,
  objectiveCloseLines,
  settlerSiteLines,
  upkeepText,
  plainOrderError,
  type ReportContext,
} from './format';

const ctx: ReportContext = {
  me: 0,
  team: [0, 1],
  playerName: (id) => (id === null ? 'nobody' : ['Ana', 'Ben', 'Hollow Crown', 'Saint Veyra'][id] ?? `E${id}`),
  unitKind: (id) => (id === 7 ? 'Spearman' : id === 9 ? 'Scout' : null),
  cityName: (id) => (id === 1 ? 'Tithe Station' : 'Somewhere'),
  cityPos: (id) => (id === 1 ? { q: 2, r: 2 } : null),
};

it('reports the capture resistance', () => {
  const report = formatEvent({ type: 'CityCaptured', city: 1, from: 2, to: 0 }, ctx);
  expect(report.details.join(' ')).toContain('resist for 6 turns');
});

it('reports the Athenian capture reward as a public technology gain', () => {
  const report = formatEvent({ type: 'AthensCityCaptured', city: 1, from: 2, to: 0 },
    { ...ctx, cityName: () => 'Delphi' });
  expect(report).toMatchObject({ category: 'science', tone: 'good', text: 'Captured Delphi from Athens: you gain one free technology. Choose it.' });
});

it('reports a same-empire handoff as one city line with the tile count', () => {
  const context = { ...ctx, cityName: (id: number) => id === 27 ? 'Sparta Colony 11' : 'Shizuoka' };
  const report = formatEvent({ type: 'TilesReassigned', from: 11, to: 27, owner: 0,
    tiles: [{ q: 12, r: 11 }, { q: 11, r: 11 }] }, context);
  expect(report).toMatchObject({ category: 'cities', tone: 'good', at: { q: 12, r: 11 },
    text: 'Sparta Colony 11 took over 2 tiles from Shizuoka so it can work them' });
});

it('reports the Gold actually seized from a Swiss capture', () => {
  const report = formatEvent({ type: 'CityCaptured', city: 1, from: 2, to: 0, bounty: 168 }, ctx);
  expect(report.text).toContain('168 Gold');
});

it.each([
  ['Athens', { kind: 'Building', building: 'Library' }, 'Lyceum'],
  ['Germany', { kind: 'Building', building: 'Forge' }, 'Ruhr Foundry'],
  ['India', { kind: 'Building', building: 'Market' }, 'Revenue Office'],
  ['Mongols', { kind: 'Unit', unit: 'Cataphract' }, 'Keshig'],
  ['Mongols', { kind: 'Unit', unit: 'Horseman' }, 'Horseman'],
  ['Sparta', { kind: 'Unit', unit: 'Spearman' }, 'Spartan Hoplite'],
  ['Byzantium', { kind: 'Unit', unit: 'Pikeman' }, 'Tagmatic Soldier'],
] as [Faction, ProductionItem, string][])(
  'names %s production, purchases and losses for the event owner', (faction, item, name) => {
    const context = { ...ctx, playerFaction: (id: number): Faction => id === 2 ? faction : 'Switzerland' };
    for (const event of [
      { type: 'ProductionCompleted', city: 1, owner: 2, item },
      { type: 'ItemBought', city: 1, owner: 2, item, gold: 80 },
      item.kind === 'Unit'
        ? { type: 'UnitDestroyed', unit: 9, owner: 2, unit_kind: item.unit, at: { q: 2, r: 2 } }
        : { type: 'BuildingDismantled', city: 1, owner: 2, building: item.kind === 'Building' ? item.building : 'Granary', reason: 'captured duplicate' },
    ] as Event[]) expect(formatEvent(event, context).text).toContain(name);
  },
);

it('names the tile an improvement was built on, by bearing, never coordinates', () => {
  const item: ProductionItem = { kind: 'Improvement', improvement: 'Mine', tile: { q: 3, r: 2 } };
  const completed = formatEvent({ type: 'ProductionCompleted', city: 1, owner: 0, item }, ctx);
  expect(completed.text).toMatch(/^Tithe Station completed Mine, 1\u00a0tile\u00a0(NE|E|SE|SW|W|NW|N|S)$/);
  expect(completed.text).not.toMatch(/\d,/);
  expect(formatEvent({ type: 'ItemBought', city: 1, owner: 0, item, gold: 80 }, ctx).text).toMatch(/bought Mine, 1\u00a0tile\u00a0.* for 80 Gold/);
});

const battle: Event = {
  type: 'Battle',
  attacker: 7,
  attacker_owner: 0,
  defender: { kind: 'Unit', unit: 9, owner: 2, unit_kind: 'Scout' },
  at: { q: 3, r: 1 },
  attacker_strength: { total: 20, parts: [['Spearman', 20]] },
  defender_strength: { total: 8, parts: [['Scout', 5], ['Hills terrain', 5], ['Wounded', -2]] },
  damage_to_attacker: 12,
  damage_to_defender: 55,
  attacker_roll: 104,
  defender_roll: 96,
  attacker_hp_after: 88,
  defender_hp_after: 0,
  ranged: false,
  meeting: false,
};

describe('Alliance reports', () => {
  // Team 0 is the viewer's (Ana & Ben); team 2 is Hollow Crown.
  const allianceCtx: ReportContext = { ...ctx, teamId: 0, treatyKind: () => 'Alliance',
    teamPlayers: (t) => (t === 0 ? [0, 1] : [t]), empireName: (t) => `Team ${t}` };
  const relation = { a: 0, b: 2, state: 'Alliance' as const, effective_turn: 3, guarantee_until: 0, version: 1,
    war_id: null, declarers: [], security: { id: 7, started_turn: 3, protected_until: 13 } };

  it('names the signing, upkeep and the protected window', () => {
    const event: Event = { type: 'TreatyActivated', relation };
    const report = formatEvent(event, allianceCtx);
    // Named by the other empire, never by the internal treaty id.
    expect(report.text).toBe('Alliance with Team 2 signed · begins turn 3');
    expect(formatEvent(event, { ...allianceCtx, teamId: 5 }).text).toBe('Alliance between Team 0 and Team 2 signed · begins turn 3');
    expect(report.details).toEqual([
      'Allies defend each other at once, and each ally chooses whether to join the other’s wars. They share sight and can cross each other’s borders.',
      'Each empire pays 1 Gold a turn for every city it owns, and never less than 1 Gold.',
      'Protected until turn 13: leaving before then costs 120 Gold and is a betrayal.',
    ]);
    // The fixed Standard breach cost is 120 Gold.
    expect(report.details[2]).toContain('costs 120 Gold');
  });

  it('reports Gold payments as a cost for the payer', () => {
    const payment: Event = { type: 'TreatyPayment', player: 0, gold: 30, cause: 'Alliance breach', treaty_id: 7, effective_turn: 3 };
    const mine = formatEvent(payment, allianceCtx);
    expect(mine.text).toBe('Paid 30 Gold: Alliance breach');
    expect(mine).toMatchObject({ category: 'economy', tone: 'bad', players: [0] });
    const teammate = formatEvent({ ...payment, player: 1, gold: 60 }, allianceCtx);
    expect(teammate.text).toBe('Ben paid 60 Gold: Alliance breach');
    expect(teammate.tone).toBe('bad');
  });

  it('names who left the Alliance, coloured from the viewer\'s side', () => {
    const ended = (reason: string): Event => ({ type: 'TreatyEnded', treaty_id: 7, a: 0, b: 2, effective_turn: 20, reason });
    const theyLeft = formatEvent(ended('Hollow Crown left the Alliance'), allianceCtx);
    expect(theyLeft.text).toBe('Team 2 left the Alliance with you · turn 20');
    expect(theyLeft.tone).toBe('bad');
    const youLeft = formatEvent(ended('Ana & Ben left the Alliance'), allianceCtx);
    expect(youLeft.text).toBe('You left the Alliance with Team 2 · turn 20');
    expect(youLeft.tone).toBe('neutral');
    expect(formatEvent(ended('Team eliminated'), allianceCtx).text).toBe('Alliance with Team 2 ended turn 20: Team eliminated');
  });

  it('reports leaving early with its Gold cost: bad for the leaver and for the one left', () => {
    const breach: Event = { type: 'TreatyBreached', treaty_id: 7, by: 0, with: 2, player: 0, cost: 60, effective_turn: 8 };
    const mine = formatEvent(breach, allianceCtx);
    expect(mine.text).toBe('You broke the Alliance with Team 2 early · 60 Gold · turn 8');
    expect(mine.tone).toBe('bad');
    expect(mine.details).toEqual(['Leaving within 10 turns of signing costs the leaver 60 Gold.']);
    // Broken against you: "with you", never the breaker's own name twice.
    const against = formatEvent({ ...breach, by: 2, with: 0, player: 2 }, allianceCtx);
    expect(against.text).toBe('Hollow Crown broke the Alliance with you early · 60 Gold · turn 8');
    expect(against.tone).toBe('bad');
    expect(formatEvent({ ...breach, by: 2, with: 3, player: 2 }, allianceCtx).tone).toBe('neutral');
  });

  it('reports a betrayal and how long the betrayed team refuses an Alliance', () => {
    const betrayed = (by: number, against: number): Event => ({ type: 'AllianceBetrayed', betrayal: { by, against, turn: 52, until: 72 } });
    const onYou = formatEvent(betrayed(2, 0), allianceCtx);
    expect(onYou.text).toBe("Team 2 betrayed you: you won't ally with them until turn 72");
    expect(onYou).toMatchObject({ category: 'diplomacy', tone: 'bad', empire: 2, weight: 2 });
    expect(onYou.details).toEqual(["Leaving an Alliance within 10 turns of signing, or declaring war on an ally, is a betrayal: the betrayed team won't ally with the betrayer for 20 turns."]);
    const byYou = formatEvent(betrayed(0, 2), allianceCtx);
    expect(byYou.text).toBe("You betrayed Team 2: Team 2 won't ally with you until turn 72");
    expect(byYou.tone).toBe('bad');
    const others = formatEvent(betrayed(2, 3), allianceCtx);
    expect(others.text).toBe("Team 2 betrayed Team 3: Team 3 won't ally with them until turn 72");
    expect(others.tone).toBe('neutral');
  });

  it('reports a bought tile with its Gold price', () => {
    const bought: Event = { type: 'TileBought', city: 1, owner: 0, tile: { q: 3, r: 2 }, gold: 40 };
    expect(formatEvent(bought, allianceCtx)).toMatchObject({ text: 'Tithe Station bought a tile for 40 Gold', category: 'economy', tone: 'neutral', at: { q: 3, r: 2 } });
    expect(formatEvent({ ...bought, owner: 2 }, allianceCtx).text).toBe("Hollow Crown's city Tithe Station bought a tile for 40 Gold");
  });
});

describe('split counterattack reports', () => {
  const at = { q: 2, r: 2 };
  const context: ReportContext = { ...ctx, place: () => 'Tessitura',
    unitKind: (id: number) => ([7, 8, 10].includes(id) ? 'Spearman' : id === 11 ? 'Pikeman' : id === 9 ? 'Spearman' : null),
    playerFaction: (id: number) => (id === 0 ? 'Sparta' : 'India') };
  const assault = (attackers: number[], taken: number[], hpAfter: number, defender: Extract<Event, { type: 'Battle' }>['defender']) =>
    attackers.map((attacker, i): Event => ({ ...battle, attacker, at, defender,
      damage_to_attacker: taken[i], damage_to_defender: 15, defender_hp_after: hpAfter, attacker_hp_after: 100 - taken[i] }));
  const spearman = { kind: 'Unit', unit: 9, owner: 2, unit_kind: 'Spearman' } as const;

  it('names the defender once, by place, and only that line says destroyed', () => {
    const events = [...assault([7, 8, 10], [7, 7, 7], 0, spearman),
      { type: 'UnitDestroyed', unit: 9, owner: 2, unit_kind: 'Spearman', at } as Event];
    const lines = formatEvents(events, context);
    expect(lines.map((l) => l.text)).toEqual([
      "Hollow Crown's Spearman was destroyed at Tessitura in a combined assault by your 3 Spartan Hoplites; its counterattack was split, −7 each",
    ]);
    expect(lines[0].tone).toBe('good');
    expect(lines[0].details).toHaveLength(3);
  });

  it('shows the remainder going to one attacker and a defender that held', () => {
    const lines = formatEvents(assault([7, 11], [11, 10], 12, spearman), context);
    expect(lines.map((l) => l.text)).toEqual([
      "Hollow Crown's Spearman held at Tessitura against a combined assault by your Spartan Hoplite and Pikeman; its counterattack was split, −11, −10 (12 HP left)",
    ]);
  });

  it('reports a city that fell with its garrison in one combined line', () => {
    const city = { kind: 'City', city: 1, owner: 2 } as const;
    const events: Event[] = [...assault([7, 8, 10], [5, 5, 5], 0, city),
      { type: 'UnitDestroyed', unit: 9, owner: 2, unit_kind: 'Spearman', at },
      { type: 'CityCaptured', city: 1, from: 2, to: 0 }];
    const lines = formatEvents(events, context);
    expect(lines.map((l) => l.text)).toEqual([
      'Tithe Station fell to a combined assault by your 3 Spartan Hoplites; its garrison was destroyed and the city captured. Its counterattack was split, −5 each',
    ]);
    expect(lines.some((l) => /\d+,\s*\d+/.test(l.text))).toBe(false);
  });

  it('keeps a lone attacker\'s line unchanged', () => {
    const lines = formatEvents([battle], ctx);
    expect(lines[0].text).not.toContain('combined');
  });
});

describe('event formatting', () => {
  it('never exposes CamelCase identifiers in player-facing event messages', () => {
    const at = { q: 1, r: 1 };
    const events: Event[] = [
      ...UNIT_KINDS.flatMap((unit): Event[] => [
        { type: 'ProductionCompleted', city: 1, owner: 0, item: { kind: 'Unit', unit } },
        { type: 'UnitDestroyed', unit: 7, owner: 0, unit_kind: unit, at },
        { type: 'UnitUpgraded', unit: 7, owner: 0, to: unit, gold: 40 },
        { type: 'UnitPromoted', unit: 7, owner: 0, unit_kind: unit, level: 2, at },
      ]),
      ...BUILDING_KINDS.map((building): Event => ({ type: 'ProductionCompleted', city: 1, owner: 0, item: { kind: 'Building', building } })),
      ...TECHS.map((tech): Event => ({ type: 'TechResearched', player: 0, tech })),
    ];
    for (const event of events) {
      const formatted = formatEvent(event, ctx);
      expect([formatted.text, ...formatted.details].join(' ')).not.toMatch(/[a-z][A-Z]/);
    }
  });
  it('reports a promotion with its level and strength bonus, on the map', () => {
    const line = formatEvent({ type: 'UnitPromoted', unit: 7, owner: 0, unit_kind: 'Musketman', level: 3, at: { q: 4, r: 2 } }, { ...ctx, playerFaction: () => 'Sparta' });
    expect(line.text).toBe('Your Musketman was promoted to level 3 (+30% strength)');
    expect(line).toMatchObject({ category: 'combat', tone: 'good', at: { q: 4, r: 2 }, promotion: { level: 3, faction: 'Sparta' } });
  });
  it('reports bankruptcy disbands with the treasury shortfall', () => {
    const line = formatEvent({ type: 'UnitDisbanded', unit: 7, owner: 0,
      reason: 'Treasury ran dry (-3 Gold): could not pay unit upkeep' }, ctx);
    expect(line.text).toBe('Your Spearman was disbanded: Treasury ran dry (-3 Gold): could not pay unit upkeep');
    expect(line.tone).toBe('bad');
    expect(filterLines([line], 'mine', ctx)).toEqual([line]);
  });

  it('headlines a stopped move under Orders with what blocks it; ordinary blocks stay movement detail', () => {
    const stopped = formatEvent({ type: 'MoveBlocked', unit: 7, owner: 0, at: { q: 1, r: 1 }, toward: { q: 2, r: 1 },
      reason: `${MOVE_STOPPED_PREFIX}blocked by Saint Veyra's Scout at 2,1` }, ctx);
    expect(stopped).toMatchObject({ category: 'orders', text: "Your Spearman stopped and needs orders: blocked by Saint Veyra's Scout at 2,1", tone: 'bad' });
    expect(stopped.weight).toBeGreaterThan(0);
    const once = formatEvent({ type: 'MoveBlocked', unit: 7, owner: 0, at: { q: 1, r: 1 }, toward: { q: 2, r: 1 }, reason: 'enemy in the way' }, ctx);
    expect(once.category).toBe('movement');
  });

  it('says when an attack ended because a shot destroyed its target first', () => {
    const l = formatEvent({ type: 'MoveBlocked', unit: 7, owner: 0, at: { q: 1, r: 1 }, toward: { q: 2, r: 1 }, reason: TARGET_GONE }, ctx);
    expect(l).toMatchObject({ category: 'orders', weight: 2, tone: 'neutral', text: "Your Spearman didn't attack: its target was destroyed first. Needs orders." });
  });

  it('reports an Outrider escape as the order result', () => {
    const reason = 'The Swiss Horseman rode away before your Spearman reached it.';
    const result = formatEvent({ type: 'MoveBlocked', unit: 7, owner: 0, at: { q: 1, r: 1 }, toward: { q: 2, r: 1 }, reason }, ctx);
    expect(result).toMatchObject({ category: 'orders', text: reason, tone: 'bad', weight: 2 });
  });

  it('G2: a move onto a unit of a team at Peace is an Orders headline naming it, never hidden movement', () => {
    const l = formatEvent({ type: 'MoveBlocked', unit: 7, owner: 0, at: { q: 1, r: 1 }, toward: { q: 2, r: 1 },
      reason: `${PEACE_BLOCK_PREFIX}Saint Veyra's Scout` }, ctx);
    expect(l).toMatchObject({ category: 'orders', weight: 2, text: "Your Spearman couldn't enter this tile: Saint Veyra's Scout is there and you're at Peace" });
  });

  it('a unit outmatched for a tile by a stronger unit at Peace says who moved in and both strengths', () => {
    const l = formatEvent({ type: 'MoveBlocked', unit: 7, owner: 0, at: { q: 1, r: 1 }, toward: { q: 2, r: 1 },
      reason: `${OUTMATCHED_PREFIX}Saint Veyra's Hoplite is stronger (28 against 20) and moved in` }, ctx);
    expect(l).toMatchObject({ category: 'orders', weight: 2, tone: 'neutral',
      text: "Your Spearman couldn't enter this tile: Saint Veyra's Hoplite is stronger (28 against 20) and moved in" });
    expect(l.details[0]).toContain('keeps its order');
  });

  it("a rival's outmatched unit gets no advice to re-route it", () => {
    const l = formatEvent({ type: 'MoveBlocked', unit: 9, owner: 3, at: { q: 1, r: 1 }, toward: { q: 2, r: 1 },
      reason: `${OUTMATCHED_PREFIX}your Spearman is stronger (28 against 20) and moved in` }, ctx);
    expect(l.details[0]).not.toContain('new route');
  });

  it('G2: attack pursuit outcomes say plainly what happened', () => {
    const base = { type: 'AttackPursuit' as const, unit: 7, owner: 0, at: { q: 1, r: 1 }, target: 9, target_owner: 3, target_kind: 'Scout' as const };
    const followed = formatEvent({ ...base, target_at: { q: 3, r: 1 }, outcome: 'Followed' }, ctx);
    expect(followed).toMatchObject({ category: 'orders', weight: 2,
      text: "Saint Veyra's Scout slipped away; your Spearman followed but couldn't reach it this turn. Needs orders." });
    expect(followed.details[0]).toContain('lasts one turn');
    // G18: an attack on a target out of this turn's reach ends too.
    expect(formatEvent({ ...base, target_at: { q: 4, r: 1 }, outcome: 'OutOfReach' }, ctx))
      .toMatchObject({ category: 'orders', weight: 2, text: "Your Spearman couldn't reach Saint Veyra's Scout at this tile this turn. Needs orders." });
    expect(formatEvent({ ...base, target_at: null, outcome: 'LostSight' }, ctx))
      .toMatchObject({ category: 'orders', text: "Saint Veyra's Scout slipped away; your Spearman lost sight of it and needs orders" });
    expect(formatEvent({ ...base, target_at: { q: 3, r: 1 }, outcome: 'Attacked' }, ctx).text)
      .toBe("Saint Veyra's Scout slipped away; your Spearman followed and attacked it at this tile");
    expect(formatEvent({ ...base, target_at: { q: 2, r: 1 }, outcome: 'AwaitingWar' }, ctx).text)
      .toBe("Your Spearman will attack Saint Veyra's Scout at this tile when War begins next turn");
  });

  it('says why an order failed in the line itself', () => {
    const line = formatEvent({ type: 'OrderFailed', player: 0, reason: 'The game ended before diplomacy could activate' }, ctx);
    expect(line.text).toBe('Order failed: The game ended before diplomacy could activate');
    expect(line.tone).toBe('bad');
  });

  it('reports fog crossings without inventing a destination or animating across hidden steps', () => {
    const events: Event[] = [
      { type: 'UnitMoved', unit: 9, owner: 2, from: { q: 6, r: 5 }, to: { q: 7, r: 5 } },
      { type: 'UnitMoved', unit: 9, owner: 2, from: { q: 7, r: 5 }, to: null },
      { type: 'UnitMoved', unit: 9, owner: 2, from: null, to: { q: 5, r: 7 } },
      { type: 'MoveBlocked', unit: 9, owner: 2, at: null, toward: { q: 5, r: 7 }, reason: 'movement blocked' },
    ];
    const lines = formatEvents(events, ctx);
    expect(lines.map((l) => l.text)).toEqual([
      "Hollow Crown's Scout moved 1 tile, now at this tile",
      "Hollow Crown's Scout left sight at this tile",
      "Hollow Crown's Scout entered sight at this tile",
      "Hollow Crown's Scout was blocked beyond sight",
    ]);
    expect(lines[1].at).toEqual({ q: 7, r: 5 });
    expect(lines[3].at).toBeNull();
    expect(movementPaths(events).has(9)).toBe(false);
  });

  it('explains battles with the numbers that decided them', () => {
    const l = formatEvent(battle, ctx);
    expect(l.category).toBe('combat');
    expect(l.text).toBe("Your Spearman attacked Hollow Crown's Scout at this tile: defender −55 HP, attacker −12 HP. The defender was destroyed.");
    expect(l.hp).toEqual([{ text: '−55 HP', tone: 'good' }, { text: '−12 HP', tone: 'bad' }]);
    expect(l.details[0]).toBe('Strength 20 (Spearman 20) vs 8 (Scout 5, Hills terrain +5, Wounded −2)');
    expect(l.details[1]).toContain('Dealt 55 (roll 104%), took 12 (roll 96%)');
    expect(l.tone).toBe('good');
    expect(l.weight).toBe(2);
    expect(l.at).toEqual({ q: 3, r: 1 });
  });

  it('cities fire back: a city shot names the city and the damage, for both sides, under Combat', () => {
    const shot = { ...battle, attacker: { city: 1 }, attacker_owner: 0, damage_to_attacker: 0, defender_roll: 0,
      damage_to_defender: 14, defender_hp_after: 66, attacker_hp_after: 200, ranged: true,
      attacker_strength: { total: 16, parts: [['City', 10], ['Population', 6]] } } as Event;
    const mine = formatEvent(shot, ctx);
    expect(mine.category).toBe('combat');
    expect(mine.text).toBe("Tithe Station fired on Hollow Crown's Scout: −14 HP (66 left)");
    expect(mine.details[0]).toBe('Strength 16 (City 10, Population +6) vs 8 (Scout 5, Hills terrain +5, Wounded −2)');
    expect(mine.details.join(' ')).toContain('weakest (lowest-HP) visible enemy military unit within 2 tiles');
    expect(mine.tone).toBe('good');
    const theirs = formatEvent({ ...shot, attacker_owner: 2, defender: { kind: 'Unit', unit: 7, owner: 0, unit_kind: 'Spearman' } } as Event, ctx);
    expect(theirs.text).toBe("Hollow Crown's city Tithe Station fired on your Spearman: −14 HP (66 left)");
    expect(theirs.tone).toBe('bad');
    expect(formatEvent({ ...shot, defender_hp_after: 0 } as Event, ctx).text).toBe("Tithe Station fired on Hollow Crown's Scout: −14 HP, destroyed");
    // The arrow starts at the city.
    expect(mine).toMatchObject({ at: battle.type === 'Battle' ? battle.at : null, from: { q: 2, r: 2 }, hp: [{ text: '−14 HP', tone: 'good' }] });
    expect(theirs.hp).toEqual([{ text: '−14 HP', tone: 'bad' }]);
  });

  it('G18: an attack whose defender survives says what happened and that the attacker needs orders', () => {
    const camp = { ...ctx, campAt: (h: { q: number; r: number }) => h.q === 3 && h.r === 1, place: () => '2 tiles NE of Levy Reach' };
    const barbarian: Event = { ...battle, defender: { kind: 'Unit', unit: 9, owner: 255, unit_kind: 'Spearman' },
      damage_to_defender: 20, damage_to_attacker: 32, attacker_hp_after: 68, defender_hp_after: 80 };
    expect(formatEvent(barbarian, camp)).toMatchObject({ category: 'combat', weight: 2,
      text: 'Your Spearman attacked the Barbarian Spearman at the camp 2 tiles NE of Levy Reach: defender −20 HP, attacker −32 HP (68 HP left). Needs orders.' });
    // The defender's side reads as before; a kill or a loss needs no "Needs orders".
    expect(formatEvent({ ...barbarian, attacker_owner: 3, defender: { kind: 'Unit', unit: 9, owner: 0, unit_kind: 'Spearman' } }, ctx).text)
      .not.toContain('Needs orders');
    expect(formatEvent(battle, ctx).text).toContain('The defender was destroyed.');
  });

  it('G21: a unit hit while pulling back says where it got to, for both sides', () => {
    const leaving: Event = { ...battle, attacker: 9, attacker_owner: 2, defender: { kind: 'Unit', unit: 7, owner: 0, unit_kind: 'Archer' },
      damage_to_defender: 38, defender_hp_after: 20, attacker_hp_after: 80, defender_leaving: true };
    const moved: Event = { type: 'UnitMoved', unit: 7, owner: 0, from: { q: 3, r: 1 }, to: { q: 3, r: 0 } };
    const lines = formatEvents([leaving, moved], { ...ctx, place: () => 'Brasslock' });
    expect(lines[0]).toMatchObject({ text: "Your Archer was hit by Hollow Crown's Scout from out of sight while pulling back (−38 HP) and reached Brasslock",
      at: { q: 3, r: 0 }, tone: 'bad', hp: [{ text: '−38 HP', tone: 'bad' }] });
    expect(lines[0].details.join(' ')).toContain('no fortification');
    const theirs = formatEvents([{ ...leaving, attacker: 7, attacker_owner: 0, defender: { kind: 'Unit', unit: 9, owner: 2, unit_kind: 'Archer' } },
      { ...moved, unit: 9, owner: 2 }], ctx);
    expect(theirs[0].text).toBe('Your Spearman hit the retreating Archer (−38 HP); it got away with 20 HP. Needs orders.');
    expect(theirs[0].tone).toBe('good');
    expect(formatEvents([{ ...leaving, attacker_at: { q: 4, r: 1 } }], ctx)[0].text)
      .toBe("Your Archer was hit by Hollow Crown's Scout from 1 tile E while pulling back (−38 HP) and was stopped");
  });

  it('G20: a surprise war is reported as starting this turn, and the victim hears what hit it', () => {
    const relation = { a: 0, b: 2, state: 'War' as const, effective_turn: 12, guarantee_until: 0, version: 2, security: null, war_id: 1, declarers: [2] };
    const victim = { ...ctx, turn: 12, teamId: 0, teamPlayers: (t: number) => (t === 0 ? [0, 1] : [t]), empireName: (t: number) => (t === 2 ? 'Switzerland' : `Team ${t}`) };
    const captured: Event = { type: 'SettlerCaptured', unit: 5, from: 0, to: 2, at: { q: 4, r: 4 } };
    const lines = formatEvents([{ type: 'WarDeclared', relation }, captured], victim);
    expect(lines[0]).toMatchObject({ category: 'diplomacy', tone: 'bad', text: 'Switzerland declared war and captured your Settler at this tile' });
    const declarer = { ...victim, teamId: 2, teamName: (t: number) => (t === 2 ? 'you' : `Team ${t}`) };
    expect(formatEvents([{ type: 'WarDeclared', relation }], declarer)[0].text).toBe('War: Team 0 and you · started this turn');
    // A breach still takes effect next turn.
    expect(formatEvents([{ type: 'WarDeclared', relation: { ...relation, effective_turn: 13 } }], declarer)[0].text).toContain('begins turn 13');
  });

  it('names a war joined through a newly signed Alliance', () => {
    const war = { a: 0, b: 2, state: 'War' as const, effective_turn: 12, guarantee_until: 0, version: 2, security: null, war_id: 1, declarers: [0] };
    const alliance = { ...war, a: 1, b: 2, state: 'Alliance' as const, declarers: [] };
    const events: Event[] = [
      { type: 'TreatyActivated', relation: alliance },
      { type: 'WarJoined', team: 2, ally: 1, relation: war },
    ];
    const victim = { ...ctx, teamId: 0, empireName: (t: number) => ({ 1: 'Athens', 2: 'Mongols' }[t] ?? `Team ${t}`), events };
    expect(formatEvent(events[1], victim).text).toBe('The Mongols allied with Athens and joined its war against you');
  });

  it('marks ranged and meeting engagements', () => {
    const r = formatEvent({ ...battle, ranged: true, defender_hp_after: 40 }, ctx);
    expect(r.text).toContain('bombarded');
    expect(r.details[1]).toContain('no retaliation');
    const m = formatEvent({ ...battle, meeting: true, defender_hp_after: 40 }, ctx);
    expect(m.text).toContain('clashed with');
    expect(m.details.join(' ')).toContain('Meeting engagement');
  });

  it('reports a unit that reached its own city first, and how the fight there ended', () => {
    const own = { type: 'CityReinforced', unit: 7, owner: 0, unit_kind: 'Spearman', city: 1, at: { q: 2, r: 2 } } as Event;
    const held = formatEvents([own], ctx).find((l) => l.text.includes('first'))!;
    expect(held.text).toBe('Your Spearman reached Tithe Station first and held it');
    expect([held.category, held.tone, held.weight]).toEqual(['combat', 'good', 2]);
    const lost = formatEvents([own, { type: 'UnitDestroyed', unit: 7, owner: 0, unit_kind: 'Spearman', at: { q: 2, r: 2 } } as Event], ctx)[0];
    expect(lost.text).toBe('Your Spearman reached Tithe Station first and fought as its garrison until it was destroyed');
    expect(lost.tone).toBe('bad');
    const fell = formatEvents([own, { type: 'CityCaptured', city: 1, from: 0, to: 2 } as Event], ctx)[0];
    expect(fell.text).toBe('Your Spearman reached Tithe Station first and fought as its garrison, but the city fell');
    const theirs = formatEvent({ ...own, owner: 2 } as Event, ctx);
    expect(theirs.text).toContain('reached Tithe Station first');
    expect(theirs.tone).toBe('neutral');
  });

  it('formats breakdowns with signs', () => {
    expect(formatBreakdown({ total: 3, parts: [['A', 5], ['B', -2]] })).toBe('3 (A 5, B −2)');
    expect(formatBreakdown({ total: 0, parts: [] })).toBe('0');
  });

  it('era events read naturally, with the era named by the core', () => {
    const named = { ...ctx, eraName: (era: number) => (era === 2 ? 'Classical' : undefined) };
    expect(formatEvent({ type: 'EraEntered', player: 0, era: 2, first: false }, named)).toMatchObject({
      text: 'You entered the Classical era', tone: 'good', category: 'victory',
    });
    // Your own era-up can be told again from its report line; a rival's cannot.
    expect(formatEvent({ type: 'EraEntered', player: 0, era: 2, first: true }, named).reopenEra).toEqual({ era: 2, first: true });
    expect(formatEvent({ type: 'EraEntered', player: 2, era: 2, first: true }, named).reopenEra).toBeUndefined();
    // No objectives posted (a legacy-rules game, or an era the view does not name): the era's own name, 1-based, as research names it.
    expect(formatEvent({ type: 'EraEntered', player: 2, era: 3, first: true }, named)).toMatchObject({
      text: 'Hollow Crown entered the Medieval era first', category: 'science', weight: 2,
    });
    expect(formatEvent({ type: 'EraObjectiveScored', player: 2, era: 2, slot: 1, objective: 'Gain 4 cities' }, named)).toMatchObject({
      text: 'Hollow Crown scored the Classical Age with Gain 4 cities: +1 VP', tone: 'neutral', weight: 2,
    });
    expect(formatEvent({ type: 'EraObjectiveScored', player: 0, era: 2, slot: 0, objective: 'Gain 4 cities' }, named).tone).toBe('good');
    // A World Age game: the first empire's entry opens the Age for everyone, and a scored card is named.
    const age = { ...named, ageCard: (_era: number, slot: number) => (slot === 1 ? { kind: 'Veterans', n: 4 } : { kind: 'Breach' }) };
    expect(formatEvent({ type: 'EraEntered', player: 2, era: 2, first: true }, age).text).toBe('Hollow Crown entered the Classical era first');
    expect(formatEvent({ type: 'AgeBegan', era: 2, first: 2 }, age)).toMatchObject({
      text: 'The Classical Age began: Hollow Crown entered the Classical era first. Its card is open to every empire.', openLedger: true, category: 'victory',
      reopenEra: { era: 2, first: false, age: 2 },
    });
    expect(formatEvent({ type: 'AgeBegan', era: 2, first: 0 }, age)).toMatchObject({
      text: 'The Classical Age began: You entered the Classical era first. Its card is open to every empire.', reopenEra: { era: 2, first: true, age: 0 },
    });
    // One line about it: the first empire's own entry line folds into the Age line.
    const texts = (evs: Event[]) => formatEvents(evs, age).map((l) => l.text);
    expect(texts([{ type: 'EraEntered', player: 2, era: 2, first: true }, { type: 'AgeBegan', era: 2, first: 2 }])).toEqual([
      'The Classical Age began: Hollow Crown entered the Classical era first. Its card is open to every empire.']);
    expect(texts([{ type: 'EraEntered', player: 2, era: 2, first: false }, { type: 'AgeBegan', era: 2, first: 0 }])).toEqual([
      'Hollow Crown entered the Classical era', 'The Classical Age began: You entered the Classical era first. Its card is open to every empire.']);
    expect(formatEvent({ type: 'EraEntered', player: 2, era: 2, first: false }, age).text).toBe('Hollow Crown entered the Classical era');
    expect(formatEvent({ type: 'EraObjectiveScored', player: 0, era: 2, slot: 1, objective: 'Veterans: earn 4 promotions in battle (a promotion is a level a unit gains from combat experience after this Age began)' }, age).text).toBe('You scored the Classical Age with Veterans: +1 VP');
    // A rival's score is one line with its name.
    expect(formatEvent({ type: 'EraObjectiveScored', player: 2, era: 2, slot: 0, objective: 'Breach: capture a walled city' }, age)).toMatchObject({ text: 'Hollow Crown scored the Classical Age with Breach: +1 VP', category: 'victory', weight: 2 });
    expect(formatEvent({ type: 'EraObjectiveScored', player: 0, era: 2, slot: 0, objective: 'Breach: capture a walled city' }, age).text).toBe('You scored the Classical Age with Breach: +1 VP');
  });

  it('cities read naturally', () => {
    expect(formatEvent({ type: 'CityGrew', city: 1, owner: 0, population: 3 }, ctx)).toMatchObject({
      text: 'Tithe Station grew to 3 population',
      at: { q: 2, r: 2 },
    });
    expect(formatEvent({ type: 'TechResearched', player: 0, tech: 'Philosophy' }, ctx).text).toBe('You researched Philosophy');
  });

  it('collapses single-step moves into one line per unit and keeps paths for animation', () => {
    const ev: Event[] = [
      { type: 'UnitMoved', unit: 9, owner: 2, from: { q: 0, r: 0 }, to: { q: 1, r: 0 } },
      { type: 'UnitMoved', unit: 7, owner: 0, from: { q: 5, r: 5 }, to: { q: 5, r: 4 } },
      { type: 'UnitMoved', unit: 9, owner: 2, from: { q: 1, r: 0 }, to: { q: 2, r: 0 } },
      { type: 'UnitMoved', unit: 9, owner: 2, from: { q: 2, r: 0 }, to: { q: 2, r: 1 } },
    ];
    const lines = formatEvents(ev, ctx);
    expect(lines).toHaveLength(2);
    expect(lines[0].text).toBe("Hollow Crown's Scout moved 3 tiles, now at this tile");
    const paths = movementPaths(ev);
    expect(paths.get(9)).toEqual([
      { q: 0, r: 0 },
      { q: 1, r: 0 },
      { q: 2, r: 0 },
      { q: 2, r: 1 },
    ]);
    expect(paths.get(7)).toHaveLength(2);
  });

  it('filters by mine/team/all and groups by category', () => {
    const ev: Event[] = [
      battle,
      { type: 'CityFounded', city: 1, owner: 1, name: 'Harmonia', at: { q: 0, r: 0 } },
      { type: 'CityFounded', city: 2, owner: 3, name: 'Veyra', at: { q: 9, r: 9 } },
      { type: 'GameWon', team: 0, reason: 'VP target reached' },
    ];
    const lines = formatEvents(ev, ctx);
    expect(filterLines(lines, 'all', ctx)).toHaveLength(4);
    expect(filterLines(lines, 'team', ctx)).toHaveLength(3); // battle, Harmonia, GameWon (global)
    expect(filterLines(lines, 'mine', ctx)).toHaveLength(2); // battle, GameWon
    const groups = groupLines(lines);
    expect(groups.map((g) => g.category)).toEqual(['victory', 'combat', 'cities']);
  });
});


it('reports research advancement and a finished queue as the next decision', () => {
  expect(formatEvent({ type: 'ResearchQueueAdvanced', player: 0, next: 'Writing' }, ctx).text).toBe('Next research: Writing');
  expect(formatEvent({ type: 'ResearchQueueAdvanced', player: 0, next: null }, ctx).text).toBe('Research queue finished: choose next');
});

it('scopes diplomacy reports to the involved teams and only announces arrivals to recipients', () => {
  const recipient: ReportContext = { ...ctx, teamId: 0, teamPlayers: (t) => t === 0 ? [0, 1] : [t + 1] };
  const offer: Event = { type: 'ProposalCreated', proposal: { id: 7, kind: 'Alliance', from: 1, to: 0, proposer: 2, created_turn: 2, respond_by: 4, relation_version: 0 } };
  expect(formatEvent(offer, recipient)).toMatchObject({ openInboxTeam: 1, players: [2, 0, 1] });
  expect(formatEvent(offer, recipient).text).toContain('arrived');
  // The line names the other empire by faction and seat, and carries it for the color mark.
  const named = formatEvent(offer, { ...recipient, playerFaction: () => 'Celts', playerName: (id) => `AI ${id}` });
  expect(named.text).toBe('Alliance offer from Celts arrived · respond by turn 4');
  expect(formatEvent({ type: 'ProposalAccepted', proposal: offer.proposal, effective_turn: 5 }, { ...recipient, playerFaction: () => 'Celts', playerName: (id) => `AI ${id}` }).text)
    .toBe('Alliance offer from Celts accepted · turn 5');
  expect(named.empire).toBe(1);
  const sent = formatEvent(offer, { ...recipient, me: 2, teamId: 1 });
  expect(sent.text).toContain('sent'); expect(sent.openInboxTeam).toBeUndefined();
  const foreign: Event = { type: 'AllianceBetrayed', betrayal: { by: 1, against: 2, turn: 5, until: 25 } };
  const lines = formatEvents([foreign], recipient);
  expect(filterLines(lines, 'team', recipient)).toEqual([]);
  expect(filterLines(lines, 'all', recipient)).toHaveLength(1);
});

it('reports the pairwise Peace and the allied war that continues', () => {
  const reportCtx: ReportContext = { ...ctx, teamId: 0, teamPlayers: (team) => [team],
    empireName: (team) => ['Mongols', 'Byzantium', 'Sparta'][team],
    peaceContinuing: () => [[0, 2]] };
  const proposal = { id: 17, kind: 'Peace' as const, from: 1, to: 0, proposer: 1, created_turn: 86,
    respond_by: 88, relation_version: 2, required_teams: [0, 1] };
  const accepted = formatEvent({ type: 'ProposalAccepted', proposal, effective_turn: 87 }, reportCtx);
  expect(accepted.text).toContain('Peace with Byzantium');
  expect(accepted.details.join(' ')).toContain('Your war with Sparta continues.');
  expect(accepted.players).toEqual([0, 1]);
});

it.each([
  [0, 'Peace with Byzantium', 'your war with Byzantium', 'Your war with Sparta continues.'],
  [1, 'Peace with Mongols', 'your war with Mongols', 'Your war with Sparta continues.'],
  [2, 'Peace offer from Byzantium to Mongols', 'the war between Byzantium and Mongols', 'Your war with Mongols continues.'],
  [3, 'Peace offer from Byzantium to Mongols', 'the war between Byzantium and Mongols', 'The war between Byzantium and Sparta continues.'],
])('Peace report reads from viewer team %i', (viewer, offerText, warText, continued) => {
  const context: ReportContext = { ...ctx, teamId: viewer, teamPlayers: (team) => [team],
    empireName: (team) => ['Mongols', 'Byzantium', 'Sparta', 'Athens'][team],
    peaceContinuing: () => [[0, 2], [1, 2]] };
  const proposal = { id: 17, kind: 'Peace' as const, from: 1, to: 0, proposer: 1, created_turn: 86,
    respond_by: 88, relation_version: 2, required_teams: [0, 1] };
  const created = formatEvent({ type: 'ProposalCreated', proposal }, context);
  const accepted = formatEvent({ type: 'ProposalAccepted', proposal, effective_turn: 87 }, context);
  expect(created.text).toContain(offerText);
  expect(created.text).toContain(`would end ${warText}`);
  expect(accepted.text).toContain(`ended ${warText}`);
  expect(accepted.details.join(' ')).toContain(continued);
  expect(`${created.text} ${accepted.text} ${accepted.details.join(' ')}`).not.toMatch(/their war/);
});

it('names players as the rest of the UI does: You, or the empire, never a raw seat name', () => {
  const player = (id: number, name: string, faction: Faction) => ({ id, name, faction, team: id, is_human: id === 0, alive: true, vp: 0, vp_sources: [], cities: 1, faction_goal_done: false });
  const view = {
    me: 0, team: 0, units: [{ id: 9, kind: 'Scout', owner: 1, pos: { q: 3, r: 1 } }], cities: [],
    players: [player(0, 'Player 1', 'India'), player(1, 'Swiss envoy', 'Switzerland'), player(2, 'AI 3', 'Celts'), player(3, 'AI 4', 'Celts')],
    diplomacy: { relations: [] },
  } as unknown as PlayerView;
  const ctx = contextFromViews(null, view);
  expect(formatEvent({ type: 'TreatyPayment', player: 0, gold: 30, cause: 'Alliance breach', treaty_id: 1, effective_turn: 4 }, ctx).text)
    .toBe('Paid 30 Gold: Alliance breach');
  expect(formatEvent({ type: 'UnitMoved', unit: 9, owner: 1, from: { q: 2, r: 1 }, to: { q: 3, r: 1 } }, ctx).text).toBe("Switzerland's Scout moved 1 tile, now at this tile");
  // Two teams play Celts, so the player's name tells them apart.
  expect(formatEvent({ type: 'CityFounded', owner: 2, city: 4, name: 'Cairnhold', at: { q: 1, r: 1 } } as Event, ctx).text).toBe('Celts · AI 3 founded Cairnhold');
  expect(formatEvent({ type: 'CityFounded', owner: 0, city: 5, name: 'Brasslock', at: { q: 1, r: 1 } } as Event, ctx).text).toBe('You founded Brasslock');
  expect(formatEvent({ type: 'MoveBlocked', unit: 7, owner: 0, at: { q: 2, r: 1 }, toward: { q: 3, r: 1 }, reason: `${PEACE_BLOCK_PREFIX}Swiss envoy's Scout` } as Event, { ...ctx, unitKind: () => 'Spearman' }).text)
    .toBe("Your Spearman couldn't enter this tile: Switzerland's Scout is there and you're at Peace");
  expect(ctx.renamePlayers!("blocked by Player 1's Spearman and AI 4's Scout")).toBe("blocked by your Spearman and Celts · AI 4's Scout");
  // A fallen city's force contest names the winning side by its empire, never its seat label.
  const cityTie = (reason: string) => formatEvent({ type: 'MoveBlocked', unit: 7, owner: 0, at: { q: 2, r: 1 }, toward: { q: 3, r: 1 }, reason } as Event,
    { ...ctx, unitKind: () => 'Spearman' }).details?.[0];
  expect(cityTie('AI 4 brought equal force (30 each) and won the draw for the city')).toContain('Celts · AI 4 brought equal force (30 each)');
  expect(cityTie('AI 4 took the city with force 40 against your 30')).toContain('Celts · AI 4 took the city');
  expect(cityTie('Player 1 brought equal force (30 each) and won the draw for the city')).toBe('You brought equal force (30 each) and won the draw for the city');
  // The reason is written for the losing unit's owner: a rival's loss names that rival, never "your".
  const rivalLoss = (reason: string) => formatEvent({ type: 'MoveBlocked', unit: 9, owner: 1, at: { q: 2, r: 1 }, toward: { q: 3, r: 1 }, reason } as Event,
    { ...ctx, unitKind: () => 'Spearman' }).details?.[0];
  expect(rivalLoss('Player 1 took the city with force 40 against your 30')).toBe("You took the city with force 40 against Switzerland's 30");
});

it('renames leading player subjects in invalidation reasons, including elimination and regex characters', () => {
  const view = {
    me: 0, team: 0, units: [], cities: [], diplomacy: { relations: [] },
    players: [
      { id: 0, name: 'Player 1', faction: 'India', team: 0 },
      { id: 1, name: 'AI 2', faction: 'Mongols', team: 1 },
      { id: 2, name: 'A.[3]+ (Kin)', faction: 'Celts', team: 2 },
      { id: 3, name: 'Player 10', faction: 'Byzantium', team: 0 },
    ],
  } as unknown as PlayerView;
  const ctx = contextFromViews(null, view);
  const invalidated = (reason: string) => formatEvent({ type: 'ProposalInvalidated',
    proposal: { id: 1, from: 1, to: 0, proposer: 1, kind: 'Alliance', created_turn: 9, respond_by: 11, relation_version: 0 },
    effective_turn: 11, reason }, ctx).text;
  expect(invalidated('Player 1 were eliminated')).toContain('invalidated: you were eliminated');
  expect(invalidated('AI 2 was eliminated')).toContain('invalidated: Mongols was eliminated');
  expect(invalidated('A.[3]+ (Kin) were eliminated')).toContain('invalidated: Celts were eliminated');
  expect(invalidated('Player 10 was eliminated')).toContain('invalidated: Byzantium was eliminated');
  for (const reason of ['Relationship changed', 'Unknown was eliminated', 'A team was eliminated',
    'Player 100 was eliminated', 'The report said Player 1 was eliminated', 'Ax33 Kin were eliminated']) {
    expect(invalidated(reason)).toContain(`invalidated: ${reason}`);
  }
  expect(formatEvent({ type: 'OrderFailed', player: 0, reason: 'Player 1 was eliminated' }, ctx).text)
    .toBe('Order failed: you were eliminated');
});

it.each([
  [0, 'Alex', 'Alex was eliminated'],
  [1, 'Alex', 'Alex was eliminated'],
  [0, 'Blake', 'you were eliminated'],
  [1, 'Blake', 'India was eliminated'],
] as const)('only renames a unique leading subject (viewer %i, other name %s)', (me, otherName, expected) => {
  const view = {
    me, team: me, units: [], cities: [], diplomacy: { relations: [] },
    players: [
      { id: 0, name: 'Alex', faction: 'India', team: 0 },
      { id: 1, name: otherName, faction: 'Mongols', team: 1 },
    ],
  } as unknown as PlayerView;
  const ctx = contextFromViews(null, view);
  const reason = 'Alex was eliminated';
  expect(formatEvent({ type: 'ProposalInvalidated',
    proposal: { id: 1, from: 1, to: 0, proposer: 1, kind: 'Alliance', created_turn: 9, respond_by: 11, relation_version: 0 },
    effective_turn: 11, reason }, ctx).text).toContain(`invalidated: ${expected}`);
  expect(formatEvent({ type: 'OrderFailed', player: me, reason }, ctx).text).toBe(`Order failed: ${expected}`);
});

it('says where by the nearest known city even when it is far away', () => {
  const view = {
    me: 0, team: 0, units: [], diplomacy: { relations: [] },
    players: [{ id: 0, name: 'Player 1', faction: 'India', team: 0, is_human: true, alive: true, vp: 0, vp_sources: [], cities: 1, faction_goal_done: false }],
    cities: [{ id: 5, owner: 0, name: 'Brasslock', pos: { q: 1, r: 1 } }],
  } as unknown as PlayerView;
  const ctx = contextFromViews(null, view);
  expect(formatEvent({ type: 'BarbarianCampDiscovered', camp: 0, team: 0, at: { q: 3, r: 1 } }, ctx).text).toBe('Barbarian camp sighted 2 tiles E of Brasslock');
  expect(formatEvent({ type: 'BarbarianCampSpawned', camp: 0, at: { q: 30, r: 20 } } as Event, ctx).text).toBe('A barbarian camp appeared 48 tiles SE of Brasslock');
});

it('names a failed Settler by kind and place, never by its id, and pluralizes nothing it should not', () => {
  const named: ReportContext = { ...ctx, unitKind: (id) => (id === 62 ? 'Settler' : null), unitPos: (id) => (id === 62 ? { q: 3, r: 2 } : null),
    place: (hex) => (hex.q === 3 ? '1 tile E of Brasslock' : null) };
  expect(formatEvent({ type: 'OrderFailed', player: 0, reason: 'Settler 62 could not found a city: Too close to Brasslock (1 hex; cities need at least 4)' }, named).text)
    .toBe('Your Settler 1 tile E of Brasslock could not found a city: Too close to Brasslock (1 hex; cities need at least 4)');
  // Core wording, and an id in the middle of a reason.
  expect(formatEvent({ type: 'OrderFailed', player: 0, reason: 'Settler 62: another city is being founded within 3 hexes; neither is founded.' }, named).text)
    .toBe('Your Settler 1 tile E of Brasslock: another city is being founded within 3 hexes; neither is founded.');
  expect(formatEvent({ type: 'OrderFailed', player: 0, reason: 'Ranged attack by unit 62 failed: no target' }, named).text)
    .toBe('Order failed: Ranged attack by your Settler 1 tile E of Brasslock failed: no target');
});

it('reads a city that had to stop or skip a build as city news with what happens next, never "Order failed"', () => {
  const cities: ReportContext = { ...ctx, cityNamed: (name) => (name === 'Hymnal' ? { q: 4, r: 4 } : null) };
  const stopped = formatEvent({ type: 'OrderFailed', player: 0, reason: 'Hymnal can no longer produce Settler: Settler needs population 2 (currently 1); its 38 Production stays banked' }, cities);
  expect(stopped).toMatchObject({ category: 'cities', at: { q: 4, r: 4 }, tone: 'bad',
    text: 'Hymnal can no longer build Settler: it needs 2 population and the city has 1. Its 38 Production stays banked for the Settler. Choose a new build.' });
  expect(formatEvent({ type: 'OrderFailed', player: 0, reason: 'Hymnal skipped queued Settler: Settler needs population 2 (currently 1)' }, cities).text)
    .toBe('Hymnal dropped Settler from its plan: it needs 2 population and the city has 1.');
  expect(formatEvent({ type: 'OrderFailed', player: 0, reason: 'Hymnal skipped queued Spearman: No free spawn tile in or beside the city; move a blocking unit first; kept in queue' }, cities).text)
    .toBe("Hymnal's queued Spearman is waiting: no free spawn tile in or beside the city. It stays in the queue and starts by itself once it can.");
  expect(formatEvent({ type: 'OrderFailed', player: 0, reason: 'Hymnal skipped queued Pikeman: No Iron to spare: all 6 of your Iron are in use. Bring another Iron deposit inside your borders, or build a Mine on one for +1 Iron; kept in queue' }, cities).text)
    .toBe("Hymnal's queued Pikeman is waiting: no Iron to spare: all 6 of your Iron are in use. Bring another Iron deposit inside your borders, or build a Mine on one for +1 Iron. It stays in the queue and starts by itself once it can.");
});

it('reads the supply events: found, short and restored', () => {
  const found = formatEvent({ type: 'ResourcesRevealed', player: 0, resource: 'Iron', city: 1, sources: 1, supply: 2 }, ctx);
  expect(found.text).toMatch(/^Iron found near .+: \+2 Iron supply$/);
  expect(found.icon).toBe('resource-iron');
  const here = { q: 3, r: 4 };
  const short = formatEvent({ type: 'SupplyShortage', player: 0, resource: 'Iron', idle: 2, supplied: 6, used: 8 }, { ...ctx, idleSupplyAt: () => here });
  expect(short).toMatchObject({ text: 'Short of Iron: 2 buildings or units are idle. You have 6 Iron and need 8. Bring another Iron deposit inside your borders, or build a Mine on one for +1 Iron.', tone: 'bad', category: 'economy', icon: 'resource-iron', at: here });
  expect(formatEvent({ type: 'SupplyShortage', player: 0, resource: 'Coal', idle: 1, supplied: 2, used: 3 }, ctx).text).toBe('Short of Coal: 1 building or unit is idle. You have 2 Coal and need 3. Bring another Coal deposit inside your borders, or build a Mine on one for +1 Coal.');
  expect(formatEvent({ type: 'SupplyShortage', player: 0, resource: 'Coal', idle: 1, supplied: 0, used: 1 }, ctx).text).toContain('You have no Coal and need 1. Bring a Coal deposit inside your borders');
  expect(formatEvent({ type: 'SupplyShortage', player: 0, resource: 'Iron', idle: 0, supplied: 8, used: 8 }, ctx)).toMatchObject({ text: 'Iron supply restored: every building and unit works again', tone: 'good' });
  expect(formatEvent({ type: 'SupplyShortage', player: 0, resource: 'Oil', idle: 1, supplied: 1, used: 2 }, { ...ctx, idleSupplyNames: () => ['Bomber'] }).text).toContain('Short of Oil: 1 building or unit is idle. Idle: Bomber.');
});

it('explains a Settler whose destination can no longer hold a city, from what the player knows', async () => {
  const client = new FixtureClient();
  await client.newGame(await client.standardSetup('42', 'India', 'Byzantium', 'Ada', 'Bo'));
  await client.endTurn({});
  const before: PlayerView = structuredClone(client.view(0));
  const capital = before.cities.find((c) => c.owner === before.me)!;
  const dest = { q: capital.pos.q + 5, r: capital.pos.r };
  const di = tileIndex(dest, before.width, before.height);
  before.tiles[di] = { ...before.tiles[di], terrain: 'Plains', explored: true, owner: null };
  const spearman = before.units.find((u) => u.owner === before.me && u.kind === 'Spearman')!;
  const settler = { ...spearman, kind: 'Settler' as const, military: false, pos: { q: capital.pos.q + 1, r: capital.pos.r }, goto: dest };
  before.units = before.units.map((u) => (u.id === settler.id ? settler : u));
  // A rival founds Brasslock two tiles past the destination.
  const after: PlayerView = structuredClone(before);
  after.cities = [...after.cities, { ...capital, id: 99, owner: 1, name: 'Brasslock', pos: { q: dest.q + 2, r: dest.r } }];
  const moved = { ...settler, pos: { q: capital.pos.q + 2, r: capital.pos.r } };
  after.units = after.units.map((u) => (u.id === settler.id ? moved : u));
  const [line] = settlerSiteLines(before, after, contextFromViews(before, after));
  expect(line.text).toMatch(/^Your Settler .+ is heading for a site where no city can be founded now: Too close to Brasslock: cities must be at least 3 tiles apart\.$/);
  expect(line).toMatchObject({ category: 'orders', weight: 2, at: dest });
  // Arrived: it stopped there. Nothing when the site was already bad, or is still fine.
  after.units = after.units.map((u) => (u.id === settler.id ? { ...moved, pos: dest, goto: null } : u));
  expect(settlerSiteLines(before, after, contextFromViews(before, after))[0].text).toMatch(/stopped at its destination, but no city can be founded there now: Too close to Brasslock/);
  expect(settlerSiteLines(after, after, contextFromViews(after, after))).toEqual([]);
  expect(settlerSiteLines(before, before, contextFromViews(before, before))).toEqual([]);
});

describe('upkeepText', () => {
  it('says a civilian (no upkeep field) has no upkeep', () => {
    expect(upkeepText(undefined)).toBe('No upkeep');
    expect(upkeepText(null)).toBe('No upkeep');
  });

  it('says a unit is free while free units remain, naming how many more', () => {
    expect(upkeepText({ free: true, free_slots_left: 1, rate: 1, gold_per_turn_after: 5 }))
      .toBe('Upkeep: free (1 more free unit)');
    expect(upkeepText({ free: true, free_slots_left: 2, rate: 1, gold_per_turn_after: 5 }))
      .toBe('Upkeep: free (2 more free units)');
  });

  it('shows the paid rate once free slots run out', () => {
    expect(upkeepText({ free: false, free_slots_left: 0, rate: 1, gold_per_turn_after: 4 }))
      .toBe('Upkeep: −1 Gold/turn');
  });

  it('shows Switzerland\' higher rate', () => {
    expect(upkeepText({ free: false, free_slots_left: 0, rate: 3, gold_per_turn_after: 2 }))
      .toBe('Upkeep: −3 Gold/turn');
  });
});

describe('Byzantium’s Heir to the Throne in the report', () => {
  // Players: 0 You (India), 1 Ben (your Byzantium partner), 2 Hollow Crown (rival), 3 Saint Veyra (rival).
  const faction = (id: number) => (id === 1 ? 'Byzantium' : 'India') as Faction;
  const c: ReportContext = { ...ctx, playerFaction: faction };
  const lost = (player: number): Event => ({ type: 'FactionGoalLost', player, faction: 'Byzantium', vp: 2 });
  const got = (player: number, again?: boolean): Event => ({ type: 'FactionGoalCompleted', player, faction: 'Byzantium', vp: 2, ...(again ? { again } : {}) });
  const lines = (events: Event[], context = c) => formatEvents(events, context).map((l) => [l.text, l.tone]);

  it('says in the viewer’s words who captured him, as one line per hand-over', () => {
    expect(lines([lost(2), got(0)])).toEqual([['You captured Byzantium\'s Heir from Hollow Crown (+2 VP)', 'good']]);
    expect(lines([lost(1), got(2)])).toEqual([['Hollow Crown captured Byzantium\'s Heir from Ben (−2 VP)', 'bad']]);
    expect(lines([lost(0), got(2)], { ...c, playerFaction: (id) => (id === 0 ? 'Byzantium' : 'India') })).toEqual([['Hollow Crown captured your Heir (−2 VP)', 'bad']]);
    expect(lines([lost(2), got(3)])).toEqual([['Byzantium\'s Heir changed hands: Saint Veyra now holds him', 'neutral']]);
  });

  it('words training, a retake, freeing him and losing him with nobody to take him', () => {
    expect(lines([got(1)])).toEqual([['Ben trained the Heir (+2 VP)', 'good']]);
    expect(lines([lost(2), got(1, true)])).toEqual([['Ben retook its Heir from Hollow Crown (+2 VP)', 'good']]);
    expect(lines([got(0)])).toEqual([['You freed Byzantium\'s Heir (+2 VP)', 'good']]);
    expect(lines([lost(1)])).toEqual([['Ben lost the Heir: nobody scores until he is held again (−2 VP)', 'bad']]);
  });
});

describe('legacy Sakoku losses in the report', () => {
  const lost = (player: number): Event => ({ type: 'FactionGoalLost', player, faction: 'India', vp: 2 });
  const reopened = (player: number): Event => ({ type: 'SakokuReopened', player, turns: 40 });
  const lines = (events: Event[]) => formatEvents(events, ctx).map((l) => [l.text, l.tone]);

  it('names the cause when an enemy unit breaks the vow', () => {
    expect(lines([lost(2)])[0][1]).toBe('good');
  });

  it('keeps only the reopen line when India gave the VP up by reopening', () => {
    expect(lines([lost(0), reopened(0)])).toHaveLength(1);
    expect(lines([lost(0), reopened(0)])[0][0]).toContain('You reopened the country after 40 turns closed');
  });
});

it('reports the Imperial Diwan as built or transferred with its city', () => {
  const gained: Event = { type: 'FactionGoalCompleted', player: 0, faction: 'India', vp: 2 };
  const built: Event = { type: 'ProductionCompleted', city: 1, owner: 0, item: { kind: 'Building', building: 'ImperialDiwan' } };
  expect(formatEvents([built, gained], ctx).find((line) => line.text.includes('Imperial Diwan') && line.category === 'victory')?.text).toContain('built the Imperial Diwan in');
  const captured: Event = { type: 'CityCaptured', city: 1, from: 2, to: 0 };
  const lost: Event = { type: 'FactionGoalLost', player: 2, faction: 'India', vp: 2 };
  const lines = formatEvents([captured, lost, gained], ctx).filter((line) => line.text.includes('Imperial Diwan') && line.category === 'victory');
  expect(lines).toHaveLength(1);
  expect(lines[0].text).toContain('took the Imperial Diwan in');
});

describe('the Mongols’ Yam Relay Station in the report', () => {
  // Players: 0 You (India), 1 Ben (your Mongols partner), 2 Hollow Crown (rival), 3 Saint Veyra (rival). City 1 is Tithe Station.
  const faction = (id: number) => (id === 1 ? 'Mongols' : 'India') as Faction;
  const c: ReportContext = { ...ctx, playerFaction: faction };
  const lost = (player: number): Event => ({ type: 'FactionGoalLost', player, faction: 'Mongols', vp: 2 });
  const got = (player: number, again?: boolean): Event => ({ type: 'FactionGoalCompleted', player, faction: 'Mongols', vp: 2, ...(again ? { again } : {}) });
  const taken = (from: number, to: number): Event => ({ type: 'CityCaptured', city: 1, from, to });
  const built = (owner: number): Event => ({ type: 'ProductionCompleted', city: 1, owner, item: { kind: 'Building', building: 'YamRelayStation' } });
  const goal = (events: Event[], context = c) => formatEvents(events, context).filter((l) => l.category === 'victory').map((l) => [l.text, l.tone]);

  it('words the moment the Station is built, from the viewer’s side', () => {
    expect(goal([built(0), got(0)])).toEqual([['You built the Yam Relay Station in Tithe Station: every empire may cross your land at Peace now (+2 VP)', 'good']]);
    expect(goal([built(1), got(1)])).toEqual([['Ben built the Yam Relay Station in Tithe Station: their land is open to every empire at Peace', 'good']]);
    expect(goal([got(2)])).toEqual([['Hollow Crown built the Yam Relay Station: their land is open to every empire at Peace', 'neutral']]);
  });

  it('is one line per capture: the open road and the points move together, in the viewer’s tone', () => {
    expect(goal([taken(2, 0), lost(2), got(0)])).toEqual([['You took the Yam Relay Station in Tithe Station from Hollow Crown: the open road is yours now, and every empire crosses your land at Peace (+2 VP)', 'good']]);
    expect(goal([taken(1, 2), lost(1), got(2)])).toEqual([['Hollow Crown took the Yam Relay Station in Tithe Station from Ben: the open road and the points went with the city. Retake it to get them back', 'bad']]);
    expect(goal([taken(0, 2), lost(0), got(2)])).toEqual([['Hollow Crown took the Yam Relay Station in Tithe Station from you: the open road and the points went with the city. Retake it to get them back (−2 VP)', 'bad']]);
    expect(goal([taken(2, 3), lost(2), got(3)])).toEqual([['Saint Veyra took the Yam Relay Station in Tithe Station: every empire may now cross Saint Veyra\'s land at Peace', 'neutral']]);
    expect(goal([taken(255, 0), got(0, true)])[0][0]).toBe('You took the Yam Relay Station back from the barbarians in Tithe Station (+2 VP)');
  });

  it('says a razed Hall is gone and can be built again; with no raze, nobody scores until the city is retaken', () => {
    const razed = (from: number): Event => ({ type: 'CityRazed', city: 1, from, by: 3, at: { q: 2, r: 2 } });
    expect(goal([razed(0), lost(0)])).toEqual([['Your Yam Relay Station is gone: its city was sacked, and the open road with it. It can be built again (−2 VP)', 'bad']]);
    expect(goal([razed(2), lost(2)])[0][1]).toBe('neutral');
    expect(goal([lost(1)])).toEqual([['Ben lost the Yam Relay Station: nobody scores until its city is retaken (−2 VP)', 'bad']]);
  });

  it('reports a Celtic sack and the Gold carried off', () => {
    const event: Event = { type: 'CityRazed', city: 1, from: 2, by: 0, at: { q: 2, r: 2 }, sack_gold: 175 };
    expect(formatEvent(event, ctx).text).toContain("sacked Hollow Crown's city at this tile and carried off 175 Gold");
  });

  it('reports an ordered raze and its private Gold without calling it a Celtic sack', () => {
    const event: Event = { type: 'CityRazed', city: 1, from: 2, by: 0, at: { q: 2, r: 2 }, sack_gold: 30, ordered: true };
    expect(formatEvent(event, ctx).text).toBe('You razed at this tile and gained 30 Gold');
  });

  it('adds the warband camp to a sack line, only when present, and names no technology', () => {
    const event: Event = { type: 'CityRazed', city: 1, from: 2, by: 0, at: { q: 2, r: 2 }, sack_gold: 180, camp: { q: 3, r: 2 } };
    expect(formatEvent(event, ctx).text).toBe("You sacked Hollow Crown's city at this tile and carried off 180 Gold and raised a warband camp near the ruins");
    const victim: Event = { type: 'CityRazed', city: 1, from: 0, by: 2, at: { q: 2, r: 2 }, sack_gold: 180 };
    expect(formatEvent(victim, ctx).details.join(' ')).not.toMatch(/technolog/);
    const rival: Event = { type: 'CityRazed', city: 1, from: 2, by: 1, at: { q: 2, r: 2 }, camp: { q: 3, r: 2 } };
    expect(formatEvent(rival, ctx).text).toBe("Ben sacked Hollow Crown's city at this tile and raised a warband camp near the ruins");
    const bare: Event = { type: 'CityRazed', city: 1, from: 2, by: 0, at: { q: 2, r: 2 } };
    expect(formatEvent(bare, ctx).text).toBe("You sacked Hollow Crown's city at this tile");
  });
});

describe('objectiveCloseLines', () => {
  it('names an Age card the turn you come within one of it, as a good, notable report line', async () => {
    const client = new FixtureClient();
    await client.newGame(await client.standardSetup('42', 'India', 'Byzantium', 'Ada', 'Bo'));
    const base = structuredClone(client.view(0));
    const before = withV2(base, { era: 2, scored: [], progress: { '2:0': [6, 9], '2:1': [2, 4], '2:2': [0, 1] } });
    const after = withV2(base, { era: 2, scored: [], progress: { '2:0': [6, 9], '2:1': [3, 4], '2:2': [0, 1] } });
    const lines = objectiveCloseLines(before, after);
    expect(lines).toHaveLength(1);
    expect(lines[0]).toMatchObject({ category: 'victory', tone: 'good', weight: 2, players: [after.me] });
    expect(lines[0].text).toBe('Close to the Classical Age’s VP: Dominion, gain 4 cities (3 of 4, 1 to go)');
    expect(lines[0].details[0]).toContain('Meet the Classical Age’s card to score its 1 VP');
    // The next turn it is still one away: the line does not repeat.
    expect(objectiveCloseLines(after, after)).toEqual([]);
  });
});

it('says a full military-unit cap in words, not a fraction', () => {
  expect(plainOrderError('Astra cannot produce Spearman: Levies full: 3 of 2')).toBe("Astra can't train a Spearman: you have 3 military units and room for 2.");
});
