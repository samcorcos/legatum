// Sound for the feel.* API. Which sound plays for which event lives in app/public/audio/manifest.json
// (with its source, license and rationale); the events themselves are the haptics contract in
// docs/design/haptics.md. haptics.ts calls playSound() after its
// shared debounce, so a sound is never heard without the matching feel call being accepted.
//
// One-shots: Web Audio buffers decoded once, started with no scheduling delay (latency is the
// context's base + output latency, typically 5–25 ms). Music: streamed media elements routed
// through Web Audio gain, so a 7-minute track is never decoded whole into memory. It is a playlist:
// the viewer's people and whether they are at war pick the pool, and a track that ends hands over
// to another from the pool after a short silence. Ambience: one looping buffer for the land under
// the camera. Battle sounds: one-shots scheduled against the map's battle animation.
// Nothing starts before the first user gesture (browser autoplay rules).
import manifest from '../../public/audio/manifest.json';
import type { FeelName } from './haptics';
import type { Faction, PlayerView, Terrain } from '../types';
import { useSound, type SoundPrefs } from './soundPreferences';

export type CelebrationSound = 'sting.minor' | 'sting.notable' | 'sting.major' | 'fanfare.legendary';
export type SoundName = FeelName | 'unready' | CelebrationSound;
export interface SoundManifestEntry {
  file: string; events: string[]; title: string; author: string; source: string; license: string;
  attribution: string; modified: string; rationale: string; retrieved?: string; terms?: string;
  /** Music only: when it plays, and for whom (empty: every people). */
  mood?: 'peace' | 'war'; peoples?: string[];
  /** Music only: the eras it plays in, first and last, 1-based (absent: every era). */
  eras?: number[];
  /** Battle sounds only: playback gain that evens out their loudness. */
  gain?: number;
}
export const SOUND_MANIFEST = manifest as SoundManifestEntry[];

const byEvent = new Map(SOUND_MANIFEST.flatMap((e) => e.events.map((ev) => [ev, e.file] as const)));
const LAYERED = /^(music|ambience|fx):/;
/** Event -> file under public/audio. Missing events are caught by sound.test.ts. */
export const SOUND_FILES = Object.fromEntries([...byEvent].filter(([ev]) => !LAYERED.test(ev))) as Partial<Record<SoundName, string>>;

export type MusicMood = 'title' | 'peace' | 'war';
export interface MusicChoice { mood: MusicMood; people: Faction | null; /** The listener's era, 1-based (absent: the first). */ era?: number }
export const TITLE_MUSIC = byEvent.get('music:atlas')!;
const MUSIC = SOUND_MANIFEST.filter((e) => e.mood);

/**
 * The tracks a choice draws from, the people's own first (its opening track leads). Peace adds the
 * pieces every people shares, so no people is left with one track on repeat. The era turns the
 * shelf: the ancient-world music holds the first eras and the age-of-sail pieces take over after,
 * but a people's opening track stays in rotation all game, so it keeps its own voice.
 */
export function musicPool({ mood, people, era = 1 }: MusicChoice): string[] {
  if (mood === 'title') return [TITLE_MUSIC];
  const now = (e: SoundManifestEntry) => !e.eras || (era >= e.eras[0] && era <= e.eras[1]);
  if (mood === 'war') return MUSIC.filter((e) => e.mood === 'war' && now(e)).map((e) => e.file);
  const peace = MUSIC.filter((e) => e.mood === 'peace');
  const own = people ? peace.filter((e) => e.peoples?.includes(people)) : [];
  own.sort((a, b) => a.peoples!.indexOf(people!) - b.peoples!.indexOf(people!));
  return [...own.filter((e, i) => i === 0 || now(e)), ...peace.filter((e) => !e.peoples?.length && now(e))].map((e) => e.file);
}

/** The next track: any in the pool but the one that just played. */
export function nextTrack(pool: string[], last: string | null, random: () => number = Math.random): string | null {
  const rest = pool.length > 1 ? pool.filter((f) => f !== last) : pool;
  return rest[Math.min(rest.length - 1, Math.floor(random() * rest.length))] ?? null;
}

/** Only a War involving the viewer's team and another empire calls for the war music. */
export function musicForView(view: Pick<PlayerView, 'team' | 'players' | 'diplomacy'> | null): MusicMood {
  if (!view) return 'title';
  // Barbarians (player 255) have units but are not an empire or a diplomacy participant.
  const rivals = view.players.filter((p) => p.team !== view.team && p.id !== 255 && p.alive);
  if (view.diplomacy?.legacy) return rivals.length ? 'war' : 'peace';
  return view.diplomacy?.relations.some((r) => r.state === 'War' &&
    (r.a === view.team || r.b === view.team) && rivals.some((p) => p.team === (r.a === view.team ? r.b : r.a)))
    ? 'war' : 'peace';
}

