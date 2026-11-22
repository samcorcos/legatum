import { hasBuilding, RULES } from '../data';
import { hexAt, tileIndex } from '../hex';
import type { MState } from './fixtureState';

// Translate fixture data only, including what each team has already observed.
// Write bitset words as exact decimal u64s, never lossy JavaScript numbers. After
// loading, WASM save strings stay opaque on every round trip.
// Fixtures use the current core save version.
export function coreFixture(s: MState): string {
  return JSON.stringify({
    save_version: RULES.saveVersion,
    ...(s.barbarians ? { barbarians: s.barbarians } : {}),
    diplomacy: s.diplomacy,
    setup: s.setup, turn: s.turn, winner: s.winner, finished: s.finished,
    ...(s.wondersDiscovered?.length ? { wonders_discovered: s.wondersDiscovered.map((i) => hexAt(i, s.width)) } : {}),
    map: { width: s.width, height: s.height, tiles: s.terrain.map((terrain, i) => ({
      terrain, ruin: !!s.ruin[i], owner: s.owner[i] < 0 ? null : s.owner[i], city: s.tileCity[i] < 0 ? null : s.tileCity[i],
      ...(s.wonder?.[i] ? { wonder: s.wonder[i] } : {}),
      ...(s.landmark?.[i] ? { landmark: s.landmark[i] } : {}),
      ...(s.resource?.[i] ? { resource: s.resource[i] } : {}),
      ...(s.improved?.[i] ? { improved: true } : {}),
      ...(s.road?.[i] ? { road: s.road[i] } : {}),
    })), ...(s.roadHub ? { road_hub: s.roadHub } : {}), ...(s.roaded ? { roaded: true } : {}) },
    players: s.players.map((p) => {
      const words = Array<bigint>(Math.ceil(s.terrain.length / 64)).fill(0n);
      for (let i = 0; i < p.explored.length; i++) if (p.explored[i]) words[Math.floor(i / 64)] |= 1n << BigInt(i % 64);
      const observedCities = new Map(Object.values(s.knownCities?.[p.team] ?? {}).map((c) => [tileIndex(c.pos, s.width, s.height), c]));
      const tile_memory = p.explored.map((explored, i) => {
        if (!explored && !s.knownTerrain?.[p.team]?.[i]) return null;
        const c = observedCities.get(i), owner = s.knownOwner[p.team]?.[i] ?? -1;
        return {
          owner: owner < 0 ? null : owner, ruin: s.knownRuins?.[p.team]?.[i] === 1,
          ...(s.knownImproved?.[p.team]?.[i] === 1 ? { improved: true } : {}),
          city: c ? { id: c.id, owner: c.owner, name: c.name, population: c.population, hp: c.hp,
            walls: hasBuilding(c.buildings, 'Walls'), citadel: hasBuilding(c.buildings, 'Castle'), star_fort: hasBuilding(c.buildings, 'StarFort'), is_capital: c.isCapital } : null,
        };
      });
      return {
        id: p.id, name: p.name, faction: p.faction, team: p.team, is_human: p.isHuman, alive: p.alive,
        gold: p.gold, science_stored: p.science, researching: p.research, research_queue: p.research_queue, techs: p.techs,
        explored: { len: s.terrain.length, words }, tile_memory, ruins_explored: 0,
        settlers_trained: p.settlersTrained, military_purchases: p.militaryPurchases ?? 0, science_history: p.scienceHistory,
        ...(p.eras ? { eras: p.eras } : {}), faction_goal_done: p.factionGoalDone,
        ...(p.factionGoalReachedBefore ? { faction_goal_reached_before: true } : {}),
        ...(p.goalHolder !== undefined && p.goalHolder !== null ? { goal_holder: p.goalHolder } : {}),
        ...(p.hallCity !== undefined && p.hallCity !== null ? { hall_city: p.hallCity } : {}),
        ...(p.diwanCity !== undefined && p.diwanCity !== null ? { diwan_city: p.diwanCity } : {}),
        ...(p.coloniaCity !== undefined && p.coloniaCity !== null ? { colonia_city: p.coloniaCity } : {}),
        ...(p.tombWork ? { tomb_work: true } : {}),
        ...(p.tombTurns ? { tomb_turns: p.tombTurns } : {}),
        ...(p.tombCity !== undefined && p.tombCity !== null ? { tomb_city: p.tombCity } : {}),
        ...(p.freeTechsPending ? { free_techs_pending: p.freeTechsPending } : {}),
        ...(p.veteransRetired ? { veterans_retired: p.veteransRetired } : {}),
        ...(p.sakokuTurns !== undefined && p.sakokuTurns !== null ? { sakoku_turns: p.sakokuTurns } : {}),
        ...(p.goalRule ? { goal_rule: p.goalRule } : {}),
        ...(p.freeTechsFrom !== undefined && p.freeTechsFrom !== null ? { free_techs_from: p.freeTechsFrom } : {}),
        great_work_completed: p.greatWorkCompleted ?? null,
        ...(p.greatWorkHolder !== undefined ? { great_work_holder: p.greatWorkHolder } : {}),
        // Core omits an empty supply record list and a false `idle`.
        ...(p.supply?.length ? { supply: p.supply.map((r) => ({
          user: r.user.kind === 'Building' ? { kind: 'building', city: r.user.city, building: r.user.building } : { kind: 'unit', unit: r.user.unit },
          resource: r.resource, since: r.since, ...(r.idle ? { idle: true } : {}),
        })) } : {}),
      };
    }),
    cities: Object.fromEntries(s.cities.map((c) => [c.id, {
      id: c.id, owner: c.owner, founder: c.founder, name: c.name, pos: c.pos, population: c.population,
      food_stored: c.food, production_stored: c.prod, producing: c.producing, bought: c.bought,
      // Core omits an empty bank.
      ...(c.productionBanked?.length ? { production_banked: c.productionBanked } : {}),
      production_queue: c.productionQueue ?? [], production_queue_notified: c.productionQueueNotified ?? [],
      last_purchase_turn: c.lastPurchaseTurn ?? null,
      ...(c.lastAttackedTurn !== undefined ? { last_attacked_turn: c.lastAttackedTurn } : {}),
      buildings: c.buildings, hp: c.hp, grown_tiles: c.grownTiles, tiles_claimed: c.tilesClaimed,
      is_capital: c.isCapital, ...(c.originalCapital ? { original_capital: true } : {}), founded_turn: c.foundedTurn, resistance: c.resistance,
    }])),
    units: Object.fromEntries(s.units.map((u) => [u.id, {
      id: u.id, owner: u.owner, kind: u.kind, pos: u.pos, hp: u.hp, moves_left: u.moves,
      goto: u.goto, path: u.path ?? [], fortify: u.fortify, acted: u.acted,
      path_stop_on_block: u.path_stop_on_block ?? false,
      automation: u.automation ?? null, needs_orders: u.needs_orders ?? false,
      standing_order: u.standing_order ?? null, xp: u.xp ?? 0,
      ...(u.siege ? { siege: true } : {}),
      ...(u.heir_of !== undefined ? { heir_of: u.heir_of } : {}),
      ...(u.contract_until !== undefined ? { contract_until: u.contract_until, origin: u.origin } : {}),
      ...(u.attack_target !== undefined ? { attack_target: u.attack_target } : {}),
    }])),
    next_unit_id: s.nextUnitId, next_city_id: s.nextCityId,
    // Core omits the map while nobody holds a VP target.
    ...(s.vpHolds && Object.keys(s.vpHolds).length ? { vp_holds: s.vpHolds } : {}),
    ...(s.eraFirsts && Object.keys(s.eraFirsts).length ? { era_firsts: s.eraFirsts } : {}),
  }, (_, value) => typeof value === 'bigint' ? { fixture_u64: value.toString() } : value)
    .replace(/\{"fixture_u64":"(\d+)"\}/g, '$1');
}
