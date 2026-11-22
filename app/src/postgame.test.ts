import { describe, expect, it } from 'vitest';
import { drawnLength, eraAt, eraBands, postGame, timelineByEra } from './postgame';
import type { Chronicle, EmpireHistory, GameHistory, PlayerPublic, PlayerView } from './types';

const empire = (vp: number[], extra: Partial<EmpireHistory> = {}): EmpireHistory => ({
  vp, cities: vp.map(() => 2), population: vp.map(() => 5), science: vp.map(() => 3), gold: vp.map(() => 50), army: vp.map((_, i) => 10 + i), techs: vp.map(() => 4), ...extra,
});

function view(history: GameHistory, over: Partial<PlayerView> = {}): PlayerView {
  const players = [
    { id: 0, name: 'Player 1', faction: 'Switzerland', team: 0, alive: true, era: 3 },
    { id: 1, name: 'AI 2', faction: 'Mongols', team: 1, alive: true, era: 3 },
    { id: 2, name: 'AI 3', faction: 'Sparta', team: 2, alive: false, era: 2 },
  ] as PlayerPublic[];
  return {
    me: 0, team: 0, players, winner: 0, finished: true, history,
    standings: players.map((p) => ({ team: p.team, members: [p.id], vp: 0, target: 10 })),
    ...over,
  } as unknown as PlayerView;
}

const chronicle: Chronicle[] = [
  { kind: 'Founded', turn: 1, player: 0, city: 'Bern' },
  { kind: 'Founded', turn: 1, player: 1, city: 'Karakorum' },
  { kind: 'War', turn: 3, a: 1, b: 2, by: 1 },
  { kind: 'Era', turn: 3, player: 0, era: 2, first: true },
  { kind: 'Captured', turn: 4, by: 1, from: 2, city: 'Sparta' },
  { kind: 'Eliminated', turn: 4, player: 2 },
  { kind: 'War', turn: 5, a: 0, b: 1, by: 1 },
  { kind: 'Won', turn: 6, team: 0, reason: 'Player 1 held 10 of 10 VP for 10 turns' },
];

const history: GameHistory = {
  first_turn: 1,
  empires: [
    empire([0, 1, 2, 4, 8, 10], { era: [1, 1, 2, 2, 3, 3] }),
    empire([0, 1, 2, 3, 3, 4], { era: [1, 1, 1, 2, 2, 3] }),
    empire([0, 0, 1, 1, 0, 0], { era: [1, 1, 1, 2, 2, 2], cities: [1, 1, 1, 1, 0, 0] }),
  ],
  chronicle,
};

describe('postGame', () => {
  it('calls an ordered burn Razed and does not count it as a second conquest', () => {
    const h = structuredClone(history);
    h.chronicle!.push({ kind: 'Razed', turn: 5, by: 1, from: 2, city: 'Sparta', ordered: true });
    const report = postGame(view(h))!;
    expect(report.stats.find((e) => e.empire.id === 1)).toMatchObject({ captured: 1, lost: 1 });
    expect(report.timeline.some((e) => e.text === 'Mongols razed Sparta')).toBe(true);
  });
  it('is null without a recorded history', () => {
    expect(postGame(view({ first_turn: 1, empires: [] }))).toBeNull();
    expect(postGame({ ...view(history), history: undefined })).toBeNull();
  });

  it('orders empires winner first, then by final VP, and names the viewer You', () => {
    const pg = postGame(view(history))!;
    expect(pg.empires.map((e) => e.id)).toEqual([0, 1, 2]);
    expect(pg.empires[0]).toMatchObject({ label: 'You', mine: true, winner: true });
    expect(pg.empires[1].label).toBe('Mongols');
    expect(pg.firstTurn).toBe(1);
    expect(pg.lastTurn).toBe(6);
    expect(pg.target).toBe(10);
    expect(pg.partial).toBe(false);
  });

  it('ends a fallen empire\'s line at the turn it fell', () => {
    const pg = postGame(view(history))!;
    expect(pg.empires.find((e) => e.id === 2)?.fell).toBe(4);
    expect(drawnLength(pg, history, 2)).toBe(4);
    expect(drawnLength(pg, history, 0)).toBe(6);
  });

  it('starts each era band at the first empire to enter it', () => {
    expect(eraBands(history, chronicle)).toEqual([
      { era: 2, name: 'Classical', turn: 3, by: 0 },
      { era: 3, name: expect.any(String), turn: 5, by: 0 },
    ]);
    // Backfilled history has no era series: the chronicle's Era entries stand in.
    const noEras = { ...history, empires: history.empires.map((e) => ({ ...e, era: undefined })) };
    expect(eraBands(noEras, chronicle)).toEqual([{ era: 2, name: 'Classical', turn: 3, by: 0 }]);
    expect(eraAt(eraBands(history, chronicle), 2)).toBe(1);
    expect(eraAt(eraBands(history, chronicle), 4)).toBe(2);
  });

  it('words the timeline in the viewer\'s terms and flags turning points', () => {
    const pg = postGame(view(history))!;
    const text = pg.timeline.map((t) => t.text);
    expect(text).toContain('You founded Bern');
    expect(text).toContain('Mongols captured Sparta from Sparta');
    expect(text).toContain('Sparta was eliminated');
    expect(text).toContain('Mongols declared war on you');
    expect(text).toContain('You entered the Classical era first');
    expect(text.at(-1)).toBe('You won. Player 1 held 10 of 10 VP for 10 turns.');
    const war = pg.timeline.find((t) => t.text === 'Mongols declared war on you')!;
    expect(war).toMatchObject({ tone: 'against', yours: true });
    expect(pg.timeline.find((t) => t.kind === 'Eliminated')).toMatchObject({ major: true, yours: false });
    expect(pg.timeline.find((t) => t.text === 'Mongols founded Karakorum')).toMatchObject({ major: false, yours: false });
  });

  it('puts wars and captures on the VP chart\'s lanes', () => {
    const pg = postGame(view(history))!;
    expect(pg.markers.map((m) => [m.lane, m.turn])).toEqual([['war', 3], ['capture', 4], ['war', 5]]);
    expect(pg.markers[0]).toMatchObject({ who: 1, mine: false });
  });

  it('sums the statistics from the record', () => {
    const pg = postGame(view(history))!;
    const you = pg.stats.find((r) => r.empire.id === 0)!;
    expect(you).toMatchObject({ vp: 10, peakVp: 10, cities: 2, science: 18, peakArmy: 15, founded: 1, captured: 0, wars: 1 });
    const mongols = pg.stats.find((r) => r.empire.id === 1)!;
    expect(mongols).toMatchObject({ captured: 1, wars: 2 });
    expect(pg.stats.find((r) => r.empire.id === 2)).toMatchObject({ lost: 1 });
  });

  it('groups the timeline by era with each era\'s turns', () => {
    const pg = postGame(view(history))!;
    const groups = timelineByEra(pg, pg.timeline);
    expect(groups.map((g) => [g.era, g.from, g.to])).toEqual([[1, 1, 2], [2, 3, 4], [3, 5, 6]]);
  });

  it('marks a record that starts mid-game', () => {
    const pg = postGame(view({ ...history, first_turn: 40 }))!;
    expect(pg.partial).toBe(true);
    expect(pg.lastTurn).toBe(45);
  });
});