export type Ambience = 'birds' | 'wind' | 'snow' | 'sea';
export const AMBIENCE_FILES = Object.fromEntries([...byEvent].filter(([ev]) => ev.startsWith('ambience:')).map(([ev, f]) => [ev.slice(9), f])) as Record<Ambience, string>;
/** The sound of the land or the sea under the view. Unexplored map is silent. */
export function ambienceFor(terrain: Terrain | null | undefined): Ambience | null {
  switch (terrain) {
    case 'Grassland': case 'Plains': case 'Forest': return 'birds';
    case 'Desert': case 'Hills': case 'Mountains': return 'wind';
    case 'Tundra': return 'snow';
    case 'Ocean': case 'Coast': return 'sea';
    default: return null;
  }
}

export type BattleSound = 'bow' | 'arrowHit' | 'clash' | 'hooves' | 'siegeLaunch' | 'siegeHit' | 'death' | 'collapse'
  | 'gunshot' | 'cannon' | 'shellFire' | 'explosion' | 'aircraft' | 'jet' | 'musket' | 'drone' | 'fall';
/** Battle sound -> its takes (file and gain). A cue's seed picks the take, so a volley does not machine-gun one sample. */
export const BATTLE_FILES = SOUND_MANIFEST.reduce((all, e) => {
  for (const ev of e.events) {
    const [kind, name] = ev.split(':');
    if (kind === 'fx') (all[name as BattleSound] ??= []).push({ file: e.file, gain: e.gain ?? 1 });
  }
  return all;
}, {} as Partial<Record<BattleSound, { file: string; gain: number }[]>>);

export const MAX_VOICES = 4;
export const MUSIC_FADE_S = 3;
/** Silence between two tracks of a playlist. */
export const MUSIC_GAP_MS = 4000;
export const AMBIENCE_FADE_S = 2;
/** Ambience sits far under everything else: it is felt more than heard. */
export const AMBIENCE_LEVEL = 0.5;
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
  voices: Voice[]; tracks: Map<string, Track>; active: string | null;
  battle: Voice[];
  ambience: { key: Ambience; src: AudioBufferSourceNode; gain: GainNode } | null;
  loops: Map<string, Promise<AudioBuffer | null>>;
}

let engine: Engine | null = null;
let installed = false;
let choice: MusicChoice = { mood: 'title', people: null };
/** The playlist's current track; it changes when the choice changes or the track ends. */
let chosen: string | null = TITLE_MUSIC;
let gap: ReturnType<typeof setTimeout> | undefined;
let wantedAmbience: Ambience | null = null;
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

const oneShots = () => [...new Set([...Object.values(SOUND_FILES), ...Object.values(BATTLE_FILES).flatMap((takes) => takes.map((t) => t.file))])];

/** Fetch the (small) one-shot files ahead of the first gesture; decoding waits for the context. */
export function prefetchSounds(): void {
  if (typeof fetch !== 'function') return;
  for (const file of oneShots()) fetchBytes(file).catch(() => {});
}

