import { afterEach, describe, expect, it, vi } from 'vitest';
import { drawPromotion } from './sprites';

function context() {
  const rect = vi.fn();
  const stroke = vi.fn();
  const ctx = {
    save: vi.fn(), restore: vi.fn(), translate: vi.fn(), scale: vi.fn(),
    beginPath: vi.fn(), rect, fill: vi.fn(), stroke,
    moveTo: vi.fn(), lineTo: vi.fn(),
  } as unknown as CanvasRenderingContext2D;
  return { ctx, rect, stroke };
}

describe('map promotion mark', () => {
  afterEach(() => vi.unstubAllGlobals());

  it.each([1, 2, 3, 4, 5, 6] as const)('draws level %i through the shared path badge', (level) => {
    vi.stubGlobal('Path2D', class { constructor(_path: string) {} });
    const { ctx, rect, stroke } = context();
    drawPromotion(ctx, 'atlas', level, 100, 100, 20, level === 6 ? 'Sparta' : 'India');
    expect(rect).not.toHaveBeenCalled();
    expect(ctx.translate).toHaveBeenCalledTimes(1);
    expect(stroke).toHaveBeenCalledTimes(1 + 2 * level + (level === 6 ? 2 : 0));
  });

  it('draws nothing at level 0', () => {
    const { ctx, rect, stroke } = context();
    drawPromotion(ctx, 'atlas', 0, 100, 100, 20, 'India');
    expect(rect).not.toHaveBeenCalled();
    expect(stroke).not.toHaveBeenCalled();
  });
});
