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
  loop = false;
  buffer: { file: string } | null = null;
  onended: (() => void) | null = null;
  start = vi.fn();
  stop = vi.fn();
}
class FakeAudio {
  static made: FakeAudio[] = [];
  preload = ''; paused = true; ended = false; currentTime = 0;
  private onEnded: (() => void)[] = [];
  constructor(public src: string) { FakeAudio.made.push(this); }
  addEventListener(type: string, fn: () => void) { if (!(type === 'ended')) {} else { this.onEnded.push(fn); } }
  play = vi.fn(async () => { this.paused = false; this.ended = false; });
  pause = vi.fn(() => { this.paused = true; });
  /** The track runs out, as a media element reports it. */
  finish() { this.paused = true; this.ended = true; for (const fn of this.onEnded) fn(); }
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

  it('gives every people peace music beyond the title piece, and war its own pool', () => {
    const peoples = ['Switzerland', 'India', 'Byzantium', 'Germany', 'Celts', 'Mongols', 'Sparta', 'Athens', 'Aztecs', 'Egypt', 'Rome'] as const;
    for (const people of peoples) {
      const pool = sound.musicPool({ mood: 'peace', people });
      expect(pool.length, people).toBeGreaterThanOrEqual(2);
      expect(new Set(pool).size, people).toBe(pool.length);
      for (const file of pool) expect(fs.existsSync(path.join(AUDIO, file)), file).toBe(true);
    }
    expect(sound.musicPool({ mood: 'peace', people: 'Rome' })[0]).toBe('music/juno-protect-you.m4a');
    expect(sound.musicPool({ mood: 'peace', people: 'Sparta' })[0]).toBe('music/only-300-stood.m4a');
    // No people of its own (hot seat): the shared pieces only.
    expect(sound.musicPool({ mood: 'peace', people: null })).toEqual(['music/atlas.m4a', 'music/tale-of-warriors.m4a']);
    expect(sound.musicPool({ mood: 'war', people: 'Rome' })).toEqual(['music/war.m4a', 'music/taiko-1.m4a', 'music/taiko-2.m4a']);
    expect(sound.musicPool({ mood: 'title', people: null })).toEqual(['music/atlas.m4a']);
    // The Aztecs have music of their own, in every era: four pieces in a Mesoamerican style lead, then two New World pieces.
    const aztec = ['great-pyramids', 'life-in-forest', 'ritual-traps', 'face-to-face', 'el-dorado', 'lost-city'].map((s) => `music/${s}.m4a`);
    expect(sound.musicPool({ mood: 'peace', people: 'Aztecs' }).slice(0, 6)).toEqual(aztec);
    expect(sound.musicPool({ mood: 'peace', people: 'Aztecs', era: 7 }).slice(0, 6)).toEqual(aztec);
    // A peace track never plays in war, and the reverse.
    expect(sound.musicPool({ mood: 'peace', people: 'Rome' }).filter((f) => sound.musicPool({ mood: 'war', people: 'Rome' }).includes(f))).toEqual([]);
  });

