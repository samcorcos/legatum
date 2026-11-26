// UI-side copy of the display data in crates/core/src/data.rs and the first-playable spec.
// Numbers here are only for display fallbacks and the client-side path preview; the core is
// authoritative for every rule.
import type {
  BuildingKind,
  Faction,
  ImprovementKind,
  ProductionItem,
  RuleSwitches,
  TechId,
  Terrain,
  UnitClass,
  UnitKind,
  Yields,
} from './types';
// The attribute lets Node's ESM loader (Playwright specs import data.ts) load it, as Vite does.
import coreRules from './rules.generated.json' with { type: 'json' };

/** Offline presentation data exported and checked by crates/ai/tests/ui_rules.rs. */
export const RULES = coreRules;

/** The turn the game ends on: the configured limit, else the default (core `data::DEFAULT_TURN_LIMIT`, rules 62). */
export const effectiveTurnLimit = (limit: number | null | undefined): number => limit ?? RULES.defaultTurnLimit;
type UnitNumbers = Pick<(typeof UNIT_STATS)[UnitKind], 'cost' | 'moves' | 'strength' | 'ranged' | 'range' | 'military' | 'requires' | 'upgradesTo'>;
type BuildingNumbers = Pick<(typeof BUILDING_STATS)[BuildingKind], 'cost' | 'yields' | 'cityStrength' | 'requires' | 'warBuilding'>;
const factionUnitStats = RULES.factionUnitStats as Record<Faction, Record<UnitKind, UnitNumbers>>;
const standardUnitStats = RULES.standardUnitStats as Record<Faction, Record<UnitKind, UnitNumbers>>;
const factionBuildingStats = RULES.factionBuildingStats as Record<Faction, Record<BuildingKind, BuildingNumbers>>;
const standardBuildingStats = RULES.standardBuildingStats as Record<Faction, Record<BuildingKind, BuildingNumbers>>;
/** What an aircraft's Recon order does, in the one wording every surface uses (core `Order::AirRecon`). */
export const reconRule = (): string => `Recon scouts any tile within its range and uses the aircraft's order. You see the ${RULES.reconRadius} tiles around it next turn only, so you can order a strike on what it finds.`;

/** What a Fighter on guard does, in the one wording every surface uses (build card, ready card, Air wing, Guide). */
export const interceptRule = (kind: 'Fighter' | 'StealthFighter' = 'Fighter'): string => kind === 'StealthFighter'
  ? 'intercepts enemy air strikes within 5 tiles of its city, a Stealth Bomber\u2019s included: the strike fails and the two aircraft trade damage'
  : `intercepts enemy air strikes within ${RULES.fighterRange} tiles of its city, except a Stealth Bomber\u2019s: the strike fails and the two aircraft trade damage`;
const CALENDAR_LABELS = RULES.calendarLabels as readonly string[];
/** The display year of a turn, "1040 AD" (core `calendar::calendar_label`, exported with the rules; display only, never a rule). */
export const calendarLabel = (turn: number): string => CALENDAR_LABELS[Math.min(Math.max(0, Math.floor(turn)), CALENDAR_LABELS.length - 1)];
/** The one plain line that explains the year wherever it is shown. */
export const CALENDAR_NOTE = 'The year follows the turn number and changes nothing in the game.';
export const CAPTURE_RESISTANCE_TURNS: number = RULES.captureResistanceTurns;
export const ignoresWounds = (faction: Faction | undefined, kind: UnitKind): boolean =>
  !!faction && (RULES.woundImmunity[faction] as readonly string[]).includes(kind);

/** Every unit kind in core `UnitKind::ALL` order (exported with the rules). */
export const UNIT_KINDS: UnitKind[] = RULES.unitOrder as UnitKind[];
/** The class bonus (core `combat::CLASS_BONUS_PERCENT`): the percent of its base strength a unit gains against the class it beats. */
export const CLASS_BONUS_PERCENT: number = RULES.classBonusPercent;
/** Core `combat::MOUNTED_CITY_PENALTY_PERCENT`: the percent of its base strength a pre-World War mounted unit loses charging a city. */
export const MOUNTED_CITY_PENALTY_PERCENT: number = RULES.mountedCityPenaltyPercent;
/** Display names of every unit kind (core `data::unit_name`). An identifier is the PascalCase of its display name. */
export const UNIT_NAME = RULES.unitNames as Record<UnitKind, string>;
/** Moves of every mounted unit (core `data::MOUNTED_MOVES`). */
export const MOUNTED_MOVES: number = RULES.mountedMoves;
/** The Tank's moves (core `data::TANK_MOVES`): one more than the rest of the mounted class; the Autonomous Armour has one more again. */
export const TANK_MOVES: number = RULES.tankMoves;
/** The EarlyTank's moves (core `data::EARLY_TANK_MOVES`): the first armour is slow. */
export const EARLY_TANK_MOVES: number = RULES.earlyTankMoves;
/** Core `data::indirect_fire`: the gun line that fires over terrain once Steel is known (Howitzer, Artillery, Rocket Artillery, Combat Drone). */
export const firesIndirect = (kind: UnitKind): boolean => (RULES.indirectFire as readonly string[]).includes(kind);
export const AUTONOMOUS_ARMOUR_MOVES: number = RULES.autonomousArmourMoves;
export const BUILDING_KINDS: BuildingKind[] = [
  'RiceStorehouse',
  'Granary',
  'Workshop',
  'Market',
  'Library',
  'Walls',
  'GreatLibrary',
  'Pyramids',
  'HagiaSophia',
  'EiffelTower',
  'IterFusionReactor',
  'Mill',
  'Forge',
  'Agoge',
  'University',
  'Bank',
  'Factory',
  'UpgradeWalls',
  'RoyalArsenal',
  'GrandBazaar',
  'ResearchLab',
  'YamRelayStation',
  'ImperialDiwan',
  'Roadworks',
  'StockExchange',
  'Supermarket',
  'AirDefence',
];
/** Every technology in research order: five per era, nine eras (core `eras::TECHS`, exported with the rules). */
export const TECHS: TechId[] = RULES.techOrder as TechId[];
export const FACTIONS: Faction[] = ["Switzerland", "India", "Byzantium", "Germany", "Celts", "Mongols", "Sparta", "Athens", "Aztecs", "Egypt", "Rome"];
export const GREAT_WORK_ICON = 'milestone-great-library';
export const GREAT_WORK_NAME = 'World wonder';
export const GREAT_WORK_DESC = 'Complete one of the seven world wonders on a chosen tile. Each can be completed only once in the world. Its holder scores 2 VP, and capturing it transfers those points.';
export const GREAT_WORK_RULE = 'Complete a world wonder';

const UNIT_DATA = RULES.unitStats as unknown as Record<
  UnitKind,
  {
    cost: number;
    moves: number;
    strength: number;
    ranged: number;
    range: number;
    military: boolean;
    requires: TechId | null;
    upgradesTo: UnitKind | null;
    class: UnitClass | null;
    eraTier: number;
    designStrength: number;
    designRanged: number;
    siegeBonus: number;
  }
>;

/**
 * Core `data::era_scaled`: a design strength (the old straight-line number a unit or building was
 * balanced with) on the era curve, so a city or faction edge follows the same percentage per era.
 * Rounds half up, half away from zero for a penalty.
 */
export function eraScaled(design: number, era: number): number {
  const e = Math.min(Math.max(era, 0), RULES.eraCurve.meleeLine.length - 1);
  const num = RULES.eraCurve.meleeLine[e];
  const den = RULES.eraCurve.designMeleeLine[e];
  return Math.sign(design) * Math.floor((2 * Math.abs(design) * num + den) / (2 * den));
}
/** Strength a siege unit or Bomber adds against city HP (core `data::siege_city_bonus`, on the era curve). */
export const siegeBonusOf = (kind: UnitKind): number => UNIT_DATA[kind].siegeBonus;

/** The plural of a unit's display name: "2 Spearmen", "3 Infantry", "4 Combat Drones", "5 Legionaries". */
export function pluralUnit(name: string): string {
  if (/(Infantry|Cavalry|Artillery|Armour)$/.test(name)) return name;
  if (/man$/.test(name)) return `${name.slice(0, -3)}men`;
  if (/[^aeiou]y$/.test(name)) return `${name.slice(0, -1)}ies`;
  return name.endsWith('s') ? name : `${name}s`;
}

/** The class a unit belongs to (core `data::unit_class`); null for Scouts and civilians. */
export const unitClass = (kind: UnitKind): UnitClass | null => UNIT_DATA[kind].class;
/** Core `data::attacks_in_melee`: the melee line, mounted units (the Keshig too, though it can also shoot) and Scouts close in and can take a city; a shooter's charge is a desperate one. */
/** Core `data::KESHIG_KIND`: the unit the Mongol Keshig is (the Classical mounted slot, the Cataphract's). */
export const KESHIG_KIND = RULES.keshig.kind as UnitKind;
export const attacksInMelee = (kind: UnitKind): boolean => UNIT_STATS[kind].ranged === 0 || unitClass(kind) === 'Mounted';
/** The era a unit arrives in (rules `eraTier`, 0 = Ancient). */
export const unitEraTier = (kind: UnitKind): number => UNIT_DATA[kind].eraTier;
export const CLASS_NAME: Record<UnitClass, string> = { Melee: 'Melee line', Ranged: 'Ranged', Mounted: 'Mounted', Air: 'Air' };

/** What each class does, in plain words (the triangle, with the numbers). */
export const CLASS_NOTE: Record<UnitClass, string> = {
  Melee: `Beats mounted: +${CLASS_BONUS_PERCENT}% strength against mounted units, attacking or defending.`,
  Ranged: 'Beats the melee line: strikes from range with no retaliation.',
  Mounted: `Beats ranged: +${CLASS_BONUS_PERCENT}% strength charging ranged units. Horses have −${MOUNTED_CITY_PENALTY_PERCENT}% strength charging a city.`,
  Air: 'Based in a city; strikes from the air.',
};

/** Siege's rule, in one wording for the unit card, tech card and Guide. */
/** Indirect fire on a card or tip: the headline (the Guide prints ARTILLERY_RULE in full). */
export const ARTILLERY_HEADLINE = 'Fires over terrain at targets your team can see, with built-in Siege.';
export const ARTILLERY_RULE = 'From Steel, Howitzers, Artillery, Rocket Artillery and Combat Drones fire over terrain at targets your team can see. They have Siege built in. It covers Walls up to level IV in the Industrial era and one more level in each later era. Cities still fire and defend.';
/** Siege on a card or tip: the headline (the Guide prints SIEGE_RULE in full). */
export const SIEGE_HEADLINE = 'Siege lets a unit ignore enemy Walls up to a set level when it attacks a city.';
export const SIEGE_RULE = "A unit with Siege ignores enemy Walls up to a set level when it attacks a city. Mathematics covers level I, Machinery level II and Metallurgy level III. It is bought once with Gold, in your own territory, and the unit keeps it for good. Industrial and later units have it built in: their Siege covers Walls up to level IV in the Industrial era and one more level in each later era.";
/** Core `data::siege_gold`: Gold for Siege on a unit of this kind, the base price scaled by the unit's era. */
export const siegeGold = (kind: UnitKind): number => eraScaled(RULES.siege.baseGold, unitEraTier(kind));
/** Core `data::SIEGE_USES_IRON`: each Siege unit uses 1 Iron supply while it exists. */
export const SIEGE_USES_IRON: boolean = RULES.siege.usesIron;
/** Core `data::MONGOL_SIEGE_REASON`. */
export const MONGOL_SIEGE_REASON: string = RULES.mongolSiegeReason;
/** Core `data::siege_available`: the Mongols buy Siege only from the Medieval era. */
export const siegeAvailable = (faction: Faction, era: number): boolean => faction !== 'Mongols' || era >= MONGOL_SIEGE_ERA;
/** Core `data::BUILT_IN_SIEGE_ERA`: from this era (0-based unit era) land military units have Siege built in: Industrial. */
export const BUILT_IN_SIEGE_ERA: number = RULES.siege.builtInEra;
/** Core `data::BUILT_IN_SIEGE_NOTE`: what built-in Siege does, after the unit's name. */
export const BUILT_IN_SIEGE_NOTE: string = RULES.siege.builtInNote;
const landMilitary = (kind: UnitKind): boolean => UNIT_STATS[kind].military && kind !== 'Scout' && unitClass(kind) !== 'Air';
/** Core `data::can_auto_explore`: a Scout or any land military unit (designer, 2026-11-14); civilians and aircraft never. */
export const canAutoExplore = (kind: UnitKind): boolean => kind === 'Scout' || landMilitary(kind);
/** Core `data::built_in_siege`: an Industrial or later land military unit ignores every Walls tier without buying Siege (designer, 2026-10-28). */
export const builtInSiege = (kind: UnitKind): boolean => landMilitary(kind) && unitEraTier(kind) >= BUILT_IN_SIEGE_ERA;
/** Core `data::can_have_siege`: land military units except Scouts, before the Industrial era (later ones have it built in). */
export const canHaveSiege = (kind: UnitKind): boolean => landMilitary(kind) && !builtInSiege(kind);

