// What an unlock's hovercard says, as plain data (so it is testable without a DOM): units with the class
// triangle and a comparison with the unit they replace, buildings with what they give and why you want them,
// effects with a real icon (an upgrade shows both miniatures). Numbers come from the rules data and the view's
// unlock forecast, never from prose here; the flavour lines live in unlockLore.ts.
import {
  BUILDING_STATS, CLASS_BONUS_PERCENT, MOUNTED_CITY_PENALTY_PERCENT, RULES, WALLS_FIRE_RULE, CLASS_NAME, KESHIG_SHOT_LIMIT, MOUNTED_MOVES, UNIT_NAME, UNIT_STATS, attacksInMelee, buildingStatsFor,
  familyNames, itemName, pluralUnit, unitClass, unitEraTier, unitStatsFor,
} from '../data';
import { pieceComparison, type PieceComparison } from '../pieceCompare';
import type { BuildingKind, Faction, UnitClass, UnitKind } from '../types';
import { eraName } from './eraResearch';
import { BUILDING_LORE, BUILDING_WHY, PIECE_LORE, UNIT_LORE } from './unlockLore';
import { YIELD_KEYS, type YieldKey } from './yieldKeys';
import type { TechCardUnlock } from './techCardUnlocks';

/** One compared number: "26" against "20", with the change in plain terms. */
export interface HoverStat {
  id: 'strength' | 'ranged' | 'moves' | 'range';
  label: string;
  value: number;
  /** The same stat on the unit this one replaces; absent when there is nothing to compare. */
  from?: number;
  /** "+30%" for a stronger unit; null when unchanged or not comparable. */
  delta: { text: string; tone: 'good' | 'bad' | 'neutral' } | null;
}

export interface UnitMatchup {
  /** Plain words: "Holds off mounted units (+8 strength against them)." */
  line: string;
  beats: string[];
  weakTo: string[];
}

export interface UnitHover {
  type: 'unit';
  kind: UnitKind;
  name: string;
  /** "Melee line", "Mounted", ... ; null for a Scout or a Settler. */
  classLabel: string | null;
  era: string;
  flavour: string;
  matchup: UnitMatchup | null;
  /** The unit's own rule line when it is no part of the triangle (siege, air, scout, settler). */
  rule: string | null;
  stats: HoverStat[];
  /** The unit this one upgrades from, with the Gold the upgrade costs. */
  replaces: { kind: UnitKind; name: string } | null;
  upgradeCost: number | null;
  buyCost: number | null;
  cost: number;
  /** Why this faction cannot build it yet (core `locked_note`): the card says it in place of the Build and Buy prices. */
  locked: string | null;
  /** How this faction piece differs from the standard unit it replaces; null for an ordinary unit. */
  piece: PieceComparison | null;
}

export interface BuildingHover {
  type: 'building';
  kind: BuildingKind;
  name: string;
  tag: string;
  flavour: string;
  why: string;
  yields: [YieldKey, number][];
  cityStrength: number;
  /** The building it upgrades, by name. */
  upgradesFrom: string | null;
  minPopulation: number | null;
  cost: number;
  buyCost: number | null;
  locked: string | null;
  rule: string;
  /** How this faction piece differs from the standard building it replaces; null for an ordinary building. */
  piece: PieceComparison | null;
}

export interface EffectHover {
  type: 'effect';
  text: string;
  icon: string;
  /** What it does for you, one plain sentence. */
  why: string;
  /** "Spearmen can upgrade to Phalangites": the two units to draw side by side. */
  upgrade: { from: { kind: UnitKind; name: string }; to: { kind: UnitKind; name: string } } | null;
}

export type UnlockHover = UnitHover | BuildingHover | EffectHover;

const MATCHUP: Record<UnitClass, { beats: UnitClass[]; weakTo: UnitClass[] }> = {
  Melee: { beats: ['Mounted'], weakTo: ['Ranged'] },
  Ranged: { beats: ['Melee'], weakTo: ['Mounted'] },
  Mounted: { beats: ['Ranged'], weakTo: ['Melee'] },
  Air: { beats: [], weakTo: [] },
};
const WORDS: Record<UnitClass, string> = { Melee: 'the melee line', Ranged: 'ranged units', Mounted: 'mounted units', Air: 'aircraft' };

/** `bow`: a mounted unit that also shoots (the Keshig). Its shot fights as ranged, so the melee line gets no bonus against its arrows. */
export function unitMatchup(cls: UnitClass | null, bow = false, modern = false): UnitMatchup | null {
  if (!cls || cls === 'Air') return null;
  // The triangle ends at the World War era (core `combat::TRIANGLE_END_ERA_TIER`): such a unit has no class bonus, and no class beats it or is beaten by it.
  if (modern) return { line: 'No class bonuses from the World War era: it differs by strength, cost and moves.', beats: [], weakTo: [] };
  const { beats, weakTo } = MATCHUP[cls];
  const b = `+${CLASS_BONUS_PERCENT}% strength`;
  const line = cls === 'Melee' ? `Holds off mounted units (${b} against them). Ranged units shoot it down.`
    : cls === 'Ranged' ? `Shoots the melee line from range with no reply. Mounted units ride it down in a charge (${b} against it), but get no bonus under its fire.`
    : `Rides down ranged units in a charge (${b} against them, none under their fire), ${MOUNTED_MOVES} moves; −${MOUNTED_CITY_PENALTY_PERCENT}% charging a city. The melee line holds it off${bow ? ', but gets no bonus against its arrows' : ''}.`;
  return { line, beats: beats.map((c) => WORDS[c]), weakTo: weakTo.map((c) => WORDS[c]) };
}

