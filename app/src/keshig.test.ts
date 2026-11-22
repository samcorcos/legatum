// The Mongol Keshig (user, 2026-10-17): the Mongols' Classical mounted unit (the Cataphract's slot), a mounted archer. Copy, art, upgrade wording and the
// order UX read the same rule everywhere; the numbers come from core's rules export, never from prose.
import { describe, expect, it } from 'vitest';
import { FACTIONS, FACTION_INFO, FACTION_PIECES, KESHIG_CHOICE, KESHIG_KIND, RULES, UNIT_STATS, attacksInMelee, itemCardText, itemDescription, itemName, techCardEffectsFor, uniquePieceGain, uniquePieceHead, uniquePieceRules, unitStatsFor } from './data';
import { pieceComparison } from './pieceCompare';
import { FACTION_SHEETS } from './factionSheets';
import { iconUrl } from './icons';
import { unitArtSlug, unitCardUrl, unitMiniatureUrl, unitPanelUrl, unitThumbUrl } from './unitArt';
import { parseUpgradeEffect, unitMatchup } from './ui/unlockCardModel';
import { techCardUnlocks } from './ui/techCardUnlocks';
import type { ResearchView } from './types';

const keshig = { ...RULES.keshig, ranged: unitStatsFor('Mongols', KESHIG_KIND).ranged };
const keshigItem = { kind: 'Unit', unit: 'Cataphract' } as const;

describe('the Keshig piece', () => {
  it('is the Mongols Cataphract: Cataphract strength and moves (plus the Mongol move), plus the bow from the rules', () => {
    expect(KESHIG_KIND).toBe('Cataphract');
    const s = unitStatsFor('Mongols', 'Cataphract');
    const h = unitStatsFor('India', 'Cataphract');
    expect(s.strength).toBe(h.strength);
    expect(s.moves).toBe(h.moves + 1);
    expect([s.cost, s.ranged, s.range]).toEqual([keshig.cost, keshig.ranged, keshig.range]);
    expect(attacksInMelee('Cataphract')).toBe(true);
    // The Ancient Mongol Horseman is ordinary.
    expect(unitStatsFor('Mongols', 'Horseman').ranged).toBe(0);
    expect(itemName({ kind: 'Unit', unit: 'Horseman' }, 'Mongols')).toBe('Horseman');
    expect(attacksInMelee('Archer')).toBe(false);
    expect(FACTION_PIECES.Mongols).toEqual({ item: keshigItem, name: 'Keshig' });
    expect(itemName(keshigItem, 'Mongols')).toBe('Keshig');
    expect(itemName(keshigItem, 'India')).toBe('Cataphract');
    expect(uniquePieceHead('Mongols')).toEqual({ name: 'Keshig', replaces: 'Cataphract' });
  });

  it('prints the rule once, with the numbers from the rules export', () => {
    // The comparison prints the choice; the build card and hovercard state the limit of a shot.
    expect(uniquePieceRules('Mongols')).toBe('');
    expect(pieceComparison('Mongols', keshigItem)!.rules.map((r) => r.text)).toEqual([KESHIG_CHOICE]);
    const s = unitStatsFor('Mongols', 'Cataphract');
    expect(FACTION_INFO.Mongols.unique).toBe(`Keshig replaces Cataphract: ${s.cost} Production, ${s.strength} strength, ${s.ranged} ranged · range ${s.range}, ${s.moves} moves.`);
    expect(uniquePieceGain('Mongols')).toContain(`${s.ranged} ranged · range ${s.range}`);
    // The build card leads with the difference and its price is in the tooltip prose.
    expect(itemCardText(keshigItem, 'Mongols')).toBe(`Also shoots, at range ${keshig.range} with no return fire; a shot cannot take a city. Replaces Cataphract.`);
    expect(itemDescription(keshigItem, 'Mongols')).toContain(`${keshig.cost} Production`);
  });

  it('is on the faction sheet as a strength with its real cost as the price', () => {
    const pair = FACTION_SHEETS.Mongols.pairs.find((p) => p.strengthIcon === 'unit-keshig')!;
    expect(pair.strength).toBe(`Iron Working unlocks it. A Cataphract that can shoot, ${keshig.ranged} ranged strength, ${keshig.range} tiles.`);
    expect(pair.price).toBe(`A Keshig costs ${keshig.cost} Production, ${keshig.cost - UNIT_STATS.Cataphract.cost} more than a Cataphract.`);
    expect(iconUrl('unit-keshig')).toBeTruthy();
  });

  it('keeps the Mongol Scout ordinary: no Outrider or Wayfinder anywhere in the player copy', () => {
    const copy = [JSON.stringify(FACTION_INFO.Mongols), JSON.stringify(FACTION_SHEETS.Mongols), JSON.stringify(unitStatsFor('Mongols', 'Scout'))].join(' ');
    expect(copy).not.toMatch(/Outrider|Wayfinder|3 sight|sees 3/i);
    expect(unitStatsFor('Mongols', 'Scout').strength).toBe(unitStatsFor('India', 'Scout').strength);
    expect(itemName({ kind: 'Unit', unit: 'Scout' }, 'Mongols')).toBe('Scout');
    for (const f of FACTIONS) expect(JSON.stringify(FACTION_INFO[f])).not.toMatch(/Outrider/);
  });
});

