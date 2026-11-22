// Turn report tone (playtest-5: "all the events here feel equivalent"): every line is good, bad or
// neutral from the viewer's side, and combat lines answer "from where?".
import { describe, expect, it } from 'vitest';
import type { Event, Hex } from './types';
import { formatEvents, lineSegments, toneCounts, type ReportContext, type Tone } from './format';
import { chipCounts } from './ui/TurnReport';

// The viewer is player 0 on team 0 with teammate 1; player 2 (team 2) is another empire.
const ctx: ReportContext = {
  me: 0,
  team: [0, 1],
  teamId: 0,
  turn: 12,
  teamPlayers: (t) => (t === 0 ? [0, 1] : [t]),
  teamName: (t) => (t === 0 ? 'you' : `Team ${t}`),
  empireName: (t) => `Team ${t}`,
  playerName: (id) => (id === null ? 'nobody' : `P${id}`),
  unitKind: (id) => (id === 7 ? 'Archer' : id === 9 ? 'Spearman' : null),
  cityName: (id) => (id === 1 ? 'Beacon' : 'Elsewhere'),
  cityPos: (id) => (id === 1 ? { q: 2, r: 2 } : null),
  place: () => 'Beacon',
};
const at: Hex = { q: 3, r: 1 };
const ev = (e: Record<string, unknown>) => e as unknown as Event;
const relation = (over: Record<string, unknown> = {}) => ({ a: 0, b: 2, state: 'War', effective_turn: 13, guarantee_until: 0, version: 1, security: null, war_id: 1, declarers: [2], ...over });
const proposal = { id: 4, from: 2, to: 0, proposer: 2, created_turn: 11, respond_by: 14, relation_version: 1, kind: 'Peace' };
const battle = (over: Record<string, unknown>) => ev({
  type: 'Battle', attacker: 7, attacker_owner: 0, defender: { kind: 'Unit', unit: 9, owner: 2, unit_kind: 'Spearman' }, at,
  attacker_at: { q: 5, r: 1 }, attacker_strength: { total: 15, parts: [] }, defender_strength: { total: 10, parts: [] },
  damage_to_attacker: 0, damage_to_defender: 35, attacker_roll: 100, defender_roll: 100, attacker_hp_after: 100,
  defender_hp_after: 65, ranged: true, meeting: false, ...over,
});
const enemyShot = battle({ attacker: 9, attacker_owner: 2, defender: { kind: 'Unit', unit: 7, owner: 0, unit_kind: 'Archer' } });

