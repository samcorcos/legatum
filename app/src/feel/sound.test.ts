// @vitest-environment happy-dom
import fs from 'node:fs';
import path from 'node:path';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import type { PlayerView } from '../types';

const AUDIO = path.resolve(__dirname, '../../public/audio');
const CONTRACT = path.resolve(__dirname, '../../../docs/design/haptics.md');

class FakeParam {
  value = 1;
  setTargetAtTime = vi.fn((v: number) => { this.value = v; });
  setValueAtTime = vi.fn();
  linearRampToValueAtTime = vi.fn((v: number) => { this.value = v; });
  cancelScheduledValues = vi.fn();
}
class FakeNode { connect = vi.fn((n: unknown) => n); disconnect = vi.fn(); }
class FakeGain extends FakeNode { gain = new FakeParam(); }
class FakeSource extends FakeNode {
  buffer: { file: string } | null = null;
  onended: (() => void) | null = null;
  start = vi.fn();
  stop = vi.fn();
}
class FakeAudio {
  static made: FakeAudio[] = [];
  loop = false; preload = ''; paused = true;
  constructor(public src: string) { FakeAudio.made.push(this); }
  play = vi.fn(async () => { this.paused = false; });
  pause = vi.fn(() => { this.paused = true; });
}
let sources: FakeSource[];
let contexts: FakeContext[];
class FakeContext {
  state = 'suspended'; currentTime = 0; destination = new FakeNode(); baseLatency = 0.005; outputLatency = 0.01;
  constructor(public options: unknown) { contexts.push(this); }
  gains: FakeGain[] = [];
  createGain() { const g = new FakeGain(); this.gains.push(g); return g; }
  createBufferSource() { const s = new FakeSource(); sources.push(s); return s; }
  createMediaElementSource() { return new FakeNode(); }
  resume = vi.fn(async () => { this.state = 'running'; });
  suspend = vi.fn(async () => { this.state = 'suspended'; });
  // 1 ms of silence (44 samples at 44.1 kHz), then signal: stands in for AAC encoder priming.
  decodeAudioData = vi.fn(async (bytes: ArrayBuffer) => {
    const data = new Float32Array(441).fill(0.5, 44);
    return { file: new TextDecoder().decode(bytes), sampleRate: 44100, length: 441, getChannelData: () => data };
  });
}

let sound: typeof import('./sound');
let haptics: typeof import('./haptics');
let hapticPrefs: typeof import('./preferences');
let soundPrefs: typeof import('./soundPreferences');

const played = () => sources.filter((s) => s.start.mock.calls.length).map((s) => s.buffer!.file);

beforeEach(async () => {
  vi.resetModules();
  vi.useFakeTimers();
  localStorage.clear();
  sources = [];
  contexts = [];
  FakeAudio.made = [];
  vi.stubGlobal('AudioContext', FakeContext);
  vi.stubGlobal('Audio', FakeAudio);
  vi.stubGlobal('fetch', vi.fn(async (url: string) => ({
    ok: true, arrayBuffer: async () => new TextEncoder().encode(url.slice(url.indexOf('audio/') + 6)).buffer,
  })));
  sound = await import('./sound');
  haptics = await import('./haptics');
  hapticPrefs = await import('./preferences');
  soundPrefs = await import('./soundPreferences');
});
afterEach(() => {
  haptics.cancelFeedback();
  vi.useRealTimers();
  vi.unstubAllGlobals();
});

