// @vitest-environment happy-dom
// Where a natural wonder's bonus shows up besides the tile: the city's breakdown (its own named line),
// the city panel's row, the Science hover, the Settle card and the Guide.
import { act } from 'react';
import { createRoot } from 'react-dom/client';
import { renderToStaticMarkup } from 'react-dom/server';
import { afterEach, expect, it, vi } from 'vitest';
import { WONDERS, WONDER_INFO } from '../wonders';
import type { CityYields, SettlementSite } from '../types';
import { YieldSourceTable } from './bits';
import { GUIDE_SECTIONS, GuideContent } from './GuideContent';
import { SettlementCard, SettlementDetails } from '../map/SettlementCard';

afterEach(() => { document.body.innerHTML = ''; vi.unstubAllGlobals(); });

function mount(node: React.ReactNode) {
  vi.stubGlobal('IS_REACT_ACT_ENVIRONMENT', true);
  const host = document.body.appendChild(document.createElement('div'));
  const root = createRoot(host);
  act(() => root.render(node));
  return { host, unmount: () => act(() => root.unmount()) };
}

const yields = {
  total: { food: 4, production: 4, gold: 2, science: 15 },
  sources: [
    { label: 'Citizens', yields: { food: 4, production: 4, gold: 2, science: 12 } },
    { label: 'Grand Canyon (+25%)', yields: { food: 0, production: 0, gold: 0, science: 3 } },
  ],
  worked: [], focus: 'Balanced', focus_reason: 'x',
} as unknown as CityYields;

it("the city's Science breakdown has a named wonder line: its painting, 'Grand Canyon +25%', and the amount with its icon", () => {
  const { host, unmount } = mount(<YieldSourceTable yields={yields} k="science" />);
  const row = host.querySelector('[data-wonder="GrandCanyon"]')!;
  expect(row.textContent).toBe('Grand Canyon +25%+3 ');
  expect(row.querySelector('img')?.getAttribute('src')).toContain('grand-canyon-b-128.png');
  expect(row.querySelector('.yield-icon')).not.toBeNull();
  // Other yields do not list it.
  unmount();
  const food = mount(<YieldSourceTable yields={yields} k="food" />);
  expect(food.host.querySelector('[data-wonder]')).toBeNull();
  food.unmount();
});

it('the Settle card names an unclaimed wonder in reach and what its owner would get, and keeps its points for Details', () => {
  const site = {
    pos: { q: 2, r: 2 }, score: 50, quality: 60, band: 'Great', rank: 1, travel_turns: 2, threats: 0, resources: [], foreign_tiles: 0, defense_bonus: 0, unknown_tiles: 0, border_risk: false,
    potential_yields: { food: 6, production: 4, gold: 0, science: 0 }, wonders: [{ wonder: 'GiantsCauseway', distance: 1, value: 80 }], breakdown: [],
  } as unknown as SettlementSite;
  const { host, unmount } = mount(<SettlementCard site={site} anchor={{ x: 0, y: 0 } as never} />);
  const text = host.querySelector('[data-testid="settle-wonders"]')!.textContent!;
  expect(text).toContain("Giant's Causeway");
  expect(text).toContain('1 tile away');
  expect(text).toContain('+25% Production');
  // The hover says what the wonder gives, never its points; those are one row of the Details arithmetic.
  expect(host.textContent).not.toContain('(+80)');
  expect(host.querySelector('[data-settle-line="wonders"]')).toBeNull();
  unmount();
  const detail = mount(<SettlementDetails site={{ ...site, ring_one: [], center_yields: site.potential_yields, terrain_bonus: {}, own_border_tiles: 0, nearest_city: null } as unknown as SettlementSite} />);
  expect(detail.host.querySelector('.settlement-math')!.textContent).toContain('+80');
  detail.unmount();
  const none = mount(<SettlementCard site={{ ...site, wonders: [] } as SettlementSite} anchor={{ x: 0, y: 0 } as never} />);
  expect(none.host.querySelector('[data-testid="settle-wonders"]')).toBeNull();
  none.unmount();
});

it('the Guide lists every wonder with its bonus and reward', () => {
  const html = GUIDE_SECTIONS.map((s) => renderToStaticMarkup(<GuideContent section={s.id} />)).join('').replace(/&#x27;/g, "'");
  expect(html).toContain('Natural wonders</h3>');
  for (const w of WONDERS) {
    expect(html).toContain(WONDER_INFO[w].name);
  }
  expect(html).toContain('+25% Science');
  expect(html).toContain('+25% Production');
  expect(html).toContain('+25% Gold');
  expect(html).toContain('+25% Food');
});
