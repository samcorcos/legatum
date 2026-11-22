// TI4-style faction sheet content for the full-screen faction picker (ui/FactionPicker.tsx).
// Mechanics numbers here must match .claude/skills/legatum-rules/references/game-rules.md and the
// FACTION_INFO entries in data.ts; where the exact wording already exists there (motto, goal, the
// unique piece's stat line), this file points at it instead of retyping it, so a rule has one
// wording. Ratings and the lore paragraph are new presentation, not duplicates of anything.
import {
  FLOWER_WARS, AZTEC_NO_HORSES, BUILDING_STATS, INDIA_CAPITAL_RADIUS, INDIA_NETWORK_PERCENT_PER_CITY, INDIA_NETWORK_RADIUS, INDIA_YIELD_PERCENT_PER_HEX,
  GERMANY_PROJECT_GOLD_PERCENT, GERMANY_PROJECT_SCIENCE_PERCENT, EGYPT_NILE_BUILD_PERCENT,
  KESHIG_KIND, RULES, SPECIAL_UNIT, TECH_NAME, UNIT_NAME, UNIT_STATS, specialUnitEdge, specialUnitName, companyRule, THIN_LEVIES, factionGoalVp, itemName, swissInterestRule, swissTollRule, SWISS_CHEAP_BUYS, SWISS_CAPTURE_RULE, unitStatsFor,
} from './data';
import { pieceAgainst } from './pieceCompare';
import type { BuildingKind, Faction, ProductionItem, TechId } from './types';

/** Tiles from the capital at which an Indian city's Food, Gold and Science reach zero. */
const indiaZeroAt = INDIA_CAPITAL_RADIUS + 100 / INDIA_YIELD_PERCENT_PER_HEX;
/** Germany's Industry project shares in words (half, a quarter), falling back to a percentage. */
const share = (percent: number) => ({ 50: 'half', 25: 'a quarter' } as Record<number, string>)[percent] ?? `${percent}%`;
const keshig = unitStatsFor('Mongols', KESHIG_KIND);
/** The row for a faction's special unit: what it replaces and the edge that sets it apart (the Guide prints it; the sheet shows the unit's card instead). */
const specialRow = (faction: Faction, icon: string): FactionPair =>
  ({ strength: `${specialUnitName(faction)} replaces the ${UNIT_NAME[SPECIAL_UNIT[faction]]}: ${specialUnitEdge(faction)}.`, strengthIcon: icon, onCard: true });
const cataphractCost = UNIT_STATS.Cataphract.cost;

interface FactionPair {
  /** The strength card (green): what the faction can do that others can't. */
  strength?: string;
  /** The cost-column rule (red). A drawback that only restates a strength is not a cost. */
  price?: string;
  /** Icon-set names for the medallions on this row (src/assets/icons). */
  strengthIcon?: string;
  priceIcon?: string;
  /** The row states a special unit or building that the faction sheet shows as a painted card; the Guide still prints it. */
  onCard?: true;
}

export interface FactionSheet {
  /** The pair lines already state the unique piece's numbers and what it is compared with, so the Guide's card does not say them again. */
  piecePairs?: boolean;
  /** Three-word identity shown on the gallery tile. */
  identity: string;
  /** An early design: playable, but with no faction goal yet (Egypt and Rome; marked on the tile and sheet). */
  early?: true;
  /** 2-3 sentence lore paragraph in the existing voice. */
  lore: string;
  /** 1-5 pips each. */
  ratings: { aggression: number; economy: number; complexity: number };
  /** 2-4 strength/price rows. */
  pairs: FactionPair[];
  /** The one weakness a rival can use that no price card already states. */
  lose?: string;
  /** What a rival does about it; the sheet leaves the line out for a faction that has none yet. */
  counteredBy?: string;
}

/** A special unit or building shown as a painted card on the sheet. */
export interface SheetCard {
  item: ProductionItem;
  name: string;
  /** The standard item it replaces; null for one only this faction can build. */
  replaces: string | null;
  /** The technology it needs and any rule its numbers cannot show. */
  text: string;
}

