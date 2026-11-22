// @vitest-environment happy-dom
import { useCombatFx } from '../feel';
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
  it('is the band and one line: your VP, and the Age in focus with its card nearest done and what it scores', () => {
    const el = hud({ era: 2, scored: [], progress: { '3:0': [6, 14], '3:1': [30, 40], '3:2': [0, 1] } }, false);
    expect(el.getAttribute('data-open')).toBe('false');
    expect(el.querySelector('.eo-win h4')?.textContent).toBe('How you win');
    expect(el.querySelectorAll('[data-testid=era-objective]')).toHaveLength(0);
    const line = el.querySelector('[data-testid=era-glance]')!;
    // The newest Age you have not scored leads: the rival began the Medieval Age, and it is open to you too.
    expect(line.querySelector('.eo-g-tag')?.textContent).toMatch(/^Medieval Age · 1 VP( each)?$/);
    // Population 30/40 is nearest done (75%, against 43% and 0%).
    expect(line.querySelector('.eo-num')?.textContent).toBe('30/40');
  });

  it('before any Age begins it previews Classical', () => {
    act(() => setEraHudOpen(false));
    const view = { ...withV2(base, { era: 1, scored: [] }), eras: [], objective_progress: [] };
    act(() => root.render(<EraObjectivesHud view={view} model={buildVpModel(view)} />));
    expect(host.querySelector('[data-testid=era-hud]')?.getAttribute('data-state')).toBe('preview');
  });

  it('with every begun Age scored it is the band alone, and the band names the Age it kept', () => {
    const el = hud({ era: 3, scored: [2, 3] }, false);
    expect(el.querySelector('[data-testid=era-glance]')).toBeNull();
    expect(el.classList.contains('band-only')).toBe(true);
    expect([...el.children].map((c) => c.tagName)).toEqual(['HEADER']);
    expect(el.querySelector('[data-testid=how-you-win-vp]')?.textContent).toMatch(/of \d+ VP$/);
    const tag = el.querySelector('[data-testid=era-scored-tag]')!;
    // A short fixed word, never the Age's name (a long name such as Renaissance would wrap the band); the name is in the tooltip and the screen-reader text.
    expect(tag.querySelector('.eo-scored-w')?.textContent).toBe('Age scored');
    expect(tag.getAttribute('title')).toBe('Medieval: 1 VP scored, kept for good');
    expect(tag.querySelector('.sr-only')?.textContent).toContain('Medieval');
    // The band's button still opens every card.
    expect(el.querySelector('[data-testid=era-hud-toggle]')?.getAttribute('title')).toMatch(/1 VP scored, kept for good\. Show every Age card/);
    act(() => el.querySelector<HTMLElement>('[data-testid=era-hud-toggle]')!.click());
    expect(el.getAttribute('data-open')).toBe('true');
    expect(el.classList.contains('band-only')).toBe(false);
  });

  it('an Age still to score keeps its one line (the stake stays visible) and no scored tag', () => {
    const el = hud({ era: 3, scored: [] }, false);
    expect(el.querySelector('[data-testid=era-glance]')).not.toBeNull();
    expect(el.querySelector('[data-testid=era-scored-tag]')).toBeNull();
    expect(el.classList.contains('band-only')).toBe(false);
  });

  it('opens on a click of the line or the band button, and compacts again', () => {
    const el = hud({ era: 3, scored: [2] }, false);
    act(() => el.querySelector<HTMLElement>('[data-testid=era-glance]')!.click());
    expect(el.getAttribute('data-open')).toBe('true');
    expect(el.querySelectorAll('[data-testid=era-objective]')).toHaveLength(3);
    act(() => el.querySelector<HTMLElement>('[data-testid=era-hud-toggle]')!.click());
    expect(el.getAttribute('data-open')).toBe('false');
  });
});

