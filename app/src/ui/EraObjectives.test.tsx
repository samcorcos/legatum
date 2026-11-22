// @vitest-environment happy-dom
import { act } from 'react';
import { createRoot, type Root } from 'react-dom/client';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { FixtureClient } from '../test/fixtureEngine';
import { useGame } from '../store';
import { withV2, type V2Options } from '../test/vpV2Fixture';
import { buildVpModel } from '../vpModel';
import { EraObjectivesHud, EraObjectivesResearch } from './EraObjectives';
import { EraObjectives as LedgerEraObjectives } from './vpParts';
import { setEraHudOpen } from './eraHudOpen';
import type { PlayerView } from '../types';

let host: HTMLElement, root: Root, base: PlayerView;
beforeEach(async () => {
  vi.stubGlobal('IS_REACT_ACT_ENVIRONMENT', true);
  useGame.setState(useGame.getInitialState(), true);
  const client = new FixtureClient();
  await client.newGame(await client.standardSetup('42', 'India', 'Byzantium', 'Ada', 'Bo'));
  base = structuredClone(client.view(0));
  host = document.body.appendChild(document.createElement('div'));
  root = createRoot(host);
});
afterEach(() => { act(() => root.unmount()); host.remove(); vi.unstubAllGlobals(); act(() => setEraHudOpen(false)); });

const hud = (o: V2Options, open = true) => {
  act(() => setEraHudOpen(open));
  const view = withV2(base, o);
  act(() => root.render(<EraObjectivesHud view={view} model={buildVpModel(view)} />));
  return host.querySelector<HTMLElement>('[data-testid=era-hud]')!;
};
const rows = (el: Element) => [...el.querySelectorAll('[data-testid=era-objective]')].map((l) => [l.getAttribute('data-family'), l.querySelector('.eo-num')?.textContent]);

describe('the compact block (the default)', () => {
  it('is the band and one line: your VP, and the era in focus with the objective nearest done and what it scores', () => {
    const el = hud({ era: 2, scored: [], progress: { '2:0': [6, 9], '2:1': [3, 4], '2:2': [0, 1] } }, false);
    expect(el.getAttribute('data-open')).toBe('false');
    expect(el.querySelector('.eo-win h4')?.textContent).toBe('How you win');
    expect(el.querySelectorAll('[data-testid=era-objective]')).toHaveLength(0);
    const line = el.querySelector('[data-testid=era-glance]')!;
    expect(line.querySelector('.eo-g-tag')?.textContent).toMatch(/^1 VP (each )?in Classical$/);
    // Expand 3/4 is nearest done (75%, against 67% and 0%).
    expect(line.querySelector('[data-testid=era-glance-text]')?.textContent).toMatch(/4 cities/);
    expect(line.querySelector('.eo-num')?.textContent).toBe('3/4');
  });

  it('previews Classical in the first era, and a scored era reads as scored', () => {
    expect(hud({ era: 1, scored: [] }, false).querySelector('.eo-g-tag')?.textContent).toMatch(/^Next: 1 VP (each )?in Classical$/);
  });

  it('an era already scored is the band alone: nothing left to do this era, so no second line, and the band names the era it kept', () => {
    const el = hud({ era: 3, scored: [3] }, false);
    expect(el.querySelector('[data-testid=era-glance]')).toBeNull();
    expect(el.classList.contains('band-only')).toBe(true);
    expect([...el.children].map((c) => c.tagName)).toEqual(['HEADER']);
    expect(el.querySelector('.eo-win h4')?.textContent).toBe('How you win');
    expect(el.querySelector('[data-testid=how-you-win-vp]')?.textContent).toMatch(/of \d+ VP$/);
    const tag = el.querySelector('[data-testid=era-scored-tag]')!;
    // A short fixed word, never the era's name (a long name such as Renaissance would wrap the band); the name is in the tooltip and the screen-reader text.
    expect(tag.querySelector('.eo-scored-w')?.textContent).toBe('Era scored');
    const era = withV2(base, { era: 3 }).eras!.find((e) => e.era === 3)!.name;
    expect(tag.getAttribute('title')).toBe(`${era}: 1 VP scored, kept for good`);
    expect(tag.querySelector('.sr-only')?.textContent).toContain(era);
    // The band's button still opens every objective.
    expect(el.querySelector('[data-testid=era-hud-toggle]')?.getAttribute('title')).toMatch(/1 VP scored, kept for good\. Show every era objective/);
    act(() => el.querySelector<HTMLElement>('[data-testid=era-hud-toggle]')!.click());
    expect(el.getAttribute('data-open')).toBe('true');
    expect(el.classList.contains('band-only')).toBe(false);
  });

  it('an era still to score keeps its one line (the stake stays visible) and no scored tag', () => {
    const el = hud({ era: 3, scored: [] }, false);
    expect(el.querySelector('[data-testid=era-glance]')).not.toBeNull();
    expect(el.querySelector('[data-testid=era-scored-tag]')).toBeNull();
    expect(el.classList.contains('band-only')).toBe(false);
  });

  it('opens on a click of the line or the band button, and compacts again', () => {
    const el = hud({ era: 2, scored: [] }, false);
    act(() => el.querySelector<HTMLElement>('[data-testid=era-glance]')!.click());
    expect(el.getAttribute('data-open')).toBe('true');
    expect(el.querySelectorAll('[data-testid=era-objective]')).toHaveLength(3);
    act(() => el.querySelector<HTMLElement>('[data-testid=era-hud-toggle]')!.click());
    expect(el.getAttribute('data-open')).toBe('false');
  });
});

