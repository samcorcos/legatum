// @vitest-environment happy-dom
import { act, StrictMode } from 'react';
import { createRoot, type Root } from 'react-dom/client';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { CelebrationProvider, CounterDelta, MomentLane } from './MomentLane';
import { celebrationFixture, unlockCelebrationFixture } from '../test/celebrationFixture';
import { useGame } from '../store';
import { SeatBar } from '../ui/SeatBar';
import { TurnReport } from '../ui/TurnReport';
import { cancelFeedback } from '../feel';

let root: Root, host: HTMLDivElement;
beforeEach(async () => {
  vi.stubGlobal('IS_REACT_ACT_ENVIRONMENT', true);
  document.body.innerHTML = '';
  host = document.body.appendChild(document.createElement('div'));
  root = createRoot(host);
  useGame.setState(useGame.getInitialState(), true);
  vi.useFakeTimers();
  await celebrationFixture('notable', 5);
});
afterEach(() => {
  act(() => root.unmount());
  cancelFeedback();
  useGame.setState(useGame.getInitialState(), true);
  vi.useRealTimers(); vi.restoreAllMocks(); vi.unstubAllGlobals();
});
function render() {
  act(() => root.render(<StrictMode><CelebrationProvider><MomentLane /><CounterDelta counter="gold" /><SeatBar /><TurnReport /></CelebrationProvider></StrictMode>));
}
const card = () => host.querySelector('.moment-card');
const click = (selector: string) => act(() => host.querySelector<HTMLButtonElement>(selector)!.click());

it('auto-dismisses on schedule, never plays a fourth card, and removes the overflow chip', () => {
  render(); expect(card()).not.toBeNull();
  expect(host.querySelector('.moment-more')?.textContent).toContain('+2 more');
  act(() => vi.advanceTimersByTime(3499)); expect(card()).not.toBeNull();
  act(() => vi.advanceTimersByTime(1)); expect(card()).not.toBeNull();
  act(() => vi.advanceTimersByTime(3500)); expect(card()).not.toBeNull();
  act(() => vi.advanceTimersByTime(3500)); expect(card()).toBeNull();
  expect(host.querySelector('.moment-more')).toBeNull();
});

it('a pointer on End Turn interrupts the story and still resolves the real next turn', async () => {
  render();
  const endTurn = host.querySelector<HTMLButtonElement>('.touch-turn-controls .primary')!;
  expect(endTurn.disabled).toBe(false); expect(card()).not.toBeNull();
  await act(async () => {
    const pointer = new PointerEvent('pointerdown', { bubbles: true, cancelable: true });
    expect(endTurn.dispatchEvent(pointer)).toBe(true);
    expect(pointer.defaultPrevented).toBe(false);
    endTurn.click();
    await vi.advanceTimersByTimeAsync(0);
  });
  // With decisions pending, End Turn asks once (B1); confirming resolves the turn.
  await act(async () => {
    [...host.querySelectorAll<HTMLButtonElement>('dialog.end-turn-confirmation button')].find((b) => b.textContent === 'End turn anyway')!.click();
    await vi.advanceTimersByTimeAsync(200);
  });
  expect(useGame.getState().views![0].turn).toBe(3);
  expect(host.textContent).not.toContain('The buried hoard');
});

it('a viewer or session change discards the old private reward before the next timer', () => {
  render(); expect(card()).not.toBeNull();
  const view = useGame.getState().views![0];
  act(() => useGame.setState({ report: null, views: [{ ...view, me: 2, team: 2 }] }));
  expect(card()).toBeNull();
  act(() => vi.advanceTimersByTime(10000)); expect(card()).toBeNull();
});

it('Escape and background taps cancel all queued cards without taking keyboard focus', () => {
  const opener = document.body.appendChild(document.createElement('button')); opener.focus();
  render(); expect(document.activeElement).toBe(opener);
  const escape = new KeyboardEvent('keydown', { key: 'Escape', bubbles: true, cancelable: true });
  act(() => window.dispatchEvent(escape));
  expect(card()).toBeNull();
  // Consumed, so the game's Escape does not also close the report the story was holding.
  expect(escape.defaultPrevented).toBe(true);
  act(() => vi.advanceTimersByTime(20000)); expect(card()).toBeNull();
  expect(document.activeElement).toBe(opener);
});

it('opens collapsed discoveries during the sequence and removes every queued card', () => {
  render(); click('.moment-more');
  expect(card()).toBeNull();
  expect(host.querySelector('.moment-more')).toBeNull();
  expect(useGame.getState()).toMatchObject({ reportOpen: true, reportFocus: 'discoveries' });
  expect(host.querySelector('.report')).not.toBeNull();
});

