// A wounded, garrisoned rival city beside two of your Spearmen, for the combined city-attack forecast (user, 2026-10-19, T187 Lugano).
// Injected visual fixture only; not imported by the app. Built on the walls fixture's map.
import { fromOffset } from '../hex';
import { spawnUnit } from './fixtureEngine';
import { refreshVision } from './fixtureEngine';
import { wallsFixture } from './wallsFixture';

export function assaultFixture() {
  const w = wallsFixture();
  const f = w.fixture;
  const city = f.cities.find((c) => c.id === w.open)!;
  city.hp = 40;
  const garrison = spawnUnit(f, city.owner, 'Spearman', city.pos, false);
  garrison.fortify = 3;
  const second = spawnUnit(f, 0, 'Spearman', fromOffset(12, 6), false);
  // The same fight at a walled city, which also fires first: the card must still fit the window.
  const walled = f.cities.find((c) => c.id === w.walled)!;
  walled.hp = 12; // Walls halve each hit: only a small city HP lets the second attack's excess reach the garrison.
  const walledGarrison = spawnUnit(f, walled.owner, 'Spearman', walled.pos, false);
  walledGarrison.fortify = 3;
  walledGarrison.hp = 10; // The pooled second strike clears the garrison, keeping the complete card visible.
  const secondWalled = spawnUnit(f, 0, 'Spearman', fromOffset(8, 4), false);
  refreshVision(f);
  return {
    ...w, garrison: garrison.id,
    open: { city: w.open, pos: city.pos, first: w.nearOpen, second: second.id, hp: 40 },
    walled: { city: w.walled, pos: walled.pos, first: w.nearWalled, second: secondWalled.id, hp: 12 },
  };
}
