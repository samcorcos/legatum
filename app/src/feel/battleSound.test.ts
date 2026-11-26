import { describe, expect, it } from 'vitest';
import type { CombatPlan } from '../map/combatFx';
import { battleSoundCues, CUE_MERGE_MS, MAX_BATTLE_CUES } from './battleSound';

const plan = (p: Partial<Record<'shots' | 'clashes' | 'deaths' | 'captures', unknown[]>>) =>
  ({ shots: [], clashes: [], deaths: [], captures: [], ...p }) as unknown as CombatPlan;
const names = (p: CombatPlan) => battleSoundCues(p).map((c) => `${c.sound}@${c.t}`);

describe('battle sound cues', () => {
  it('has nothing to say without a battle', () => {
    expect(battleSoundCues(null)).toEqual([]);
    expect(battleSoundCues(plan({}))).toEqual([]);
  });

  it('sounds a volley as it leaves and as it lands, and only a hit lands', () => {
    expect(names(plan({ shots: [
      { kind: 'arrows', launch: 100, flight: 500, hit: true, seed: 1 },
      { kind: 'bolts', launch: 900, flight: 400, hit: false, seed: 2 },
      { kind: 'stone', launch: 1500, flight: 800, hit: true, seed: 3 },
    ] }))).toEqual(['bow@100', 'arrowHit@600', 'bow@900', 'siegeLaunch@1500', 'siegeHit@2300']);
  });

  it('sounds muskets, rifles, cannon, shells, drones and aircraft; a gunpowder-age soldier falls with a cry and a machine with a blast', () => {
    expect(names(plan({
      shots: ['shot', 'ball', 'shell', 'drone'].map((kind) => ({ kind, launch: 0, flight: 300, hit: true, seed: 1 })),
      clashes: [{ t: 500, mounted: false, guns: true, rifles: true, seed: 1 }, { t: 900, mounted: false, guns: true, seed: 2 }],
      deaths: [{ unit: { kind: 'Rifleman' }, t: 700, seed: 1 }, { unit: { kind: 'Tank' }, t: 900, seed: 1 }],
    }))).toEqual(['cannon@0', 'musket@0', 'shellFire@0', 'drone@0', 'explosion@300', 'gunshot@500', 'fall@700', 'explosion@900', 'musket@900']);
    expect(names(plan({ deaths: [{ unit: { kind: 'Settler' }, t: 100, seed: 1 }] }))).toEqual([]);
    // An aircraft is heard before its bombs land: a propeller one and a jet each by its own recording.
    expect(names(plan({ shots: [{ kind: 'bombs', craft: 'Bomber', launch: 2000, flight: 600, hit: true, seed: 1 }] }))).toEqual(['aircraft@500', 'explosion@2600']);
    expect(names(plan({ shots: [{ kind: 'bombs', craft: 'JetBomber', launch: 2000, flight: 600, hit: true, seed: 1 }] }))).toEqual(['jet@500', 'explosion@2600']);
    expect(names(plan({ shots: [{ kind: 'bombs', craft: 'Fighter', launch: 2000, flight: 600, hit: true, seed: 1 }] }))).toEqual(['aircraft@500', 'explosion@2600']);
    expect(names(plan({ shots: [{ kind: 'bombs', craft: 'StealthFighter', launch: 2000, flight: 600, hit: true, seed: 1 }] }))).toEqual(['jet@500', 'explosion@2600']);
  });

  it('leads a mounted charge with hooves, voices an old-world unit’s fall and a razed city’s collapse', () => {
    expect(names(plan({
      clashes: [{ t: 1000, mounted: true, guns: false, seed: 5 }, { t: 100, mounted: true, guns: false, seed: 6 }],
      deaths: [{ unit: { kind: 'Spearman' }, t: 1300, seed: 1 }],
      captures: [{ t: 2000, razed: true, city: 3 }, { t: 2600, razed: false, city: 4 }],
    }))).toEqual(['hooves@0', 'clash@100', 'clash@1000', 'death@1300', 'collapse@2000']);
  });

  it('merges the same sound struck together and keeps only a few of each kind', () => {
    const together = plan({ clashes: [0, 40, CUE_MERGE_MS - 1, CUE_MERGE_MS + 5].map((t) => ({ t, mounted: false, guns: false, seed: t })) });
    expect(names(together)).toEqual(['clash@0', `clash@${CUE_MERGE_MS + 5}`]);
    const crowded = plan({ clashes: Array.from({ length: 40 }, (_, i) => ({ t: i * 300, mounted: false, guns: false, seed: i })) });
    expect(names(crowded)).toEqual(['clash@0', 'clash@300']);
  });

  it('a crowded turn keeps what matters most: the volleys never silence the clash, the fall and the collapse', () => {
    const cues = battleSoundCues(plan({
      shots: Array.from({ length: 5 }, (_, i) => ({ kind: 'arrows', launch: i * 190, flight: 500, hit: true, seed: i })),
      clashes: [{ t: 2000, mounted: true, guns: false, seed: 1 }],
      deaths: [{ unit: { kind: 'Archer' }, t: 2300, seed: 1 }],
      captures: [{ t: 3000, razed: true, city: 7 }],
    }));
    expect(cues.length).toBeLessThanOrEqual(MAX_BATTLE_CUES);
    expect(cues.map((c) => c.sound)).toEqual(['bow', 'bow', 'arrowHit', 'arrowHit', 'hooves', 'clash', 'death', 'collapse']);
    expect(cues.map((c) => c.t)).toEqual([...cues.map((c) => c.t)].sort((a, b) => a - b));
  });
});
