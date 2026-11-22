// Sound for the feel.* API. Which sound plays for which event lives in app/public/audio/manifest.json
// (with its source, license and rationale); the events themselves are the haptics contract in
// docs/design/haptics.md. haptics.ts calls playSound() after its
// shared debounce, so a sound is never heard without the matching feel call being accepted.
//
// One-shots: Web Audio buffers decoded once, started with no scheduling delay (latency is the
// context's base + output latency, typically 5–25 ms). Music: streamed media elements routed
// through Web Audio gain, so a 7-minute track is never decoded whole into memory.
// Nothing starts before the first user gesture (browser autoplay rules).
import manifest from '../../public/audio/manifest.json';
import type { FeelName } from './haptics';
import type { PlayerView } from '../types';
import { useSound, type SoundPrefs } from './soundPreferences';

export type CelebrationSound = 'sting.minor' | 'sting.notable' | 'sting.major' | 'fanfare.legendary';
export type SoundName = FeelName | 'unready' | CelebrationSound;
export interface SoundManifestEntry {
  file: string; events: string[]; title: string; author: string; source: string; license: string;
  attribution: string; modified: string; rationale: string; retrieved?: string; terms?: string;
}
export const SOUND_MANIFEST = manifest as SoundManifestEntry[];

const byEvent = new Map(SOUND_MANIFEST.flatMap((e) => e.events.map((ev) => [ev, e.file] as const)));
/** Event -> file under public/audio. Missing events are caught by sound.test.ts. */
export const SOUND_FILES = Object.fromEntries([...byEvent].filter(([ev]) => !ev.startsWith('music:'))) as Partial<Record<SoundName, string>>;
export const MUSIC_FILES = { atlas: byEvent.get('music:atlas')!, war: byEvent.get('music:war')! };
type MusicKey = keyof typeof MUSIC_FILES;

/** Only a War involving the viewer's team and another empire calls for the war track. */
export function musicForView(view: Pick<PlayerView, 'team' | 'players' | 'diplomacy'> | null): MusicKey {
  if (!view) return 'atlas';
  // Barbarians (player 255) have units but are not an empire or a diplomacy participant.
  const rivals = view.players.filter((p) => p.team !== view.team && p.id !== 255 && p.alive);
  if (view.diplomacy?.legacy) return rivals.length ? 'war' : 'atlas';
  return view.diplomacy?.relations.some((r) => r.state === 'War' &&
    (r.a === view.team || r.b === view.team) && rivals.some((p) => p.team === (r.a === view.team ? r.b : r.a)))
    ? 'war' : 'atlas';
}

export const MAX_VOICES = 4;
export const MUSIC_FADE_S = 3;
const STEAL_FADE_S = 0.015;
const LEAD_SCAN_S = 0.06;

/** Slider position -> amplitude. Squaring approximates perceived loudness, so 50% sounds half as loud. */
export function perceptual(v: number): number {
  const x = Math.min(1, Math.max(0, Number.isFinite(v) ? v : 0));
  return x * x;
}

/** Final gain for a bus: master mute and the bus toggle win, otherwise master × bus (both perceptual). */
export function busGain(p: SoundPrefs, bus: 'sfx' | 'music'): number {
  if (p.muted || !(bus === 'sfx' ? p.sfxOn : p.musicOn)) return 0;
  return perceptual(p.master) * perceptual(bus === 'sfx' ? p.sfx : p.music);
}

/** Whether a feel call would be heard; haptics.ts uses it to decide if an event is worth emitting. */
export function soundAudible(): boolean {
  return busGain(useSound.getState(), 'sfx') > 0;
}

const url = (file: string) => `${import.meta.env.BASE_URL ?? './'}audio/${file}`;

/**
 * Samples of encoder delay or near-silence (< -60 dBFS) before the attack. Browsers differ in
 * whether decodeAudioData trims AAC priming, so skip it at play time to keep one-shots immediate.
 */
export function leadingSilence(buffer: Pick<AudioBuffer, 'getChannelData' | 'sampleRate' | 'length'>): number {
  const data = buffer.getChannelData(0);
  const limit = Math.min(buffer.length, Math.round(buffer.sampleRate * LEAD_SCAN_S));
  for (let i = 0; i < limit; i++) if (Math.abs(data[i]) > 1e-3) return i / buffer.sampleRate;
  return 0;
}

