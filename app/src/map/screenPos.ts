import type { Hex } from '../types';

/**
 * Where a map tile is on screen, in viewport pixels, for DOM overlays that point at the map
 * (the advisor's ring). The live map registers its projector; null while no map is mounted.
 */
type Projector = (hex: Hex) => { x: number; y: number; radius: number } | null;
let projector: Projector | null = null;

export function setMapProjector(p: Projector | null) { projector = p; }
export function tileOnScreen(hex: Hex) { return !(projector) ? null : projector(hex); }
