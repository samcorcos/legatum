const IDLE_LIMIT_MS = 3 * 60 * 1000;

/** Monotonic, focus-aware play clock. Wall-clock timestamps are added only when a turn ends. */
export class ActivePlayClock {
  private focused: boolean;
  private lastActivity: number | null = null;
  private activeSince: number | null = null;
  private savedMs = 0;

  constructor(focused: boolean) { this.focused = focused; }

  private flush(now: number) {
    if (this.focused && this.activeSince !== null) {
      const idleDeadline = (this.lastActivity ?? now) + IDLE_LIMIT_MS;
      this.savedMs += Math.max(0, Math.min(now, idleDeadline) - this.activeSince);
      this.activeSince = Math.min(now, idleDeadline);
      if (now >= idleDeadline) this.activeSince = null;
    }
  }

  input(now: number) {
    this.flush(now);
    this.lastActivity = now;
    this.activeSince = this.focused ? now : null;
  }

  setFocused(focused: boolean, now: number) {
    this.flush(now);
    this.focused = focused;
    this.activeSince = focused && this.lastActivity !== null && now - this.lastActivity < IDLE_LIMIT_MS ? now : null;
  }

  seconds(now: number) {
    this.flush(now);
    return Math.floor(this.savedMs / 1000);
  }

  reset(now: number) {
    this.savedMs = 0;
    this.activeSince = this.focused && this.lastActivity !== null && now - this.lastActivity < IDLE_LIMIT_MS ? now : null;
  }
}


const playSessionId = () => globalThis.crypto?.randomUUID?.() ?? `session-${Date.now()}-${Math.random().toString(36).slice(2)}`;

let activeSession: { clock: ActivePlayClock; sessionId: string } | null = null;

export function currentActivePlaySeconds() { return activeSession?.clock.seconds(performance.now()) ?? 0; }
export function resetActivePlayTurn() { activeSession?.clock.reset(performance.now()); }
export function currentPlaySessionId() { return activeSession?.sessionId ?? (activeSession = { clock: new ActivePlayClock(false), sessionId: playSessionId() }).sessionId; }

export function startActivePlayClock() {
  const clock = new ActivePlayClock(document.visibilityState === 'visible' && document.hasFocus());
  const sessionId = playSessionId();
  activeSession = { clock, sessionId };
  const now = () => performance.now();
  const input = () => clock.input(now());
  const blur = () => clock.setFocused(false, now());
  const focus = () => clock.setFocused(document.visibilityState === 'visible', now());
  const visibility = () => clock.setFocused(document.visibilityState === 'visible' && document.hasFocus(), now());
  for (const type of ['pointerdown', 'pointermove', 'keydown', 'touchstart', 'wheel', 'input'] as const) window.addEventListener(type, input, { passive: true });
  window.addEventListener('blur', blur);
  window.addEventListener('focus', focus);
  document.addEventListener('visibilitychange', visibility);
  return {
    sessionId,
    seconds: () => clock.seconds(now()),
    dispose() {
      for (const type of ['pointerdown', 'pointermove', 'keydown', 'touchstart', 'wheel', 'input'] as const) window.removeEventListener(type, input);
      window.removeEventListener('blur', blur);
      window.removeEventListener('focus', focus);
      document.removeEventListener('visibilitychange', visibility);
      if (activeSession?.clock === clock) activeSession = null;
    },
  };
}
