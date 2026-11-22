// How a faction's own piece differs from the standard item it replaces, as plain data: numbers from the same
// stat functions the cards read (`unitStatsFor` and `buildingStatsFor` against their `standard...` twins, which
// leave out the piece's own modifiers but keep the faction-wide ones), plus whole sentences for rule differences
// the numbers can't show. Every surface that shows a unique piece (the faction sheet, the build card, the unlock
// hovercard, the Guide) renders this one comparison, so they never disagree. Legatum-ui: a unique piece always
// says how it differs from what it replaces, and never restates what the replaced item also does.
import {
  HOPLITE_PHALANX_STRENGTH, KESHIG_CHOICE, RULES, itemKey, itemName, standardBuildingStatsFor, standardUnitStatsFor, buildingStatsFor, unitStatsFor,
} from './data';
import { YIELD_KEYS, YIELD_LABEL, type YieldKey } from './ui/yieldKeys';
import type { BuildingKind, Faction, ProductionItem, UnitKind } from './types';

export type Tone = 'good' | 'bad';
export type DiffId = 'cost' | 'moves' | 'strength' | 'ranged' | 'range' | 'cityStrength' | YieldKey;

/** One number that differs: `now` on the piece, `was` on the item it replaces. `tone` says whether that is better. */
export interface PieceDiff { id: DiffId; now: number; was: number; tone: Tone }
/** A difference in how it works that the numbers can't show, in a whole sentence. */
export interface PieceRule { text: string; tone: Tone }

export interface PieceComparison {
  /** The piece's own name. */
  name: string;
  /** The standard item it is measured against, by name ("Walls"). */
  against: string;
  /** `replaces`: the faction builds this instead of `against`. `adds`: it stands beside `against`, which the faction can still build. */
  mode: 'replaces' | 'adds';
  diffs: PieceDiff[];
  rules: PieceRule[];
  /** Nothing differs at all: same numbers, no rule. */
  same: boolean;
}

interface PieceSpec { against: ProductionItem; mode?: 'adds'; rules?: PieceRule[] }
const unit = (u: UnitKind): ProductionItem => ({ kind: 'Unit', unit: u });
const building = (b: BuildingKind): ProductionItem => ({ kind: 'Building', building: b });

/** The faction pieces that stand against a standard item, keyed by the piece's `itemKey`. Names come from core's replacement names; a rename with the same stats is listed too, so it reads as the rename it is. */
const PIECES: Partial<Record<Faction, Record<string, PieceSpec>>> = {
  Egypt: { 'b:Granary': { against: building('Granary') }, 'u:Spearman': { against: unit('Spearman'), rules: [{ text: '+1 sight.', tone: 'good' }] } },
  Rome: {
    'u:Phalangite': { against: unit('Phalangite'), rules: [{ text: 'A city founded by this Legionary starts at population 2 instead of 1.', tone: 'good' }] },
    'b:Walls': { against: building('Walls'), rules: [{ text: 'Upgrading a melee unit in this city restores 50 HP, up to full health, once per order.', tone: 'good' }] },
  },
  Aztecs: { 'u:Spearman': { against: unit('Spearman') } },
  Celts: {
    'u:Phalangite': {
      against: unit('Phalangite'),
      rules: [
        { text: `+${RULES.specialUnitNumbers.gaesataeAttack} strength when attacking.`, tone: 'good' },
        { text: `−${-RULES.specialUnitNumbers.gaesataeDefense} strength when defending.`, tone: 'bad' },
      ],
    },
  },
  Switzerland: {
    'b:Walls': { against: building('Walls') },
    'u:Pikeman': { against: unit('Pikeman'), rules: [{ text: `+${RULES.specialUnitNumbers.reislauferMountedPercent}% of its strength against mounted units, on top of the usual bonus.`, tone: 'good' }] },
  },
  India: {
    'b:Market': { against: building('Market'), rules: [{ text: 'In a non-capital city, this office starts sending half the Gold the city makes to your treasury.', tone: 'good' }] },
    'u:Knight': { against: unit('Knight'), rules: [{ text: 'No strength penalty charging a city.', tone: 'good' }] },
  },
  Byzantium: {
    'u:Pikeman': { against: unit('Pikeman') },
  },
  Germany: {
    'b:Forge': { against: building('Forge') },
    'u:Tank': { against: unit('Tank') },
  },
  Mongols: { 'u:Cataphract': { against: unit('Cataphract'), rules: [{ text: KESHIG_CHOICE, tone: 'good' }] } },
  Sparta: {
    'u:Spearman': {
      against: unit('Spearman'),
      rules: [
        { text: `+${HOPLITE_PHALANX_STRENGTH} strength next to another Hoplite (the phalanx).`, tone: 'good' },
        { text: 'It is lost on upgrade.', tone: 'bad' },
      ],
    },
  },
  Athens: { 'b:Library': { against: building('Library') }, 'u:CompositeArcher': { against: unit('CompositeArcher') } },
};