/** The unit `kind` upgrades from (the one whose upgrade goes to it), if any. */
export function upgradesFromUnit(kind: UnitKind): UnitKind | null {
  return (Object.keys(UNIT_STATS) as UnitKind[]).find((k) => UNIT_STATS[k].upgradesTo === kind) ?? null;
}

const deltaPercent = (now: number, was: number): HoverStat['delta'] => {
  if (!was || now === was) return null;
  const pct = Math.round(((now - was) * 100) / was);
  return { text: `${pct > 0 ? '+' : '−'}${Math.abs(pct)}%`, tone: pct > 0 ? 'good' : 'bad' };
};

function unitHover(entry: TechCardUnlock, unit: UnitKind, faction: Faction): UnitHover {
  const u = entry.unlock;
  const name = entry.label;
  const now = unitStatsFor(faction, unit);
  const prevKind = upgradesFromUnit(unit);
  const prev = prevKind ? unitStatsFor(faction, prevKind) : null;
  const cls = unitClass(unit);
  const ranged = now.ranged > 0;
  const sameBasis = !!prev && (prev.ranged > 0) === ranged;
  const stats: HoverStat[] = [];
  // A shooter that also charges (the Keshig) shows its melee strength as well as its bow.
  const bow = ranged && attacksInMelee(unit);
  if (bow) stats.push({ id: 'strength', label: 'Melee strength', value: now.strength, from: undefined, delta: null });
  if (now.military && (ranged || now.strength)) {
    const value = ranged ? now.ranged : now.strength;
    const from = sameBasis ? (ranged ? prev!.ranged : prev!.strength) : undefined;
    stats.push({ id: ranged ? 'ranged' : 'strength', label: ranged ? 'Ranged strength' : 'Strength', value, from, delta: from === undefined ? null : deltaPercent(value, from) });
  }
  if (ranged && now.range) stats.push({ id: 'range', label: 'Range', value: now.range, from: prev?.range ? prev.range : undefined, delta: null });
  if (now.military || now.moves) {
    const from = prev?.moves;
    const moves = u?.unit_moves ?? now.moves;
    stats.push({ id: 'moves', label: 'Moves', value: moves, from, delta: from === undefined || from === moves ? null : { text: `${moves > from ? '+' : '−'}${Math.abs(moves - from)}`, tone: moves > from ? 'good' : 'bad' } });
  }
  const triangle = cls === 'Melee' || cls === 'Ranged' || cls === 'Mounted';
  return {
    type: 'unit', kind: unit, name,
    classLabel: cls ? CLASS_NAME[cls] : null,
    era: eraName(unitEraTier(unit)),
    flavour: PIECE_LORE[name] ?? UNIT_LORE[unit],
    matchup: unitMatchup(cls, bow, unitEraTier(unit) >= RULES.triangleEndsAtEraTier),
    rule: bow ? KESHIG_SHOT_LIMIT : !cls || !triangle ? UNIT_STATS[unit].blurb : null,
    stats,
    replaces: prevKind ? { kind: prevKind, name: itemName({ kind: 'Unit', unit: prevKind }, faction) } : null,
    upgradeCost: u?.upgrade_cost ?? null,
    buyCost: u?.buy_cost ?? null,
    cost: u?.cost ?? now.cost,
    locked: entry.locked,
    piece: pieceComparison(faction, { kind: 'Unit', unit }),
  };
}

function buildingHover(entry: TechCardUnlock, building: BuildingKind, faction: Faction): BuildingHover {
  const u = entry.unlock;
  const stats = buildingStatsFor(faction, building);
  const base = BUILDING_STATS[building];
  const yields = YIELD_KEYS.filter((k) => stats.yields[k]).map((k): [YieldKey, number] => [k, stats.yields[k] as number]);
  const family = BUILDING_STATS[building].predecessor ?? null;
  return {
    type: 'building', kind: building, name: entry.label,
    tag: stats.warBuilding ? 'War building' : 'Building',
    flavour: PIECE_LORE[entry.label] ?? BUILDING_LORE[building],
    why: BUILDING_WHY[building],
    yields,
    cityStrength: stats.cityStrength,
    upgradesFrom: family ? itemName({ kind: 'Building', building: family }, faction) : null,
    minPopulation: u?.min_population ?? stats.minPopulation ?? null,
    cost: u?.cost ?? stats.cost,
    buyCost: u?.buy_cost ?? null,
    locked: entry.locked,
    // Walls and the piece that replaces them differ in strength: the line carries the faction's own number.
    rule: building === 'Walls' ? `+${stats.cityStrength} city strength; ${WALLS_FIRE_RULE}` : base.blurb,
    piece: pieceComparison(faction, { kind: 'Building', building }),
  };
}

