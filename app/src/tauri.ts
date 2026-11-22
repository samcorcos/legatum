// The desktop shell. Tauri v2 injects `__TAURI_INTERNALS__` into every webview (and `__TAURI__`
// when `withGlobalTauri` is on); the browser build has neither. Calling the window plugin through
// the internals keeps the web bundle free of @tauri-apps/api for the one command it needs.

interface TauriInternals {
  invoke(cmd: string, args?: Record<string, unknown>): Promise<unknown>;
  metadata?: { currentWindow?: { label?: string } };
}

function internals(): TauriInternals | null {
  const w = globalThis as { __TAURI_INTERNALS__?: TauriInternals };
  return w.__TAURI_INTERNALS__ ?? null;
}

/** Call a command of the desktop app (`app/src-tauri/src/lib.rs`). Rejects outside it. */
export function tauriInvoke(cmd: string, args?: Record<string, unknown>): Promise<unknown> {
  const t = internals();
  return t ? t.invoke(cmd, args) : Promise.reject(new Error('not in the desktop app'));
}

/**
 * The page the desktop app's own game server serves (http://127.0.0.1:<port>/). The app lets it call
 * only `online_info` and `online_host_stop`: every other command, plugin and Quit is refused there,
 * so this page uses the server-mode paths, as a browser would.
 */
export function onHostPage(): boolean {
  return globalThis.location?.hostname === '127.0.0.1';
}

// The Tauri API's own `isTauri()` reads this flag; on the host page it must say no.
if (onHostPage() && internals() !== null) {
  try {
    (globalThis as { isTauri?: boolean }).isTauri = false;
  } catch {
    // best effort
  }
}

/** Whether the Tauri bridge exists in this page (true on the host page too). */
export function hasTauriBridge(): boolean {
  return internals() !== null;
}

/** Whether the UI runs inside the Tauri desktop app and may call its commands. */
export function inTauri(): boolean {
  if (onHostPage()) return false;
  return internals() !== null || '__TAURI__' in globalThis;
}

/** iOS or Android. iPadOS reports a Mac user agent, so a Mac UA with touch points counts as iPad. */
export function isMobileDevice(userAgent: string, maxTouchPoints: number): boolean {
  return /iPhone|iPad|iPod|Android/.test(userAgent) || (/Macintosh/.test(userAgent) && maxTouchPoints > 1);
}

/**
 * Whether the app may offer Quit: desktop Tauri only. Mobile apps don't quit themselves, and
 * closing the only window on iOS leaves a black screen with the process still running.
 */
export function canQuit(): boolean {
  return inTauri() && !isMobileDevice(navigator.userAgent, navigator.maxTouchPoints ?? 0);
}

/** Close this window; closing the app's only window quits it. Needs `core:window:allow-close`. */
export async function quitApp(): Promise<void> {
  const t = internals();
  if (!t) return;
  await t.invoke('plugin:window|close', { label: t.metadata?.currentWindow?.label ?? 'main' });
}
