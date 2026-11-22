// What razing a captured city destroys, from the core's raze rule (`rules/cities.rs::apply_order`, Order::Raze): the city and every
// building in it, the improvements and landmarks on its tiles (a world wonder or a goal building goes with its held VP), and the
// aircraft based there. The confirm lists exactly these, so a wonder never burns unannounced.
import { BUILDING_NAME, GREAT_WORK_VP, factionGoalVp, isGreatWork, itemName } from './data';
import { tileIndex } from './hex';
import { GOAL_CITY_FACTION, GOAL_CITY_TITLE, goalCitiesAt } from './goalCities';
import type { BuildingKind, CityView, Faction, PlayerView } from './types';

export interface RazeLosses {
  /** World wonders in the city's tiles, with the held VP each costs its holder. */
  works: { name: string; vp: number }[];
  /** Capturable goal buildings (the Yam Relay Station, the Imperial Diwan) and goal cities (the Colonia Victrix, the Royal Tomb), with the goal VP each carries. */
  goals: { name: string; vp: number }[];
  buildings: string[];
  /** Improved tiles among every tile the city owns. */
  improved: number;
  aircraft: number;
}

/** The faction whose goal a capturable goal building scores. */
const GOAL_BUILDING_FACTION: Partial<Record<BuildingKind, Faction>> = { YamRelayStation: 'Mongols', ImperialDiwan: 'India' };

const named = (name: string) => (/^Hagia /.test(name) ? name : `the ${name}`);

export function razeLosses(view: PlayerView, city: CityView): RazeLosses {
  const owner = view.players.find((p) => p.id === city.owner)?.faction;
  const landmarks = view.tiles.flatMap((t) => (t.landmark && t.landmark.city === city.id ? [t.landmark.building] : []));
  const detail = city.detail;
  return {
    works: landmarks.filter(isGreatWork).map((b) => ({ name: BUILDING_NAME[b], vp: GREAT_WORK_VP })),
    goals: [...landmarks.filter((b) => !isGreatWork(b)).map((b) => ({ name: BUILDING_NAME[b], vp: GOAL_BUILDING_FACTION[b] ? factionGoalVp(GOAL_BUILDING_FACTION[b]!) : 0 })),
      // Rome's Colonia and Egypt's tomb are the city itself: razing it ends the goal and its held VP.
      ...goalCitiesAt(view, city.id).map((g) => ({ name: GOAL_CITY_TITLE[g.kind], vp: factionGoalVp(GOAL_CITY_FACTION[g.kind]) }))],
    buildings: (detail?.buildings ?? []).map((b) => itemName({ kind: 'Building', building: b }, owner)),
    improved: (detail?.territory ?? []).filter((h) => view.tiles[tileIndex(h, view.width, view.height)]?.improvement).length,
    aircraft: (view.aircraft ?? []).filter((a) => a.city === city.id).length,
  };
}

/** "the Pyramids (−2 VP)": a wonder or goal building and the points it carries. */
export const razeWorkText = (w: { name: string; vp: number }): string => `${named(w.name)}${w.vp ? ` (−${w.vp} VP)` : ''}`;

/** One line per kind of loss, wonders first. Empty kinds are left out. */
export function razeLossLines(l: RazeLosses): string[] {
  const plural = (n: number, one: string, many = `${one}s`) => `${n} ${n === 1 ? one : many}`;
  return [
    ...l.works.map(razeWorkText),
    ...l.goals.map(razeWorkText),
    ...(l.buildings.length ? [`${plural(l.buildings.length, 'building')}: ${l.buildings.join(', ')}`] : []),
    ...(l.improved ? [`${plural(l.improved, 'improvement')} on its tiles`] : []),
    ...(l.aircraft ? [`${plural(l.aircraft, 'aircraft', 'aircraft')} based there`] : []),
  ];
}
