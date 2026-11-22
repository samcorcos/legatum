// Telling same-class units of different eras apart on the map (user, 2026-10-11: "The Hoplite and the plus-one era from
// the Hoplite looked nearly identical... I just attacked some things not realizing it"; decided from the mocks in
// docs/design/unit-art/era-distinct.md). Besides a distinct silhouette per era (the painted set), two cues:
//   - the era plaque: a small square tab with the era's Roman numeral on the front of every military miniature's base,
//     normal zoom only (far out the class roundel is too small to carry it);
//   - the strength cue: with one of your military units selected, each enemy it could attack this turn carries how much
//     stronger (red, rising mark) or weaker (green, falling mark) it is in that fight, class bonus counted; far out, only
//     the mark. The attack forecast leads with "Phalangite, one era newer: 26 vs your 22".
import type { Faction, PlayerView, UnitKind, UnitView } from '../types';
import { attacksInMelee, itemName, unitEraTier, unitStatsFor } from '../data';
import { addFightClassBonus, unitAttackStrength, unitDefenseStrength, unitRangedStrength } from '../combat';
import { atWar } from '../diplomacy';
import { distance, tileIndex } from '../hex';
import { reachTiles } from './reach';
import { ATLAS } from './style';

export const ROMAN = ['I', 'II', 'III', 'IV', 'V', 'VI', 'VII', 'VIII', 'IX'];

/** The plaque's fill per era: one metal each, mostly alternating in lightness. The Machine Age's blued steel (VI, placeholder
 * until its art pass) is dark like the World War era's olive (VII) beside it, told apart by hue; the Modern era's teal (VIII,
 * a placeholder too) is dark as well. */
export const ERA_METAL = ['#a8692a', '#5c5853', '#aeb9c4', '#2f2c2a', '#c9a23a', '#3e4c57', '#6b7044', '#2f6f6a', '#e6f0f2'];
const DARK_METAL = new Set([0, 1, 3, 5, 6, 7]);

export const unitEra = (kind: UnitKind): number => unitEraTier(kind);

/** A unit's strength in a fight against `other`, as the forecast's first lines count it (base, promotion, wounds, class bonus); no terrain or fortify. */
export function matchupStrength(u: UnitView, faction: Faction | undefined, other: UnitView, attacking: boolean): number {
  const f = faction ?? 'India';
  const kind = u.kind as UnitKind;
  const shot = shoots(attacking ? u : other, attacking ? other : u);
  const b = attacking
    ? (shot ? unitRangedStrength(kind, u.hp, f, u.level ?? 0) : unitAttackStrength(kind, u.hp, f, u.level ?? 0))
    : unitDefenseStrength(kind, u.hp, 'Grassland', 0, f, u.level ?? 0);
  addFightClassBonus(b, kind, other.kind as UnitKind, attacking, shot, f);
  return b.total;
}

/** Whether `attacker`'s attack on `target` is a shot: a ranged unit fires, and a mounted archer (the Keshig) shoots what is within its range and charges the rest. */
export function shoots(attacker: UnitView, target: UnitView): boolean {
  if (attacker.ranged <= 0) return false;
  return !attacksInMelee(attacker.kind) || distance(attacker.pos, target.pos) <= Math.max(1, attacker.range ?? 1);
}

/**
 * The enemy military units `sel` could attack this turn: within its range for a ranged unit, else on a tile it can reach
 * (moving onto an enemy is the attack). Enemies are units of empires at War with you, and barbarians.
 */
export function attackableEnemies(view: PlayerView, sel: UnitView): UnitView[] {
  const enemies = view.units.filter((u) => u.military && u.owner !== view.me && atWar(view, u.owner));
  if (!enemies.length || !sel.military) return [];
  const inRange = (u: UnitView) => distance(u.pos, sel.pos) <= Math.max(1, sel.range ?? 2);
  if (sel.ranged > 0 && !attacksInMelee(sel.kind)) return enemies.filter(inRange);
  const reach = reachTiles(view, sel);
  const reachable = (u: UnitView) => reach.has(tileIndex(u.pos, view.width, view.height));
  // A Keshig can shoot what is in range and charge what it can reach.
  if (sel.ranged > 0) return enemies.filter((u) => inRange(u) || reachable(u));
  return enemies.filter(reachable);
}

/** "one era newer", "two eras older", "same era". */
export function eraDeltaWords(their: UnitKind, mine: UnitKind): string {
  const d = unitEraTier(their) - unitEraTier(mine);
  if (d === 0) return 'same era';
  const n = ['', 'one', 'two', 'three', 'four', 'five', 'six'][Math.abs(d)];
  return `${n} era${Math.abs(d) === 1 ? '' : 's'} ${d > 0 ? 'newer' : 'older'}`;
}

/**
 * A newer unit that is stronger by its own kit, not just by promotion or wounds: a newer era, a higher base strength and
 * the higher total in this fight. The forecast then leads with the warning the user needed (he attacked a Phalangite with
 * a Hoplite without noticing).
 */