/** The item keys that have a comparison for `faction`. */
const comparedPieces = (faction: Faction): string[] => Object.keys(PIECES[faction] ?? {});

interface Numbers { cost: number; moves?: number; strength?: number; ranged?: number; range?: number; cityStrength?: number; yields: Partial<Record<YieldKey, number>> }

function numbersOf(item: ProductionItem, faction: Faction, own: boolean): Numbers {
  if (item.kind === 'Unit') {
    const s = own ? unitStatsFor(faction, item.unit) : standardUnitStatsFor(faction, item.unit);
    return { cost: s.cost, moves: s.moves, strength: s.strength, ranged: s.ranged, range: s.range, yields: {} };
  }
  if (item.kind !== 'Building') return { cost: 0, yields: {} };
  const b = own ? buildingStatsFor(faction, item.building) : standardBuildingStatsFor(faction, item.building);
  return { cost: b.cost, cityStrength: b.cityStrength, yields: b.yields };
}

/** Lower is better for a price; higher for everything else. */
const better = (id: DiffId, now: number, was: number): boolean => (id === 'cost' ? now < was : now > was);

/** The standard item a faction's piece is measured against (so a caller can look up its quoted cost); null for an item that is no piece. */
export const pieceAgainst = (faction: Faction, item: ProductionItem): ProductionItem | null => PIECES[faction]?.[itemKey(item)]?.against ?? null;

/** The two prices a city quotes right now: `now` for the piece, `was` for the item it is measured against. */
export interface QuotedCosts { now: number; was: number }

/**
 * How `item` (a piece of `faction`) differs from the standard item it replaces; null when it replaces nothing.
 * `quoted` carries the two options' actual costs: the Spartan premium and the trained-Settler step change a city's price,
 * so a gap taken from the static tables disagreed with the two prices on screen (8.9.4 review).
 */
export function pieceComparison(faction: Faction, item: ProductionItem, quoted?: QuotedCosts | null): PieceComparison | null {
  const spec = PIECES[faction]?.[itemKey(item)];
  if (!spec) return null;
  const now = numbersOf(item, faction, true);
  const was = numbersOf(spec.against, faction, false);
  const diffs: PieceDiff[] = [];
  const consider = (id: DiffId, a: number | undefined, b: number | undefined) => {
    if (a === undefined || b === undefined || a === b) return;
    diffs.push({ id, now: a, was: b, tone: better(id, a, b) ? 'good' : 'bad' });
  };
  if (quoted) {
    if (quoted.now !== quoted.was) diffs.push({ id: 'cost', now: quoted.now, was: quoted.was, tone: quoted.now < quoted.was ? 'good' : 'bad' });
  } else consider('cost', now.cost, was.cost);
  consider('moves', now.moves, was.moves);
  consider('strength', now.strength, was.strength);
  consider('ranged', now.ranged, was.ranged);
  // A range belongs to a ranged number: only compare it when the standard item shoots too (a new bow reads as "+10 ranged").
  if ((was.ranged ?? 0) > 0) consider('range', now.range, was.range);
  for (const k of YIELD_KEYS) consider(k, now.yields[k] ?? 0, was.yields[k] ?? 0);
  consider('cityStrength', now.cityStrength, was.cityStrength);
  const rules = spec.rules ?? [];
  return {
    name: itemName(item, faction),
    against: itemName(spec.against),
    mode: spec.mode ?? 'replaces',
    diffs, rules,
    same: diffs.length === 0 && rules.length === 0,
  };
}