const MELEE_NOTE = `+${CLASS_BONUS_PERCENT}% strength against mounted units.`;
const MOUNTED_NOTE = `+${CLASS_BONUS_PERCENT}% strength charging ranged units, −${MOUNTED_CITY_PENALTY_PERCENT}% charging a city.`;
const UNIT_COPY: Record<UnitKind, { summary: string; blurb: string }> = {
  Scout: { summary: 'Explores quickly and finds ruins and rivals', blurb: 'Fast explorer. Finds ruins and rivals.' },
  Settler: { summary: 'Founds a city. Costs 1 population.', blurb: 'Founds a new city. A city needs 2 population to train one, and loses 1 population when it does.' },
  HeirToTheThrone: { summary: `Whoever holds him scores ${RULES.factionGoals.Byzantium.vp} VP`, blurb: `The last heir of the old empire, a civilian. Whoever holds him scores ${RULES.factionGoals.Byzantium.vp} VP.` },
  Spearman: { summary: 'Holds ground and takes cities; strong against mounted', blurb: `The spear line: holds ground and takes cities. ${MELEE_NOTE}` },
  Phalangite: { summary: 'Holds ground and takes cities; strong against mounted', blurb: `The pike phalanx of the Classical age. ${MELEE_NOTE}` },
  Pikeman: { summary: 'Holds ground and takes cities; strong against mounted', blurb: `The pike line of the Medieval age. ${MELEE_NOTE}` },
  Musketman: { summary: 'Holds ground and takes cities; strong against mounted', blurb: `The musket line of the Renaissance. ${MELEE_NOTE}` },
  Rifleman: { summary: 'Holds ground and takes cities; strong against mounted', blurb: `The rifle line of the Industrial age. ${MELEE_NOTE}` },
  TrenchInfantry: { summary: 'Holds ground and takes cities; strong against mounted', blurb: `The trench line of the Machine Age. ${MELEE_NOTE}` },
  Infantry: { summary: 'World War infantry; takes cities without class bonuses', blurb: 'The infantry of the World War era. No class bonuses from here on.' },
  MechanisedInfantry: { summary: `Infantry with ${RULES.mechanisedInfantryMoves} moves, keeping pace with armour; takes cities`, blurb: `Modern infantry in armoured carriers: ${RULES.mechanisedInfantryMoves} moves, so it keeps up with a Tank. Takes cities.` },
  HunterKiller: { summary: 'Armed autonomous ground robot; takes cities', blurb: `A four-legged armed autonomous robot on the melee line. Mechanised Infantry upgrades to it. No class bonuses from the World War era on.` },
  Archer: { summary: 'Strikes from 2 tiles away without retaliation', blurb: 'Strikes from 2 tiles away, with no retaliation.' },
  CompositeArcher: { summary: 'Strikes from 2 tiles away without retaliation', blurb: 'Classical composite bow. Strikes from 2 tiles away, with no retaliation.' },
  Crossbowman: { summary: 'Strikes from 2 tiles away without retaliation', blurb: 'Medieval crossbow line. Strikes from 2 tiles away, with no retaliation.' },
  Arquebusier: { summary: 'Strikes from 2 tiles away without retaliation', blurb: 'Renaissance gunpowder shooters. Strike from 2 tiles away, with no retaliation.' },
  FieldCannon: { summary: 'Strikes from 2 tiles away without retaliation', blurb: 'Industrial gun line. Strikes from 2 tiles away, with no retaliation.' },
  Howitzer: { summary: 'Indirect fire and built-in Siege', blurb: 'Machine Age gun line. Fires over terrain from 2 tiles away. Its built-in Siege covers Walls up to level V.' },
  Artillery: { summary: 'Indirect fire and built-in Siege', blurb: 'World War gun line. Fires over terrain at visible targets. Its built-in Siege covers Walls up to level VI.' },
  RocketArtillery: { summary: `Indirect fire from ${RULES.rocketArtilleryRange} tiles away; built-in Siege`, blurb: `Modern rocket launchers. Fire over terrain from ${RULES.rocketArtilleryRange} tiles away. Their built-in Siege covers Walls up to level VII.` },
  CombatDrone: { summary: 'Quick indirect fire and built-in Siege', blurb: 'Future strike drones. Quick, and fire over terrain at visible targets. Their built-in Siege covers Walls up to level VIII.' },
  Horseman: { summary: 'Fast riders; strong against ranged units', blurb: `Fast riders. ${MOUNTED_NOTE}` },
  Cataphract: { summary: 'Armoured riders; strong against ranged units', blurb: `Armoured lancers of the Classical age. ${MOUNTED_NOTE}` },
  Knight: { summary: 'Armoured riders; strong against ranged units', blurb: `Armoured riders of the Medieval age. ${MOUNTED_NOTE}` },
  Cuirassier: { summary: 'Fast riders; strong against ranged units', blurb: `Breastplated heavy horse of the Renaissance. ${MOUNTED_NOTE}` },
  Cavalry: { summary: 'Fast riders; strong against ranged units', blurb: `Sabre cavalry of the Industrial age. ${MOUNTED_NOTE}` },
  EarlyTank: { summary: `Slow first armour, ${EARLY_TANK_MOVES} moves; strong against ranged units`, blurb: `The first armour, of the Machine Age. +${CLASS_BONUS_PERCENT}% strength charging ranged units, with no penalty charging a city.` },
  Tank: { summary: `Heavy armour with ${TANK_MOVES} moves`, blurb: `Armour of the World War era. ${TANK_MOVES} moves. No class bonuses from the World War era on.` },
  MainBattleTank: { summary: `Modern armour with ${TANK_MOVES} moves`, blurb: `Armour of the Modern era, stronger than the Tank. ${TANK_MOVES} moves. No class bonuses.` },
  AutonomousArmour: { summary: `Unmanned armour with ${AUTONOMOUS_ARMOUR_MOVES} moves`, blurb: `Unmanned armour of the Future era. ${AUTONOMOUS_ARMOUR_MOVES} moves. No class bonuses.` },
  Biplane: { summary: `Sees ${RULES.aircraftSight} tiles around its city, strikes ${RULES.biplaneStrikeRange} tiles away and guards ${RULES.biplaneGuardRange}`, blurb: `Its strikes take no retaliation and do ${RULES.biplaneCityDamagePercent}% damage to city health. Lives in a city's hangar, never on the map.` },
  Fighter: { summary: `Guards ${RULES.fighterRange} tiles against air strikes, or strikes up to ${RULES.fighterStrikeRange} tiles away`, blurb: `Its strikes take no retaliation. Lives in a city's hangar, never on the map.` },
  JetFighter: { summary: `Guards ${RULES.fighterRange} tiles against air strikes, or strikes up to ${RULES.jetFighterStrikeRange} tiles away`, blurb: `Its strikes take no retaliation. Lives in a city's hangar, never on the map.` },
  Bomber: { summary: `Strikes units or cities within ${RULES.bomberRange} tiles, +${siegeBonusOf('Bomber')} vs city health`, blurb: `Strikes with no retaliation and +${siegeBonusOf('Bomber')} against city health. Lives in a city's hangar, never on the map.` },
  JetBomber: { summary: `Strikes units or cities within ${RULES.jetBomberRange} tiles, +${siegeBonusOf('JetBomber')} vs city health`, blurb: `Strikes with no retaliation and +${siegeBonusOf('JetBomber')} against city health. Lives in a city's hangar, never on the map.` },
  StealthFighter: { summary: 'Strikes up to 7 tiles away; guards 5 tiles around its city', blurb: `The only aircraft that stops a Stealth Bomber. Lives in a city's hangar, never on the map.` },
  StealthBomber: { summary: 'Strikes up to 10 tiles away; only Stealth Fighters intercept', blurb: `Strikes with no retaliation and +${siegeBonusOf('StealthBomber')} against city health. Lives in a city's hangar, never on the map.` },
};

export const UNIT_STATS = Object.fromEntries(
  UNIT_KINDS.map((k) => {
    const { class: _class, eraTier: _tier, designStrength: _ds, designRanged: _dr, siegeBonus: _sb, ...stats } = UNIT_DATA[k];
    return [k, { ...stats, ...UNIT_COPY[k] }];
  }),
) as Record<
  UnitKind,
  {
    cost: number;
    moves: number;
    strength: number;
    ranged: number;
    range: number;
    military: boolean;
    requires: TechId | null;
    upgradesTo: UnitKind | null;
    summary: string;
    blurb: string;
  }
>;

/** What a walled city does besides defending, one wording for Walls and the pieces that replace them. */
export const WALLS_FIRE_RULE = 'the city fires at enemies within 2 tiles (only walled cities fire)';

export const BUILDING_STATS: Record<
  BuildingKind,
  { cost: number; yields: Partial<Yields>; cityStrength: number; requires: TechId | null; summary: string; blurb: string; warBuilding?: boolean; predecessor?: BuildingKind; minPopulation?: number }
