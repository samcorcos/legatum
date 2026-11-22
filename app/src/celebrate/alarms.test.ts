import { beforeAll, describe, expect, it } from 'vitest';
import { FixtureClient } from '../test/fixtureEngine';
import type { Event, PlayerView, Relation } from '../types';
import { alarmFocus, alarmHeadline, alarmsForTurn, holdAlarm, holdSubline, threatFrom } from './alarms';
import { holdFacts, nearFacts } from './HoldDetail';
import { formatEvent, contextFromViews } from '../format';
import { feedbackForTurn } from '../feel/turnFeedback';

// Ada (0) and Bo (1) are team 0; the AI empires 2..7 are teams 1..6.
let view: PlayerView;
let prev: PlayerView;
beforeAll(async () => {
  const client = new FixtureClient();
  await client.newGame(await client.standardSetup('42', 'India', 'Byzantium', 'Ada', 'Bo'));
  prev = client.view(0);
  client.endTurn({ 0: [], 1: [] });
  view = client.view(0);
  expect(view.turn).toBe(prev.turn + 1);
});

const relation = (a: number, b: number, declarers: number[], effective: number, state: Relation['state'] = 'War'): Relation =>
  ({ a, b, state, security: null, effective_turn: effective, guarantee_until: 0, version: 1, war_id: 1, declarers });
const war = (a: number, b: number, declarers: number[], effective = view.turn - 1): Event => ({ type: 'WarDeclared', relation: relation(a, b, declarers, effective) });
/** An Alliance carried `team` into `ally`'s war on `enemy`. */
const joined = (team: number, enemy: number, ally: number, declarers: number[]): Event => ({ type: 'WarJoined', team, ally, relation: relation(Math.min(team, enemy), Math.max(team, enemy), declarers, view.turn - 1) });
const betrayed = (by: number, against: number): Event => ({ type: 'AllianceBetrayed', betrayal: { by, against, turn: view.turn - 1, until: view.turn + 19 } });
const named = (team: number) => view.players.find((p) => p.team === team)!;