/** True for a replacement that is the same as what it replaces in every number and rule (a rename). A design flag, not a player-facing state. */
export const isIdentical = (c: PieceComparison): boolean => c.mode === 'replaces' && c.same;
/** True for a replacement that is worse in some number and better in none, with no better rule. A design flag. */
export const isStrictlyWorse = (c: PieceComparison): boolean =>
  c.mode === 'replaces' && c.diffs.some((d) => d.tone === 'bad') && !c.diffs.some((d) => d.tone === 'good') && !c.rules.some((r) => r.tone === 'good');

/** A rule sentence as a list item: no capital, no closing full stop. */
export const ruleClause = (r: PieceRule): string => r.text.replace(/\.$/, '').replace(/^./, (c) => c.toLowerCase());
const signed = (n: number): string => `${n > 0 ? '+' : '−'}${Math.abs(n)}`;
const signedYield = (n: number): string => (n === 0 ? '0' : signed(n));

/** The numbers' words: `value` as the piece shows it and `label`, in one place so every surface reads the same. */
function parts(d: PieceDiff): { value: string; label: string; was: string } {
  switch (d.id) {
    case 'cost': return { value: String(d.now), label: 'Production', was: String(d.was) };
    case 'moves': return { value: String(d.now), label: d.now === 1 ? 'move' : 'moves', was: String(d.was) };
    case 'strength': return { value: String(d.now), label: 'strength', was: String(d.was) };
    case 'ranged': return { value: String(d.now), label: 'ranged', was: String(d.was) };
    case 'range': return { value: String(d.now), label: 'range', was: String(d.was) };
    case 'cityStrength': return { value: `+${d.now}`, label: 'city strength', was: `+${d.was}` };
    default: return { value: signedYield(d.now), label: YIELD_LABEL[d.id], was: signedYield(d.was) };
  }
}

/** The standard item's number as the piece shows it: "120", "+10". */
export const diffWas = (d: PieceDiff): string => parts(d).was;

/** "+15 city strength (Walls: +10)", "120 Production (Walls: 100)": the piece's number and the standard item's. */
export function diffText(d: PieceDiff, against: string): string {
  const p = parts(d);
  return `${p.value} ${p.label} (${against}: ${p.was})`;
}

/** "+5 city strength", "costs 20 more Production", "−1 move": only the change, for a one-line summary. A price reads as a price ("costs"), never as a yield. */
export function deltaText(d: PieceDiff): string {
  const change = d.now - d.was;
  if (d.id === 'cost') return `costs ${Math.abs(change)} ${change > 0 ? 'more' : 'less'} Production`;
  const p = parts(d);
  const one = Math.abs(change) === 1;
  const label = d.id === 'moves' ? (one ? 'move' : 'moves') : p.label;
  return `${signed(change)} ${label}`;
}

/** "the Archer", but plural names (Walls) read without an article. */
export const the = (name: string): string => (name === 'Walls' ? name : `the ${name}`);
/** What to say about a piece that differs in nothing. */
export const sameText = (c: PieceComparison): string => `Same numbers as ${the(c.against)}.`;
/** What to say about a piece that stands beside the item rather than replacing it. */
export const addsText = (c: PieceComparison): string => `Not a replacement: a city can have both it and ${the(c.against)}.`;

/** The gains before the losses, a price last in each group, so a summary opens with what you get. */
export const inGoodFirstOrder = (diffs: PieceDiff[]): PieceDiff[] => {
  const group = (tone: Tone) => [...diffs.filter((d) => d.tone === tone && d.id !== 'cost'), ...diffs.filter((d) => d.tone === tone && d.id === 'cost')];
  return [...group('good'), ...group('bad')];
};