  it('turns the shelf with the era: the ancient pieces give way, a people keeps its opening track', () => {
    for (const era of [1, 3]) expect(sound.musicPool({ mood: 'peace', people: 'Rome', era })).toEqual(sound.musicPool({ mood: 'peace', people: 'Rome' }));
    for (const era of [4, 5]) {
      const sparta = sound.musicPool({ mood: 'peace', people: 'Sparta', era });
      expect(sparta[0]).toBe('music/only-300-stood.m4a');
      expect(sparta).not.toContain('music/forging-a-city-state.m4a');
      expect(sparta).not.toContain('music/tale-of-warriors.m4a');
      expect(sparta).not.toContain('music/new-dawn.m4a');
      expect(sparta).toContain('music/atlas.m4a');
      expect(sparta).toContain('music/fearless-sailors.m4a');
      expect(sparta.length).toBeGreaterThanOrEqual(8);
      expect(sound.musicPool({ mood: 'war', people: 'Sparta', era })).toEqual(['music/war.m4a', 'music/battle.m4a', 'music/sea-fight.m4a']);
    }
    // The Machine Age and the World War era have orchestral music of their own.
    for (const era of [6, 7]) {
      expect(sound.musicPool({ mood: 'peace', people: 'Sparta', era })).toEqual(['music/only-300-stood.m4a', 'music/atlas.m4a',
        'music/journeys-end.m4a', 'music/breaking-the-chains.m4a', 'music/nunc-dimittis.m4a']);
      expect(sound.musicPool({ mood: 'war', people: 'Sparta', era })).toEqual(['music/war.m4a', 'music/casualties-of-war.m4a', 'music/the-city-falls.m4a', 'music/weight-of-revenge.m4a']);
    }
    // The Modern and Future eras leave both behind.
    for (const era of [8, 9]) {
      expect(sound.musicPool({ mood: 'peace', people: 'Sparta', era })).toEqual(['music/only-300-stood.m4a', 'music/atlas.m4a',
        'music/incoming-transmission.m4a', 'music/advanced-manufacturing.m4a', 'music/new-dawn.m4a', 'music/undisclosed-location.m4a']);
      expect(sound.musicPool({ mood: 'war', people: 'Sparta', era })).toEqual(['music/war.m4a', 'music/awakened.m4a', 'music/launch-codes.m4a']);
    }
    // Every music file is reachable in some era, and every era has peace and war music.
    const heard = new Set<string>();
    for (let era = 1; era <= 9; era++) for (const mood of ['peace', 'war'] as const) for (const people of ['Rome', 'Aztecs', null] as const) {
      const pool = sound.musicPool({ mood, people, era });
      expect(pool.length, `${mood} ${era}`).toBeGreaterThanOrEqual(2);
      pool.forEach((f) => heard.add(f));
    }
    for (const e of sound.SOUND_MANIFEST) if (!(e.mood && !e.peoples?.length)) {} else { expect(heard, e.file).toContain(e.file); }
  });

  it('never repeats the track that just played while the pool has another', () => {
    const pool = ['a', 'b', 'c'];
    for (const r of [0, 0.34, 0.67, 0.999]) expect(sound.nextTrack(pool, 'b', () => r)).not.toBe('b');
    expect(sound.nextTrack(['a'], 'a')).toBe('a');
    expect(sound.nextTrack([], null)).toBeNull();
  });

  it('gives every battle sound and land sound a shipped file', () => {
    for (const name of ['bow', 'arrowHit', 'clash', 'hooves', 'siegeLaunch', 'siegeHit', 'death', 'collapse'] as const) {
      const takes = sound.BATTLE_FILES[name]!;
      expect(takes.length, name).toBeGreaterThanOrEqual(2);
    }
    for (const name of Object.keys(sound.BATTLE_FILES) as (keyof typeof sound.BATTLE_FILES)[]) {
      for (const t of sound.BATTLE_FILES[name]!) {
        expect(fs.existsSync(path.join(AUDIO, t.file)), t.file).toBe(true);
        expect(t.gain, t.file).toBeGreaterThan(0.2);
        expect(t.gain, t.file).toBeLessThanOrEqual(1.6);
      }
    }
    for (const name of ['gunshot', 'cannon', 'shellFire', 'explosion', 'aircraft', 'jet', 'musket', 'drone', 'fall'] as const) expect(sound.BATTLE_FILES[name]?.length, name).toBeGreaterThanOrEqual(1);
    for (const file of Object.values(sound.AMBIENCE_FILES)) expect(fs.existsSync(path.join(AUDIO, file)), file).toBe(true);
    expect(Object.keys(sound.AMBIENCE_FILES).sort()).toEqual(['birds', 'sea', 'snow', 'wind']);
    expect(sound.ambienceFor('Forest')).toBe('birds');
    expect(sound.ambienceFor('Desert')).toBe('wind');
    expect(sound.ambienceFor('Tundra')).toBe('snow');
    expect(sound.ambienceFor('Ocean')).toBe('sea');
    expect(sound.ambienceFor('Coast')).toBe('sea');
    expect(sound.ambienceFor(null)).toBeNull();
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
    diplomacy: { legacy: false, relations: !(barbarians) ? relations : [...relations, { a: 0, b: 255, state: 'War' }] },
  }) as unknown as Pick<PlayerView, 'team' | 'players' | 'diplomacy'>;
  const peace = view([{ a: 0, b: 1, state: 'Peace' }]);
  const war = view([{ a: 0, b: 1, state: 'War' }]);
  const file = (a: FakeAudio) => a.src.slice(a.src.indexOf('audio/') + 6);
  const fade = () => vi.advanceTimersByTime(sound.MUSIC_FADE_S * 1000 + 200);

