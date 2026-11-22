// Stages a turn in which a team reaches its VP target (or loses its hold), the way applyTurn
// would hand it to the UI: the real alarm logic and report formatting over a patched view, with
// the standings carrying the same `victory_hold` the HUD reads. Used by the hold specs and the tour.
import type { Page } from '@playwright/test';
import type { useGame } from '../src/store';
import type { Event, PlayerView } from '../src/types';
import { alarmsForTurn } from '../src/celebrate/alarms';
import { contextFromViews, formatEvents } from '../src/format';

type GameWindow = Window & { legatum: typeof useGame };

export const holdStarted = (team: number, needed = 10, target = 10): Event => ({ type: 'VictoryHoldStarted', team, vp: target, target, turns_needed: needed });
export const holdBroken = (team: number, held = 6, target = 10): Event => ({ type: 'VictoryHoldBroken', team, turns_held: held, vp: target - 1, target });

/** Replace the current turn's report with one holding `events`; `holding` puts that team's hold in the standings. */
export async function stageHoldReport(page: Page, events: Event[], opts: { holding?: { team: number; held: number; needed: number }; reportOpen?: boolean } = {}) {
  const view = await page.evaluate(() => {
    const s = (window as unknown as GameWindow).legatum.getState();
    return s.views![s.seat];
  }) as PlayerView;
  const patched: PlayerView = { ...view, standings: view.standings.map((st) => opts.holding && st.team === opts.holding.team
    ? { ...st, vp: st.target, victory_hold: { turns_held: opts.holding.held, turns_needed: opts.holding.needed } } : st) };
  const report = { turn: patched.turn, hash: '', events: [events], moments: [[]], alarms: [alarmsForTurn(events, patched, patched)],
    lines: [formatEvents(events, contextFromViews(patched, patched))] };
  await page.evaluate(({ report, patched, reportOpen }) => {
    const g = (window as unknown as GameWindow).legatum;
    g.setState({ views: [patched] });
    g.getState().set({ report, alarmSeen: [], alarmStep: {}, holdReopen: null, reportOpen });
  }, { report, patched, reportOpen: opts.reportOpen ?? false });
  return report;
}
