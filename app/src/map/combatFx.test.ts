import { describe, expect, it } from 'vitest';
import { buildCombatPlan, cityShotKind, fxUnits, hpAt, HP_DRAIN_MS, lungeAmount, shotKindFor, stepPos, world } from './combatFx';
import { aircraftPassProgress } from './combatFxDraw';
import { cuesAt, battleCues } from './cues';
import type { Event, Hex, PlayerView } from '../types';

const view = (units: { id: number; owner: number; kind: string; pos: Hex }[], cities: { id: number; owner: number; pos: Hex }[] = []) => ({
  me: 0, team: 0, width: 40, height: 30,
  players: [{ id: 0, team: 0, era: 3 }, { id: 2, team: 1, era: 1 }],
  units: units.map((u) => ({ hp: 100, military: true, ranged: 0, ...u })),
  cities: cities.map((c) => ({ hp: 100, max_hp: 100, walls: true, name: 'Lugano', ...c })),
  aircraft: [],
}) as unknown as PlayerView;

const battle = (over: Partial<Extract<Event, { type: 'Battle' }>>) => ({
  type: 'Battle', attacker: 7, attacker_owner: 0, defender: { kind: 'Unit', unit: 9, owner: 2, unit_kind: 'Spearman' }, at: { q: 5, r: 3 },
  attacker_at: { q: 3, r: 3 }, damage_to_attacker: 0, damage_to_defender: 30, ranged: true, meeting: false, ...over,
}) as Event;
const moved = (unit: number, from: Hex, to: Hex, owner = 0) => ({ type: 'UnitMoved', unit, owner, from, to }) as Event;

const before = view([
  { id: 7, owner: 0, kind: 'Archer', pos: { q: 3, r: 3 } },
  { id: 8, owner: 0, kind: 'Spearman', pos: { q: 3, r: 4 } },
  { id: 9, owner: 2, kind: 'Spearman', pos: { q: 5, r: 3 } },
], [{ id: 1, owner: 2, pos: { q: 6, r: 4 } }]);