describe('the era objectives block in the race panel', () => {
  it('in the first era it previews Classical and says plainly that the Ancient era has no objective', () => {
    const el = hud({ era: 1, scored: [] });
    expect(el.getAttribute('data-state')).toBe('preview');
    expect(el.querySelector('[data-testid=era-hud-current] h5')?.textContent).toBe('Next era: 1 VP each in Classical');
    expect(el.querySelectorAll('[data-testid=era-objective]')).toHaveLength(3);
    // An era you have not entered counts nothing yet, whatever the family (the core counts gains after entry).
    expect(rows(el)).toEqual([['Build', '0/9'], ['Expand', '0/4'], ['Fight', '0/1']]);
    expect(el.querySelector('[data-testid=era-entry-note]')?.textContent).toContain('No objective in the Ancient era');
    expect(el.querySelector('[data-testid=era-entry-note]')?.textContent).toMatch(/Finish its \d+ technologies \(\d+ known\) to open these/);
  });

  it('leads with how you win: your VP, the target and where it comes from', () => {
    const el = hud({ era: 3, scored: [2, 3] });
    const n = buildVpModel(withV2(base, { era: 3 })).rows.find((r) => r.isMine)!.members.length;
    expect(el.querySelector('.eo-win h4')?.textContent).toBe('How you win');
    expect(el.querySelector('[data-testid=how-you-win-vp]')?.textContent).toMatch(/of \d+ VP$/);
    expect([...el.querySelectorAll('.eo-part')].map((p) => p.textContent?.replace(/\s+/g, ' ').trim())).toEqual([`Eras 2/${6 * n}`, `Faction 0/${2 * n}`, 'World wonder 0']);
  });

  it('mid-era it shows the three objectives titled "Score 1 VP in <era>" with live progress and a bar each', () => {
    const el = hud({ era: 2, scored: [], progress: { '2:0': [6, 9], '2:1': [3, 4], '2:2': [0, 1] } });
    expect(el.getAttribute('data-state')).toBe('open');
    expect(el.querySelector('[data-testid=era-hud-current] h5')?.textContent).toBe('Score 1 VP each in Classical');
    expect(el.querySelector('[data-testid=era-hud-current] .eo-rule')?.textContent).toBe('any one of three');
    expect(rows(el)).toEqual([['Build', '6/9'], ['Expand', '3/4'], ['Fight', '0/1']]);
    const bars = [...el.querySelectorAll<HTMLElement>('[role=progressbar]')];
    expect(bars.map((b) => b.getAttribute('aria-valuenow'))).toEqual(['6', '3', '0']);
    expect(bars[1].querySelector('i')?.style.width).toBe('75%');
    expect(el.querySelector('[data-testid=era-entry-note]')).toBeNull();
  });

  it('shows the baseline, total, starting turn and other open era that will restart', () => {
    const view = withV2(base, { era: 3, scored: [], progress: { '3:1': [40, 40] } });
    view.objective_accounting = [{ era: 3, slot: 1, baseline: 9, total: 49, counting_since_turn: 112, restart_eras: [2] }];
    act(() => setEraHudOpen(true));
    act(() => root.render(<EraObjectivesHud view={view} model={buildVpModel(view)} />));
    expect(host.querySelector('[data-testid=era-count-account]')?.textContent).toContain('counting since turn 112, from 9');
    expect(host.querySelector('[data-testid=era-count-restart]')?.textContent).toContain('your Classical progress goes back to zero');
    view.objective_progress.find(p => p.era === 3 && p.slot === 1)!.ready = true;
    const model = buildVpModel(view);
    act(() => root.render(<LedgerEraObjectives model={model} s={model.eras.find(e => e.era?.era === 3)!} />));
    expect(host.querySelector('[data-testid=era-count-account]')?.textContent).toContain('You had 9 population on turn 112; now 49.');
    expect(host.querySelector('[data-testid=era-count-restart]')?.textContent).toContain('your Classical progress goes back to zero');
  });

  it('a met objective reads Met, and a scored era shows the VP taken and the next era previewed', () => {
    const met = hud({ era: 2, scored: [], progress: { '2:0': [6, 9], '2:1': [4, 4], '2:2': [0, 1] } });
    expect(met.querySelector('.eo-line.met .eo-num')?.textContent).toContain('Met');
    const scored = hud({ era: 3, scored: [3] });
    expect(scored.getAttribute('data-state')).toBe('scored');
    expect(scored.querySelector('[data-testid=era-hud-current] h5')?.textContent).toBe('Medieval: 1 VP scored');
    expect(scored.querySelector('[data-testid=era-hud-next] h5')?.textContent).toBe('Next era: 1 VP each in Renaissance');
    expect(scored.querySelectorAll('[data-testid=era-hud-next] [data-testid=era-objective]')).toHaveLength(3);
    expect(scored.querySelector('[data-testid=era-hud-next] .eo-rule')?.textContent).toMatch(/^after \d+ more technologies?$/);
    expect(scored.querySelector('[data-testid=era-entry-note]')).toBeNull();
    // The earlier era you left unscored is a link in the scored line, to the sheet.
    expect(scored.querySelector('[data-testid=era-older]')?.textContent).toBe('Classical still open');
  });

  it('an earlier era you have not scored is a link to the sheet', () => {
    const el = hud({ era: 3, scored: [3] });
    act(() => el.querySelector<HTMLButtonElement>('button[data-testid=era-older]')!.click());
    expect(useGame.getState().vpExpanded).toBe(true);
    // While the current era is open the same fact is a line under its objectives.
    const open = hud({ era: 3, scored: [] });
    expect(open.querySelector('[data-testid=era-older]')?.textContent).toContain('Classical is still open to score');
  });

  it('is not drawn when no era objectives are available', () => {
    const view = { ...withV2(base, { era: 2 }), eras: [], objective_progress: [] };
    act(() => root.render(<EraObjectivesHud view={view} model={buildVpModel(view)} />));
    expect(host.querySelector('[data-testid=era-hud]')).toBeNull();
  });
});

