import type { Faction } from './types';

/**
 * Each faction keeps one colour in every game (the way Civ gives each leader a preferred colour):
 * Sparta is always crimson, Athens azure. `color` is the bright UI colour for the map and dark
 * panels; `ink` is its shade for ink on parchment: outlines, bars, bands and glyphs. The bright
 * colours are faint against the paper (steel 1.1:1, lime 1.4:1, jade 1.5:1, amber 1.7:1), so each
 * ink keeps its hue recognisable at 3:1 or better on --panel, and none sits within 20 degrees of
 * the gilt (#8f6f1c), which is reserved for "yours". Fills and tints keep the bright colour itself.
 */
export const FACTION_PALETTE: Record<Faction, { color: string; ink: string }> = {
  Sparta: { color: '#e5534b', ink: '#cf3d35' }, // crimson
  Athens: { color: '#4aa8ff', ink: '#1b6fb8' }, // azure
  Byzantium: { color: '#a97ff0', ink: '#7c4fd0' }, // violet, the imperial purple
  Celts: { color: '#3fd6a4', ink: '#168a63' }, // jade
  Mongols: { color: '#f2a63b', ink: '#c4571a' }, // amber, its ink shifted toward orange so it never reads as the gilt that means "yours"
  India: { color: '#e86fb8', ink: '#c23f92' }, // rose
  Aztecs: { color: '#b9d24a', ink: '#6b8a1f' }, // lime
  Germany: { color: '#d8dde3', ink: '#5f6b7a' }, // steel
  Switzerland: { color: '#1fa3a8', ink: '#12858c' }, // deep teal
  Egypt: { color: '#3aa774', ink: '#1e6549' }, // Nile green
  Rome: { color: '#b87f67', ink: '#84462d' }, // terracotta
};

/**
 * The faction whose colour each empire wears, by player id. Normally its own faction; when two
 * empires share a faction (the lobby allows it), the later one takes the colour of a faction that
 * is not in the game, so no two empires ever share a colour.
 */
export function colorFactions(factions: readonly Faction[]): Faction[] {
  const free = (Object.keys(FACTION_PALETTE) as Faction[]).filter((f) => !factions.includes(f));
  const seen = new Set<Faction>();
  return factions.map((f) => {
    if (!seen.has(f)) { seen.add(f); return f; }
    return free.shift() ?? f;
  });
}

let seatFactions: Faction[] = [];

/** The game on screen: which faction's colour each player id wears (see colorFactions). Set by the store whenever a game's players are known. */
export function setSeatFactions(factions: readonly Faction[]) {
  seatFactions = colorFactions(factions);
}

/** The faction whose colour this player id wears, or undefined for barbarians, unknown ids and no game. */
export const seatColorFaction = (id: number | null | undefined): Faction | undefined =>
  id === null || id === undefined ? undefined : seatFactions[id];

/** Barbarians (core PlayerId::BARBARIAN = 255): brick red on black, never a seat color. */
export const BARBARIAN_COLOR = '#b3261e';

export const factionColor = (f: Faction) => FACTION_PALETTE[f].color;

/** An empire's colour: its faction's, resolved from the game on screen. Barbarians keep brick red; no owner or no game is grey. */
export const playerColor = (id: number | null | undefined) => {
  const f = seatColorFaction(id);
  return id === 255 ? BARBARIAN_COLOR : f ? factionColor(f) : '#888';
};

/** The parchment-safe shade of a faction's colour (see FACTION_PALETTE). */
export const factionInk = (f: Faction) => FACTION_PALETTE[f].ink;

/** The parchment-safe shade of an empire's colour; barbarians and unknowns keep their colour. */
export const playerInk = (id: number | null | undefined) => {
  const f = seatColorFaction(id);
  return id === 255 ? BARBARIAN_COLOR : f ? factionInk(f) : '#888';
};

/** The parchment-safe shade of a faction colour given as a hex (a crest's ring or glyph); any other colour passes through. */
export const inkOfColor = (color: string): string => {
  const entry = Object.values(FACTION_PALETTE).find((p) => p.color === color);
  return entry ? entry.ink : color;
};

/**
 * Health at a glance, the same on every surface (map tokens, city bars, the
 * unit panel): green when healthy, through amber, to red when low. Readable on paper and lapis.
 */
export function healthColor(fraction: number): string {
  const f = Math.max(0, Math.min(1, fraction));
  if (f > 0.6) return '#3f9b3a';
  if (f > 0.3) return '#e0a526';
  return '#d6362b';
}


/** Tiny territory edges cannot fit numbers: keep a distinct pattern for each player. */
export const PLAYER_BORDER_DASH = [[], [6, 3], [2, 3], [8, 3, 2, 3], [10, 4], [4, 3, 4, 7], [8, 3, 2, 3, 2, 3], [2, 3, 2, 7]];

/** Yield inks from brand/tokens.css. CSS values: use in styles, not on canvas. */
export const YIELD_COLORS = {
  food: 'var(--y-food)',
  production: 'var(--y-production)',
  gold: 'var(--y-gold)',
  science: 'var(--y-science)',
} as const;

function hexToRgb(c: string): [number, number, number] {
  const n = parseInt(c.slice(1), 16);
  return [(n >> 16) & 255, (n >> 8) & 255, n & 255];
}

export function rgba(c: string, a: number): string {
  const [r, g, b] = hexToRgb(c);
  return `rgba(${r},${g},${b},${a})`;
}