it('holds the report for only the first card, while the later cards and overflow continue', () => {
  useGame.setState({ reportOpen: true });
  render();
  expect(host.querySelector('.report')).toBeNull();
  act(() => vi.advanceTimersByTime(3499));
  expect(host.querySelector('.report')).toBeNull();
  act(() => vi.advanceTimersByTime(1));
  expect(host.querySelector('.report')).not.toBeNull();
  expect(card()).not.toBeNull();
  expect(host.querySelector('.moment-more')).not.toBeNull();
  act(() => vi.advanceTimersByTime(3500));
  expect(card()).not.toBeNull();
  expect(host.querySelector('.report')).not.toBeNull();
  act(() => vi.advanceTimersByTime(3500));
  expect(card()).toBeNull();
  expect(host.querySelector('.moment-more')).toBeNull();
  expect(host.querySelector('.report')).not.toBeNull();
});

it.each(['.moment-effect', '.moment-show-map', '.moment-dismiss'])('a tap on %s releases the report immediately without discarding later cards', (selector) => {
  useGame.setState({ reportOpen: true }); render();
  expect(host.querySelector('.report')).toBeNull();
  click(selector);
  expect(host.querySelector('.report')).not.toBeNull();
  expect(card()).not.toBeNull();
});

it.each(['Escape', 'background'])('%s releases the report and interrupts the sequence', (gesture) => {
  useGame.setState({ reportOpen: true }); render();
  act(() => {
    if (gesture === 'Escape') window.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', cancelable: true }));
    else document.body.dispatchEvent(new PointerEvent('pointerdown', { bubbles: true, cancelable: true }));
  });
  expect(host.querySelector('.report')).not.toBeNull();
  expect(card()).toBeNull();
});

it('keeps a tapped selection control in place until its click is delivered', () => {
  useGame.setState({ reportOpen: true }); render();
  const action = document.body.appendChild(document.createElement('button'));
  const clicked = vi.fn(); action.addEventListener('click', clicked);
  act(() => action.dispatchEvent(new PointerEvent('pointerdown', { bubbles: true })));
  expect(card()).not.toBeNull();
  expect(host.querySelector('.report')).toBeNull();
  act(() => action.click());
  expect(clicked).toHaveBeenCalledOnce();
  expect(card()).toBeNull();
  expect(host.querySelector('.report')).not.toBeNull();
});

it('starts a fresh first-card hold on the next report without reopening a dismissed report', async () => {
  useGame.setState({ reportOpen: true }); render(); click('.moment-dismiss');
  click('.report-head button[aria-label="Close report"]');
  act(() => vi.advanceTimersByTime(10000));
  expect(host.querySelector('.report')).toBeNull();
  await act(async () => { await celebrationFixture('major', 3); useGame.setState({ reportOpen: true }); });
  expect(host.querySelector('.report')).toBeNull();
  act(() => vi.advanceTimersByTime(5000));
  expect(host.querySelector('.report')).not.toBeNull();
  expect(card()).not.toBeNull();
});

it('keeps Show on map prominent and never offers an empty details disclosure', async () => {
  await celebrationFixture('legendary'); render();
  expect(host.querySelector('button.moment-effect')).toBeNull();
  expect(host.querySelector('.moment-effect')?.textContent).toContain('Completed in');
  const focus = useGame.getState().focus;
  click('.moment-show-map');
  expect(useGame.getState().focus).not.toBe(focus);
  expect(card()).not.toBeNull();
});

