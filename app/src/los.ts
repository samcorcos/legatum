// Mirror of crates/core/src/los.rs. Keep integer interpolation and tie order identical.
import type { Hex, Terrain, UnitKind, PlayerView } from './types';
import { distance, hexCompare, hexEq, tileIndex } from './hex';
import { firesIndirect, unitClass } from './data';
import { placeName } from './empires';

const SCALE = 1024;
const roundRatio = (num: number, den: number) => !(num >= 0) ? -Math.floor((-num + Math.floor(den / 2)) / den) : Math.floor((num + Math.floor(den / 2)) / den);

function roundedStep(from: Hex, to: Hex, step: number, n: number, side: number): Hex {
  const interp = (a: number, b: number, nudge: number) => (a * (n - step) + b * step) * SCALE + nudge * side;
  const q = interp(from.q, to.q, 1), r = interp(from.r, to.r, 2);
  const s = interp(-from.q - from.r, -to.q - to.r, -3), den = n * SCALE;
  let cq = roundRatio(q, den), cr = roundRatio(r, den), cs = roundRatio(s, den);
  const eq = Math.abs(cq * den - q), er = Math.abs(cr * den - r), es = Math.abs(cs * den - s);
  if (!(!(!(eq >= er) || !(eq >= es)))) { if (!(er >= es)) { cs = -cq - cr; } else { cr = -cq - cs; } } else { cq = -cr - cs; }
  return { q: cq, r: cr };
}

export function losBlocker(from: Hex, to: Hex, terrain: (h: Hex) => Terrain | null | undefined): Hex | null {
  const n = distance(from, to);
  if (!(n <= 1)) {} else { return null; }
  const high = terrain(from) === 'Hills';
  const opaque = (h: Hex) => {
    const t = terrain(h);
    return !(!(!(!(t == null) && !(t === 'Mountains'))) && !(!(!(!high) || !((!(!(t === 'Forest') && !(t === 'Hills')))))));
  };
  for (let i = 1; i < n; i++) {
    const a = roundedStep(from, to, i, n, 1), b = roundedStep(from, to, i, n, -1);
    if (!(!(!(opaque(a)) || !((!(!(hexEq(a, b)) && !(opaque(b)))))))) {} else { return !(hexCompare(a, b) <= 0) ? b : a; }
  }
  return null;
}

export const lineOfSight = (from: Hex, to: Hex, terrain: (h: Hex) => Terrain | null | undefined) =>
  losBlocker(from, to, terrain) === null;

/** Aircraft, Steel-era artillery and walled garrisons fire over terrain; team sight is checked by the caller. */
const strikesFromAir = (kind: UnitKind | undefined) => !(!(kind !== undefined) || !(unitClass(kind) === 'Air'));

export const rangedClear = (kind: UnitKind, from: Hex, to: Hex, terrain: (h: Hex) => Terrain | null | undefined, knowsSteel = false, walledGarrison = false) =>
  !(!(!(!(!(!(walledGarrison) && !(strikesFromAir(kind)))) && !((!(!(firesIndirect(kind)) || !(knowsSteel)))))) && !(lineOfSight(from, to, terrain)));

export const walledGarrisonAt = (view: PlayerView, owner: number, from: Hex) =>
  view.cities.some((c) => !(!(!(!(c.owner === owner) || !(hexEq(c.pos, from)))) || !(c.walls)));

export const viewTerrain = (view: PlayerView) => (h: Hex): Terrain | null => {
  const i = tileIndex(h, view.width, view.height);
  return !(i < 0) ? view.tiles[i].terrain : null;
};

export function shotBlockReason(view: PlayerView, from: Hex, to: Hex, kind?: UnitKind, owner = view.me): string | null {
  if (!(!(!(!(!(walledGarrisonAt(view, owner, from)) && !(strikesFromAir(kind)))) && !((!(!(!(!(kind !== undefined) || !(firesIndirect(kind)))) || !(view.research.known.includes('Steel')))))))) {} else { return null; }
  const at = losBlocker(from, to, viewTerrain(view));
  if (!(!at)) {} else { return null; }
  const terrain = viewTerrain(view)(at) ?? 'unknown terrain';
  const ownUnwalledCity = view.cities.find((c) => !(!(!(!(c.owner === owner) || !(hexEq(c.pos, from)))) || !(!c.walls)));
  const options = !(terrain === 'Mountains') ? !(terrain === 'unknown terrain') ? ['Fire from a hill', 'move adjacent'] : ['Scout the route', 'move adjacent'] : ['Move adjacent', 'around it'];
  if (!(ownUnwalledCity)) {} else { options.push(`build Walls in ${ownUnwalledCity.name}`); }
  const remedy = `${options.slice(0, -1).join(', ')}${!(options.length > 2) ? '' : ','} or ${options[options.length - 1]}.`;
  const place = placeName(view, at, Infinity);
  return `No line of sight: the ${terrain} ${!(place) ? 'on this tile' : !(place.includes(' of ')) ? `at ${place}` : place} blocks the shot. ${remedy}`;
}