> = {
  RiceStorehouse: { cost: 50, yields: { food: 2 }, cityStrength: 0, requires: null, summary: '+2 Food', blurb: 'Not available to any people yet' },
  Granary: { cost: 50, yields: { food: 2 }, cityStrength: 0, requires: null, summary: '+2 Food', blurb: '+2 Food' },
  Workshop: { cost: 60, yields: { production: 2 }, cityStrength: 0, requires: 'BronzeWorking', summary: '+2 Production', blurb: '+2 Production' },
  Market: { cost: 60, yields: { gold: 3 }, cityStrength: 0, requires: 'Currency', summary: '+3 Gold', blurb: '+3 Gold' },
  ImperialDiwan: { cost: 300, yields: {}, cityStrength: 0, requires: 'Currency', summary: 'Build after every city has a Revenue Office; whoever holds its city scores 2 VP', blurb: 'India only: build after every owned city has a Revenue Office. Whoever holds its city scores 2 VP.' },
  Library: { cost: 80, yields: { science: 3 }, cityStrength: 0, requires: 'Writing', summary: '+3 Science', blurb: '+3 Science' },
  Walls: { cost: 100, yields: {}, cityStrength: 10, requires: 'Masonry', warBuilding: true, summary: '+10 city strength and a Walls bar of 100 that takes hits first. Fires at enemies', blurb: '+10 city strength. Hits land on the Walls first; the city fires at enemies within 2 tiles.' },
  Supermarket: { cost: 360, yields: { food: 4 }, cityStrength: 0, requires: 'GreenRevolution', predecessor: 'Mill', summary: '+4 Food; upgrades your Mill', blurb: '+4 Food, on top of the Granary and Mill' },
  AirDefence: { cost: 320, yields: {}, cityStrength: 0, requires: 'GuidedMissiles', summary: 'Meets enemy air strikes on this city', blurb: 'Stops one enemy air strike on this city each turn. Stealth Bombers pass it.' },
  Mill: { cost: 200, yields: { food: 3 }, cityStrength: 0, requires: 'Feudalism', predecessor: 'Granary', summary: '+3 Food; upgrades your Granary', blurb: '+3 Food' },
  Forge: { cost: 320, yields: { production: 3 }, cityStrength: 0, requires: 'IronWorking', predecessor: 'Workshop', minPopulation: 4, summary: '+3 Production; upgrades your Workshop', blurb: '+3 Production' },
  Agoge: { cost: 160, yields: {}, cityStrength: 0, requires: 'Chivalry', warBuilding: true, summary: 'Units trained here start one level higher', blurb: 'Spartan military units trained here start one level higher. Veterans retire here to Honour the Veterans.' },
  ResearchLab: { cost: 360, yields: { science: 6 }, cityStrength: 0, requires: 'Electronics', predecessor: 'University', minPopulation: 8, summary: '+6 Science; upgrades your University', blurb: '+6 Science, on top of the Library and University' },
  University: { cost: 240, yields: { science: 4 }, cityStrength: 0, requires: 'Education', predecessor: 'Library', minPopulation: 5, summary: '+4 Science; upgrades your Library', blurb: '+4 Science' },
  Bank: { cost: 200, yields: { gold: 4 }, cityStrength: 0, requires: 'Banking', predecessor: 'Market', minPopulation: 5, summary: '+4 Gold; upgrades your Market', blurb: '+4 Gold' },
  StockExchange: { cost: 360, yields: { gold: 6 }, cityStrength: 0, requires: 'Electricity', predecessor: 'Bank', minPopulation: 8, summary: '+6 Gold; upgrades your Bank', blurb: '+6 Gold, on top of the Market and Bank' },
  Factory: { cost: 440, yields: { production: 5 }, cityStrength: 0, requires: 'Industrialization', predecessor: 'Forge', minPopulation: 6, summary: '+5 Production; upgrades your Forge', blurb: '+5 Production' },
  UpgradeWalls: { cost: 240, yields: {}, cityStrength: 0, requires: null, warBuilding: true, summary: '+50 Walls for each new level; repairs the Walls', blurb: 'Adds a Walls level, up to one per era you have reached. Walls never heal, so this repairs them.' },
  GreatLibrary: { cost: 400, yields: {}, cityStrength: 0, requires: 'Philosophy', summary: 'World wonder: whoever holds its tile scores +2 VP', blurb: 'World wonder: whoever holds its tile scores +2 VP. Requires a Library in this city.' },
  Pyramids: { cost: 360, yields: { production: 1 }, cityStrength: 0, requires: 'Masonry', summary: 'World wonder: +1 Production and +2 held VP', blurb: 'The Pyramids can be completed once in the world. The holder gains +1 Production in its city and +2 VP.' },
  HagiaSophia: { cost: 400, yields: { science: 2 }, cityStrength: 0, requires: 'Machinery', summary: 'World wonder: +2 Science and +2 held VP', blurb: 'Hagia Sophia can be completed once in the world. The holder gains +2 Science in its city and +2 VP.' },
  EiffelTower: { cost: 720, yields: { gold: 3 }, cityStrength: 0, requires: 'Steel', summary: 'World wonder: +3 Gold and +2 held VP', blurb: 'One Eiffel Tower can be completed in the world. The holder gains +3 Gold in its city and +2 VP.' },
  IterFusionReactor: { cost: 900, yields: { science: 4 }, cityStrength: 0, requires: 'FusionPower', summary: 'World wonder: +4 Science and +2 held VP', blurb: 'The ITER Fusion Reactor can be completed once in the world. The holder gains +4 Science in its city and +2 VP.' },
  RoyalArsenal: { cost: 600, yields: {}, cityStrength: 0, requires: 'Cannonry', summary: 'World wonder: whoever holds its tile scores +2 VP', blurb: 'World wonder: whoever holds its tile scores +2 VP. Requires a Factory in this city.' },
  GrandBazaar: { cost: 600, yields: {}, cityStrength: 0, requires: 'Banking', summary: 'World wonder: whoever holds its tile scores +2 VP', blurb: 'World wonder: whoever holds its tile scores +2 VP. Own two Banks; build in one of them.' },
  YamRelayStation: { cost: 600, yields: {}, cityStrength: 0, requires: null, summary: `Links cities by road; whoever holds its tile scores ${RULES.factionGoals.Mongols.vp} VP`, blurb: `Each Mongol empire gets one. It lays roads to every city on the map, and whoever holds its tile scores ${RULES.factionGoals.Mongols.vp} VP.` },
  Roadworks: { cost: 140, yields: { gold: 1 }, cityStrength: 0, requires: 'Engineering', summary: '+1 Gold; links cities within 8 tiles (12 with Railways)', blurb: '+1 Gold. Builds roads to cities within 8 tiles, or 12 once you know Railways, including cities founded later. Rivals can use them too.' },
};

/** What each technology does, in a line (the core table has names, eras and costs). */
const TECH_BLURB: Record<TechId, string> = {
  Irrigation: "+1 Food in every city",
  BronzeWorking: "Unlocks Workshop",
  Masonry: "Unlocks Walls and the Pyramids; Quarries +1 Production",
  Currency: "Unlocks Market",
  Writing: "Unlocks Library",
  Mathematics: `Unlocks Phalangite; Spearmen can upgrade. Units can buy Siege, which covers Walls up to level I.`,
  Engineering: "Unlocks Roadworks and Composite Archer; Archers can upgrade",
  IronWorking: "Unlocks Forge and Cataphract. Mines improve; Horsemen can upgrade",
  CodeOfLaws: "Each city claims its best tile 2 tiles out",
  Philosophy: "Unlocks the Great Library; +1 Science in every city",
  Feudalism: "Unlocks Pikeman and Mill; Phalangites can upgrade",
  Machinery: "Unlocks Crossbowman and Hagia Sophia. Workshops gain 1 Production. Siege covers Walls up to level II",
  Chivalry: "Unlocks Knight and Sparta's Agoge. Cataphracts can upgrade to Knights.",
  Education: "Library → University",
  CropRotation: "Farms and Pastures upgrade; Granaries +1 Food",
  Gunpowder: "Unlocks Musketman and Arquebusier; Pikemen and Crossbowmen can upgrade. Reveals Niter",
  Metallurgy: "Unlocks Cuirassier; Knights can upgrade. Siege covers Walls up to level III",
  MilitaryEngineering: "Upgrading units costs half the Gold",
  Banking: "Market → Bank; unlocks Grand Bazaar",
  PrintingPress: "+1 Science per Library; +1 Gold per Market",
  Rifling: "Unlocks Rifleman; Musketmen can upgrade",
  MilitaryScience: "Unlocks Cavalry; land units cost 10% less Production",
  Cannonry: "Unlocks Field Cannon and Royal Arsenal; Arquebusiers can upgrade",
  Industrialization: "Forge → Factory. Reveals Coal",
  Railways: "Road start +2 moves. Roadworks link cities within 12 tiles. Mines gain 1 Production",
  ReplaceableParts: "Unlocks Trench Infantry; Riflemen can upgrade. Factories +2 Production",
  Combustion: "Unlocks Early Tank; Cavalry can upgrade. Oil Wells +1 Gold. Reveals Oil",
  Steel: `Unlocks Howitzer and the Eiffel Tower; Field Cannons can upgrade. Howitzers fire over terrain; their built-in Siege covers Walls up to level V.`,
  Flight: `Unlocks Biplane (sees ${RULES.aircraftSight} tiles around its city, strikes ${RULES.biplaneStrikeRange} tiles out, guards ${RULES.biplaneGuardRange}); it lives in a city's hangar`,
  Electricity: "Bank → Stock Exchange",
  MassMobilisation: "Unlocks Infantry; Trench Infantry can upgrade",
  Rocketry: "Unlocks Artillery; Howitzers can upgrade",
  ArmouredWarfare: "Unlocks Tank; Early Tanks can upgrade",
  AirPower: `Unlocks Fighter (guards a city, strikes ${RULES.fighterStrikeRange} tiles out) and Bomber (strikes ${RULES.bomberRange} tiles out); each Biplane can upgrade to either`,
  Electronics: "University → Research Lab",
  MechanisedWarfare: `Unlocks Mechanised Infantry (${RULES.mechanisedInfantryMoves} moves); Infantry can upgrade`,
  CompositeArmour: "Unlocks Main Battle Tank; Tanks can upgrade",
  GuidedMissiles: `Unlocks Rocket Artillery (strikes ${RULES.rocketArtilleryRange} tiles out) and Air Defence; Artillery can upgrade`,
  JetEngines: `Unlocks Jet Fighter (strikes ${RULES.jetFighterStrikeRange} tiles out) and Jet Bomber (strikes ${RULES.jetBomberRange} tiles out); Fighters and Bombers can upgrade`,
  GreenRevolution: "Mill → Supermarket",
  AutonomousSystems: "Unlocks Combat Drone and Autonomous Armour; Rocket Artillery and Main Battle Tanks can upgrade. Reveals Silicon",
  Robotics: "Unlocks Hunter-Killer; Mechanised Infantry can upgrade",
  FusionPower: "+5 Production in every city. Factories and Research Labs need no Coal or Oil. Unlocks ITER Fusion Reactor",
  StealthTechnology: "Jet Fighters and Jet Bombers upgrade to Stealth aircraft; hangars hold 3",
  RecursiveAi: "Reveals the map to your team; +20% Production in every city",
};

const TECH_TABLE = RULES.techs as unknown as Record<TechId, { name: string; era: number; cost: number }>;

/** Listed cost and blurb of each technology; core `eras::TECHS`. */
export const TECH_STATS: Record<TechId, { cost: number; blurb: string }> = Object.fromEntries(TECHS.map((t) => [t, { cost: TECH_TABLE[t].cost, blurb: TECH_BLURB[t] }])) as Record<TechId, { cost: number; blurb: string }>;

/** The era (0-based) each technology belongs to. */
export const TECH_ERA: Record<TechId, number> = Object.fromEntries(TECHS.map((t) => [t, TECH_TABLE[t].era])) as Record<TechId, number>;

/** A unit that starts its turn on a road gets this many extra moves; with Railways, the second (core `ROAD_MOVES`, `RAILWAY_MOVES`). */
export const ROAD_MOVES = 1;
export const RAILWAY_MOVES = RULES.railwayMoves;

/** Tech diffusion: each other living empire that knows a tech takes this much off its cost, up to the cap (core `TECH_DIFFUSION_*`). */
export const TECH_DIFFUSION_PER_EMPIRE = RULES.techDiffusionPerEmpire;
export const TECH_DIFFUSION_MAX = RULES.techDiffusionMax;

/** The Spartan Hoplite's phalanx: strength beside another Hoplite (core `data::HOPLITE_PHALANX_STRENGTH`). */
export const HOPLITE_PHALANX_STRENGTH: number = RULES.hoplitePhalanxStrength;

/** Era names in order. */
export const ERA_NAMES = RULES.eraNames as readonly string[];
export const ERA_COUNT = ERA_NAMES.length;
export const eraName = (era: number): string => ERA_NAMES[Math.min(Math.max(0, era), ERA_COUNT - 1)];
/** An Age's name from its era's name: "Medieval Age"; "Machine Age" stays as it is, because that era's name already ends in Age. */
export const ageName = (era: string): string => (/\bAge$/.test(era) ? era : `${era} Age`);
/** The technologies of one era, in research order. */
export const eraTechs = (era: number): TechId[] => TECHS.filter((t) => TECH_ERA[t] === era);


export const TERRAIN_YIELDS = RULES.terrainYields as Record<Terrain, Yields>;

/** Land move cost; null = impassable (water and mountains). */
export function moveCost(t: Terrain | null): number | null {
  switch (t) {
    case 'Ocean':
    case 'Coast':
    case 'Mountains':
    case null:
      return null;
    case 'Forest':
    case 'Hills':
      return 2;
    default:
      return 1;
  }
}

/** Core `Map::move_cost`: a road tile costs 1 whatever its terrain. `road` is the tile's road mask. */
export function tileMoveCost(t: Terrain | null, road?: number | null): number | null {
  const c = moveCost(t);
  return c !== null && road ? 1 : c;
}

/** Core `data::chariot_step_cost`: an Egyptian Chariot Archer pays one more than its full moves for rough ground (Forest or Hills without a road), so entering it takes its whole turn. */
export function chariotStepCost(cost: number | null, maxMoves: number): number | null {
  return cost !== null && cost > 1 ? Math.max(cost, maxMoves + 1) : cost;
}

export function defenseBonus(t: Terrain): number {
  return t === 'Hills' ? 5 : t === 'Forest' ? 3 : 0;
}


export const TERRAIN_NAME: Record<Terrain, string> = {
  Ocean: 'Ocean',
  Coast: 'Coast',
  Grassland: 'Grassland',
  Plains: 'Plains',
  Forest: 'Forest',
  Hills: 'Hills',
  Desert: 'Desert',
  Tundra: 'Tundra',
  Mountains: 'Mountains',
};

/** Display names (core `data::building_name`). An identifier is the PascalCase of its display name. */
export const BUILDING_NAME = RULES.buildingNames as Record<BuildingKind, string>;



export const TECH_NAME: Record<TechId, string> = Object.fromEntries(TECHS.map((t) => [t, TECH_TABLE[t].name])) as Record<TechId, string>;



