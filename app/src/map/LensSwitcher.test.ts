// @vitest-environment happy-dom
import { act, createElement } from 'react';
import { createRoot } from 'react-dom/client';
import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import { LensSwitcher } from './LensSwitcher';
import type { MapLens } from './lenses';
import type { SettlementScore } from '../types';
import { DEFAULT_OVERLAYS, getOverlays, reloadOverlays, setOverlay } from './overlays';

(globalThis as { IS_REACT_ACT_ENVIRONMENT?: boolean }).IS_REACT_ACT_ENVIRONMENT = true;

// Overlays are the viewer's own and live in the page, so each test starts from the first-run state.
beforeEach(() => { vi.stubGlobal('localStorage', { getItem: () => null, setItem: () => {} }); reloadOverlays(); });
afterEach(() => { vi.unstubAllGlobals(); reloadOverlays(); });

it('collapses to the compact toggle on desktop while a city is selected, and restores the dock after', () => {
  const media = Object.assign(new EventTarget(), { matches: false }) as MediaQueryList;
  const matchMedia = vi.spyOn(window, 'matchMedia').mockReturnValue(media);
  const host = document.body.appendChild(document.createElement('div'));
  const root = createRoot(host);
  let lens: MapLens = 'default';
  const render = (collapsed: boolean) => act(() => root.render(createElement(LensSwitcher, {
    lens, onChange: (next: MapLens) => { lens = next; }, sites: [], onInspect: () => {}, settlerId: null, players: [], collapsed,
  })));
  try {
    render(false);
    expect(host.querySelector('.lens-toggle')).toBeNull();
    expect(host.querySelector<HTMLDivElement>('.lens-content')!.hidden).toBe(false);
    render(true);
    const toggle = host.querySelector<HTMLButtonElement>('.lens-toggle')!;
    // The Default lens beside a panel is a small chip that still names the lens to assistive tech.
    expect(toggle.textContent).toBe('Lens');
    expect(toggle.getAttribute('aria-label')).toBe('Lens: Default');
    expect(host.querySelector('.map-lenses')!.getAttribute('data-chip')).toBe('true');
    expect(host.querySelector('.map-lenses')!.getAttribute('data-compact')).toBe('true');
    expect(host.querySelector<HTMLDivElement>('.lens-content')!.hidden).toBe(true);
    act(() => toggle.click());
    expect(host.querySelector<HTMLDivElement>('.lens-content')!.hidden).toBe(false);
    const settle = [...host.querySelectorAll<HTMLButtonElement>('[role="radio"]')].find((b) => b.textContent === 'Settle')!;
    act(() => settle.click());
    expect(lens).toBe('settle');
    expect(host.querySelector<HTMLDivElement>('.lens-content')!.hidden).toBe(true);
    render(false);
    expect(host.querySelector('.lens-toggle')).toBeNull();
    expect(host.querySelector<HTMLDivElement>('.lens-content')!.hidden).toBe(false);
  } finally {
    act(() => root.unmount()); host.remove(); matchMedia.mockRestore();
  }
});

it.each([false, true])('keeps the settle legend visible and recommendations score-free (phone: %s)', (phone) => {
  const matchMedia = vi.spyOn(window, 'matchMedia').mockImplementation((query) => Object.assign(new EventTarget(), {
    matches: phone && (query === '(max-width: 699px)' || query === '(pointer: coarse)'),
  }) as MediaQueryList);
  const host = document.body.appendChild(document.createElement('div'));
  const root = createRoot(host), inspect = vi.fn();
  const sites: SettlementScore[] = [1, 2, 3].map((rank) => ({ pos: { q: rank, r: 0 }, rank, score: 160 - rank, quality: 160 - rank, band: 'Good' as const, border_risk: false, travel_turns: rank }));
  try {
    act(() => root.render(createElement(LensSwitcher, { lens: 'settle', onChange: () => {}, sites, onInspect: inspect, settlerId: 7, players: [] })));
    const legend = host.querySelector('.settle-legend')!;
    expect(legend.textContent).toContain(`Better places to found a city are darker green. ${phone ? 'Tap' : 'Hover'} a tile for why. No tint: not a site.`);
    expect(legend.querySelector('.settle-pips, [data-testid="settle-floor"]')).toBeNull();
    expect(legend.closest('[hidden]')).toBeNull();
    expect([...legend.querySelectorAll('.settle-scale [data-settle-band]')].map((el) => el.getAttribute('data-settle-band'))).toEqual(['Poor', 'Fair', 'Good', 'Great', 'Exceptional']);
    expect(legend.querySelector('.settle-risk-key')).toBeNull();
    expect(host.textContent).not.toMatch(/scores in points|0–59|159|158|157/);
    if (phone) {
      expect(host.querySelector<HTMLDivElement>('.lens-content')!.hidden).toBe(true);
      act(() => host.querySelector<HTMLButtonElement>('.lens-toggle')!.click());
    }
    const recommendations = [...host.querySelectorAll<HTMLButtonElement>('[data-settle-rank]')];
    expect(recommendations.map((el) => el.textContent)).toEqual(['Best spot', '#2', '#3']);
    act(() => recommendations[0].click());
    expect(inspect).toHaveBeenCalledWith(sites[0].pos);
    expect(legend.closest('[hidden]')).toBeNull();
  } finally {
    act(() => root.unmount()); host.remove(); matchMedia.mockRestore();
  }
});

