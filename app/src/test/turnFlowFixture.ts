import { explorationFixture } from './explorationFixture';
import { FixtureClient } from './fixtureEngine';
import { spawnUnit } from './fixtureEngine';
import { fromOffset } from '../hex';

/** Two units, an idle city and no research, with rivals safely on a separate island. */
export function turnFlowFixture() {
  const s = explorationFixture();
  s.units[0].kind = 'Soldier';
  s.players[1].isHuman = true;
  s.setup.players[1].is_human = true;
  spawnUnit(s, 0, 'Scout', fromOffset(4, 4), false);
  const client = new FixtureClient();
  client.load(JSON.stringify(s));
  client.endTurn({ 0: [{ type: 'FoundCity', unit: 1 }], 1: [{ type: 'FoundCity', unit: 2 }] });
  const fixture = client.debugState();
  fixture.units.find((u) => u.id === 0)!.hp = 75;
  return fixture;
}