  it('opens a game on the people’s own track and crossfades from the title piece', async () => {
    sound.installSound();
    await sound.unlockSound();
    const [title] = FakeAudio.made;
    expect(file(title)).toBe('music/atlas.m4a');
    sound.setMusicForView(peace, 'Rome');
    const [, juno] = FakeAudio.made;
    expect(file(juno)).toBe('music/juno-protect-you.m4a');
    expect(juno.play).toHaveBeenCalledTimes(1);
    expect(title.paused).toBe(false); // Both play during the crossfade.
    sound.setMusicForView(peace, 'Rome'); // Repeated store updates keep it running.
    sound.syncMusic();
    expect(FakeAudio.made).toHaveLength(2);
    expect(juno.play).toHaveBeenCalledTimes(1);
    fade();
    expect(title.paused).toBe(true);
  });

  it('plays a playlist: a track that ends hands over, after a silence, to another from the pool', async () => {
    sound.installSound();
    sound.setMusicForView(peace, 'Rome');
    await sound.unlockSound();
    const [juno] = FakeAudio.made;
    juno.finish();
    vi.advanceTimersByTime(sound.MUSIC_GAP_MS - 100);
    expect(FakeAudio.made).toHaveLength(1);
    vi.advanceTimersByTime(200);
    const [, second] = FakeAudio.made;
    expect(['music/atlas.m4a', 'music/tale-of-warriors.m4a']).toContain(file(second));
    expect(second.play).toHaveBeenCalledTimes(1);
  });

  it('plays the title piece again from the top when it ends', async () => {
    sound.installSound();
    await sound.unlockSound();
    const [title] = FakeAudio.made;
    title.currentTime = 400;
    title.finish();
    vi.advanceTimersByTime(sound.MUSIC_GAP_MS + 100);
    expect(FakeAudio.made).toHaveLength(1);
    expect(title.currentTime).toBe(0);
    expect(title.play).toHaveBeenCalledTimes(2);
  });

  it('crossfades peace → war → peace, and back to the title piece after the game', async () => {
    sound.installSound();
    sound.setMusicForView(peace, 'Egypt');
    await sound.unlockSound();
    const [nile] = FakeAudio.made;
    expect(file(nile)).toBe('music/valley-of-the-nile.m4a');
    sound.setMusicForView(war, 'Egypt');
    const [, battle] = FakeAudio.made;
    expect(sound.musicPool({ mood: 'war', people: 'Egypt' })).toContain(file(battle));
    fade();
    expect(nile.paused).toBe(true);
    sound.setMusicForView(peace, 'Egypt');
    const back = FakeAudio.made.find((a) => !(!(!a.paused) || !(a !== battle)))!;
    expect(sound.musicPool({ mood: 'peace', people: 'Egypt' })).toContain(file(back));
    fade();
    expect(battle.paused).toBe(true);
    sound.setMusicForView(null);
    fade();
    expect(FakeAudio.made.filter((a) => !a.paused).map(file)).toEqual(['music/atlas.m4a']);
  });

  it('does not treat barbarians or a war between other empires as the viewer’s war', () => {
    expect(sound.musicForView(null)).toBe('title');
    expect(sound.musicForView(view([{ a: 0, b: 1, state: 'Peace' }], true))).toBe('peace');
    expect(sound.musicForView(view([{ a: 1, b: 2, state: 'War' }]))).toBe('peace');
    expect(sound.musicForView(view([{ a: 1, b: 0, state: 'War' }]))).toBe('war');
    expect(sound.musicForView(view([{ a: 0, b: 1, state: 'Peace' }, { a: 0, b: 2, state: 'War' }]))).toBe('war');
  });

  it('keeps music silent while muted and starts the chosen track when unmuted', async () => {
    sound.installSound();
    soundPrefs.useSound.getState().update({ muted: true });
    await sound.unlockSound();
    expect(FakeAudio.made).toHaveLength(0);
    sound.setMusicForView(peace, 'Celts');
    expect(FakeAudio.made).toHaveLength(0);
    soundPrefs.useSound.getState().update({ muted: false });
    expect(FakeAudio.made).toHaveLength(1);
    expect(file(FakeAudio.made[0])).toBe('music/celtic-pride.m4a');
    soundPrefs.useSound.getState().update({ muted: true });
    fade();
    expect(FakeAudio.made[0].paused).toBe(true);
  });