/** A piece that changes no number and only adds good rules: it reads "Same as the Library, plus ...", not "vs Library: ...". */
export const onlyAddsRules = (c: PieceComparison): boolean => c.mode === 'replaces' && c.diffs.length === 0 && c.rules.length > 0 && c.rules.every((r) => r.tone === 'good');

/** One line for a build card ("vs Walls: +5 city strength, +20 Production") or running text ("against the Archer: ..."). Whole text; the card colours the parts itself. */
export function summaryText(c: PieceComparison, word: 'vs' | 'against' = 'vs'): string {
  if (c.mode === 'adds') return addsText(c);
  const bits = [...inGoodFirstOrder(c.diffs).map(deltaText), ...c.rules.map(ruleClause)];
  if (onlyAddsRules(c)) return `same as ${the(c.against)}, plus ${bits.join(', ')}`;
  const lead = `${word} ${word === 'vs' ? c.against : the(c.against)}`;
  return bits.length ? `${lead}: ${bits.join(', ')}` : `${lead}: same numbers`;
}

/**
 * What the standard item itself does in the numbers the piece changed, as its own sentence: "A Forge gives +3 Production."
 * Never the difference, so a bare "+3" cannot mean the Forge's own number on one surface and the change on another. The piece's
 * rule sentences follow, each whole. A piece that changes no number, or only adds good rules, keeps `summaryText`'s wording.
 */
export function standardText(c: PieceComparison): string {
  if (c.mode === 'adds' || c.same || onlyAddsRules(c)) return summaryText(c, 'against').replace(/^./, (ch) => ch.toUpperCase());
  const plural = the(c.against) === c.against;
  const [gives, costs, has] = plural ? ['give', 'cost', 'have'] : ['gives', 'costs', 'has'];
  const bits: string[] = [];
  for (const d of c.diffs) {
    if (d.id === 'cost') bits.push(`${costs} ${d.was} Production`);
    else if (d.id === 'moves') bits.push(`${has} ${d.was} ${d.was === 1 ? 'move' : 'moves'}`);
    else if (d.id === 'strength') bits.push(`${has} ${d.was} strength`);
    else if (d.id === 'ranged') { if (d.was > 0) bits.push(`${has} ${d.was} ranged`); }
    else if (d.id === 'range') continue;
    else if (d.id === 'cityStrength') { if (d.was > 0) bits.push(`${gives} +${d.was} city strength`); }
    else if (d.was !== 0) bits.push(`${gives} ${signed(d.was)} ${YIELD_LABEL[d.id]}`);
  }
  const list = bits.length > 1 ? `${bits.slice(0, -1).join(', ')} and ${bits[bits.length - 1]}` : bits[0];
  const subject = plural ? c.against : `${/^[AEIOU]/.test(c.against) ? 'An' : 'A'} ${c.against}`;
  const standard = list ? [`${subject} ${list}.`] : [];
  return [...standard, ...c.rules.map((r) => r.text)].join(' ').replace(/\.$/, '');
}

/** Prose for the Guide and tooltips, which already state the piece's own numbers: the piece named, then only what changes against the standard item (gains first, so it reads better and worse). */
export function comparisonSentence(c: PieceComparison): string {
  if (c.mode === 'adds') return `${c.name}: ${addsText(c).replace(/^Not a replacement: a/, 'not a replacement, a')}`;
  if (c.same) return `${c.name}: ${sameText(c).replace(/^Same/, 'same')}`;
  const bits = [...inGoodFirstOrder(c.diffs).map(deltaText), ...c.rules.map(ruleClause)];
  return `${c.name} compared with ${the(c.against)}: ${bits.join(c.rules.length > 1 ? '; ' : ', ')}.`;
}

/** The faction's own pieces that have a comparison, as items (the faction sheet's piece first). */
export function comparedItems(faction: Faction): ProductionItem[] {
  return comparedPieces(faction).map((key): ProductionItem => key.startsWith('u:')
    ? { kind: 'Unit', unit: key.slice(2) as UnitKind }
    : { kind: 'Building', building: key.slice(2) as BuildingKind });
}
