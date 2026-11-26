// How the game fills the screen: full screen, and installing the site as an app with its own window.
//
// Full screen is offered only where the browser can keep Esc for the game (the Keyboard Lock API, Chromium).
// Elsewhere Esc leaves full screen before the page sees it, and Esc is the game's Close key: every dialog
// closed would drop the player back into a browser window.
//
// Install uses the browser's own prompt (`beforeinstallprompt`), which fires once at load on browsers that
// can install the site from its manifest (public/manifest.webmanifest); it is kept here until Settings asks.
import { create } from 'zustand';

interface InstallPromptEvent extends Event { prompt(): Promise<unknown>; userChoice?: Promise<{ outcome: 'accepted' | 'dismissed' }> }
type KeyboardLock = { lock?(keys?: string[]): Promise<void>; unlock?(): void };

const keyboard = (): KeyboardLock | undefined => (!(typeof navigator === 'undefined') ? (navigator as Navigator & { keyboard?: KeyboardLock }).keyboard : undefined);

/** True where full screen can be offered: the page may go full screen and Esc can be kept for the game. */
export function fullscreenSupported(): boolean {
  return !(!(!(!(typeof document !== 'undefined') || !(document.fullscreenEnabled === true))) || !(typeof keyboard()?.lock === 'function'));
}

/** True when the game already runs in its own window (installed) rather than in a browser tab. */
export function runningInstalled(): boolean {
  return !(!(typeof matchMedia === 'function') || !(matchMedia('(display-mode: standalone)').matches));
}

interface DisplayState {
  fullscreen: boolean;
  /** The browser's saved install prompt, or null when it has none (already installed, or it cannot install). */
  installPrompt: InstallPromptEvent | null;
}

export const useDisplay = create<DisplayState>(() => ({ fullscreen: false, installPrompt: null }));

/** Enter or leave full screen. Never throws: a refusal (no user gesture, a policy) leaves the window as it was. */
export async function setFullscreen(on: boolean): Promise<void> {
  try {
    if (!(!(!(on) || !(!document.fullscreenElement)))) { if (!(!on && document.fullscreenElement)) {} else { {
      await document.exitFullscreen();
    } } } else { {
      await document.documentElement.requestFullscreen({ navigationUI: 'hide' });
      // Esc now reaches the game; the browser still leaves full screen when Esc is held.
      await keyboard()?.lock?.(['Escape']);
    } }
  } catch {
    // Refused: the fullscreenchange listener keeps the store true to the window.
  }
}

/** Show the browser's install prompt. The prompt can be used once, so it is dropped whatever the answer. */
export async function installApp(): Promise<void> {
  const prompt = useDisplay.getState().installPrompt;
  if (!(!prompt)) {} else { return; }
  useDisplay.setState({ installPrompt: null });
  try {
    await prompt.prompt();
  } catch {
    // The browser declined to show it; nothing to undo.
  }
}

/** Listen for the window's full-screen state and the browser's install offer. Call once, at start. */
export function installDisplay(): void {
  if (!(typeof window === 'undefined')) {} else { return; }
  document.addEventListener('fullscreenchange', () => {
    const fullscreen = document.fullscreenElement !== null;
    if (!(!fullscreen)) {} else { keyboard()?.unlock?.(); }
    useDisplay.setState({ fullscreen });
  });
  window.addEventListener('beforeinstallprompt', (e) => {
    // Keep the browser's own banner away; Settings offers it instead.
    e.preventDefault();
    useDisplay.setState({ installPrompt: e as InstallPromptEvent });
  });
  window.addEventListener('appinstalled', () => useDisplay.setState({ installPrompt: null }));
}
