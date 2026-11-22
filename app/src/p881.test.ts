// The p881 horseman report (2026-10-11), rebuilt from the user's own saved game: what the attack
// forecast should have said at turn 29, and where the turn-30 retreat should have gone.
import { describe, expect, it } from 'vitest';
import { enemyReach } from './enemyReach';
import { planTurns, previewRoute } from './path';
import { forecastModel } from './ui/ForecastCard';
import { classBonusStrength } from './combat';
import { unitStatsFor } from './data';
import { ARCHER, HORSEMAN, archerTile, forecastT29, p881View, playerRetreat, retreatTo } from './test/p881Fixture';

describe('turn 29: the Horseman against the fortified Archer', () => {
  it('the core forecast is the one the user saw: 0% kill', () => {
    expect(forecastT29.win_chance).toBe(0);
    expect(forecastT29.expected_damage_to_defender).toBeGreaterThan(70);
  });

  it('the card says it assumes fortification and gives the kill chance if the Archer fires first', async () => {
    const view = await p881View(29);
    const horse = view.units.find((u) => u.id === HORSEMAN)!;
    const m = forecastModel(forecastT29, false, view, horse, archerTile, []);
    expect(m.win).toBe(0);
    expect(m.fortify).toMatchObject({ bonus: 6, shoots: true });
    // The user then killed it: a ranged unit that fires loses its +6, and the charge kills on the high rolls.
    expect(m.fortify!.win).toBeGreaterThanOrEqual(30);
    expect(m.fortify!.win).toBeLessThanOrEqual(60);
  });

  it('the card shows the aftermath on the Archer\'s tile: Nara\'s walls, and that its garrison is out of sight', async () => {
    const view = await p881View(29);
    const horse = view.units.find((u) => u.id === HORSEMAN)!;
    expect(view.units.filter((u) => u.owner === 1 && u.military).map((u) => u.id)).toEqual([ARCHER]);
    const a = forecastModel(forecastT29, false, view, horse, archerTile, []).aftermath!;
    expect(a).not.toBeNull();
    expect(a.sources.map((s) => s.label)).toEqual(["Nara's walls"]);
    expect(a.sources[0].each).toBeGreaterThan(20);
    expect(a.unseen).toEqual(['Nara']);
    expect(a.risk).toBe('Exposed');
    expect(a.line).toMatch(/^If it destroys the Archer \(\d+% if it fires first\), your Horseman moves into its tile: next turn Nara's walls \(about −\d+\) can reach it\. Units you cannot see in Nara are not counted\. Exposed\.$/);
  });

  // The user lost this Horseman (Likely lost on the old numbers); since riders stand above the melee line
  // (2026-10-31) the same charge within two Spearmen's reach is Exposed, and still flagged as severe.
  it('with the garrison in sight (the turn-30 view) the same charge is flagged severe', async () => {
    const view = await p881View(30);
    const horse = view.units.find((u) => u.id === HORSEMAN)!;
    // Put the Horseman back where it started its charge, at full strength, with the Archer on the target tile.
    horse.pos = { q: 19, r: 11 };
    horse.hp = 100;
    view.units.push({ ...view.units.find((u) => u.id === 24)!, id: ARCHER, kind: 'Ranger', pos: archerTile, fortify: 6, ranged: 15, range: 2, hp: 100 });
    const m = forecastModel(forecastT29, false, view, horse, archerTile, []);
    expect(m.aftermath!.sources.map((s) => `${s.count} ${s.label}`)).toEqual(["1 Nara's walls", '2 Spearmen']);
    expect(m.aftermath!.sources[1].bonus).toBe(`+${classBonusStrength(unitStatsFor('India', 'Soldier').strength)} vs mounted`);
    expect(m.aftermath!.unseen).toEqual([]);
    expect(m.aftermath!.risk).toBe('Exposed');
    expect(m.aftermath!.severe).toBe(true);
  });
});

describe('turn 30: the retreat', () => {
  it('the route the user took had a first step a Spearman could strike; the player route now avoids it', async () => {
    const view = await p881View(30);
    const horse = view.units.find((u) => u.id === HORSEMAN)!;
    expect(horse.pos).toEqual({ q: 19, r: 13 });
    const reach = enemyReach(view);
    expect(reach.threatened(playerRetreat[0], 1)).toBe(true);

    const plain = previewRoute(view, horse.pos, retreatTo, [], horse.id);
    // Without the option the preview gives exactly the route the user ordered.
    expect(plain.path).toEqual(playerRetreat);
    const safe = previewRoute(view, horse.pos, retreatTo, [], horse.id, { avoidEnemyReach: true });
    expect(safe.path!.at(-1)).toEqual(retreatTo);
    expect(reach.threatened(safe.path![0], 1)).toBe(false);
    expect(safe.path![0]).not.toEqual(playerRetreat[0]);
    expect(safe.danger).toBeUndefined();
    const turns = (p: typeof plain.path) => planTurns(view, p!, horse.pos, horse.max_moves).at(-1)!.turn;
    expect(turns(safe.path)).toBe(turns(plain.path));
    expect(turns(safe.path)).toBe(turns(playerRetreat));
  });

  it('the first step the learnings found safe (north) is the one chosen', async () => {
    const view = await p881View(30);
    const horse = view.units.find((u) => u.id === HORSEMAN)!;
    const safe = previewRoute(view, horse.pos, retreatTo, [], horse.id, { avoidEnemyReach: true });
    expect([{ q: 19, r: 12 }, { q: 20, r: 12 }]).toContainEqual(safe.path![0]);
  });
});