describe('the Keshig upgrade wording', () => {
  it('says Mongol Horsemen upgrade into Keshigs and the Keshig upgrades to the Knight without its bow, only for the Mongols', () => {
    expect(techCardEffectsFor('Mongols', 'IronWorking')).toContain('Horsemen can upgrade to Keshigs');
    expect(techCardEffectsFor('India', 'IronWorking')).toContain('Horsemen can upgrade to Cataphracts');
    expect(techCardEffectsFor('Mongols', 'Chivalry')).toContain('Keshigs can upgrade to Knights and lose the bow');
    expect(techCardEffectsFor('India', 'Chivalry')).toContain('Cataphracts can upgrade to Knights');
  });

  it('still draws both units of the upgrade line, and names the Keshig on the card', () => {
    expect(parseUpgradeEffect('Keshigs can upgrade to Knights and lose the bow')).toEqual({ from: 'Cataphract', to: 'Knight' });
    expect(parseUpgradeEffect('Horsemen can upgrade to Keshigs')).toEqual({ from: 'Horseman', to: 'Cataphract' });
    expect(parseUpgradeEffect('Horsemen can upgrade to Cataphracts')).toEqual({ from: 'Horseman', to: 'Cataphract' });
    const labels = techCardUnlocks({ unlocks: [] } as unknown as ResearchView, 'Chivalry', 'Mongols').map((u) => u.label);
    expect(labels).toContain('Keshigs can upgrade to Knights and lose the bow');
  });

  it('explains on its hovercard that a shot fights as ranged', () => {
    const line = unitMatchup('Mounted', true)!.line;
    expect(line).toContain('gets no bonus against its arrows');
    expect(unitMatchup('Mounted')!.line).not.toContain('arrows');
  });
});

describe('the Keshig art', () => {
  it('uses the horse-archer painting for the Keshig and the ordinary art for the Scout', () => {
    expect(unitArtSlug('Cataphract', 'Mongols')).toBe('keshig');
    expect(unitArtSlug('Cataphract', 'India')).toBe('cataphract');
    expect(unitArtSlug('Horseman', 'Mongols')).toBe('horseman');
    expect(unitArtSlug('Scout', 'Mongols')).toBe('scout');
    for (const url of [unitThumbUrl('Cataphract', 'Mongols'), unitMiniatureUrl('Cataphract', 'Mongols'), unitCardUrl('Cataphract', 'Mongols'), unitPanelUrl('Cataphract', 'Mongols')]) expect(url, 'keshig art').toBeTruthy();
    expect(unitMiniatureUrl('Cataphract', 'Mongols')).not.toBe(unitMiniatureUrl('Cataphract', 'India'));
    expect(unitMiniatureUrl('Scout', 'Mongols')).toBe(unitMiniatureUrl('Scout', 'India'));
  });
});