/** The buildings only a faction can build, each with painted art: its replacements (core `data::building_name_for`) and its goal and war buildings. */
const SPECIAL_BUILDINGS: Partial<Record<Faction, BuildingKind[]>> = {
  Switzerland: ['Walls'],
  India: ['Market', 'ImperialDiwan'],
  Germany: ['MachineShop'],
  Egypt: ['Granary'],
  Rome: ['Walls'],
  Mongols: ['CourierHall'],
  Sparta: ['WarShrine'],
  Athens: ['Laboratory'],
};

/** Rules a card must state because its numbers cannot: the Agoge's levels (the Yam and Diwan are told by the goal block). */
const BUILDING_RULE: Partial<Record<BuildingKind, string>> = { WarShrine: BUILDING_STATS.WarShrine.summary };

const requiresText = (tech: TechId | null): string[] => tech ? [`Requires ${TECH_NAME[tech]}.`] : [];

const sheetCard = (faction: Faction, item: ProductionItem, text: string[]): SheetCard => {
  const against = pieceAgainst(faction, item);
  return { item, name: itemName(item, faction), replaces: against ? itemName(against) : null, text: text.join(' ') };
};

/** The cards under a faction's traits: its special unit first, then its special buildings (none for most). */
export function sheetCards(faction: Faction): SheetCard[] {
  const unit = SPECIAL_UNIT[faction];
  const cards = [sheetCard(faction, { kind: 'Unit', unit }, requiresText(UNIT_STATS[unit].requires))];
  for (const building of SPECIAL_BUILDINGS[faction] ?? []) {
    const rule = BUILDING_RULE[building];
    cards.push(sheetCard(faction, { kind: 'Building', building }, [...requiresText(BUILDING_STATS[building].requires), ...(rule ? [`${rule}.`] : [])]));
  }
  return cards;
}

