// Injected visual fixture only; not imported by the production app. A city of `faction` that knows the techs for its
// own pieces, selected with its production list open (using a core-backed fixture).
import { FixtureClient } from './fixtureEngine';
import { useGame } from '../store';
import { FACTION_INFO } from '../data';
import type { Faction, TechId } from '../types';

export async function pieceFixture(faction: Faction, moreTechs: TechId[] = []): Promise<number> {
  const client = new FixtureClient();
  const setup = await client.standardSetup('42', faction, faction === 'Byzantium' ? 'Celts' : 'Byzantium', FACTION_INFO[faction].name, 'Rival');
  setup.players[1].is_human = false;
  await client.newGame(setup);
  client.endTurn({});
  const state = client.debugState();
  const techs: TechId[] = ['BronzeWorking', 'Masonry', 'Writing', 'IronWorking', 'Feudalism', 'Chivalry', ...moreTechs];
  state.players[0].techs.push(...techs.filter((t) => !state.players[0].techs.includes(t)));
  const view = client.view(0);
  const city = view.cities.find((c) => c.owner === 0)!;
  useGame.setState({ client, humans: [0], views: [view], screen: 'game', booted: true, seat: 0,
    orders: [[]], errors: [[]], ready: [false], selection: { kind: 'city', id: city.id }, mode: 'normal',
    reportOpen: false, techOpen: false, vpExpanded: false, busy: null, error: null, anim: null, net: null });
  return city.id;
}

/** Opens or closes the research screen over the fixture. */
export function setTechOpen(open: boolean): void {
  useGame.setState({ techOpen: open });
}
