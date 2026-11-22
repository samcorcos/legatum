// Atlas map palette for the painterly terrain pipeline (terrain.ts). Lengths are in mockup pixels
// (hex radius 27) and are scaled to world units by K at draw time.
import type { Faction, SiteBand, Terrain } from '../types';
import type { LookId } from '../brand/look';
import { factionColor, seatColorFaction } from '../theme';

/**
 * One bold border look. From the boundary inward the layers are: an ink edge (`ink`), a lighter
 * outer line (`outer`, a tint of the seat colour), the seat-colour line (`line`), then a band that
 * fades out over `band`. Each empire draws only on its own side of a shared boundary, so two
 * neighbours sit side by side instead of overpainting each other. `shared` widens the ink edge on a
 * boundary with another empire.
 */
export interface BorderSpec {
  tint: number;
  band: number; bandMin: number; bandAlpha: number;
  line: number; lineMin: number;
  ink: number; inkMin: number; inkAlpha: number;
  outer: number; outerMin: number; outerMix: number;
  shared: number;
}

export interface MapStyle {
  id: LookId;
  pal: Record<Terrain, string>;
  deep: string;
  shallow: string;
  seaMottle: number;
  mottleBlend: GlobalCompositeOperation;
  light: string;
  shadow: string;
  relief: number;
  grain: number;
  mottle: number;
  /** Land color blend blur (mockup px). */
  blend: number;
  tree: { dark: string; light: string; shadow: string };
  peak: { light: string; dark: string; snow: string; line: string; base: string };
  hill: { fill: string; line: string };
  dune: string;
  snow: string;
  tuft: { grass: string; plains: string };
  wave: string;
  town: { ground: string; roof: string; roofLight: string; shadow: string };
  grid: string;
  /** The grid line over forest, hill and mountain art (organic terrain): a pale casing under an ink core, widths in CSS px. */
  gridArt: { light: string; dark: string; casing: number; core: number };
  gridWater: string;
  coast: string;
  coastWidth: number;
  waveLine: string;
  waveAlpha: number;
  unexplored: string;
  unexploredEdge: string;
  unexploredBlur: number;
  unexploredPattern: 'hatch' | 'scales';
  unexploredInk: string;
  fog: {
    desat: number;
    tint: string;
    alpha: number;
    blend: GlobalCompositeOperation;
    hatch: string;
    hatchAlpha: number;
    edge: string;
    glyph: number;
  };
  border: { width: number; band: number; bandAlpha: number; tint: number; blur: number };
  /** The bolder border looks (see `borderVariant.ts`): every width is a world px, with a floor in CSS px so the line never thins out zoomed out. */
  borders: Record<'a' | 'b' | 'c', BorderSpec>;
  vignette: number;
  vignetteColor: string;
  /** Empire colours on the map are tints: the faction's UI colour mixed with umber so the hue stays and the chroma settles into the atlas palette. */
  seat: { umber: string; mix: number; paper: string; plate: string; plateMilitary: string; border: Record<Faction, string> };
  /** The pen-and-parchment inks every sprite and overlay shares (counters, banners, tags, halos, cues). */
  tone: { ink: string; ink2: string; paper: string; vellum: string; rule: string; red: string; light: string; shadow: string; good: string; bad: string; gold: string; goldNotable: string; growth: string; production: string; chevron: string; chevronEdge: string; iron: string; track: string; wash: string; attackWash: string };
  /** Barbarians: an aged plate hatched dark, ink rims and the one accent red. */
  barb: { plate: string; ink: string; blood: string };
  /** Screen-space pen weights (CSS px), the one set every map sprite and overlay draws with. */
  pen: { hair: number; thin: number; line: number; heavy: number };
  /** The Courier Hall's landmark (`courierHall.ts`): rhumb lines across the holder's land and the compass-rose plaque. */
  hall: { plaque: number; plaqueRim: number; rhumbStart: number; rhumbEnd: number; rhumbStartFar: number; rhumbEndFar: number; remembered: number; reach: number };
  /** Lens washes: translucent over the terrain, with only a thin accent line stronger. */
  lens: { wash: number; faded: number; rim: number; rimAlpha: number; ink: string; paper: string; ownerWash: number; badge: string; badgeText: string; settle: Record<SiteBand, string>; /** How much of each band's colour covers the ground: Poor is a faint tint, Exceptional nearly opaque. */ settleAlpha: Record<SiteBand, number>; tint: { ruin: string; easy: string; rough: string; blocked: string; threat: string; edge: string } };
  /** Resource categories shared by far-zoom marks and the Resources overlay. */
  resources: { bonus: string; strategic: string };
  /** Overlay inks: route, selection, reach, labels. */
  ink: {
    route: string;
    routeShadow: string | null;
    routeDash: number[] | null;
    attack: string;
    select: string;
    reachFill: string;
    reachEdge: string;
    reachDash: number[] | null;
    hover: string;
    marker: string;
    markerFill: string;
    sea: string;
    land: string;
    ruin: string;
  };
}