interface Track { el: HTMLAudioElement; gain: GainNode; pause?: ReturnType<typeof setTimeout> }
interface Voice { src: AudioBufferSourceNode; gain: GainNode }
interface Engine {
  ctx: AudioContext; sfx: GainNode; music: GainNode;
  buffers: Map<string, { buffer: AudioBuffer; offset: number }>;
  voices: Voice[]; tracks: Partial<Record<MusicKey, Track>>; active: MusicKey | null;
}

let engine: Engine | null = null;
let installed = false;
let selectedMusic: MusicKey = 'atlas';
const bytes = new Map<string, Promise<ArrayBuffer>>();

function fetchBytes(file: string): Promise<ArrayBuffer> {
  let p = bytes.get(file);
  if (!p) {
    p = fetch(url(file)).then((r) => r.ok ? r.arrayBuffer() : Promise.reject(new Error(`${r.status} ${file}`)));
    p.catch(() => bytes.delete(file)); // Retry on the next unlock/preload.
    bytes.set(file, p);
  }
  return p;
}

/** Fetch the (small) one-shot files ahead of the first gesture; decoding waits for the context. */
export function prefetchSounds(): void {
  if (typeof fetch !== 'function') return;
  for (const file of new Set(Object.values(SOUND_FILES))) fetchBytes(file).catch(() => {});
}

function decodeAll(e: Engine): Promise<unknown> {
  return Promise.all([...new Set(Object.values(SOUND_FILES))].map(async (file) => {
    try {
      // decodeAudioData detaches its input, so decode a copy and keep the cached bytes reusable.
      const buffer = await e.ctx.decodeAudioData((await fetchBytes(file)).slice(0));
      e.buffers.set(file, { buffer, offset: leadingSilence(buffer) });
    } catch { /* A missing or undecodable file just stays silent. */ }
  }));
}

function hidden(): boolean {
  return typeof document !== 'undefined' && document.visibilityState === 'hidden';
}

/** Creates the context inside a user gesture. Safe to call repeatedly; resumes a suspended context. */
export function unlockSound(): Promise<unknown> | undefined {
  const AC = globalThis.AudioContext ?? (globalThis as { webkitAudioContext?: typeof AudioContext }).webkitAudioContext;
  // Automated browsers (Playwright sets navigator.webdriver) stay silent: headless test runs
  // otherwise play the game's music through the user's speakers.
  if (!AC || (import.meta.env.MODE !== 'test' && typeof navigator !== 'undefined' && navigator.webdriver)) return;
  if (engine) {
    if (engine.ctx.state !== 'running' && !hidden()) void engine.ctx.resume().catch(() => {});
    syncMusic(true);
    return;
  }
  try {
    // iOS (WebKit Audio Session API): "ambient" obeys the ring/silent switch and mixes with other
    // apps' audio. Without it, playing a media element would ignore silent mode.
    const session = (navigator as Navigator & { audioSession?: { type: string } }).audioSession;
    if (session) session.type = 'ambient';
  } catch { /* Not supported: nothing to respect. */ }
  const ctx = new AC({ latencyHint: 'interactive' });
  const master = ctx.createGain();
  master.connect(ctx.destination);
  const sfx = ctx.createGain();
  const music = ctx.createGain();
  sfx.connect(master);
  music.connect(master);
  engine = { ctx, sfx, music, buffers: new Map(), voices: [], tracks: {}, active: null };
  applyVolumes(useSound.getState());
  void ctx.resume().catch(() => {});
  const loaded = decodeAll(engine);
  syncMusic();
  return loaded;
}

function applyVolumes(p: SoundPrefs): void {
  const e = engine;
  if (!e) return;
  const t = e.ctx.currentTime;
  e.sfx.gain.setTargetAtTime(busGain(p, 'sfx'), t, 0.02);
  e.music.gain.setTargetAtTime(busGain(p, 'music'), t, 0.05);
}