describe('alarmsForTurn', () => {
  it('raises a war moment for either side of a declaration involving you', () => {
    const [alarm] = alarmsForTurn([war(0, 1, [1])], prev, view);
    expect(alarm).toMatchObject({ kind: 'war', team: 1, surprise: true, events: [0] });
    expect(alarm.line.tone).toBe('bad');
    expect(alarmsForTurn([war(0, 1, [0])], prev, view)).toMatchObject([{ kind: 'warByYou', team: 1 }]);
    expect(alarmsForTurn([war(1, 2, [1])], prev, view)).toEqual([]); // Between others.
  });

  it('a broken Alliance and new War each have a moment; the war starts the turn being planned', () => {
    const events: Event[] = [
      { type: 'TreatyBreached', treaty_id: 4, by: 1, with: 0, player: 2, cost: 120, effective_turn: view.turn - 1 },
      betrayed(1, 0),
      war(0, 1, [1], view.turn),
    ];
    const alarms = alarmsForTurn(events, prev, view);
    expect(alarms).toHaveLength(2);
    expect(alarms[0]).toMatchObject({ kind: 'war', surprise: false, effectiveTurn: view.turn, events: [2] });
    expect(alarms[1].broke?.text).toContain('120');
    // Leaving an Alliance early without war is its own, lighter alarm, one per betrayer.
    const left = alarmsForTurn(events.slice(0, 2), prev, view);
    expect(left).toMatchObject([{ kind: 'breach', team: 1, events: [0, 1] }]);
    expect(alarmHeadline(view, left)).toMatch(/ break their Alliance with you$/);
  });

  it('war on an ally after its protected window (a betrayal with no payment) still names the broken Alliance', () => {
    const alarms = alarmsForTurn([betrayed(1, 0), war(0, 1, [1], view.turn)], prev, view);
    expect(alarms).toHaveLength(2);
    expect(alarms[0]).toMatchObject({ kind: 'war', team: 1, events: [1] });
    expect(alarms[1].broke).toBeDefined();
    // Someone else's betrayal is not yours to be alarmed by.
    expect(alarmsForTurn([betrayed(1, 2)], prev, view)).toEqual([]);
  });

  it('every declaration on you shares one screen', () => {
    const alarms = alarmsForTurn([war(0, 1, [1]), war(0, 2, [2])], prev, view);
    expect(alarms.map((a) => [a.kind, a.team])).toEqual([['war', 1], ['war', 2]]);
    expect(alarmHeadline(view, alarms)).toMatch(/^The \w[\w ]* and the \w[\w ]* declare war on you$/);
    expect(alarmsForTurn([war(0, 1, [1]), war(0, 2, [2]), war(0, 3, [3])], prev, view).length).toBe(3);
    expect(alarmHeadline(view, alarmsForTurn([war(0, 1, [1]), war(0, 2, [2]), war(0, 3, [3])], prev, view))).toBe('3 empires declare war on you');
  });

  it('headline: the faction as a people, always "declare"', () => {
    const faction = ({ Switzerland: 'Swiss', India: 'Indians', Byzantium: 'Byzantines', Germany: 'Germans', Celts: 'Celts', Mongols: 'Mongols', Sparta: 'Spartans', Athens: 'Athenians', Aztecs: 'Aztecs', Egypt: 'Egyptians', Rome: 'Romans' } as const)[named(1).faction];
    expect(alarmHeadline(view, alarmsForTurn([war(0, 1, [1])], prev, view))).toBe(`The ${faction} declare war on you`);
  });

  it('allies defend each other: your ally is attacked and you are in it; the attacker\'s ally joins a war on you', () => {
    const allied = structuredClone(view);
    allied.diplomacy!.relations = allied.diplomacy!.relations.map((r) => (r.a === 0 && r.b === 2) || (r.a === 2 && r.b === 0) ? { ...r, state: 'Alliance' } : r);
    // Team 1 attacked team 2; the Alliance carried us in. Only the join is yours to notice.
    const together = alarmsForTurn([war(1, 2, [1]), joined(0, 1, 2, [1])], allied, allied);
    expect(together).toMatchObject([{ kind: 'allyWar', team: 2, enemy: 1, events: [1] }]);
    expect(alarmHeadline(allied, together)).toMatch(/ attacked .+, your ally, and you are at War with them too$/);
    // Joining an ally's war by answering its call to arms is your own choice: no alarm.
    expect(alarmsForTurn([war(1, 2, [2]), joined(0, 1, 2, [0])], allied, allied)).toEqual([]);
    // Team 1 declared on us; its ally, team 3, joined.
    const against = alarmsForTurn([war(0, 1, [1]), joined(3, 0, 1, [1])], prev, view);
    expect(against).toMatchObject([{ kind: 'war', team: 1, surprise: true }, { kind: 'war', team: 3, joined: true }]);
    expect(alarmHeadline(view, against)).toMatch(/ declare war on you, and .+ join them$/);
    expect(alarmHeadline(view, [against[1]])).toMatch(/ join the war against you$/);
    const newlyAllied: Event = { type: 'TreatyActivated', relation: relation(1, 3, [], view.turn - 1, 'Alliance') };
    const alliedWar = alarmsForTurn([newlyAllied, joined(3, 0, 1, [1])], prev, view);
    expect(alliedWar[0].line.text).toMatch(/allied with .* and joined its war against you/);
    expect(alarmHeadline(view, alliedWar)).toBe(alliedWar[0].line.text.replace(/ · started turn \d+$/, ''));
    const city = structuredClone(view);
    city.cities.push({ ...structuredClone(city.cities[0] ?? { id: 0, owner: 1, name: 'Home', pos: { q: 3, r: 3 } } as never), id: 77, owner: 1, name: 'Brasslock', pos: { q: 3, r: 3 } });
    const lost = alarmsForTurn([{ type: 'CityCaptured', city: 77, from: 1, to: 2 }], city, city);
    expect(lost).toMatchObject([{ kind: 'cityLost', team: 1, city: { name: 'Brasslock', owner: 1 } }]);
    expect(alarmHeadline(city, lost)).toBe('Brasslock has fallen');
    expect(alarmFocus(city, lost)).toEqual({ q: 3, r: 3 });
  });

  it('nothing for a finished game or an eliminated viewer', () => {
    expect(alarmsForTurn([war(0, 1, [1])], prev, { ...view, finished: true })).toEqual([]);
  });

  it('queues peace, Alliance changes, and call answers by importance', () => {
    const peace: Event = { type: 'PeaceEstablished', relation: relation(0, 1, [], view.turn, 'Peace') };
    const alliance: Event = { type: 'TreatyActivated', relation: relation(0, 2, [], view.turn, 'Alliance') };
    const ended: Event = { type: 'TreatyEnded', treaty_id: 9, a: 0, b: 2, effective_turn: view.turn, reason: 'Alliance ended' };
    const call: Event = { type: 'CallAnswered', joined: true, effective_turn: view.turn,
      call: { id: 7, team: 2, ally: 0, target: 1, created_turn: view.turn - 1, respond_by: view.turn, alliance_version: 1, war_version: 1 } };
    const alarms = alarmsForTurn([peace, alliance, ended, call], prev, view);
    expect(alarms.map((a) => [a.kind, a.team])).toEqual([
      ['callJoined', 2], ['allianceEnded', 2], ['peace', 1], ['alliance', 2],
    ]);
    expect(alarmHeadline(view, [alarms[2]])).toContain('Peace');
    expect(alarmsForTurn([{ ...call, joined: false }], prev, view)[0].kind).toBe('callDeclined');
  });
});