describe('event -> sound mapping', () => {
  it('gives every feel event in the haptics contract (and unready) a shipped sound', () => {
    const contract = [...fs.readFileSync(CONTRACT, 'utf8').matchAll(/^\| `(\w+)` \|/gm)].map((m) => m[1]);
    expect(contract.length).toBeGreaterThan(20);
    const names = [...new Set([...contract, ...Object.keys(haptics.feel), 'unready', 'sting.minor', 'sting.notable', 'sting.major', 'fanfare.legendary'])];
    for (const name of names) {
      const file = sound.SOUND_FILES[name as keyof typeof sound.SOUND_FILES];
      expect(file, name).toMatch(/\.m4a$/);
      expect(fs.existsSync(path.join(AUDIO, file!)), file).toBe(true);
    }
    // Each file serves one event, and nothing in the manifest is unused.
    expect(new Set(Object.values(sound.SOUND_FILES)).size).toBe(Object.keys(sound.SOUND_FILES).length);
    expect(Object.keys(sound.SOUND_FILES).sort()).toEqual(names.sort());
  });

  it('has both Eclipse Legion music tracks', () => {
    for (const file of Object.values(sound.MUSIC_FILES)) expect(fs.existsSync(path.join(AUDIO, file!)), file).toBe(true);
    expect(sound.MUSIC_FILES).toEqual({ atlas: 'music/atlas.m4a', war: 'music/war.m4a' });
  });
});

describe('volume math', () => {
  const d = () => ({ ...soundPrefsDefaults() });
  const soundPrefsDefaults = () => soundPrefs.SOUND_DEFAULTS;

  it('maps slider positions to perceptual gain and clamps', () => {
    expect(sound.perceptual(0)).toBe(0);
    expect(sound.perceptual(0.5)).toBe(0.25);
    expect(sound.perceptual(1)).toBe(1);
    expect(sound.perceptual(2)).toBe(1);
    expect(sound.perceptual(-1)).toBe(0);
    expect(sound.perceptual(Number.NaN)).toBe(0);
  });

  it('multiplies master by bus, and mutes win', () => {
    expect(sound.busGain(d(), 'sfx')).toBeCloseTo(0.8 ** 2 * 0.7 ** 2);
    expect(sound.busGain(d(), 'music')).toBeCloseTo(0.8 ** 2 * 0.4 ** 2);
    // Defaults: effects moderate, music on and clearly quieter than effects.
    expect(sound.busGain(d(), 'music')).toBeLessThan(sound.busGain(d(), 'sfx') / 2);
    expect(sound.busGain({ ...d(), muted: true }, 'sfx')).toBe(0);
    expect(sound.busGain({ ...d(), muted: true }, 'music')).toBe(0);
    expect(sound.busGain({ ...d(), sfxOn: false }, 'sfx')).toBe(0);
    expect(sound.busGain({ ...d(), sfxOn: false }, 'music')).toBeGreaterThan(0);
    expect(sound.busGain({ ...d(), master: 0 }, 'sfx')).toBe(0);
  });

  it('persists preferences and ignores corrupt storage', () => {
    soundPrefs.useSound.getState().update({ sfx: 0.3, musicOn: false });
    expect(JSON.parse(localStorage.getItem(soundPrefs.SOUND_KEY)!)).toMatchObject({ sfx: 0.3, musicOn: false });
    localStorage.setItem(soundPrefs.SOUND_KEY, '{"sfx": 7, "music": "loud", "muted": 1}');
    expect(soundPrefs.readSoundPrefs()).toEqual({ ...soundPrefs.SOUND_DEFAULTS, sfx: 1 });
    localStorage.setItem(soundPrefs.SOUND_KEY, 'not json');
    expect(soundPrefs.readSoundPrefs()).toEqual(soundPrefs.SOUND_DEFAULTS);
  });

  it('applies bus gains to the graph when preferences change', async () => {
    sound.installSound();
    await sound.unlockSound();
    const [, sfx, music] = contexts[0].gains; // master, sfx, music
    expect(sfx.gain.value).toBeCloseTo(0.8 ** 2 * 0.7 ** 2);
    expect(music.gain.value).toBeCloseTo(0.8 ** 2 * 0.4 ** 2);
    soundPrefs.useSound.getState().update({ sfx: 0.5 });
    expect(sfx.gain.value).toBeCloseTo(0.64 * 0.25);
    soundPrefs.useSound.getState().update({ muted: true });
    expect(sfx.gain.value).toBe(0);
    expect(music.gain.value).toBe(0);
  });
});

