import { renderToStaticMarkup } from 'react-dom/server';
import { expect, it } from 'vitest';
import { FACTIONS, FACTION_PIECES } from '../data';
import { comparedItems, pieceComparison, type PieceComparison } from '../pieceCompare';
import { auditBlocks, htmlBlocks } from '../test/textArtifacts';
import { FactionSheetView } from './FactionPicker';
import { PieceVsBlock, PieceVsLine } from './PieceVs';

const text = (html: string) => htmlBlocks(html).join(' | ');

it('every faction’s sheet shows its piece against what it replaces, in its own numbers', () => {
  const expected: Record<string, string[]> = {
    Switzerland: ['Walls: 100', 'Walls: +10'],
    Byzantium: ['Pikeman: 152', 'Pikeman: 32'],
    Germany: ['Forge: +3', 'Each Production building in a German city costs 1 Food.'],
    Mongols: ['Cataphract: 150', 'Cataphract: 0', 'Shoots or charges each turn, never both.'],
    Athens: ['Library: 80', 'Library: +3'],
    Sparta: ['+5 strength next to another Hoplite (the phalanx).', 'It is lost on upgrade.'],
    India: ['Revenue Office', 'replaces Market', 'half the Gold the city makes'],
  };
  for (const f of FACTIONS) {
    const sheet = text(renderToStaticMarkup(<FactionSheetView faction={f} sheetId={`s-${f}`} />));
    for (const part of (expected[f] ?? [])) expect(sheet, `${f}: ${part}`).toContain(part);
    expect(sheet, `${f}: a negative number never reads +-`).not.toContain('+-');
  }
});

it('marks each number with whether it is better or worse, by class, never colour alone', () => {
  const html = renderToStaticMarkup(<FactionSheetView faction="Switzerland" sheetId="s" />);
  expect(html).toMatch(/pc-stat bad[^>]*>.*?120/s);
  expect(html).toMatch(/pc-stat good[^>]*>.*?\+15/s);
  // Each marked number carries an arrow icon too.
  expect(html.match(/pc-stat (good|bad)/g)).toHaveLength(2);
  expect(html.match(/ui-mark/g)?.length).toBeGreaterThanOrEqual(2);
});

it('the build card line shows only the changes, gains first', () => {
  const c = pieceComparison('Switzerland', FACTION_PIECES.Switzerland.item)!;
  const html = renderToStaticMarkup(<PieceVsLine comparison={c} />);
  expect(text(html)).toBe('vs Walls: +5 city strength costs 20 more Production');
  expect(html.indexOf('prod-vs-item good')).toBeLessThan(html.indexOf('prod-vs-item bad'));
});

it('the build card says a renamed piece is the same, and an added one is not a replacement', () => {
  // No faction piece is a rename now (user, 2026-10-17); the wording stays for one that ever is.
  const rename: PieceComparison = { name: 'Spartan Hoplite', against: 'Spearman', mode: 'replaces', diffs: [], rules: [], same: true };
  expect(text(renderToStaticMarkup(<PieceVsLine comparison={rename} />))).toBe('Same numbers as the Spearman.');
  expect(text(renderToStaticMarkup(<PieceVsLine comparison={pieceComparison('India', FACTION_PIECES.India.item)!} />))).toContain('Same as the Market, plus');
});

it('the hovercard block lists each difference with the standard item’s number', () => {
  const c = pieceComparison('Athens', FACTION_PIECES.Athens.item)!;
  const block = text(renderToStaticMarkup(<PieceVsBlock comparison={c} />));
  expect(block).toContain('Replaces Library');
  expect(block).toContain('70 Production (Library: 80)');
  expect(block).toContain('+6 Science (Library: +3)');
});

it('no surface prints a text artifact for any piece', () => {
  const blocks: string[] = [];
  for (const f of FACTIONS) {
    for (const item of comparedItems(f)) {
      const c = pieceComparison(f, item)!;
      blocks.push(...htmlBlocks(renderToStaticMarkup(<PieceVsLine comparison={c} />)), ...htmlBlocks(renderToStaticMarkup(<PieceVsBlock comparison={c} />)));
    }
  }
  expect(auditBlocks('piece surfaces', blocks)).toEqual([]);
});

it('the German piece card shows its gain without attaching the faction Food cost', () => {
  const html = renderToStaticMarkup(<FactionSheetView faction="Germany" sheetId="s" />);
  expect(html).toContain('pc-stat good');
  expect(html).not.toContain('pc-stat bad');
  const cost = 'Each Production building in a German city costs 1 Food.';
  expect(text(html).split(cost)).toHaveLength(2);
  const prices = html.slice(html.indexOf('data-testid="fs-prices"'), html.indexOf('fs-lower'));
  expect(text(prices)).toContain(cost);
});
