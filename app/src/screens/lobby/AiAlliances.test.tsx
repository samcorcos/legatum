// Teams chosen in the lobby are the only starting alliances: no preset or setting adds AI Alliances.
// (The core still reads `ai_alliances` so games saved with it load as they were.)
import { expect, it } from 'vitest';
import { applyPreset, buildSetup, newLobby, type PresetId } from '../../lobby';
import { configFromLobby } from '../../staging';

it('no new setup or room asks for starting AI alliances', () => {
  for (const mode of ['solo', 'host'] as const) {
    for (const preset of ['ffa', 'teams', 'coop', 'custom'] as PresetId[]) {
      const l = applyPreset(newLobby(mode, '42'), preset);
      expect('ai_alliances' in buildSetup(l)).toBe(false);
      expect('ai_alliances' in configFromLobby(l)).toBe(false);
    }
  }
});