function decodeAll(e: Engine): Promise<unknown> {
  return Promise.all(oneShots().map(async (file) => {
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
  engine = { ctx, sfx, music, buffers: new Map(), voices: [], tracks: new Map(), active: null, battle: [], ambience: null, loops: new Map() };
  applyVolumes(useSound.getState());
  void ctx.resume().catch(() => {});
  const loaded = decodeAll(engine);
  syncMusic();
  syncAmbience();
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

function track(e: Engine, file: string): Track {
  let t = e.tracks.get(file);
  if (!t) {
    const el = new Audio(url(file));
    el.preload = 'auto';
    // A playlist, not a loop: the track that ends hands over after a short silence.
    el.addEventListener('ended', () => {
      if (chosen !== file) return;
      clearTimeout(gap);
      gap = setTimeout(() => advanceMusic(file), MUSIC_GAP_MS);
    });
    const gain = e.ctx.createGain();
    gain.gain.value = 0;
    e.ctx.createMediaElementSource(el).connect(gain);
    gain.connect(e.music);
    t = { el, gain };
    e.tracks.set(file, t);
  }
  return t;
}

function advanceMusic(ended: string): void {
  if (chosen !== ended) return;
  const e = engine;
  const next = nextTrack(musicPool(choice), ended);
  if (e && next === ended) {
    // A pool of one (the title): play it again from the top.
    const t = track(e, ended);
    t.el.currentTime = 0;
    if (e.active === ended) void t.el.play().catch(() => {});
    return;
  }
  chosen = next;
  if (e && e.active === ended) e.active = null; // It has stopped by itself: nothing to fade out.
  if (e && next) track(e, next).gain.gain.value = 1; // After silence a track starts at full level, not faded in.
  syncMusic();
}

function ramp(e: Engine, t: Track, to: number): void {
  const now = e.ctx.currentTime;
  t.gain.gain.cancelScheduledValues(now);
  t.gain.gain.setValueAtTime(t.gain.gain.value, now);
  t.gain.gain.linearRampToValueAtTime(to, now + MUSIC_FADE_S);
}

/**
 * Selects the soundtrack from the committed viewer's diplomacy state and the people the music is
 * for (null: no single people, as in hot seat). Repeated store updates on one turn keep the
 * current track running. A new people starts on its opening track; war and the return to peace
 * pick any track of the new pool.
 */
export function setMusicForView(view: Pick<PlayerView, 'team' | 'players' | 'diplomacy'> | null, people: Faction | null = null, era = 1): void {
  const next: MusicChoice = { mood: musicForView(view), people: view ? people : null, era: view ? era : 1 };
  // A new era changes what plays next, never what is playing.
  if (next.mood === choice.mood && next.people === choice.people) { choice = next; return; }
  const samePeople = next.people === choice.people && choice.mood !== 'title' && next.mood !== 'title';
  const pool = musicPool(next);
  choice = next;
  clearTimeout(gap);
  chosen = samePeople ? nextTrack(pool, chosen) : pool[0] ?? null;
  syncMusic();
}

let filmPlaying = false;
/** A cinematic owns the sound: the soundtrack fades out while it plays and back in after. */
export function setFilmPlaying(on: boolean): void {
  if (filmPlaying === on) return;
  filmPlaying = on;
  syncMusic();
}

/** Fade between tracks, or fade out when hidden, muted or under a film. `retry` runs inside a user gesture. */
export function syncMusic(retry = false): void {
  const e = engine;
  if (!e) return;
  try {
    const want = !hidden() && !filmPlaying && busGain(useSound.getState(), 'music') > 0;
    const next = want ? chosen : null;
    if (next === e.active) {
      if (retry && next) { const t = track(e, next); if (t.el.paused && !t.el.ended) void t.el.play().catch(() => {}); }
      return;
    }
    if (e.active) {
      const out = track(e, e.active);
      ramp(e, out, 0);
      clearTimeout(out.pause);
      out.pause = setTimeout(() => out.el.pause(), MUSIC_FADE_S * 1000 + 100);
    }
    if (next) {
      const t = track(e, next);
      clearTimeout(t.pause);
      if (t.el.ended) t.el.currentTime = 0;
      ramp(e, t, 1);
      if (t.el.paused) void t.el.play().catch(() => {}); // Retried on the next gesture.
    }
    e.active = next;
  } catch { /* Music is best-effort. */ }
}

/** Decoded once per land sound and kept mono: a 40 s stereo loop would hold 14 MB. */
function loop(e: Engine, file: string): Promise<AudioBuffer | null> {
  let p = e.loops.get(file);
  if (!p) {
    p = fetchBytes(file).then((b) => e.ctx.decodeAudioData(b.slice(0))).then((full) => {
      if (full.numberOfChannels < 2 || typeof e.ctx.createBuffer !== 'function') return full;
      const mono = e.ctx.createBuffer(1, full.length, full.sampleRate);
      const out = mono.getChannelData(0), l = full.getChannelData(0), r = full.getChannelData(1);
      for (let i = 0; i < out.length; i++) out[i] = (l[i] + r[i]) / 2;
      return mono;
    }).catch(() => null);
    e.loops.set(file, p);
  }
  return p;
}

/** The land under the middle of the map view; null off the map screen. Changes crossfade. */
export function setAmbience(key: Ambience | null): void {
  if (key === wantedAmbience) return;
  wantedAmbience = key;
  syncAmbience();
}

function syncAmbience(): void {
  const e = engine;
  if (!e) return;
  try {
    const want = !hidden() && soundAudible() ? wantedAmbience : null;
    if ((e.ambience?.key ?? null) === want) return;
    const now = e.ctx.currentTime;
    const old = e.ambience;
    if (old) {
      old.gain.gain.cancelScheduledValues(now);
      old.gain.gain.setValueAtTime(old.gain.gain.value, now);
      old.gain.gain.linearRampToValueAtTime(0, now + AMBIENCE_FADE_S);
      try { old.src.stop(now + AMBIENCE_FADE_S); } catch { /* Already stopped. */ }
      old.src.onended = () => old.gain.disconnect();
      e.ambience = null;
    }
    if (!want) return;
    void loop(e, AMBIENCE_FILES[want]).then((buffer) => {
      if (!buffer || engine !== e || e.ambience || wantedAmbience !== want || hidden() || !soundAudible()) return;
      const src = e.ctx.createBufferSource(), gain = e.ctx.createGain();
      src.buffer = buffer;
      src.loop = true;
      src.connect(gain);
      gain.connect(e.sfx);
      const t = e.ctx.currentTime;
      gain.gain.setValueAtTime(0, t);
      gain.gain.linearRampToValueAtTime(AMBIENCE_LEVEL, t + AMBIENCE_FADE_S);
      src.start();
      e.ambience = { key: want, src, gain };
    });
  } catch { /* Ambience is best-effort. */ }
}

export interface BattleCue { t: number; sound: BattleSound; seed: number }

/** Stops battle sounds that have not played yet (a new turn, or the page hidden). */
export function cancelBattleSounds(): void {
  const e = engine;
  if (!e) return;
  for (const v of e.battle.splice(0)) { try { v.src.stop(); } catch { /* Not started. */ } v.gain.disconnect(); }
}

/**
 * Schedules a battle's sounds against its animation: `cues` hold times in ms from the animation's
 * start, which was `elapsedMs` ago. Cues already past are dropped, never bunched up at the start.
 */
export function playBattleSounds(cues: BattleCue[], elapsedMs = 0): void {
  try {
    cancelBattleSounds();
    const e = engine;
    if (!e || e.ctx.state !== 'running' || hidden() || !soundAudible()) return;
    const now = e.ctx.currentTime;
    for (const cue of cues) {
      const takes = BATTLE_FILES[cue.sound];
      const take = takes?.[Math.abs(cue.seed) % takes.length];
      const loaded = take ? e.buffers.get(take.file) : undefined;
      const delay = (cue.t - elapsedMs) / 1000;
      if (!take || !loaded || delay < -0.05) continue;
      const gain = e.ctx.createGain(), src = e.ctx.createBufferSource();
      gain.gain.value = take.gain;
      src.buffer = loaded.buffer;
      // A sound with one take is never the same sample twice in a row: the cue's seed bends its pitch a little.
      if (takes!.length === 1) src.playbackRate.value = 0.94 + (Math.abs(cue.seed) % 7) * 0.02;
      src.connect(gain);
      gain.connect(e.sfx);
      const voice = { src, gain };
      src.onended = () => {
        const i = e.battle.indexOf(voice);
        if (i >= 0) e.battle.splice(i, 1);
        gain.disconnect();
      };
      src.start(now + Math.max(0, delay), loaded.offset);
      e.battle.push(voice);
    }
  } catch { /* Sound must never affect gameplay. */ }
}

function onVisibility(): void {
  const e = engine;
  if (!e) return;
  if (hidden()) {
    for (const t of e.tracks.values()) {
      clearTimeout(t.pause);
      t.gain.gain.cancelScheduledValues(0);
      t.gain.gain.value = 0;
      t.el.pause();
    }
    e.active = null;
    cancelBattleSounds();
    syncAmbience();
    void e.ctx.suspend().catch(() => {});
  } else {
    void e.ctx.resume().catch(() => {});
    syncMusic();
    syncAmbience();
  }
}

/** Wire gesture unlock, visibility and preference changes. Idempotent; call once at startup. */
export function installSound(): void {
  if (installed || typeof window === 'undefined') return;
  installed = true;
  const gesture = () => { void unlockSound(); };
  for (const type of ['pointerdown', 'keydown', 'touchend']) window.addEventListener(type, gesture, { capture: true, passive: true });
  document.addEventListener('visibilitychange', onVisibility);
  useSound.subscribe((p) => { applyVolumes(p); syncMusic(); syncAmbience(); });
  const idle = (globalThis as { requestIdleCallback?: (cb: () => void) => void }).requestIdleCallback ?? ((cb: () => void) => setTimeout(cb, 500));
  idle(prefetchSounds);
}


/** Diagnostics for the sound board and QA: the context's own latency figures, in ms. */
export function soundLatencyMs(): number | null {
  const ctx = engine?.ctx;
  if (!ctx) return null;
  return Math.round(((ctx.baseLatency ?? 0) + (ctx.outputLatency ?? 0)) * 1000);
}
