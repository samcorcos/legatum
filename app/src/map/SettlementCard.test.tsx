// @vitest-environment happy-dom
// The Settle hover in plain words (user, 2026-10-17: "I don't understand what this settler view is trying to tell me").
// A sentence first, the key numbers, then the relative line; the arithmetic is behind a Details toggle.
import { act } from 'react';
import { createRoot } from 'react-dom/client';
import { afterEach, expect, it, vi } from 'vitest';
import type { SettlementSite } from '../types';
import { auditBlocks } from '../test/textArtifacts';
import { SettlementCard, SettlementDetails } from './SettlementCard';

afterEach(() => { document.body.innerHTML = ''; vi.unstubAllGlobals(); });

function mount(node: React.ReactNode) {
  vi.stubGlobal('IS_REACT_ACT_ENVIRONMENT', true);
  const host = document.body.appendChild(document.createElement('div'));
  const root = createRoot(host);
  act(() => root.render(node));
  return { host, unmount: () => act(() => root.unmount()) };
}

const site = {
  pos: { q: 2, r: 2 }, score: 212, quality: 214, band: 'Great', rank: 1, travel_turns: 3, threats: 0,
  potential_yields: { food: 23, production: 36, gold: 0, science: 0 }, center_yields: { food: 2, production: 1, gold: 0, science: 0 }, terrain_bonus: {}, ring_one: [],
  resources: [{ resource: 'Wheat', distance: 3 }], wonders: [], foreign_tiles: 0, own_border_tiles: 0, nearest_city: 5, defense_bonus: 3, unknown_tiles: 0, border_risk: false,
  breakdown: [{ label: 'Travel', value: -2 }],
} as unknown as SettlementSite;

it('leads with one plain sentence, then the key numbers, then the route', () => {
  const { host, unmount } = mount(<SettlementCard site={site} anchor={{ x: 0, y: 0 } as never} />);
  const text = host.querySelector('.settlement-card')!.textContent!;
  expect(host.querySelector('[data-settle-band]')!.textContent).toBe('Great spot for a city: plenty of Food and Production nearby, Wheat 3 tiles away and the terrain gives units here +3 defence.');
  expect(host.querySelector('[data-settle-line="land"]')!.textContent).toMatch(/^Nearby land: \+23 .*Food.*\+36 .*Production/s);
  expect(host.querySelector('[data-settle-route]')!.textContent).toBe('Best spot · 3 turns away');
  // Order on the card: sentence, numbers, route.
  const order = ['spot for a city', 'Nearby land', 'Best spot'].map((w) => text.indexOf(w));
  expect([...order].sort((a, b) => a - b)).toEqual(order);
  expect(host.querySelector('.settlement-math, .settlement-breakdown')).toBeNull();
  expect(text).not.toMatch(/points|outer ring|×|quality|\bhex/i);
  expect(auditBlocks('settle hover', [text])).toEqual([]);
  unmount();
});

it('keeps every weight and point behind a closed Details toggle', () => {
  const { host, unmount } = mount(<SettlementDetails site={site} />);
  const details = host.querySelector<HTMLDetailsElement>('.settlement-breakdown')!;
  expect(details.open).toBe(false);
  expect(details.querySelector('summary')!.textContent).toBe('Details');
  expect(details.textContent).toContain('outer ring at half');
  expect(details.textContent).toContain('Site quality: 214 (Great');
  // Outside the toggle the card is as plain as the hover.
  const outside = [...host.children].flatMap((el) => el.matches('.settlement-breakdown') ? [] : [el.textContent ?? '']).join(' ');
  expect(outside).not.toMatch(/outer ring|×|points/);
  unmount();
});

it('keeps unseen-city risk in the hover and click-through heading', () => {
  const risky = { ...site, border_risk: true } as SettlementSite;
  const hover = mount(<SettlementCard site={risky} anchor={{ x: 0, y: 0 } as never} />);
  expect(hover.host.querySelector('[data-settle-band]')?.textContent).toMatch(/^Great spot for a city \(unconfirmed\):/);
  hover.unmount();
  const clickThrough = mount(<SettlementDetails site={risky} />);
  expect(clickThrough.host.querySelector('[data-settle-band]')?.textContent).toMatch(/^Great spot for a city \(unconfirmed\):/);
  clickThrough.unmount();
});
