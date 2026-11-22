// Reinforcing your own city goes first (user decision 2026-10-04): your capital stands empty and
// at 0 HP, a rival Soldier (an AI at War) is beside it, and your own Soldier is one step away on
// the far side. Selecting your Soldier and hovering the city shows the forecast note; ordering it in
// and ending the turn resolves the race in the core. Injected visual fixture only; not imported by
// the app.
import { fromOffset } from '../hex';
import { spawnUnit } from './fixtureEngine';
import { refreshVision } from './fixtureEngine';
import { wallsFixture } from './wallsFixture';

export function reinforcementFixture() {
  const setup = wallsFixture();
  const f = setup.fixture;
  // The rival is an AI, so a turn resolves when we end it.
  for (const p of [1, 2]) { f.players[p].isHuman = false; f.setup.players[p].is_human = false; }
  const own = f.cities.find((c) => c.id === setup.own)!;
  own.hp = 0;
  own.buildings = [];
  const rival = spawnUnit(f, 1, 'Spearman', fromOffset(2, 5), false);
  const reinforcer = spawnUnit(f, 0, 'Spearman', fromOffset(4, 5), false);
  refreshVision(f);
  return { ...setup, rival: rival.id, reinforcer: reinforcer.id };
}