export const ATLAS: MapStyle = {
  id: 'atlas',
  pal: {
    Ocean: '#93b6bf',
    Coast: '#b4cfd1',
    Grassland: '#b3bf85',
    Plains: '#d8cb93',
    Forest: '#8ea473',
    Hills: '#cdb283',
    Desert: '#e8d4a0',
    Tundra: '#d9dcd2',
    Mountains: '#c2b194',
  },
  deep: '#8fb1ba',
  shallow: '#bcd6d6',
  seaMottle: 0.22,
  mottleBlend: 'soft-light',
  light: '#fffbea',
  shadow: '#4a3920',
  relief: 0.32,
  grain: 0.1,
  mottle: 0.35,
  blend: 5,
  tree: { dark: '#5b7649', light: '#8aa56b', shadow: 'rgba(70,55,30,.22)' },
  peak: { light: '#f1e8d4', dark: '#a8967a', snow: '#fcf9f1', line: '#4a3e2e', base: 'rgba(80,60,30,.16)' },
  hill: { fill: 'rgba(120,90,50,.1)', line: '#7c6547' },
  dune: '#b09459',
  snow: 'rgba(255,255,255,.7)',
  tuft: { grass: '#7c8f55', plains: '#a89a5e' },
  wave: 'rgba(60,100,110,.4)',
  town: { ground: '#eadcc0', roof: '#9d4b32', roofLight: '#cf7d5e', shadow: 'rgba(60,40,20,.35)' },
  grid: 'rgba(70,52,28,.14)',
  gridArt: { light: 'rgba(246,238,218,.42)', dark: 'rgba(44,32,16,.46)', casing: 2.6, core: 1 },
  gridWater: 'rgba(40,80,90,.1)',
  coast: '#3c4745',
  coastWidth: 1.1,
  waveLine: '#557f89',
  waveAlpha: 0.7,
  unexplored: '#ebe2cb',
  unexploredEdge: 'rgba(60,45,25,.55)',
  unexploredBlur: 1.5,
  unexploredPattern: 'hatch',
  unexploredInk: 'rgba(110,85,50,.14)',
  // Remembered terrain reads as faded ink: a little color drained, a paper wash, a light hatch.
  fog: {
    desat: 0.2,
    tint: '#f5eedb',
    alpha: 0.34,
    blend: 'source-over',
    hatch: 'rgba(120,96,60,.3)',
    hatchAlpha: 0.45,
    edge: 'rgba(60,45,25,.25)',
    glyph: 0.75,
  },
  border: { width: 1.3, band: 8, bandAlpha: 0.32, tint: 0.07, blur: 3.5 },
  borders: {
    // A: a bold seat-colour line with a thin ink edge and a narrow inner band, about a third of a hex deep.
    a: { tint: 0.08, band: 12, bandMin: 5, bandAlpha: 0.55, line: 3.4, lineMin: 2.6, ink: 1.1, inkMin: 1, inkAlpha: 0.85, outer: 0, outerMin: 0, outerMix: 0, shared: 1 },
    // B: Civ-style two-tone: the seat colour inside, a lighter line of it outside, and a soft tint over the whole territory.
    b: { tint: 0.2, band: 10, bandMin: 4, bandAlpha: 0.3, line: 2.8, lineMin: 2.2, ink: 0, inkMin: 0, inkAlpha: 0, outer: 1.7, outerMin: 1.3, outerMix: 0.62, shared: 1 },
    // C: A with a clear tint over the whole territory and a heavier ink rule where two empires meet.
    c: { tint: 0.18, band: 14, bandMin: 6, bandAlpha: 0.5, line: 3.4, lineMin: 2.6, ink: 1.1, inkMin: 1, inkAlpha: 0.9, outer: 0, outerMin: 0, outerMix: 0, shared: 1.9 },
  },
  vignette: 0.28,
  vignetteColor: '#5a4526',
  seat: {
    umber: '#3b2a1a', mix: 0.3, paper: '#f2ead8', plate: '#f7f1e3', plateMilitary: '#e9dfc6',
    // Border lines and washes: each faction's hue family (theme.ts FACTION_PALETTE), with the lightness staggered so that no
    // two factions collapse together for protan, deutan or tritan sight (seatColors.test.ts checks it).
    border: {
      Athens: '#4f93ff', Celts: '#37c095', Sparta: '#b7000f', Mongols: '#bd8625', Byzantium: '#7e3298', Egypt: '#329664', Rome: '#8c4b46',
      India: '#f255a7', Aztecs: '#c1c21c', Germany: '#727272', Switzerland: '#065a66',
    },
  },
  tone: {
    ink: '#2b2419', ink2: '#5b503f', paper: '#f2ead8', vellum: '#f7f1e3', rule: '#9d8c6c', red: '#9a3322', light: '#fbf6ea',
    shadow: 'rgba(50,38,20,.28)', good: '#2f7a2b', bad: '#d6362b', gold: '#80620d', goldNotable: '#276b78',
    growth: '#4e762e', production: '#965020', chevron: '#f2cf62', chevronEdge: '#e9c35a', iron: '#d6d0c3', track: '#dfd2b5',
    wash: 'rgba(80, 82, 82, 0.20)', attackWash: 'rgba(80, 82, 82, 0.65)',
  },
  barb: { plate: '#d8cdb3', ink: '#2b2419', blood: '#9a3322' },
  pen: { hair: 0.6, thin: 1, line: 1.5, heavy: 2.2 },
  hall: { plaque: 0.5, plaqueRim: 0.85, rhumbStart: 0.62, rhumbEnd: 0.18, rhumbStartFar: 0.5, rhumbEndFar: 0.14, remembered: 0.6, reach: 6.2 },
  lens: { wash: 0.6, faded: 0.38, rim: 1, rimAlpha: 0.8, ink: '#2b2419', paper: '#f5eedb', ownerWash: 0.3, badge: '#704509', badgeText: '#fff8df', settle: { Poor: '#17a05e', Fair: '#0e8a4c', Good: '#097238', Great: '#06552c', Exceptional: '#033820' }, settleAlpha: { Poor: 0.26, Fair: 0.5, Good: 0.66, Great: 0.77, Exceptional: 0.84 }, tint: { ruin: '#ad7924', easy: '#268064', rough: '#b27936', blocked: '#555566', threat: '#bc3949', edge: '#287f9d' } },
  resources: { bonus: '#3f8a5c', strategic: '#b3392b' },
  ink: {
    route: '#9a3322',
    routeShadow: null,
    routeDash: [6, 4],
    attack: '#a3392a',
    select: '#9a3322',
    reachFill: 'rgba(255,250,235,.28)',
    reachEdge: '#2b2419',
    reachDash: [1.5, 3],
    hover: 'rgba(43,36,25,.7)',
    marker: '#9a3322',
    markerFill: '#f4ecd9',
    sea: 'rgba(40,70,78,.75)',
    land: 'rgba(70,52,28,.55)',
    ruin: '#6b4a2a',
  },
};