/** One or more events per event type, each with the tone the viewer should see. TS enforces every type is here. */
const CASES: Record<Event['type'], [string, Event, Tone][]> = {
  WarDeclared: [
    ['surprise war on you', ev({ type: 'WarDeclared', relation: relation({ effective_turn: 12 }) }), 'bad'],
    ['war on you next turn', ev({ type: 'WarDeclared', relation: relation() }), 'bad'],
    ['your declaration', ev({ type: 'WarDeclared', relation: relation({ declarers: [0] }) }), 'neutral'],
    ['others at war', ev({ type: 'WarDeclared', relation: relation({ a: 2, b: 3 }) }), 'neutral'],
  ],
  PeaceEstablished: [['peace', ev({ type: 'PeaceEstablished', relation: relation({ state: 'Peace' }) }), 'neutral']],
  WarJoined: [
    ['you joined an ally\'s war', ev({ type: 'WarJoined', team: 0, ally: 2, relation: relation({ a: 0, b: 3 }) }), 'neutral'],
    ['an ally joined your war', ev({ type: 'WarJoined', team: 2, ally: 0, relation: relation({ a: 2, b: 3 }) }), 'good'],
    ['a team joined a war on you', ev({ type: 'WarJoined', team: 3, ally: 2, relation: relation({ a: 0, b: 3 }) }), 'bad'],
  ],
  CallToArms: [
    ['your ally calls you', ev({ type: 'CallToArms', call: { id: 9, team: 0, ally: 2, target: 3, created_turn: 5, respond_by: 6, alliance_version: 1, war_version: 1 } }), 'neutral'],
  ],
  CallAnswered: [
    ['your ally joined your war', ev({ type: 'CallAnswered', joined: true, effective_turn: 6, call: { id: 9, team: 2, ally: 0, target: 3, created_turn: 5, respond_by: 6, alliance_version: 1, war_version: 1 } }), 'good'],
    ['your ally declined your call', ev({ type: 'CallAnswered', joined: false, effective_turn: 6, call: { id: 9, team: 2, ally: 0, target: 3, created_turn: 5, respond_by: 6, alliance_version: 1, war_version: 1 } }), 'bad'],
    ['you declined', ev({ type: 'CallAnswered', joined: false, effective_turn: 6, call: { id: 9, team: 0, ally: 2, target: 3, created_turn: 5, respond_by: 6, alliance_version: 1, war_version: 1 } }), 'neutral'],
  ],
  CallLapsed: [
    ['no answer', ev({ type: 'CallLapsed', effective_turn: 7, reason: 'No answer: the Alliance and the Peace stand', call: { id: 9, team: 0, ally: 2, target: 3, created_turn: 5, respond_by: 6, alliance_version: 1, war_version: 1 } }), 'neutral'],
  ],
  HeirBorn: [
    ['your Heir', ev({ type: 'HeirBorn', city: 1, owner: 0, city_name: 'Constantinople', at: { q: 3, r: 3 } }), 'good'],
    ['a rival\'s Heir', ev({ type: 'HeirBorn', city: 1, owner: 2, city_name: 'Pella', at: { q: 3, r: 3 } }), 'neutral'],
  ],
  CompanyHired: [
    ['you hired', ev({ type: 'CompanyHired', city: 1, owner: 0, seller: 2, unit: 'Spearman', gold: 80, until: 41 }), 'neutral'],
    ['you sold', ev({ type: 'CompanyHired', city: 1, owner: 2, seller: 0, unit: 'Spearman', gold: 80, until: 41 }), 'good'],
  ],
  CompanyDeparted: [['a contract ended', ev({ type: 'CompanyDeparted', unit: 'Spearman', owner: 0, origin: 'Sparta', pos: { q: 3, r: 3 } }), 'neutral']],
  CompanyTurned: [
    ['yours turned', ev({ type: 'CompanyTurned', unit: 'Spearman', owner: 0, origin: 'Sparta', pos: { q: 3, r: 3 } }), 'bad'],
    ['a rival\'s turned', ev({ type: 'CompanyTurned', unit: 'Spearman', owner: 2, origin: 'Sparta', pos: { q: 3, r: 3 } }), 'neutral'],
  ],
  TreatyActivated: [['alliance', ev({ type: 'TreatyActivated', relation: relation({ state: 'Alliance', guarantee_until: 0, security: { id: 3, started_turn: 12, protected_until: 22 } }) }), 'neutral']],
  TreatyEnded: [
    ['your ally left you', ev({ type: 'TreatyEnded', treaty_id: 3, a: 0, b: 2, effective_turn: 12, reason: 'P2 left the Alliance' }), 'bad'],
    ['you left', ev({ type: 'TreatyEnded', treaty_id: 3, a: 0, b: 2, effective_turn: 12, reason: 'P0 & P1 left the Alliance' }), 'neutral'],
    ['eliminated', ev({ type: 'TreatyEnded', treaty_id: 3, a: 0, b: 2, effective_turn: 12, reason: 'Team eliminated' }), 'neutral'],
  ],
  TreatyBreached: [
    ['broken against you', ev({ type: 'TreatyBreached', treaty_id: 3, by: 2, with: 0, player: 2, cost: 60, effective_turn: 12 }), 'bad'],
    ['you broke it and paid', ev({ type: 'TreatyBreached', treaty_id: 3, by: 0, with: 2, player: 0, cost: 60, effective_turn: 12 }), 'bad'],
    ['others', ev({ type: 'TreatyBreached', treaty_id: 3, by: 2, with: 3, player: 2, cost: 60, effective_turn: 12 }), 'neutral'],
  ],
  TreatyPayment: [
    ['you paid to leave an Alliance', ev({ type: 'TreatyPayment', player: 0, gold: 30, cause: 'Alliance breach', treaty_id: 3, effective_turn: 12 }), 'bad'],
    ['you paid the breach cost', ev({ type: 'TreatyPayment', player: 0, gold: 30, cause: 'Alliance breach', treaty_id: 3, effective_turn: 12 }), 'bad'],
    ['your teammate paid', ev({ type: 'TreatyPayment', player: 1, gold: 60, cause: 'Alliance breach', treaty_id: 3, effective_turn: 12 }), 'bad'],
  ],
  AllianceBetrayed: [
    ['a rival betrayed you', ev({ type: 'AllianceBetrayed', betrayal: { by: 2, against: 0, turn: 12, until: 32 } }), 'bad'],
    ['you betrayed them', ev({ type: 'AllianceBetrayed', betrayal: { by: 0, against: 2, turn: 12, until: 32 } }), 'bad'],
    ['others', ev({ type: 'AllianceBetrayed', betrayal: { by: 2, against: 3, turn: 12, until: 32 } }), 'neutral'],
  ],
  ProposalCreated: [['offer', ev({ type: 'ProposalCreated', proposal }), 'neutral']],
  ProposalAccepted: [['offer', ev({ type: 'ProposalAccepted', proposal, effective_turn: 12 }), 'neutral']],
  ProposalRejected: [['offer', ev({ type: 'ProposalRejected', proposal, effective_turn: 12 }), 'neutral']],
  ProposalWithdrawn: [['offer', ev({ type: 'ProposalWithdrawn', proposal, effective_turn: 12 }), 'neutral']],
  ProposalExpired: [['offer', ev({ type: 'ProposalExpired', proposal, effective_turn: 12 }), 'neutral']],
  ProposalInvalidated: [['offer', ev({ type: 'ProposalInvalidated', proposal, effective_turn: 12, reason: 'war' }), 'neutral']],
  OrderSuperseded: [['superseded', ev({ type: 'OrderSuperseded', decision: { team: 0, player: 1, order: { type: 'DeclareWar', against: 2 }, winner: 0, winner_order: { type: 'DeclareWar', against: 2 }, reported_turn: 12 } }), 'neutral']],
  UnitAwakened: [['yours', ev({ type: 'UnitAwakened', unit: 7, owner: 0, unit_kind: 'Archer', at }), 'neutral']],
  UnitHealed: [['yours', ev({ type: 'UnitHealed', unit: 7, owner: 0, unit_kind: 'Archer', at }), 'good']],
  AutomationStopped: [['yours', ev({ type: 'AutomationStopped', unit: 7, owner: 0, at, reason: 'EnemySighted' }), 'neutral']],
  UnitMoved: [['moved', ev({ type: 'UnitMoved', unit: 9, owner: 2, from: at, to: { q: 4, r: 1 } }), 'neutral']],
  MoveBlocked: [
    ['your move stopped', ev({ type: 'MoveBlocked', unit: 7, owner: 0, at, toward: at, reason: 'move stopped after 2 blocked turns: blocked' }), 'bad'],
    ['ordinary block', ev({ type: 'MoveBlocked', unit: 7, owner: 0, at, toward: at, reason: 'occupied' }), 'neutral'],
  ],
  CityReinforced: [
    ['your unit reached its city first', ev({ type: 'CityReinforced', unit: 9, owner: 0, unit_kind: 'Spearman', city: 1, at }), 'good'],
    ['their unit reached its city first', ev({ type: 'CityReinforced', unit: 9, owner: 2, unit_kind: 'Spearman', city: 5, at }), 'neutral'],
  ],
  Bounce: [['bounce', ev({ type: 'Bounce', units: [7, 8], tile: at }), 'neutral']],
  AttackPursuit: [
    ['your target slipped away', ev({ type: 'AttackPursuit', unit: 7, owner: 0, at, target: 9, target_owner: 2, target_kind: 'Spearman', target_at: at, outcome: 'Followed' }), 'bad'],
    ['awaiting war', ev({ type: 'AttackPursuit', unit: 7, owner: 0, at, target: 9, target_owner: 2, target_kind: 'Spearman', target_at: at, outcome: 'AwaitingWar' }), 'neutral'],
  ],
  Battle: [
    ['your Ranger bombarded them', battle({}), 'good'],
    ['their shot hit your Ranger', enemyShot, 'bad'],
    ['your city fired on them', battle({ attacker: { city: 1 }, attacker_at: { q: 2, r: 2 } }), 'good'],
    ['their city fired on you', battle({ attacker: { city: 5 }, attacker_owner: 2, defender: { kind: 'Unit', unit: 7, owner: 0, unit_kind: 'Archer' } }), 'bad'],
    ['barbarians attacked you', battle({ attacker: 9, attacker_owner: 255, ranged: false, damage_to_attacker: 20, attacker_hp_after: 80, defender: { kind: 'Unit', unit: 7, owner: 0, unit_kind: 'Archer' } }), 'bad'],
    ['their attacker died on your defender', battle({ attacker: 9, attacker_owner: 2, ranged: false, damage_to_attacker: 100, attacker_hp_after: 0, defender: { kind: 'Unit', unit: 7, owner: 0, unit_kind: 'Archer' } }), 'good'],
    ['your attacker died', battle({ ranged: false, damage_to_attacker: 100, attacker_hp_after: 0 }), 'bad'],
    ['your city was hit', battle({ attacker: 9, attacker_owner: 2, defender: { kind: 'City', city: 1, owner: 0 } }), 'bad'],
    ['a meeting you lost more in', battle({ ranged: false, meeting: true, damage_to_attacker: 40, attacker_hp_after: 60, damage_to_defender: 20, defender_hp_after: 80 }), 'bad'],
    ['others fighting', battle({ attacker_owner: 3, defender: { kind: 'Unit', unit: 9, owner: 2, unit_kind: 'Spearman' } }), 'neutral'],
  ],
  UnitDestroyed: [
    ['yours', ev({ type: 'UnitDestroyed', unit: 7, owner: 0, unit_kind: 'Archer', at }), 'bad'],
    ['theirs, no battle of yours', ev({ type: 'UnitDestroyed', unit: 9, owner: 2, unit_kind: 'Spearman', at }), 'neutral'],
  ],
  UnitDisbanded: [['yours', ev({ type: 'UnitDisbanded', unit: 7, owner: 0, reason: 'upkeep' }), 'bad']],
  SettlerCaptured: [
    ['yours taken', ev({ type: 'SettlerCaptured', unit: 4, from: 0, to: 2, at }), 'bad'],
    ['you took one', ev({ type: 'SettlerCaptured', unit: 4, from: 2, to: 0, at }), 'good'],
  ],
  CityFounded: [
    ['yours', ev({ type: 'CityFounded', city: 1, owner: 0, name: 'Beacon', at }), 'good'],
    ['theirs', ev({ type: 'CityFounded', city: 5, owner: 2, name: 'Far', at }), 'neutral'],
  ],
  FlowerWars: [['yours', ev({ type: 'FlowerWars', city: 1, owner: 0, population: 3 }), 'good'], ['theirs', ev({ type: 'FlowerWars', city: 1, owner: 2, population: 3 }), 'neutral']],
  CityGrew: [['yours', ev({ type: 'CityGrew', city: 1, owner: 0, population: 3 }), 'good']],
  CityStarved: [['yours', ev({ type: 'CityStarved', city: 1, owner: 0, population: 2 }), 'bad']],
  CityCaptured: [
    ['yours lost', ev({ type: 'CityCaptured', city: 1, from: 0, to: 2 }), 'bad'],
    ['you took it', ev({ type: 'CityCaptured', city: 1, from: 2, to: 0 }), 'good'],
  ],
  AthensCityCaptured: [
    ['your city rewarded a rival', ev({ type: 'AthensCityCaptured', city: 1, from: 0, to: 2 }), 'bad'],
    ['you earned a free technology', ev({ type: 'AthensCityCaptured', city: 1, from: 2, to: 0 }), 'good'],
  ],
  CapitalTaken: [
    ['your held point', ev({ type: 'CapitalTaken', captor: 0, capital: 5, name: 'Rome' }), 'good'],
    ['rival held point', ev({ type: 'CapitalTaken', captor: 2, capital: 5, name: 'Rome' }), 'neutral'],
  ],
  CapitalSacked: [
    ['you sacked a capital', ev({ type: 'CapitalSacked', captor: 0, capital: 5, name: 'Rome' }), 'good'],
    ['a rival sacked a capital', ev({ type: 'CapitalSacked', captor: 2, capital: 5, name: 'Rome' }), 'neutral'],
  ],
  GoldGiven: [
    ['you gave a teammate Gold', ev({ type: 'GoldGiven', from: 0, to: 1, amount: 40 }), 'neutral'],
    ['a teammate gave you Gold', ev({ type: 'GoldGiven', from: 1, to: 0, amount: 40 }), 'good'],
  ],
  CapitalLost: [
    ['your held point lost', ev({ type: 'CapitalLost', player: 0, capital: 5, name: 'Rome' }), 'bad'],
    ['rival held point lost', ev({ type: 'CapitalLost', player: 2, capital: 5, name: 'Rome' }), 'neutral'],
  ],
  SpartaConquest: [['your regiments healed', ev({ type: 'SpartaConquest', city: 1, player: 0, units_healed: 2 }), 'good']],
  SpartaKillHeal: [['your regiments healed by a kill', ev({ type: 'SpartaKillHeal', player: 0, unit: 9, victim_owner: 2, unit_kind: 'Archer', hp_healed: 100 }), 'good']],
  CityRazed: [
    ['yours sacked', ev({ type: 'CityRazed', city: 1, from: 0, by: 2, at }), 'bad'],
    ['you sacked', ev({ type: 'CityRazed', city: 1, from: 2, by: 0, at }), 'good'],
  ],
  BuildingDismantled: [['yours', ev({ type: 'BuildingDismantled', city: 1, owner: 0, building: 'Workshop', reason: 'experimental capture plunder' }), 'bad']],
  ItemBought: [['routine', ev({ type: 'ItemBought', city: 1, owner: 0, item: { kind: 'Unit', unit: 'Spearman' }, gold: 60 }), 'neutral']],
  ProductionCompleted: [['yours', ev({ type: 'ProductionCompleted', city: 1, owner: 0, item: { kind: 'Unit', unit: 'Spearman' } }), 'good']],
  MegaprojectStarted: [['theirs', ev({ type: 'MegaprojectStarted', city: 5, owner: 2, building: 'GreatLibrary' }), 'neutral']],
  WonderRaceLost: [['your refund', ev({ type: 'WonderRaceLost', city: 1, owner: 0, winner: 2, building: 'Pyramids', production: 140, gold: 70 }), 'bad']],
  EraEntered: [
    ['yours', ev({ type: 'EraEntered', player: 0, era: 2, first: true }), 'good'],
    ['theirs', ev({ type: 'EraEntered', player: 2, era: 2, first: false }), 'neutral'],
  ],
  TechResearched: [
    ['yours', ev({ type: 'TechResearched', player: 0, tech: 'BronzeWorking' }), 'good'],
    ['theirs', ev({ type: 'TechResearched', player: 2, tech: 'BronzeWorking' }), 'neutral'],
  ],
  ResearchQueueAdvanced: [['routine', ev({ type: 'ResearchQueueAdvanced', player: 0, next: 'BronzeWorking' }), 'neutral']],
  AircraftRebased: [
    ['yours', ev({ type: 'AircraftRebased', plane: 40, owner: 0, kind: 'Bomber', from: 1, to: 2 }), 'neutral'],
    ['theirs', ev({ type: 'AircraftRebased', plane: 41, owner: 2, kind: 'Fighter', from: 5, to: 6 }), 'neutral'],
  ],
  TileBought: [
    ['yours', ev({ type: 'TileBought', city: 1, owner: 0, tile: at, gold: 40 }), 'neutral'],
    ['theirs', ev({ type: 'TileBought', city: 5, owner: 2, tile: at, gold: 40 }), 'neutral'],
  ],
  TileClaimed: [
    ['yours', ev({ type: 'TileClaimed', city: 1, owner: 0, tile: at }), 'good'],
    ['theirs', ev({ type: 'TileClaimed', city: 1, owner: 2, tile: at }), 'neutral'],
  ],
  TileReleased: [
    ['yours', ev({ type: 'TileReleased', city: 1, owner: 0, tile: at }), 'neutral'],
    ['theirs', ev({ type: 'TileReleased', city: 1, owner: 2, tile: at }), 'neutral'],
  ],
  TilesReassigned: [
    ['yours', ev({ type: 'TilesReassigned', from: 1, to: 2, owner: 0, tiles: [at] }), 'good'],
    ['theirs', ev({ type: 'TilesReassigned', from: 1, to: 2, owner: 2, tiles: [at] }), 'neutral'],
  ],
  RuinExplored: [['yours', ev({ type: 'RuinExplored', unit: 7, owner: 0, at, reward: { kind: 'Science', amount: 25, research: null } }), 'good']],
  WonderDiscovered: [['yours', ev({ type: 'WonderDiscovered', owner: 0, at, wonder: 'VictoriaFalls', gold: 40, science: 0 }), 'good']],
  GreatWorkCompleted: [['you hold it', ev({ type: 'GreatWorkCompleted', player: 0, of: 2, building: 'GreatLibrary', vp: 2, again: false }), 'good']],
  GreatWorkLost: [['you lost it', ev({ type: 'GreatWorkLost', player: 0, of: 0, building: 'GreatLibrary', vp: 2 }), 'bad']],
  AgeBegan: [
    ['yours', ev({ type: 'AgeBegan', era: 2, first: 0 }), 'good'],
    ['theirs', ev({ type: 'AgeBegan', era: 2, first: 2 }), 'neutral'],
  ],
  EraObjectiveScored: [
    ['your VP', ev({ type: 'EraObjectiveScored', player: 0, era: 2, slot: 1, objective: 'Gain 4 cities' }), 'good'],
    ['theirs', ev({ type: 'EraObjectiveScored', player: 2, era: 2, slot: 1, objective: 'Gain 4 cities' }), 'neutral'],
  ],
  FactionGoalCompleted: [
    ['yours', ev({ type: 'FactionGoalCompleted', player: 0, faction: 'India', vp: 2 }), 'good'],
    ['regained', ev({ type: 'FactionGoalCompleted', player: 0, faction: 'Switzerland', vp: 2, again: true }), 'good'],
    ['theirs', ev({ type: 'FactionGoalCompleted', player: 2, faction: 'Switzerland', vp: 2 }), 'neutral'],
    ['your Colonia', ev({ type: 'FactionGoalCompleted', player: 0, faction: 'Rome', vp: 2 }), 'good'],
    ['a rival Royal Tomb', ev({ type: 'FactionGoalCompleted', player: 2, faction: 'Egypt', vp: 2 }), 'neutral'],
  ],
  FactionGoalLost: [
    ['you spent below the hoard', ev({ type: 'FactionGoalLost', player: 0, faction: 'Switzerland', vp: 2 }), 'bad'],
    ['a rival did', ev({ type: 'FactionGoalLost', player: 2, faction: 'Switzerland', vp: 2 }), 'good'],
    ['a camp of yours fell', ev({ type: 'FactionGoalLost', player: 0, faction: 'Celts', vp: 2 }), 'bad'],
    ['the Celts lost it', ev({ type: 'FactionGoalLost', player: 2, faction: 'Celts', vp: 2 }), 'good'],
    ['you lost a Colonia you held', ev({ type: 'FactionGoalLost', player: 0, faction: 'Rome', vp: 2 }), 'bad'],
    ['a rival lost the Royal Tomb', ev({ type: 'FactionGoalLost', player: 2, faction: 'Egypt', vp: 2 }), 'neutral'],
  ],
  TombStarted: [
    ['yours', ev({ type: 'TombStarted', player: 0, turns: 0 }), 'neutral'],
    ['theirs', ev({ type: 'TombStarted', player: 2, turns: 4 }), 'neutral'],
  ],
  TombPaused: [
    ['yours', ev({ type: 'TombPaused', player: 0, turns: 6 }), 'neutral'],
    ['theirs', ev({ type: 'TombPaused', player: 2, turns: 6 }), 'neutral'],
  ],
  TombRaised: [
    ['yours', ev({ type: 'TombRaised', player: 0, city: 1, name: 'Thebes', at: { q: 3, r: 3 } }), 'good'],
    ['theirs', ev({ type: 'TombRaised', player: 2, city: 5, name: 'Memphis', at: { q: 3, r: 3 } }), 'neutral'],
  ],
  VeteranRetired: [['yours', ev({ type: 'VeteranRetired', player: 0, unit: 7, kind: 'Spearman', city: 1, retired: 2, of: 3 }), 'good']],
  WildCalled: [
    ['yours', ev({ type: 'WildCalled', player: 0, camps: 3 }), 'good'],
    ['theirs', ev({ type: 'WildCalled', player: 2, camps: 3 }), 'neutral'],
  ],
  SakokuClosed: [
    ['yours', ev({ type: 'SakokuClosed', player: 0 }), 'neutral'],
    ['theirs', ev({ type: 'SakokuClosed', player: 2 }), 'neutral'],
  ],
  SakokuReopened: [
    ['yours', ev({ type: 'SakokuReopened', player: 0, turns: 12 }), 'neutral'],
    ['theirs', ev({ type: 'SakokuReopened', player: 2, turns: 12 }), 'good'],
  ],
  SwarmUnleashed: [
    ['yours', ev({ type: 'SwarmUnleashed', player: 0, cities: [{ city: 1, lost: 20, population: 5, tiles: 20 }] }), 'good'],
    ['theirs', ev({ type: 'SwarmUnleashed', player: 2, cities: [{ city: 5, lost: 20, population: 5, tiles: 20 }] }), 'neutral'],
  ],
  SignalBroadcast: [
    ['yours', ev({ type: 'SignalBroadcast', from: 0, receivers: [2], techs_each: 3 }), 'good'],
    ['you receive', ev({ type: 'SignalBroadcast', from: 2, receivers: [0], techs_each: 3 }), 'good'],
    ['between others', ev({ type: 'SignalBroadcast', from: 2, receivers: [3], techs_each: 3 }), 'neutral'],
  ],
  RoadsLaid: [
    ['yours', ev({ type: 'RoadsLaid', owner: 0, city: 1, at: { q: 3, r: 3 }, linked: 5 }), 'good'],
    ['a rival’s', ev({ type: 'RoadsLaid', owner: 2, city: 1, at: { q: 3, r: 3 }, linked: 5 }), 'neutral'],
  ],
  FreeTechsChosen: [['yours', ev({ type: 'FreeTechsChosen', player: 0, from: 2, techs: ['BronzeWorking', 'Currency'] }), 'good']],
  SiegeBought: [['yours', ev({ type: 'SiegeBought', unit: 7, owner: 0, gold: 160 }), 'neutral']],
  UnitUpgraded: [['routine', ev({ type: 'UnitUpgraded', unit: 7, owner: 0, to: 'Crossbowman', gold: 80 }), 'neutral']],
  BarbarianCampSpawned: [['camp', ev({ type: 'BarbarianCampSpawned', camp: 1, at }), 'neutral']],
  BarbarianCampDiscovered: [['camp', ev({ type: 'BarbarianCampDiscovered', camp: 1, team: 0, at }), 'neutral']],
  BarbarianScoutSpotted: [['your city spotted', ev({ type: 'BarbarianScoutSpotted', unit: 9, city: 1, owner: 0, at }), 'bad']],
  BarbarianRaidLaunched: [['raid on you', ev({ type: 'BarbarianRaidLaunched', camp: 1, city: 1, owner: 0, direction: 'north' }), 'bad']],
  BarbarianCampCleared: [['you cleared it', ev({ type: 'BarbarianCampCleared', camp: 1, at, by: 0, unit: 7, gold: 50 }), 'good']],
  ResourcesRevealed: [['yours', ev({ type: 'ResourcesRevealed', player: 0, resource: 'Iron', city: 1, sources: 1, supply: 2 }), 'good']],
  SupplyShortage: [
    ['short, yours', ev({ type: 'SupplyShortage', player: 0, resource: 'Iron', idle: 2, supplied: 6, used: 8 }), 'bad'],
    ['restored, yours', ev({ type: 'SupplyShortage', player: 0, resource: 'Iron', idle: 0, supplied: 8, used: 8 }), 'good'],
    ['a rival’s', ev({ type: 'SupplyShortage', player: 2, resource: 'Iron', idle: 2, supplied: 6, used: 8 }), 'neutral'],
  ],
  UnitPromoted: [['yours', ev({ type: 'UnitPromoted', unit: 7, owner: 0, unit_kind: 'Archer', level: 2, at }), 'good']],
  OrderFailed: [['yours', ev({ type: 'OrderFailed', player: 0, reason: 'no' }), 'bad']],
  PlayerEliminated: [
    ['you', ev({ type: 'PlayerEliminated', player: 0 }), 'bad'],
    ['another empire', ev({ type: 'PlayerEliminated', player: 2 }), 'neutral'],
  ],
  VictoryHoldStarted: [
    ['your team reaches it', ev({ type: 'VictoryHoldStarted', team: 0, vp: 10, target: 10, turns_needed: 10 }), 'good'],
    ['a rival reaches it', ev({ type: 'VictoryHoldStarted', team: 2, vp: 10, target: 10, turns_needed: 10 }), 'bad'],
  ],
  VictoryHoldBroken: [
    ['your hold is broken', ev({ type: 'VictoryHoldBroken', team: 0, turns_held: 6, vp: 9, target: 10 }), 'bad'],
    ['a rival hold is broken', ev({ type: 'VictoryHoldBroken', team: 2, turns_held: 6, vp: 9, target: 10 }), 'good'],
  ],
  GameWon: [
    ['you win', ev({ type: 'GameWon', team: 0, reason: 'VP' }), 'good'],
    ['they win', ev({ type: 'GameWon', team: 2, reason: 'VP' }), 'bad'],
  ],
  GameDrawn: [
    ['an exact tie', ev({ type: 'GameDrawn', reason: 'Draw at the turn limit: tied on 2 VP' }), 'neutral'],
  ],
};