describe('threatFrom', () => {
  it('lists their units in sight, fighting units before Scouts, and where they are', () => {
    const v = structuredClone(view);
    v.players.find((p) => p.id === 2)!.faction = 'Byzantium';
    const mine = v.units.find((u) => u.owner === 0)!;
    const base = { ...mine, owner: 2, military: true };
    v.cities = [];
    v.units = [mine,
      { ...base, id: 900, kind: 'Scout', pos: { q: mine.pos.q + 1, r: mine.pos.r } },
      { ...base, id: 901, kind: 'Pikeman', pos: { q: mine.pos.q + 3, r: mine.pos.r } },
      { ...base, id: 902, kind: 'Pikeman', pos: { q: mine.pos.q + 4, r: mine.pos.r } }];
    const t = threatFrom(v, 1);
    expect(t.summary).toBe('2 Tagmatic Soldiers, 1 Scout');
    expect(t.nearest?.at).toEqual({ q: mine.pos.q + 3, r: mine.pos.r });
    expect(t.nearest?.text).toMatch(/^Nearest: Tagmatic Soldier, 3 tiles E of your /);
    expect(t.city).toBeNull();
  });
});

describe('turn feedback', () => {
  it('plays warDeclared for a war declared on your team, not one you declared', () => {
    expect(feedbackForTurn([war(0, 1, [1])], view)).toContain('warDeclared');
    expect(feedbackForTurn([war(0, 1, [0])], view)).not.toContain('warDeclared');
    expect(feedbackForTurn([war(1, 2, [1])], view)).not.toContain('warDeclared');
  });
});