/** The army-strength rule in plain words. One wording on every surface. */
export const ARMY_RULE = 'Most army points: each military unit and hangar aircraft’s strength squared, divided by ' + RULES.armyStrengthReference + ', then scaled by health';
export const ARMY_DETAIL = 'Use the higher of melee or ranged strength, including promotions and supply shortages. Scouts and aircraft count; civilians do not. Terrain, fortification and class counters depend on the battle.';

/** Decadence is proportional to the lead: `L * c / 100` percent, `c` being the slope. The only place the app mirrors the core's default (`DEFAULT_DECADENCE_SLOPE`); the view's `rule_experiments` overrides it. */
export const DEFAULT_DECADENCE_SLOPE = 0;
/** The optional decadence setting of a game. */
export interface DecadenceRules { decadenceSlope: number }

export function decadenceRules(r?: RuleSwitches): DecadenceRules {
  return { decadenceSlope: r?.decadence_slope ?? DEFAULT_DECADENCE_SLOPE };
}

/**
 * Decadence, in percent of every income, for a lead over the average empire: `L * c / 100` to the nearest percent, in
 * proportion to the lead, with no maximum. Mirrors `RuleExperiments::decadence_percent` exactly (integer maths).
 */
export function decadencePercent(lead: number, rules: DecadenceRules = { decadenceSlope: DEFAULT_DECADENCE_SLOPE }): number {
  if (lead <= 0) return 0;
  return Math.floor((lead * rules.decadenceSlope + 50) / 100);
}

/** The decadence rate in words, for tooltips and the Guide: computed from the same function as the numbers. */
export function decadenceCurveText(rules?: DecadenceRules): string {
  const at = [30, 60, 100, 200].map((l) => `${l}% ahead cuts ${decadencePercent(l, rules)}%`);
  const examples = `For example, ${at.slice(0, -1).join(', ')} and ${at[at.length - 1]}.`;
  return `The cut grows in proportion to your lead, with no maximum. ${examples}`;
}

/** The blanket rule in plain words, one wording on every surface. */
export const DECADENCE_RULE = 'The further your empire leads the average empire, the more complacent your people grow. One rate, in proportion to your lead, slows your growth (smaller families) and cuts your Production (sloth), Gold (luxury) and Science (complacency) alike. The same rate is taken from any other income, such as Switzerland\'s interest.';

/** What decadence does not do to Food, one wording (it slows growth instead): nothing is taken, so no city starves. */
export const DECADENCE_FOOD_RULE = 'Food itself is never cut, so no city starves.';

/** Power in plain words, one wording on every surface. */
export const POWER_RULE = `Power is an empire's total yields (Food, Production, Gold and Science) per turn, averaged over the last ${RULES.decadence.powerWindowTurns} turns, so a conquest raises it gradually. The average empire's Power is the mean of everyone still alive, you included. Decadence compares you with it.`;

/** The era rule in plain words, one wording on every surface. */

export interface FactionInfo {
  name: string;
  motto: string;
  /** Complete mechanics text for existing faction tooltips. */
  rule: string;
  ability: string;
  unique: string;
  goal: string;
  weakness: string;
  crest: string;
  glyph: string;
}

const factionInfo = (id: string, name: string, motto: string, ability: string, unique: string, goal: string, weakness: string): FactionInfo => ({
  name, motto, ability, unique, goal, weakness,
  rule: `${ability}${unique ? ` Unique piece: ${unique}` : ''} Weakness: ${weakness}`,
  crest: new URL(`./assets/heraldry/raster/crest-${id}-512.png`, import.meta.url).href,
  glyph: new URL(`./assets/heraldry/glyph-${id}.svg`, import.meta.url).href,
});

/**
 * A faction's unique piece in structured form. The card, the unlock sentence and the prose all read this,
 * so none of them strips numbers out of a sentence: `cost` is its Production price, `effect` the other
 * numbers (the card prints them as icon pairs), `requires` the technology it needs, `rules` whole sentences
 * about this piece only, `other` whole sentences about a sibling piece.
 */
interface UniquePiece { name: string; replaces: string | null; lead?: string; cost: number; effect: string; requires?: string; rules: string[]; other?: string[] }
/** What sets the Keshig apart from a Cataphract besides its numbers (core: one order a turn, `data::attacks_in_melee`): the comparison prints the choice; the build card and hovercard state the limit of a shot. */
export const KESHIG_CHOICE = 'Shoots or charges each turn, never both.';
export const KESHIG_SHOT_LIMIT = 'A shot never moves it and cannot take a city.';
const UNIQUE_PIECE: Record<Exclude<Faction, 'Aztecs' | 'Celts'>, UniquePiece> & { Aztecs: null; Celts: null } = {
  Aztecs: null,
  Switzerland: { name: 'Alpine Redoubt', replaces: 'Walls', cost: 120, effect: '+15 city strength, more each era', requires: 'Masonry', rules: [] },
  India: { name: 'Revenue Office', replaces: 'Market', cost: 60, effect: '+3 Gold; non-capital cities remit half the Gold they make', requires: 'Currency', rules: ['Only India can build the Imperial Diwan after every owned city has one.'] },
  // The Tagmatic Soldier is the Pikeman plus 4 design strength on the era curve, read from the stats so a retuned curve follows (a getter, as the Keshig's).
  get Byzantium(): UniquePiece {
    return { name: 'Tagmatic Soldier', replaces: 'Pikeman', cost: 162, effect: `${unitStatsFor('Byzantium', 'Pikeman').strength} strength, 2 moves`, requires: 'Feudalism', rules: ['Phalangite upgrade costs 84 Gold.'] };
  },
  Germany: { name: 'Ruhr Foundry', replaces: 'Forge', cost: 320, effect: '+6 Production', requires: 'Iron Working', rules: [], other: ['Upgrades a Workshop in a city of 4 or more. Needs Iron.'] },
  Celts: null,
  // The Keshig's numbers are the Cataphract's plus the bow, read from the stats so a retuned bow follows (a getter: `unitStatsFor` needs constants declared further down).
  get Mongols(): UniquePiece {
    const s = unitStatsFor('Mongols', KESHIG_KIND);
    return { name: 'Keshig', replaces: 'Cataphract', cost: s.cost, effect: `${s.strength} strength, ${s.ranged} ranged · range ${s.range}, ${s.moves} moves`, rules: [] };
  },
  Sparta: { name: 'Spartan Hoplite', replaces: 'Spearman', cost: 80, effect: '20 strength, 2 moves', rules: [], other: ['Military units trained at the Agoge start one level higher.'] },
  Athens: { name: 'Lyceum', replaces: 'Library', cost: 70, effect: '+6 Science', requires: 'Writing', rules: [] },
  Egypt: { name: 'Royal Granary', replaces: 'Granary', cost: 50, effect: '+3 Food total', rules: [] },
  Rome: { name: 'Castrum', replaces: 'Walls', cost: 100, effect: '+10 city strength', requires: 'Masonry', rules: ['Upgrading a melee unit here restores 50 health.'] },
};
/** Egypt's Grow project (the Nile): the city grows with its surplus Food instead of building. */
export const isGrow = (item: ProductionItem | null | undefined): boolean => item?.kind === 'Project' && item.project === 'Grow';
/** The yield a faction pays its build costs in: Egypt has no Production, so its surplus Food builds (the Nile). */
export const costYield = (faction: Faction): 'Food' | 'Production' => (faction === 'Egypt' ? 'Food' : 'Production');
/** A build cost as the faction pays it: Egypt pays in surplus Food at its conversion rate (one for one by default). */
export const costInFactionYield = (faction: Faction, cost: number): number =>
  faction === 'Egypt' ? Math.ceil((cost * 100) / Math.max(1, RULES.egyptNileBuildPercent)) : cost;
const piecePrice = (faction: Faction, p: UniquePiece): string => `${costInFactionYield(faction, p.cost)} ${costYield(faction)}`;
const requiresSentence = (p: UniquePiece): string[] => p.requires ? [`Needs ${p.requires}.`] : [];
const uniquePieceText = (p: UniquePiece, faction?: Faction): string =>
  [`${p.lead ?? `${p.name} replaces ${p.replaces}`}: ${faction ? piecePrice(faction, p) : `${p.cost} Production`}, ${p.effect}.`, ...requiresSentence(p), ...p.rules, ...(p.other ?? [])].join(' ');
/** The unique piece's line without its numbers: what it replaces, its price (in Food for Egypt), the technology it needs and its rules. For a surface whose faction lines already say the numbers. */
export const uniquePieceBare = (faction: Faction): string => {
  const p = UNIQUE_PIECE[faction];
  return p ? [`${p.lead ?? `${p.name} replaces ${p.replaces}`}: ${piecePrice(faction, p)}.`, ...requiresSentence(p), ...p.rules, ...(p.other ?? [])].join(' ') : '';
};
/** The unique piece's name and what it replaces, for the faction sheet's heading. */
export const uniquePieceHead = (faction: Faction): { name: string; replaces: string | null } => ({ name: UNIQUE_PIECE[faction]?.name ?? '', replaces: UNIQUE_PIECE[faction]?.replaces ?? null });
/** The unique piece's rule sentences for its card: the numbers are icon pairs there, so only the technology it needs and its rules; empty when it is pure stats. */
export const uniquePieceRules = (faction: Faction): string => {
  const piece = UNIQUE_PIECE[faction];
  return piece ? [...requiresSentence(piece), ...piece.rules].join(' ') : '';
};
/** What the unique piece does besides its price, as one run for an unlock card (no closing full stop, like the other unlock sentences): its numbers, then its rules. */
export const uniquePieceGain = (faction: Faction): string => {
  const piece = UNIQUE_PIECE[faction];
  return piece ? [piece.effect, ...piece.rules.map((r) => r.replace(/\.$/, ''))].join('. ') : '';
};
const BYZANTIUM_LEGIONARY = uniquePieceText(UNIQUE_PIECE.Byzantium);
/** Byzantium's Mercenary Company rule in plain words, every number from core's `company` rules. */
export const companyRule = (): string =>
  `Hire a Company: the special unit of an empire you have met and are not at War with, as that empire fields it. You pay ${RULES.company.hirePercent === 100 ? 'its Production cost' : `${RULES.company.hirePercent}% of its Production cost`} in Gold, and the seller keeps it.`;
/** Switzerland's tolls in plain words: +1 Gold for every N people in a city (the number is core's `data::SWISS_TOLL_POPULATION`). */
export const SWISS_CHEAP_BUYS = 'You buy buildings and units at 175% of their Production cost in Gold (others pay 200%).';
export const SWISS_CAPTURE_RULE = 'Lose a city and its captor takes that city’s share of your treasury, by population (twice that for your capital).';
export const swissTollRule = (): string => `Each city earns +1 Gold for every ${RULES.swissTollPopulation} population.`;
/** Switzerland's interest in plain words: no maximum (user, 2026-10-17); the number is core's `data::SWISS_INTEREST_THRESHOLD`. */
export const swissInterestRule = (): string => `Every ${RULES.swissInterestThreshold} Gold in your treasury earns +1 Gold a turn in interest.`;
/** Thin levies (mirrors core `data::LEVY_EXTRA`): the Byzantium fields at most one armed unit per city, plus this many. */
export const LEVY_EXTRA = 1;
/** The Byzantium's drawback in plain words, kept on one line so a redesign is one edit. */
export const THIN_LEVIES = `Thin levies: you can field at most one military unit per city, plus ${LEVY_EXTRA === 1 ? 'one' : LEVY_EXTRA} more.`;
/** The Guide's fine print for Thin levies (the card says only the headline above). */
export const THIN_LEVIES_DETAIL = `${THIN_LEVIES} Aircraft count; Scouts don't. Losing a city never disbands units, but you can't raise new ones until you are back under the limit.`;
/** Unique-piece prose where a faction has more than one piece. */
const PIECE_TEXT: Partial<Record<Faction, Partial<Record<string, string>>>> = {
  Byzantium: { 'u:Pikeman': BYZANTIUM_LEGIONARY },
};
/** The build card's own line for a faction piece: the effect first, short enough to read whole in
 * the production list (the tooltip keeps the full text). */
const CARD_TEXT: Partial<Record<Faction, Partial<Record<string, string>>>> = {
  Mongols: {
    'u:Cataphract': `Also shoots, at range ${RULES.keshig.range} with no return fire; a shot cannot take a city. Replaces Cataphract.`,
  },
  Sparta: {
    'u:Spearman': '20 strength, 2 moves. Replaces Spearman.',
    'b:Agoge': 'Units trained here start at level 2, and veterans retire here for Honour the Veterans.',
  },
};