describe('turn report tone, from the viewer\'s side', () => {
  it.each(Object.entries(CASES).flatMap(([type, cases]) => cases.map(([label, event, tone]) => [type, label, event, tone] as const)))(
    '%s: %s is %s', (_type, _label, event, tone) => {
      expect(formatEvents([event], ctx)[0].tone).toBe(tone);
    },
  );

  it('the destroyed line of a unit you killed is good, and names who did it and from where', () => {
    const kill = battle({ defender_hp_after: 0, damage_to_defender: 100 });
    const lines = formatEvents([kill, ev({ type: 'UnitDestroyed', unit: 9, owner: 2, unit_kind: 'Spearman', at })], ctx);
    expect(lines[1]).toMatchObject({ tone: 'good', from: { q: 5, r: 1 } });
    const lost = formatEvents([enemyShot, ev({ ...enemyShot, defender_hp_after: 0 } as Record<string, unknown>),
      ev({ type: 'UnitDestroyed', unit: 7, owner: 0, unit_kind: 'Archer', at })], ctx);
    expect(lost[2].tone).toBe('bad');
    expect(lost[2].text).toBe("Your Archer was destroyed at Beacon by P2's Spearman from 2 tiles E");
  });
});

describe('"from where?" on combat lines', () => {
  it('names the attacker, its position relative to the target, and a signed HP change', () => {
    const [mine] = formatEvents([battle({})], ctx);
    expect(mine.text).toBe("Your Archer bombarded P2's Spearman at Beacon, from 2 tiles E: −35 HP (65 left)");
    expect(mine).toMatchObject({ at, from: { q: 5, r: 1 }, hp: [{ text: '−35 HP', tone: 'good' }] });
    const [theirs] = formatEvents([enemyShot], ctx);
    expect(theirs.text).toBe("P2's Spearman bombarded your Archer at Beacon, from 2 tiles E: −35 HP (65 left)");
    expect(theirs).toMatchObject({ tone: 'bad', hp: [{ text: '−35 HP', tone: 'bad' }] });
  });

  it('respects fog: an attacker whose tile the viewer never saw is "out of sight", with no arrow origin', () => {
    const [hidden] = formatEvents([battle({ attacker: 9, attacker_owner: 2, attacker_at: undefined, defender: { kind: 'Unit', unit: 7, owner: 0, unit_kind: 'Archer' } })], ctx);
    expect(hidden.text).toContain('from out of sight');
    expect(hidden.from).toBeNull();
    expect(hidden.at).toEqual(at);
  });

  it('names a striking city, and its arrow starts at the city', () => {
    const [shot] = formatEvents([battle({ attacker: { city: 5 }, attacker_owner: 2, attacker_at: { q: 1, r: 1 }, defender: { kind: 'Unit', unit: 7, owner: 0, unit_kind: 'Archer' } })], ctx);
    expect(shot.text).toBe("P2's city Elsewhere fired on your Archer at Beacon: −35 HP (65 left)");
    expect(shot).toMatchObject({ from: { q: 1, r: 1 }, at, tone: 'bad' });
  });

  it('melee names both losses, each coloured for its side', () => {
    const [m] = formatEvents([battle({ ranged: false, attacker_at: { q: 4, r: 1 }, damage_to_attacker: 12, attacker_hp_after: 88 })], ctx);
    expect(m.text).toBe("Your Archer attacked P2's Spearman at Beacon, from 1 tile E: defender −35 HP, attacker −12 HP (88 HP left). Needs orders.");
    expect(m.hp).toEqual([{ text: '−35 HP', tone: 'good' }, { text: '−12 HP', tone: 'bad' }]);
    expect(lineSegments(m).filter((s) => s.tone).map((s) => [s.text, s.tone])).toEqual([['−35 HP', 'good'], ['−12 HP', 'bad']]);
    expect(lineSegments(m).map((s) => s.text).join('')).toBe(m.text);
  });
});

