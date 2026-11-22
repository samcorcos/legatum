import { BARBARIAN } from '../barbarians';
import { isGreatWork } from '../data';
import { barbariansSpareMe } from '../diplomacy';
import { distance } from '../hex';
import type { Event, PlayerView } from '../types';

// Consequential losses and outcomes outrank routine gains. Ties keep the first narrative beat.
const priority = {
  turnResolved: 0, buildingComplete: 10, vpGained: 20, battleWon: 30, techComplete: 40,
  discovery: 44, cityFounded: 45, warning: 50, vpLost: 60, battleLost: 70, greatWork: 80,
  cityCaptured: 85, cityLost: 90, warDeclared: 95, eliminated: 100, victory: 110, defeat: 110,
} as const;
export type TurnFeedback = keyof typeof priority;

/** Only private/visible view data; never rival public counts or hidden core state. */
function warnings(view: PlayerView): Set<string> {
  const result = new Set<string>();
  if (view.finished || view.players.find((p) => p.id === view.me)?.alive === false) return result;
  if (view.economy?.bankruptcy?.disband && view.economy.bankruptcy.gold.total < 0) result.add('bankruptcy');
  // Barbarians never attack the Celts, so they are no warning to them.
  const enemies = new Set([...(barbariansSpareMe(view) ? [] : [BARBARIAN]), ...view.players.filter((p) => p.team !== view.team).map((p) => p.id)]);
  for (const city of view.cities.filter((c) => c.owner === view.me)) {
    if (city.detail && city.detail.yields.total.food < city.population * 2) result.add(`starving:${city.id}`);
    if (view.units.some((u) => enemies.has(u.owner) && distance(u.pos, city.pos) === 1)) result.add(`enemy:${city.id}`);
  }
  return result;
}

/**
 * Pure mapping of the viewer's ALREADY FILTERED turn report, never TurnResult.events from a
 * local engine. The optional previous view makes forecast warnings edge-triggered across turns.
 * Deduplicate, retain the four most important calls, then restore their original narrative order.
 */
export function feedbackForTurn(events: readonly Event[], viewer: PlayerView, previous?: PlayerView): TurnFeedback[] {
  const calls = new Set<TurnFeedback>();
  if (!viewer.finished) calls.add('turnResolved');
  const priorWarnings = previous ? warnings(previous) : new Set<string>();
  for (const e of events) {
    switch (e.type) {
      case 'Battle':
        // A city's routine shot (cities fire back) only counts when it destroys the unit.
        if ((e.attacker_owner === viewer.me && e.attacker_hp_after > 0 &&
          (e.defender_hp_after <= 0 || (e.damage_to_attacker === 0 && typeof e.attacker === 'number'))) ||
          (e.defender.kind === 'Unit' && e.defender.owner === viewer.me && e.defender_hp_after > 0 &&
          (e.attacker_hp_after <= 0 || e.damage_to_defender === 0))) calls.add('battleWon');
        break;
      case 'UnitDestroyed': if (e.owner === viewer.me) calls.add('battleLost'); break;
      case 'RuinExplored': if (e.owner === viewer.me) calls.add('discovery'); break;
      case 'CityFounded': if (e.owner === viewer.me) calls.add('cityFounded'); break;
      case 'CityCaptured':
        if (e.to === viewer.me) calls.add('cityCaptured');
        if (e.from === viewer.me) calls.add('cityLost');
        break;
      case 'TechResearched': case 'EraEntered': if (e.player === viewer.me) calls.add('techComplete'); break;
      case 'ProductionCompleted':
        if (e.owner === viewer.me && (e.item.kind === 'Building' || e.item.kind === 'Wonder')) {
          calls.add(isGreatWork(e.item.building) ? 'greatWork' : 'buildingComplete');
        }
        break;
      case 'EraObjectiveScored':
        if (e.player === viewer.me) calls.add('greatWork');
        break;
      case 'FactionGoalCompleted':
      case 'GreatWorkCompleted':
        if (e.player === viewer.me && e.vp > 0) calls.add('again' in e && e.again ? 'vpGained' : 'greatWork');
        break;
      case 'FactionGoalLost':
      case 'GreatWorkLost':
        if (e.player === viewer.me) calls.add('vpLost');
        break;
      case 'CityStarved':
        if (e.owner === viewer.me && !priorWarnings.has(`starving:${e.city}`)) calls.add('warning');
        break;
      case 'WarDeclared': {
        // Another empire declared war on the viewer's team (it also gets the full-screen alarm).
        const r = e.relation;
        if ([r.a, r.b].includes(viewer.team) && !r.declarers.includes(viewer.team)) calls.add('warDeclared');
        break;
      }
      case 'VictoryHoldStarted':
        // Everyone is told: your own hold is a triumph, a rival's is the same alarm as a war on you.
        calls.add(e.team === viewer.team ? 'greatWork' : 'warDeclared');
        break;
      case 'VictoryHoldBroken': {
        // A broken hold hurts that team and its allies and relieves everyone else.
        const allied = viewer.diplomacy?.relations.some((r) => r.state === 'Alliance' && ((r.a === viewer.team && r.b === e.team) || (r.b === viewer.team && r.a === e.team)));
        calls.add(e.team === viewer.team || allied ? 'vpLost' : 'vpGained');
        break;
      }
      case 'PlayerEliminated': if (e.player === viewer.me) calls.add('eliminated'); break;
      case 'GameWon': calls.add(e.team === viewer.team ? 'victory' : 'defeat'); break;
    }
  }
  if ([...warnings(viewer)].some((key) => !priorWarnings.has(key))) calls.add('warning');
  return [...calls].map((name, index) => ({ name, index }))
    .sort((a, b) => priority[b.name] - priority[a.name] || a.index - b.index).slice(0, 4)
    .sort((a, b) => a.index - b.index).map(({ name }) => name);
}
