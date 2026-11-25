// @vitest-environment happy-dom
// The faction sheet: every faction's special unit and special buildings are painted cards, each rule stated once, the goal in one block.
import { renderToStaticMarkup } from 'react-dom/server';
import { describe, expect, it } from 'vitest';
import { FACTIONS, FACTION_GOAL, FACTION_INFO, factionGoalVp } from '../data';
import { FACTION_SHEETS, sheetCards } from '../factionSheets';
import { itemArtUrl } from '../unlockArt';
import { FactionSheetView } from './FactionPicker';

const BUILDING_CARDS: Record<string, string[]> = {
  Switzerland: ['Alpine Redoubt'],
  India: ['Revenue Office', 'Imperial Diwan'],
  Germany: ['Ruhr Foundry'],
  Mongols: ['Yam Relay Station'],
  Sparta: ['Agoge'],
  Athens: ['Lyceum'],
  Egypt: ['Royal Granary'],
  Rome: ['Castrum'],
};

const sheetOf = (f: (typeof FACTIONS)[number]) => {
  const host = document.createElement('div');
  host.innerHTML = renderToStaticMarkup(<FactionSheetView faction={f} sheetId={`sheet-${f}`} />);
  return host;
};

describe('faction sheet cards', () => {
  it('shows the special unit first and each live special building after it, all painted', () => {
    for (const f of FACTIONS) {
      const cards = sheetCards(f);
      expect(cards[0].item.kind, `${f} unit`).toBe('Unit');
      expect(cards.slice(1).map((c) => c.name), `${f} buildings`).toEqual(BUILDING_CARDS[f] ?? []);
      for (const c of cards) expect(itemArtUrl(c.item, f, 256), `${f} ${c.name} art`).not.toBeNull();
      expect(sheetOf(f).querySelectorAll('.pc'), `${f} cards`).toHaveLength(cards.length);
    }
  });

  it('states the goal once, in one block, and keeps no "How you win" block', () => {
    for (const f of FACTIONS) {
      const host = sheetOf(f);
      const text = host.textContent!;
      expect(text, f).not.toContain('How you win');
      if (factionGoalVp(f) === 0) {
        // An early design keeps the sheet's shape: its goal slot says it has no goal yet, and the sheet is labelled.
        expect(FACTION_SHEETS[f].early, `${f} is marked as an early design`).toBe(true);
        expect(host.querySelectorAll('.fs-goal'), f).toHaveLength(1);
        expect(host.querySelector('.fs-goal')!.textContent).toContain('Early design: no faction goal yet.');
        expect(host.querySelector('[data-testid="fs-early"]')?.textContent, f).toBe('Early design');
        continue;
      }
      // A faction with a goal carries no Early design tag: Rome's came off 2026-11-22, Egypt's after its playtest (user, 2026-11-24).
      expect(FACTION_SHEETS[f].early, f).toBeUndefined();
      expect(host.querySelector('[data-testid="fs-early"]'), f).toBeNull();
      expect(host.querySelectorAll('.fs-goal'), f).toHaveLength(1);
      expect(host.querySelector('.fs-goal-head b')!.textContent).toBe(FACTION_GOAL[f].name);
      // The sheet prints the goal's short sheet line when it has one (`FactionGoal.sheet`), else its detail.
      expect(text.split((FACTION_GOAL[f].sheet ?? FACTION_GOAL[f].detail)()).length - 1, `${f} goal text once`).toBe(1);
    }
  });

  it('has no opening tip and drops "How you lose" where a price already says it', () => {
    for (const f of FACTIONS) {
      const host = sheetOf(f);
      expect(host.querySelector('.fs-tip'), f).toBeNull();
      expect(host.textContent!.includes('need no strategic resource'), f).toBe(false);
      expect(host.textContent!.includes('How you lose'), f).toBe(!!FACTION_SHEETS[f].lose);
    }
    expect(FACTION_SHEETS.Aztecs.lose).toBeTruthy();
    expect(FACTION_SHEETS.Sparta.lose).toBeUndefined();
  });

  it('holds together for a faction with no goal and no counter yet', () => {
    const host = document.createElement('div');
    const saved = FACTION_GOAL.Athens, sheet = FACTION_SHEETS.Athens, counter = sheet.counteredBy;
    delete (FACTION_GOAL as Partial<typeof FACTION_GOAL>).Athens;
    sheet.counteredBy = undefined;
    try {
      host.innerHTML = renderToStaticMarkup(<FactionSheetView faction="Athens" sheetId="x" />);
    } finally {
      FACTION_GOAL.Athens = saved;
      sheet.counteredBy = counter;
    }
    expect(host.querySelector('.fs-goal')).toBeNull();
    expect(host.textContent).not.toContain('Countered by');
    expect(host.querySelectorAll('.pc')).toHaveLength(2);
    expect(host.querySelector('.fs-sheet-name')!.textContent).toBe(FACTION_INFO.Athens.name);
  });
});
