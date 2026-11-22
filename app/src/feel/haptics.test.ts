// @vitest-environment happy-dom
import { afterEach, beforeEach, expect, it, vi } from 'vitest';

const native = vi.hoisted(() => ({
  impactFeedback: vi.fn().mockResolvedValue(null), notificationFeedback: vi.fn().mockResolvedValue(null),
  selectionFeedback: vi.fn().mockResolvedValue(null),
}));
vi.mock('@tauri-apps/plugin-haptics', () => native);
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
  ['Macintosh; Intel Mac OS X', 0, false], ['Macintosh; Intel Mac OS X', 0, true],
  ['Windows NT 10.0', 0, true], ['X11; Linux x86_64', 0, false], ['iPhone', 1, false],
])('is a hardware no-op on %s (touch=%s, Tauri=%s)', async (userAgent, maxTouchPoints, tauri) => {
  vi.stubGlobal('navigator', { userAgent, maxTouchPoints, vibrate });
  if (tauri) vi.stubGlobal('__TAURI_INTERNALS__', {});
  haptics.feel.commit();
  await vi.runAllTimersAsync();
  expect(vibrate).not.toHaveBeenCalled();
  expect(native.impactFeedback).not.toHaveBeenCalled();
  expect(window.__feelLog?.map((e) => e.name)).toEqual(['commit']);
});

it.each([['iPhone', 1], ['Android', 1], ['Macintosh; Intel Mac OS X', 5]] as const)(
  'loads the plugin only in the %s native shell', async (userAgent, maxTouchPoints) => {
    vi.stubGlobal('navigator', { userAgent, maxTouchPoints, vibrate });
    vi.stubGlobal('__TAURI_INTERNALS__', {});
    haptics.feel.commitAttack();
    await vi.dynamicImportSettled();
    expect(native.impactFeedback).toHaveBeenCalledWith('rigid');
    expect(vibrate).not.toHaveBeenCalled();
  },
);

it('uses light feedback for Unready and preserves the victory timing', async () => {
  vi.stubGlobal('__TAURI_INTERNALS__', {});
  haptics.feel.ready(false);
  await vi.dynamicImportSettled();
  expect(native.impactFeedback).toHaveBeenLastCalledWith('light');
  vi.advanceTimersByTime(80);
  haptics.feel.victory();
  await vi.dynamicImportSettled();
  expect(native.notificationFeedback).toHaveBeenLastCalledWith('success');
  await vi.advanceTimersByTimeAsync(179);
  expect(native.impactFeedback).toHaveBeenCalledTimes(1);
  await vi.advanceTimersByTimeAsync(1);
  expect(native.impactFeedback).toHaveBeenLastCalledWith('heavy');
  await vi.advanceTimersByTimeAsync(180);
  expect(native.impactFeedback).toHaveBeenLastCalledWith('rigid');
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

it('catches absent native plugin errors and throwing browser vibration', async () => {
  vi.stubGlobal('__TAURI_INTERNALS__', {});
  native.impactFeedback.mockRejectedValueOnce(new Error('plugin not registered'));
  expect(() => haptics.feel.commit()).not.toThrow();
  await vi.dynamicImportSettled();
  vi.unstubAllGlobals();
  vi.advanceTimersByTime(80);
  vi.stubGlobal('navigator', { userAgent: 'Android', vibrate: () => { throw new Error('unavailable'); } });
  expect(() => haptics.feel.commit()).not.toThrow();
});

it('does not replay an import that completes after the toggle turns off', async () => {
  vi.stubGlobal('__TAURI_INTERNALS__', {});
  haptics.feel.commit();
  preferences.useHaptics.getState().setEnabled(false);
  await vi.dynamicImportSettled();
  expect(native.impactFeedback).not.toHaveBeenCalled();
});

it('keeps the native Ready acknowledgement when a hot-seat handoff cancels turn feedback during the import', async () => {
  vi.stubGlobal('__TAURI_INTERNALS__', {});
  haptics.feel.ready();
  haptics.playTurnFeedback(['turnResolved', 'cityFounded']);
  haptics.cancelTurnFeedback();
  await vi.dynamicImportSettled();
  await vi.advanceTimersByTimeAsync(500);
  expect(native.impactFeedback.mock.calls).toEqual([['heavy']]);
  expect(native.notificationFeedback).not.toHaveBeenCalled();
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
