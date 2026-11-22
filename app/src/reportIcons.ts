// Icons for the turn report: one per category (the header medallion) and one per event (the line's
// own medallion), all from the existing sets. A line without a specific icon falls back to its
// category's, so every line has one.
import { buildingIconName, itemIcon, unitIconName } from './data';
import type { Faction, Event } from './types';
import type { EventCategory } from './format';

export const CATEGORY_ICON: Record<EventCategory, string> = {
  attack: 'ui-siege', diplomacy: 'treaty-offer', victory: 'vp', discoveries: 'ruin', combat: 'unit-soldier', cities: 'city',
  science: 'yield-science', economy: 'yield-gold', movement: 'mark-move', orders: 'mark-skip',
};

/** The icon for one event, or undefined to use its category's. `faction` names a player's faction; `mine` says whether a player is on the viewer's team (a sack you carried out wears the capture emblem, one you suffered the alert). */
export function eventIcon(e: Event, faction?: (player: number) => Faction | undefined, mine?: (player: number) => boolean): string | undefined {
  switch (e.type) {
    case 'CityCaptured': return 'emblem-captured';
    case 'CityRazed': return mine?.(e.by) ? 'emblem-captured' : 'ui-alert';
    case 'Battle': return e.ranged ? 'unit-ranger' : 'unit-soldier';
    case 'UnitDestroyed': return unitIconName(e.unit_kind, faction?.(e.owner));
    case 'UnitPromoted': return unitIconName(e.unit_kind, faction?.(e.owner));
    case 'CityFounded': return 'emblem-founded';
    case 'FlowerWars':
    case 'CityGrew': return 'emblem-growth';
    case 'CityStarved': return 'yield-food';
    case 'ProductionCompleted': return itemIcon(e.item, faction?.(e.owner));
    case 'ItemBought': case 'TileBought': case 'TreatyPayment': case 'WonderRaceLost': return 'yield-gold';
    case 'AircraftRebased': return unitIconName(e.kind, faction?.(e.owner));
    case 'SiegeBought': return 'ui-siege';
    case 'UnitUpgraded': return unitIconName(e.to, faction?.(e.owner));
    case 'MegaprojectStarted': return 'emblem-greatwork';
    case 'TechResearched': case 'EraEntered': return 'emblem-tech';
    case 'GreatWorkCompleted': case 'GreatWorkLost': return buildingIconName(e.building);
    case 'EraEntered': case 'AgeBegan': case 'EraObjectiveScored': return 'emblem-milestone';
    case 'CapitalTaken': return 'vp';
    case 'CapitalSacked': return 'vp';
    case 'GoldGiven': return 'yield-gold';
    case 'CapitalLost': return 'vp';
    case 'FactionGoalCompleted': return e.faction === 'Byzantium' ? 'unit-heir' : e.faction === 'Mongols' ? 'building-courierhall' : 'emblem-milestone';
    case 'FactionGoalLost': return e.faction === 'Byzantium' ? 'unit-heir' : e.faction === 'Mongols' ? 'building-courierhall' : 'ui-trend-down';
    case 'SwarmUnleashed': return 'emblem-milestone';
    case 'SakokuClosed': case 'SakokuReopened': return 'ui-sakoku';
    case 'SignalBroadcast': return 'building-listeninghouse';
    case 'RoadsLaid': return 'building-courierhall';
    case 'VeteranRetired': return 'building-warshrine';
    case 'WildCalled': return 'ui-camp';
    case 'FreeTechsChosen': return 'emblem-tech';
    case 'RuinExplored': return 'emblem-ruin';
    case 'WonderDiscovered': return 'emblem-ruin';
    case 'BarbarianCampCleared': return 'emblem-camp';
    case 'ResourcesRevealed': case 'SupplyShortage': return `resource-${e.resource.toLowerCase()}`;
    case 'HeirBorn': return 'unit-heir';
    case 'CompanyHired': return 'yield-gold';
    case 'CompanyDeparted': case 'CompanyTurned': return unitIconName(e.unit, faction?.(e.owner));
    case 'WarDeclared': case 'WarJoined': case 'CallToArms': case 'CallAnswered': case 'CallLapsed': return 'unit-soldier';
    case 'PeaceEstablished': case 'TreatyActivated': return 'treaty-peace';
    case 'TreatyEnded': case 'TreatyBreached': case 'AllianceBetrayed': return 'ui-alert';
    case 'ProposalCreated': case 'ProposalAccepted': case 'ProposalRejected': case 'ProposalWithdrawn': case 'ProposalExpired': case 'ProposalInvalidated':
      return 'treaty-offer';
    case 'PlayerEliminated': return 'ui-cross';
    case 'GameWon': case 'GameDrawn': case 'VictoryHoldStarted': return 'emblem-milestone';
    case 'VictoryHoldBroken': return 'vp';
    default: return undefined;
  }
}
