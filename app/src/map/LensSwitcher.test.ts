// @vitest-environment happy-dom
import { act, createElement } from 'react';
import { createRoot } from 'react-dom/client';
import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import { LensSwitcher } from './LensSwitcher';
import type { MapLens } from './lenses';
import type { PlayerPublic, SettlementScore, SupplyView } from '../types';
import { factionName } from '../data';
import { DEFAULT_OVERLAYS, getOverlays, reloadOverlays, setOverlay } from './overlays';

(globalThis as { IS_REACT_ACT_ENVIRONMENT?: boolean }).IS_REACT_ACT_ENVIRONMENT = true;

// Overlays are the viewer's own and live in the page, so each test starts from the first-run state.
beforeEach(() => { vi.stubGlobal('localStorage', { getItem: () => null, setItem: () => {} }); reloadOverlays(); });
afterEach(() => { vi.unstubAllGlobals(); reloadOverlays(); });

it('says so when a lens has nothing to mark, instead of looking like it did not switch', () => {
  const host = document.createElement('div');
  document.body.append(host);
  const root = createRoot(host);
  const render = (empty: boolean) => act(() => root.render(createElement(LensSwitcher, {
    lens: 'threat', onChange: () => {}, sites: [], onInspect: () => {}, settlerId: null, players: [], empty,
  })));
  render(true);
  expect(host.querySelector('.lens-empty')?.textContent).toMatch(/No known enemy military/);
  render(false);
  expect(host.querySelector('.lens-empty')).toBeNull();
  act(() => root.unmount());
  host.remove();
});

it('names visible barbarians in the Celts Threat empty state', () => {
  const host = document.body.appendChild(document.createElement('div'));
  const root = createRoot(host);
  const render = (faction: string, visibleBarbarians: boolean) => act(() => root.render(createElement(LensSwitcher, {
    lens: 'threat', onChange: () => {}, sites: [], onInspect: () => {}, settlerId: null,
    players: [{ id: 0, faction } as PlayerPublic], me: 0, empty: true, visibleBarbarians,
  })));
  try {
    render('Celts', true);
    expect(host.querySelector('.lens-empty')?.textContent).toBe('Nothing in sight can attack you next turn. Barbarians never attack you.');
    render('Celts', false);
    expect(host.querySelector('.lens-empty')?.textContent).toBe('No known enemy military can reach any tile next turn.');
    render('India', true);
    expect(host.querySelector('.lens-empty')?.textContent).toBe('No known enemy military can reach any tile next turn.');
  } finally { act(() => root.unmount()); host.remove(); }
});

