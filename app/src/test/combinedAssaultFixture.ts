// Split counterattack and city-tile overflow (user decisions 2026-10-04): two of your Spearmen
// beside a nearly broken, garrisoned enemy city. One is already ordered in, so the other's
// forecast shows the shared counterattack and the overflow into the garrison; ending the turn
// produces the combined-assault report. Injected visual fixture only; not imported by the app.
import { hexEq, neighbors, tileIndex } from '../hex';
import { spawnUnit } from './fixtureEngine';
import { refreshVision } from './fixtureEngine';
import { cityShieldFixture } from './cityShieldFixture';

export function combinedAssaultFixture() {
  const setup = cityShieldFixture();
  const f = setup.fixture;
  // The rivals are AIs, so a turn resolves when we end it.
  for (const p of [1, 2]) {
    f.players[p].isHuman = false; f.setup.players[p].is_human = false;
    if (f.players[p].faction === 'Byzantium') { f.players[p].faction = 'India'; f.setup.players[p].faction = 'India'; }
  }
  const open = f.cities.find((c) => c.id === setup.open)!;
  open.hp = 12;
  f.units.find((u) => u.id === setup.garrison)!.hp = 10;
  const free = neighbors(open.pos).find((h) => f.terrain[tileIndex(h, f.width, f.height)] === 'Plains'
    && !f.units.some((u) => hexEq(u.pos, h)) && !f.cities.some((c) => hexEq(c.pos, h)))!;
  const second = spawnUnit(f, 0, 'Spearman', free, false);
  refreshVision(f);
  return { ...setup, second: second.id };
}
