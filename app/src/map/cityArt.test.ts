import { describe, expect, it, vi } from 'vitest';
import { fileURLToPath } from 'node:url';
import { alphaOf } from '../test/pngAlpha';
import { CITY_ERAS, CITY_GROUND, CITY_TIERS, WALLED_TOWN_SCALE, cityArtArrivals, cityArtEpoch, cityDrawDrop, cityDrawScale, cityImage, cityStem, cityTier, cityUrl, markStem, missingCityArt, ringEraOf, spriteSize, traceCityArt, wallDrawScale, wallStem, wallTierOf } from './cityArt';
import cityScaleData from './cityScale.json';
import { HEX_SIZE, HEX_W, BANNER_BOTTOM } from './grid';
import { CITY_WORLD } from './terrain';

const only = (...stems: string[]) => (stem: string, size: 64 | 128 | 512) => size === 512 && stems.includes(stem);

describe('city art', () => {
  it('normalizes representative house height while footprints grow by tier', () => {
    expect(cityDrawScale(markStem('hamlet'))).toBe(1);
    const bounds = [[0.55, 0.66], [0.73, 0.84], [0.96, 1.08], [1.24, 1.36]];
    for (const era of CITY_ERAS) for (const [index, tier] of CITY_TIERS.entries()) {
      const stem = `${era}-${tier}` as keyof typeof cityScaleData.sprites;
      const { houseHeightPx, scale } = cityScaleData.sprites[stem];
      expect(cityDrawScale(stem)).toBe(scale * cityScaleData.globalScaleFactor);
      expect(houseHeightPx * cityDrawScale(stem), stem).toBeCloseTo(cityScaleData.targetHouseHeightPx * cityScaleData.globalScaleFactor, 1);
      const width = (CITY_GROUND.x1 - CITY_GROUND.x0) * CITY_WORLD / 512 * cityDrawScale(stem) / HEX_W;
      expect(width, stem).toBeGreaterThanOrEqual(bounds[index][0]);
      expect(width, stem).toBeLessThanOrEqual(bounds[index][1]);
      expect(wallDrawScale(cityDrawScale(stem)) * WALLED_TOWN_SCALE, stem).toBeCloseTo(Math.min(cityDrawScale(stem), WALLED_TOWN_SCALE));
      // A conservative high roof (master y=60) ends below the banner's lower edge.
      const roofY = cityDrawDrop(cityDrawScale(stem)) - (CITY_GROUND.middle - 60) * CITY_WORLD / 512 * cityDrawScale(stem);
      expect(roofY, stem).toBeGreaterThan(-BANNER_BOTTOM * HEX_SIZE);
    }
  });
  it('measures opaque sprite widths at map scale, including the hamlet floor', () => {
    const widths = (tier: string) => CITY_ERAS.map((era) => {
      const stem = `${era}-${tier}`;
      const png = alphaOf(fileURLToPath(new URL(`../assets/cities/${stem}.png`, import.meta.url)));
      let left = png.width, right = -1, top = png.height;
      for (let y = 0; y < png.height; y++) for (let x = 0; x < png.width; x++) {
        if (png.alpha[y * png.width + x] < 128) continue;
        left = Math.min(left, x); right = Math.max(right, x); top = Math.min(top, y);
      }
      const scale = cityDrawScale(stem);
      const roofTop = cityDrawDrop(scale) + (top - CITY_GROUND.middle) * CITY_WORLD / png.height * scale;
      expect(roofTop, `${stem} roof and banner`).toBeGreaterThan(-BANNER_BOTTOM * HEX_SIZE);
      return (right - left + 1) * CITY_WORLD / png.width * scale / HEX_W;
    }).sort((a, b) => a - b);
    expect(widths('metropolis')[4]).toBeCloseTo(1.3, 1);
    expect(widths('hamlet')[0]).toBeGreaterThanOrEqual(0.45);
  });
  it('keeps every walled tier within 1.4 hex widths, including the complete ring', () => {
    const widthOf = (path: URL) => {
      const png = alphaOf(fileURLToPath(path));
      let left = png.width, right = -1;
      for (let y = 0; y < png.height; y++) for (let x = 0; x < png.width; x++) {
        if (png.alpha[y * png.width + x] < 128) continue;
        left = Math.min(left, x); right = Math.max(right, x);
      }
      return (right - left + 1) / png.width;
    };
    const townWidths = new Map(CITY_ERAS.flatMap((era) => CITY_TIERS.map((tier) => {
      const stem = `${era}-${tier}`;
      return [stem, widthOf(new URL(`../assets/cities/${stem}.png`, import.meta.url))] as const;
    })));
    for (const ringEra of CITY_ERAS) {
      const ringWidth = widthOf(new URL(`../assets/cities/${ringEra}-walls.png`, import.meta.url));
      for (const townEra of CITY_ERAS) for (const tier of CITY_TIERS) {
        const stem = `${townEra}-${tier}`;
        const scale = wallDrawScale(cityDrawScale(stem));
        const footprint = Math.max(ringWidth, townWidths.get(stem)! * WALLED_TOWN_SCALE) * CITY_WORLD * scale;
        expect(footprint / HEX_W, `${ringEra} ring around ${stem}`).toBeLessThanOrEqual(1.4);
      }
    }
  });
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
    // The ring follows the Walls level, not the owner's era: I Ancient .. IX Future.
    expect([1, 2, 4, 9].map((level) => wallStem(ringEraOf(level), false))).toEqual(['ancient-walls', 'classical-walls', 'renaissance-walls', 'future-walls']);
    expect(ringEraOf(0)).toBe(0);
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

it('gives the banner the visible Walls level, and nothing out of sight', () => {
  const c = (o: object) => ({ visible: true, walls: true, ...o });
  expect(wallTierOf(c({ walls: false }))).toBe(0);
  expect(wallTierOf(c({}))).toBe(1);
  expect(wallTierOf(c({ walls_level: 2 }))).toBe(2);
  expect(wallTierOf(c({ walls_level: 9 }))).toBe(9);
  expect(wallTierOf(c({ visible: false, walls_level: 3 }))).toBe(0);
});
