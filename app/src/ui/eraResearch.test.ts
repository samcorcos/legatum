import { describe, expect, it } from 'vitest';
import type { Breakdown, ResearchView, TechId, TechOption } from '../types';
import {
  appendToQueue, arrivalText, cheapestFirst, costParts, currentEraTechs, diffusionLine, eraFirst, eraRows,
  moveInQueue, queueSchedule, requiresEra, researchFirst,
} from './eraResearch';

const breakdown = (parts: [string, number][]): Breakdown => ({ parts, total: parts.reduce((n, [, v]) => n + v, 0) });
const option = (tech: TechId, era: number, total: number, extra: Partial<TechOption> = {}): TechOption => ({
  tech, era, researchable: false, turns: null, known_by: 0, cost: breakdown([['Listed cost', total]]),
  ...extra,
});

/** Ancient fully known, Classical current with two known, Medieval locked. */
function research(overrides: Partial<ResearchView> = {}): ResearchView {
  const catalog: TechOption[] = [
    option('Irrigation', 0, 20), option('BronzeWorking', 0, 25),
    option('Mathematics', 1, 55, { researchable: false }),
    option('Engineering', 1, 60, { researchable: true }),
    option('IronWorking', 1, 65, { researchable: true }),
    option('CodeOfLaws', 1, 70, { researchable: true }),
    option('Philosophy', 1, 75, { researchable: true }),
    option('Feudalism', 2, 110), option('Machinery', 2, 115),
  ];
  return {
    era: 1, era_name: 'Classical', era_total: 5, era_known: 1, era_firsts: [[1, 3]], schedule: [], current: null, queue: [],
    progress: 0, cost: null, turns_left: null, known: ['Irrigation', 'BronzeWorking', 'Mathematics'], available: [], catalog, unlocks: [], ...overrides,
  };
}

describe('eras', () => {
  it('one row per era, past, current and locked', () => {
    const rows = eraRows(research());
    expect(rows.map((r) => [r.name, r.state, r.known, r.total])).toEqual([
      ['Ancient', 'past', 2, 2], ['Classical', 'current', 1, 5], ['Medieval', 'locked', 0, 2],
    ]);
    expect(rows[2].techs).toEqual(['Feudalism', 'Machinery']);
  });
  it('the current era is the only set of cards', () => {
    expect(currentEraTechs(research()).map((t) => t.tech)).toEqual(['Mathematics', 'Engineering', 'IronWorking', 'CodeOfLaws', 'Philosophy']);
  });
  it('says who entered an era first', () => {
    expect(eraFirst(research(), 1)).toBe(3);
    expect(eraFirst(research(), 2)).toBeNull();
  });
  it('words a locked tech by the era it waits on', () => {
    const view = { research: research() };
    expect(requiresEra(view, 'Feudalism')).toBe(' (Medieval era)');
    expect(requiresEra(view, 'Engineering')).toBe('');
  });
});

describe('queue forecast', () => {
  it('sums costs against stored Science and income, in queue order', () => {
    const r = research({ progress: 10 });
    const slots = queueSchedule(r, ['Engineering', 'IronWorking', 'Philosophy'], 10, 20);
    expect(slots.map((s) => [s.tech, s.cost, s.turns, s.arrivalTurn])).toEqual([
      ['Engineering', 60, 5, 25],   // (60-10)/10
      ['IronWorking', 65, 12, 32],  // (125-10)/10 = 11.5 -> 12
      ['Philosophy', 75, 19, 39],   // (200-10)/10
    ]);
  });
  it('follows a reorder: the same techs in another order arrive at other turns', () => {
    const r = research();
    const ab = queueSchedule(r, ['Engineering', 'Philosophy'], 10, 1);
    const ba = queueSchedule(r, ['Philosophy', 'Engineering'], 10, 1);
    expect(ab.map((s) => s.arrivalTurn)).toEqual([7, 15]);
    expect(ba.map((s) => s.arrivalTurn)).toEqual([9, 15]);
  });
  it('keeps arrivals strictly increasing (one tech per turn)', () => {
    const r = research({ progress: 500 });
    expect(queueSchedule(r, ['Engineering', 'IronWorking', 'CodeOfLaws'], 10, 5).map((s) => s.arrivalTurn)).toEqual([6, 7, 8]);
  });
  it('has no arrival without Science income, and says so', () => {
    const slots = queueSchedule(research(), ['Engineering', 'IronWorking'], 0, 3);
    expect(slots.map((s) => s.arrivalTurn)).toEqual([null, null]);
    expect(arrivalText(slots[0])).toBe('Needs Science');
    expect(arrivalText(slots[0], true)).toBe('No Science yet');
  });
  it('words an arrival as a turn and a distance', () => {
    expect(arrivalText({ turns: 6, arrivalTurn: 24 })).toBe('Turn 24 · in 6 turns');
    expect(arrivalText({ turns: 1, arrivalTurn: 9 })).toBe('Turn 9 · in 1 turn');
  });
});

describe('queue edits', () => {
  it('moves one step and stops at the ends', () => {
    expect(moveInQueue(['a', 'b', 'c'], 1, -1)).toEqual(['b', 'a', 'c']);
    expect(moveInQueue(['a', 'b', 'c'], 1, 1)).toEqual(['a', 'c', 'b']);
    expect(moveInQueue(['a', 'b'], 0, -1)).toEqual(['a', 'b']);
    expect(moveInQueue(['a', 'b'], 1, 1)).toEqual(['a', 'b']);
  });
  it('research-now puts a tech first and keeps the rest; queueing appends once', () => {
    expect(researchFirst(['Engineering', 'IronWorking'], 'IronWorking')).toEqual(['IronWorking', 'Engineering']);
    expect(researchFirst(['Engineering'], 'Philosophy')).toEqual(['Philosophy', 'Engineering']);
    expect(appendToQueue(['Engineering'], 'Philosophy')).toEqual(['Engineering', 'Philosophy']);
    expect(appendToQueue(['Engineering'], 'Engineering')).toEqual(['Engineering']);
  });
  it('auto-order is cheapest first among the researchable ones, discounts counted', () => {
    const r = research();
    r.catalog = r.catalog.map((t) => t.tech === 'Philosophy' ? { ...t, cost: breakdown([['Listed cost', 75], ['Tech diffusion (4 empires know it)', -40]]) } : t);
    expect(cheapestFirst(r)).toEqual(['Philosophy', 'Engineering', 'IronWorking', 'CodeOfLaws']);
  });
});

describe('cost text', () => {
  const cost = breakdown([['Listed cost', 100], ['Tech diffusion (4 empires know it)', -40]]);
  it('reads base and diffusion from the breakdown', () => {
    const parts = costParts(cost);
    expect(parts.base).toBe(100);
    expect(parts.total).toBe(60);
    expect(parts.diffusion).toEqual({ percent: 40, empires: 4 });
    expect(diffusionLine(parts)).toBe('−40%: known by 4 empires');
  });
  it('shows no discount lines for a plain price', () => {
    const parts = costParts(breakdown([['Listed cost', 100], ['Tech diffusion (0 empires know it)', 0]]));
    expect(parts.diffusion).toBeNull();
    expect(diffusionLine(parts)).toBeNull();
  });
});