describe('the tech unlock card', () => {
  const unlock = () => host.querySelector('.moment-card.unlock');
  const key = (name: string) => { const event = new KeyboardEvent('keydown', { key: name, bubbles: true, cancelable: true }); act(() => { window.dispatchEvent(event); }); return event; };
  const buttons = () => [...unlock()!.querySelectorAll('footer button')].map((b) => b.textContent);

  it('waits for the player: no timer ends it, and it names what it unlocks and what to do', async () => {
    await unlockCelebrationFixture(); render();
    expect(unlock()?.textContent).toContain('Bronze Working');
    expect(unlock()?.textContent).toContain('Build a Workshop in any city: +2 Production');
    expect(buttons()[0]).toMatch(/^Show Workshop in .+Return$/);
    act(() => vi.advanceTimersByTime(60_000));
    expect(unlock()).not.toBeNull();
  });

  it('Escape puts it off, and is consumed so the game does not also act on it', async () => {
    await unlockCelebrationFixture(); render();
    expect(key('Escape').defaultPrevented).toBe(true);
    expect(unlock()).toBeNull();
    expect(useGame.getState().selection).toBeNull();
  });

  it('Return opens the city that builds it, is consumed so End Turn does not also fire, and keeps the cards queued behind it', async () => {
    await unlockCelebrationFixture(true, true, 2); render();
    const city = useGame.getState().views![0].cities.find((c) => c.owner === 0)!;
    expect(key('Enter').defaultPrevented).toBe(true);
    expect(useGame.getState().selection).toEqual({ kind: 'city', id: city.id });
    expect(unlock()).toBeNull();
    expect(useGame.getState().ready[0]).toBe(false);
    // The two ordinary celebrations are still to play, as after Esc.
    expect(card()?.textContent).toContain('The buried hoard');
    act(() => vi.advanceTimersByTime(3500));
    expect(card()?.textContent).toContain('The buried hoard');
  });

  it('with nothing buildable yet and research set the primary is Continue, which only dismisses', async () => {
    await unlockCelebrationFixture(false); render();
    expect(buttons()).toEqual(['Continue Return']);
    key('Enter');
    expect(unlock()).toBeNull();
    expect(useGame.getState().selection).toBeNull();
    expect(useGame.getState().techOpen).toBe(false);
  });

  it('idle research gets a Choose research button beside Later; the building stays primary', async () => {
    await unlockCelebrationFixture(true, false); render();
    expect(unlock()?.textContent).toContain('No research is set.');
    expect(buttons().map((t) => t?.replace(/\s+/g, ' '))).toEqual([expect.stringMatching(/^Show Workshop in .+ Return$/), 'Choose research T', 'Later Esc']);
    act(() => unlock()!.querySelectorAll<HTMLButtonElement>('footer button')[1].click());
    expect(unlock()).toBeNull();
    expect(useGame.getState().techOpen).toBe(true);
    expect(useGame.getState().selection).toBeNull();
  });

  it('idle research with nothing to build makes Choose research the primary on Return', async () => {
    await unlockCelebrationFixture(false, false); render();
    expect(buttons().map((t) => t?.replace(/\s+/g, ' '))).toEqual(['Choose research Return', 'Later Esc']);
    expect(unlock()?.querySelector('.btn.primary')?.textContent).toContain('Choose research');
    key('Enter');
    expect(unlock()).toBeNull();
    expect(useGame.getState().techOpen).toBe(true);
  });

  it('T dismisses it while the game opens the tech panel', async () => {
    await unlockCelebrationFixture(true, false); render();
    expect(key('t').defaultPrevented).toBe(false);
    expect(unlock()).toBeNull();
  });

  it('only its own controls close it: not a click or drag elsewhere, nor an opened report', async () => {
    await unlockCelebrationFixture(true, true, 1); render();
    const endTurn = host.querySelector<HTMLButtonElement>('.touch-turn-controls .primary')!;
    act(() => { endTurn.dispatchEvent(new PointerEvent('pointerdown', { bubbles: true, cancelable: true })); });
    act(() => { document.body.dispatchEvent(new PointerEvent('pointerdown', { bubbles: true, cancelable: true })); document.body.click(); });
    act(() => { window.dispatchEvent(new PointerEvent('pointermove', { bubbles: true })); });
    expect(unlock()).not.toBeNull();
    // The report stays held behind it until the card is dealt with.
    useGame.setState({ reportOpen: true });
    expect(host.querySelector('.report')).toBeNull();
    key('Escape');
    expect(unlock()).toBeNull();
    expect(host.querySelector('.report')).not.toBeNull();
  });

  it('can be reopened from the report line, plays alone, and leaves the turn’s own story alone', async () => {
    await unlockCelebrationFixture(true, true, 1); render();
    key('Escape');
    act(() => vi.advanceTimersByTime(20_000));
    expect(card()).toBeNull();
    act(() => useGame.getState().reopenUnlock('BronzeWorking'));
    expect(unlock()?.textContent).toContain('Bronze Working');
    key('Escape');
    expect(card()).toBeNull();
    expect(useGame.getState().unlockReopen).toBeNull();
    act(() => vi.advanceTimersByTime(20_000));
    expect(card()).toBeNull();
    act(() => useGame.getState().reopenUnlock('BronzeWorking'));
    expect(unlock()).not.toBeNull();
  });

  it('research picked while it waits behind the research screen drops Choose research, so Return does not reopen it', async () => {
    await unlockCelebrationFixture(false, false); render();
    const dialog = document.body.appendChild(document.createElement('dialog'));
    await act(async () => { dialog.setAttribute('open', ''); await Promise.resolve(); });
    expect(unlock()).toBeNull();
    const tech = useGame.getState().views![0].research.available[0].tech;
    act(() => useGame.setState({ orders: [[{ type: 'SetResearchQueue', techs: [tech] }]] }));
    await act(async () => { dialog.removeAttribute('open'); await Promise.resolve(); });
    dialog.remove();
    expect(unlock()?.textContent).not.toContain('No research is set.');
    expect(buttons()).toEqual(['Continue Return']);
    key('Enter');
    expect(unlock()).toBeNull();
    expect(useGame.getState().techOpen).toBe(false);
  });

  it('waits out an open dialog instead of hiding behind it, and Esc is left to the dialog', async () => {
    await unlockCelebrationFixture(); render();
    const dialog = document.body.appendChild(document.createElement('dialog'));
    await act(async () => { dialog.setAttribute('open', ''); await Promise.resolve(); });
    expect(unlock()).toBeNull();
    expect(key('Escape').defaultPrevented).toBe(false);
    expect(key('Enter').defaultPrevented).toBe(false);
    await act(async () => { dialog.removeAttribute('open'); await Promise.resolve(); });
    expect(unlock()).not.toBeNull();
    dialog.remove();
  });
});
