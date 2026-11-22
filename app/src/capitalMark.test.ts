import { describe, expect, it } from 'vitest';
import { capitalLine, capitalLineOf, capitalMark, capitalStar, isReplacementCapital, replacementKickerLine, replacementTileLine } from './capitalMark';
import { bannerSpec } from './map/render';
import { bannerTooltip } from './map/bannerTooltip';
import { capitalSealRadius } from './map/sprites';
import { playerInk, setSeatFactions } from './theme';
import type { CityView, PlayerView } from './types';

// You are empire 0 (India, team 0). Empire 1 is a rival (Byzantium), 2 your teammate, 3 a second rival (Switzerland).
const player = (id: number, faction: string, team: number, taken: number[] = []) =>
  ({ id, name: faction, faction, team, capitals_taken: taken.map((c) => [c, `City ${c}`]) });
setSeatFactions(['India', 'Byzantium', 'Germany', 'Switzerland']);
const view = (taken: Record<number, number[]> = {}) => ({
  me: 0, team: 0,
  players: [player(0, 'India', 0, taken[0]), player(1, 'Byzantium', 1, taken[1]), player(2, 'Germany', 0, taken[2]), player(3, 'Switzerland', 3, taken[3])],
}) as unknown as PlayerView;
const city = (id: number, owner: number, extra: Partial<CityView> = {}) =>
  ({ id, owner, name: `City ${id}`, population: 3, visible: true, walls: false, is_capital: false, ...extra }) as CityView;

/** The four cases the map must tell apart. */
const held = city(10, 1, { is_capital: true, original_capital: true, original_capital_of: 1 }); // a rival's capital, held by its founder
const captured = city(11, 3, { is_capital: false, original_capital: true, original_capital_of: 1 }); // Byzantium's capital, now Swiss
const replacement = city(12, 1, { is_capital: true, original_capital: false, original_capital_of: null }); // Byzantium's new capital
const ordinary = city(13, 1);

describe('capital mark', () => {
  it('names population, holder, city and the founder-specific scoring star', () => {
    expect(bannerTooltip(view(), captured)).toBe("Population 3 · Switzerland · City 11 · Original capital of Byzantium · a rival scores 1 VP while it holds it");
    expect(bannerTooltip(view(), ordinary)).toBe('Population 3 · Byzantium · City 13');
    expect(bannerTooltip(view(), { ...held, visible: false })).toContain('Population unknown · Byzantium');
  });
  it('marks an original capital held by its founder, in the founder ink', () => {
    const star = capitalStar(view(), held)!;
    expect(star).toEqual({ ink: playerInk(1), scored: false });
  });

  it('keeps the star after capture, in the founder ink and not the captor ink', () => {
    const mark = capitalMark(view(), captured)!;
    expect(mark.founder).toBe(1);
    expect(mark.ink).toBe(playerInk(1));
    expect(mark.ink).not.toBe(playerInk(captured.owner));
  });

  it('gives a replacement capital and an ordinary city no star', () => {
    expect(capitalMark(view(), replacement)).toBeNull();
    expect(capitalMark(view(), ordinary)).toBeNull();
    expect(capitalLineOf(view(), replacement)).toBeNull();
  });

  it('reads as scored only when you took it yourself: the point is per captor, not per team', () => {
    expect(capitalMark(view({ 0: [11] }), captured)!.scored).toBe('you');
    expect(capitalMark(view({ 2: [11] }), captured)!.scored).toBe('teammate');
    expect(capitalMark(view({ 1: [11] }), captured)!.scored).toBeNull(); // a rival's score is not yours
    expect(capitalStar(view({ 0: [11] }), captured)!.scored).toBe(true);
    // A teammate's capture leaves your star filled: you can still score it yourself.
    expect(capitalStar(view({ 2: [11] }), captured)!.scored).toBe(false);
  });

  it('marks your own founding city as yours', () => {
    const mine = city(1, 0, { is_capital: true, original_capital: true, original_capital_of: 0 });
    expect(capitalMark(view(), mine)).toMatchObject({ own: true, founder: 0, scored: null });
  });

  it('marks a city the founder lost (no founder field) by its owner', () => {
    expect(capitalMark(view(), city(5, 1, { original_capital: true }))!.founder).toBe(1);
  });
});

