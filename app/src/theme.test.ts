import { describe, expect, it } from 'vitest';
import { FACTION_PALETTE, colorFactions, inkOfColor, playerColor, playerInk, setSeatFactions } from './theme';
import type { Faction } from './types';

const FACTIONS = Object.keys(FACTION_PALETTE) as Faction[];
const COLORS = FACTIONS.map((f) => FACTION_PALETTE[f].color);
const INKS = FACTIONS.map((f) => FACTION_PALETTE[f].ink);

const PAPER = '#f2ead8'; // --panel
const luminance = (hex: string) => {
  const [r, g, b] = [1, 3, 5].map((i) => parseInt(hex.slice(i, i + 2), 16) / 255).map((c) => (c <= 0.03928 ? c / 12.92 : ((c + 0.055) / 1.055) ** 2.4));
  return 0.2126 * r + 0.7152 * g + 0.0722 * b;
};
const contrast = (a: string, b: string) => {
  const [hi, lo] = [luminance(a), luminance(b)].sort((x, y) => y - x);
  return (hi + 0.05) / (lo + 0.05);
};
const hue = (hex: string) => {
  const [r, g, b] = [1, 3, 5].map((i) => parseInt(hex.slice(i, i + 2), 16) / 255);
  const max = Math.max(r, g, b), min = Math.min(r, g, b), d = max - min;
  if (d < 0.001) return 0;
  const h = max === r ? ((g - b) / d) % 6 : max === g ? (b - r) / d + 2 : (r - g) / d + 4;
  return (h * 60 + 360) % 360;
};

describe('per-empire ink on parchment', () => {
  it('has a shade for every faction colour, readable as a marker (3:1) on paper', () => {
    INKS.forEach((ink, i) => expect(contrast(ink, PAPER), `${FACTIONS[i]} ${ink}`).toBeGreaterThanOrEqual(3));
  });
  it('keeps every hue recognisable and every empire distinguishable', () => {
    COLORS.forEach((c, i) => {
      if (FACTIONS[i] === 'Germany') return; // steel has almost no hue: it must stay grey-blue
      const d = Math.abs(hue(c) - hue(INKS[i]));
      expect(Math.min(d, 360 - d), FACTIONS[i]).toBeLessThan(25);
    });
    expect(new Set(INKS).size).toBe(INKS.length);
    expect(new Set(COLORS).size).toBe(COLORS.length);
  });
  it('follows the faction, not the seat: Sparta is crimson and Athens azure wherever they sit', () => {
    setSeatFactions(['Athens', 'Sparta']);
    expect([playerColor(0), playerColor(1)]).toEqual(['#4aa8ff', '#e5534b']);
    setSeatFactions(['Sparta', 'Athens']);
    expect([playerColor(0), playerColor(1)]).toEqual(['#e5534b', '#4aa8ff']);
  });
  it('gives a second empire of the same faction the colour of a faction that is not playing', () => {
    const colored = colorFactions(['Sparta', 'Sparta', 'Athens']);
    expect(colored.slice(0, 1).concat(colored.slice(2))).toEqual(['Sparta', 'Athens']);
    expect(new Set(colored).size).toBe(3);
    expect(colored[1]).not.toBe('Sparta');
  });
  it('maps colours and seats alike, and passes unknown colours through', () => {
    setSeatFactions(FACTIONS);
    expect(inkOfColor(playerColor(7))).toBe(playerInk(7));
    expect(inkOfColor('#123456')).toBe('#123456');
    expect(playerInk(255)).toBe(playerColor(255));
  });
  it('keeps every empire’s ink clear of the gilt that means "yours"', () => {
    const GILT = '#8f6f1c';
    const sat = (hex: string) => { const [r, g, b] = [1, 3, 5].map((i) => parseInt(hex.slice(i, i + 2), 16) / 255); const max = Math.max(r, g, b), min = Math.min(r, g, b); return max === min ? 0 : (max - min) / (1 - Math.abs(max + min - 1)); };
    INKS.forEach((ink, i) => {
      if (sat(ink) < 0.25) return; // steel is grey: its saturation already tells it from gilt
      const d = Math.abs(hue(ink) - hue(GILT));
      expect(Math.min(d, 360 - d), `${FACTIONS[i]} ${ink}`).toBeGreaterThanOrEqual(20);
    });
  });
});
