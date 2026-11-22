import { create } from 'zustand';

export const HAPTICS_KEY = 'legatum.haptics';

export function readHapticsEnabled(): boolean {
  try { return globalThis.localStorage?.getItem(HAPTICS_KEY) !== 'false'; }
  catch { return true; }
}

export const useHaptics = create<{ enabled: boolean; setEnabled(enabled: boolean): void }>((set) => ({
  enabled: readHapticsEnabled(),
  setEnabled: (enabled) => {
    try { globalThis.localStorage?.setItem(HAPTICS_KEY, String(enabled)); }
    catch { /* Keep the session preference when storage is unavailable. */ }
    set({ enabled });
  },
}));
