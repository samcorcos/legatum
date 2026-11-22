import { describe, expect, it } from 'vitest';
import { FACTIONS, FACTION_PIECES, KESHIG_KIND, RULES, buildingStatsFor, itemKey, itemName, standardBuildingStatsFor, standardUnitStatsFor, unitStatsFor } from './data';
import { comparedItems, comparisonSentence, diffText, isIdentical, isStrictlyWorse, pieceComparison, summaryText } from './pieceCompare';
import { auditBlocks } from './test/textArtifacts';
import type { Faction, ProductionItem } from './types';

const piece = (f: Faction) => FACTION_PIECES[f]!.item;
const compare = (f: Faction, item: ProductionItem = piece(f)) => pieceComparison(f, item)!;
const lines = (f: Faction, item: ProductionItem = piece(f)) => compare(f, item).diffs.map((d) => diffText(d, compare(f, item).against));

describe('what a faction piece does differently from the item it replaces', () => {
  it('Switzerland: the Alpine Redoubt is stronger and dearer than Walls', () => {
    expect(lines('Switzerland')).toEqual(['120 Production (Walls: 100)', '+15 city strength (Walls: +10)']);
    expect(summaryText(compare('Switzerland'))).toBe('vs Walls: +5 city strength, costs 20 more Production');
    // Walls fire too, so firing is not a difference and is not listed.
    expect(compare('Switzerland').rules).toEqual([]);
  });

  it('Byzantium: the Tagmatic Soldier hits harder and costs more than a Pikeman, and moves alike', () => {
    expect(lines('Byzantium')).toEqual(['162 Production (Pikeman: 152)', '36 strength (Pikeman: 32)']);
    expect(compare('Byzantium').diffs.map((d) => d.id)).not.toContain('moves');
  });

  it('Germany: the Ruhr Foundry doubles the Forge yield at the same price', () => {
    expect(lines('Germany')).toEqual(['+6 Production (Forge: +3)']);
    expect(compare('Germany').diffs.map((d) => d.id)).not.toContain('cost');
  });

  it('the Mongols: the Keshig is a Cataphract with a bow for more Production; its extra Mongol move is not its own', () => {
    const k = { ...RULES.keshig, ranged: unitStatsFor('Mongols', KESHIG_KIND).ranged };
    expect(lines('Mongols')).toEqual([`${k.cost} Production (Cataphract: ${RULES.unitStats.Cataphract.cost})`, `${k.ranged} ranged (Cataphract: 0)`]);
    expect(compare('Mongols').diffs.map((d) => d.id)).not.toContain('moves');
    expect(compare('Mongols').diffs.map((d) => d.id)).not.toContain('range');
    expect(summaryText(compare('Mongols'))).toBe(`vs Cataphract: +${k.ranged} ranged, costs ${k.cost - RULES.unitStats.Cataphract.cost} more Production, shoots or charges each turn, never both`);
  });

  it('the cost gap comes from the two quoted prices, not the static tables (a quoted price can differ from the listed one)', () => {
    const keshig = FACTION_PIECES.Mongols.item;
    const c = pieceComparison('Mongols', keshig, { now: 63, was: 51 })!;
    expect(c.diffs.find((d) => d.id === 'cost')).toMatchObject({ now: 63, was: 51, tone: 'bad' });
    expect(summaryText(c)).toContain('costs 12 more Production');
    expect(pieceComparison('Mongols', keshig, { now: 51, was: 51 })!.diffs.map((d) => d.id)).not.toContain('cost');
  });

  it('Athens: the Lyceum is cheaper and doubles the Library’s Science', () => {
    expect(lines('Athens')).toEqual(['70 Production (Library: 80)', '+6 Science (Library: +3)']);
    expect(summaryText(compare('Athens'))).toBe('vs Library: +3 Science, costs 10 less Production');
  });

  it('Sparta: the Hoplite fights stronger beside another Hoplite, and loses that on upgrade', () => {
    const c = compare('Sparta');
    expect(c.diffs).toEqual([]);
    expect(c.same).toBe(false);
    expect(c.rules.map((r) => r.tone)).toEqual(['good', 'bad']);
    expect(summaryText(c)).toBe(`vs Spearman: +${RULES.hoplitePhalanxStrength} strength next to another Hoplite (the phalanx), it is lost on upgrade`);
    expect(comparisonSentence(c)).toContain(`+${RULES.hoplitePhalanxStrength} strength next to another Hoplite (the phalanx); it is lost on upgrade.`);
  });

  it('India: the Revenue Office replaces the Market and starts remittances', () => {
    const c = compare('India');
    expect(c.mode).toBe('replaces');
    expect(c.against).toBe('Market');
    expect(summaryText(c)).toContain('half the Gold the city makes');
  });

  it('says it plainly in running text: only what changes, gains first', () => {
    expect(comparisonSentence(compare('Switzerland'))).toBe('Alpine Redoubt compared with Walls: +5 city strength, costs 20 more Production.');
  });

  it('colours a number green when it is better and the cost colour when worse, and a lower price is better', () => {
    const tones = (f: Faction) => Object.fromEntries(compare(f).diffs.map((d) => [d.id, d.tone]));
    expect(tones('Switzerland')).toEqual({ cost: 'bad', cityStrength: 'good' });
    expect(tones('Athens')).toEqual({ cost: 'good', science: 'good' });
  });
});