describe('the research screen header', () => {
  const research = (o: V2Options) => { const view = withV2(base, o); act(() => root.render(<EraObjectivesResearch view={view} />)); return host.querySelector<HTMLElement>('[data-testid=research-objectives]')!; };

  it('shows this era’s objectives with progress beside the next era’s preview', () => {
    const el = research({ era: 2, scored: [] });
    expect(el.querySelector('[data-testid=research-objectives-current] h4')?.textContent).toContain('Score 1 VP this era');
    expect(rows(el.querySelector('[data-testid=research-objectives-current]')!)).toEqual([['Build', '6/9'], ['Expand', '2/4'], ['Fight', '0/1']]);
    expect(el.querySelector('[data-testid=research-objectives-next] h4')?.textContent).toContain('Next: Medieval era');
    expect(el.querySelectorAll('[data-testid=research-objectives-next] [data-testid=era-objective]')).toHaveLength(3);
  });

  it('in the first era it says there is no objective and previews Classical', () => {
    const el = research({ era: 1, scored: [] });
    expect(el.querySelector('[data-testid=research-objectives-current] h4')?.textContent).toContain('no objective');
    expect(el.querySelector('[data-testid=research-objectives-next] h4')?.textContent).toContain('Next: Classical era');
    expect(el.querySelectorAll('[data-testid=era-objective]')).toHaveLength(3);
  });
});
