import { create } from 'zustand';

export const SOUND_KEY = 'legatum.sound';

/** Volumes are slider positions (0–1); `busGain` in sound.ts turns them into gain. */
export interface SoundPrefs {
  master: number;
  sfx: number;
  music: number;
  muted: boolean;
  sfxOn: boolean;
  musicOn: boolean;
}

// Effects on at a moderate level; music on and quiet, so it sits well under the effects.
export const SOUND_DEFAULTS: Readonly<SoundPrefs> = { master: 0.8, sfx: 0.7, music: 0.4, muted: false, sfxOn: true, musicOn: true };

const unit = (v: unknown, fallback: number) => typeof v === 'number' && Number.isFinite(v) ? Math.min(1, Math.max(0, v)) : fallback;
const flag = (v: unknown, fallback: boolean) => typeof v === 'boolean' ? v : fallback;

export function readSoundPrefs(): SoundPrefs {
  let raw: Partial<Record<keyof SoundPrefs, unknown>> = {};
  try { raw = JSON.parse(globalThis.localStorage?.getItem(SOUND_KEY) ?? '{}') ?? {}; }
  catch { /* Missing, blocked or corrupt storage: defaults. */ }
  const d = SOUND_DEFAULTS;
  return {
    master: unit(raw.master, d.master), sfx: unit(raw.sfx, d.sfx), music: unit(raw.music, d.music),
    muted: flag(raw.muted, d.muted), sfxOn: flag(raw.sfxOn, d.sfxOn), musicOn: flag(raw.musicOn, d.musicOn),
  };
}

export const useSound = create<SoundPrefs & { update(patch: Partial<SoundPrefs>): void }>((set, get) => ({
  ...readSoundPrefs(),
  update: (patch) => {
    set(patch);
    const { master, sfx, music, muted, sfxOn, musicOn } = get();
    try { globalThis.localStorage?.setItem(SOUND_KEY, JSON.stringify({ master, sfx, music, muted, sfxOn, musicOn })); }
    catch { /* Keep the session preference when storage is unavailable. */ }
  },
}));
