import { beforeAll, describe, expect, it } from 'vitest';
import { FixtureClient } from './test/fixtureEngine';
import type { PlayerView } from './types';
import { atWarLine, nearestRival, rivalLine, rivalWin } from './rivalWin';

let view: PlayerView;
beforeAll(async () => {
  const client = new FixtureClient();
  await client.newGame(await client.standardSetup('42', 'India', 'Byzantium', 'Ada', 'Bo'));
  view = client.view(0);
});

const at = (team: number, vp: number): PlayerView => ({ ...view, standings: view.standings.map((s) => (s.team === team ? { ...s, vp, target: 10 } : s)) });

describe('rivalWin', () => {
  it('says how far a rival is from winning and nothing for your own team', () => {
    const v = at(1, 8);
    expect(rivalWin(v, v.team)).toBeNull();
    const r = rivalWin(v, 1)!;
    expect(r).toMatchObject({ team: 1, vp: 8, target: 10, left: 2 });
    expect(rivalLine(v, r)).toMatch(/ are 2 VP from winning\.$/);
    expect(rivalLine(at(1, 10), rivalWin(at(1, 10), 1)!)).toMatch(/ have 10 VP and are holding it\.$/);
  });

  it('names who is at war with them, you first, and omits the line when nobody is', () => {
    const v = structuredClone(at(1, 8));
    v.diplomacy.relations = v.diplomacy.relations.filter((r) => r.a !== 1 && r.b !== 1);
    expect(atWarLine(v, rivalWin(v, 1)!)).toBeNull();
    const rel = (a: number, b: number) => ({ ...view.diplomacy.relations[0], a, b, state: 'War' as const });
    v.diplomacy.relations.push(rel(1, 2), rel(0, 1));
    expect(atWarLine(v, rivalWin(v, 1)!)).toMatch(/^At war with them: you, the /);
  });

  it('offers War only when it is legal and hides it when already at War', () => {
    const v = at(1, 8);
    const r = rivalWin(v, 1)!;
    expect(r.declare === null || 'action' in r.declare || 'blocked' in r.declare).toBe(true);
    const warring = structuredClone(v);
    warring.diplomacy.relations = warring.diplomacy.relations.map((x) => ((x.a === 0 && x.b === 1) || (x.a === 1 && x.b === 0) ? { ...x, state: 'War' as const } : x));
    expect(rivalWin(warring, 1)!.declare).toBeNull();
  });

  it('picks the leading rival within two VP of its target', () => {
    expect(nearestRival(at(1, 7))).toBeNull();
    expect(nearestRival(at(1, 8))!.team).toBe(1);
  });
});
