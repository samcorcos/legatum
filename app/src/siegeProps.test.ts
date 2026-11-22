import { describe, expect, it } from 'vitest';
import { tokenHalf } from './map/grid';
import { siegePropFor, siegePropRect, type SiegeProp } from './siegeProps';
import type { PlayerView, TechId, UnitKind, UnitView } from './types';

const PROPS: SiegeProp[] = ['catapult', 'trebuchet', 'bombard', 'ladders', 'siege-tower'];

it('a covered unit\'s siege gear stays inside its own hex at every zoom (user, 2026-10-19, 8.9.6)', () => {
  for (const hexPx of [22, 28, 36, 45, 60, 80, 120]) {
    const w = tokenHalf(hexPx), half = hexPx * (Math.sqrt(3) / 2), cx = 500;
    for (const prop of PROPS) {
      const r = siegePropRect(prop, w, cx, half, 300);
      expect(r.right, `${prop} at ${hexPx}px: right edge`).toBeLessThanOrEqual(cx + half);
      expect(r.left, `${prop} at ${hexPx}px: left edge`).toBeGreaterThanOrEqual(cx - half);
      expect(r.right - r.left, `${prop} at ${hexPx}px: width`).toBeLessThanOrEqual(w * 2.2 + 0.001);
    }
  }
});

const view = (...known: TechId[]) => ({ me: 0, research: { known }, players: [] }) as unknown as PlayerView;
const unit = (kind: UnitKind, siege = true) => ({ owner: 0, kind, military: true, siege }) as unknown as UnitView;
const all: TechId[] = ['Mathematics', 'Machinery', 'Metallurgy'];

describe('siege gear belongs to the unit\'s own era (user, 2026-10-28)', () => {
  it('ancient and medieval units wear the gear of the siege tier', () => {
    expect(siegePropFor(view('Mathematics'), unit('Archer'))).toBe('catapult');
    expect(siegePropFor(view('Mathematics', 'Machinery'), unit('Crossbowman'))).toBe('trebuchet');
    expect(siegePropFor(view(...all), unit('Crossbowman'))).toBe('trebuchet');
    expect(siegePropFor(view('Mathematics'), unit('Crossbowman'))).toBe('catapult');
    expect(siegePropFor(view('Mathematics'), unit('Spearman'))).toBe('ladders');
    expect(siegePropFor(view(...all), unit('Knight'))).toBe('siege-tower');
    expect(siegePropFor(view(...all), unit('Knight', false))).toBeNull();
  });
  it('the prop is capped by the unit\'s own era, whatever the tier (designer, 2026-10-28)', () => {
    for (const kind of ['Archer', 'CompositeArcher'] as const) {
      for (const known of [['Mathematics'], ['Mathematics', 'Machinery'], all] as TechId[][]) expect(siegePropFor(view(...known), unit(kind)), `${kind} ${known.length}`).toBe('catapult');
    }
    expect(siegePropFor(view(...all), unit('Spearman'))).toBe('siege-tower');
    expect(siegePropFor(view('Mathematics'), unit('Spearman'))).toBe('ladders');
  });
  it('Renaissance units get the bombard or ladders, never a catapult or siege tower', () => {
    expect(siegePropFor(view('Mathematics'), unit('Arquebusier'))).toBe('bombard');
    expect(siegePropFor(view(...all), unit('Arquebusier'))).toBe('bombard');
    expect(siegePropFor(view(...all), unit('Musketman'))).toBe('ladders');
  });
  it('Industrial and later units have Siege built in and wear no gear (designer, 2026-10-28)', () => {
    for (const kind of ['Rifleman', 'FieldCannon', 'Cavalry', 'Infantry', 'Artillery', 'Tank'] as const) {
      expect(siegePropFor(view(...all), unit(kind, false)), kind).toBeNull();
      expect(siegePropFor(view(...all), unit(kind)), kind).toBeNull();
    }
  });
});
