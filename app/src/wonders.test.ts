import { describe, expect, it } from 'vitest';
import type { PlayerView, Wonder } from './types';
import { tileIndex } from './hex';
import {
  WONDERS, WONDER_INFO, cityWonders, holdingCity, wonderArtAlpha, wonderBadgeText, wonderBonus, wonderBonusSentence, wonderDiscoveryLine,
  wonderLineLabel, wonderOfLabel, wonderRewardText,
} from './wonders';
import { wonderBadgeLayout } from './map/wonderArt';

const tiles = (w: number, h: number) => Array.from({ length: w * h }, () => ({ explored: true, visible: true, terrain: 'Hills', ruin: false, owner: null as number | null, wonder: null as Wonder | null }));
const view = (): PlayerView => ({ width: 8, height: 8, tiles: tiles(8, 8), cities: [] }) as unknown as PlayerView;

describe('wonder bonus text', () => {
  it('names the yield for each of the eight wonders, in plain words', () => {
    const want: Record<Wonder, string> = {
      MountFuji: 'Science', GrandCanyon: 'Science', GreatBarrierReef: 'Science', GiantsCauseway: 'Production',
      VictoriaFalls: 'Food', HaLongBay: 'Food', IguazuFalls: 'Food', Uluru: 'Gold',
    };
    for (const w of WONDERS) expect(wonderBonusSentence(w)).toBe(`The city that owns this tile gets +25% ${want[w]}.`);
  });
  it('the map badge says +25%', () => expect(wonderBadgeText()).toBe('+25%'));
  it('the discovery reward is 50 Science or 80 Gold, and a plain note once paid', () => {
    expect(wonderRewardText('MountFuji')).toBe('50 Science');
    expect(wonderRewardText('Uluru')).toBe('80 Gold');
    expect(wonderDiscoveryLine('Uluru', false)).toBe('Discovery reward: 80 Gold, paid once to the first empire to see it.');
    expect(wonderDiscoveryLine('Uluru', true)).toContain('already paid');
  });
  it('the bonus rounds up from the pre-wonder base', () => {
    expect(wonderBonus('GrandCanyon', { food: 9, production: 9, gold: 9, science: 9 })).toEqual({ food: 0, production: 0, gold: 0, science: 3 });
    expect(wonderBonus('Uluru', { food: 0, production: 0, gold: 1, science: 0 }).gold).toBe(1);
  });
  it('recognises the core breakdown label and names the line', () => {
    expect(wonderOfLabel('Grand Canyon (+25%)')).toBe('GrandCanyon');
    expect(wonderOfLabel('Capital')).toBeNull();
    expect(wonderLineLabel('GrandCanyon')).toBe('Grand Canyon +25%');
    for (const w of WONDERS) expect(wonderOfLabel(`${WONDER_INFO[w].name} (+25%)`)).toBe(w);
  });
});

describe('wonders in play', () => {
  it('a city lists the wonders in its territory with the amount from its own breakdown', () => {
    const v = view();
    v.tiles[tileIndex({ q: 2, r: 2 }, 8, 8)].wonder = 'GiantsCauseway';
    const city = { detail: { territory: [{ q: 2, r: 2 }, { q: 3, r: 2 }], yields: { sources: [{ label: 'Giant\'s Causeway (+25%)', yields: { food: 0, production: 4, gold: 0, science: 0 } }] } } } as never;
    expect(cityWonders(v, city)).toEqual([{ wonder: 'GiantsCauseway', hex: { q: 2, r: 2 }, amount: 4 }]);
    expect(cityWonders(v, { detail: null } as never)).toEqual([]);
    expect(holdingCity({ cities: [city] }, { q: 2, r: 2 })).toBe(city);
    expect(holdingCity({ cities: [city] }, { q: 5, r: 5 })).toBeNull();
  });
});

describe('wonder art on the map', () => {
  it('drops under every lens and stays quieter when remembered', () => {
    expect(wonderArtAlpha(true, undefined)).toBe(0.95);
    expect(wonderArtAlpha(true, 'default')).toBe(0.95);
    for (const lens of ['settle', 'movement', 'threat', 'discovery']) expect(wonderArtAlpha(true, lens)).toBeLessThan(0.6);
    // The Yields overlay only steps the painting back a little, whichever lens is on.
    expect(wonderArtAlpha(true, 'default', true)).toBeLessThan(0.95);
    expect(wonderArtAlpha(true, 'default', true)).toBeGreaterThan(0.6);
    expect(wonderArtAlpha(true, 'settle', true)).toBe(wonderArtAlpha(true, 'settle'));
    expect(wonderArtAlpha(false, undefined)).toBeLessThan(wonderArtAlpha(true, undefined));
  });
  it('the badge fits inside the hex at normal zoom and sits in its upper half', () => {
    for (const hexPx of [24, 36, 48, 72]) {
      const b = wonderBadgeLayout(hexPx);
      expect(b.dy).toBeLessThan(0);
      expect(b.font).toBeGreaterThanOrEqual(10);
      // Never wider than the hex at its height once zoomed in a little (the floor font needs about 32 px).
      if (hexPx >= 32) expect(b.width).toBeLessThanOrEqual(hexPx * 1.45);
    }
  });
});

describe('layering', () => {
  it('the painting is drawn before every overlay it could overhang, and the badge after the lens tint and yield icons', async () => {
    const { readFileSync } = await import('node:fs');
    const src = readFileSync(new URL('./map/render.ts', import.meta.url), 'utf8');
    const at = (needle: string) => { const i = src.indexOf(needle); expect(i, needle).toBeGreaterThan(0); return i; };
    const art = at('drawWonderArt(ctx');
    for (const later of ['drawImprovements(ctx', 'drawLens(ctx, inp.lensTiles', 'drawYieldIcons(ctx', 'drawResourceIcons(ctx', 'drawRoads(ctx', 'drawUnit(']) expect(art, later).toBeLessThan(at(later));
    const badge = at('drawWonderBadges(ctx');
    expect(badge).toBeGreaterThan(at('drawYieldIcons(ctx'));
    expect(badge).toBeGreaterThan(at('drawLens(ctx, inp.lensTiles'));
    expect(badge).toBeLessThan(at('drawUnit('));
  });
});
