import { describe, expect, it } from 'vitest';
import { SELECT_BURST_MS, selectionBreath, selectionBurst } from './selectionMark';

describe('selection mark', () => {
  it('bursts out from the tile and fades, then stops', () => {
    expect(selectionBurst(0)).toEqual({ scale: 1, alpha: 1 });
    const mid = selectionBurst(SELECT_BURST_MS / 2)!;
    expect(mid.scale).toBeGreaterThan(1);
    expect(mid.alpha).toBeCloseTo(0.5);
    expect(selectionBurst(SELECT_BURST_MS)).toBeNull();
    expect(selectionBurst(-1)).toBeNull();
  });
  it('holds still when motion is off', () => {
    expect(selectionBreath(1234, true)).toBe(0.5);
    expect(selectionBreath(1234, false)).toBeGreaterThanOrEqual(0);
  });
});