const mount = (props: Partial<Parameters<typeof LensSwitcher>[0]> = {}) => {
  const host = document.body.appendChild(document.createElement('div'));
  const root = createRoot(host);
  let lens: MapLens = props.lens ?? 'default';
  const render = () => act(() => root.render(createElement(LensSwitcher, { lens, onChange: (next: MapLens) => { lens = next; render(); }, sites: [], onInspect: () => {}, settlerId: null, players: [], ...props })));
  render();
  return { host, render, get lens() { return lens; }, set lens(next: MapLens) { lens = next; render(); }, done: () => { act(() => root.unmount()); host.remove(); } };
};
const checkbox = (host: HTMLElement, name: string) => [...host.querySelectorAll<HTMLButtonElement>('[role="checkbox"]')].find((b) => b.textContent?.startsWith(name))!;
const radio = (host: HTMLElement, name: string) => [...host.querySelectorAll<HTMLButtonElement>('[role="radio"]')].find((b) => b.textContent === name)!;

it('shows one row with Default, Settle and Yields, and no group labels', () => {
  const m = mount();
  try {
    expect([...m.host.querySelectorAll('.lens-bar button')].map((b) => b.textContent)).toEqual(['Default', 'Settle', 'YieldsY']);
    expect(m.host.querySelector('.lens-group-label')).toBeNull();
    expect([...m.host.querySelectorAll('[role="radio"]')].map((b) => b.textContent)).toEqual(['Default', 'Settle']);
    expect([...m.host.querySelectorAll('[role="checkbox"]')].map((b) => b.getAttribute('aria-keyshortcuts'))).toEqual(['Y']);
  } finally { m.done(); }
});

it('Yields stacks over either lens and shows its one-line key', () => {
  const m = mount();
  try {
    act(() => checkbox(m.host, 'Yields').click());
    expect(getOverlays().yields).toBe(true);
    expect(m.host.querySelector('.lens-yield-line')?.textContent).toContain('1 icon per point');
    act(() => radio(m.host, 'Settle').click());
    expect(m.lens).toBe('settle');
    expect(getOverlays().yields).toBe(true);
    act(() => checkbox(m.host, 'Yields').click());
    expect(getOverlays()).toEqual(DEFAULT_OVERLAYS);
  } finally { m.done(); }
});

it('arrow keys wrap between the two lenses and leave Yields on', () => {
  setOverlay('yields', true);
  const m = mount();
  try {
    const group = m.host.querySelector('[role="radiogroup"]')!;
    const press = (key: string) => act(() => group.dispatchEvent(new KeyboardEvent('keydown', { key, bubbles: true, cancelable: true })));
    press('ArrowRight'); expect(m.lens).toBe('settle');
    press('ArrowRight'); expect(m.lens).toBe('default');
    press('ArrowLeft'); expect(m.lens).toBe('settle');
    press('Home'); expect(m.lens).toBe('default');
    press('End'); expect(m.lens).toBe('settle');
    expect(getOverlays().yields).toBe(true);
  } finally { m.done(); }
});

it('compact toggle names Yields when it is on', () => {
  const media = Object.assign(new EventTarget(), { matches: false }) as MediaQueryList;
  const matchMedia = vi.spyOn(window, 'matchMedia').mockReturnValue(media);
  const m = mount({ collapsed: true });
  try {
    expect(m.host.querySelector('.lens-toggle')!.textContent).toBe('Lens');
    act(() => setOverlay('yields', true));
    expect(m.host.querySelector('.lens-toggle')!.textContent).toBe('Lens: Default · Yields');
  } finally { m.done(); matchMedia.mockRestore(); }
});

it('the Settle legend names the selected unit, and a Colonia founder is shown its Colonia sites', () => {
  const host = document.body.appendChild(document.createElement('div'));
  const root = createRoot(host);
  const site = (rank: number, colonia: boolean): SettlementScore => ({ pos: { q: rank, r: 0 }, score: 50, quality: 50, band: 'Good', border_risk: false, rank, travel_turns: 2, ...(colonia ? { colonia } : {}) });
  const render = (sites: SettlementScore[], coloniaFounder: boolean, founderName?: string) => act(() => root.render(createElement(LensSwitcher, {
    lens: 'settle', onChange: () => {}, sites, onInspect: () => {}, settlerId: 7, players: [], coloniaFounder, founderName,
  })));
  try {
    render([site(1, true), site(2, true)], true, 'Spearman');
    expect(host.querySelector('[data-testid="settle-top-key"]')?.textContent).toBe('Top three Colonia sites for this veteran, marked on the map:');
    expect(host.querySelector('.settle-recommendations.colonia [data-settle-rank="1"]')?.textContent).toBe('Best Colonia site');
    render([site(1, false), site(2, false)], true, 'Spearman');
    expect(host.querySelector('[data-testid="settle-top-key"]')?.textContent).toBe('Top three for this Spearman, marked on the map:');
    expect(host.querySelector('[data-testid="settle-colonia-none"]')?.textContent).toMatch(/^No site you know founds your Colonia Victrix\. A site counts if/);
    render([site(1, false)], false);
    expect(host.querySelector('[data-testid="settle-top-key"]')?.textContent).toBe('Top three for this Settler, marked on the map:');
  } finally { act(() => root.unmount()); host.remove(); }
});
