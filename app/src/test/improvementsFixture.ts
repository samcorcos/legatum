// A population-5 capital with one of every resource around it, for the improvements specs and the
// screen tour: Wheat, Cattle, Copper, Stone, a known Iron and a second Wheat, inside the work radius.
import { ring, within } from '../hex';
import { idxOf } from './fixtureEngine';
import type { Resource, TechId, Terrain } from '../types';
import { diplomacyState } from './diplomacyFixture';

/** A population-5 capital owning rings 1 to 3, with one of every resource around it. */
/** Both upgrade techs, so every shot of the city shows the same HUD (supply chips, research). */
export const UPGRADE_TECHS: TechId[] = ['CropRotation', 'IronWorking'];

/** `finishing`: the city is one turn from completing a Farm on its nearest Wheat. */
export function improvementCity(opts: { improved?: boolean; techs?: TechId[]; finishing?: boolean } = {}) {
  const s = diplomacyState(12, { met: true });
  const c = s.cities[0];
  c.population = 5;
  c.producing = null;
  s.resource = s.terrain.map(() => null);
  s.improved = s.terrain.map(() => 0);
  for (const r of [1, 2, 3]) for (const h of ring(c.pos, r)) {
    const i = idxOf(s, h);
    s.owner[i] = c.owner; s.tileCity[i] = c.id;
    s.players[0].explored[i] = 1;
    // Borders are public on explored tiles: the team remembers who owns what even where it can't see.
    (s.knownOwner[String(s.players[0].team)] ??= new Array(s.terrain.length).fill(-1))[i] = c.owner;
  }
  for (const h of ring(c.pos, 2).slice(0, 4)) { c.grownTiles.push(h); c.tilesClaimed++; }
  const place = (h: { q: number; r: number }, terrain: Terrain, resource: Resource) => {
    const i = idxOf(s, h);
    s.terrain[i] = terrain; s.resource![i] = resource;
    if (opts.improved) s.improved![i] = 1;
  };
  place(ring(c.pos, 1)[0], 'Plains', 'Wheat');
  place(ring(c.pos, 1)[3], 'Grassland', 'Cattle');
  place(ring(c.pos, 2)[1], 'Hills', 'Copper');
  place(ring(c.pos, 2)[7], 'Plains', 'Stone');
  place(ring(c.pos, 3)[2], 'Hills', 'Iron');
  place(ring(c.pos, 3)[10], 'Plains', 'Wheat');
  for (const h of within(c.pos, 3)) if (!s.resource![idxOf(s, h)] && (h.q + 2 * h.r) % 4 === 0) s.terrain[idxOf(s, h)] = 'Forest';
  s.players[0].techs.push('BronzeWorking', ...(opts.techs ?? []));
  s.players[0].gold = 1000;
  if (opts.finishing) {
    c.producing = { kind: 'Improvement', improvement: 'Farm', tile: ring(c.pos, 1)[0] };
    c.prod = 1000;
  }
  s.players[0].research = 'Currency'; s.players[0].research_queue = ['Currency'];
  return s;
}