describe('victory hold alarm', () => {
  const started = (team: number, target = 10, needed = 10): Event => ({ type: 'VictoryHoldStarted', team, vp: target, target, turns_needed: needed });
  const withHold = (team: number, held = 1, needed = 10): PlayerView => {
    const v = structuredClone(view);
    v.standings = v.standings.map((s) => s.team === team ? { ...s, vp: s.target, victory_hold: { turns_held: held, turns_needed: needed } } : s);
    return v;
  };
  const faction = (team: number) => {
    const f = named(team).faction;
    return ({ Switzerland: 'Swiss', India: 'Indians', Byzantium: 'Byzantines', Germany: 'Germans', Celts: 'Celts', Mongols: 'Mongols', Sparta: 'Spartans', Athens: 'Athenians', Aztecs: 'Aztecs', Egypt: 'Egyptians', Rome: 'Romans' } as const)[f];
  };

  it('every viewer gets one: yours, your ally\'s and a rival\'s, each worded from their side', () => {
    const mine = alarmsForTurn([started(0)], prev, withHold(0));
    expect(mine).toMatchObject([{ kind: 'victoryHold', team: 0, hold: { stance: 'self', held: 1, needed: 10, target: 10 } }]);
    expect(alarmHeadline(view, mine)).toMatch(/^Your team has 10 VP$|^You have 10 VP$/);
    expect(holdSubline(mine[0])).toBe('Hold it for 10 turns to win; every rival will now come for you.');

    const rival = alarmsForTurn([started(1)], prev, withHold(1));
    expect(rival[0].hold!.stance).toBe('rival');
    expect(alarmHeadline(view, rival)).toBe(`The ${faction(1)} have 10 VP`);
    expect(holdSubline(rival[0])).toBe('They must hold it for 10 turns unless someone stops them.');

    const allied = withHold(2);
    allied.diplomacy!.relations = allied.diplomacy!.relations.map((r) => (r.a === 0 && r.b === 2) || (r.a === 2 && r.b === 0) ? { ...r, state: 'Alliance' } : r);
    const ally = alarmsForTurn([started(2)], allied, allied);
    expect(ally[0].hold!.stance).toBe('ally');
    expect(alarmHeadline(allied, ally)).toBe(`The ${faction(2)}, your allies, have 10 VP`);
    expect(holdSubline(ally[0])).toContain('an Alliance does not share victory');
  });

  it('a war declared on you still leads, and the hold waits on the same screen; the counts come from the event', () => {
    const alarms = alarmsForTurn([started(1, 10, 5), war(0, 2, [2])], prev, withHold(1, 1, 5));
    expect(alarms.map((a) => a.kind)).toEqual(['war', 'victoryHold']);
    expect(alarms[1].hold).toMatchObject({ needed: 5, held: 1 });
    expect(holdSubline(alarms[1])).toBe('They must hold it for 5 turns unless someone stops them.');
  });

  it('two teams reaching their targets together share one headline; an eliminated team raises none', () => {
    const both = alarmsForTurn([started(1), started(2)], prev, view);
    expect(both.map((a) => a.team)).toEqual([1, 2]);
    expect(alarmHeadline(view, both)).toBe('2 empires have reached their VP target');
    const dead = structuredClone(view);
    dead.players = dead.players.map((p) => p.team === 1 ? { ...p, alive: false } : p);
    expect(alarmsForTurn([started(1)], prev, dead)).toEqual([]);
  });

  it('the report line reopens the same card from its event numbers, and a broken hold says so', () => {
    const line = formatEvent(started(1), contextFromViews(prev, view));
    expect(line.reopenHold).toEqual({ team: 1, vp: 10, target: 10, turnsNeeded: 10 });
    const live = holdAlarm(withHold(1, 4), line.reopenHold!, line);
    expect(live.hold).toMatchObject({ stance: 'rival', held: 4 });
    const ended = holdAlarm(view, line.reopenHold!, line);
    expect(ended.hold!.held).toBe(0);
    expect(holdFacts(ended)[0].text).toBe('This hold has ended: their VP fell below 10.');
  });

  it('facts state the rule, never a route: who wins at what count, what adds, what resets', () => {
    const [rival] = alarmsForTurn([started(1)], prev, withHold(1));
    const facts = holdFacts(rival);
    expect(facts.map((f) => f.text)).toEqual([
      'They win the game when the count reaches 10 turns.',
      'Each turn end at or above 10 VP adds one turn to the count. The turn the target was reached is the first.',
      'Below 10 VP at any turn end, the count goes back to zero.',
    ]);
    expect(facts[0].tone).toBe('bad');
    const [mine] = alarmsForTurn([started(0)], prev, withHold(0));
    expect(holdFacts(mine)[0]).toMatchObject({ tone: 'good', text: 'You win the game when the count reaches 10 turns.' });
  });
});

