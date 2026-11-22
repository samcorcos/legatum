import { describe, expect, it } from 'vitest';
import type { Event, Hex, PlayerView } from '../types';
import { buildCombatPlan, FLIGHT_MS } from './combatFx';

type Case = { shots: number; clashes: number; movers: number; rounds: number; deaths: number; captures: number };
const hex = (q: number, r: number): Hex => ({ q, r });

function warTurn({ shots, clashes, movers, rounds, deaths, captures }: Case) {
  const shotUnits = Array.from({ length: shots }, (_, i) => ({ id: i + 1, owner: 0, kind: 'Ranger', pos: hex(2, i) }));
  const movingUnits = Array.from({ length: movers }, (_, i) => ({ id: i + 101, owner: 0, kind: 'Soldier', pos: hex(5, i) }));
  const defenders = Array.from({ length: clashes }, (_, i) => ({ id: i + 201, owner: 2, kind: 'Soldier', pos: hex(12, i) }));
  const cities = Array.from({ length: captures }, (_, i) => ({ id: i + 1, owner: 2, pos: hex(5 + rounds, i) }));
  const view = {
    me: 0, team: 0, width: 40, height: 40,
    players: [{ id: 0, team: 0, era: 3 }, { id: 2, team: 1, era: 3 }],
    units: [...shotUnits, ...movingUnits, ...defenders, { id: 999, owner: 2, kind: 'Soldier', pos: hex(9, 3) }]
      .map((u) => ({ hp: 100, military: true, ranged: 0, ...u })),
    cities: cities.map((c) => ({ hp: 100, max_hp: 100, walls: true, name: 'City', ...c })),
    aircraft: [],
  } as unknown as PlayerView;
  const events: Event[] = [];
  for (const u of shotUnits) events.push({
    type: 'Battle', attacker: u.id, attacker_owner: 0,
    defender: { kind: 'Unit', unit: 999, owner: 2, unit_kind: 'Soldier' },
    at: hex(9, 3), attacker_at: u.pos, damage_to_attacker: 0, damage_to_defender: 10,
    ranged: true, meeting: false,
  } as Event);
  let clash = 0;
  for (let round = 0; round < rounds; round++) {
    for (const u of movingUnits) events.push({
      type: 'UnitMoved', unit: u.id, owner: u.owner,
      from: hex(5 + round, u.pos.r), to: hex(6 + round, u.pos.r),
    } as Event);
    const through = Math.floor((round + 1) * clashes / rounds);
    while (clash < through) {
      const defender = defenders[clash];
      events.push({
        type: 'Battle', attacker: movingUnits[clash % movers].id, attacker_owner: 0,
        defender: { kind: 'Unit', unit: defender.id, owner: 2, unit_kind: 'Soldier' },
        at: defender.pos, attacker_at: hex(6 + round, clash % movers),
        damage_to_attacker: 0, damage_to_defender: 30, ranged: false, meeting: false,
      } as Event);
      // Ordinary turns lose units early; the huge case also loses units in its final round.
      if ((clashes < 10 && clash < deaths) || (clashes >= 10 && clash >= clashes - deaths)) events.push({
        type: 'UnitDestroyed', unit: defender.id, owner: 2, unit_kind: 'Soldier', at: defender.pos,
      } as Event);
      clash++;
    }
  }
  for (const city of cities) events.push({ type: 'CityCaptured', city: city.id, from: 2, to: 0 } as Event);
  return { events, view };
}

describe('combat playback pacing budget', () => {
  it.each([
    ['quiet turn', { shots: 1, clashes: 0, movers: 0, rounds: 0, deaths: 0, captures: 0 }, 2500],
    ['typical war turn', { shots: 6, clashes: 4, movers: 6, rounds: 4, deaths: 2, captures: 0 }, 4500],
    ['huge turn', { shots: 30, clashes: 20, movers: 25, rounds: 6, deaths: 8, captures: 2 }, 6000],
  ] as const)('%s finishes within budget and leaves a readable tail', (_name, spec, budget) => {
    const { events, view } = warTurn(spec);
    const plan = buildCombatPlan(events, view, view)!;
    expect(plan.shots).toHaveLength(spec.shots);
    expect(plan.clashes).toHaveLength(spec.clashes);
    expect(plan.moves.size).toBe(spec.movers);
    expect(plan.deaths).toHaveLength(spec.deaths);
    expect(plan.captures).toHaveLength(spec.captures);
    expect(plan.total).toBeLessThanOrEqual(budget);
    const lastAction = Math.max(plan.volley, ...plan.clashes.map((c) => c.t),
      ...plan.captures.map((c) => c.t),
      ...[...plan.moves.values()].flatMap((glides) => glides.map((g) => g.t1)));
    expect(lastAction).toBeLessThanOrEqual(plan.total - 600);
    if (spec.rounds) expect(plan.stepMs * spec.rounds).toBeLessThanOrEqual(1400);
    for (const shot of plan.shots) expect(shot.flight).toBeGreaterThanOrEqual(FLIGHT_MS[shot.kind]);
    if (spec.shots === 6) {
      for (let i = 1; i < plan.shots.length; i++) {
        const gap = plan.shots[i].launch - plan.shots[i - 1].launch;
        expect(gap).toBeGreaterThanOrEqual(150);
        expect(gap).toBeLessThanOrEqual(300);
      }
    }
    if (spec.shots === 30) {
      for (let i = 1; i < plan.shots.length; i++) {
        expect(plan.shots[i].launch - plan.shots[i - 1].launch).toBeCloseTo(60);
      }
    }
  });
});
