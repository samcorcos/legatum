import { describe, expect, it } from 'vitest';
import { buildTracks, leadingRival, vpProgress, vpTicks } from './vp';
import type { PlayerPublic, TeamStanding } from './types';

const player = (id: number, team: number, alive = true): PlayerPublic => ({
  id,
  name: `P${id}`,
  faction: 'India',
  team,
  is_human: id < 2,
  alive,
  vp: 0,
  vp_sources: [],
  cities: 1,
  faction_goal_done: false,
  era: 1,
});

const players = [player(0, 0), player(1, 0), player(2, 1), player(3, 2), player(4, 3, false)];
const standings: TeamStanding[] = [
  { team: 0, members: [0, 1], vp: 9, target: 20 },
  { team: 1, members: [2], vp: 6, target: 10 },
  { team: 2, members: [3], vp: 3, target: 10 },
  { team: 3, members: [4], vp: 0, target: 10 },
];

describe('VP track', () => {
  it('progress is vp/target clamped to 0..1', () => {
    expect(vpProgress(5, 10)).toBe(0.5);
    expect(vpProgress(12, 10)).toBe(1);
    expect(vpProgress(3, 0)).toBe(0);
  });

  it('orders teams by progress toward their own target, not raw VP', () => {
    const t = buildTracks({ standings, players, team: 0 });
    // 6/10 = 0.6 beats 9/20 = 0.45.
    expect(t.map((r) => r.team)).toEqual([1, 0, 2, 3]);
    expect(t[1].isMine).toBe(true);
    expect(t[1].needed).toBe(11);
    expect(t[1].label).toBe('India · P0 & India · P1');
    expect(t[3].alive).toBe(false);
  });

  it('ties share a rank', () => {
    const t = buildTracks({
      standings: [
        { team: 1, members: [2], vp: 2, target: 10 },
        { team: 2, members: [3], vp: 2, target: 10 },
        { team: 0, members: [0, 1], vp: 0, target: 20 },
      ],
      players,
      team: 0,
    });
    expect(t.map((r) => r.rank)).toEqual([1, 1, 3]);
    expect(t.map((r) => r.team)).toEqual([1, 2, 0]);
  });

  it('finds the leading rival and draws one tick per VP', () => {
    const t = buildTracks({ standings, players, team: 0 });
    expect(leadingRival(t)?.team).toBe(1);
    expect(vpTicks(10)).toHaveLength(9);
    expect(vpTicks(10)[0]).toBeCloseTo(0.1);
  });
});

describe('VP track, living empires only', () => {
  const eliminatedLeader: TeamStanding[] = [
    { team: 0, members: [0, 1], vp: 2, target: 20 },
    { team: 1, members: [2], vp: 1, target: 10 },
    { team: 2, members: [3], vp: 0, target: 10 },
    { team: 3, members: [4], vp: 9, target: 10 },
  ];

  it('drops an eliminated rival and ranks the rest among themselves', () => {
    const all = buildTracks({ standings: eliminatedLeader, players, team: 0 });
    expect(all.map((r) => r.team)).toEqual([3, 0, 1, 2]);
    const living = buildTracks({ standings: eliminatedLeader, players, team: 0 }, { aliveOnly: true });
    expect(living.map((r) => r.team)).toEqual([0, 1, 2]);
    expect(living.map((r) => r.rank)).toEqual([1, 2, 3]);
    expect(living.some((r) => !r.alive)).toBe(false);
  });

  it('keeps your own row after your team falls, so you can still see yourself', () => {
    const fallen = players.map((p) => (p.team === 0 ? { ...p, alive: false } : p));
    const living = buildTracks({ standings: eliminatedLeader, players: fallen, team: 0 }, { aliveOnly: true });
    expect(living.map((r) => r.team)).toEqual([0, 1, 2]);
    expect(living.find((r) => r.isMine)?.alive).toBe(false);
  });

  it('a two-empire team stays while either member lives', () => {
    const half = players.map((p) => (p.id === 0 ? { ...p, alive: false } : p));
    const living = buildTracks({ standings: eliminatedLeader, players: half, team: 1 }, { aliveOnly: true });
    expect(living.some((r) => r.team === 0)).toBe(true);
  });
});
