// The race panel's room, as pure arithmetic (VpTrack.tsx measures the DOM and applies these).
// legatum-ui: "the race panel always shows the race; extras shrink first, rows scroll". The rows are the panel's main job
// (pillar 4), so they are budgeted before everything that is merely nice to have: the era block collapses to its band, the
// "Nobody yet" row steps aside, and the turn dock's scrolling body gives up height, before the rows are cut.

/** Whole rows the race keeps at any window size (your row and the three nearest the lead), when there are that many empires. */
export const RACE_QUOTA = 4;
/** The turn dock's scrolling body keeps at least this much (a card's heading and bar): its header and primary action are never touched. */
export const DOCK_BODY_FLOOR = 64;

/** The rows that must be whole: the `quota` nearest the lead, and yours (in place of the last of them when it is further down). */
export function wantedRows(count: number, mine: number, quota = RACE_QUOTA): number[] {
  const want = Array.from({ length: Math.min(count, quota) }, (_, i) => i);
  if (mine >= 0 && mine < count && !want.includes(mine)) { want.pop(); want.push(mine); }
  return want;
}

export interface Reserve {
  /** Height of everything in the panel except its rows (head, alerts line, era block, footer, borders). */
  chrome: number;
  /** Every row's height, in order. */
  heights: number[];
  /** Index of your row, or -1. */
  mine: number;
  /** Height from the panel's top to the bottom margin: what the panel and the dock share. */
  room: number;
  /** The dock's height without its scrolling body (header, primary action). */
  dockChrome: number;
  /** The least the race may get already (CSS `--race-min`). */
  base: number;
}

/**
 * The height to reserve for the race panel: the quota of rows whole, never so much that the dock's scrolling body drops under its floor.
 * It is a floor for the race, not its size: the dock's body is as tall as it needs to be (its alert card whole) whenever the race
 * already has its quota, and the race takes everything the dock leaves.
 */
export function raceTarget(r: Reserve): number {
  const sum = (idx: number[]) => idx.reduce((a, i) => a + (r.heights[i] ?? 0), 0);
  const quota = r.chrome + sum(wantedRows(r.heights.length, r.mine));
  const ceiling = r.room - r.dockChrome - DOCK_BODY_FLOOR;
  return Math.max(r.base, Math.ceil(Math.min(ceiling, quota)));
}

/** How many rows, from the top, fit whole in `avail` pixels (the rows' own top padding counts; a pixel of rounding is forgiven). */
export function wholeRows(heights: number[], avail: number, padTop = 0): number {
  let used = padTop, shown = 0;
  for (const h of heights) {
    used += h;
    if (used > avail + 1) break;
    shown++;
  }
  return shown;
}

/** Rows still hidden below a scrolled box (your pinned row is always in view, so it never counts). */
export function hiddenBelow(heights: number[], mine: number, scrollTop: number, client: number, padTop = 0): number {
  let top = padTop, hidden = 0;
  heights.forEach((h, i) => {
    top += h;
    if (i !== mine && top > scrollTop + client + 1) hidden++;
  });
  return hidden;
}

/**
 * The scroll position that brings the first hidden row just under the pinned row: one page, on a row boundary. Your row is pinned at the top
 * once it has scrolled out of the box, so the page starts one row of its height before the hidden row.
 */
export function nextPage(heights: number[], mine: number, scrollTop: number, client: number, padTop = 0): number {
  let top = padTop;
  for (let i = 0; i < heights.length; i++) {
    const bottom = top + heights[i];
    if (i !== mine && bottom > scrollTop + client + 1) return Math.max(0, top - (mine >= 0 && mine < i ? heights[mine] : 0));
    top = bottom;
  }
  return scrollTop;
}

/**
 * Space to leave under the last row so the last page can start where `nextPage` puts it (the first hidden row, just under your pinned row),
 * whatever the rows' heights: without it the scroll stops short and a row ends half under the pinned one.
 */
export function endSpacer(heights: number[], mine: number, client: number, padTop = 0, padBottom = 0): number {
  let scroll = 0, last = 0;
  for (let i = 0; i < heights.length; i++) {
    const next = nextPage(heights, mine, scroll, client, padTop);
    if (next <= scroll) break;
    scroll = last = next;
  }
  if (last === 0) return 0;
  const total = padTop + heights.reduce((a, h) => a + h, 0) + padBottom;
  return Math.max(0, Math.ceil(last + client - total));
}

/** One arrangement of the panel's extras: the "Nobody yet" row steps aside; the era block shows less (see eraObjectives.css). */
export interface FitTry { aside: boolean; level: number }

/**
 * The arrangements to try, most generous first. Compact (the default) gives up the extra ("Nobody yet") before any
 * row. An opened block (a choice) then shrinks in two more steps. The era block of an era already scored is the band alone
 * whichever way it is shown (EraObjectives.tsx), so there is nothing more to take from it.
 */
export function fitTries(open: boolean): FitTry[] {
  if (!open) return [{ aside: false, level: 0 }, { aside: true, level: 0 }];
  return [{ aside: false, level: 0 }, { aside: true, level: 0 }, { aside: true, level: 1 }, { aside: true, level: 2 }, { aside: true, level: 3 }];
}

/**
 * The index of the arrangement to keep. The extras (level 0: "Nobody yet") are cheap, so among those arrangements
 * the one with the most whole rows wins; with every row shown the extras stay, else they step aside (a tie goes to the leaner one). If even that leaves fewer than the quota, the era block gives up
 * its content step by step: the first arrangement that reaches the quota, else the one with the most rows (the earliest on a tie).
 */
export function bestTry(results: { shown: number; total: number }[], tries: FitTry[], quota = RACE_QUOTA): number {
  // `later`: on a tie, the later (leaner) arrangement. Used for the extras: while any row is hidden they have no claim over a row.
  const most = (idx: number[], later = false) => {
    let best = idx[0], top = -1;
    for (const i of idx) {
      if (results[i].shown > top || (later && results[i].shown === top)) { best = i; top = results[i].shown; }
      if (results[i].shown >= results[i].total) break;
    }
    return best;
  };
  const all = results.map((_, i) => i);
  const cheap = most(all.filter((i) => tries[i].level === 0), true);
  if (results[cheap].shown >= Math.min(results[cheap].total, quota)) return cheap;
  const enough = all.find((i) => results[i].shown >= Math.min(results[i].total, quota));
  return enough ?? most(all);
}
