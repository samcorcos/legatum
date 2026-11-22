import { eraName, eraNumeral } from '../ui/eraResearch';
import { BUILDING_STATS, FACTION_PIECES, GERMANY_FACTORY_NOTE, TECH_NAME, isGreatWork, itemDescription, itemIcon, itemKey, itemName, sameItem, uniquePieceGain } from '../data';
import { addsText, pieceComparison, standardText } from '../pieceCompare';
import { TECH_QUOTE } from '../techQuotes';
import { techCardUnlocks, techUnlockSummary } from '../ui/techCardUnlocks';
import type { CityView, Faction, PlayerView, ProductionItem, TechId, UnitKind } from '../types';

/** One unlock on the card: an item you can now build, or a tech effect with no item behind it. */
export interface UnlockTile {
  /** `itemKey` of the item; null for an effect. These are the keys the "new" set tracks. */
  key: string | null;
  name: string;
  /** What the component's painted miniature is looked up from (`unlockArt.ts` `unlockEntryArt`): the item (null for an effect), the tech and faction. */
  item: ProductionItem | null;
  tech: TechId;
  faction: Faction;
  icon: string;
  kind: 'Unit' | 'Building' | 'Wonder' | 'Project' | 'Improvement' | 'Effect';
  cost: number | null;
  /** Build time in the best city that can build it now (the city named by `show`, when it is that one). */
  turns: number | null;
  inCity: string | null;
  /** One plain sentence on what it does, from the rules data. Null for an effect, whose name already says it. */
  sentence: string | null;
  /** Why this faction cannot build it yet (core `locked_note`): the sentence says this instead, and the tile shows no price. */
  locked: string | null;
  upgradeFrom: string | null;
  /** What the technology's rule lines say about this unit (reach, guard, intercept, the supply it uses): shown under the sentence. */
  notes: string[];
  /** The units a merged upgrade rule sends to (empty for every other tile). */
  upgrades: { from: UnitKind; to: UnitKind }[];
}

export interface UnlockCard {
  tech: TechId;
  kicker: string;
  quote: string;
  icon: string;
  tiles: UnlockTile[];
  /** Where "Show ... in ..." goes: the first unlocked item some city can build now. Null: the primary is Continue. */
  show: { item: ProductionItem; itemName: string; city: number; cityName: string } | null;
  /** The research that carries on, with what it unlocks. */
  next: { name: string; turns: number | null; unlocks: string | null } | null;
  /** No research is set and some is available: the card says so and offers "Choose research". */
  chooseNext: boolean;
  /** What Return does: show the unlock in a city; else open the tech panel when research is idle; else just close. */
  primary: 'show' | 'research' | 'continue';
  /** Item keys this card unlocks: the entries the "new" marks follow. */
  keys: string[];
}

/** Plural-form names read without an article ("Build Walls", never "a Walls"). */
const PLURAL_NAMES = new Set(['Walls']);
/** "a University", "a European", "a One-Day Market": a vowel letter that sounds like "you" or "wuh" takes "a". */
const vowelSound = (name: string) => /^[AEIOU]/i.test(name) && !/^(uni|use|usu|eu|one\b)/i.test(name);
const withArticle = (name: string) => PLURAL_NAMES.has(name) ? name : `${vowelSound(name) ? 'an' : 'a'} ${name}`;
const firstSentence = (text: string) => text.split(/\.\s/)[0].replace(/\.$/, '');
/** A faction replacement's prose restates its name, price and tech; the sentence keeps only what it does. */
const stripReplacement = (text: string) => text.replace(/^[^:]+ replaces [^:]+:\s*/, '').replace(/^\d+ Production,\s*/, '').replace(/;\s*requires [^;.]+\.?$/, '').replace(/\b1 moves\b/, '1 move');

/** What a building or unit does, in one plain sentence, in the faction's own words and numbers. */
export function unlockSentence(item: ProductionItem, faction: Faction): string {
  const sentence = plainSentence(item, faction);
  // A faction piece also says how it differs from the standard item it replaces.
  const c = pieceComparison(faction, item);
  if (!c) return sentence;
  return c.mode === 'adds' ? `${sentence}. ${addsText(c).replace(/\.$/, '')}` : `${sentence}. ${standardText(c)}`;
}

