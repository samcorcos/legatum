import { BUILDING_NAME, UNIT_NAME, isGreatWork, itemKey, itemName, pluralUnit, buildingIconName, techCardEffectsFor, unitIconName } from '../data';
import type { BuildingKind, Faction, ResearchView, TechId, TechUnlock, UnitKind } from '../types';
import { YIELD_KEYS, YIELD_LABEL, type YieldKey } from './yieldKeys';
import { revealedByTech, unitNeed } from '../resources';
import { parseUpgradeDetail, parseUpgradeEffect } from './unlockCardModel';

const NAMED_ICONS: [string, string][] = [
  ...Object.entries(BUILDING_NAME).map(([kind, name]): [string, string] => [name, buildingIconName(kind as BuildingKind)]),
  ...Object.entries(UNIT_NAME).flatMap(([kind, name]): [string, string][] => [[name, unitIconName(kind as UnitKind)], [pluralUnit(name), unitIconName(kind as UnitKind)]]),
  ['Keshig', 'unit-keshig'], ['Keshigs', 'unit-keshig'],
];

/** An effect's glyph is the first building or unit its text names, else its yield, so a card
 * shows what changes (a Library, Scouts, cheaper tiles) rather than a generic mark. */
function effectIcon(label: string) {
  if (label.startsWith('Stealth aircraft use Silicon')) return 'resource-silicon';
  // Siege lines name Walls, Castles and Star Forts: the glyph is Siege's own.
  if (/\bSiege\b/.test(label)) return 'ui-siege';
  const named = NAMED_ICONS.map(([name, icon]) => ({ at: label.search(new RegExp(`\\b${name}`)), name, icon }))
    .filter((m) => m.at >= 0)
    .sort((a, b) => a.at - b.at || b.name.length - a.name.length)[0];
  if (named) return named.icon;
  if (/ruin/i.test(label)) return 'ruin';
  const yieldKey = YIELD_KEYS.find((k) => label.includes(YIELD_LABEL[k]));
  if (yieldKey) return `yield-${yieldKey}`;
  if (/upkeep/i.test(label)) return 'yield-gold';
  return 'effect-insight';
}

export type TechCardUnlock = {
  label: string;
  /** `itemKey` of the building or unit; null for an effect. The "new" marks follow this key. */
  key: string | null;
  icon: string;
  kind: 'Unit' | 'Building' | 'Wonder' | 'Project' | 'Improvement' | 'Effect';
  /** What the unlock gives, shown under its name: building yields, or a unit's strength. */
  yields: [YieldKey, number][];
  strength: string | null;
  /** Upgrade path folded into a unit unlocked by this same technology. */
  upgradeFrom: string | null;
  /** What a unit row adds after its strength, taken from the card's own rule lines (reach, guard, intercept, the supply it uses). */
  notes: string[];
  /** The units a merged upgrade rule row sends to (empty for every other row). */
  upgrades: { from: UnitKind; to: UnitKind }[];
  /** The view's unlock forecast behind the entry (cost, buy and upgrade price); null for an effect.
   * Its item's painting comes from `unlockArt.ts` (`unlockEntryArt`), looked up in the UI so this
   * module stays importable outside Vite. */
  unlock: TechUnlock | null;
  /** Why this faction cannot build it yet (core `locked_note`): said in place of the build line, price and forecasts. */
  locked: string | null;
  /** A world wonder: it leads the card. */
  wonder: boolean;
  /** The technology that gives it (the effect paintings are per technology). */
  tech: TechId;
};

/** Compact wording for places where an unlock is listed on one line. */
export const techUnlockSummary = (entry: TechCardUnlock) =>
  `${entry.label}${entry.upgradeFrom ? ` (from ${entry.upgradeFrom})` : ''}`;

/** What `tech` gives this faction. The view's unlocks are already faction-accurate (core
 * `research::tech_unlocks`: replacements under their own names, nothing the faction can't use),
 * and the effects come from the generated card effects (core
 * `data::tech_card_effects_for`), so both clients render the same card. */