export function outclasses(defKind: UnitKind, defFaction: Faction | undefined, myKind: UnitKind, myFaction: Faction | undefined, theirs: number, mine: number): boolean {
  return unitEraTier(defKind) > unitEraTier(myKind) && theirs > mine
    && unitStatsFor(defFaction ?? 'India', defKind).strength > unitStatsFor(myFaction ?? 'India', myKind).strength;
}

/** The forecast's warning line when the defender outclasses you: "Phalangite, one era newer: 26 vs your 22. It outclasses you." */
export function forecastEraHeadline(defKind: UnitKind, defFaction: Faction | undefined, myKind: UnitKind, theirs: number, mine: number): string {
  return `${itemName({ kind: 'Unit', unit: defKind }, defFaction)}, ${eraDeltaWords(defKind, myKind)}: ${theirs} vs your ${mine}. It outclasses you.`;
}

const ink = () => ATLAS.tone.ink;

/** The plaque's height for token half-width `w`. */
export const plaqueHeight = (w: number) => Math.max(11, Math.min(15, w * 0.78));

/**
 * The era plaque on the front of a miniature's base: a small square-cornered tab in the era's metal with its numeral (light
 * on the dark metals, ink on the light ones), so it never reads as the route preview's red-ringed turn numerals.
 */
export function drawEraPlaque(ctx: CanvasRenderingContext2D, era: number, x: number, y: number, w: number, font: string) {
  const e = Math.max(0, Math.min(6, era));
  const h = plaqueHeight(w);
  const text = ROMAN[e];
  ctx.save();
  ctx.font = `700 ${Math.round(h * 0.84)}px ${font}`;
  const pw = Math.max(h * 1.05, ctx.measureText(text).width + h * 0.6);
  const path = () => { ctx.beginPath(); ctx.roundRect(x - pw / 2, y - h / 2, pw, h, 2); };
  ctx.translate(0.6, 0.9); path(); ctx.fillStyle = ATLAS.tone.shadow; ctx.fill(); ctx.translate(-0.6, -0.9);
  path(); ctx.fillStyle = ERA_METAL[e]; ctx.fill();
  ctx.lineWidth = 1; ctx.strokeStyle = ink(); ctx.stroke();
  ctx.fillStyle = DARK_METAL.has(e) ? ATLAS.tone.light : ink();
  ctx.textAlign = 'center';
  ctx.textBaseline = 'middle';
  ctx.fillText(text, x, y + 0.6);
  ctx.restore();
}

/**
 * An enemy's strength against your selected unit, as the difference: a rising red mark and "4" when it is 4 stronger, a
 * falling green mark and "18" when it is 18 weaker, an even mark when equal. `markOnly` (far zoom) drops the number.
 */
export function drawStrengthChip(ctx: CanvasRenderingContext2D, value: number, against: number, x: number, y: number, font: string, scale = 1, markOnly = false, neutral = false) {
  const h = 14 * scale;
  const dir = value > against ? 1 : value < against ? -1 : 0;
  const color = neutral ? ATLAS.tone.ink : dir > 0 ? ATLAS.tone.bad : dir < 0 ? ATLAS.tone.good : ink();
  ctx.save();
  ctx.font = `700 ${Math.round(11 * scale)}px ${font}`;
  const text = markOnly || dir === 0 ? '' : String(Math.abs(value - against));
  const tw = text ? ctx.measureText(text).width : 0;
  const tri = 8 * scale;
  const pw = tw + tri + (text ? 10 : 8) * scale;
  const left = x - pw / 2;
  ctx.beginPath(); ctx.roundRect(left + 0.6, y - h / 2 + 1, pw, h, 3 * scale); ctx.fillStyle = ATLAS.tone.shadow; ctx.fill();
  ctx.beginPath(); ctx.roundRect(left, y - h / 2, pw, h, 3 * scale);
  ctx.fillStyle = dir === 0 ? ATLAS.tone.light : color; ctx.fill();
  ctx.lineWidth = 1; ctx.strokeStyle = ink(); ctx.stroke();
  const fg = dir === 0 ? ink() : ATLAS.tone.light;
  const tx = left + 4 * scale + tri / 2, s = 3.6 * scale;
  ctx.beginPath();
  if (dir > 0) { ctx.moveTo(tx - s, y + s * 0.8); ctx.lineTo(tx + s, y + s * 0.8); ctx.lineTo(tx, y - s); }
  else if (dir < 0) { ctx.moveTo(tx - s, y - s * 0.8); ctx.lineTo(tx + s, y - s * 0.8); ctx.lineTo(tx, y + s); }
  else { ctx.rect(tx - s, y - s * 0.75, 2 * s, s * 0.5); ctx.rect(tx - s, y + s * 0.25, 2 * s, s * 0.5); }
  ctx.closePath(); ctx.fillStyle = fg; ctx.fill();
  ctx.fillStyle = fg;
  ctx.textAlign = 'left';
  ctx.textBaseline = 'middle';
  if (text) ctx.fillText(text, left + 5 * scale + tri, y + 0.6);
  ctx.restore();
}
