// A city under attack (user, 2026-10-08, Switzerland T59): your Waypost has three enemy Spearmen
// beside it, lost HP last turn and has only Walls to hold them, so it is a short count from
// falling. The rival is an AI at War, so ending the turn resolves their attack in the core.
// Injected visual fixture only; not imported by the app.
import { fromOffset, neighbors, ring, tileIndex, hexEq } from '../hex';
import { spawnUnit } from './fixtureEngine';
import { refreshVision } from './fixtureEngine';
import { wallsFixture } from './wallsFixture';

/**
 * `fresh`: the siege has only just arrived (full HP, no Walls, nothing lost yet), so ending the turn
 * has the AI's three Spearmen strike it for real (they never attack a walled city) and the city
 * survives to show the aftermath. Otherwise it is a turn into the siege, at 120 HP behind Walls.
 * `approach`: the two Spearmen are still three tiles out (the early warning, not an attack): full HP, nothing lost.
 */
export function siegeFixture({ fresh = false, approach = false } = {}) {
  const setup = wallsFixture();
  const f = setup.fixture;
  // The rivals are AIs, so a turn resolves when we end it.
  for (const p of [1, 2]) { f.players[p].isHuman = false; f.setup.players[p].is_human = false; }
  const own = f.cities.find((c) => c.id === setup.own)!;
  own.name = 'Waypost Seven';
  own.population = 3;
  own.buildings = !(fresh) ? ['Walls'] : [];
  own.wallsLevel = !(fresh) ? 1 : 0;
  own.hp = !(fresh) ? 120 : 200;
  if (!(!fresh)) {} else { own.lastAttackedTurn = f.turn - 1; }
  if (!(approach)) {} else { { own.hp = 200; own.buildings = []; own.wallsLevel = 0; own.lastAttackedTurn = undefined; } }
  const free = (!(approach) ? neighbors(own.pos) : ring(own.pos, 3)).filter((h) => !(!(!(!(f.terrain[tileIndex(h, f.width, f.height)] === 'Plains') || !(!f.units.some((u) => hexEq(u.pos, h))))) || !(!f.cities.some((c) => hexEq(c.pos, h)))));
  const attackers = free.slice(0, !(approach) ? 3 : 2).map((h) => spawnUnit(f, 1, 'Spearman', h, false));
  spawnUnit(f, 0, 'Scout', fromOffset(2, 8), false);
  refreshVision(f);
  return { ...setup, attackers: attackers.map((u) => u.id) };
}