/** Faction variants change the item's effect; the build list reads this instead of the base summary. */
const CARD_SUMMARY: Partial<Record<Faction, Partial<Record<string, string>>>> = {
  Switzerland: { 'b:Walls': '+15 city strength and a Walls bar that takes hits first. Fires at enemies' },
  Byzantium: { 'u:Pikeman': 'Stronger Pikeman; holds ground and takes cities' },
  Germany: { 'b:Forge': '+6 Production', 'b:Factory': '+5 Production; upgrades your Ruhr Foundry' },
  Mongols: { 'u:Cataphract': 'Rides or shoots at range 2; a shot cannot take a city' },
  Athens: { 'b:Library': '+6 Science' },
  Egypt: { 'u:Archer': `${RULES.specialUnitNumbers.chariotArcherMoves} moves, +${RULES.specialUnitNumbers.chariotArcherVolleyPercent}% shooting melee units; Forest or Hills take its whole turn` },
};

/** Core `goals::spec`, exported with the rules: every number a goal counts toward comes from here. */
const goalSpec = (faction: Faction) => RULES.factionGoals[faction];
const goalVp = (faction: Faction): number => goalSpec(faction).vp;
const goalTarget = (faction: Faction): number => goalSpec(faction).target;
const goalLive = (faction: Faction): boolean => goalSpec(faction).kind === 'Live';
const goalHeld = (faction: Faction): boolean => goalSpec(faction).kind === 'Held';

export interface FactionGoal {
  name: string;
  /** The rule in plain words, short enough for a card. */
  rule: () => string;
  /** Every condition the core checks, for the faction sheet and the Guide (the card's `rule` leaves out the fine print). */
  detail: () => string;
  /** One sentence for the faction sheet; the fine print (`detail`) lives in the Guide. Falls back to `detail`. */
  sheet?: () => string;
  /** The goal as one sentence of what to do, for the Age score check, where `rule` runs longer; defaults to `rule`. */
  ask?: () => string;
  /** A live goal is held, not earned: it scores only while the condition stands and can be lost. */
  live: boolean;
  /** A held goal belongs to a capturable item (Byzantium's Heir): whoever holds the item scores it, so it can change hands. Not `live` (its condition is not the owner's own). */
  held?: boolean;
  /** For a held goal: the item whose holder scores ("the Heir"), as the terms line names it. */
  heldItem?: string;
  /** What a live goal adds to its rule: the VP is lost when the condition stops holding. */
  liveNote?: string;
  /** The icon of the thing the goal counts. */
  icon: string;
  /** A goal earned by a one-time empire action (`Order::Action`): the action's button label on the race panel. */
  action?: string;
  /** A goal earned by a one-time unit action, offered on the eligible unit itself: the action's button label. */
  unitAction?: string;
  /** Progress rivals can already see (a India city count): the unit and the number it needs. Everything else about a rival's goal is private until it is done. */
  publicCount?: { unit: string; target: number };
}

/**
 * Every faction goal's name, rule, live-or-permanent status and public count, in one place. The
 * victory-race cards, the faction sheets and the Guide all read it, so a redesigned goal is one
 * edit here (the core stays authoritative for what actually scores).
 */
/**
 * The Aztecs use the existing Swarm rule as The New Fire: the one-time sacrifice of population.
 * Its action, dialog and map preview are still built and reachable by any empire whose core goal rule is the Swarm.
 */
export const SWARM_GOAL: FactionGoal = {
  name: 'The New Fire', live: false, icon: 'goal-new-fire', action: 'Kindle the New Fire',
  rule: () => `Offer ${RULES.swarmPopulation} population in one rite (every city keeps 1)`,
  detail: () => `Offer ${RULES.swarmPopulation} population in one rite, biggest cities first; every city keeps 1. Cities and borders shrink`,
};

export const FACTION_GOAL: Record<Faction, FactionGoal> = {
  Egypt: {
    name: 'The Royal Tomb', live: goalLive('Egypt'), held: goalHeld('Egypt'), heldItem: 'its city', icon: 'goal-royal-tomb', action: 'Build the Royal Tomb',
    rule: () => `Build the Royal Tomb: ${RULES.tombTurns} turns in which no city builds or grows. Whoever holds its city scores ${goalVp('Egypt')} VP`,
    ask: () => `Build the Royal Tomb: ${RULES.tombTurns} turns in which no city builds or grows`,
    detail: () => `While you build it, none of your cities builds or grows. After ${RULES.tombTurns} turns of work it stands at your capital. Whoever holds that city scores ${goalVp('Egypt')} VP`,
  },
  Aztecs: SWARM_GOAL,
  Switzerland: {
    name: 'The Reserve', live: goalLive('Switzerland'), icon: 'yield-gold',
    rule: () => `Keep ${fmtInt(goalTarget('Switzerland'))} Gold in your treasury`,
    detail: () => `Keep ${fmtInt(goalTarget('Switzerland'))} Gold in your treasury. You lose the ${goalVp('Switzerland')} VP while it is below that, and regain it once it recovers`,
    sheet: () => `Keep ${fmtInt(goalTarget('Switzerland'))} Gold in your treasury to hold ${goalVp('Switzerland')} VP`,
  },
  India: {
    name: 'The Imperial Diwan', live: goalLive('India'), held: goalHeld('India'), heldItem: 'the Imperial Diwan', icon: 'building-imperialdiwan',
    rule: () => `Build a Revenue Office in every owned city, then build the Imperial Diwan. Whoever holds its city scores ${goalVp('India')} VP`,
    ask: () => 'Build a Revenue Office in each of your cities, then the Imperial Diwan',
    detail: () => `Every city you own needs a Revenue Office before you can build the Imperial Diwan. Whoever holds its city scores ${goalVp('India')} VP; capture transfers the points`,
    sheet: () => `Build a Revenue Office in every city, then the Imperial Diwan: whoever holds its city scores ${goalVp('India')} VP`,
  },
  Byzantium: {
    name: 'The Heir to the Throne', live: goalLive('Byzantium'), held: goalHeld('Byzantium'), heldItem: 'the Heir', icon: 'unit-heirtothethrone',
    rule: () => `Train the Heir and keep him: whoever holds him scores ${goalVp('Byzantium')} VP`,
    ask: () => 'Train the Heir and keep him',
    detail: () => `Train the Heir and keep him. Whoever holds him scores ${goalVp('Byzantium')} VP; if a rival captures him, be at War and retake him to get the points back. Costs ${UNIT_STATS.HeirToTheThrone.cost} Production · cannot be bought`,
    sheet: () => `Train the Heir and keep him: whoever holds him scores ${goalVp('Byzantium')} VP`,
  },
  Germany: {
    name: 'Five Ruhr Foundries', live: goalLive('Germany'), icon: 'building-foundry',
    rule: () => 'Own five cities with a Ruhr Foundry',
    detail: () => `Own five cities with a Ruhr Foundry to hold ${goalVp('Germany')} VP. You lose the VP if your Foundry count drops below five, and regain it when you own five again. Cities without a Foundry do not affect the goal`,
    sheet: () => `Own five cities with a Ruhr Foundry to hold ${goalVp('Germany')} VP`,
    liveNote: 'You lose the VP when you own fewer than five Ruhr Foundries, and regain it when you own five again',
  },
  Celts: {
    name: 'Raise the Warbands', live: goalLive('Celts'), icon: 'ui-camp', action: 'Raise the Warbands',
    rule: () => 'Raise the Warbands to keep a warband camp in the territory of every city you own',
    ask: () => 'Keep a warband camp in the territory of each of your cities',
    detail: () => `Once per turn you may raise a warband camp in the territory of each Celtic city. Barbarians don't attack the Celts, so the camps raid your rivals only. You hold the VP while every city has one`,
    sheet: () => `Keep a warband camp in the territory of every city you own to hold ${goalVp('Celts')} VP`,
    liveNote: 'You lose the VP in any turn a city has no warband camp, and regain it when every city has one again',
  },
  Mongols: {
    name: 'The Yam Network', live: goalLive('Mongols'), held: goalHeld('Mongols'), heldItem: 'the Yam Relay Station', icon: 'building-yamrelaystation',
    rule: () => `Build the Yam Relay Station: whoever holds its tile scores ${goalVp('Mongols')} VP`,
    ask: () => 'Build the Yam Relay Station on a tile you choose',
    detail: () => `Build the Yam Relay Station on a chosen tile, one per empire. It replaces that tile's yield and improvement, and lays roads to every city on the map. Whoever holds its tile scores the VP, and every empire's units may cross that holder's land at Peace. Capture the city and both follow you. If the station is destroyed, it can be rebuilt`,
    sheet: () => `Build the Yam Relay Station: whoever holds its tile scores ${goalVp('Mongols')} VP`,
  },
  Sparta: {
    name: 'Honour the Veterans', live: goalLive('Sparta'), icon: 'building-agoge', unitAction: 'Retire this veteran',
    rule: () => `Retire ${goalTarget('Sparta')} veterans in a city with an Agoge`,
    detail: () => `Promote ${goalTarget('Sparta')} veterans to level ${RULES.spartaRetireLevel} or higher, build an Agoge, and retire them there. Each one leaves your army for good`,
  },
  Athens: {
    name: 'Share the Learning', live: goalLive('Athens'), icon: 'building-lyceum', action: 'Share the Learning',
    rule: () => `Share your learning once: every rival gains ${RULES.signalFreeTechs} free technologies of its choice`,
    detail: () => `Share your learning once: every living empire outside your team gains ${RULES.signalFreeTechs} free technologies of its choice, at once`,
  },
  Rome: {
    name: 'Colonia Victrix', live: goalLive('Rome'), held: goalHeld('Rome'), heldItem: 'its city', icon: 'goal-colonia',
    rule: () => `Found a city with a level ${RULES.coloniaLevel} melee veteran within ${RULES.coloniaBorderReach} tiles of a rival's land. Whoever holds it scores ${goalVp('Rome')} VP`,
    ask: () => `Found a city with a level ${RULES.coloniaLevel} melee veteran within ${RULES.coloniaBorderReach} tiles of a rival's land`,
    detail: () => `Found a city with a level ${RULES.coloniaLevel} melee veteran within ${RULES.coloniaBorderReach} tiles of a rival's land. Whoever holds that city scores ${goalVp('Rome')} VP`,
  },
};


const fmtInt = (n: number) => n.toLocaleString('en-US');
/** The goal's short rule (the cards); the full condition is `FACTION_GOAL[f].detail`. */
export const factionGoalRule = (faction: Faction): string => FACTION_GOAL[faction].rule();
/** What a goal scores, in words: "+2 VP once, kept for good" or "+2 VP while held". */
export const factionGoalTerms = (faction: Faction): string => {
  const g = FACTION_GOAL[faction];
  return `+${goalVp(faction)} VP ${g.held ? `while you hold ${g.heldItem ?? 'it'}` : g.live ? 'while held' : 'once, kept for good'}`;
};
/** The goal as one line (the picker, the Guide). */
const goalLine = (faction: Faction): string => {
  const g = FACTION_GOAL[faction];
  if (faction === 'Byzantium' || faction === 'Rome' || faction === 'Egypt') return `${g.name}: ${g.detail()}.`;
  return `${g.name}: ${g.detail()} (+${goalVp(faction)} ${g.held ? `VP while you hold ${g.heldItem ?? 'it'}` : g.live ? 'VP while held' : 'VP once, kept for good'}).`;
};