describe('victory hold feedback', () => {
  it('your own hold is a triumph; a rival\'s is an alarm; a broken one hurts its team and allies', () => {
    expect(feedbackForTurn([{ type: 'VictoryHoldStarted', team: 0, vp: 10, target: 10, turns_needed: 10 }], view)).toContain('greatWork');
    expect(feedbackForTurn([{ type: 'VictoryHoldStarted', team: 1, vp: 10, target: 10, turns_needed: 10 }], view)).toContain('warDeclared');
    expect(feedbackForTurn([{ type: 'VictoryHoldBroken', team: 0, turns_held: 3, vp: 9, target: 10 }], view)).toContain('vpLost');
    expect(feedbackForTurn([{ type: 'VictoryHoldBroken', team: 1, turns_held: 3, vp: 9, target: 10 }], view)).toContain('vpGained');
  });
});

describe('near-victory alarm', () => {
  const at = (team: number, vp: number, base: PlayerView = view): PlayerView => {
    const v = structuredClone(base);
    v.standings = v.standings.map((s) => s.team === team ? { ...s, vp, target: 10, victory_hold: vp >= 10 ? { turns_held: 1, turns_needed: 10 } : null } : s);
    return v;
  };
  const near = (before: PlayerView | null, after: PlayerView, events: Event[] = []) => alarmsForTurn(events, before, after).filter((a) => a.kind === 'victoryNear');

  it('fires once when a team crosses to within two VP, for a rival, an ally and your own team', () => {
    const rival = near(at(1, 7), at(1, 8));
    expect(rival).toHaveLength(1);
    expect(rival[0]).toMatchObject({ team: 1, near: { stance: 'rival', vp: 8, target: 10 } });
    expect(rival[0].line).toMatchObject({ tone: 'bad', icon: 'vp', reopenHold: { team: 1, vp: 8, target: 10, near: true } });
    expect(alarmHeadline(view, rival)).toMatch(/ 2 VP from victory: 8 of 10$/);
    expect(near(at(0, 6), at(0, 9))[0]).toMatchObject({ near: { stance: 'self', vp: 9 } });
    expect(near(at(0, 6), at(0, 9))[0].line.tone).toBe('good');
    const allied = at(2, 8);
    allied.diplomacy!.relations = allied.diplomacy!.relations.map((r) => (r.a === 0 && r.b === 2) || (r.a === 2 && r.b === 0) ? { ...r, state: 'Alliance' } : r);
    expect(near(at(2, 7), allied)[0].near!.stance).toBe('ally');
  });

  it('does not repeat while it stays at 8 or 9, and fires again after dropping below and coming back', () => {
    expect(near(at(1, 8), at(1, 9))).toEqual([]);
    expect(near(at(1, 8), at(1, 8))).toEqual([]);
    expect(near(at(1, 8), at(1, 6))).toEqual([]);
    expect(near(at(1, 6), at(1, 8))).toHaveLength(1);
  });

  it('is suppressed when the hold starts the same turn, and with no before view', () => {
    const hold = at(1, 10);
    const started: Event = { type: 'VictoryHoldStarted', team: 1, vp: 10, target: 10, turns_needed: 10 };
    const alarms = alarmsForTurn([started], at(1, 7), hold);
    expect(alarms.map((a) => a.kind)).toEqual(['victoryHold']);
    expect(near(null, at(1, 8))).toEqual([]);
  });

  it('ranks just below the hold, names what scored, and states the rule', () => {
    const after = at(1, 8);
    const hold = at(2, 10, after);
    const scored: Event = { type: 'EraObjectiveScored', player: named(1).id, era: 2, slot: 1, objective: 'Gain 4 cities' };
    const alarms = alarmsForTurn([{ type: 'VictoryHoldStarted', team: 2, vp: 10, target: 10, turns_needed: 10 }, scored], at(2, 9, at(1, 7)), hold);
    expect(alarms.map((a) => a.kind)).toEqual(['victoryHold', 'victoryNear']);
    const facts = nearFacts(alarms[1], hold).map((f) => f.text);
    expect(facts.some((t) => /Gain 4 cities/.test(t))).toBe(true);
    expect(facts.at(-1)).toBe('World wonders and some faction goals score for whoever holds them. Capture the city that holds one to take its VP.');
  });
});
