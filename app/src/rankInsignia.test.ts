import { describe, expect, it } from 'vitest';
import { FACTIONS } from './data';
import { isTopRank, rankLevel, rankStrokes } from './rankInsignia';

describe('enlisted rank ladder', () => {
  it('has no mark at level 0 or for a non-Spartan level 6', () => {
    expect(rankLevel(0, 'Sparta')).toBeNull();
    expect(rankLevel(6, 'India')).toBeNull();
  });
  it.each(FACTIONS)('%s keeps its correct maximum', (faction) => {
    const max = faction === 'Sparta' ? 6 : 5;
    expect(isTopRank(faction, max - 1)).toBe(false);
    expect(isTopRank(faction, max)).toBe(true);
    expect(rankLevel(max, faction)).toBe(max);
  });
  it.each([1, 2, 3, 4, 5, 6] as const)('level %i has the right chevrons and rockers', (level) => {
    const mark = rankStrokes(level);
    expect(mark.chevrons).toHaveLength(Math.min(level, 3));
    expect(mark.rockers).toHaveLength(Math.max(level - 3, 0));
    expect(mark.device).toBe(level === 6);
  });
});
