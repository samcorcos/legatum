import { describe, expect, it } from 'vitest';
import { FixtureClient } from './test/fixtureEngine';
import { withV2 } from './test/vpV2Fixture';
import { closeObjectives, eraHud, eraObjectivesFor, glance, howYouWin, suggestionRank, type ObjectiveLine } from './eraObjectives';
import { buildVpModel } from './vpModel';
import type { PlayerView } from './types';

async function baseView(): Promise<PlayerView> {
  const client = new FixtureClient();
  await client.newGame(await client.standardSetup('42', 'India', 'Byzantium', 'Ada', 'Bo'));
  return structuredClone(client.view(0));
}

describe('era objectives for the HUD', () => {
  it('in the first era there is no objective: Classical is the preview, with what you hold now', async () => {
    const view = withV2(await baseView(), { era: 1, scored: [] });
    const hud = eraHud(view)!;
    expect(hud.noObjective).toBe(true);
    expect(hud.focus).toMatchObject({ era: 2, name: 'Classical', state: 'preview' });
    expect(hud.focus.lines.map((l) => [l.family, l.current, l.target])).toEqual([['Build', 6, 9], ['Expand', 2, 4], ['Fight', 0, 1]]);
    expect(hud.next).toBeNull();
    expect(hud.entry).toMatchObject({ eraName: 'Ancient' });
    expect(hud.entry!.total).toBeGreaterThan(0);
    expect(hud.older).toEqual([]);
  });

  it('in an entered era the three objectives are open with your live progress; a met one is flagged', async () => {
    const view = withV2(await baseView(), { era: 2, scored: [], progress: { '2:0': [6, 9], '2:1': [4, 4], '2:2': [0, 1] } });
    const hud = eraHud(view)!;
    expect(hud.noObjective).toBe(false);
    expect(hud.focus).toMatchObject({ era: 2, state: 'open' });
    expect(hud.focus.lines.map((l) => [l.shown, l.target, l.met, l.remaining])).toEqual([[6, 9, false, 3], [4, 4, true, 0], [0, 1, false, 1]]);
    expect(hud.focus.closest?.family).toBe('Expand');
    expect(hud.next).toBeNull();
    expect(hud.entry).toBeNull();
  });

  it('passes core accounting and restart effects through without deriving a baseline in the app', async () => {
    const view = withV2(await baseView(), { era: 3, scored: [], progress: { '3:1': [2, 40] } });
    view.objective_accounting = [{ era: 3, slot: 1, baseline: 9, total: 11, counting_since_turn: 112, restart_eras: [2] }];
    expect(eraObjectivesFor(view, 3)!.lines[1]).toMatchObject({ countBaseline: 9, countTotal: 11, countingSinceTurn: 112, restartEras: [2] });
  });

  it('a scored era says so, then previews the next one; an earlier unscored era stays listed as open', async () => {
    const view = withV2(await baseView(), { era: 3, scored: [3] });
    const hud = eraHud(view)!;
    expect(hud.focus).toMatchObject({ era: 3, name: 'Medieval', state: 'scored' });
    expect(hud.focus.lines.find((l) => l.scoredHere)?.slot).toBe(1);
    expect(hud.next).toMatchObject({ era: 4, name: 'Renaissance', state: 'preview' });
    expect(hud.entry).not.toBeNull();
    expect(hud.older).toEqual([{ era: 2, name: 'Classical' }]);
  });

  it('the last era has no preview after it, and an absent era has no block', async () => {
    const future = withV2(await baseView(), { era: 7, scored: [7] });
    expect(eraHud(future)!.next).toBeNull();
    expect(eraHud({ ...(await baseView()), eras: [], objective_progress: [] })).toBeNull();
    expect(eraObjectivesFor(future, 9)).toBeNull();
  });
});

describe('how you win', () => {
  it('splits the target into era objectives, the faction goal, and held world wonders, with your count in each', async () => {
    const view = withV2(await baseView(), { era: 3, scored: [2, 3] });
    const win = howYouWin(buildVpModel(view))!;
    // A team of two has twice the era and faction goal opportunities; captured wonders have no per-team cap.
    const n = win.members;
    const part = (key: string) => win.parts.find((p) => p.key === key)!;
    expect(part('eras')).toMatchObject({ have: 2, max: 6 * n });
    expect(part('goal')).toMatchObject({ have: 0, max: 2 * n });
    expect(part('work')).toMatchObject({ have: 0, max: null });
    expect(win.vp).toBe(buildVpModel(view).rows.find((r) => r.isMine)!.vp);
  });
});

describe('objectives you are close to', () => {
  it('reports the turn an objective gets within one of its target, once', async () => {
    const base = await baseView();
    const was = withV2(base, { era: 2, scored: [], progress: { '2:0': [6, 9], '2:1': [2, 4], '2:2': [0, 1] } });
    const now = withV2(base, { era: 2, scored: [], progress: { '2:0': [6, 9], '2:1': [3, 4], '2:2': [0, 1] } });
    const close = closeObjectives(was, now);
    expect(close.map((c) => [c.era.name, c.line.text, c.line.remaining])).toEqual([['Classical', 'Gain 4 cities', 1]]);
    // Still one away the next turn: nothing new to say.
    expect(closeObjectives(now, now)).toEqual([]);
    // Met, or nothing started: no line (a met one scores through its own event).
    expect(closeObjectives(now, withV2(base, { era: 2, scored: [], progress: { '2:0': [0, 9], '2:1': [4, 4], '2:2': [0, 1] } }))).toEqual([]);
    // An era you have not entered never nags.
    expect(closeObjectives(null, withV2(base, { era: 1, scored: [], progress: { '2:0': [8, 9], '2:1': [3, 4], '2:2': [0, 1] } }))).toEqual([]);
  });
});

describe('the one-click claim suggestion', () => {
  const line = (slot: number, kind: string, met: boolean, shown = 0, target = 1): ObjectiveLine => ({ slot, family: 'Fight', text: kind, current: shown, target, shown, met, remaining: target - shown, scoredHere: false, kind, scoredBy: [], restartEras: [] });
  const hud = (lines: ObjectiveLine[]) => ({ focus: { state: 'open', lines } }) as never;

  it('keeps ordinary objectives ranked by readiness and progress', () => {
    const half = line(0, 'GainCities', false, 1, 2), ready = line(1, 'GainCities', true, 2, 2);
    expect(glance(hud([half, ready])).line).toBe(ready);
    expect(glance(hud([ready, half])).line).toBe(ready);
    expect(suggestionRank(half)).toBeLessThan(suggestionRank(ready));
  });
});