describe('capital line', () => {
  it('states the rule for a rival capital, naming its founder', () => {
    expect(capitalLineOf(view(), held)).toBe("Byzantium's original capital. Hold it to score 1 VP.");
  });

  it('names the founder, not the holder, after capture', () => {
    expect(capitalLineOf(view(), captured)).toContain("Byzantium's original capital.");
  });

  it('says you score it while you hold it', () => {
    expect(capitalLineOf(view({ 0: [11] }), { ...captured, owner: 0 })).toBe("Byzantium's original capital. You score 1 VP while you hold it.");
  });

  it('tells you a teammate holds its point', () => {
    expect(capitalLineOf(view({ 2: [11] }), { ...captured, owner: 2 })).toBe("Byzantium's original capital. Your teammate scores 1 VP while holding it.");
  });

  it('warns the owner that a rival scores while holding it', () => {
    const mine = city(1, 0, { is_capital: true, original_capital: true, original_capital_of: 0 });
    expect(capitalLineOf(view(), mine)).toBe('Your original capital. A rival scores 1 VP while it holds it.');
    // Still true after it is lost.
    expect(capitalLineOf(view(), { ...mine, owner: 1 })).toBe('Your original capital. A rival scores 1 VP while it holds it.');
  });

  it("words a teammate's capital as theirs, not as a point you can score", () => {
    const mate = city(20, 3, { original_capital: true, original_capital_of: 2 });
    expect(capitalLineOf(view(), mate)).toBe("Your teammate's original capital (Germany). A rival scores 1 VP while it holds it.");
  });

  it('is built from the mark alone', () => {
    const mark = capitalMark(view(), held)!;
    expect(capitalLine(view(), mark)).toBe(capitalLineOf(view(), held));
  });
});

describe('replacement capital', () => {
  // Byzantium (1) lost its original capital, city 11, to Switzerland; city 12 is its capital now.
  const v = { ...view(), cities: [captured, replacement] } as PlayerView;
  it('says the original capital fell and that capturing it scores nothing', () => {
    expect(replacementKickerLine(v, replacement)).toBe('Capital since City 11 fell. Capturing it scores no VP.');
    expect(replacementTileLine(v, replacement)).toBe("Capturing it scores no VP: Byzantium's original capital is City 11.");
  });
  it('words it for your own replacement capital', () => {
    const mine = city(30, 0, { is_capital: true });
    const own = { ...view(), cities: [city(31, 1, { original_capital: true, original_capital_of: 0 }), mine] } as PlayerView;
    expect(replacementTileLine(own, mine)).toBe('Capturing it scores no VP: your original capital is City 31.');
  });
  it('falls back when the original capital is not known', () => {
    const lone = { ...view(), cities: [replacement] } as PlayerView;
    expect(replacementKickerLine(lone, replacement)).toBe('Capital since the original capital was lost. Capturing it scores no VP.');
    expect(replacementTileLine(lone, replacement)).toBe('Capturing it scores no VP: it is not an original capital.');
  });
  it('is only for a visible current capital that is not original', () => {
    expect(replacementKickerLine(v, held)).toBeNull();
    expect(replacementKickerLine(v, ordinary)).toBeNull();
    expect(replacementKickerLine(v, { ...replacement, visible: false })).toBeNull();
    expect(isReplacementCapital(replacement)).toBe(true);
  });
});

describe('banner spec', () => {
  const spec = (c: CityView, taken: Record<number, number[]> = {}) => bannerSpec(view(taken), c, null, false).capital;

  it('carries the star for an original capital held by its founder', () => {
    expect(spec(held)).toEqual({ ink: playerInk(1), scored: false });
  });
  it('keeps the star on a captured original capital, in the founder ink', () => {
    expect(spec(captured)).toEqual({ ink: playerInk(1), scored: false });
  });
  it('has no star on a replacement capital, even though it is the current capital', () => {
    expect(replacement.is_capital).toBe(true);
    expect(spec(replacement)).toBeNull();
  });
  it('shows the star as scored once the viewer has taken it', () => {
    expect(spec(captured, { 0: [11] })).toEqual({ ink: playerInk(1), scored: true });
  });
  it('keeps the star filled when only a teammate has taken it', () => {
    expect(spec(captured, { 2: [11] })).toEqual({ ink: playerInk(1), scored: false });
  });
  it('gives the banner seal a disc at least 16 px across at the smallest banner (1000x600, zoom 0.75)', () => {
    const hexPx = 36 * 0.75, fs = Math.round(Math.max(11, Math.min(16, hexPx * 0.42)) * 2) / 2;
    expect(capitalSealRadius(fs) * 2).toBeGreaterThanOrEqual(16);
    expect(capitalSealRadius(16) * 2).toBeGreaterThan(Math.round(16 * 1.73));
  });
  it('shows the star through fog: an original capital is public on an explored city', () => {
    expect(spec({ ...held, visible: false })).toEqual({ ink: playerInk(1), scored: false });
  });
});
