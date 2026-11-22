import { describe, expect, it } from 'vitest';
import { BUILD_MS, CONSTRUCTION_ANIMATION, FLIP_MS, FOUND, LINGER, buildMomentPlan, buildPhases, holdAt, momentTime, momentsLive, movesEnd } from './momentFx';
import { buildCombatPlan } from './combatFx';
import { tileIndex } from '../hex';
import type { Event, Hex, PlayerView } from '../types';

const W = 20, H = 16;
type Tile = { explored: boolean; visible: boolean; owner: number | null; resource?: 'Iron' | null; road?: number; improvement?: { kind: string } | null };
const tiles = (own: [Hex, number][] = []): Tile[] => {
  const t: Tile[] = Array.from({ length: W * H }, () => ({ explored: true, visible: true, owner: null }));
  for (const [h, o] of own) t[tileIndex(h, W, H)].owner = o;
  return t;
};
const view = (t: Tile[], cities: { id: number; owner: number; pos: Hex; population?: number }[]) => ({
  me: 0, team: 0, width: W, height: H, tiles: t, units: [], aircraft: [],
  players: [{ id: 0, team: 0, era: 2, faction: 'Switzerland' }, { id: 2, team: 1, era: 1, faction: 'Mongols' }],
  cities: cities.map((c) => ({ hp: 100, max_hp: 100, walls: false, name: 'Bern', population: 1, visible: true, ...c })),
}) as unknown as PlayerView;

const CITY = { q: 6, r: 6 };
const ring1: Hex[] = [{ q: 7, r: 6 }, { q: 5, r: 6 }, { q: 6, r: 5 }, { q: 7, r: 5 }, { q: 6, r: 7 }, { q: 5, r: 7 }];
const founded: Event = { type: 'CityFounded', city: 4, owner: 0, name: 'Bern', at: CITY } as Event;