function plainSentence(item: ProductionItem, faction: Faction): string {
  const name = itemName(item, faction);
  const description = itemDescription(item, faction);
  if (item.kind === 'Project' || item.kind === 'Improvement') return description;
  // A faction's unique piece is structured data (numbers and rule sentences), never prose to strip.
  const unique = FACTION_PIECES[faction];
  if (unique && sameItem(item, unique.item)) {
    const gain = uniquePieceGain(faction);
    if (item.kind === 'Unit') return `Train ${withArticle(name)} in any city: ${gain}`;
    const stats = BUILDING_STATS[item.building];
    if (stats.predecessor) {
      const from = itemName({ kind: 'Building', building: stats.predecessor }, faction);
      return `Upgrade ${withArticle(from)} to ${withArticle(name)}${stats.minPopulation ? ` in a city of ${stats.minPopulation} or more` : ''}: ${gain}`;
    }
    return `Build ${withArticle(name)} in any city: ${gain}`;
  }
  if (item.kind === 'Unit') return `Train ${withArticle(name)} in any city: ${stripReplacement(firstSentence(description))}`;
  if (isGreatWork(item.building)) {
    const rest = description.replace(/^World wonder: [^.]*\.\s*/, '');
    return `Build it on a chosen tile; its holder scores +2 VP.${rest && rest !== description ? ` ${rest.replace(/^Requires\b/, 'Needs').replace(/\bin this city\b/, 'in the same city')}` : ''}`;
  }
  const stats = BUILDING_STATS[item.building];
  // An upgrade's description is built from its fields ("+3 Production. Upgrades your Workshop. Needs 4 population."), so its first sentence is the gain.
  const upgrade = stats.predecessor;
  const plain = stripReplacement(description);
  let gain = firstSentence(plain);
  // A faction with no use for a yield keeps the rest of what the building does.
  if (!gain) gain = firstSentence(plain.replace(/^\.\s*/, '')).replace(/^No Science:/, 'no Science:');
  if (upgrade && faction === 'Germany' && item.building === 'Autoworks') {
    return `Add a Factory to a Ruhr Foundry city of ${stats.minPopulation} or more: ${gain}. ${GERMANY_FACTORY_NOTE.replace(/\.$/, '')}`;
  }
  if (upgrade) {
    const from = itemName({ kind: 'Building', building: stats.predecessor! }, faction);
    return `Upgrade ${withArticle(from)} to ${withArticle(name)}${stats.minPopulation ? ` in a city of ${stats.minPopulation} or more` : ''}: ${gain}`;
  }
  return `Build ${withArticle(name)} in any city: ${gain}`;
}

/** The own city that builds `item` soonest right now, or null when none can. */
function bestCity(view: PlayerView, item: ProductionItem): { city: CityView; turns: number | null; cost: number } | null {
  const found = view.cities.filter((c) => c.owner === view.me && c.detail).flatMap((city) => {
    const option = city.detail!.options.find((o) => sameItem(o.item, item));
    return option && !option.resource_blocker ? [{ city, turns: option.turns > 0 ? option.turns : null, cost: option.cost }] : [];
  });
  return found.sort((a, b) => (a.turns ?? 1e9) - (b.turns ?? 1e9) || Number(b.city.is_capital) - Number(a.city.is_capital) || a.city.id - b.city.id)[0] ?? null;
}

/** The unlock card for a technology the viewer just finished, from the view after the turn. */
export function unlockCardFor(view: PlayerView, tech: TechId): UnlockCard {
  const faction = view.players.find((p) => p.id === view.me)?.faction ?? 'India';
  const era = view.research.catalog.find((t) => t.tech === tech)?.era ?? view.research.era;
  const unlocks = techCardUnlocks(view.research, tech, faction);
  const items = view.research.unlocks.filter((u) => u.tech === tech);
  const tiles = unlocks.map((entry): UnlockTile => {
    const unlock = entry.unlock;
    if (!unlock) return { key: null, name: entry.label, item: null, tech, faction, icon: entry.icon, kind: 'Effect', cost: null, turns: null, inCity: null, sentence: null, locked: null, upgradeFrom: null, notes: [], upgrades: entry.upgrades };
    const best = bestCity(view, unlock.item);
    const locked = unlock.locked_note ?? null;
    return { key: itemKey(unlock.item), name: unlock.name, item: unlock.item, tech, faction, icon: itemIcon(unlock.item, faction), kind: unlock.item.kind, cost: locked ? null : best?.cost ?? unlock.cost,
      turns: best?.turns ?? null, inCity: best?.city.name ?? null, sentence: locked ?? unlockSentence(unlock.item, faction), locked, upgradeFrom: entry.upgradeFrom, notes: entry.notes, upgrades: [] };
  });
  const target = items.map((u) => ({ u, best: bestCity(view, u.item) })).find((x) => x.best);
  const current = view.research.current;
  const show = target?.best ? { item: target.u.item, itemName: target.u.name, city: target.best.city.id, cityName: target.best.city.name } : null;
  const chooseNext = !current && view.research.available.length > 0;
  const nextUnlocks = current ? techCardUnlocks(view.research, current, faction).map(techUnlockSummary) : [];
  return {
    tech, quote: TECH_QUOTE[tech], icon: 'yield-science',
    kicker: `Technology discovered · Era ${eraNumeral(era)} · ${eraName(era)}`,
    tiles, keys: tiles.flatMap((t) => t.key ? [t.key] : []),
    show, primary: show ? 'show' : chooseNext ? 'research' : 'continue',
    chooseNext,
    next: current ? { name: TECH_NAME[current], turns: view.research.turns_left, unlocks: nextUnlocks.length ? nextUnlocks.join(', ') : null } : null,
  };
}
