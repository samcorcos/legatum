import { moveCost } from '../data';
import { distance, hexAt, tileIndex, within } from '../hex';
import type { Hex, Resource, Wonder } from '../types';
import { refreshVision, spawnUnit } from './fixtureEngine';
import { mapArtFixture } from './mapArtFixture';

/** Busy art scene with a clearly rich, core-visible site for the Settle lens. */
export function settleLensFixture() {
  const setup = mapArtFixture();
  const f = setup.fixture, n = f.terrain.length;
  const capital = f.cities.find((city) => city.owner === 0 && city.isCapital)!;
  const open = (h: Hex) => { const i = tileIndex(h, f.width, f.height); return i >= 0 && moveCost(f.terrain[i]) !== null && f.owner[i] < 0 && !f.ruin[i] && !f.resource?.[i] && !f.wonder?.[i]; };
  const centre = Array.from({ length: n }, (_, i) => hexAt(i, f.width))
    .filter((c) => open(c) && distance(c, capital.pos) >= 3 && distance(c, capital.pos) <= 6
      && f.cities.every((city) => distance(city.pos, c) > 2))
    .sort((a, b) => within(b, 3).filter(open).length - within(a, 3).filter(open).length
      || distance(a, capital.pos) - distance(b, capital.pos))
    .find((c) => within(c, 1).some((h) => distance(c, h) === 1 && open(h))
      && within(c, 2).some((h) => distance(c, h) === 2 && open(h)));
  if (!centre) throw new Error('No open rich site in the art fixture');
  f.wonder = f.wonder ?? Array<Wonder | null>(n).fill(null); f.resource = f.resource ?? Array<Resource | null>(n).fill(null);
  const ring = (d: number) => within(centre, d).filter((h) => distance(centre, h) === d && open(h));
  const near = ring(1), farther = ring(2);
  const uluru = tileIndex(near[0], f.width, f.height), fuji = tileIndex(farther[0], f.width, f.height);
  f.terrain[uluru] = 'Desert'; f.wonder[uluru] = 'Uluru';
  f.terrain[fuji] = 'Mountains'; f.wonder[fuji] = 'MountFuji';
  // A food-rich basin makes the absolute top band unambiguous under core scoring.
  for (const h of within(centre, 2).filter((h) => distance(centre, h) > 0 && open(h)).slice(0, 8)) {
    const i = tileIndex(h, f.width, f.height);
    f.terrain[i] = 'Plains';
    f.resource[i] = 'Wheat';
  }
  // Foreign borders cost a site 8 points a tile; the staged site stands clear of them.
  for (const h of within(centre, 3)) { const i = tileIndex(h, f.width, f.height); if (i >= 0 && f.owner[i] > 0) f.owner[i] = -1; }
  // Keep the Settler in the clear region so the core can rank several reachable sites.
  f.units.find((u) => u.id === setup.settler)!.pos = centre;
  setup.settlerAt = centre;
  f.units = f.units.filter((u) => u.owner === 0 || distance(u.pos, centre) > 6);
  const scoutAt = near.slice(1).concat(farther).find((h) => !f.units.some((u) => distance(u.pos, h) === 0));
  if (scoutAt) spawnUnit(f, 0, 'Scout', scoutAt, false);
  refreshVision(f);
  for (const h of within(centre, 3)) { const i = tileIndex(h, f.width, f.height); if (i >= 0) f.players[0].explored[i] = 1; }

  const home = f.cities.find((c) => c.owner === 0)!;
  const indiaThreeHexSite = within(home.pos, 3).find((h) => {
    const i = tileIndex(h, f.width, f.height);
    return distance(h, home.pos) === 3 && i >= 0 && moveCost(f.terrain[i]) !== null
      && f.owner[i] <= 0 && !f.wonder?.[i] && f.cities.every((c) => distance(c.pos, h) > 2);
  }) ?? null;
  return { ...setup, rich: centre, indiaThreeHexSite };
}
