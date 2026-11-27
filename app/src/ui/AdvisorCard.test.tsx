// @vitest-environment jsdom
import { act } from 'react';
import { cleanup, fireEvent, render, screen } from '@testing-library/react';
import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import { FixtureClient } from '../test/fixtureEngine';
import { itemKey } from '../data';
import { useGame } from '../store';
import type { PlayerView, UnitView } from '../types';
import { Advisor, targetElement } from './Advisor';
import { useAdvisor } from './advisorModel';
import { CityPanel } from './CityPanel';
import { UnitPanel } from './UnitPanel';
import { TechTree } from './TechTree';

let view: PlayerView;
let client: FixtureClient;
beforeEach(async () => {
  vi.stubGlobal('matchMedia', () => ({ matches: false, addEventListener: () => {}, removeEventListener: () => {} }));
  vi.stubGlobal('ResizeObserver', class { observe() {} disconnect() {} });
  vi.spyOn(HTMLElement.prototype, 'getClientRects').mockImplementation(() => [new DOMRect(0, 0, 100, 30)] as unknown as DOMRectList);
  client = new FixtureClient();
  await client.newGame(await client.standardSetup('4545', 'Athens', 'India', 'Ada', 'Bo'));
  view = client.view(0);
  act(() => {
    useGame.setState(useGame.getInitialState(), true);
    useGame.setState({ client, views: [view], seat: 0, humans: [view.me], orders: [[]], errors: [[]], ready: [false] });
    useAdvisor.setState({ enabled: true, seen: [], skipped: [], skippedTurn: -1 });
  });
});
afterEach(() => { cleanup(); vi.restoreAllMocks(); vi.unstubAllGlobals(); });

const control = (name: 'found' | 'fortify' | 'explore' | 'ranged' | 'build', value?: string) => targetElement({ kind: 'control', name, value });

it('finds the queue strip and each current or future add-to-queue control', () => {
  const mounted = render(<dialog className="tech-modal" open>
    <div className="research-queue">Research queue</div>
    <article className="era-card" data-tech="Masonry"><button data-testid="queue-add">+ Queue</button></article>
  </dialog>);
  const queueTarget = (value: string) => targetElement({ kind: 'control', name: 'queueTech', value });
  expect(targetElement({ kind: 'control', name: 'researchQueue' })?.className).toBe('research-queue');
  expect(queueTarget('Masonry')?.getAttribute('data-testid')).toBe('queue-add');
  expect(queueTarget('Writing')).toBeNull();
  mounted.rerender(<dialog className="tech-modal tree-mode" open><div className="tree-node" data-tech="Masonry">Masonry</div></dialog>);
  expect(queueTarget('Masonry')).toBeNull();
});

it.each([false, true])('targets real tree queue buttons and the later-era confirmation (touch: %s)', (touch) => {
  vi.stubGlobal('matchMedia', () => ({ matches: touch, addEventListener: () => {}, removeEventListener: () => {} }));
  const onQueue = vi.fn();
  render(<dialog className="tech-modal tree-mode" open><button data-testid="research-view-era">This era</button>
    <TechTree view={view} faction={view.players.find((p) => p.id === view.me)!.faction} queue={[]} schedule={[]} goal={null} locked={false}
      onResearchNow={() => {}} onQueue={onQueue} onRemove={() => {}} onGoal={() => {}} /></dialog>);
  const tech = view.research.available[0].tech;
  const pick = (value: string) => targetElement({ kind: 'control', name: 'queueTech', value });
  expect(pick(tech)?.getAttribute('data-testid')).toBe('tree-queue');
  expect(pick(tech)?.closest('.tree-node')).toBeNull();
  fireEvent.click(pick(tech)!);
  expect(onQueue).toHaveBeenCalledWith(tech);
  onQueue.mockClear();
  const later = document.querySelector<HTMLElement>('.tree-node.locked');
  expect(later).toBeTruthy();
  const laterTech = later!.dataset.tech!;
  fireEvent.click(pick(laterTech)!);
  expect(onQueue).not.toHaveBeenCalled();
  expect(screen.getByTestId('tree-ask').textContent).toContain('Finish');
  expect(pick(laterTech)).toBe(screen.getByTestId('tree-ask-queue'));
  fireEvent.click(pick(laterTech)!);
  expect(onQueue).toHaveBeenCalledWith(laterTech);
});

