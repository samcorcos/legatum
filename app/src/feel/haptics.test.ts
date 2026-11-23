// @vitest-environment happy-dom
import { afterEach, beforeEach, expect, it, vi } from 'vitest';

let haptics: typeof import('./haptics');
let preferences: typeof import('./preferences');
let vibrate: ReturnType<typeof vi.fn>;

beforeEach(async () => {
  vi.resetModules();
  vi.clearAllMocks();
  vi.useFakeTimers();
  localStorage.clear();
  delete window.__feelLog;
  vibrate = vi.fn();
  vi.stubGlobal('navigator', { userAgent: 'Android', maxTouchPoints: 1, vibrate });
  haptics = await import('./haptics');
  preferences = await import('./preferences');
});
afterEach(() => {
  haptics.cancelFeedback();
  vi.useRealTimers();
  vi.unstubAllGlobals();
  vi.unstubAllEnvs();
});

it('debounces across semantic calls, then accepts a separate action', () => {
  haptics.feel.select();
  haptics.feel.commit();
  haptics.feel.select();
  expect(vibrate.mock.calls).toEqual([[8]]);
  vi.advanceTimersByTime(80);
  haptics.feel.commit();
  expect(vibrate.mock.calls).toEqual([[8], [20]]);
  expect(window.__feelLog?.map((e) => e.name)).toEqual(['select', 'commit']);
});

it('defaults on, persists the toggle and cancels pending patterns and turn feedback when off', async () => {
  expect(preferences.readHapticsEnabled()).toBe(true);
  haptics.feel.victory();
  preferences.useHaptics.getState().setEnabled(false);
  expect(localStorage.getItem(preferences.HAPTICS_KEY)).toBe('false');
  expect(preferences.readHapticsEnabled()).toBe(false);
  haptics.feel.commit();
  haptics.playTurnFeedback(['cityLost', 'eliminated']);
  await vi.advanceTimersByTimeAsync(1000);
  expect(vibrate).toHaveBeenCalledTimes(1);
  preferences.useHaptics.getState().setEnabled(true);
  haptics.feel.select();
  expect(vibrate).toHaveBeenCalledTimes(2);
});

it.each([
  ['Macintosh; Intel Mac OS X', 0], ['Windows NT 10.0', 0], ['X11; Linux x86_64', 0], ['iPhone', 1], ['Macintosh; Intel Mac OS X', 5],
])('is a hardware no-op on %s (touch=%s)', async (userAgent, maxTouchPoints) => {
  vi.stubGlobal('navigator', { userAgent, maxTouchPoints, vibrate });
  expect(haptics.hapticsAvailable()).toBe(false);
  haptics.feel.commit();
  await vi.runAllTimersAsync();
  expect(vibrate).not.toHaveBeenCalled();
  expect(window.__feelLog?.map((e) => e.name)).toEqual(['commit']);
});

it('uses light feedback for Unready and preserves the victory timing', async () => {
  haptics.feel.ready(false);
  expect(vibrate).toHaveBeenLastCalledWith(12);
  vi.advanceTimersByTime(80);
  haptics.feel.victory();
  expect(vibrate).toHaveBeenLastCalledWith([12, 25, 18]);
  await vi.advanceTimersByTimeAsync(179);
  expect(vibrate).toHaveBeenCalledTimes(2);
  await vi.advanceTimersByTimeAsync(1);
  expect(vibrate).toHaveBeenLastCalledWith(35);
  await vi.advanceTimersByTimeAsync(180);
  expect(vibrate).toHaveBeenLastCalledWith(25);
});

it('spaces turn calls 120 ms apart, replacing old turns and stopping when the viewer changes', async () => {
  haptics.feel.ready();
  let current = true;
  haptics.playTurnFeedback(['turnResolved', 'cityFounded', 'techComplete'], () => current);
  await vi.advanceTimersByTimeAsync(120);
  expect(window.__feelLog?.map((e) => e.name)).toEqual(['ready', 'turnResolved']);
  await vi.advanceTimersByTimeAsync(120);
  expect(window.__feelLog?.at(-1)?.name).toBe('cityFounded');
  current = false;
  await vi.advanceTimersByTimeAsync(120);
  expect(window.__feelLog?.map((e) => e.at)).toEqual([0, 120, 240]);
  haptics.playTurnFeedback(['cityLost']);
  haptics.playTurnFeedback(['vpGained']);
  await vi.advanceTimersByTimeAsync(120);
  expect(window.__feelLog?.at(-1)?.name).toBe('vpGained');
});

it('catches throwing browser vibration', () => {
  vi.stubGlobal('navigator', { userAgent: 'Android', vibrate: () => { throw new Error('unavailable'); } });
  expect(() => haptics.feel.commit()).not.toThrow();
});

it('drops a queued pulse when the toggle turns off', async () => {
  haptics.feel.victory();
  preferences.useHaptics.getState().setEnabled(false);
  await vi.advanceTimersByTimeAsync(500);
  expect(vibrate).toHaveBeenCalledTimes(1);
});

it('keeps the Ready acknowledgement when a turn feedback sequence is cancelled', async () => {
  haptics.feel.ready();
  haptics.playTurnFeedback(['turnResolved', 'cityFounded']);
  haptics.cancelTurnFeedback();
  await vi.advanceTimersByTimeAsync(500);
  expect(vibrate.mock.calls).toEqual([[35]]);
});

it('does not create a production log', () => {
  vi.stubEnv('DEV', false);
  haptics.feel.select();
  expect(window.__feelLog).toBeUndefined();
});

it('delivers the first real discovery after a long first-render stall without losing it to debounce', async () => {
  // Simulate a busy event loop: timers are overdue, but performance.now reflects delivery time.
  let now = 0;
  const clock = vi.spyOn(performance, 'now').mockImplementation(() => now);
  haptics.playTurnFeedback(['turnResolved', 'discovery'], () => true, { discovery: 'sting.notable' });
  now = 1500;
  await vi.advanceTimersByTimeAsync(240);
  expect(window.__feelLog?.map((e) => e.name)).toEqual(['turnResolved']);
  now = 1620;
  await vi.advanceTimersByTimeAsync(120);
  expect(window.__feelLog?.map((e) => e.name)).toEqual(['turnResolved', 'discovery']);
  expect(window.__feelLog?.at(-1)?.soundHook).toBe('sting.notable');
  clock.mockRestore();
});

it('interrupts celebration hooks while keeping city and battle losses queued', async () => {
  haptics.playTurnFeedback(['turnResolved', 'discovery', 'cityLost', 'battleLost'], () => true, { discovery: 'sting.major' });
  await vi.advanceTimersByTimeAsync(130);
  haptics.cancelCelebrationFeedback();
  haptics.feel.select();
  await vi.advanceTimersByTimeAsync(600);
  expect(window.__feelLog?.map((e) => e.name)).toEqual(['turnResolved', 'cityLost', 'battleLost']);
});