export const FLOWER_WARS = 'Flower Wars: when Aztec melee units help kill an enemy military unit and survive, the nearest Aztec city gains +1 population (no barbarians or Scouts).';
export const AZTEC_NO_HORSES = RULES.aztecNoHorses;
export const HORSE_MOUNTED = RULES.horseMounted as UnitKind[];
export const FACTION_INFO: Record<Faction, FactionInfo> = {
  Egypt: factionInfo('egypt', 'Egypt', 'The river builds everything.', `Egyptian cities get double Food. Food beyond what their people eat builds whatever the city is working on, units and Settlers included${RULES.egyptNileBuildPercent === 100 ? '' : `: ${RULES.egyptNileBuildPercent} progress for every 100 Food`}.`, uniquePieceText(UNIQUE_PIECE.Egypt, 'Egypt'), goalLine('Egypt'), "Egypt has no Production. A city that is building doesn't grow: switch it to Grow to grow it."),
  Aztecs: factionInfo('aztecs', 'Aztecs', 'War feeds the city. Torches carry the New Fire.', FLOWER_WARS, '', goalLine('Aztecs'), AZTEC_NO_HORSES + '.'),
  Switzerland: factionInfo("switzerland", "Switzerland", "Guard the passes. Keep the accounts.", `${swissTollRule()} ${swissInterestRule()} ${SWISS_CHEAP_BUYS}`, uniquePieceText(UNIQUE_PIECE.Switzerland), goalLine('Switzerland'), `Military unit upkeep is triple: 3 Gold a turn instead of 1. ${SWISS_CAPTURE_RULE}`),
  India: factionInfo("india", "India", "Akbar's regional courts bind a populous empire.", `Cities can be founded 3 tiles apart. +1 Food in every city. Each other Indian city within ${RULES.indiaNetworkRadius} tiles adds ${RULES.indiaNetworkPercentPerCity}% to Food and Production.`, uniquePieceText(UNIQUE_PIECE.India), goalLine('India'), `Non-capital cities send no Gold to your treasury until they have a Revenue Office, then half the Gold they make. Corruption cuts all four city yields by ${RULES.indiaYieldPercentPerHex}% per tile beyond ${RULES.indiaCapitalRadius} from the capital, up to 100%. Production never falls below 3.`),
  Byzantium: factionInfo("byzantium", "Byzantium", "Hold Constantinople. Hire the swords.", companyRule(), BYZANTIUM_LEGIONARY, goalLine('Byzantium'), THIN_LEVIES_DETAIL),
  Germany: factionInfo("germany", "Germany", "Coal, steel, and skilled hands.", `Your strategic deposits supply twice as much. Your Gold and Science projects pay more: ${RULES.germanyProjectGoldPercent}% and ${RULES.germanyProjectSciencePercent}% of a city’s Production. Other empires get ${RULES.projectGoldPercent}% and ${RULES.projectSciencePercent}%.`, uniquePieceText(UNIQUE_PIECE.Germany), goalLine('Germany'), 'Each Production building in a German city costs 1 Food.'),
  Celts: factionInfo("celts", "Celts", "Raid, burn and move on.", `A sack takes ${RULES.celtsSackGoldPerPopulation} Gold per population and raises a warband camp. A sacked rival capital scores 1 VP for good. Barbarians don't attack the Celts.`, '', goalLine('Celts'), "Celts can't keep cities they capture: they sack and burn them. A city you founded comes home intact when you retake it."),
  Mongols: factionInfo("mongols", "Mongols", "Across the steppe, news travels.", "Military units, Scouts too, get +1 move. Settlers keep 2 moves. A city with a Market at least 8 tiles from your capital earns +2 Gold.", uniquePieceText(UNIQUE_PIECE.Mongols), goalLine('Mongols'), "Mongol military units get no defence bonus from terrain, though fortifying still works. The Mongols never build or upgrade Walls, and a city they capture loses its Walls. They buy no Siege before the Medieval era."),
  Sparta: factionInfo("sparta", "Sparta", "Train in the agoge. Hold the line.", `Spartan military units start at level 1, gain XP 50% faster, and alone can reach level 6, Spartan Veteran. Each level adds ${RULES.promotion.strengthPercent}% to a unit's strength. Hoplites gain +${HOPLITE_PHALANX_STRENGTH} strength next to another Hoplite. The melee line fights at full strength when wounded. Destroying an enemy military unit restores up to 100 health, shared among the Spartan units that hit it that turn and survived. Capturing a city fully heals Spartan military units within 2 tiles.`, uniquePieceText(UNIQUE_PIECE.Sparta), goalLine('Sparta'), `Settlers and non-military buildings cost 50% more Production, except Walls and its upgrades and the Agoge.`),
  Athens: factionInfo("athens", "Athens", "Debate in the agora. Study at the Lyceum.", "Every ruin also gives you 20 Science.", uniquePieceText(UNIQUE_PIECE.Athens), goalLine('Athens'), `Markets yield only +1 Gold instead of +3, at the normal ${RULES.buildingStats.Market.cost} Production cost. A rival that captures one of your cities gets one free technology (once per city for each rival).`),
  Rome: factionInfo('rome', 'Rome', 'Serve the legion. Earn the land.', 'A full-health melee veteran (level 1 or higher) can found a city; the unit is consumed.', 'Legionary replaces the Phalangite: a city it founds starts at population 2. ' + uniquePieceText(UNIQUE_PIECE.Rome), goalLine('Rome'), 'Rome cannot train or buy Settlers.'),

};

/** Permanent award for the paid faction objective, matching core scoring. */
export const factionGoalVp = (faction: Faction): number => goalVp(faction);
export const GERMANY_PROJECT_GOLD_PERCENT = RULES.germanyProjectGoldPercent;
export const EGYPT_NILE_BUILD_PERCENT = RULES.egyptNileBuildPercent;
export const GERMANY_PROJECT_SCIENCE_PERCENT = RULES.germanyProjectSciencePercent;
/** Every other empire's Gold and Science projects, and everyone's Food project (core `data::PROJECT_*_PERCENT`). */
export const PROJECT_GOLD_PERCENT = RULES.projectGoldPercent;
export const PROJECT_SCIENCE_PERCENT = RULES.projectSciencePercent;
export const PROJECT_FOOD_PERCENT: number = RULES.projectFoodPercent;
/** A faction's share of city Production a project converts (core `project_percent`). */
export const projectPercent = (faction: Faction | undefined, project: 'Gold' | 'Science' | 'Food'): number =>
  project === 'Food' ? PROJECT_FOOD_PERCENT
    : faction === 'Germany'
      ? project === 'Gold' ? GERMANY_PROJECT_GOLD_PERCENT : GERMANY_PROJECT_SCIENCE_PERCENT
      : project === 'Gold' ? PROJECT_GOLD_PERCENT : PROJECT_SCIENCE_PERCENT;
/** Food for the first growth (population 1 to 2) (core `FIRST_GROWTH`). */
export const FIRST_GROWTH = RULES.firstGrowth;
/** The game's expected length in turns: the horizon forecasts and the AI pace against (core `data::EXPECTED_GAME_TURNS`). */
export const EXPECTED_GAME_TURNS = 300;
/** The first third of it: the Spartan peace grace lasts this long (core `data::FIRST_THIRD_TURNS`). */
export const FIRST_THIRD_TURNS = EXPECTED_GAME_TURNS / 3;
/** Food a later growth costs: `GROWTH_BASE` plus `GROWTH_PER_POP` for every population above 1 (core `data::GROWTH_BASE`, `GROWTH_PER_POP`). */
export const GROWTH_BASE: number = RULES.growthBase;
export const GROWTH_PER_POP: number = RULES.growthPerPop;
/** Each Settler an empire has trained makes the next one cost this much more (core `data::SETTLER_COST_STEP`). */
export const SETTLER_COST_STEP: number = RULES.settlerCostStep;
/** Core `data::FIRST_SETTLER_PERCENT`: an empire's first Settler costs this share of its price. */
export const FIRST_SETTLER_PERCENT = RULES.firstSettlerPercent;

/** Core `data::faction_unavailable_reason`: why `availableToFaction` rejects `item` (validation
 *  errors and greyed-out text). Such items are left out of production choices. */
export function factionUnavailableReason(item: ProductionItem, faction: Faction): string | null {
  if (faction === 'Rome' && item.kind === 'Unit' && item.unit === 'Settler') return 'Rome cannot train or buy Settlers; full-health melee veterans can found cities';
  if (availableToPlayer(faction, item)) return null;
  if (faction === 'Aztecs' && item.kind === 'Unit' && HORSE_MOUNTED.includes(item.unit)) return AZTEC_NO_HORSES;
  if (item.kind === 'Project' && item.project === 'Grow') return 'Only Egyptian cities can choose Grow';
  if (item.kind === 'Project' && item.project === 'Food') return 'Egyptian cities use Grow instead';
  if (faction === 'Egypt' && item.kind === 'Building' && isWorkshopLine(item.building)) return 'Workshops, Forges and Factories do nothing for Egypt, which has no Production';
  if (item.kind === 'Building' && item.building === 'RiceStorehouse') return 'No people can build the Rice Storehouse yet';
  if ((item.kind === 'Building' || item.kind === 'Wonder') && item.building === 'YamRelayStation') return 'Only Mongols can build the Yam Relay Station';
  if (faction === 'Mongols' && item.kind === 'Building' && isWallsLine(item.building)) return MONGOL_WALLS_REASON;
  if (item.kind === 'Unit' && item.unit === 'HeirToTheThrone') return 'Only Byzantium can train a Heir to the Throne';
  return `Does nothing for ${factionName(faction)}`;
}

/** Core `improvements`: names, base costs and upgrade techs, generated. */
export const IMPROVEMENT_INFO = RULES.improvements.kinds as unknown as Record<ImprovementKind, { name: string; upgradedName: string; cost: number; upgradeTech: TechId }>;
export const IMPROVEMENT_KINDS = Object.keys(IMPROVEMENT_INFO).sort() as ImprovementKind[];
/** Every improvement has one build-list line; tile-specific yield forecasts stay in More. */
export const IMPROVEMENT_SUMMARY: Record<ImprovementKind, string> = {
  Farm: 'Adds Food to a worked tile; improves automatically later',
  QuartzMine: 'Adds tile yields and Silicon supply',
  Mine: 'Adds Production to a worked tile, plus supply on Iron or Coal',
  OilWell: 'Adds tile yields and Oil supply',
  Pasture: 'Adds Food to a worked tile; improves automatically later',
  Quarry: 'Adds Production to a worked tile',
  SaltpetreWorks: 'Adds tile yields and Niter supply',
};
/** A project's card text. Egypt's projects convert the build progress its surplus Food makes. */
function projectCopy(project: 'Gold' | 'Science' | 'Food' | 'Grow', faction?: Faction): { summary: string; blurb: string } {
  if (project === 'Grow') return { summary: 'Grows the city with its surplus Food instead of building', blurb: 'This city builds nothing, and its surplus Food grows it. It never finishes: pick something to build to stop it.' };
  const pct = projectPercent(faction, project);
  const from = faction === 'Egypt' ? 'the city’s Food-made build progress' : 'city Production';
  return {
    summary: `Turns ${pct}% of ${from} into ${project} each turn`,
    blurb: `While it runs, this city builds nothing else, and ${pct}% of ${faction === 'Egypt' ? 'the city’s Food-made build progress' : 'its Production'} comes out as ${project} every turn. It never finishes: pick another item to stop it.`,
  };
}
export const improvementName = (kind: ImprovementKind, upgraded = false): string =>
  upgraded ? IMPROVEMENT_INFO[kind].upgradedName : IMPROVEMENT_INFO[kind].name;

export const factionName = (faction: Faction): string => FACTION_INFO[faction].name;

export const FACTION_ADJECTIVE: Record<Faction, string> = {
  Switzerland: 'Swiss', Mongols: 'Mongol', Sparta: 'Spartan', Germany: 'German',
  Aztecs: 'Aztec', India: 'Indian', Byzantium: 'Byzantine', Celts: 'Celtic', Athens: 'Athenian',
  Egypt: 'Egyptian', Rome: 'Roman',
};

/** Faction replacement names, generated from core (`data::building_name_for`, `unit_name_for`). */
const FACTION_NAMES = RULES.factionNames as Record<Faction, Partial<Record<BuildingKind | UnitKind, string>>>;
export const factionItemName = (faction: Faction, kind: BuildingKind | UnitKind): string | undefined => FACTION_NAMES[faction]?.[kind];

/** Core `data::MONGOL_SIEGE_ERA`: the era from which the Mongols buy Siege (core `siege_available`). */
export const MONGOL_SIEGE_ERA: number = RULES.mongolSiegeEra;
/** Core `data::MONGOL_WALLS_REASON`. */
const MONGOL_WALLS_REASON: string = RULES.mongolWallsReason;

/** Core `data::is_walls_line`: Walls and every later wall level (the Walls family). */
export function isWallsLine(building: BuildingKind): boolean {
  for (let b: BuildingKind | undefined = building; b; b = BUILDING_STATS[b].predecessor) if (b === 'Walls' || b === 'UpgradeWalls') return true;
  return false;
}