export const FACTION_SHEETS: Record<Faction, FactionSheet> = {
  Egypt: {
    identity: 'The river builds everything',
    early: true,
    lore: 'The Nile flood fed the builders of Egypt. In play, each city chooses between building with its Food and growing with it.',
    ratings: { aggression: 2, economy: 5, complexity: 3 },
    pairs: [
      { strength: `Egyptian cities get double Food. Food beyond what their people eat builds whatever the city is working on, units and Settlers included${EGYPT_NILE_BUILD_PERCENT === 100 ? '' : `: ${EGYPT_NILE_BUILD_PERCENT} progress for every 100 Food`}.`, price: "Egypt has no Production. A city that is building doesn't grow: switch it to Grow to grow it.", strengthIcon: 'yield-food', priceIcon: 'yield-production' },
      { strength: 'Royal Granary replaces the Granary and gives +3 Food total.', strengthIcon: 'building-royalgranary', onCard: true },
      specialRow('Egypt', 'unit-medjay'),
    ],
    counteredBy: 'Strike while its cities are busy growing, and take its Food-rich sites.',
  },
  Rome: {
    identity: 'Veterans found cities',
    early: true,
    lore: 'Roman veterans received land after military service. In play, each new city costs a seasoned soldier, leaving fewer defenders in the field.',
    ratings: { aggression: 4, economy: 2, complexity: 3 },
    pairs: [
      { strength: 'A full-health melee veteran (level 1 or higher) can found a city; the unit is consumed.', price: 'Rome cannot train or buy Settlers.', strengthIcon: 'faction-rome', priceIcon: 'unit-settler' },
      specialRow('Rome', 'unit-legionary'),
      { strength: 'Castrum replaces Walls. Upgrading a melee unit in its city restores 50 HP, up to full health, once per upgrade order.', strengthIcon: 'building-castrum', onCard: true },
    ],
    counteredBy: 'Wound a founder before the city appears, claim the site first, or attack the army left behind.',
  },
  Aztecs: {
    identity: 'War feeds cities',
    lore: 'From Tenochtitlan, a city of canals and causeways, the Mexica fought to take captives. At the New Fire ceremony, torches carried a renewed flame to the cities.',
    ratings: { aggression: 5, economy: 3, complexity: 2 },
    pairs: [{ strength: FLOWER_WARS, price: AZTEC_NO_HORSES + '.', strengthIcon: 'milestone-twenty-population', priceIcon: 'unit-horseman' }, specialRow('Aztecs', 'unit-eaglewarrior')],
    lose: 'Ranged armies and Walls deny you the melee victories that grow your cities.',
    counteredBy: 'Keep your soldiers alive; contest the smaller cities after the rite.',
  },
  Switzerland: {
    identity: 'Alpine stewards of coin',
    lore: 'Swiss communities guarded Alpine passes and later developed strong traditions of banking and public finance. In play, that wealth buys buildings and units cheaply but is exposed when a city falls.',
    ratings: { aggression: 1, economy: 5, complexity: 3 },
    pairs: [
      { strength: `${swissTollRule()} ${swissInterestRule()}`, price: 'Military unit upkeep is triple: 3 Gold a turn instead of 1.', strengthIcon: 'yield-gold', priceIcon: 'unit-soldier' },
      { strength: SWISS_CHEAP_BUYS, price: SWISS_CAPTURE_RULE, strengthIcon: 'building-market', priceIcon: 'city' },
      specialRow('Switzerland', 'unit-reislaufer'),
    ],
    counteredBy: 'Capture their cities to take a share of their treasury.',
  },
  India: {
    identity: 'The regional courts',
    lore: 'Akbar ruled a populous Mughal empire through provincial courts and a central treasury. Compact settlements grow quickly, but distant provinces and local revenues resist the centre.',
    ratings: { aggression: 2, economy: 4, complexity: 3 },
    pairs: [
      { strength: `A city gains ${INDIA_NETWORK_PERCENT_PER_CITY}% Food and Production for every other Indian city within ${INDIA_NETWORK_RADIUS} tiles.`, price: `Corruption cuts all four yields by ${INDIA_YIELD_PERCENT_PER_HEX}% per tile beyond ${INDIA_CAPITAL_RADIUS} from the capital. At ${indiaZeroAt} tiles its Food, Gold and Science reach zero, though Production never falls below 3.`, strengthIcon: 'city', priceIcon: 'capital' },
      { strength: 'Cities can be founded 3 tiles apart, closer than anyone else. Every city gets +1 Food.', price: 'Non-capital cities send no Gold to the treasury until they have a Revenue Office, then half the Gold they make.', strengthIcon: 'yield-food', priceIcon: 'yield-gold' },
      { strength: `Revenue Office replaces the Market and keeps +3 Gold. Build one in every owned city to unlock the Imperial Diwan; whoever holds its city scores ${factionGoalVp('India')} VP.`, strengthIcon: 'building-market-india', onCard: true },
      specialRow('India', 'unit-warelephant'),
    ],
    counteredBy: 'Capture the Imperial Diwan to take its points. Contest the compact city network.',
  },
  Byzantium: {
    identity: 'Armies for hire',
    lore: 'The eastern Roman Empire held Constantinople for a thousand years. When its thin professional army could not win, it opened the treasury and hired other empires’ armies.',
    ratings: { aggression: 2, economy: 3, complexity: 3 },
    pairs: [
      { strength: companyRule(), price: THIN_LEVIES, strengthIcon: 'fortify', priceIcon: 'unit-soldier' },
    ],
    counteredBy: 'Outlast every Company: its countdown is public, its upkeep drains the treasury, and an unpaid Company turns barbarian.',
  },
  Germany: {
    identity: 'Industry shapes the land',
    lore: 'Nineteenth-century German-speaking lands became centres of engineering, coal and steel. Their workshops and foundries draw workers from farms to drive industry.',
    ratings: { aggression: 2, economy: 5, complexity: 3 },
    piecePairs: true,
    pairs: [
      { strength: 'Ruhr Foundry: +6 Production (a Forge gives +3).', strengthIcon: 'building-foundry', onCard: true },
      { price: 'Each Production building in a German city costs 1 Food.', priceIcon: 'yield-food' },
      { strength: 'Your strategic deposits supply twice as much.', strengthIcon: 'resource-iron' },
      { strength: `A city can work on an Industry project instead of building: each turn it turns ${share(GERMANY_PROJECT_GOLD_PERCENT)} of its Production into Gold, or ${share(GERMANY_PROJECT_SCIENCE_PERCENT)} into Science.`, strengthIcon: 'yield-gold' },
      specialRow('Germany', 'unit-panzer'),
    ],
    counteredBy: 'Attack early, before their Production builds up.',
  },
  Celts: {
    identity: 'Communities of the land',
    lore: 'Celtic-speaking peoples formed many communities across Europe, not one state. In play, warbands raid and burn rather than hold what they take.',
    ratings: { aggression: 4, economy: 3, complexity: 4 },
    pairs: [
      { strength: `A sack carries off ${RULES.celtsSackGoldPerPopulation} Gold per population and raises a warband camp near the ruins.`, price: "Celts can't keep cities they capture: they sack and burn them. A city you founded comes home intact when you retake it.", strengthIcon: 'yield-gold', priceIcon: 'city' },
      { strength: 'Barbarians never attack the Celts, all game. No camp, scout or raider touches your units, cities or Settlers, and the warband camps you raise raid your rivals only.', strengthIcon: 'ui-camp' },
      specialRow('Celts', 'unit-gaesatae'),
    ],
    counteredBy: 'Garrison your cities and build Walls: a sacked city is gone, but a held one costs the Celts their army.',
  },
  Mongols: {
    identity: 'Riders of the steppe',
    lore: 'Mongol horse archers shot at full gallop; the yam relay network linked an empire.',
    ratings: { aggression: 4, economy: 2, complexity: 2 },
    pairs: [
      { strength: 'Your military units, Scouts too, get +1 move.', price: 'Your units get no terrain defence. You raise no Walls, Castles or Star Forts and buy no Siege before the Medieval era.', strengthIcon: 'mark-move', priceIcon: 'fortify' },
      { strength: `Iron Working unlocks it. A Cataphract that can shoot, ${keshig.ranged} ranged strength, ${keshig.range} tiles.`, price: `A Keshig costs ${keshig.cost} Production, ${keshig.cost - cataphractCost} more than a Cataphract.`, strengthIcon: 'unit-keshig', priceIcon: 'yield-production', onCard: true },
      { strength: 'A city with a Market at least 8 tiles from your capital earns +2 Gold more.', strengthIcon: 'building-market' },
    ],
    counteredBy: 'Build Walls early and defend your cities: before the Medieval era Mongols buy no Siege, so Walls halve everything they throw at you.',
  },
  Sparta: {
    identity: 'Raised for war',
    lore: 'Spartan citizens trained for war through the agoge and fought in disciplined hoplite formations, supported by coerced helot labour.',
    ratings: { aggression: 5, economy: 1, complexity: 3 },
    pairs: [
      { strength: `Your military units start at level 1, gain XP 50% faster and can reach level 6, Spartan Veteran. Each level adds ${RULES.promotion.strengthPercent}% to a unit’s strength.`, strengthIcon: 'unit-oathbearer' },
      { strength: 'Destroying an enemy unit restores up to 100 HP, shared among your units that hit it. Capturing a city fully heals your military units within 2 tiles.', strengthIcon: 'mark-heal' },
      { price: 'Settlers and non-military buildings cost 50% more Production. Walls, Castles, Star Forts and the Agoge cost the normal price.', priceIcon: 'unit-settler' },
    ],
    counteredBy: 'Hold behind Walls and deny them kills, so their wounded units stay wounded.',
  },
  Athens: {
    identity: 'Learning in the agora',
    lore: 'Classical Athens was known for civic debate and learning. In play, Athenian scholarship brings quick discoveries at a cost to its treasury.',
    ratings: { aggression: 1, economy: 4, complexity: 2 },
    pairs: [
      { strength: 'Every ruin also gives you 20 Science.', price: `Your Markets give +1 Gold a turn instead of +3, at the normal ${BUILDING_STATS.Market.cost} Production.`, strengthIcon: 'ruin', priceIcon: 'building-market' },
      { strength: 'The Lyceum replaces the Library and gives +6 Science a turn.', strengthIcon: 'building-listeninghouse', onCard: true },
      { price: 'A rival that captures one of your cities gets one free technology (once per city for each rival).', priceIcon: 'ui-trend-down' },
      specialRow('Athens', 'unit-peltast'),
    ],
    counteredBy: 'Capture an Athenian city for one free technology. Each empire can earn this once per city.',
  },
};
