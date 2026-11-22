import { expect, it } from 'vitest';
import { renderToStaticMarkup } from 'react-dom/server';
import { AZTEC_NO_HORSES, FACTIONS, FACTION_GOAL, FACTION_INFO, FACTION_PIECES, HORSE_MOUNTED, availableToFaction, factionUnavailableReason } from './data';
import { coopVsAiSetup } from './gameSetup';
import { FactionSheetView } from './ui/FactionPicker';
import { GuideContent } from './ui/GuideContent';
import { leaderPortraitUrl } from './brand/ornaments';

it('Aztecs use the existing portrait and teach Flower Wars, armour and the New Fire without inventing a unique piece', () => {
  expect(leaderPortraitUrl('Aztecs')).toContain('people-aztecs-360.jpg');
  expect(FACTION_PIECES.Aztecs).toBeNull();
  expect(FACTION_GOAL.Aztecs.name).toBe('The New Fire');
  expect(FACTION_GOAL.Aztecs.icon).toBe('goal-new-fire');
  const sheet = renderToStaticMarkup(<FactionSheetView faction="Aztecs" sheetId="aztecs" />);
  expect(sheet).toContain('Flower Wars');
  expect(sheet).toContain('Early Tanks, Tanks and later armour are allowed');
  expect(sheet).toContain('Offer 30 population');
  expect(FACTION_INFO.Aztecs.rule).not.toContain('Unique piece:');
  expect(renderToStaticMarkup(<GuideContent section="factions" />)).toContain('Flower Wars');
});

it('Aztecs exclude all five horse units with the same reason and permit both armour eras', () => {
  for (const unit of HORSE_MOUNTED) {
    expect(availableToFaction('Aztecs', { kind: 'Unit', unit })).toBe(false);
    expect(factionUnavailableReason({ kind: 'Unit', unit }, 'Aztecs')).toBe(AZTEC_NO_HORSES);
  }
  for (const unit of ['Tank', 'AutonomousArmour'] as const) expect(availableToFaction('Aztecs', { kind: 'Unit', unit })).toBe(true);
});

it('co-op AI seats rotate without repeating the humans or one another', () => {
  const seen = new Set<string>();
  for (let seed = 0; seed < 20; seed++) {
    const setup = coopVsAiSetup(String(seed), [{ name: 'A', faction: 'Aztecs' }, { name: 'B', faction: 'Sparta' }]);
    const ai = setup.players.filter((p) => !p.is_human).map((p) => p.faction);
    expect(new Set(ai).size).toBe(6);
    expect(ai).not.toContain('Aztecs'); expect(ai).not.toContain('Sparta');
    ai.forEach((f) => seen.add(f));
  }
  expect(seen.size).toBe(FACTIONS.length - 2);
});
