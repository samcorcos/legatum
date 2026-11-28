// @vitest-environment happy-dom
import { afterEach, describe, expect, it, vi } from 'vitest';
import { observeHudLayout } from './hudObserver';

afterEach(() => {
  document.body.innerHTML = '';
  vi.restoreAllMocks();
  vi.unstubAllGlobals();
});

describe('observeHudLayout', () => {
  it('samples immediately without setInterval polling, reacts to DOM mutations, ignores compass writes, and cleans up', async () => {
    const intervalSpy = vi.spyOn(window, 'setInterval');
    const game = document.createElement('div');
    game.className = 'game';
    const wrap = document.createElement('div');
    const compass = document.createElement('div');
    compass.className = 'map-compass';
    wrap.appendChild(compass);
    game.appendChild(wrap);
    document.body.appendChild(game);

    const rafQueue: FrameRequestCallback[] = [];
    vi.stubGlobal('requestAnimationFrame', (cb: FrameRequestCallback) => { rafQueue.push(cb); return rafQueue.length; });
    vi.stubGlobal('cancelAnimationFrame', vi.fn());

    const disconnectRo = vi.fn();
    const observeRo = vi.fn();
    vi.stubGlobal('ResizeObserver', class {
      observe = observeRo;
      unobserve = vi.fn();
      disconnect = disconnectRo;
    });

    const sample = vi.fn();
    const stop = observeHudLayout(wrap, '.game .panel', sample);

    expect(intervalSpy).not.toHaveBeenCalled();
    expect(sample).toHaveBeenCalledTimes(1);
    expect(observeRo).toHaveBeenCalledWith(wrap);

    // Self-mutation on .map-compass must not schedule a re-sample.
    compass.style.transform = 'translate(12px, 34px)';
    await Promise.resolve();
    expect(rafQueue.length).toBe(0);

    // Mounting a HUD panel schedules a coalesced re-sample and starts observing the panel.
    const panel = document.createElement('div');
    panel.className = 'panel';
    game.appendChild(panel);
    await Promise.resolve();
    expect(rafQueue.length).toBe(1);
    rafQueue.shift()!(16);
    expect(sample).toHaveBeenCalledTimes(2);
    expect(observeRo).toHaveBeenCalledWith(panel);

    // Window resize also schedules a re-sample.
    window.dispatchEvent(new Event('resize'));
    expect(rafQueue.length).toBe(1);
    rafQueue.shift()!(32);
    expect(sample).toHaveBeenCalledTimes(3);

    // Cleanup disconnects observers and stops listening to resize.
    stop();
    expect(disconnectRo).toHaveBeenCalledTimes(1);
    window.dispatchEvent(new Event('resize'));
    expect(rafQueue.length).toBe(0);
  });

  it('resamples when a transform-animated panel finishes its animation/transition and ignores compass transitions', async () => {
    const game = document.createElement('div');
    game.className = 'game';
    const wrap = document.createElement('div');
    const compass = document.createElement('div');
    compass.className = 'map-compass';
    wrap.appendChild(compass);
    game.appendChild(wrap);
    document.body.appendChild(game);

    const rafQueue: FrameRequestCallback[] = [];
    vi.stubGlobal('requestAnimationFrame', (cb: FrameRequestCallback) => { rafQueue.push(cb); return rafQueue.length; });
    vi.stubGlobal('cancelAnimationFrame', vi.fn());
    vi.stubGlobal('ResizeObserver', class {
      observe = vi.fn();
      unobserve = vi.fn();
      disconnect = vi.fn();
    });

    let panelTop = 90; // Mid-slideUp transform offset (+10px from final top=80)
    const panel = document.createElement('div');
    panel.className = 'panel';
    vi.spyOn(panel, 'getBoundingClientRect').mockImplementation(() => new DOMRect(0, panelTop, 200, 120));

    let recordedTop: number | null = null;
    const stop = observeHudLayout(wrap, '.game .panel', () => {
      const el = document.querySelector('.game .panel');
      recordedTop = el ? el.getBoundingClientRect().top : null;
    });

    // Mount panel during slideUp animation -> initial mutation samples mid-animation top=90.
    game.appendChild(panel);
    await Promise.resolve();
    expect(rafQueue.length).toBe(1);
    rafQueue.shift()!(16);
    expect(recordedTop).toBe(90);

    // Self-write transition on .map-compass must not schedule a resample.
    compass.dispatchEvent(new Event('transitionend', { bubbles: true }));
    expect(rafQueue.length).toBe(0);

    // Panel reaches its final transform position (top=80) and fires animationend with no resize/DOM mutation.
    panelTop = 80;
    panel.dispatchEvent(new Event('animationend', { bubbles: true }));
    expect(rafQueue.length).toBe(1);
    rafQueue.shift()!(240);
    expect(recordedTop).toBe(80);

    // Transition settling also triggers a coalesced resample.
    panelTop = 76;
    panel.dispatchEvent(new Event('transitionend', { bubbles: true }));
    expect(rafQueue.length).toBe(1);
    rafQueue.shift()!(320);
    expect(recordedTop).toBe(76);

    // Cleanup removes animation/transition listeners.
    stop();
    panel.dispatchEvent(new Event('animationend', { bubbles: true }));
    panel.dispatchEvent(new Event('transitionend', { bubbles: true }));
    expect(rafQueue.length).toBe(0);
  });

  it('falls back cleanly when ResizeObserver and MutationObserver are unavailable', () => {
    vi.stubGlobal('ResizeObserver', undefined);
    vi.stubGlobal('MutationObserver', undefined);
    const wrap = document.createElement('div');
    document.body.appendChild(wrap);
    const sample = vi.fn();

    const stop = observeHudLayout(wrap, '.panel', sample);
    expect(sample).toHaveBeenCalledTimes(1);
    expect(() => stop()).not.toThrow();
  });
});
