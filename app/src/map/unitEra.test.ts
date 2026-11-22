// The era cues (user, 2026-10-11): the forecast headline's wording, and strength chips only on enemies the selected unit
// could attack this turn.
import { describe, expect, it } from 'vitest';
import { FixtureClient } from '../test/fixtureEngine';
import { unitEraFixture } from '../test/unitEraFixture';
import { attackableEnemies, drawStrengthChip, eraDeltaWords, forecastEraHeadline, matchupStrength, outclasses, shoots } from './unitEra';
import { classBonusStrength, unitDefenseStrength } from '../combat';
import { unitStatsFor } from '../data';
import { addUnit, arenaView } from '../test/arenaView';
import { distance } from '../hex';
import { ATLAS } from './style';

describe('era cues', () => {
  it('uses neutral ink for a Celtic view of a barbarian strength chip', () => {
    const fills: string[] = [];
    const fake = {
      fillStyle: '', save() {}, restore() {}, measureText: () => ({ width: 8 }), beginPath() {}, roundRect() {},
      fill() { fills.push(this.fillStyle); }, stroke() {}, moveTo() {}, lineTo() {}, closePath() {}, rect() {}, fillText() {},
    };
    drawStrengthChip(fake as unknown as CanvasRenderingContext2D, 30, 20, 20, 20, 'sans-serif', 1, false, true);
    expect(fills).toContain(ATLAS.tone.ink);
    expect(fills).not.toContain(ATLAS.tone.bad);
  });
  it('words the era gap', () => {
    expect(eraDeltaWords('Phalangite', 'Spearman')).toBe('one era newer');
    expect(eraDeltaWords('Spearman', 'Musketman')).toBe('three eras older');
    expect(eraDeltaWords('Horseman', 'Spearman')).toBe('same era');
    expect(forecastEraHeadline('Phalangite', 'Athens', 'Spearman', 26, 22)).toBe('Phalangite, one era newer: 26 vs your 22. It outclasses you.');
    // Outclassed only by a newer unit that is stronger by its own kit, not by promotion or wounds.
    expect(outclasses('Phalangite', 'Athens', 'Spearman', 'Sparta', 26, 22)).toBe(true);
    expect(outclasses('Horseman', 'Athens', 'Spearman', 'Sparta', 20, 40)).toBe(false);
    expect(outclasses('Spearman', 'Athens', 'Phalangite', 'Sparta', 30, 26)).toBe(false);
    expect(outclasses('Phalangite', 'Athens', 'Spearman', 'Sparta', 18, 22)).toBe(false);
  });

  it('marks only enemies the selected unit could attack this turn', () => {
    const setup = unitEraFixture();
    const client = new FixtureClient();
    client.load(JSON.stringify(setup.fixture));
    const view = client.view(0);
    const hoplite = view.units.find((u) => u.id === setup.hoplite)!;
    const targets = attackableEnemies(view, hoplite);
    expect(targets.map((u) => u.id)).toContain(setup.phalangite);
    // Two moves: nothing more than two tiles away, and never your own units.
    expect(targets.every((u) => u.owner !== view.me && distance(u.pos, hoplite.pos) <= 2)).toBe(true);
    const far = view.units.filter((u) => u.owner !== view.me && u.military && distance(u.pos, hoplite.pos) > 2);
    expect(far.length).toBeGreaterThan(0);
    // A ranged unit marks enemies within its range of where it stands.
    const archer = view.units.find((u) => u.owner === view.me && u.kind === 'Archer')!;
    expect(attackableEnemies(view, archer).every((u) => distance(u.pos, archer.pos) <= archer.range)).toBe(true);
  });

  it('marks what a Keshig can shoot and what it can reach, and counts the spear line bonus only against its charge', async () => {
    const v = await arenaView('Mongols');
    const keshig = addUnit(v, 0, 'Cataphract', { q: 8, r: 5 });
    const near = addUnit(v, 2, 'Spearman', { q: 9, r: 5 });
    const far = addUnit(v, 2, 'Spearman', { q: 11, r: 5 });
    expect(shoots(keshig, near)).toBe(true);
    expect(shoots(keshig, far)).toBe(false);
    // An Archer shoots everything within its range and nothing else; a Cataphract never shoots.
    expect(attackableEnemies(v, keshig).map((u) => u.id).sort()).toEqual([near.id, far.id].sort());
    const base = unitDefenseStrength('Spearman', 100, 'Grassland', 0, 'Byzantium', 0).total;
    expect(matchupStrength(near, 'Byzantium', keshig, false)).toBe(base);
    expect(matchupStrength(far, 'Byzantium', keshig, false)).toBe(base + classBonusStrength(unitStatsFor('Byzantium', 'Spearman').strength));
  });
});