it('finds real unit action buttons on mounted panels', () => {
  const original = view.units.find((u) => u.owner === view.me && u.military)!;
  const unit: UnitView = { ...original, kind: 'Spearman', military: true, ranged: 3, range: 2 };
  const mounted = render(<div className="unit-panel"><UnitPanel unit={unit} view={view} /></div>);
  expect(control('fortify')?.textContent).toContain('Fortify');
  expect(control('ranged')?.textContent).toContain('Ranged Attack');
  mounted.rerender(<div className="unit-panel"><UnitPanel unit={{ ...unit, kind: 'Settler', military: false, ranged: 0 }} view={view} /></div>);
  expect(control('found')?.textContent).toContain('Found City');
  expect(control('fortify')?.textContent).toContain('Wait');
  mounted.rerender(<div className="unit-panel"><UnitPanel unit={{ ...unit, kind: 'Scout', military: false, ranged: 0 }} view={view} /></div>);
  expect(control('explore')?.textContent).toContain('Auto-explore');
});

it('finds Build on the chosen item in a mounted city panel', () => {
  const city = view.cities.find((c) => c.owner === view.me)!;
  const item = city.detail!.options[0].item;
  render(<CityPanel city={city} view={view} />);
  expect(control('build', itemKey(item))).toBeInstanceOf(HTMLButtonElement);
  expect(control('build', itemKey(item))?.getAttribute('aria-label')).toMatch(/^Build /);
});

it('points at the "Also" list when the build it recommends is folded inside it', () => {
  document.body.innerHTML = '<div class="city-panel"><details class="unit-line"><summary>Also: Spearman</summary><div class="production-choice" data-item="u:Spearman"><button class="prod">Build</button></div></details></div>';
  expect(control('build', 'u:Spearman')?.tagName).toBe('SUMMARY');
  document.querySelector('details')!.open = true;
  expect(control('build', 'u:Spearman')?.className).toBe('prod');
  document.body.innerHTML = '';
});

it('points at the recommended technology: its tree node first, else its era card Research now', () => {
  const tech = view.research.available[0].tech;
  const pick = () => targetElement({ kind: 'control', name: 'tech', value: tech });
  const tree = render(<dialog className="tech-modal" open><TechTree view={view} faction={view.players.find((p) => p.id === view.me)!.faction} queue={[]} schedule={[]} goal={null} locked={false}
    onResearchNow={() => {}} onQueue={() => {}} onRemove={() => {}} onGoal={() => {}} /></dialog>);
  expect(pick()?.classList.contains('tree-node')).toBe(true);
  expect(pick()?.dataset.tech).toBe(tech);
  tree.unmount();
  document.body.innerHTML = `<dialog class="tech-modal" open><div class="era-card" data-tech="${tech}"><button data-testid="research-now">Research now</button></div></dialog>`;
  expect(pick()?.getAttribute('data-testid')).toBe('research-now');
  document.body.innerHTML = '';
});

it('turns off, moves on, and pauses for busy, ready and blocking overlays', async () => {
  render(<div className="game"><Advisor /></div>);
  expect(screen.getByTestId('advisor').dataset.advisorId).toBe('welcome');
  act(() => useAdvisor.getState().markSeen('welcome'));
  const first = screen.getByTestId('advisor').dataset.advisorId;
  fireEvent.click(screen.getByRole('button', { name: 'Not now' }));
  expect(screen.queryByTestId('advisor')?.getAttribute('data-advisor-id')).not.toBe(first);
  const pauses: Record<string, object> = {
    busy: { busy: 'working' }, ready: { ready: [true] },
  };
  for (const patch of Object.values(pauses)) {
    act(() => useGame.setState(patch));
    expect(screen.queryByTestId('advisor')).toBeNull();
    act(() => useGame.setState({ busy: null, ready: [false] }));
    expect(screen.queryByTestId('advisor')).not.toBeNull();
  }
  const overlay = document.body.appendChild(document.createElement('div'));
  overlay.setAttribute('data-game-overlay', '');
  await act(async () => { await Promise.resolve(); });
  expect(screen.queryByTestId('advisor')).toBeNull();
  overlay.remove();
  await act(async () => { await Promise.resolve(); });
  expect(screen.queryByTestId('advisor')).not.toBeNull();
  fireEvent.click(screen.getByRole('button', { name: 'Turn off' }));
  expect(screen.queryByTestId('advisor')).toBeNull();
});