describe('category chips', () => {
  it('count bad and good lines, never by colour alone', () => {
    const lines = formatEvents([enemyShot, battle({}), ev({ type: 'UnitMoved', unit: 9, owner: 2, from: at, to: at })], ctx);
    expect(toneCounts(lines)).toEqual({ good: 1, bad: 1, neutral: 1 });
    const combat = lines.filter((l) => l.category === 'combat');
    expect(chipCounts(combat).map((c) => c.text)).toEqual(['1 bad', '1 good']);
    expect(chipCounts(lines.filter((l) => l.category === 'movement')).map((c) => c.text)).toEqual(['1']);
  });
});

describe('victory hold lines', () => {
  const started = (team: number) => ev({ type: 'VictoryHoldStarted', team, vp: 10, target: 10, turns_needed: 10 });
  const broken = (team: number) => ev({ type: 'VictoryHoldBroken', team, turns_held: 6, vp: 9, target: 10 });
  const allied = { ...ctx, allied: (t: number) => t === 3 };

  it('reaching the target names the count to hold, from the viewer\'s side, and can be reopened', () => {
    const [mine] = formatEvents([started(0)], allied);
    expect(mine).toMatchObject({ category: 'victory', tone: 'good', text: 'Your team reached 10 VP and must hold it for 10 turns to win', icon: 'emblem-milestone' });
    const [rival] = formatEvents([started(2)], allied);
    expect(rival).toMatchObject({ tone: 'bad', text: 'Team 2 reached 10 VP and must hold it for 10 turns to win', reopenHold: { team: 2, vp: 10, target: 10, turnsNeeded: 10 } });
    expect(formatEvents([started(3)], allied)[0].tone).toBe('neutral');
    expect(rival.details.join(' ')).toContain('first turn held');
  });

  it('a broken hold is bad for the team and its allies and good for everyone else', () => {
    expect(formatEvents([broken(0)], allied)[0]).toMatchObject({ tone: 'bad', text: 'Your hold on 10 VP was broken after 6 of 10 turns', icon: 'ui-alert' });
    expect(formatEvents([broken(3)], allied)[0].tone).toBe('bad');
    expect(formatEvents([broken(2)], allied)[0]).toMatchObject({ tone: 'good', text: "Team 2's hold on 10 VP was broken after 6 of 10 turns" });
    const brokenEarly = ev({ type: 'VictoryHoldBroken', team: 2, turns_held: 3, vp: 9, target: 10 });
    expect(formatEvents([brokenEarly], allied)[0].text).toBe("Team 2's hold on 10 VP was broken after 3 of 10 turns");
  });
});