describe('combat plan', () => {
  it('is null for a turn with no fighting, so plain moves keep the glide', () => {
    expect(buildCombatPlan([moved(8, { q: 3, r: 4 }, { q: 4, r: 4 })], before, before)).toBeNull();
  });

  it('fires the ranged phase first, then moves the units round by round', () => {
    const events = [
      battle({}),
      battle({ attacker: { city: 1 }, attacker_owner: 2, attacker_at: { q: 6, r: 4 }, defender: { kind: 'Unit', unit: 8, owner: 0, unit_kind: 'Spearman' }, at: { q: 3, r: 4 }, damage_to_defender: 12 }),
      moved(8, { q: 3, r: 4 }, { q: 4, r: 4 }),
      moved(8, { q: 4, r: 4 }, { q: 5, r: 4 }),
    ];
    const plan = buildCombatPlan(events, before, before)!;
    expect(plan.shots.map((s) => [s.kind, s.city])).toEqual([['arrows', false], ['arrows', true]]);
    // The volley ends before anyone moves.
    const lastImpact = Math.max(...plan.shots.map((s) => s.launch + s.flight));
    expect(plan.volley).toBeGreaterThan(lastImpact);
    // Two steps in consecutive rounds are one glide spanning both rounds.
    const glides = plan.moves.get(8)!;
    expect(glides).toHaveLength(1);
    expect(glides[0].t0).toBe(plan.volley);
    expect(glides[0].t1).toBeCloseTo(plan.volley + plan.stepMs * 1.94);
    expect(glides[0].path).toHaveLength(3);
    // Each hex's damage cue waits for its own blow.
    expect(plan.cueDelay.get('5,3')).toBe(plan.shots[0].launch + plan.shots[0].flight);
    expect(plan.cueDelay.get('3,4')).toBe(plan.shots[1].launch + plan.shots[1].flight);
  });

  it('lands a melee clash mid-step of its round, and the attacker that did not step in lunges and comes back', () => {
    const events = [
      moved(8, { q: 3, r: 4 }, { q: 4, r: 4 }),
      moved(8, { q: 4, r: 4 }, { q: 4, r: 3 }),
      battle({ attacker: 8, ranged: false, attacker_at: { q: 4, r: 3 }, damage_to_attacker: 10 }),
    ];
    const plan = buildCombatPlan(events, before, before)!;
    expect(plan.volley).toBe(0);
    const c = plan.clashes[0];
    expect(c.t).toBe(plan.stepMs * 1.5);
    expect(c.lunge).toBe(true);
    const at = fxUnits(plan, c.t).shift.get(8)!;
    const dir = { x: world({ q: 5, r: 3 }).x - world({ q: 4, r: 3 }).x, y: world({ q: 5, r: 3 }).y - world({ q: 4, r: 3 }).y };
    expect(at.x * dir.x + at.y * dir.y).toBeGreaterThan(0);
    expect(fxUnits(plan, c.t + plan.stepMs * 2).shift.get(8)).toBeUndefined();
    // The defender flinches on contact.
    expect(fxUnits(plan, c.t + 30).shift.get(9)).toBeDefined();
  });

  it('a killer that steps into the tile charges with its own step, no lunge', () => {
    const events = [
      battle({ attacker: 8, ranged: false, attacker_at: { q: 4, r: 3 } }),
      { type: 'UnitDestroyed', unit: 9, owner: 2, unit_kind: 'Spearman', at: { q: 5, r: 3 } } as Event,
      moved(8, { q: 4, r: 3 }, { q: 5, r: 3 }),
    ];
    const plan = buildCombatPlan(events, before, before)!;
    expect(plan.clashes[0].lunge).toBe(false);
    // The fallen unit stands until the blow, then sinks and fades out.
    const d = plan.deaths[0];
    expect(d.t).toBeGreaterThanOrEqual(plan.clashes[0].t);
    expect(fxUnits(plan, d.t - 10).ghosts[0]).toMatchObject({ alpha: 1, sink: 0 });
    expect(fxUnits(plan, d.t + 350).ghosts[0].alpha).toBeLessThan(1);
    expect(fxUnits(plan, d.t + 2000).ghosts).toEqual([]);
    expect(plan.cueDelay.get('5,3')).toBe(plan.clashes[0].t);
  });

  it('takes a city when the taker arrives, and ends after the last effect', () => {
    const events = [
      battle({ attacker: 7, defender: { kind: 'City', city: 1, owner: 2 }, at: { q: 6, r: 4 } }),
      moved(8, { q: 3, r: 4 }, { q: 4, r: 4 }),
      moved(8, { q: 4, r: 4 }, { q: 5, r: 4 }),
      moved(8, { q: 5, r: 4 }, { q: 6, r: 4 }),
      { type: 'CityCaptured', city: 1, from: 2, to: 0 } as Event,
    ];
    const plan = buildCombatPlan(events, before, before)!;
    expect(plan.captures).toHaveLength(1);
    const arrive = plan.moves.get(8)![0].t1;
    expect(plan.captures[0].t).toBe(arrive);
    expect(plan.total).toBeGreaterThan(plan.captures[0].t + 1000);
    expect(plan.shots[0].atCity).toBe(true);
  });

  it('marks an ordered raze for the map label', () => {
    const event = { type: 'CityRazed', city: 1, from: 2, by: 0, at: { q: 6, r: 4 }, ordered: true } as Event;
    const plan = buildCombatPlan([event], before, before)!;
    expect(plan.captures[0]).toMatchObject({ city: 1, razed: true, orderedRaze: true });
  });

  it('drains each HP bar as its blow lands, not before', () => {
    const plan = buildCombatPlan([battle({ defender_hp_after: 70 }), battle({ attacker: 8, attacker_at: { q: 3, r: 4 }, defender_hp_after: 45 })], before, before)!;
    const [a, b] = plan.shots.map((s) => s.launch + s.flight);
    expect(hpAt(plan, 'u9', 0)).toBe(100);
    expect(hpAt(plan, 'u9', a)).toBe(100);
    const mid = hpAt(plan, 'u9', a + HP_DRAIN_MS / 2)!;
    expect(mid).toBeLessThan(100);
    expect(mid).toBeGreaterThan(70);
    expect(hpAt(plan, 'u9', b + HP_DRAIN_MS)).toBe(45);
    expect(hpAt(plan, 'u8', b)).toBeUndefined();
  });

  it('is deterministic', () => {
    const events = [battle({}), battle({ attacker: 8, attacker_at: { q: 3, r: 4 } })];
    expect(JSON.stringify([...buildCombatPlan(events, before, before)!.shots])).toBe(JSON.stringify([...buildCombatPlan(events, before, before)!.shots]));
  });
});

describe('an air strike', () => {
  it.each(['Fighter', 'Bomber', 'StealthFighter', 'StealthBomber'])('carries its %s into the plan and crosses the map before it releases', (kind) => {
    const air = { ...before, aircraft: [{ id: 40, owner: 0, kind, city: 1, hp: 100 }] } as unknown as PlayerView;
    const plan = buildCombatPlan([battle({ attacker: 40, attacker_at: undefined })], air, air)!;
    const shot = plan.shots[0];
    expect([shot.kind, shot.craft]).toEqual(['bombs', kind]);
    expect(shot.launch).toBeGreaterThanOrEqual(shot.flight * 1.3);
    expect(aircraftPassProgress(shot, shot.launch)).toBeCloseTo(0.5);
    expect(aircraftPassProgress(shot, shot.launch - shot.flight * 1.3 - 1)).toBeNull();
    expect(aircraftPassProgress(shot, shot.launch + shot.flight * 1.3 + 1)).toBeNull();
  });
});

