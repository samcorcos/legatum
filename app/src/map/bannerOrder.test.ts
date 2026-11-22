import { describe, expect, it } from 'vitest';
import { bannerLift, neutralBarbarianCue } from './render';
import { BARBARIAN } from '../barbarians';
import { clearLensChip } from './drawLens';
import { bannerCityAt } from './bannerTooltip';

describe('banner legibility', () => {
  it('raises a token behind a banner to the banner top, at most half a hex, and leaves others alone', () => {
    const cover = [{ x: 0, y: 50, w: 120, h: 40 }];
    expect(bannerLift(100, 66, 10, 22, cover)).toBe(11); // the hex above: capped at half a hex
    expect(bannerLift(100, 56, 10, 40, cover)).toBe(6); // its centre goes to the banner's top edge
    expect(bannerLift(100, 95, 10, 22, cover)).toBe(0); // below the banner (a garrison, a hex beside it)
    expect(bannerLift(200, 66, 10, 22, cover)).toBe(0); // clear of it sideways
    expect(bannerLift(100, 30, 10, 22, cover)).toBe(0); // wholly above it
  });

  it('sends a click on a zoomed-in banner to its city, the top banner first, but not a zoomed-out seal', () => {
    const city = (id: number) => ({ id, pos: { q: id, r: 0 } }) as never;
    const box = { x: 100, y: 50, w: 120, h: 40 };
    const hits = [{ city: city(1), parts: [box], far: false }, { city: city(2), parts: [{ ...box, x: 180 }], far: false }];
    expect(bannerCityAt(hits, 110, 85)).toMatchObject({ id: 1 });
    expect(bannerCityAt(hits, 200, 60)).toMatchObject({ id: 2 });
    expect(bannerCityAt(hits, 110, 95)).toBeUndefined();
    expect(bannerCityAt([{ ...hits[0], far: true }], 110, 85)).toBeUndefined();
  });

  it('moves a Threat chip away from a neighbouring banner name', () => {
    const banner = { x: 90, y: 70, w: 160, h: 48 };
    const chip = clearLensChip(200, 100, 28, 16, [banner]);
    expect(chip).not.toBeNull();
    expect(chip!.x + 14 <= banner.x || chip!.x - 14 >= banner.x + banner.w || chip!.y + 8 <= banner.y || chip!.y - 8 >= banner.y + banner.h).toBe(true);
  });

  it('uses neutral strength marks only for barbarians seen by the Celts', () => {
    const view = { me: 0, players: [{ id: 0, faction: 'Celts' }] } as Parameters<typeof neutralBarbarianCue>[0];
    expect(neutralBarbarianCue(view, BARBARIAN)).toBe(true);
    expect(neutralBarbarianCue(view, 1)).toBe(false);
    expect(neutralBarbarianCue({ ...view, players: [{ ...view.players[0], faction: 'India' }] }, BARBARIAN)).toBe(false);
  });
});