describe('playback', () => {
  it('stays silent until a user gesture creates the context', async () => {
    sound.installSound();
    haptics.feel.select();
    expect(contexts).toHaveLength(0);
    window.dispatchEvent(new Event('pointerdown'));
    expect(contexts).toHaveLength(1);
    expect(contexts[0].options).toEqual({ latencyHint: 'interactive' });
    await vi.advanceTimersByTimeAsync(0);
    await vi.advanceTimersByTimeAsync(100);
    haptics.feel.select();
    expect(played()).toEqual(['ui/select.m4a']);
  });

  it('starts one-shots immediately, past the leading silence', async () => {
    await sound.unlockSound();
    haptics.feel.commit();
    const [src] = sources;
    expect(src.start).toHaveBeenCalledWith(0, 44 / 44100);
    expect(sound.soundLatencyMs()).toBe(15);
  });

  it('shares the haptics debounce, then accepts a separate action', async () => {
    await sound.unlockSound();
    haptics.feel.select();
    haptics.feel.commit();
    haptics.feel.select();
    expect(played()).toEqual(['ui/select.m4a']);
    vi.advanceTimersByTime(80);
    haptics.feel.commit();
    expect(played()).toEqual(['ui/select.m4a', 'orders/commit.m4a']);
  });

  it('plays with haptics off, and haptics without sound when muted', async () => {
    await sound.unlockSound();
    hapticPrefs.useHaptics.getState().setEnabled(false);
    haptics.feel.cityFounded();
    expect(played()).toEqual(['stings/city-founded.m4a']);
    vi.advanceTimersByTime(100);
    soundPrefs.useSound.getState().update({ muted: true });
    haptics.feel.cityLost();
    expect(played()).toHaveLength(1);
  });

  it('plays unready for feel.ready(false) and a sound per turn beat', async () => {
    await sound.unlockSound();
    haptics.feel.ready(false);
    vi.advanceTimersByTime(100);
    haptics.playTurnFeedback(['turnResolved', 'techComplete']);
    await vi.advanceTimersByTimeAsync(300);
    expect(played()).toEqual(['orders/unready.m4a', 'turn/turn-resolved.m4a', 'stings/tech-complete.m4a']);
  });

  it('caps overlapping one-shots, fading out the oldest', async () => {
    await sound.unlockSound();
    const names = ['select', 'commit', 'purchase', 'battleWon', 'battleLost'] as const;
    for (const name of names) { haptics.feel[name](); vi.advanceTimersByTime(100); }
    expect(played()).toHaveLength(5);
    expect(sources[0].stop).toHaveBeenCalledTimes(1);
    expect(sources.slice(1).every((s) => s.stop.mock.calls.length === 0)).toBe(true);
    expect(sound.MAX_VOICES).toBe(4);
  });

  it('never plays while the page is hidden', async () => {
    await sound.unlockSound();
    Object.defineProperty(document, 'visibilityState', { value: 'hidden', configurable: true });
    haptics.feel.select();
    expect(played()).toHaveLength(0);
    Object.defineProperty(document, 'visibilityState', { value: 'visible', configurable: true });
  });
});

