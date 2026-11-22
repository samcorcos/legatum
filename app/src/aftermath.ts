// "After the attack": what waits for a melee winner on the tile it advances into (p881 horseman,
// 2026-10-11). A melee unit that kills its defender moves onto the defender's tile (core
// `rules/movement.rs`, module doc step 5), so the fight's own numbers say nothing about the next
// turn. This evaluates that tile with what the viewer knows: walled enemy cities that can fire there
// (core `forecast_city_strike` at that tile) and visible enemy units that can reach it, each with its
// expected damage including the class bonus. Fog is never read: a unit out of sight is not counted.
import type { BattleForecast, Hex, PlayerView, UnitKind, UnitView } from './types';
import { cityCanFire, exactOdds, fightClassBonus, unitAttackStrength, unitDefenseStrength, unitRangedStrength } from './combat';
import { distance, tileIndex } from './hex';
import { atWar, isThreat } from './diplomacy';
import { firesIndirect, itemName, pluralUnit, unitClass } from './data';
import {rangedClear, viewTerrain, walledGarrisonAt} from './los';
import { unitCanReach } from './settlement';

/** Cities this far from the tile can send out units next turn (a 2-move unit steps twice, then strikes). */
const UNSEEN_CITY_RANGE = 3;

type AftermathRisk = 'Safe' | 'Exposed' | 'Likely lost';

/** One thing that can hit the unit next turn: a walled city, or `count` enemy units of one kind. */
interface AftermathSource {
  kind: 'city' | 'unit';
  /** "Nara's walls", "Spearman" / "Spearmen" (already plural). */
  label: string;
  count: number;
  /** Expected damage of one hit at average rolls. */
  each: number;
  /** "+8 vs mounted" when the class matchup favours the enemy. */
  bonus?: string;
  /** City strikes land before movement, ranged shots before melee. */
  phase: 'city' | 'ranged' | 'melee';
}

export interface Aftermath {
  /** The tile the winner advances into. */
  tile: Hex;
  /** "Archer": what it kills, for the sentence. */
  defenderName: string;
  unitName: string;
  sources: AftermathSource[];
  /** Enemy cities near the tile whose garrison is out of sight: what they hold is not counted. */
  unseen: string[];
  /** HP it stands on after winning (kill heals included). */
  hp: number;
  /** Expected total damage, hits applied one after another (each wound weakens the next defence). */
  total: number;
  risk: AftermathRisk;
  /** The risk is bad enough for the forecast headline. */
  severe: boolean;
  line: string;
}

const expectedHit = (attack: number, defense: number, cap: number) =>
  exactOdds(attack, defense, 1, cap, true).expToDef;

/**
 * `hpAfter` is the attacker's HP once this attack's own fight and city fire are paid
 * (`forecastModel().hpAfter`). `killHeals` restores a surviving Sparta attacker to full HP.
 * `canWin` is false when no roll kills the defender even without its fortification (the p881
 * Archer's forecast said 0% and the charge still killed it, so the default is the forecast's own
 * chance and callers add the unfortified one).
 * Null when the attack cannot advance: ranged, a city target, or no chance to kill.
 */
