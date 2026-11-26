// Which sounds a battle makes, and when: read off the same plan the map animates (map/combatFx.ts),
// so a volley is heard as it leaves the bow and a clash as the lines meet. Only weapons we have a
// recording for make a sound (docs/design/sound-sources.md). Pure, so it is tested without audio.
import type { CombatPlan } from '../map/combatFx';
import type { UnitKind } from '../types';
import type { BattleCue, BattleSound } from './sound';

/** A turn with twenty fights is a wall of noise: a few sounds of each kind carry it. */
export const MAX_BATTLE_CUES = 10;
/**
 * What is kept when a turn has more to say than that, most consequential first (a razed city over a
 * fallen unit over a clash over a volley), with how many of each. Never the first ten by time: the
 * volleys open every turn and would leave the clash, the deaths and the collapse silent.
 */
export const CUE_QUOTA: [BattleSound, number][] = [
  ['collapse', MAX_BATTLE_CUES], ['death', 2], ['fall', 2], ['explosion', 2], ['clash', 2], ['gunshot', 2], ['cannon', 2], ['musket', 2], ['shellFire', 1], ['aircraft', 1], ['jet', 1], ['drone', 1],
  ['siegeHit', 2], ['arrowHit', 2], ['bow', 2], ['siegeLaunch', 1], ['hooves', 1],
];
/** The same sound twice inside this window reads as one. */
export const CUE_MERGE_MS = 160;
const HOOVES_LEAD_MS = 450;
const AIRCRAFT_LEAD_MS = 1500;

/** The jets; every other craft is a propeller aircraft, as is a strike with no craft named (combatFx falls back to the Bomber). */
const JETS = new Set(['JetBomber', 'JetFighter', 'StealthBomber', 'StealthFighter']);

/** Units from before gunpowder fall with one set of cries; soldiers of the gunpowder age and later with another. */
/** Machines and their crews: they are destroyed with a blast, not a cry. */
const MACHINES = new Set<UnitKind>(['Tank', 'EarlyTank', 'MainBattleTank', 'AutonomousArmour', 'HunterKiller', 'MechanisedInfantry', 'Artillery', 'Howitzer', 'RocketArtillery',
  'CombatDrone', 'Biplane', 'Fighter', 'Bomber', 'JetFighter', 'JetBomber', 'StealthFighter', 'StealthBomber']);
const OLD_WORLD = new Set<UnitKind>(['Scout', 'Spearman', 'Archer', 'Pikeman', 'Crossbowman', 'Horseman', 'Knight', 'Phalangite', 'CompositeArcher', 'Cataphract', 'HeirToTheThrone']);

export function battleSoundCues(plan: CombatPlan | null | undefined): BattleCue[] {
  if (!plan) return [];
  const all: BattleCue[] = [];
  const add = (t: number, sound: BattleSound, seed: number) => all.push({ t: Math.max(0, Math.round(t)), sound, seed });
  for (const s of plan.shots) {
    if (s.kind === 'arrows' || s.kind === 'bolts') {
      add(s.launch, 'bow', s.seed);
      if (s.hit) add(s.launch + s.flight, 'arrowHit', s.seed);
    } else if (s.kind === 'stone') {
      add(s.launch, 'siegeLaunch', s.seed);
      add(s.launch + s.flight, 'siegeHit', s.seed);
    } else if (s.kind === 'shot') add(s.launch, 'musket', s.seed);
    else if (s.kind === 'drone') add(s.launch, 'drone', s.seed);
    else if (s.kind === 'ball') add(s.launch, 'cannon', s.seed);
    else if (s.kind === 'shell') {
      add(s.launch, 'shellFire', s.seed);
      add(s.launch + s.flight, 'explosion', s.seed);
    } else if (s.kind === 'bombs') {
      // An aircraft is heard on its way in, a propeller one and a jet each by its own recording; the bombs when they land.
      add(s.launch - AIRCRAFT_LEAD_MS, s.craft && JETS.has(s.craft) ? 'jet' : 'aircraft', s.seed);
      add(s.launch + s.flight, 'explosion', s.seed);
    }
  }
  for (const c of plan.clashes) {
    if (c.guns) { add(c.t, c.rifles ? 'gunshot' : 'musket', c.seed); continue; }
    if (c.mounted) add(c.t - HOOVES_LEAD_MS, 'hooves', c.seed);
    add(c.t, 'clash', c.seed);
  }
  for (const d of plan.deaths) {
    if (d.unit.kind === 'Settler') continue;
    add(d.t, OLD_WORLD.has(d.unit.kind) ? 'death' : MACHINES.has(d.unit.kind) ? 'explosion' : 'fall', d.seed);
  }
  for (const c of plan.captures) if (c.razed) add(c.t, 'collapse', c.city);
  all.sort((a, b) => a.t - b.t);
  const last = new Map<BattleSound, number>();
  const distinct = all.filter((cue) => {
    const prev = last.get(cue.sound);
    if (prev != null && cue.t - prev < CUE_MERGE_MS) return false;
    last.set(cue.sound, cue.t);
    return true;
  });
  const kept = CUE_QUOTA.flatMap(([sound, most]) => distinct.filter((c) => c.sound === sound).slice(0, most)).slice(0, MAX_BATTLE_CUES);
  return kept.sort((a, b) => a.t - b.t);
}
