import { describe, expect, it, vi } from 'vitest';
import { CITY_ERAS, CITY_TIERS, cityArtArrivals, cityArtEpoch, cityImage, cityStem, cityTier, cityUrl, markStem, missingCityArt, spriteSize, traceCityArt, wallStem } from './cityArt';

const only = (...stems: string[]) => (stem: string, size: 64 | 128 | 512) => size === 512 && stems.includes(stem);

describe('city art', () => {
  it('sizes follow the art bands: 1-3, 4-7, 8-12, 13+', () => {
    expect([1, 3, 4, 7, 8, 12, 13, 30].map(cityTier)).toEqual(['hamlet', 'hamlet', 'town', 'town', 'city', 'city', 'metropolis', 'metropolis']);
  });

  it('every era and size has a painted sprite in the bundle, and the far marks and walls exist', () => {
    expect(missingCityArt()).toEqual([]);
    CITY_ERAS.forEach((era, index) => {
      for (const tier of CITY_TIERS) {
        expect(cityStem(index, tier)).toBe(`${era}-${tier}`);
        expect(cityUrl(`${era}-${tier}`, 128)).toBeTruthy();
      }
    });
    for (const tier of CITY_TIERS) expect(cityUrl(markStem(tier), 64)).toBeTruthy();
  });

  it('a missing sprite falls back to the nearest finished era, then the nearest size, never to nothing', () => {
    const none = only('ancient-town', 'medieval-city');
    expect(cityStem(0, 'town', none)).toBe('ancient-town');
    // Renaissance (3) has no city: Medieval (2) is the nearest era that has one.
    expect(cityStem(3, 'city', none)).toBe('medieval-city');
    // Equidistant eras: the earlier look wins.
    expect(cityStem(1, 'town', only('ancient-town', 'medieval-town'))).toBe('ancient-town');
    // No sprite of this size anywhere in this era or any other of the same size: nearest size.
    expect(cityStem(2, 'metropolis', only('medieval-city', 'ancient-hamlet'))).toBe('medieval-city');
    expect(cityStem(2, 'hamlet', only())).toBeNull();
  });

  it('out-of-range eras clamp and a legacy game without eras shows Ancient', () => {
    expect(cityStem(-3, 'hamlet')).toBe('ancient-hamlet');
    expect(cityStem(99, 'metropolis')).toBe('future-metropolis');
    expect(cityStem(Number.NaN, 'town')).toBe('ancient-town');
  });

  it('wall overlays: each era owns its art, with a star fort for the Alpine Redoubt', () => {
    expect(wallStem(2, false)).toBe('medieval-walls');
    for (const era of [5, 6, 7, 8]) expect(wallStem(era, false)).toBe(`${CITY_ERAS[era]}-walls`);
    expect(wallStem(0, true)).toBe('renaissance-walls');
    expect(wallStem(3, false, only('ancient-walls'))).toBe('ancient-walls');
    expect(wallStem(3, false, only())).toBeNull();
  });

  it('picks the 512 master only when the sprite is drawn large', () => {
    expect([40, 80, 81, 200, 201, 400].map(spriteSize)).toEqual([64, 64, 128, 128, 512, 512]);
  });

  it('traces the sprites a drawing found still decoding, and reports their arrival, so only the chunks that waited for one repaint; a sprite with no file is not a wait', () => {
    class Decoding {
      static all: Decoding[] = [];
      complete = false; naturalWidth = 0; onload: (() => void) | null = null; src = '';
      constructor() { Decoding.all.push(this); }
    }
    vi.stubGlobal('Image', Decoding);
    try {
      const before = cityArtEpoch();
      const waiting = traceCityArt(() => {
        expect(cityImage('ancient-hamlet', 128)).toBeNull(); // starts the load
        expect(cityImage('ancient-hamlet', 128)).toBeNull(); // still decoding
        expect(cityImage('no-such-town', 128)).toBeNull();
      });
      expect([...waiting]).toEqual(['ancient-hamlet-128']);
      expect([...traceCityArt(() => { cityImage('ancient-town', 128); })]).toEqual(['ancient-town-128']);
      expect(cityArtArrivals(before)).toEqual([]);
      Decoding.all[0].onload?.();
      expect(cityArtEpoch()).toBe(before + 1);
      expect(cityArtArrivals(before)).toEqual(['ancient-hamlet-128']);
    } finally {
      vi.unstubAllGlobals();
    }
  });
});
