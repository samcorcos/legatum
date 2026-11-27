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