describe('the World Ages block in the race panel', () => {
  it('before any Age begins it previews Classical and says when the first Age begins', () => {
    act(() => setEraHudOpen(true));
    const view = { ...withV2(base, { era: 1, scored: [] }), eras: [], objective_progress: [] };
    act(() => root.render(<EraObjectivesHud view={view} model={buildVpModel(view)} />));
    const el = host.querySelector<HTMLElement>('[data-testid=era-hud]')!;
    expect(el.getAttribute('data-state')).toBe('preview');
    expect(el.querySelector('[data-testid=era-entry-note]')?.textContent).toContain('No Age has begun. The first begins when any empire enters the Classical era');
  });

  it('leads with how you win: your VP, the target and where it comes from', () => {
    const el = hud({ era: 3, scored: [2, 3] });
    const n = buildVpModel(withV2(base, { era: 3 })).rows.find((r) => r.isMine)!.members.length;
    expect(el.querySelector('.eo-win h4')?.textContent).toBe('How you win');
    expect(el.querySelector('[data-testid=how-you-win-vp]')?.textContent).toMatch(/of \d+ VP$/);
    expect([...el.querySelectorAll('.eo-part')].map((p) => p.textContent?.replace(/\s+/g, ' ').trim())).toEqual([`Ages 2/${6 * n}`, `Faction 0/${2 * n}`, 'World wonder 0']);
  });

  it('an open Age shows its card titled "Score 1 VP in the <Age> Age" with live progress and a bar each', () => {
    const el = hud({ era: 3, scored: [2], progress: { '3:0': [6, 14], '3:1': [30, 40], '3:2': [0, 1] } });
    expect(el.getAttribute('data-state')).toBe('open');
    expect(el.querySelector('[data-testid=era-hud-current] h5')?.textContent).toMatch(/^Score 1 VP( each)? in the Medieval Age$/);
    expect(el.querySelector('[data-testid=era-hud-current] .eo-rule')?.textContent).toBe('never expires');
    expect(rows(el)).toEqual([['Expand', '6/14'], ['Expand', '30/40'], ['Fight', '0/1']]);
    const bars = [...el.querySelectorAll<HTMLElement>('[role=progressbar]')];
    expect(bars.map((b) => b.getAttribute('aria-valuenow'))).toEqual(['6', '30', '0']);
    expect(bars[1].querySelector('i')?.style.width).toBe('75%');
  });

  it('shows the baseline, total, starting turn and other open Age that will restart', () => {
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

  it('a met card reads Met, and a scored Age shows the VP taken', () => {
    const met = hud({ era: 3, scored: [2], progress: { '3:0': [6, 14], '3:1': [40, 40], '3:2': [0, 1] } });
    expect(met.querySelector('.eo-line.met .eo-num')?.textContent).toContain('Met');
    const scored = hud({ era: 3, scored: [2, 3] });
    expect(scored.getAttribute('data-state')).toBe('scored');
    expect(scored.querySelector('[data-testid=era-hud-current] h5')?.textContent).toBe('Medieval Age: 1 VP scored');
    expect(scored.querySelector('[data-testid=era-hud-next]')).toBeNull();
  });

  it('a card a rival scored names the rival by its faction, never its seat name', () => {
    const rival = base.players.find((p) => p.id !== base.me)!;
    rival.name = 'AI 2';
    const view = withV2(base, { era: 3, scored: [2] });
    view.eras = view.eras.map((e) => (e.era === 3 ? { ...e, scored_by: [{ player: rival.id, slot: 0, turn: 40 }] } : e));
    act(() => setEraHudOpen(true));
    act(() => root.render(<EraObjectivesHud view={view} model={buildVpModel(view)} />));
    const by = [...host.querySelectorAll('.eo-scored-by')].map((x) => x.textContent).find((t) => t?.startsWith('Scored by'));
    expect(by).toBe('Scored by Byzantium');
  });

  it('an earlier Age you have not scored is a link to the sheet, with its card below', () => {
    const el = hud({ era: 3, scored: [] });
    expect(el.querySelector('[data-testid=era-older]')?.textContent).toContain('Classical is still open to score');
    act(() => el.querySelector<HTMLButtonElement>('[data-testid=era-older] button')!.click());
    expect(useGame.getState().vpExpanded).toBe(true);
    expect(el.querySelector('.eo-older h5')?.textContent).toBe('Classical still open');
  });
});

describe('the research screen header', () => {
  const research = (view: PlayerView) => { act(() => root.render(<EraObjectivesResearch view={view} />)); return host.querySelector<HTMLElement>('[data-testid=research-objectives]')!; };

  it('shows the card of the Age in focus with progress, and when the next Age begins', () => {
    const el = research(withV2(base, { era: 3, scored: [2] }));
    expect(el.querySelector('[data-testid=research-objectives-current] h4')?.textContent).toMatch(/^\w+ Age: score 1 VP$/);
    expect(el.querySelector('[data-testid=research-objectives-current] .eor-seal')).not.toBeNull();
    expect(rows(el.querySelector('[data-testid=research-objectives-current]')!)).toEqual([['Expand', '6/14'], ['Expand', '11/40'], ['Fight', '0/1']]);
    expect(el.querySelector('[data-testid=research-objectives-next]')?.textContent).toBe('The next Age begins when any empire enters the Renaissance era.');
  });

  it('names the Machine Age once: its era name already ends in Age', () => {
    const machine = (scored: number[]) => {
      const view = withV2(base, { era: 3, scored });
      view.eras.find((e) => e.era === 3)!.name = 'Machine Age';
      return view;
    };
    expect(research(machine([2])).querySelector('[data-testid=research-objectives-current] h4')?.textContent).toBe('Machine Age: score 1 VP');
    expect(research(machine([2, 3])).querySelector('[data-testid=research-objectives-current] h4')?.textContent).toBe('Machine Age: 1 VP scored');
  });

  it('stamps the seal in once per Age state, flashes a newly met card once, and stays still with animations off', () => {
    const media = window.matchMedia;
    window.matchMedia = ((q: string) => ({ matches: false, media: q, addEventListener() {}, removeEventListener() {} })) as unknown as typeof window.matchMedia;
    try {
      act(() => useCombatFx.getState().update({ enabled: false }));
      const still = research(withV2(base, { era: 3, scored: [], progress: { '3:0': [14, 14], '3:1': [11, 40], '3:2': [0, 1] } }));
      expect(still.querySelector('.eor-seal.stamp')).toBeNull();
      expect(still.querySelectorAll('.eo-line.flash')).toHaveLength(0);
      act(() => root.render(<></>));
      act(() => useCombatFx.getState().update({ enabled: true }));
      // A state not yet shown with animations on: the Medieval Age once you scored it, with its met card.
      const view = withV2(base, { era: 3, scored: [3], progress: { '3:0': [14, 14], '3:1': [11, 40], '3:2': [0, 1] } });
      const first = research(view);
      expect(first.querySelector('.eor-seal.stamp')).not.toBeNull();
      act(() => root.render(<></>));
      const again = research(view);
      expect(again.querySelector('.eor-seal.stamp')).toBeNull();
      expect(again.querySelectorAll('.eo-line.flash')).toHaveLength(0);
    } finally { window.matchMedia = media; }
  });

  it('before any Age begins it says so and when the first one does', () => {
    const el = research({ ...withV2(base, { era: 1, scored: [] }), eras: [], objective_progress: [] });
    expect(el.querySelector('[data-testid=research-objectives-current] h4')?.textContent).toBe('No Age has begun');
    expect(el.textContent).toContain('The first Age begins when any empire enters the Classical era');
    expect(el.querySelector('[data-testid=research-objectives-next]')).toBeNull();
    expect(el.querySelector('.eor-seal')).toBeNull();
  });
});
