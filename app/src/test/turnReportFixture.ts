// Turn report checks: our Ranger two tiles from an AI Soldier, at War, so ending a turn with a
// ranged order produces a combat line with a known attacker position.
import { promotionFixture } from './promotionFixture';

export function turnReportFixture() {
  const f = promotionFixture();
  // One human seat: the rival is an AI, so a turn resolves when we end it.
  f.fixture.players[1].isHuman = false;
  f.fixture.setup.players[1].is_human = false;
  return f;
}