/** Plays a one-shot now. Called by haptics.ts after its debounce; never throws. */
export function playSound(name: SoundName): void {
  try {
    const e = engine;
    if (!e || e.ctx.state !== 'running' || hidden() || !soundAudible()) return;
    const file = SOUND_FILES[name];
    const loaded = file ? e.buffers.get(file) : undefined;
    if (!loaded) return;
    const now = e.ctx.currentTime;
    // Cap overlapping one-shots: steal the oldest with a short fade instead of a click.
    while (e.voices.length >= MAX_VOICES) {
      const old = e.voices.shift()!;
      old.gain.gain.setValueAtTime(old.gain.gain.value, now);
      old.gain.gain.linearRampToValueAtTime(0, now + STEAL_FADE_S);
      try { old.src.stop(now + STEAL_FADE_S); } catch { /* Already stopped. */ }
    }
    const gain = e.ctx.createGain();
    const src = e.ctx.createBufferSource();
    src.buffer = loaded.buffer;
    src.connect(gain);
    gain.connect(e.sfx);
    const voice = { src, gain };
    src.onended = () => {
      const i = e.voices.indexOf(voice);
      if (i >= 0) e.voices.splice(i, 1);
      gain.disconnect();
    };
    src.start(0, loaded.offset);
    e.voices.push(voice);
  } catch { /* Sound must never affect gameplay. */ }
}

function track(e: Engine, key: MusicKey): Track {
  let t = e.tracks[key];
  if (!t) {
    const el = new Audio(url(MUSIC_FILES[key]));
    el.loop = true;
    el.preload = 'auto';
    const gain = e.ctx.createGain();
    gain.gain.value = 0;
    e.ctx.createMediaElementSource(el).connect(gain);
    gain.connect(e.music);
    t = e.tracks[key] = { el, gain };
  }
  return t;
}

function ramp(e: Engine, t: Track, to: number): void {
  const now = e.ctx.currentTime;
  t.gain.gain.cancelScheduledValues(now);
  t.gain.gain.setValueAtTime(t.gain.gain.value, now);
  t.gain.gain.linearRampToValueAtTime(to, now + MUSIC_FADE_S);
}

/**
 * Selects the soundtrack from the committed viewer's diplomacy state. Repeated store updates
 * on one turn keep the current track running instead of restarting its loop.
 */
export function setMusicForView(view: Pick<PlayerView, 'team' | 'players' | 'diplomacy'> | null): void {
  const next = musicForView(view);
  if (next === selectedMusic) return;
  selectedMusic = next;
  syncMusic();
}

/** Fade between themes, or fade out when hidden or muted. `retry` runs inside a user gesture. */
export function syncMusic(retry = false): void {
  const e = engine;
  if (!e) return;
  try {
    const want = !hidden() && busGain(useSound.getState(), 'music') > 0;
    const nextKey = want ? selectedMusic : null;
    if (nextKey === e.active) {
      if (retry && nextKey) { const t = track(e, nextKey); if (t.el.paused) void t.el.play().catch(() => {}); }
      return;
    }
    if (e.active) {
      const out = track(e, e.active);
      ramp(e, out, 0);
      clearTimeout(out.pause);
      out.pause = setTimeout(() => out.el.pause(), MUSIC_FADE_S * 1000 + 100);
    }
    if (nextKey) {
      const next = track(e, nextKey);
      clearTimeout(next.pause);
      ramp(e, next, 1);
      if (next.el.paused) void next.el.play().catch(() => {}); // Retried on the next gesture.
    }
    e.active = nextKey;
  } catch { /* Music is best-effort. */ }
}

function onVisibility(): void {
  const e = engine;
  if (!e) return;
  if (hidden()) {
    for (const t of Object.values(e.tracks)) {
      clearTimeout(t.pause);
      t.gain.gain.cancelScheduledValues(0);
      t.gain.gain.value = 0;
      t.el.pause();
    }
    e.active = null;
    void e.ctx.suspend().catch(() => {});
  } else {
    void e.ctx.resume().catch(() => {});
    syncMusic();
  }
}

/** Wire gesture unlock, visibility and preference changes. Idempotent; call once at startup. */
export function installSound(): void {
  if (installed || typeof window === 'undefined') return;
  installed = true;
  const gesture = () => { void unlockSound(); };
  for (const type of ['pointerdown', 'keydown', 'touchend']) window.addEventListener(type, gesture, { capture: true, passive: true });
  document.addEventListener('visibilitychange', onVisibility);
  useSound.subscribe((p) => { applyVolumes(p); syncMusic(); });
  const idle = (globalThis as { requestIdleCallback?: (cb: () => void) => void }).requestIdleCallback ?? ((cb: () => void) => setTimeout(cb, 500));
  idle(prefetchSounds);
}


/** Diagnostics for the sound board and QA: the context's own latency figures, in ms. */
export function soundLatencyMs(): number | null {
  const ctx = engine?.ctx;
  if (!ctx) return null;
  return Math.round(((ctx.baseLatency ?? 0) + (ctx.outputLatency ?? 0)) * 1000);
}