/** Mirrors core `data::faction_excluded_building_at`: the Mongols never have the Walls line (rules 57). */
export function factionExcludedBuilding(faction: Faction, building: BuildingKind): string | null {
  if (faction !== 'Sparta' && building === 'Agoge') return 'Only Sparta can build an Agoge';
  if (building === 'RiceStorehouse') return 'No people can build the Rice Storehouse yet';
  if (faction !== 'India' && building === 'ImperialDiwan') return 'Only India can build the Imperial Diwan';
  if (faction !== 'Mongols' && building === 'YamRelayStation') return 'Only Mongols can build the Yam Relay Station';
  if (faction === 'Mongols' && isWallsLine(building)) return MONGOL_WALLS_REASON;
  return null;
}

/** Mirrors core `data::available_to_faction`: the faction can build it and it does something
 * for the faction under its own rules. Replacements are the same item kinds. */
/** Core `data::is_workshop_line_building`: the Workshop and its upgrades. */
function isWorkshopLine(building: BuildingKind): boolean {
  for (let b: BuildingKind | undefined = building; b; b = BUILDING_STATS[b].predecessor) if (b === 'Workshop') return true;
  return false;
}

export function availableToFaction(faction: Faction, item: ProductionItem): boolean {
  // Nile Egypt grows with Grow, so it has no Food project (core `available_to_faction_rules`).
  if (item.kind === 'Project') return item.project === 'Grow' ? faction === 'Egypt' : item.project !== 'Food' || faction !== 'Egypt';
  // The Nile: Egypt has no Production, so the Workshop line would do nothing.
  if (faction === 'Egypt' && item.kind === 'Building' && isWorkshopLine(item.building)) return false;
  if (item.kind === 'Unit') return (item.unit !== 'HeirToTheThrone' || faction === 'Byzantium') && (faction !== 'Aztecs' || !HORSE_MOUNTED.includes(item.unit)) && (faction !== 'Rome' || item.unit !== 'Settler');
  if (item.kind === 'Improvement') return true;
  const b = item.building;
  if (factionExcludedBuilding(faction, b)) return false;
  return isGreatWork(b) || b === 'Agoge' || b === 'YamRelayStation' || b === 'ImperialDiwan' || buildingStatsFor(faction, b).cityStrength > 0
    || Object.values(buildingStatsFor(faction, b).yields).some((n) => !!n);
}

/** Core `data::available_to_player`: no rule reads the era since the Mongols' Walls stopped opening in the Medieval era (rules 57). */
export function availableToPlayer(faction: Faction, item: ProductionItem): boolean {
  return availableToFaction(faction, item);
}

/** The building families named in effect lines. Only the Workshop family changes with the faction: Germany's Forge is the Ruhr Foundry. */
const FAMILY_NAMES: Record<string, string> = {
  Granary: 'Granary and Mill', Library: 'Library, University and Research Lab', Market: 'Market and Bank',
};
export const familyNames = (faction: Faction | undefined, family: string): string =>
  family === 'Workshop' ? `Workshop, ${itemName({ kind: 'Building', building: 'Forge' }, faction)} and Factory` : FAMILY_NAMES[family];
/** Player-facing wording of core's effect lines: "+1 Food for each Granary and Mill" (not "family"), the viewer's own building names, a true minus. Core names the standard units; the Mongols' Cataphract is the Keshig: Horsemen upgrade into it, and its own upgrade drops the bow. */
const factionEffectLabel = (faction: Faction, label: string): string => {
  const text = label
    .replace(/^\+1 (\w+) per (Granary|Workshop|Library|Market) family$/, (_, y: string, b: string) => `+1 ${y} for each ${familyNames(faction, b)}`)
    .replace(/^Every city claims its best available ring-2 tile$/, 'Each city claims its best tile 2 tiles out')
    .replace(/^Stealth aircraft use 1 Silicon each; earlier aircraft keep Oil$/, 'Stealth aircraft use Silicon instead of Oil')
    .replace(/^Upgrade Gold costs -50%$/, 'Upgrade Gold costs −50%');
  return faction === 'Mongols'
    ? text.replace(/^Horsemen can upgrade to Cataphracts$/, 'Horsemen can upgrade to Keshigs').replace(/^Cataphracts can upgrade to (.+)$/, 'Keshigs can upgrade to $1 and lose the bow')
    : text;
};
/** Immediate research effects of a tech (generated from core), in the faction's own words. */
export const techCardEffectsFor = (faction: Faction, tech: TechId): string[] =>
  (RULES.techCardEffects as Record<Faction, Record<TechId, string[]>>)[faction][tech].map((label) => factionEffectLabel(faction, label));

/** Core `data::tech_has_benefit` at default rules: false means the faction skips the tech (known from the start, never researched). */
export const techHasBenefit = (faction: Faction, tech: TechId): boolean =>
  (RULES.techBenefit as Record<Faction, Record<TechId, boolean>>)[faction][tech];

export const FACTION_PIECES: Record<Exclude<Faction, 'Aztecs' | 'Celts'>, { item: ProductionItem; name: string }> & { Aztecs: null; Celts: null } = {
  Aztecs: null,
  Switzerland: { item: { kind: 'Building', building: 'Walls' }, name: 'Alpine Redoubt' },
  India: { item: { kind: 'Building', building: 'Market' }, name: 'Revenue Office' },
  Byzantium: { item: { kind: 'Unit', unit: 'Pikeman' }, name: 'Tagmatic Soldier' },
  Germany: { item: { kind: 'Building', building: 'Forge' }, name: 'Ruhr Foundry' },
  Celts: null,
  Mongols: { item: { kind: 'Unit', unit: 'Cataphract' }, name: 'Keshig' },
  Sparta: { item: { kind: 'Unit', unit: 'Spearman' }, name: 'Spartan Hoplite' },
  Athens: { item: { kind: 'Building', building: 'Library' }, name: 'Lyceum' },
  Egypt: { item: { kind: 'Building', building: 'Granary' }, name: 'Royal Granary' },
  Rome: { item: { kind: 'Building', building: 'Walls' }, name: 'Castrum' },
};

export const factionDescription = (f: Faction): string => {
  const info = FACTION_INFO[f];
  return `${info.name}. “${info.motto}” ${info.rule} Goal: ${info.goal}`;
};

/** What a production card says about an item: a faction piece's own short line, else the description. */
export function itemCardText(item: ProductionItem, faction?: Faction): string {
  return (faction && CARD_TEXT[faction]?.[itemKey(item)]) || itemDescription(item, faction);
}

/** A build card's More adds detail: drop the text's first sentence when it only restates a line the card already shows. */
export function moreText(text: string, ...cardLines: string[]): string {
  const norm = (s: string) => s.toLowerCase().replace(/[.\s]+$/, '').trim();
  const lead = text.match(/^(.*?\.)\s+(\S[\s\S]*)$/);
  return lead && cardLines.some((line) => line && norm(line) === norm(lead[1])) ? lead[2] : text;
}

/** One short description for every production kind; detail, blockers and forecasts are separate. */
export function itemSummary(item: ProductionItem, faction?: Faction): string {
  const override = faction && CARD_SUMMARY[faction]?.[itemKey(item)];
  if (override) return override;
  if (item.kind === 'Unit') return UNIT_STATS[item.unit].summary;
  if (item.kind === 'Building' || item.kind === 'Wonder') return BUILDING_STATS[item.building].summary;
  // The Nile: Egypt has no Production, so its Mine is for Copper's Gold and the supply.
  if (item.kind === 'Improvement') return faction === 'Egypt' && item.improvement === 'Mine' ? 'Gives Gold on Copper, and supply on Iron or Coal' : IMPROVEMENT_SUMMARY[item.improvement];
  return projectCopy(item.project, faction).summary;
}

export function itemDescription(item: ProductionItem, faction?: Faction): string {
  if (item.kind === 'Improvement') {
    const info = IMPROVEMENT_INFO[item.improvement];
    return `Improves a resource tile this city works. A resource yields its bonus only once improved. Becomes ${/^[AEIOU]/.test(info.upgradedName) ? 'an' : 'a'} ${info.upgradedName} by itself with ${TECH_NAME[info.upgradeTech]}.`;
  }
  if (item.kind === 'Project') return projectCopy(item.project, faction).blurb;
  const piece = faction ? FACTION_PIECES[faction] : null;
  const pieceText = faction && (PIECE_TEXT[faction]?.[itemKey(item)] ?? (piece && sameItem(item, piece.item) ? FACTION_INFO[faction].unique : undefined));
  if (faction && pieceText) return pieceText;
  if (faction === 'Sparta' && item.kind === 'Building' && item.building === 'Agoge') return BUILDING_STATS.Agoge.blurb;
  if (faction === 'Athens' && item.kind === 'Building' && item.building === 'Market') return '+1 Gold';
  if (item.kind === 'Building' && BUILDING_STATS[item.building].predecessor) {
    const stats = BUILDING_STATS[item.building];
    const from = itemName({ kind: 'Building', building: stats.predecessor! }, faction);
    const text = `${stats.blurb}. Upgrades your ${from}.${stats.minPopulation ? ` Needs ${stats.minPopulation} population.` : ''}`;
    if (faction === 'Germany' && item.building === 'Factory') return `${text} ${GERMANY_FACTORY_NOTE}`;
    return faction === 'Mongols' && buildingFamily(item.building) === 'Market' ? `${text} ${MONGOLS_MARKET_NOTE}` : text;
  }
  if (faction === 'Mongols' && item.kind === 'Building' && buildingFamily(item.building) === 'Market') return `${BUILDING_STATS[item.building].blurb}. ${MONGOLS_MARKET_NOTE}`;
  if (item.kind === 'Unit') {
    const stats = unitStatsFor(faction ?? 'India', item.unit);
    return `${stats.strength} strength${stats.ranged ? `, ${stats.ranged} ranged · range ${stats.range}` : ''}, ${stats.moves} moves. ${UNIT_STATS[item.unit].blurb}`;
  }
  const successor = BUILDING_KINDS.find((b) => BUILDING_STATS[b].predecessor === item.building);
  if (successor) {
    const next = BUILDING_STATS[successor];
    return `${BUILDING_STATS[item.building].blurb}. Upgrades to the ${itemName({ kind: 'Building', building: successor }, faction)}${next.requires ? ` with ${TECH_NAME[next.requires]}` : ''}.`;
  }
  return BUILDING_STATS[item.building].blurb;
}

/** What a German Factory leaves standing: the Foundry stays, and the goal still counts it (core `goals::factory_cities` counts the Forge-slot building wherever it stands). */
export const GERMANY_FACTORY_NOTE = `The city keeps its Ruhr Foundry, and it still counts for ${FACTION_GOAL.Germany.name}.`;

/** The Mongol Market bonus on Market-family cards (once per city; upgrades add no more). */
const MONGOLS_MARKET_NOTE = 'A city at least 8 tiles from your capital also earns +2 Gold, once for the Market and its upgrades.';

export function itemName(item: ProductionItem, faction?: Faction): string {
  if (item.kind === 'Project') return item.project === 'Grow' ? 'Grow' : `${item.project} project`;
  if (item.kind === 'Improvement') return IMPROVEMENT_INFO[item.improvement].name;
  const replacement = faction && factionItemName(faction, item.kind === 'Unit' ? item.unit : item.building);
  if (replacement) return replacement;
  return item.kind === 'Unit' ? UNIT_NAME[item.unit] : BUILDING_NAME[item.building];
}

/** The viewer's own name for the Forge-slot building, plural ("Forges", "Ruhr Foundries"). */
export function forgesName(faction?: Faction): string {
  const name = itemName({ kind: 'Building', building: 'Forge' }, faction);
  return name.endsWith('y') ? `${name.slice(0, -1)}ies` : `${name}s`;
}

/** An era objective names the standard building. A faction that replaces it reads its own name. */
export function factionObjectiveText(text: string, faction?: Faction): string {
  if (!faction) return text;
  let out = text;
  for (const kind of Object.keys(BUILDING_NAME) as BuildingKind[]) {
    const own = itemName({ kind: 'Building', building: kind }, faction), standard = BUILDING_NAME[kind];
    if (own !== standard) out = out.replace(new RegExp(`\\b(?:a|an) ${standard}\\b(?!-line)`), `${/^[AEIOU]/.test(own) ? 'an' : 'a'} ${own}`);
  }
  return out;
}

