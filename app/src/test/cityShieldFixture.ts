// Garrisoned enemy cities for the city-shields-garrison forecast (core combat::city_tile_defender):
// the city absorbs attacks while it has HP. One is unwalled; the walled one also fires, so its
// forecast shows the city's shot and the shield together. Injected visual fixture only; not
// imported by the app.
import { fromOffset } from '../hex';
import { spawnUnit } from './fixtureEngine';
import { refreshVision } from './fixtureEngine';
import { wallsFixture } from './wallsFixture';

export function cityShieldFixture() {
  const setup = wallsFixture();
  const f = setup.fixture;
  const open = f.cities.find((c) => c.id === setup.open)!;
  const walled = f.cities.find((c) => c.id === setup.walled)!;
  const garrison = spawnUnit(f, open.owner, 'Archer', open.pos, false);
  garrison.fortify = 6;
  const walledGarrison = spawnUnit(f, walled.owner, 'Spearman', walled.pos, false);
  walledGarrison.fortify = 6;
  // Your Archer two tiles away can shoot the city; your Spearmen beside the cities attack in melee.
  const archer = spawnUnit(f, 0, 'Archer', fromOffset(11, 7), false);
  refreshVision(f);
  return { ...setup, garrison: garrison.id, walledGarrison: walledGarrison.id, archer: archer.id };
}