export function attackAftermath(view: PlayerView, unit: UnitView, target: Hex, forecast: BattleForecast, hpAfter: number, killHeals = false, canWin = forecast.win_chance > 0, firesFirstWin: number | null = null): Aftermath | null {
  if (forecast.ranged || !canWin || view.cities.some((c) => c.pos.q === target.q && c.pos.r === target.r)) return null;
  const defender = view.units.find((u) => u.pos.q === target.q && u.pos.r === target.r && u.military && atWar(view, u.owner));
  if (!defender) return null;
  const faction = view.players.find((p) => p.id === unit.owner)?.faction;
  const tile = view.tiles[tileIndex(target, view.width, view.height)];
  if (!tile?.terrain) return null;
  const hp = Math.max(1, killHeals ? 100 : hpAfter);
  // The winner has acted, so it stands unfortified; it is the only unit on its side the shots are aimed at.
  const defenceAt = (current: number) => {
    const d = unitDefenseStrength(unit.kind, current, tile.terrain!, 0, faction, unit.level ?? 0);
    return d.total;
  };
  const defence = (current: number, against: UnitKind, shot: boolean) => defenceAt(current) + fightClassBonus(unit.kind, against, false, shot, faction).bonus;
  const terrain = viewTerrain(view);

  type Hit = AftermathSource & { strength: number; against: UnitKind | null };
  const hits: Hit[] = [];
  for (const city of view.cities) {
    const gap = distance(city.pos, target);
    if (!cityCanFire(city.walls, city.hp) || !atWar(view, city.owner) || gap < 1 || gap > 2) continue;
    hits.push({ kind: 'city', phase: 'city', label: `${city.name}'s walls`, count: 1, each: 0, strength: city.strength, against: null });
  }
  for (const enemy of view.units) {
    if (!enemy.military || enemy.hp <= 0 || enemy.id === defender.id || !isThreat(view, enemy.owner)) continue;
    const eFaction = view.players.find((p) => p.id === enemy.owner)?.faction;
    const shoots = enemy.ranged > 0 && distance(enemy.pos, target) <= enemy.range
      && rangedClear(enemy.kind, enemy.pos, target, terrain,
        firesIndirect(enemy.kind),
        walledGarrisonAt(view, enemy.owner, enemy.pos));
    if (!shoots) {
      if (!unitCanReach(view, enemy, target)) continue;
    }
    const strength = shoots
      ? unitRangedStrength(enemy.kind, enemy.hp, eFaction, enemy.level ?? 0).total
      : unitAttackStrength(enemy.kind, enemy.hp, eFaction, enemy.level ?? 0).total;
    const bonus = fightClassBonus(enemy.kind, unit.kind, true, shoots, eFaction).bonus;
    const name = itemName({ kind: 'Unit', unit: enemy.kind }, eFaction);
    const matchup = bonus > 0 ? `+${bonus} vs ${unitClass(unit.kind)?.toLowerCase() ?? 'this class'}` : undefined;
    hits.push({ kind: 'unit', phase: shoots ? 'ranged' : 'melee', label: name, count: 1, each: 0, strength: strength + bonus, against: enemy.kind, ...(matchup ? { bonus: matchup } : {}) });
  }

  // Each hit's own damage on the unit as it stands after winning, for the line ...
  for (const h of hits) h.each = expectedHit(h.strength, h.against ? defence(hp, h.against, h.phase === 'ranged') : defenceAt(hp), hp);
  // ... and the hits applied one after another in the order the turn resolves, each wound weakening
  // the next defence (a city's shot is aimed at its lowest-HP target, so it is counted in full).
  const order = { city: 0, ranged: 1, melee: 2 } as const;
  let left = hp;
  let total = 0;
  for (const h of [...hits].sort((a, b) => order[a.phase] - order[b.phase] || b.each - a.each)) {
    const current = Math.max(1, left);
    const dealt = expectedHit(h.strength, h.against ? defence(current, h.against, h.phase === 'ranged') : defenceAt(current), 2_147_483_647);
    total += dealt;
    left -= dealt;
  }
  // A city out of sight can hold units the view does not show (Nara's Spearmen at T29): never call that Safe.
  const unseen = view.cities.filter((c) => !c.visible && atWar(view, c.owner) && distance(c.pos, target) <= UNSEEN_CITY_RANGE).map((c) => c.name);
  const risk: AftermathRisk = total >= hp && hits.length ? 'Likely lost' : hits.length || unseen.length ? 'Exposed' : 'Safe';

  // Group units of one kind so "2 Spearmen" reads once.
  const sources: AftermathSource[] = [];
  for (const h of hits) {
    const same = h.kind === 'unit' && sources.find((s) => s.kind === 'unit' && s.label === h.label && s.phase === h.phase && s.each === h.each);
    if (same) same.count += 1;
    else sources.push({ kind: h.kind, phase: h.phase, label: h.label, count: h.count, each: h.each, ...(h.bonus ? { bonus: h.bonus } : {}) });
  }
  for (const s of sources) if (s.count > 1) s.label = pluralUnit(s.label);

  const unitName = itemName({ kind: 'Unit', unit: unit.kind }, faction);
  const defenderName = itemName({ kind: 'Unit', unit: defender.kind }, view.players.find((p) => p.id === defender.owner)?.faction);
  return {
    tile: target, defenderName, unitName, sources, unseen, hp, total, risk,
    severe: risk === 'Likely lost' || (risk === 'Exposed' && total * 2 >= hp),
    line: aftermathLine(unitName, defenderName, sources, risk, unseen, forecast.win_chance === 0 ? firesFirstWin : null),
  };
}

/** "If it destroys the Archer, your Horseman moves into its tile: next turn Nara's walls (about −42) and 2 Spearmen (about −63 each, +8 vs mounted) can reach it. Likely lost." */
function aftermathLine(unitName: string, defenderName: string, sources: AftermathSource[], risk: AftermathRisk, unseen: string[] = [], firesFirstWin: number | null = null): string {
  // The card's own word for the event is "destroy"; when the forecast says 0%, the line carries the chance it does hold under.
  const condition = firesFirstWin ? ` (${firesFirstWin}% if it fires first)` : '';
  const move = `If it destroys the ${defenderName}${condition}, your ${unitName} moves into its tile`;
  const word = risk === 'Safe' ? 'Safe as far as you can see' : risk;
  const hidden = unseen.length ? ` Units you cannot see in ${unseen.join(' and ')} are not counted.` : '';
  if (!sources.length) return `${move}: no visible enemy can reach it next turn.${hidden} ${word}.`;
  const part = (s: AftermathSource) => {
    const note = `about −${s.each}${s.count > 1 ? ' each' : ''}${s.bonus ? `, ${s.bonus}` : ''}`;
    return `${s.count > 1 ? `${s.count} ` : s.kind === 'unit' ? `${/^[aeiou]/i.test(s.label) ? 'an' : 'a'} ` : ''}${s.label} (${note})`;
  };
  const parts = sources.map(part);
  const list = parts.length > 1 ? `${parts.slice(0, -1).join(', ')} and ${parts.at(-1)}` : parts[0];
  return `${move}: next turn ${list} can reach it.${hidden} ${word}.`;
}
