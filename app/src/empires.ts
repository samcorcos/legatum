// Empire identity for diplomacy: who an empire is (faction, seat, map color) and where the
// viewer knows it to be. Everything here reads only the viewer's PlayerView, so nothing fogged leaks:
// cities are the visible and remembered ones, units only those in sight this turn.
import { factionName, itemName } from './data';
import { distance, hexToPixel } from './hex';
import type { Faction, Hex, PlayerPublic, PlayerView, TeamId } from './types';

export function empireMembers(view: PlayerView, team: TeamId): PlayerPublic[] {
  return view.players.filter((p) => p.team === team);
}

/** Empires still in the game. Eliminated ones belong only to history (reports, event log, final standings). */
export function livingPlayers(view: Pick<PlayerView, 'players'>): PlayerPublic[] {
  return view.players.filter((p) => p.alive);
}

/** Whether any member of the team is still in the game. */
export function isTeamAlive(view: Pick<PlayerView, 'players'>, team: TeamId): boolean {
  return view.players.some((p) => p.team === team && p.alive);
}

type Named = Pick<PlayerPublic, 'id' | 'name' | 'faction' | 'team'>;

/**
 * The one way an empire is named in text: its faction ("Switzerland"), plus " · name" only when
 * another team plays the same faction.
 * A two-empire team: "India & Byzantium".
 */
export function empireNameOf(members: Named[], everyone: Named[] = members): string {
  return members.map((p) => {
    const twin = everyone.some((o) => o.id !== p.id && o.team !== p.team && o.faction === p.faction);
    return twin && p.name !== factionName(p.faction) ? `${factionName(p.faction)} · ${p.name}` : factionName(p.faction);
  }).join(' & ');
}

export function empireTitle(view: PlayerView, team: TeamId): string {
  return empireNameOf(empireMembers(view, team), view.players) || `Team ${team}`;
}

/** One player's empire in text ("Switzerland", or "Switzerland · AI 7" when two teams share the faction). */
export function empireOfPlayer(view: Pick<PlayerView, 'players'>, id: number): string {
  const p = view.players.find((x) => x.id === id);
  return p ? empireNameOf([p], view.players) : `Empire ${id + 1}`;
}

const COMPASS = ['E', 'NE', 'N', 'NW', 'W', 'SW', 'S', 'SE'] as const;

/** 8-point compass bearing from `from` to `to` on the pointy-top map (north is up). */
export function bearing(from: Hex, to: Hex): (typeof COMPASS)[number] {
  const a = hexToPixel(from, 1), b = hexToPixel(to, 1);
  const angle = Math.atan2(a.y - b.y, b.x - a.x);
  return COMPASS[((Math.round(angle / (Math.PI / 4)) % 8) + 8) % 8];
}

/**
 * A tile named by the nearest city the viewer knows, instead of raw coordinates:
 * "Brasslock", "1 tile NE of Brasslock", or null when no known city is within `range`.
 */
export function placeName(view: Pick<PlayerView, 'cities'>, hex: Hex, range = 6): string | null {
  let best: { name: string; pos: Hex; tiles: number } | null = null;
  for (const c of view.cities) {
    const tiles = distance(c.pos, hex);
    if (tiles <= range && (!best || tiles < best.tiles)) best = { name: c.name, pos: c.pos, tiles };
  }
  if (!best) return null;
  return best.tiles === 0 ? best.name : `${best.tiles} tile${best.tiles === 1 ? '' : 's'} ${bearing(best.pos, hex)} of ${best.name}`;
}

export interface EmpireLocation {
  /** Where "Show on map" centers. */
  at: Hex;
  kind: 'city' | 'unit';
  /** Their city's name, or their unit's name ("Spearman"). */
  name: string;
  /** Hex distance to the nearest place of ours. */
  tiles: number;
  direction: (typeof COMPASS)[number];
  /** Our city (or "your Settler") it is measured from; null when we have nothing on the map. */
  from: string | null;
}