it('counts routine steps the player does, once per turn', () => {
  useAdvisor.setState({ practice: {}, practised: [], practisedTurn: -1 });
  render(<Advisor />);
  const unit = view.units.find((u) => u.owner === view.me)!;
  act(() => useGame.setState({ orders: [[{ type: 'SkipUnit', unit: unit.id }]] }));
  act(() => useGame.setState({ orders: [[{ type: 'SkipUnit', unit: unit.id }, { type: 'SleepUnit', unit: unit.id }]] }));
  expect(useAdvisor.getState().practice).toEqual({ skip: 1, fortify: 1 });
  act(() => useGame.setState({ views: [{ ...view, turn: view.turn + 1 }], orders: [[]] }));
  expect(useAdvisor.getState().practice.endTurn).toBe(1);
});

it('logs a tip when it shows, and Not now puts it away for the turn', () => {
  useAdvisor.setState({ seen: ['welcome', 'report', 'howToWin'], tips: {} });
  act(() => useGame.setState({ views: [{ ...view, turn: 30, economy: { ...view.economy, gold: 400 }, supply: [] }] }));
  render(<Advisor />);
  expect(screen.getByTestId('advisor').dataset.advisorId).toBe('tip:gold');
  expect(useAdvisor.getState().tips.gold).toBe(30);
  // A tip with a ring is dismissed by a quiet Not now; the ring is the call to action.
  fireEvent.click(screen.getByRole('button', { name: 'Not now' }));
  expect(screen.queryByTestId('advisor')?.dataset.advisorId).not.toBe('tip:gold');
});

it('a new game starts the tip log over and counts each kind', () => {
  useAdvisor.setState({ tips: { gold: 40, '#gold': 2 } });
  useAdvisor.getState().noteTip('army', 12);
  expect(useAdvisor.getState().tips).toEqual({ army: 12, '#army': 1 });
  useAdvisor.getState().noteTip('gold', 20);
  expect(useAdvisor.getState().tips).toEqual({ army: 12, '#army': 1, gold: 20, '#gold': 1 });
});

it('rings the city panel\'s Buy a tile button', () => {
  render(<div className="city-panel"><button>Build</button><button>Buy a tile · from 50 Gold</button></div>);
  expect(targetElement({ kind: 'control', name: 'buyTile' })?.textContent).toBe('Buy a tile · from 50 Gold');
});

it('rings the Make an offer button of a city purchase', () => {
  render(<div className="city-panel"><button data-testid="buy-city-offer">Make an offer (400 Gold)</button></div>);
  expect(targetElement({ kind: 'control', name: 'buyCity' })?.textContent).toBe('Make an offer (400 Gold)');
});

it('targets desktop Diplomacy and leaves the phone lesson without a ring', () => {
  const desktop = render(<div className="game"><section className="panel vp"><footer className="rivals-foot"><button>Diplomacy</button></footer></section></div>);
  expect(targetElement({ kind: 'control', name: 'diplomacy' })?.textContent).toBe('Diplomacy');
  desktop.unmount();
  const rival = view.players.find((p) => p.team !== view.team)!;
  const city = view.cities.find((c) => c.owner === view.me)!;
  const unit = view.units.find((u) => u.owner === view.me && u.military)!;
  unit.pos = city.pos;
  unit.occupation_lock = rival.id;
  city.occupation_lock = rival.id;
  view.turn = 5;
  useAdvisor.setState({ seen: ['welcome', 'report', 'howToWin'], skipped: [] });
  render(<><button className="phone-vp-strip">Victory race</button><Advisor /></>);
  expect(screen.getByTestId('advisor').dataset.advisorId).toBe('garrisonLock');
  expect(targetElement({ kind: 'control', name: 'diplomacy' })).toBeNull();
  expect(screen.queryByTestId('advisor-ring')).toBeNull();
});

it('while a tile is being placed, shows only the tip that asked for it', () => {
  useAdvisor.setState({ seen: ['welcome', 'report', 'howToWin'], tips: {} });
  const city = view.cities.find((c) => c.owner === view.me)!;
  act(() => useGame.setState({ views: [{ ...view, turn: 30, economy: { ...view.economy, gold: 400 }, supply: [] }], place: { city: city.id, building: null, intent: 'tile', pick: null } }));
  render(<Advisor />);
  expect(screen.queryByTestId('advisor')).toBeNull();
});