export function itemKey(item: ProductionItem): string {
  if (item.kind === 'Improvement') return `i:${item.improvement}:${item.tile.q},${item.tile.r}`;
  if (item.kind === 'Wonder') return `w:${item.building}:${item.tile.q},${item.tile.r}`;
  return item.kind === 'Unit' ? `u:${item.unit}` : item.kind === 'Building' ? `b:${item.building}` : `p:${item.project}`;
}

// Asset filenames still use stable faction IDs. Display copy is free to change independently.
const FACTION_UNIT_ICON: Partial<Record<Faction, Partial<Record<UnitKind, string>>>> = {
  Byzantium: { Pikeman: 'unit-tagmatic' },
  Rome: { Phalangite: 'unit-legionary' },
  Mongols: { Cataphract: 'unit-keshig' }, Sparta: { Spearman: 'unit-spartanhoplite' },
  Aztecs: { Spearman: 'unit-eaglewarrior' }, Egypt: { Archer: 'unit-chariot-archer' }, Celts: { Phalangite: 'unit-gaesatae' }, Athens: { CompositeArcher: 'unit-peltast' },
  India: { Knight: 'unit-warelephant' }, Switzerland: { Pikeman: 'unit-reislaufer' }, Germany: { Tank: 'unit-panzer' },
};
const FACTION_BUILDING_ICON: Partial<Record<Faction, Partial<Record<BuildingKind, string>>>> = {
  Switzerland: { Walls: 'building-alpineredoubt' }, Germany: { Forge: 'building-foundry' },
  Athens: { Library: 'building-lyceum' },
  India: { Market: 'building-market-india' },
  Egypt: { Granary: 'building-royalgranary' },
  Rome: { Walls: 'building-castrum' },
};

/** A unit's icon name: its faction replacement's own icon when `faction` has one. */
export function unitIconName(kind: UnitKind, faction?: Faction): string {
  return (faction && FACTION_UNIT_ICON[faction]?.[kind]) || `unit-${kind.toLowerCase()}`;
}

/** A building's icon name: its faction replacement's own icon (Alpine Redoubt, Ruhr Foundry, Lyceum) when `faction` has one. */
export function buildingIconName(kind: BuildingKind, faction?: Faction): string {
  if (kind === 'UpgradeWalls') return buildingIconName('Walls', faction);
  return (faction && FACTION_BUILDING_ICON[faction]?.[kind]) || `building-${kind.toLowerCase()}`;
}

/** The icon name for a production item (city panel, production picker and map banner meters). */
export function itemIcon(item: ProductionItem, faction?: Faction): string {
  if (item.kind === 'Improvement') return `improvement-${item.improvement.toLowerCase()}`;
  if (item.kind === 'Wonder') return buildingIconName(item.building, faction);
  return item.kind === 'Unit' ? unitIconName(item.unit, faction) : item.kind === 'Building' ? buildingIconName(item.building, faction) : item.project === 'Grow' ? 'yield-food' : `yield-${item.project.toLowerCase()}`;
}

export function sameItem(a: ProductionItem | null | undefined, b: ProductionItem | null | undefined) {
  return !!a && !!b && itemKey(a) === itemKey(b);
}

/** Higher tiers satisfy the original family without independent stacked buildings. */
function buildingSatisfies(built: BuildingKind, required: BuildingKind): boolean {
  if (built === required) return true;
  const previous = BUILDING_STATS[built].predecessor;
  return !!previous && buildingSatisfies(previous, required);
}
export function hasBuilding(buildings: readonly BuildingKind[], required: BuildingKind): boolean {
  return buildings.some((built) => buildingSatisfies(built, required));
}
export function buildingFamily(building: BuildingKind): BuildingKind {
  const previous = BUILDING_STATS[building].predecessor;
  return previous ? buildingFamily(previous) : building;
}
/** Core `data::is_workshop_line_building`. */
export const isWorkshopLineBuilding = (building: BuildingKind): boolean =>
  buildingSatisfies(building, 'Workshop');

export function terrainYieldsFor(faction: Faction, terrain: Terrain): Yields {
  void faction;
  return TERRAIN_YIELDS[terrain];
}

export function unitStatsFor(faction: Faction, kind: UnitKind) {
  return { ...UNIT_STATS[kind], ...factionUnitStats[faction][kind] };
}
/** The standard unit as `faction` would field it with only its faction-wide rules: what a faction piece is compared against ("Archer: 15 ranged"). */
export function standardUnitStatsFor(faction: Faction, kind: UnitKind) {
  return { ...UNIT_STATS[kind], ...standardUnitStats[faction][kind] };
}
/** The unit each faction's special unit replaces (core `data::special_unit`): every faction has one, and a Mercenary Company sells nothing else. */
export const SPECIAL_UNIT: Record<Faction, UnitKind> = RULES.specialUnits as Record<Faction, UnitKind>;
/** A faction's special unit, as its own people name it ("Eagle Warrior"). */
export const specialUnitName = (faction: Faction): string => itemName({ kind: 'Unit', unit: SPECIAL_UNIT[faction] }, faction);
/** What sets the special unit apart from the plain unit it replaces, in words and from the core's numbers (the stat line shows the rest). */
/** Guide-only fine print for a special unit, a sentence or ''. The card says the short rule (specialUnitEdge). */
export function specialUnitGuideNote(faction: Faction): string {
  return faction === 'Egypt' ? 'A road through Forest or Hills lets it in at the road’s cost.' : '';
}

export function specialUnitEdge(faction: Faction): string {
  const n = RULES.specialUnitNumbers;
  const own = unitStatsFor(faction, SPECIAL_UNIT[faction]);
  const plain = UNIT_STATS[SPECIAL_UNIT[faction]];
  switch (faction) {
    case 'Aztecs': return `+${n.eagleWarriorStrength} strength, +${n.eagleWarriorCost} Production`;
    case 'Celts': return `+${n.gaesataeAttack} strength attacking, −${-n.gaesataeDefense} defending`;
    case 'Athens': return `${n.peltastMoves} moves instead of ${plain.moves}, the same shot`;
    case 'Egypt': return `${n.chariotArcherMoves} moves instead of ${plain.moves}, +${n.chariotArcherVolleyPercent}% when shooting melee units; entering Forest or Hills takes its whole turn`;
    case 'India': return `+${n.warElephantStrength} strength, ${n.warElephantMoves} moves instead of ${plain.moves}, no penalty charging a city`;
    case 'Switzerland': return `+${n.reislauferMountedPercent}% against mounted units, on top of the usual bonus`;
    case 'Germany': return `+${n.panzerMoves - plain.moves} move`;
    case 'Sparta': return `+${HOPLITE_PHALANX_STRENGTH} strength beside another Hoplite`;
    case 'Mongols': return `${own.ranged} ranged · range ${own.range}: shoots or charges each turn`;
    case 'Byzantium': return `+${own.strength - plain.strength} strength, +${own.cost - plain.cost} Production`;
    case 'Rome': return 'A city it founds starts at population 2';
  }
}
export function buildingStatsFor(faction: Faction, kind: BuildingKind) {
  return { ...BUILDING_STATS[kind], ...factionBuildingStats[faction][kind], yields: { ...factionBuildingStats[faction][kind].yields } };
}
/** Match the core's own-stat-line rule, not later technology bonuses or family totals. */
export const isProductionBuilding = (kind: BuildingKind): boolean => (BUILDING_STATS[kind].yields.production ?? 0) > 0;
/** The standard building as `faction` would build it with only its faction-wide rules (the German discount on a new family): what a faction piece is compared against. */
export function standardBuildingStatsFor(faction: Faction, kind: BuildingKind) {
  return { ...BUILDING_STATS[kind], ...standardBuildingStats[faction][kind], yields: { ...standardBuildingStats[faction][kind].yields } };
}

/** The held VP of a completed world wonder (core `scoring::check_great_work`). */
export const GREAT_WORK_VP = 2;
/** A built world wonder's tile card: who scores it and what its city gets ("Whoever holds this tile scores +2 VP. Its city gets +1 Production."). */
export function wonderTileText(building: BuildingKind): string {
  const gives = Object.entries(BUILDING_STATS[building].yields).filter(([, n]) => n).map(([k, n]) => `+${n} ${k[0].toUpperCase()}${k.slice(1)}`);
  return `Whoever holds this tile scores +${GREAT_WORK_VP} VP.${gives.length ? ` Its city gets ${gives.join(', ')}.` : ''}`;
}
export const GREAT_WORKS = ['Pyramids', 'GreatLibrary', 'HagiaSophia', 'GrandBazaar', 'RoyalArsenal', 'EiffelTower', 'IterFusionReactor'] as const;
export const isGreatWork = (building: BuildingKind) => (GREAT_WORKS as readonly BuildingKind[]).includes(building);
export function greatWorkRequirement(building: BuildingKind): [BuildingKind, number] | null {
  switch (building) {
    case 'RoyalArsenal': return ['Factory', 1];
    case 'GrandBazaar': return ['Bank', 2];
    case 'GreatLibrary': return ['Library', 1];
    default: return null;
  }
}
export const greatWorkRules = (_faction?: Faction) => `Each empire can complete one world wonder, and build only one at a time. There is one wonder per era; each can be completed only once in the world. Choose an owned land tile within three tiles of the city, outside its centre. The wonder replaces that tile's yield and improvement. Its holder scores +2 VP; capturing it transfers the points without using the captor's own build. Rival builders you have met are public. When a rival completes it first, banked Production on that wonder returns as Gold at 50%. A city in resistance makes no progress. Switching builds banks Production. Capturing a city clears its queue and banks. Completion uses up the builder's one wonder permanently.`;

// City spacing and faction range, matching core/data.rs.
/** India may settle at distance 3; other factions require distance 4. */
export const minCityGap = (faction: Faction): number => RULES.cityGaps[faction];
/** Population is territory: a city works every tile it owns within this many hexes. */
export const WORK_RADIUS = RULES.territory.workRadius;
/** Bought tiles: 40 Gold plus 20 per tile the city gained since founding (grown or bought), times a ring surcharge. */
export const TILE_GOLD_BASE = RULES.territory.tileGoldBase;
export const TILE_GOLD_PER_TILE = RULES.territory.tileGoldPerTile;
export const TILE_RING_STEP_PERCENT = RULES.territory.tileRingStepPercent;
/** Core `tile_ring_percent`: 100% at ring 1 (distance from the city), plus the step per ring beyond. */
export const tileRingPercent = (ring: number) => 100 + TILE_RING_STEP_PERCENT * Math.max(0, ring - 1);
export const INDIA_CAPITAL_RADIUS = RULES.indiaCapitalRadius;
export const INDIA_NETWORK_RADIUS = RULES.indiaNetworkRadius;
export const INDIA_NETWORK_PERCENT_PER_CITY = RULES.indiaNetworkPercentPerCity;
export const INDIA_YIELD_PERCENT_PER_HEX = RULES.indiaYieldPercentPerHex;
export const SWISS_INTEREST_THRESHOLD = RULES.swissInterestThreshold;
/** Population that earns a Swiss city +1 Gold a turn (core `data::SWISS_TOLL_POPULATION`). */
export const SWISS_TOLL_POPULATION = RULES.swissTollPopulation;
/** Every empire's starting treasury (core `data::START_GOLD`). */
export const START_GOLD = RULES.startGold;

const WALL_LEVEL_NUMERALS = ['', 'I', 'II', 'III', 'IV', 'V', 'VI', 'VII', 'VIII', 'IX'];
/** A Walls level as a Roman numeral ("III"). */
export const wallLevelLabel = (level: number): string => WALL_LEVEL_NUMERALS[level] ?? String(level);
/** The build list name of Upgrade Walls for a city at this Walls level (0: no Walls yet). */
export const wallsUpgradeName = (level = 0, walls = 'Walls', repair = false): string => repair ? `Repair ${walls}` : level < 1 ? `Upgrade ${walls}` : level >= 9 ? `Upgrade ${walls} (top level)` : `Upgrade ${walls} (to ${WALL_LEVEL_NUMERALS[level + 1]})`;
/**
 * Upgrade Walls repairs instead of raising the level: the bar is damaged and the city is at its cap (level IX,
 * or its owner's era number; `era` is 1-based). Core `rules::cities::complete_item`.
 */
export const wallsRepairing = (city: { walls_level?: number; walls_hp?: number; walls_max_hp?: number }, era: number): boolean => {
  const level = city.walls_level ?? 0;
  return level > 0 && (city.walls_hp ?? 0) < (city.walls_max_hp ?? 0) && (level >= 9 || level >= era);
};
