// @vitest-environment happy-dom
import { act } from 'react';
import { createRoot, type Root } from 'react-dom/client';
import { afterEach, beforeAll, beforeEach, describe, expect, it, vi } from 'vitest';
import { FixtureClient } from '../test/fixtureEngine';
import { withV2 } from '../test/vpV2Fixture';
import type { PlayerView } from '../types';
import { momentsForTurn, type Moment } from './moments';
import { EraUpTakeover } from './EraUpTakeover';
import { eraArt } from './eraArt';
import { settleResearch } from './researchSettled';

let view: PlayerView;
beforeAll(async () => {
  const client = new FixtureClient();
  await client.newGame(await client.standardSetup('42', 'India', 'Byzantium', 'Ada', 'Bo'));
  view = client.view(0);
});

let root: Root, host: HTMLDivElement;
beforeEach(() => {
  vi.stubGlobal('IS_REACT_ACT_ENVIRONMENT', true);
  document.body.innerHTML = '';
  host = document.body.appendChild(document.createElement('div'));
  root = createRoot(host);
});
afterEach(() => { act(() => root.unmount()); vi.unstubAllGlobals(); });

function enter(opts: { first?: boolean; queued?: boolean } = {}): Moment {
  const v = withV2(view, { era: 2 });
  v.research = { ...v.research, queue: opts.queued ? [v.research.available[0]?.tech ?? v.research.known[0]] : [] } as typeof v.research;
  return momentsForTurn([{ type: 'EraEntered', player: 0, era: 2, first: opts.first ?? false }], 0, view, v)[0];
}
const render = (moment: Moment, dismiss = vi.fn()) => { act(() => root.render(<EraUpTakeover moment={moment} dismiss={dismiss} />)); return dismiss; };

describe('the era-up takeover', () => {
  it('shows the era, what changes and the one primary', () => {
    render(enter({ first: true }));
    const t = host.querySelector('[data-testid=era-takeover]')!;
    expect(t.getAttribute('aria-label')).toBe('The Classical era begins');
    expect(t.querySelector('.et-title')?.textContent).toBe('The Classical Era');
    expect(t.querySelector('.et-numeral')?.textContent).toBe('II');
    expect(t.querySelector('.et-kicker')?.textContent).toBe('You are the first empire into this era');
    // The points open now live on the Age's score check, not here.
    expect(t.querySelector('[data-testid=era-up-objectives]')).toBeNull();
    // The board change: the Ancient city beside the Classical one.
    expect(t.querySelectorAll('[data-testid=era-up-board] img')).toHaveLength(2);
    expect(t.querySelector('[data-testid=era-up-board]')?.textContent).toContain('Ancient');
    expect(t.querySelector('.et-art img')?.getAttribute('src')).toBe(eraArt(1)!.small);
    expect(t.querySelectorAll('[data-primary-action]')).toHaveLength(1);
  });

  it('the primary is Choose research when nothing is queued, else Continue; either closes the card', () => {
    const idle = render(enter());
    const button = () => host.querySelector<HTMLButtonElement>('[data-primary-action]')!;
    expect(button().textContent).toContain('Choose research');
    expect(host.querySelector('[data-testid=era-up-choose]')).not.toBeNull();
    act(() => button().click());
    expect(idle).toHaveBeenCalledTimes(1);
    act(() => root.unmount()); root = createRoot(host);
    const queued = render(enter({ queued: true }));
    expect(button().textContent).toContain('Continue');
    expect(host.querySelector('[data-testid=era-up-choose]')).toBeNull();
    act(() => button().click());
    expect(queued).toHaveBeenCalledTimes(1);
  });

  it('every era has a painting', () => {
    for (let era = 0; era < 7; era++) expect(eraArt(era)).toMatchObject({ small: expect.stringContaining('1280'), large: expect.stringContaining('1920') });
  });

  it('research picked before the card shows (behind the research screen) turns Choose research into Continue', () => {
    const idle = enter();
    expect(idle.eraUp?.primary).toBe('research');
    expect(settleResearch(idle, false)).toBe(idle);
    render(settleResearch(idle, true)!);
    expect(host.querySelector('[data-testid=era-takeover]')?.getAttribute('data-primary')).toBe('continue');
    expect(host.querySelector('[data-primary-action]')?.textContent).toContain('Continue');
    expect(host.querySelector('[data-testid=era-up-choose]')).toBeNull();
  });
});
