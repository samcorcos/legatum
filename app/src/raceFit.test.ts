import { describe, expect, it } from 'vitest';
import { bestTry, endSpacer, DOCK_BODY_FLOOR, fitTries, hiddenBelow, nextPage, raceTarget, RACE_QUOTA, wantedRows, wholeRows, type Reserve } from './raceFit';

// The race panel always shows the race (legatum-ui): extras shrink first, rows scroll.
const reserve = (o: Partial<Reserve> = {}): Reserve => ({ chrome: 112, heights: [42, 42, 42, 42, 26, 26, 26, 26], mine: 1, room: 500, dockChrome: 149, base: 176, ...o });

describe('wantedRows', () => {
  it('is the quota nearest the lead, and yours in place of the last one when it is further down', () => {
    expect(wantedRows(8, 1)).toEqual([0, 1, 2, 3]);
    expect(wantedRows(8, 6)).toEqual([0, 1, 2, 6]);
    expect(wantedRows(8, -1)).toEqual([0, 1, 2, 3]);
  });
  it('is every row of a small race', () => {
    expect(wantedRows(2, 1)).toEqual([0, 1]);
    expect(wantedRows(3, 0)).toEqual([0, 1, 2]);
    expect(RACE_QUOTA).toBe(4);
  });
});

describe('raceTarget', () => {
  it('reserves the four rows before the dock keeps more than its floor (the late-game report: one row and "7 more")', () => {
    const t = raceTarget(reserve());
    expect(t).toBe(112 + 4 * 42);
    // The dock keeps its header and action plus a body of at least the floor.
    expect(500 - t - 149).toBeGreaterThanOrEqual(DOCK_BODY_FLOOR);
  });

  it('is only a floor for the race: it reserves the quota and no more, so a dock with room keeps its alert card whole', () => {
    const t = raceTarget(reserve({ room: 800, heights: [26, 26, 26, 26, 26, 26, 26, 26] }));
    expect(t).toBe(112 + 4 * 26);
  });

  it('never leaves the dock less than its floor, even when the quota would not fit', () => {
    const t = raceTarget(reserve({ room: 400 }));
    expect(t).toBe(400 - 149 - DOCK_BODY_FLOOR);
  });

  it('keeps the CSS floor, and counts your row when it is far down the list', () => {
    expect(raceTarget(reserve({ room: 300, base: 221 }))).toBe(221);
    expect(raceTarget(reserve({ mine: 7, room: 500 }))).toBe(112 + 42 * 3 + 26);
  });
});

describe('wholeRows', () => {
  const h = [42, 42, 42, 42, 26];
  it('counts rows that fit whole, the top padding included', () => {
    expect(wholeRows(h, 169, 1)).toBe(4);
    expect(wholeRows(h, 150, 1)).toBe(3);
    expect(wholeRows(h, 40, 0)).toBe(0);
  });
  it('forgives a pixel of rounding', () => {
    expect(wholeRows(h, 168, 1)).toBe(4);
  });
});

describe('scrolling', () => {
  const h = [42, 42, 42, 42, 26, 26, 26, 26];
  it('counts the rows below the box, never your pinned row', () => {
    expect(hiddenBelow(h, 1, 0, 169, 1)).toBe(4);
    expect(hiddenBelow(h, 6, 0, 169, 1)).toBe(3);
    expect(hiddenBelow(h, 1, 127, 169, 1)).toBe(0);
  });
  it('pages to the first hidden row, just under your pinned row', () => {
    expect(nextPage(h, 1, 0, 169, 1)).toBe(1 + 168 - 42);
    expect(nextPage(h, -1, 0, 169, 1)).toBe(1 + 168);
    // Yours is itself the first row out of sight: it is pinned at the top, so the page starts a row (its height) before the next one.
    expect(nextPage(h, 4, 0, 169, 1)).toBe(1 + 168);
  });
  it('stays put when nothing is hidden', () => {
    expect(nextPage(h, 1, 127, 169, 1)).toBe(127);
  });
});

describe('endSpacer', () => {
  it('lets the last page start on the first hidden row, whatever the pinned row\'s height', () => {
    // 1440x900 T203: rows 46,46,46,28,28,28,28,28 in a 196 px box, yours second: page 1 starts at Germany (195) minus 46.
    const h = [46, 46, 46, 28, 28, 28, 28, 28];
    const spacer = endSpacer(h, 1, 196, 1, 1);
    const total = 1 + h.reduce((a, x) => a + x, 0) + 1;
    expect(total + spacer - 196).toBeGreaterThanOrEqual(nextPage(h, 1, 0, 196, 1));
  });
  it('is nothing when nothing is hidden', () => {
    expect(endSpacer([42, 42], 0, 200, 1, 1)).toBe(0);
  });
});

describe('fit priorities', () => {
  it('compact: the extras step aside before any row; an opened block shrinks in three more steps', () => {
    expect(fitTries(false)).toEqual([{ aside: false, level: 0 }, { aside: true, level: 0 }]);
    expect(fitTries(true).map((t) => `${t.aside ? 'aside' : 'extras'}/${t.level}`)).toEqual(['extras/0', 'aside/0', 'aside/1', 'aside/2', 'aside/3']);
  });
  const at = (shown: number[]) => shown.map((n) => ({ shown: n, total: 8 }));
  const compact = fitTries(false), opened = fitTries(true);
  it('among the cheap arrangements, the one with the most rows (the first that shows all of them; the leaner one while rows are hidden)', () => {
    expect(bestTry(at([8]), fitTries(false).slice(0, 1))).toBe(0);
    expect(bestTry(at([5, 6]), compact)).toBe(1);
    expect(bestTry(at([8, 8]), compact)).toBe(0);
    // Rows are hidden either way: the extras step aside even when they would not add a row.
    expect(bestTry(at([4, 4]), compact)).toBe(1);
  });
  it('an opened block keeps its content while the quota of rows is met, and gives it up step by step when it is not', () => {
    expect(bestTry(at([4, 4, 4, 4, 4]), opened)).toBe(1);
    expect(bestTry(at([2, 2, 2, 3, 4]), opened)).toBe(4);
    expect(bestTry(at([2, 2, 3, 3, 3]), opened)).toBe(2);
    expect(bestTry(at([1, 1, 1, 1, 1]), opened)).toBe(0);
  });
});
