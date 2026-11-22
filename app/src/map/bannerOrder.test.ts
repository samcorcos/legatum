import { describe, expect, it } from 'vitest';
import { bannerAboveUnits, foreignUnitBeside, neutralBarbarianCue } from './render';
import { BARBARIAN } from '../barbarians';
import { clearLensChip } from './drawLens';

describe('banner legibility', () => {
  it('lifts an own banner above units only when a foreign unit stands beside it', () => {
    expect(bannerAboveUnits(0, 0, false)).toBe(false);
    expect(bannerAboveUnits(0, 0, false, true)).toBe(true);
    expect(bannerAboveUnits(1, 0, false, true)).toBe(false);
    expect(bannerAboveUnits(1, 0, true)).toBe(true);
  });

  it('counts only units not ours on or next to the city tile', () => {
    const at = { q: 5, r: 5 };
    const unit = (owner: number, q: number, r: number) => ({ owner, pos: { q, r } });
    expect(foreignUnitBeside({ me: 0, units: [unit(0, 5, 4)] } as never, at)).toBe(false);
    expect(foreignUnitBeside({ me: 0, units: [unit(BARBARIAN, 5, 4)] } as never, at)).toBe(true);
    expect(foreignUnitBeside({ me: 0, units: [unit(BARBARIAN, 5, 2)] } as never, at)).toBe(false);
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