describe('what flies', () => {
  it('scales by the shooter and its era', () => {
    expect(shotKindFor('Archer', 0, null, false)).toBe('arrows');
    expect(shotKindFor('Crossbowman', 2, null, false)).toBe('bolts');
    expect(shotKindFor('Arquebusier', 3, null, false)).toBe('shot');
    expect(shotKindFor('FieldCannon', 4, null, false)).toBe('ball');
    expect(shotKindFor('Artillery', 5, null, false)).toBe('shell');
    expect(shotKindFor('Bomber', 5, null, false)).toBe('bombs');
    // A siege crew throws stones at a city, or shot from a bombard.
    expect(shotKindFor('Archer', 0, 'catapult', true)).toBe('stone');
    expect(shotKindFor('Crossbowman', 2, 'trebuchet', true)).toBe('stone');
    expect(shotKindFor('Arquebusier', 3, 'bombard', true)).toBe('ball');
    expect(shotKindFor('Archer', 0, 'catapult', false)).toBe('arrows');
  });

  it("a city's walls answer by its owner's era", () => {
    expect([0, 1, 2, 3, 4, 6].map(cityShotKind)).toEqual(['arrows', 'arrows', 'bolts', 'ball', 'shell', 'shell']);
  });
});

it('a step glides with easing and holds its ends', () => {
  const steps = [{ t0: 100, t1: 200, path: [{ x: 0, y: 0 }, { x: 10, y: 0 }], from: { x: 0, y: 0 }, to: { x: 10, y: 0 } }];
  expect(stepPos(steps, 0)).toEqual({ x: 0, y: 0 });
  expect(stepPos(steps, 150).x).toBeCloseTo(5);
  expect(stepPos(steps, 300)).toEqual({ x: 10, y: 0 });
});

it('a multi-hex move with nothing in its way is one smooth glide at an even pace', () => {
  const events = [
    moved(8, { q: 3, r: 4 }, { q: 4, r: 4 }),
    moved(8, { q: 4, r: 4 }, { q: 5, r: 4 }),
    moved(8, { q: 5, r: 4 }, { q: 6, r: 4 }),
    battle({ attacker: 8, ranged: false, attacker_at: { q: 6, r: 4 }, at: { q: 7, r: 4 } }),
  ];
  const plan = buildCombatPlan(events, before, before)!;
  const glides = plan.moves.get(8)!;
  expect(glides).toHaveLength(1);
  expect(glides[0].path).toHaveLength(4);
  // Mid-glide it keeps moving (no stop at each hex): equal time slices cover about equal ground.
  const g = glides[0], span = g.t1 - g.t0;
  const x = (k: number) => stepPos(glides, g.t0 + span * k).x;
  expect(x(0.5) - x(0.4)).toBeCloseTo(x(0.6) - x(0.5), 0);
  // It still arrives on time for its round, and the clash lands in the round after.
  expect(stepPos(glides, g.t1)).toEqual(world({ q: 6, r: 4 }));
  expect(plan.clashes[0].t).toBeGreaterThan(g.t1 - plan.stepMs);
});

it('a unit that waits a round starts a new glide', () => {
  const events = [
    moved(8, { q: 3, r: 4 }, { q: 4, r: 4 }),
    moved(7, { q: 3, r: 3 }, { q: 4, r: 3 }),
    moved(7, { q: 4, r: 3 }, { q: 5, r: 3 }),
    moved(7, { q: 5, r: 3 }, { q: 6, r: 3 }),
    moved(8, { q: 4, r: 4 }, { q: 5, r: 4 }),
  ];
  const plan = buildCombatPlan([...events, battle({ attacker: 7, ranged: false })], before, before)!;
  expect(plan.moves.get(7)).toHaveLength(1);
  expect(plan.moves.get(8)).toHaveLength(2);
});

it('a lunge reaches out at contact and returns to rest', () => {
  expect(lungeAmount(-1000, 300, false)).toBe(0);
  expect(lungeAmount(0, 300, false)).toBeCloseTo(0.4);
  expect(lungeAmount(0, 300, true)).toBeCloseTo(0.5);
  expect(lungeAmount(1000, 300, false)).toBe(0);
});

it('cues wait for their blow and pop in', () => {
  const cues = battleCues([battle({ ranged: false })], before);
  const delay = () => 800;
  expect(cuesAt(cues, 500, false, delay)).toEqual([]);
  expect(cuesAt(cues, 900, false, delay)[0].alpha).toBe(1);
  expect(cuesAt(cues, 500, false)[0].alpha).toBe(1);
});