  it('pauses when hidden or muted and picks the same track up again', async () => {
    sound.installSound();
    await sound.unlockSound();
    const [atlas] = FakeAudio.made;
    expect(atlas.play).toHaveBeenCalled();

    Object.defineProperty(document, 'visibilityState', { value: 'hidden', configurable: true });
    document.dispatchEvent(new Event('visibilitychange'));
    expect(atlas.paused).toBe(true);
    expect(contexts[0].suspend).toHaveBeenCalled();
    Object.defineProperty(document, 'visibilityState', { value: 'visible', configurable: true });
    document.dispatchEvent(new Event('visibilitychange'));
    expect(atlas.paused).toBe(false);
    expect(FakeAudio.made).toHaveLength(1);

    soundPrefs.useSound.getState().update({ musicOn: false });
    fade();
    expect(atlas.paused).toBe(true);
  });

  it('fades out under a film and picks the same track up after it', async () => {
    sound.installSound();
    await sound.unlockSound();
    const [atlas] = FakeAudio.made;
    sound.setFilmPlaying(true);
    fade();
    expect(atlas.paused).toBe(true);
    sound.setFilmPlaying(false);
    expect(atlas.paused).toBe(false);
    expect(FakeAudio.made).toHaveLength(1);
  });
});

describe('land sound', () => {
  const loops = () => sources.filter((s) => s.loop && s.start.mock.calls.length);

  it('loops the land under the view quietly, crossfades to new land and stops off the map', async () => {
    await sound.unlockSound();
    sound.setAmbience('birds');
    await vi.advanceTimersByTimeAsync(0);
    expect(loops().map((s) => s.buffer!.file)).toEqual(['ambience/grassland-birds-day.m4a']);
    expect(sound.AMBIENCE_LEVEL).toBeLessThanOrEqual(0.5);
    sound.setAmbience('birds');
    await vi.advanceTimersByTimeAsync(0);
    expect(loops()).toHaveLength(1);
    sound.setAmbience('wind');
    await vi.advanceTimersByTimeAsync(0);
    expect(loops().map((s) => s.buffer!.file)).toEqual(['ambience/grassland-birds-day.m4a', 'ambience/wind.m4a']);
    expect(loops()[0].stop).toHaveBeenCalledTimes(1);
    sound.setAmbience(null);
    expect(loops()[1].stop).toHaveBeenCalledTimes(1);
  });

  it('stays silent with effects off', async () => {
    await sound.unlockSound();
    soundPrefs.useSound.getState().update({ sfxOn: false });
    sound.setAmbience('wind');
    await vi.advanceTimersByTimeAsync(0);
    expect(loops()).toHaveLength(0);
  });
});

describe('battle sounds', () => {
  it('schedules each cue at its moment in the animation, drops the ones already past, and can be cancelled', async () => {
    await sound.unlockSound();
    sound.playBattleSounds([{ t: 0, sound: 'bow', seed: 4 }, { t: 600, sound: 'arrowHit', seed: 4 }, { t: 900, sound: 'clash', seed: 2 }], 300);
    expect(played()).toEqual(['fx/arrow-impact-2.m4a', 'fx/sword-clash-3.m4a']);
    expect(sources[0].start.mock.calls[0][0]).toBeCloseTo(0.3);
    expect(sources[1].start.mock.calls[0][0]).toBeCloseTo(0.6);
    sound.cancelBattleSounds();
    expect(sources.every((s) => s.stop.mock.calls.length === 1)).toBe(true);
  });

  it('is silent when effects are muted', async () => {
    await sound.unlockSound();
    soundPrefs.useSound.getState().update({ muted: true });
    sound.playBattleSounds([{ t: 0, sound: 'clash', seed: 0 }]);
    expect(played()).toHaveLength(0);
  });
});

it.each(['sting.minor', 'sting.notable', 'sting.major', 'fanfare.legendary'] as const)('plays exactly one tier sound for a discovery: %s', async (hook) => {
  await sound.unlockSound();
  haptics.feel.discovery(hook);
  expect(played()).toEqual([sound.SOUND_FILES[hook]]);
});