export const MAP_STYLES: Record<LookId, MapStyle> = { atlas: ATLAS };

/** An empire's colour as the map draws it (counter rims, borders, roots, rings): its faction's hue kept, chroma toned down. Hex, so `rgba()` works on it. */
export function mapPlayerColor(owner: number | null | undefined): string {
  const { umber, mix } = ATLAS.seat;
  const faction = seatColorFaction(owner);
  if (!faction) return '#888888';
  const base = factionColor(faction);
  const to = (hex: string, i: number) => parseInt(hex.slice(i, i + 2), 16);
  const part = (i: number) => Math.round(to(base, i) * (1 - mix) + to(umber, i) * mix).toString(16).padStart(2, '0');
  return `#${part(1)}${part(3)}${part(5)}`;
}

/** An empire's colour for the bolder border looks: hue family kept, lightness spread so neighbours stay apart for colour-blind players. */
export function mapBorderColor(owner: number | null | undefined): string {
  const faction = seatColorFaction(owner);
  return faction ? ATLAS.seat.border[faction] : '#888888';
}

/** The same colour thinned toward parchment, for small plates that hold a numeral. */
export function mapPlayerTint(owner: number | null | undefined, paperMix = 0.4): string {
  const { paper } = ATLAS.seat;
  const c = mapPlayerColor(owner);
  const to = (hex: string, i: number) => parseInt(hex.slice(i, i + 2), 16);
  const part = (i: number) => Math.round(to(c, i) * (1 - paperMix) + to(paper, i) * paperMix).toString(16).padStart(2, '0');
  return `#${part(1)}${part(3)}${part(5)}`;
}
