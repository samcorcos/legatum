// Semantic contract: docs/design/haptics.md.
// Haptics go through the Vibration API, which only Android browsers offer.
import { useHaptics } from './preferences';
import { playSound, soundAudible, SOUND_FILES, type CelebrationSound } from './sound';

type Pulse = 'selection' | 'light' | 'medium' | 'heavy' | 'soft' | 'rigid' | 'success' | 'warning' | 'error';
type Pattern = readonly (readonly [delay: number, pulse: Pulse])[];
const patterns = {
  select: [[0, 'selection']], snap: [[0, 'selection']], preview: [[0, 'soft']],
  commit: [[0, 'medium']], commitAttack: [[0, 'rigid']], queue: [[0, 'medium']],
  purchase: [[0, 'medium'], [60, 'success']], invalid: [[0, 'warning']],
  ready: [[0, 'heavy']], longPress: [[0, 'medium']], sheet: [[0, 'selection']],
  turnResolved: [[0, 'soft']], battleWon: [[0, 'medium']], battleLost: [[0, 'heavy']],
  cityFounded: [[0, 'success']], cityCaptured: [[0, 'success'], [60, 'heavy']], cityLost: [[0, 'error']],
  discovery: [[0, 'light'], [60, 'success']],
  techComplete: [[0, 'success']], buildingComplete: [[0, 'light']],
  greatWork: [[0, 'success'], [60, 'heavy']], vpGained: [[0, 'medium']], vpLost: [[0, 'warning']],
  warning: [[0, 'warning']], warDeclared: [[0, 'error'], [120, 'heavy']], eliminated: [[0, 'error'], [60, 'heavy']],
  victory: [[0, 'success'], [180, 'heavy'], [360, 'rigid']], defeat: [[0, 'heavy']],
  partnerReady: [[0, 'light']], allReady: [[0, 'medium']], seatTaken: [[0, 'warning']],
  teamPing: [[0, 'light']],
} as const satisfies Record<string, Pattern>;

export type FeelName = keyof typeof patterns;
export interface FeelLogEntry { name: FeelName; at: number; ready?: boolean; soundHook?: CelebrationSound }
declare global { interface Window { __feelLog?: FeelLogEntry[] } }

const DEBOUNCE_MS = 80;
const timers = new Set<ReturnType<typeof setTimeout>>();
let blockedUntil = -Infinity;
let generation = 0;
let turnGeneration = 0;
let celebrationGeneration = 0;
let lastWasTurn = false;

/** A map gesture skips celebratory beats, never losses or other consequential feedback. */
export function cancelCelebrationFeedback(): void {
  celebrationGeneration++;
}

/** Cancel the prior turn's feedback when the viewed game changes. */
export function cancelTurnFeedback(): void {
  turnGeneration++;
  if (lastWasTurn) blockedUntil = -Infinity;
}

/** Also invalidates queued pulses, so turning off cannot emit a late one. */
export function cancelFeedback(): void {
  generation++;
  timers.forEach(clearTimeout);
  timers.clear();
}

useHaptics.subscribe((state, previous) => {
  if (!state.enabled && previous.enabled) cancelFeedback();
});

function later(fn: () => void, delay: number): void {
  const timer = setTimeout(() => {
    timers.delete(timer);
    try { fn(); } catch { /* Feedback callbacks must not escape into gameplay. */ }
  }, delay);
  timers.add(timer);
}

/** Whether this device can give haptic feedback at all; Settings hides the toggle where it would do nothing. */
export function hapticsAvailable(): boolean {
  const nav = globalThis.navigator;
  return !!nav && /Android/i.test(nav.userAgent) && typeof nav.vibrate === 'function';
}

const vibration: Record<Pulse, number | number[]> = {
  selection: 8, soft: 10, light: 12, medium: 20, heavy: 35, rigid: 25,
  success: [12, 25, 18], warning: [20, 30, 20], error: [30, 25, 30],
};

function play(pattern: Pattern, token: number, current: () => boolean): void {
  if (!hapticsAvailable()) return;
  const pulse = (kind: Pulse) => {
    try {
      if (token !== generation || !useHaptics.getState().enabled || !current()) return;
      navigator.vibrate(vibration[kind]);
    } catch { /* Feedback must never affect gameplay. */ }
  };
  if (token !== generation || !useHaptics.getState().enabled || !current()) return;
  for (const [delay, kind] of pattern) {
    if (delay) later(() => pulse(kind), delay);
    else pulse(kind);
  }
}

/** Each feel call is a haptic and a sound; either may be off, and the debounce covers both. */
const feedbackWanted = () => useHaptics.getState().enabled || soundAudible();

function emit(name: FeelName, isReady?: boolean, current?: () => boolean, soundHook?: CelebrationSound): void {
  try {
    if (!feedbackWanted() || (current && !current())) return;
    const at = performance.now();
    if (at < blockedUntil) return;
    const pattern: Pattern = name === 'ready' && isReady === false ? [[0, 'light']] : patterns[name];
    blockedUntil = at + Math.max(DEBOUNCE_MS, pattern.at(-1)![0]);
    lastWasTurn = !!current;
    // Log accepted semantic calls even on simulators/desktop, never claim hardware delivery.
    if (import.meta.env.DEV && typeof window !== 'undefined') {
      const log = window.__feelLog ??= [];
      log.push({ name, at, ...(soundHook ? { soundHook } : {}), ...(name === 'ready' ? { ready: isReady !== false } : {}) });
      if (log.length > 500) log.splice(0, log.length - 500);
    }
    playSound(soundHook && SOUND_FILES[soundHook] ? soundHook : name === 'ready' && isReady === false ? 'unready' : name);
    if (useHaptics.getState().enabled) play(pattern, generation, current ?? (() => true));
  } catch { /* Even diagnostics are best-effort. */ }
}

export const feel = Object.fromEntries(Object.keys(patterns).map((name) =>
  [name, (hook?: CelebrationSound) => emit(name as FeelName, undefined, undefined, hook)],
)) as Record<Exclude<FeelName, 'ready'>, (soundHook?: CelebrationSound) => void> & { ready(isReady?: boolean): void };
feel.ready = (isReady: boolean = true) => emit('ready', isReady);

/** Start after Ready's acknowledgement; replace old turns and cancel on viewer/session changes. */
export function playTurnFeedback(calls: readonly FeelName[], stillCurrent: () => boolean = () => true, hooks: Partial<Record<FeelName, CelebrationSound>> = {}): void {
  cancelTurnFeedback();
  if (!feedbackWanted()) return;
  const token = turnGeneration;
  const celebrationToken = celebrationGeneration;
  const current = () => token === turnGeneration && stillCurrent();
  const sequence = calls.slice(0, 4);
  const next = (index: number) => {
    if (!current() || index >= sequence.length) return;
    const name = sequence[index];
    const accepted = () => current() && (!(hooks[name] || name === 'discovery') || celebrationToken === celebrationGeneration);
    if (accepted()) {
      // A slow first map render can make all pre-scheduled timers runnable together.
      // Space from actual delivery, and defer a beat if a player's tap owns the debounce.
      const delay = blockedUntil - performance.now();
      if (delay > 0) { later(() => next(index), delay); return; }
      emit(name, undefined, accepted, hooks[name]);
    }
    later(() => next(index + 1), 120);
  };
  later(() => next(0), 120);
}