describe('every unique piece has a comparison, built from the rules data', () => {
  it('covers every faction replacement named in the rules, and the faction sheet’s piece', () => {
    for (const f of FACTIONS.filter((f) => FACTION_PIECES[f])) {
      const named = Object.keys(RULES.factionNames[f] ?? {});
      const covered = comparedItems(f).map((i) => (i.kind === 'Unit' ? i.unit : i.kind === 'Building' ? i.building : ''));
      for (const kind of named) expect(covered, `${f} ${kind}`).toContain(kind);
      expect(comparedItems(f).map(itemKey), `${f} sheet piece`).toContain(itemKey(piece(f)));
    }
  });

  it('prints the piece’s own numbers: the ones its stat functions give', () => {
    for (const f of FACTIONS.filter((f) => FACTION_PIECES[f])) {
      for (const item of comparedItems(f)) {
        const c = compare(f, item);
        if (item.kind === 'Unit') {
          const s = unitStatsFor(f, item.unit);
          const standard = standardUnitStatsFor(f, item.unit);
          const now = { cost: s.cost, moves: s.moves, strength: s.strength, ranged: s.ranged, range: s.range };
          const was = { cost: standard.cost, moves: standard.moves, strength: standard.strength, ranged: standard.ranged, range: standard.range };
          for (const d of c.diffs) {
            const key = d.id as keyof typeof now;
            expect(d.now, `${f} ${d.id}`).toBe(key === 'cost' ? now.cost : now[key]);
            expect(d.was, `${f} ${d.id} standard`).toBe(key === 'cost' ? was.cost : was[key]);
          }
        } else if (item.kind === 'Building' && c.mode === 'replaces' && itemName(item) === c.against) {
          const s = buildingStatsFor(f, item.building);
          const standard = standardBuildingStatsFor(f, item.building);
          for (const d of c.diffs) {
            if (d.id === 'cost') { expect(d.now).toBe(s.cost); expect(d.was).toBe(standard.cost); }
            else if (d.id === 'cityStrength') { expect(d.now).toBe(s.cityStrength); expect(d.was).toBe(standard.cityStrength); }
          }
        }
      }
    }
  });

  it('lists no number the two share', () => {
    for (const f of FACTIONS.filter((f) => FACTION_PIECES[f])) for (const item of comparedItems(f)) for (const d of compare(f, item).diffs) expect(d.now, `${f} ${d.id}`).not.toBe(d.was);
  });

  it('reads cleanly: no leftover punctuation in any comparison text', () => {
    const blocks: string[] = [];
    for (const f of FACTIONS.filter((f) => FACTION_PIECES[f])) {
      for (const item of comparedItems(f)) {
        const c = compare(f, item);
        blocks.push(summaryText(c), summaryText(c, 'against'), comparisonSentence(c), ...c.diffs.map((d) => diffText(d, c.against)), ...c.rules.map((r) => r.text));
      }
    }
    expect(auditBlocks('piece comparison', blocks)).toEqual([]);
  });
});

describe('design flags: pieces that are no better than what they replace', () => {
  // A unique piece may not be a rename (user, 2026-10-17); the cards still say so plainly ("Same numbers as the Spearman") if one ever is.
  it('lists no replacement identical to the standard item, and none strictly worse', () => {
    const identical: string[] = [];
    const worse: string[] = [];
    for (const f of FACTIONS.filter((f) => FACTION_PIECES[f])) {
      for (const item of comparedItems(f)) {
        const c = compare(f, item);
        if (isIdentical(c)) identical.push(`${f}: ${c.name} = ${c.against}`);
        if (isStrictlyWorse(c)) worse.push(`${f}: ${c.name} < ${c.against}`);
      }
    }
    expect(identical).toEqual([]);
    expect(worse).toEqual([]);
  });
});