describe('music', () => {
  const view = (relations: { a: number; b: number; state: 'Peace' | 'War' }[], barbarians = false) => ({
    team: 0,
    players: [{ id: 0, team: 0, alive: true }, { id: 1, team: 1, alive: true }, { id: 2, team: 2, alive: true }],
    diplomacy: { legacy: false, relations: barbarians ? [...relations, { a: 0, b: 255, state: 'War' }] : relations },
  }) as unknown as Pick<PlayerView, 'team' | 'players' | 'diplomacy'>;

  it('crossfades peace → war → peace without restarting on repeated views', async () => {
    sound.installSound();
    sound.setMusicForView(view([{ a: 0, b: 1, state: 'Peace' }]));
    await sound.unlockSound();
    const [atlas] = FakeAudio.made;
    expect(atlas.play).toHaveBeenCalledTimes(1);

    const war = view([{ a: 0, b: 1, state: 'War' }]);
    sound.setMusicForView(war);
    const [, battle] = FakeAudio.made;
    expect(battle.src).toMatch(/audio\/music\/war\.m4a$/);
    expect(battle.loop).toBe(true);
    expect(battle.play).toHaveBeenCalledTimes(1);
    expect(atlas.paused).toBe(false); // Both play during the crossfade.
    sound.setMusicForView(war);
    sound.syncMusic();
    expect(battle.play).toHaveBeenCalledTimes(1);
    vi.advanceTimersByTime(sound.MUSIC_FADE_S * 1000 + 200);
    expect(atlas.paused).toBe(true);

    sound.setMusicForView(view([{ a: 0, b: 1, state: 'Peace' }]));
    expect(atlas.play).toHaveBeenCalledTimes(2);
    expect(battle.paused).toBe(false);
    vi.advanceTimersByTime(sound.MUSIC_FADE_S * 1000 + 200);
    expect(battle.paused).toBe(true);
    sound.setMusicForView(null); // Returning to the title keeps Atlas running.
    expect(atlas.play).toHaveBeenCalledTimes(2);
  });

  it('does not treat barbarians or a war between other empires as the viewer’s war', () => {
    expect(sound.musicForView(view([{ a: 0, b: 1, state: 'Peace' }], true))).toBe('atlas');
    expect(sound.musicForView(view([{ a: 1, b: 2, state: 'War' }]))).toBe('atlas');
    expect(sound.musicForView(view([{ a: 1, b: 0, state: 'War' }]))).toBe('war');
    expect(sound.musicForView(view([{ a: 0, b: 1, state: 'Peace' }, { a: 0, b: 2, state: 'War' }]))).toBe('war');
  });

  it('keeps music silent while muted and starts the selected theme when unmuted', async () => {
    sound.installSound();
    soundPrefs.useSound.getState().update({ muted: true });
    await sound.unlockSound();
    expect(FakeAudio.made).toHaveLength(0);
    sound.setMusicForView(view([{ a: 0, b: 1, state: 'War' }]));
    expect(FakeAudio.made).toHaveLength(0);
    soundPrefs.useSound.getState().update({ muted: false });
    expect(FakeAudio.made).toHaveLength(1);
    expect(FakeAudio.made[0].src).toMatch(/audio\/music\/war\.m4a$/);
    soundPrefs.useSound.getState().update({ muted: true });
    vi.advanceTimersByTime(sound.MUSIC_FADE_S * 1000 + 200);
    expect(FakeAudio.made[0].paused).toBe(true);
  });

  it('plays the Atlas track and pauses when hidden or muted', async () => {
    sound.installSound();
    await sound.unlockSound();
    const [atlas] = FakeAudio.made;
    expect(atlas.src).toMatch(/audio\/music\/atlas\.m4a$/);
    expect(atlas.loop).toBe(true);
    expect(atlas.play).toHaveBeenCalled();

    Object.defineProperty(document, 'visibilityState', { value: 'hidden', configurable: true });
    document.dispatchEvent(new Event('visibilitychange'));
    expect(atlas.paused).toBe(true);
    expect(contexts[0].suspend).toHaveBeenCalled();
    Object.defineProperty(document, 'visibilityState', { value: 'visible', configurable: true });
    document.dispatchEvent(new Event('visibilitychange'));
    expect(atlas.paused).toBe(false);

    soundPrefs.useSound.getState().update({ musicOn: false });
    vi.advanceTimersByTime(sound.MUSIC_FADE_S * 1000 + 200);
    expect(atlas.paused).toBe(true);
  });
});

it.each(['sting.minor', 'sting.notable', 'sting.major', 'fanfare.legendary'] as const)('plays exactly one tier sound for a discovery: %s', async (hook) => {
  await sound.unlockSound();
  haptics.feel.discovery(hook);
  expect(played()).toEqual([sound.SOUND_FILES[hook]]);
});