export function techCardUnlocks(research: ResearchView, tech: TechId, faction: Faction): TechCardUnlock[] {
  const items = research.unlocks.filter((unlock) => unlock.tech === tech).map((unlock): TechCardUnlock => ({
    label: unlock.name,
    key: itemKey(unlock.item),
    icon: unlock.item.kind === 'Unit'
      ? unitIconName(unlock.item.unit, faction)
      : unlock.item.kind === 'Building'
        ? buildingIconName(unlock.item.building, faction)
        : 'yield-production',
    kind: unlock.item.kind,
    // The Nile: Egypt has no Production, so a building's or wonder's Production is not shown to it.
    yields: YIELD_KEYS.filter((k) => unlock.yields[k] && !(k === 'production' && faction === 'Egypt')).map((k) => [k, unlock.yields[k]]),
    unlock,
    strength: unlock.unit_ranged ? `${unlock.unit_ranged} ranged` : unlock.unit_strength ? `${unlock.unit_strength} strength` : unlock.city_strength ? `+${unlock.city_strength} city strength` : null,
    upgradeFrom: null,
    notes: [],
    upgrades: [],
    locked: unlock.locked_note ?? null,
    wonder: unlock.item.kind === 'Building' && isGreatWork(unlock.item.building),
    tech,
  }));
  const effect = (label: string, extra: Partial<TechCardUnlock> = {}): TechCardUnlock =>
    ({ label, key: null, icon: effectIcon(label), kind: 'Effect', yields: [], strength: null, upgradeFrom: null, notes: [], upgrades: [], unlock: null, locked: null, wonder: false, tech, ...extra });
  const unitRow = (unit: UnitKind) => items.find((item) => item.unlock?.item.kind === 'Unit' && item.unlock.item.unit === unit);
  const effects: TechCardUnlock[] = [];
  const lines = techCardEffectsFor(faction, tech);
  // A rule that only describes a unit this card unlocks belongs in that unit's row, so the card never states a unit's stats twice.
  for (const label of lines) {
    const detail = parseUpgradeDetail(label);
    const target = detail && unitRow(detail.unit);
    if (detail && target) {
      const note = unitNote(detail.clauses);
      if (note) target.notes.push(note);
      // The upgrade path reads "From Fighter" on the unit's own row, like "From Tank" (user, 2026-10-25: never a separate "can upgrade" row).
      target.upgradeFrom = itemName({ kind: 'Unit', unit: detail.fromUnit }, faction);
    }
  }
  for (const label of lines) {
    const upgrade = parseUpgradeEffect(label);
    const target = upgrade && unitRow(upgrade.to);
    if (target && upgrade) {
      target.upgradeFrom = itemName({ kind: 'Unit', unit: upgrade.from }, faction) + (/ and lose the bow$/.test(label) ? ' (loses the bow)' : '');
      continue;
    }
    const detail = parseUpgradeDetail(label);
    if (detail && unitRow(detail.unit)) continue;
    // "Stealth aircraft use Silicon instead of Oil": the supply is on each Stealth unit's row.
    const supply = /^(\w+) .+? use (Iron|Niter|Coal|Oil|Silicon)\b/.exec(label);
    const users = supply ? items.filter((item) => item.unlock?.item.kind === 'Unit' && unitNeed(item.unlock.item.unit) === supply[2] && item.label.startsWith(`${supply[1]} `)) : [];
    if (supply && users.length) {
      for (const row of users) row.notes.push(`uses ${supply[2]}`);
      continue;
    }
    effects.push(effect(label));
  }
  const reveals = revealedByTech(tech).map((resource): TechCardUnlock => ({
    label: `Reveals ${resource} on the map${faction === 'Germany' && resource === 'Iron' ? ' · your deposits supply twice as much' : ''}`, key: null, icon: `resource-${resource.toLowerCase()}`, kind: 'Effect',
    yields: [], strength: null, upgradeFrom: null, notes: [], upgrades: [], unlock: null, locked: null, wonder: false, tech,
  }));
  return [...items, ...effects, ...reveals].map((entry, index) => ({ entry, index }))
    .sort((a, b) => rank(a.entry) - rank(b.entry) || a.index - b.index).map(({ entry }) => entry);
}

/** An upgrade line's clauses as a unit subline: "strikes 7 tiles, guards 5", "strikes 10 tiles; only Stealth Fighters stop it". */
function unitNote(clauses: string[]): string {
  const reach: [string, string][] = [];
  const rest: string[] = [];
  for (const clause of clauses) {
    const strike = /^strike up to (\d+) tiles away$/.exec(clause);
    const guard = /^guard (\d+) tiles around (?:their|its) city$/.exec(clause);
    const stop = /^only (.+) intercept (?:them|it)$/.exec(clause);
    if (strike) reach.push(['strikes', strike[1]]);
    else if (guard) reach.push(['guards', guard[1]]);
    else rest.push(stop ? `only ${stop[1]} stop it` : clause);
  }
  const reachText = reach.map(([verb, n], i) => `${verb} ${n}${i === 0 ? ' tiles' : ''}`).join(', ');
  return [reachText, ...rest].filter(Boolean).join('; ');
}

/** The card leads with what is most worth wanting: a wonder, then a building, a unit, an improvement, an effect;
 * a unit-upgrade line ("Spearmen can upgrade to Phalangites") comes last. */
function rank(entry: TechCardUnlock) {
  if (entry.wonder) return 0;
  if (entry.kind === 'Building') return 1;
  if (entry.kind === 'Unit') return 2;
  // A resource reveal precedes bonuses to its deposits: first find Oil, then Oil Wells gain Gold.
  if (/^Reveals (Iron|Niter|Coal|Oil|Silicon) on the map/.test(entry.label)) return 3;
  if (entry.kind === 'Improvement' || entry.upgrades.length) return 3;
  return /can upgrade to/.test(entry.label) ? 5 : 4;
}