// ---- Effects ----------------------------------------------------------------------------------

const UNIT_BY_WORD = (() => {
  const map = new Map<string, UnitKind>();
  for (const kind of Object.keys(UNIT_NAME) as UnitKind[]) {
    map.set(UNIT_NAME[kind], kind);
    map.set(pluralUnit(UNIT_NAME[kind]), kind);
  }
  map.set('Keshigs', 'Cataphract');
  return map;
})();

/** "Spearmen can upgrade to Phalangites": both units named, so both can be drawn. */
export function parseUpgradeEffect(text: string): { from: UnitKind; to: UnitKind } | null {
  const m = /^(.+?) can upgrade to (.+?)( and lose the bow)?$/.exec(text);
  const from = m && UNIT_BY_WORD.get(m[1]);
  const to = m && UNIT_BY_WORD.get(m[2]);
  return from && to ? { from, to } : null;
}

/** An upgrade line that carries the new unit's own rules: "Fighters can upgrade to Stealth Fighters: strike up to 7 tiles away; ..."
 * (core `data::tech_card_effects`). `from` and `to` are the lines' words ("Fighters"), `unit` the unit that gets the clauses. */
export function parseUpgradeDetail(text: string): { from: string; to: string; fromUnit: UnitKind; unit: UnitKind; clauses: string[] } | null {
  const m = /^(.+?) can upgrade to ([^:]+): (.+)$/.exec(text);
  const fromUnit = m && UNIT_BY_WORD.get(m[1]);
  const unit = m && UNIT_BY_WORD.get(m[2]);
  return m && fromUnit && unit ? { from: m[1], to: m[2], fromUnit, unit, clauses: m[3].split('; ') } : null;
}

const EFFECT_WHY: [RegExp, string][] = [
  [/^\+(\d+) (Food|Production|Gold|Science) in every city/, 'Every city you own gets it at once, however many you have: it rewards a wide empire.'],
  [/claims its best (?:available ring-2 tile|tile 2 tiles out)/, 'Existing cities gain a tile now; later cities gain one when founded.'],
  [/Upgrade Gold costs/, 'Every unit upgrade costs less Gold, so an old army modernises cheaply.'],
  [/Units that start a turn on a road get/, 'Armies on roads cover much more ground each turn.'],
  [/Each (.+) gains \+(\d+)/, 'A bigger yield from a building you already have: no new construction.'],
  [/(?:per Library family|for each Library, University)/, 'Every {Library} you own adds more Science.'],
  [/Farms and Pastures upgrade/, 'Your existing Farms and Pastures improve by themselves: more Food, no orders.'],
  [/Mines upgrade|Quarries upgrade/, 'Your existing improved tiles gain Production by themselves: no orders.'],
  [/(?:per Market family|for each Market and Bank)/, 'Every {Market} you own adds more Gold.'],
  [/(?:per Granary family|for each Granary and Mill)/, 'Every {Granary} you own adds more Food.'],
  [/(?:per Workshop family|for each Workshop, )/, 'Every {Workshop} you own adds more Production.'],
  [/^Factories \+2 Production$/, 'Each supplied Factory adds 2 Production.'],
  [/Oil Wells/, 'Your Oil Wells add Gold without being rebuilt.'],
  [/Roadworks link/, 'Your existing Roadworks reach more cities each turn.'],
];

function effectHover(entry: TechCardUnlock, faction: Faction): EffectHover {
  const text = entry.label;
  const up = parseUpgradeEffect(text);
  const why = up
    ? `Your ${pluralUnit(itemName({ kind: 'Unit', unit: up.from }, faction))} can be upgraded in your territory for Gold, keeping their experience.${/ and lose the bow$/.test(text) ? ' An upgraded Keshig loses its bow.' : ''}`
    : entry.upgrades.length
      ? `Your ${entry.upgrades.map((u) => pluralUnit(itemName({ kind: 'Unit', unit: u.from }, faction))).join(' and ')} can be upgraded in your territory for Gold, keeping their experience.`
      : (EFFECT_WHY.find(([re]) => re.test(text))?.[1] ?? 'A permanent bonus that applies from the turn after the research.').replace(/\{(\w+)\}/, (_, family: string) => familyNames(faction, family));
  return {
    type: 'effect', text, icon: entry.icon, why,
    upgrade: up ? {
      from: { kind: up.from, name: itemName({ kind: 'Unit', unit: up.from }, faction) },
      to: { kind: up.to, name: itemName({ kind: 'Unit', unit: up.to }, faction) },
    } : null,
  };
}

/** The hovercard for one unlock entry of a tech card. */
export function unlockHover(entry: TechCardUnlock, faction: Faction): UnlockHover {
  const item = entry.unlock?.item;
  if (item?.kind === 'Unit') return unitHover(entry, item.unit, faction);
  if (item?.kind === 'Building') return buildingHover(entry, item.building, faction);
  return effectHover(entry, faction);
}
