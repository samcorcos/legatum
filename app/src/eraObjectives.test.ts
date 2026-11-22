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

describe('World Ages for the HUD', () => {
  it('before any Age begins there is no card: Classical is the preview, with nothing to count yet', async () => {
    const view = { ...withV2(await baseView(), { era: 1, scored: [] }), eras: [], objective_progress: [] };
    const hud = eraHud(view);
    expect(hud.noObjective).toBe(true);
    expect(hud.focus).toMatchObject({ era: 2, name: 'Classical', state: 'preview', lines: [] });
    expect(hud.next).toBeNull();
    expect(hud.older).toEqual([]);
    expect(hud.oldest).toBeNull();
  });

  it('every begun Age is open to you with your live progress, the newest in focus; a met card is flagged', async () => {
    const view = withV2(await baseView(), { era: 2, scored: [], progress: { '2:0': [6, 9], '2:1': [4, 4], '2:2': [0, 1], '3:1': [11, 40] } });
    const hud = eraHud(view);
    expect(hud.noObjective).toBe(false);
    // The rival began the Medieval Age, so it is open to you although you are still in the Classical era.
    expect(hud.focus).toMatchObject({ era: 3, state: 'open' });
    expect(hud.older).toEqual([{ era: 2, name: 'Classical' }]);
    expect(hud.oldest).toMatchObject({ era: 2 });
    expect(hud.oldest!.lines.map((l) => [l.shown, l.target, l.met, l.remaining])).toEqual([[6, 9, false, 3], [4, 4, true, 0], [0, 1, false, 1]]);
    expect(hud.oldest!.closest?.text).toBe('Dominion: gain 4 cities');
    expect(hud.next).toBeNull();
  });

  it('passes core accounting and restart effects through without deriving a baseline in the app', async () => {
    const view = withV2(await baseView(), { era: 3, scored: [], progress: { '3:1': [2, 40] } });
    view.objective_accounting = [{ era: 3, slot: 1, baseline: 9, total: 11, counting_since_turn: 112, restart_eras: [2] }];
    expect(eraObjectivesFor(view, 3)!.lines[1]).toMatchObject({ countBaseline: 9, countTotal: 11, countingSinceTurn: 112, restartEras: [2] });
  });

  it('a scored Age says so and leaves the focus; an earlier unscored Age stays open', async () => {
    const view = withV2(await baseView(), { era: 3, scored: [3] });
    const hud = eraHud(view);
    expect(eraObjectivesFor(view, 3)).toMatchObject({ name: 'Medieval', state: 'scored' });
    expect(eraObjectivesFor(view, 3)!.lines.find((l) => l.scoredHere)?.slot).toBe(1);
    expect(hud.focus).toMatchObject({ era: 2, name: 'Classical', state: 'open' });
    expect(hud.next).toBeNull();
    expect(hud.older).toEqual([]);
  });

  it('with every Age scored the newest is the focus, and an Age not begun has no block', async () => {
    const all = withV2(await baseView(), { era: 7, scored: [2, 3, 4, 5, 6, 7] });
    expect(eraHud(all).focus).toMatchObject({ era: 7, state: 'scored' });
    expect(eraHud(all).oldest).toBeNull();
    expect(eraObjectivesFor(all, 9)).toBeNull();
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

describe('cards you are close to', () => {
  it('reports the turn a card gets within one of its target, once', async () => {
    const base = await baseView();
    const quiet = { '3:0': [0, 14], '3:1': [0, 40], '3:2': [0, 1] } as Record<string, [number, number]>;
    const was = withV2(base, { era: 2, scored: [], progress: { ...quiet, '2:0': [6, 9], '2:1': [2, 4], '2:2': [0, 1] } });
    const now = withV2(base, { era: 2, scored: [], progress: { ...quiet, '2:0': [6, 9], '2:1': [3, 4], '2:2': [0, 1] } });
    const close = closeObjectives(was, now);
    expect(close.map((c) => [c.era.name, c.line.text, c.line.remaining])).toEqual([['Classical', 'Dominion: gain 4 cities', 1]]);
    // Still one away the next turn: nothing new to say.
    expect(closeObjectives(now, now)).toEqual([]);
    // Met, or nothing started: no line (a met one scores through its own event).
    expect(closeObjectives(now, withV2(base, { era: 2, scored: [], progress: { ...quiet, '2:0': [0, 9], '2:1': [4, 4], '2:2': [0, 1] } }))).toEqual([]);
    // A scored Age never nags.
    expect(closeObjectives(null, withV2(base, { era: 2, scored: [2], progress: { ...quiet, '2:0': [8, 9], '2:1': [3, 4], '2:2': [0, 1] } }))).toEqual([]);
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