it('opens the phone chooser, selects and closes, restores focus, and follows viewport changes', () => {
  const media = Object.assign(new EventTarget(), { matches: true }) as MediaQueryList;
  const matchMedia = vi.spyOn(window, 'matchMedia').mockReturnValue(media);
  const host = document.body.appendChild(document.createElement('div'));
  const root = createRoot(host);
  const render = (lens: MapLens) => root.render(createElement(LensSwitcher, {
    lens, onChange: render, sites: [], onInspect: () => {}, settlerId: null, players: [],
  }));
  try {
    act(() => render('default'));
    const toggle = host.querySelector<HTMLButtonElement>('.lens-toggle')!;
    const content = host.querySelector<HTMLDivElement>('.lens-content')!;
    expect(toggle.textContent).toBe('Lens: Default');
    expect(content.hidden).toBe(true);
    act(() => toggle.click());
    expect(content.hidden).toBe(false);
    expect(host.querySelector('[aria-checked="true"]')).toBe(document.activeElement);
    const settle = [...host.querySelectorAll<HTMLButtonElement>('[role="radio"]')].find((b) => b.textContent === 'Settle')!;
    act(() => settle.click());
    // With no Settler selected the lens still draws from the capital, so the toggle needs no hint.
    expect(toggle.textContent).toBe('Lens: Settle');
    expect(content.hidden).toBe(true);
    expect(document.activeElement).toBe(toggle);
    act(() => toggle.click());
    act(() => settle.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', bubbles: true, cancelable: true })));
    expect(content.hidden).toBe(true);
    expect(document.activeElement).toBe(toggle);
    act(() => toggle.click());
    act(() => document.body.dispatchEvent(new PointerEvent('pointerdown', { bubbles: true })));
    expect(content.hidden).toBe(true);
    act(() => { Object.assign(media, { matches: false }); media.dispatchEvent(new Event('change')); });
    expect(host.querySelector('.lens-toggle')).toBeNull();
    expect(content.hidden).toBe(false);
    expect(settle.getAttribute('aria-checked')).toBe('true');
  } finally {
    act(() => root.unmount()); host.remove(); matchMedia.mockRestore();
  }
});

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
    expect(toggle.textContent).toBe('Lens: Default');
    expect(host.querySelector('.map-lenses')!.getAttribute('data-compact')).toBe('true');
    expect(host.querySelector<HTMLDivElement>('.lens-content')!.hidden).toBe(true);
    act(() => toggle.click());
    expect(host.querySelector<HTMLDivElement>('.lens-content')!.hidden).toBe(false);
    const threat = [...host.querySelectorAll<HTMLButtonElement>('[role="radio"]')].find((b) => b.textContent === 'Threat')!;
    act(() => threat.click());
    expect(lens).toBe('threat');
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

it('Resources key counts your strategic deposits from the supply and shows the ring shapes in one row', () => {
  const host = document.body.appendChild(document.createElement('div'));
  const root = createRoot(host);
  const supply = [
    { resource: 'Iron', revealed: true, revealed_by: 'IronWorking', supplied: { total: 5, parts: [['Iron near Rome', 2], ['Iron near Ostia, with a Mine', 3]] }, used: { total: 3, parts: [['Forge in Rome', 1], ['Pikeman ×2', 2]] }, idle: [], held: [], spare: 2 },
    { resource: 'Coal', revealed: false, revealed_by: 'IronWorking', supplied: { total: 0, parts: [] }, used: { total: 0, parts: [] }, idle: [], held: [], spare: 0 },
  ] as unknown as SupplyView[];
  try {
    setOverlay('resources', true);
    act(() => root.render(createElement(LensSwitcher, { lens: 'default', onChange: () => {}, sites: [], onInspect: () => {}, settlerId: null, players: [], supply })));
    expect([...host.querySelectorAll('[role="checkbox"]')].map((b) => b.getAttribute('data-overlay'))).toContain('resources');
    expect([...host.querySelectorAll('[role="radio"]')].map((b) => b.textContent)).not.toContain('Resources');
    const items = [...host.querySelectorAll('.lens-supply li')].map((li) => li.textContent?.trim());
    expect(items[0]).toContain('Iron: 2 deposits in your borders · 3 of 5 Iron at work');
    expect(items[0]).toContain('Used for');
    // A hidden resource is not a supply line: what reveals it is in the Strategic tip.
    expect(items).toHaveLength(1);
    expect(host.querySelector('.lens-resource-key')?.textContent).toMatch(/Rings.*Yours.*Unclaimed.*Foreign/);
    // Iron is supplied here, so nothing is lacked and the key has no "You lack it" mark.
    expect(host.querySelector('.ring-wanted')).toBeNull();
  } finally {
    act(() => root.unmount()); host.remove();
  }
});

it('on phones the Resources key stays on screen in the legend row, not inside the scrolling chooser', () => {
  const matchMedia = vi.spyOn(window, 'matchMedia').mockReturnValue(Object.assign(new EventTarget(), { matches: true }) as MediaQueryList);
  const host = document.body.appendChild(document.createElement('div'));
  host.className = 'game';
  const root = createRoot(host);
  try {
    act(() => setOverlay('resources', true));
    act(() => root.render(createElement(LensSwitcher, { lens: 'default', onChange: () => {}, sites: [], onInspect: () => {}, settlerId: null, players: [], supply: [] })));
    expect(host.querySelector('.settle-persistent-legend .lens-resource-key')?.textContent).toMatch(/Yours.*Foreign/);
    // Shown once: the chooser sheet keeps the lens text and supply lines but not a second key.
    expect(host.querySelectorAll('.lens-resource-key')).toHaveLength(1);
    expect(host.style.getPropertyValue('--settle-legend-height')).not.toBe('');
    act(() => setOverlay('resources', false));
    expect(host.querySelector('.settle-persistent-legend')).toBeNull();
    expect(host.style.getPropertyValue('--settle-legend-height')).toBe('');
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

it('has two labelled groups: three overlay checkboxes with their keys, and five exclusive lenses', () => {
  const m = mount();
  try {
    const groups = [...m.host.querySelectorAll('.lens-group-label')].map((g) => g.firstChild?.textContent?.trim());
    expect(groups).toEqual(['Overlays', 'Lens']);
    expect([...m.host.querySelectorAll('[role="checkbox"]')].map((b) => b.textContent)).toEqual(['YieldsY', 'ResourcesO', 'BordersN']);
    expect([...m.host.querySelectorAll('[role="checkbox"]')].map((b) => b.getAttribute('aria-keyshortcuts'))).toEqual(['Y', 'O', 'N']);
    expect([...m.host.querySelectorAll('[role="radio"]')].map((b) => b.textContent)).toEqual(['Default', 'Settle', 'Movement', 'Threat', 'Discovery']);
    expect(m.host.querySelectorAll('[role="checkbox"][aria-checked="true"]')).toHaveLength(0);
    expect(m.host.querySelectorAll('[role="radio"][aria-checked="true"]')).toHaveLength(1);
    expect(m.host.querySelector('.lens-legend')).toBeNull(); // nothing to explain until something is on
  } finally { m.done(); }
});

it('overlays toggle independently of the lens, in both directions, and show their legend while on', () => {
  const m = mount();
  try {
    act(() => checkbox(m.host, 'Yields').click());
    expect(checkbox(m.host, 'Yields').getAttribute('aria-checked')).toBe('true');
    expect(getOverlays().yields).toBe(true);
    expect(radio(m.host, 'Default').getAttribute('aria-checked')).toBe('true'); // the lens did not move
    expect(m.host.querySelector('.lens-group-label')?.textContent).toContain('1 icon per point · a number from 5');
    expect(m.host.querySelector('.overlay-legend')).toBeNull(); // Yields has no legend block: it sits in the group label
    expect(m.host.querySelectorAll('.lens-yield-key > *')).toHaveLength(4);
    act(() => radio(m.host, 'Threat').click());
    expect(m.lens).toBe('threat');
    expect(radio(m.host, 'Threat').getAttribute('aria-checked')).toBe('true');
    expect(radio(m.host, 'Default').getAttribute('aria-checked')).toBe('false'); // one lens at a time
    expect(checkbox(m.host, 'Yields').getAttribute('aria-checked')).toBe('true'); // the overlay stayed on
    expect(m.host.querySelector('.lens-group-label')?.textContent).toContain('1 icon per point');
    act(() => checkbox(m.host, 'Yields').click());
    expect(getOverlays()).toEqual(DEFAULT_OVERLAYS);
    expect(m.host.querySelector('.lens-group-label')?.textContent).toContain('stay on over any lens');
    expect(radio(m.host, 'Threat').getAttribute('aria-checked')).toBe('true');
  } finally { m.done(); }
});

it('overlays stay on when a lens switches itself on (a selected Settler) and off again', () => {
  setOverlay('yields', true); setOverlay('borders', true);
  const m = mount();
  try {
    m.lens = 'settle';
    expect([...m.host.querySelectorAll('[role="checkbox"][aria-checked="true"]')].map((b) => b.getAttribute('data-overlay'))).toEqual(['yields', 'borders']);
    expect(m.host.querySelector('.settle-legend')).not.toBeNull();
    expect(m.host.querySelector('.lens-group-label')?.textContent).toContain('1 icon per point');
    m.lens = 'default';
    expect([...m.host.querySelectorAll('[role="checkbox"][aria-checked="true"]')]).toHaveLength(2);
  } finally { m.done(); }
});

it('arrow keys step through the five lenses only, wrapping, and leave the overlays alone', () => {
  setOverlay('resources', true);
  const m = mount();
  try {
    const group = m.host.querySelector('[role="radiogroup"]')!;
    const press = (key: string) => act(() => { group.dispatchEvent(new KeyboardEvent('keydown', { key, bubbles: true, cancelable: true })); });
    press('ArrowRight'); expect(m.lens).toBe('settle');
    press('End'); expect(m.lens).toBe('discovery');
    press('ArrowRight'); expect(m.lens).toBe('default');
    press('ArrowLeft'); expect(m.lens).toBe('discovery');
    expect(getOverlays()).toEqual({ ...DEFAULT_OVERLAYS, resources: true });
  } finally { m.done(); }
});

it('says when an overlay has nothing to mark yet', () => {
  setOverlay('borders', true); setOverlay('resources', true);
  const m = mount({ overlayEmpty: { borders: true, resources: true } });
  try {
    expect([...m.host.querySelectorAll('.lens-empty')].map((p) => p.textContent)).toEqual(['No known resources in explored land yet.', 'No known borders yet.']);
  } finally { m.done(); }
});

it('the compact toggle names the lens and the overlays that are on', () => {
  const media = Object.assign(new EventTarget(), { matches: false }) as MediaQueryList;
  const matchMedia = vi.spyOn(window, 'matchMedia').mockReturnValue(media);
  const m = mount({ collapsed: true });
  try {
    expect(m.host.querySelector('.lens-toggle')!.textContent).toBe('Lens: Default');
    act(() => setOverlay('yields', true));
    expect(m.host.querySelector('.lens-toggle')!.textContent).toBe('Lens: Default · Yields');
    act(() => setOverlay('borders', true));
    expect(m.host.querySelector('.lens-toggle')!.textContent).toBe('Lens: Default · Yields, Borders');
  } finally { m.done(); matchMedia.mockRestore(); }
});

it.each([true, false])('overlay keys open on request in a short window, and stay open in a tall one (short: %s)', (short) => {
  const matchMedia = vi.spyOn(window, 'matchMedia').mockImplementation((query) => Object.assign(new EventTarget(), { matches: short && query === '(max-height: 700px)' }) as MediaQueryList);
  setOverlay('resources', true); setOverlay('borders', true);
  const m = mount({ players: [{ id: 0, team: 0, faction: 'Celts' }, { id: 1, team: 1, faction: 'Sparta' }] as never, me: 0 });
  try {
    const toggle = () => m.host.querySelector<HTMLButtonElement>('.lens-key-toggle');
    if (short) {
      expect(m.host.querySelector('.overlay-legend')).toBeNull();
      expect(toggle()!.getAttribute('aria-expanded')).toBe('false');
      act(() => toggle()!.click());
      expect(toggle()!.getAttribute('aria-expanded')).toBe('true');
    } else expect(toggle()).toBeNull();
    expect(m.host.querySelector('.overlay-legend .lens-resource-key')?.textContent).toMatch(/Rings.*Yours.*Unclaimed.*Foreign/);
    expect([...m.host.querySelectorAll('.lens-owner-key span')].map((s) => s.getAttribute('aria-label'))).toEqual(['1: You', `2: ${factionName('Sparta')}`]);
  } finally { m.done(); matchMedia.mockRestore(); }
});

it('the Resources key adds a "You lack it" mark when a revealed strategic resource has no supply', () => {
  const host = document.body.appendChild(document.createElement('div'));
  const root = createRoot(host);
  const supply = [
    { resource: 'Iron', revealed: true, revealed_by: 'IronWorking', supplied: { total: 0, parts: [] }, used: { total: 0, parts: [] }, idle: [], held: [], spare: 0 },
  ] as unknown as SupplyView[];
  try {
    setOverlay('resources', true);
    act(() => root.render(createElement(LensSwitcher, { lens: 'default', onChange: () => {}, sites: [], onInspect: () => {}, settlerId: null, players: [], supply })));
    expect(host.querySelector('.ring-wanted')?.parentElement?.textContent).toContain('You lack it');
  } finally {
    act(() => root.unmount()); host.remove();
  }
});