describe('payoff moments plan', () => {
  it('plans an ordinary turn of wonder production from the chosen tile and holds its old stage', () => {
    const chosen = { q: 7, r: 6 };
    const before = view(tiles([[CITY, 0], [chosen, 0]]), [{ id: 4, owner: 0, pos: CITY }]);
    const after = view(tiles([[CITY, 0], [chosen, 0]]), [{ id: 4, owner: 0, pos: CITY }]);
    for (const [v, stored] of [[before, 90], [after, 270]] as const) {
      v.cities[0].detail = { producing: { kind: 'Wonder', building: 'GreatLibrary', tile: chosen }, production_stored: stored, production_cost: 600 } as PlayerView['cities'][number]['detail'];
    }
    const plan = buildMomentPlan([], before, after, 0)!;
    expect(plan.builds).toHaveLength(1);
    expect(plan.builds[0]).toMatchObject({ at: chosen, from: .15, to: .45, wonder: true });
    if (!CONSTRUCTION_ANIMATION) return; // the held stage belongs to the construction animation, which is off until the designer approves the clips
    expect(holdAt(plan, plan.builds[0].t - 1).sites.get(4)).toBe(.15);
    expect(holdAt(plan, plan.builds[0].t + 600).sites.get(4)).toBe(.15);
    expect(holdAt(plan, plan.builds[0].t + BUILD_MS.step).sites.has(4)).toBe(false);
  });
  it('starts the first production step from the empty plot when a site is queued this turn', () => {
    const chosen = { q: 7, r: 6 };
    const before = view(tiles([[CITY, 0], [chosen, 0]]), [{ id: 4, owner: 0, pos: CITY }]);
    const after = view(tiles([[CITY, 0], [chosen, 0]]), [{ id: 4, owner: 0, pos: CITY }]);
    after.cities[0].detail = { producing: { kind: 'Wonder', building: 'GreatLibrary', tile: chosen }, production_stored: 60, production_cost: 600 } as PlayerView['cities'][number]['detail'];
    const plan = buildMomentPlan([], before, after, 0)!;
    expect(plan.builds[0]).toMatchObject({ city: 4, at: chosen, from: 0, to: .1 });
    if (!CONSTRUCTION_ANIMATION) return;
    expect(holdAt(plan, plan.builds[0].t - 1).sites.get(4)).toBe(0);
  });
  it('holds a newly revealed deposit until its ink medallion appears', () => {
    const before = view(tiles(), [{ id: 4, owner: 0, pos: CITY }]);
    const afterTiles = tiles();
    const iron = ring1[0];
    afterTiles[tileIndex(iron, W, H)].resource = 'Iron';
    const after = view(afterTiles, [{ id: 4, owner: 0, pos: CITY }]);
    const events = [{ type: 'TechResearched', player: 0, tech: 'IronWorking' }, { type: 'ResourcesRevealed', player: 0, resource: 'Iron', city: null, sources: 0, supply: 0 }] as Event[];
    const plan = buildMomentPlan(events, before, after, 0)!;
    expect(plan.reveals.map((r) => r.resource)).toEqual(['Iron']);
    const reveal = plan.reveals[0];
    expect(holdAt(plan, reveal.t - 1).resources.has(reveal.tile)).toBe(true);
    expect(holdAt(plan, reveal.t + 849).resources.has(reveal.tile)).toBe(true);
    expect(holdAt(plan, reveal.t + 850).resources.has(reveal.tile)).toBe(false);
    expect(buildMomentPlan([], before, after, 0)).toBeNull();
  });
  it('keeps a captured original capital\'s star filled until its VP seal has landed', () => {
    const mine = { id: 4, owner: 0, pos: CITY, is_capital: true };
    const taken = { id: 9, owner: 0, pos: { q: 12, r: 8 } };
    const before = view(tiles(), [mine, { ...taken, owner: 2 }]);
    const after = view(tiles(), [mine, taken]);
    const events = [{ type: 'CapitalTaken', captor: 0, capital: 9, name: 'Bern' }] as Event[];
    const plan = buildMomentPlan(events, before, after, 0)!;
    const seal = plan.seals.find((x) => x.kind === 'vp')!;
    expect(plan.capitals).toHaveLength(1);
    expect(plan.capitals[0].t).toBeGreaterThan(seal.t);
    expect(holdAt(plan, seal.t).unscored.has(9)).toBe(true);
    expect(holdAt(plan, plan.capitals[0].t - 1).unscored.has(9)).toBe(true);
    expect(holdAt(plan, plan.capitals[0].t).unscored.has(9)).toBe(false);
    expect(holdAt(plan, plan.total).unscored.size).toBe(0);
  });
  it('is null for a turn without a payoff', () => {
    const v = view(tiles(), []);
    expect(buildMomentPlan([], v, v, 0)).toBeNull();
  });

  it('founds the city after the moves, then spreads its border ring by ring, swept around the city', () => {
    const before = view(tiles(), []);
    const after = view(tiles([[CITY, 0], ...ring1.map((h) => [h, 0] as [Hex, number])]), [{ id: 4, owner: 0, pos: CITY }]);
    const plan = buildMomentPlan([founded], before, after, 900)!;
    const f = plan.founds[0];
    expect(f.t).toBe(1050);
    const centre = plan.tiles.find((x) => x.hex.q === CITY.q && x.hex.r === CITY.r)!;
    const outer = plan.tiles.filter((x) => x !== centre);
    expect(outer).toHaveLength(6);
    for (const x of outer) expect(x.t).toBeGreaterThan(centre.t + FOUND.ringStep * 0.9);
    // Swept, not all at once.
    expect(new Set(outer.map((x) => Math.round(x.t))).size).toBeGreaterThan(3);
    expect(momentsLive(plan, plan.total - 1)).toBe(true);
    expect(momentsLive(plan, plan.total + 1)).toBe(false);
  });

  it.skipIf(!CONSTRUCTION_ANIMATION)('holds back the town, its banner and its land until each one is drawn in', () => {
    const before = view(tiles(), []);
    const after = view(tiles([[CITY, 0], ...ring1.map((h) => [h, 0] as [Hex, number])]), [{ id: 4, owner: 0, pos: CITY }]);
    const plan = buildMomentPlan([founded], before, after, 0)!;
    const f = plan.founds[0];
    const at = (dt: number) => holdAt(plan, f.t + dt);
    const town = tileIndex(CITY, W, H);
    // During the moves, nothing of the new city shows.
    expect(holdAt(plan, 0).towns.has(town)).toBe(true);
    expect(holdAt(plan, 0).owner.size).toBe(7);
    expect(holdAt(plan, 0).banner.get(4)!.alpha).toBe(0);
    // The banner drops in once the town is mostly painted, and the cache takes the town back when it is done.
    expect(at(FOUND.banner - 1).banner.get(4)!.alpha).toBe(0);
    expect(at((FOUND.banner + FOUND.bannerEnd) / 2).banner.get(4)!.alpha).toBeGreaterThan(0.3);
    expect(at(FOUND.done).towns.has(town)).toBe(false);
    expect(at(FOUND.done).banner.has(4)).toBe(false);
    const end = holdAt(plan, plan.total);
    expect([end.towns.size, end.owner.size, end.banner.size, end.sig]).toEqual([0, 0, 0, holdAt(plan, plan.total + 500).sig]);
  });

  it('with construction switched off, shows new towns, buildings and walls at once but keeps them in the plan for a new design', () => {
    const before = view(tiles(), []);
    const after = view(tiles([[CITY, 0], ...ring1.map((h) => [h, 0] as [Hex, number])]), [{ id: 4, owner: 0, pos: CITY }]);
    const walls = { type: 'ProductionCompleted', city: 4, owner: 0, item: { kind: 'Building', building: 'Walls' } } as Event;
    const plan = buildMomentPlan([founded, walls], before, after, 0)!;
    expect(plan.founds).toHaveLength(1);
    expect(plan.builds.map((b) => [b.walls, b.t > 0])).toEqual([[true, true]]);
    if (CONSTRUCTION_ANIMATION) return;
    const hold = holdAt(plan, plan.founds[0].t + 10);
    expect([hold.towns.size, hold.walls.size, hold.banner.size]).toEqual([0, 0, 0]);
    // The border still spreads from the new city.
    expect(holdAt(plan, 0).owner.size).toBe(7);
    expect(momentTime(plan, CITY)).toBeNull();
  });

  it('keeps the terrain signature steady between changes, so the cache repaints only when a hold changes', () => {
    const before = view(tiles(), []);
    const after = view(tiles([[CITY, 0], ...ring1.map((h) => [h, 0] as [Hex, number])]), [{ id: 4, owner: 0, pos: CITY }]);
    const plan = buildMomentPlan([founded], before, after, 0)!;
    const sigs = new Set<string>();
    for (let t = 0; t < plan.total; t += 1000 / 60) sigs.add(holdAt(plan, t).sig);
    // One per claimed tile, one for the town, and the empty hold: never one per frame.
    expect(sigs.size).toBeLessThanOrEqual(plan.tiles.length + 3);
  });

  it.skipIf(!CONSTRUCTION_ANIMATION)('raises finished buildings one after another with the wonder last, and holds walls until they are sketched in', () => {
    const v = view(tiles([[CITY, 0]]), [{ id: 4, owner: 0, pos: CITY }, { id: 5, owner: 0, pos: { q: 10, r: 6 } }]);
    const events = [
      { type: 'ProductionCompleted', city: 5, owner: 0, item: { kind: 'Building', building: 'GreatLibrary' } },
      { type: 'ProductionCompleted', city: 4, owner: 0, item: { kind: 'Building', building: 'Walls' } },
    ] as Event[];
    const plan = buildMomentPlan(events, v, v, 400)!;
    expect(plan.builds.map((b) => [b.city, b.wonder, b.walls])).toEqual([[4, false, true], [5, true, false]]);
    expect(plan.builds[1].t).toBeGreaterThan(plan.builds[0].t);
    const walls = plan.builds[0];
    // The cache leaves the walls out while they are being built, and takes them back once the scaffold is down.
    const done = walls.t + buildPhases(walls).done;
    expect(holdAt(plan, done - 1).walls.has(walls.tile)).toBe(true);
    expect(holdAt(plan, done + LINGER).walls.has(walls.tile)).toBe(false);
    // A wonder's construction is the longest; the map's own celebration waits for its flourish.
    expect(buildPhases(plan.builds[1]).done).toBe(BUILD_MS.finish);
    expect(BUILD_MS.finish).toBeGreaterThan(BUILD_MS.building);
    expect(momentTime(plan, { q: 10, r: 6 })).toBe(plan.builds[1].t + buildPhases(plan.builds[1]).flourish);
  });

  it('flips a captured banner at the capture and floods the land after it', () => {
    const own = ring1.map((h) => [h, 2] as [Hex, number]);
    const before = view(tiles([[CITY, 2], ...own]), [{ id: 4, owner: 2, pos: CITY }]);
    const after = view(tiles([[CITY, 0], ...ring1.map((h) => [h, 0] as [Hex, number])]), [{ id: 4, owner: 0, pos: CITY }]);
    const events = [{ type: 'CityCaptured', city: 4, from: 2, to: 0 }] as Event[];
    const plan = buildMomentPlan(events, before, after, 300, () => 1200)!;
    const flip = plan.flips[0];
    expect(flip.t).toBe(1200);
    expect(holdAt(plan, 1100).cityOwner.get(4)).toBe(2);
    expect(holdAt(plan, 1200 + FLIP_MS * 0.25).banner.get(4)!.sx).toBeLessThan(1);
    expect(holdAt(plan, 1200 + FLIP_MS * 0.75).cityOwner.has(4)).toBe(false);
    // Every tile of the city changes colour after the flip has turned, outward from the city.
    for (const x of plan.tiles) expect(x.t).toBeGreaterThanOrEqual(1200 + FLIP_MS * 0.5);
    expect(holdAt(plan, 1199).owner.get(tileIndex(CITY, W, H))).toBe(2);
  });

  it('starts the moments when the combat playback has finished moving', () => {
    const v = view(tiles(), []);
    const units = { ...v, units: [{ id: 8, owner: 0, kind: 'Spearman', pos: { q: 3, r: 4 }, hp: 100, military: true }] } as unknown as PlayerView;
    const combat = buildCombatPlan([
      { type: 'UnitMoved', unit: 8, owner: 0, from: { q: 3, r: 4 }, to: { q: 4, r: 4 } },
      { type: 'UnitDestroyed', unit: 9, owner: 2, unit_kind: 'Spearman', at: { q: 5, r: 4 } },
    ] as Event[], units, units);
    expect(movesEnd(combat, 0)).toBe(combat!.moves.get(8)![0].t1);
    expect(movesEnd(null, 700)).toBe(700);
  });

  it('stamps treaties on both capitals early in the turn and the score late, and glints a promoted unit after the moves', () => {
    const v = view(tiles(), [{ id: 4, owner: 0, pos: CITY, is_capital: true } as never, { id: 9, owner: 2, pos: { q: 14, r: 6 }, is_capital: true } as never]);
    const withUnit = { ...v, units: [{ id: 3, owner: 0, kind: 'Spearman', pos: { q: 7, r: 6 }, hp: 100, military: true }] } as unknown as PlayerView;
    const relation = { a: 0, b: 1, state: 'Alliance' };
    const events = [
      { type: 'TreatyActivated', relation },
      { type: 'EraObjectiveScored', player: 2, era: 2, slot: 1, objective: 'Gain 4 cities' },
      { type: 'UnitPromoted', unit: 3, owner: 0, unit_kind: 'Spearman', level: 1, at: { q: 7, r: 6 } },
    ] as Event[];
    const plan = buildMomentPlan(events, withUnit, withUnit, 600)!;
    const alliance = plan.seals.filter((x) => x.kind === 'alliance');
    expect(alliance.map((x) => x.at)).toEqual([CITY, { q: 14, r: 6 }]);
    const score = plan.seals.find((x) => x.kind === 'vp')!;
    expect(score.owner).toBe(2);
    expect(score.t).toBeGreaterThan(Math.max(...alliance.map((x) => x.t)));
    expect(plan.glints).toEqual([expect.objectContaining({ unit: 3 })]);
    expect(plan.glints[0].t).toBeGreaterThan(600);
  });
});