/** Nearest known city of `team` (else nearest unit in sight), measured from our nearest city (else unit). */
export function empireLocation(view: PlayerView, team: TeamId): EmpireLocation | null {
  const owners = new Set(empireMembers(view, team).map((p) => p.id));
  const ours = new Set(empireMembers(view, view.team).map((p) => p.id));
  const faction = (owner: number) => view.players.find((p) => p.id === owner)?.faction;
  const theirCities = view.cities.filter((c) => owners.has(c.owner));
  const theirs = theirCities.length
    ? theirCities.map((c) => ({ at: c.pos, kind: 'city' as const, name: c.name }))
    : view.units.filter((u) => owners.has(u.owner)).map((u) => ({ at: u.pos, kind: 'unit' as const, name: itemName({ kind: 'Unit', unit: u.kind }, faction(u.owner)) }));
  if (!theirs.length) return null;
  const ourCities = view.cities.filter((c) => ours.has(c.owner));
  const anchors = ourCities.length
    ? ourCities.map((c) => ({ at: c.pos, name: c.name }))
    : view.units.filter((u) => ours.has(u.owner)).map((u) => ({ at: u.pos, name: `your ${itemName({ kind: 'Unit', unit: u.kind }, faction(u.owner))}` }));
  if (!anchors.length) return { ...theirs[0], tiles: 0, direction: 'N', from: null };
  let best: EmpireLocation | null = null;
  for (const t of theirs) {
    for (const a of anchors) {
      const tiles = distance(a.at, t.at);
      if (!best || tiles < best.tiles) best = { ...t, tiles, direction: bearing(a.at, t.at), from: a.name };
    }
  }
  return best;
}

/** One line: "Nearest city Harmonia ~16 tiles E of Astra". */
export function describeLocation(loc: EmpireLocation | null): string {
  if (!loc) return 'Location unknown: not met yet';
  const where = loc.from === null ? '' : ` ~${loc.tiles} tile${loc.tiles === 1 ? '' : 's'} ${loc.direction} of ${loc.from}`;
  return loc.kind === 'city' ? `Nearest city ${loc.name}${where}` : `City not seen yet; their ${loc.name}${where}`;
}

/** Their units in sight within `radius` of our units or cities: what a no-attack treaty protects. */
export function unitsNearOurs(view: PlayerView, team: TeamId, radius = 3) {
  const owners = new Set(empireMembers(view, team).map((p) => p.id));
  const ours = new Set(empireMembers(view, view.team).map((p) => p.id));
  const faction = (owner: number) => view.players.find((p) => p.id === owner)?.faction;
  const anchors = [
    ...view.cities.filter((c) => ours.has(c.owner)).map((c) => ({ at: c.pos, name: c.name })),
    ...view.units.filter((u) => ours.has(u.owner)).map((u) => ({ at: u.pos, name: `your ${itemName({ kind: 'Unit', unit: u.kind }, faction(u.owner))}` })),
  ];
  const near: { unit: string; at: Hex; tiles: number; near: string }[] = [];
  for (const u of view.units.filter((u) => owners.has(u.owner))) {
    let closest: { tiles: number; name: string } | null = null;
    for (const a of anchors) {
      const tiles = distance(a.at, u.pos);
      if (tiles <= radius && (!closest || tiles < closest.tiles)) closest = { tiles, name: a.name };
    }
    if (closest) near.push({ unit: itemName({ kind: 'Unit', unit: u.kind }, faction(u.owner)), at: u.pos, tiles: closest.tiles, near: closest.name });
  }
  return near.sort((a, b) => a.tiles - b.tiles);
}

export const PEOPLE_NAME: Record<Faction, string> = {
  Switzerland: 'Swiss', Mongols: 'Mongols', Sparta: 'Spartans', Germany: 'Germans',
  Aztecs: 'Aztecs', India: 'Indians', Byzantium: 'Byzantines', Celts: 'Celts', Athens: 'Athenians', Egypt: 'Egyptians', Rome: 'Romans',
};

/** A people's plural name in a headline, such as "the Swiss and the Indians". */
export function empireHeadName(view: PlayerView, team: TeamId): string {
  const names = empireMembers(view, team).map((p) => `the ${PEOPLE_NAME[p.faction]}`);
  return names.length ? names.join(' and ') : `Team ${team}`;
}
