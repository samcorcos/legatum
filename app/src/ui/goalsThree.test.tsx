// @vitest-environment happy-dom
// Faction goals increment 3 in the UI: the Heir's unit panel and disband confirmation, the held
// goal's token in its holder's race row, the capture line on a tile, and Germany's Ruhr Foundry goal.
import { act } from 'react';
import { createRoot, type Root } from 'react-dom/client';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { FixtureClient } from '../test/fixtureEngine';
import { useGame } from '../store';
import { VpTrack } from './VpTrack';
import { UnitPanel } from './UnitPanel';
import { TileCard } from './TileCard';
import { heirLines } from './HeirNote';
import { ownHeirLine } from '../vpModel';
import type { PlayerView, UnitView } from '../types';

let host: HTMLElement, root: Root, view: PlayerView, client: FixtureClient;
const byzantium = () => view.players.find((p) => p.faction === 'Byzantium')!;
const rival = () => view.players.find((p) => p.team !== view.team && p.id !== byzantium().id)!;
const heir = (owner: number, of = byzantium().id): UnitView => ({
  ...view.units.find((u) => u.owner === view.me)!, id: 7777, kind: 'HeirToTheThrone', owner, heir_of: of, military: false, hp: 100, strength: 0,
  pos: view.units.find((u) => u.owner === view.me)!.pos,
});

beforeEach(async () => {
  vi.stubGlobal('IS_REACT_ACT_ENVIRONMENT', true);
  vi.useFakeTimers();
  useGame.setState(useGame.getInitialState(), true);
  client = new FixtureClient();
  await client.newGame(await client.standardSetup('42', 'India', 'Byzantium', 'Ada', 'Bo'));
  view = structuredClone(client.view(0));
  host = document.body.appendChild(document.createElement('div'));
  root = createRoot(host);
});
afterEach(() => { act(() => root.unmount()); host.remove(); document.querySelectorAll('.vhc-pop').forEach((n) => n.remove()); vi.useRealTimers(); vi.unstubAllGlobals(); });

const atWarWith = (team: number) => { for (const r of view.diplomacy.relations) if (r.a === team || r.b === team) r.state = 'War'; };

describe('the Heir’s unit panel text', () => {
  it('says whose he is, who holds him and what a capture does, from the viewer’s side', () => {
    // A third empire's Byzantium, held by a rival: capturing him is a gain.
    const others = view.players.filter((p) => p.team !== view.team);
    const atPeace = heirLines(heir(others[0].id, others[1].id), view).map((l) => l.text);
    expect(atPeace[0]).toMatch(/^Heir of .*: its holder scores 2 VP$/);
    expect(atPeace[1]).toMatch(/^Held by .*: at Peace you cannot capture him; declare War, then use a military unit \(\+2 VP to you, −2 to them\)$/);
    atWarWith(others[0].team);
    expect(heirLines(heir(others[0].id, others[1].id), view)[1].text).toMatch(/^Held by .*: capture him with a military unit \(\+2 VP to you, −2 to them\)$/);
    // My team's Byzantium lost him: retaking returns the points; a barbarian hold scores nobody.
    const lost = heirLines(heir(rival().id), view);
    expect(lost[1].text).toContain('retake him');
    expect(lost[1].tone).toBe('bad');
    expect(heirLines(heir(255), view)[1].text).toMatch(/^Held by barbarians: nobody scores/);
    expect(heirLines(heir(view.me), view)[1].text).toContain('You hold him: +2 VP');
    expect(heirLines({ ...heir(view.me), kind: 'Spearman' }, view)).toEqual([]);
  });

  it('Disband on a Heir to the Throne asks first and says it forfeits the 2 VP; Keep him closes it', () => {
    const h = heir(view.me, view.me);
    view.units.push(h);
    useGame.setState({ client, views: [view], humans: [0], orders: [[]], errors: [[]], seat: 0, selection: { kind: 'unit', id: h.id } });
    act(() => root.render(<UnitPanel unit={h} view={view} />));
    expect(host.querySelector('[data-testid="heir-note"]')!.textContent).toContain('Heir of your empire: its holder scores 2 VP');
    expect(host.querySelector('[data-testid="heir-disband-confirm"]')).toBeNull();
    const disband = [...host.querySelectorAll('button')].find((b) => b.textContent === 'Disband')!;
    act(() => disband.click());
    const confirm = host.querySelector('[data-testid="heir-disband-confirm"]')!;
    expect(confirm.textContent).toMatch(/forfeit 2 VP/);
    expect(useGame.getState().orders[0]).toEqual([]);
    act(() => [...confirm.querySelectorAll('button')].find((b) => b.textContent === 'Keep him')!.click());
    expect(host.querySelector('[data-testid="heir-disband-confirm"]')).toBeNull();
    expect(useGame.getState().orders[0]).toEqual([]);
    const again = [...host.querySelectorAll('button')].find((b) => b.textContent === 'Disband')!;
    act(() => again.click());
    act(() => [...host.querySelectorAll('button')].find((b) => b.textContent === 'Disband him')!.click());
    expect(useGame.getState().orders[0].some((o) => o.type === 'DisbandUnit' && o.unit === h.id)).toBe(true);
  });
});

describe('the capture line on a tile', () => {
  it('offers "Capture the Heir (+2 VP)" under a selected military unit, and only for a rival-held Heir to the Throne', () => {
    const spearman = view.units.find((u) => u.owner === view.me && u.military)!;
    rival().faction = 'Byzantium'; // This case exercises the Byzantine holder's label, independent of the AI roster.
    const h = heir(rival().id);
    h.pos = { q: spearman.pos.q + 1, r: spearman.pos.r };
    atWarWith(rival().team);
    view.units.push(h);
    useGame.setState({ selection: { kind: 'unit', id: spearman.id } });
    const idx = view.tiles.length;
    expect(idx).toBeGreaterThan(0);
    act(() => root.render(<TileCard view={view} hex={h.pos} />));
    expect(host.querySelector('[data-testid="heir-capture"]')?.textContent).toBe('Capture the Heir (+2 VP)');
    useGame.setState({ selection: null });
    act(() => root.render(<TileCard view={{ ...view }} hex={h.pos} />));
    expect(host.querySelector('[data-testid="heir-capture"]')).toBeNull();
    expect(host.querySelector('[data-testid="heir-tile"]')?.textContent).toContain('Held by Byzantium: Be at War, then capture him');
  });
});

describe('the race panel', () => {
  const render = () => {
    useGame.setState({ client, views: [view], humans: [0], orders: [[]], errors: [[]], seat: 0 });
    act(() => root.render(<div className="game"><div className="seatbar" /><VpTrack /></div>));
  };

  it('puts Byzantium’s goal token in the holder’s row, wearing Byzantium’s crest, outlined (it can be captured)', () => {
    rival().vp_sources.push([{ kind: 'FactionGoal', of: byzantium().id }, 2]);
    render();
    const tok = host.querySelector<HTMLElement>(`.panel.vp [data-source="goal-${byzantium().id}"][data-owner="${rival().id}"]`)!;
    expect(tok).not.toBeNull();
    expect(tok.classList.contains('live')).toBe(true);
    expect(tok.getAttribute('aria-label')).toMatch(/held: can be captured/);
    // Not in Byzantium's own row, and not a pending crest there either.
    expect(host.querySelector(`.panel.vp [data-source="goal-${byzantium().id}"][data-owner="${byzantium().id}"]`)).toBeNull();
    act(() => tok.focus());
    const card = document.querySelector('.vhc-pop')!;
    expect(card.querySelector('h3')!.textContent).toBe('The Heir to the Throne');
    expect(card.textContent).toContain('Held goal');
    expect(card.querySelector('.vh-held')!.textContent).toContain('holds the Heir');
  });

  it('a click on the held token of a visible Heir to the Throne pans the map to him, and the card says so', () => {
    rival().vp_sources.push([{ kind: 'FactionGoal', of: byzantium().id }, 2]);
    view.units.push(heir(rival().id));
    const jumpTo = vi.fn();
    render();
    useGame.setState({ jumpTo });
    const tok = host.querySelector<HTMLElement>(`.panel.vp [data-source="goal-${byzantium().id}"][data-owner="${rival().id}"]`)!;
    expect(tok.getAttribute('aria-label')).toMatch(/show him on the map/);
    act(() => { tok.click(); });
    expect(jumpTo).toHaveBeenCalledWith(heir(rival().id).pos);
    expect(document.querySelector('[data-testid="heir-locate-hint"]')).not.toBeNull();
  });

  it('Byzantium’s own goal line says who holds him, or that nobody does', () => {
    const asByzantium = structuredClone(client.view(1));
    const mine = asByzantium.players.find((p) => p.id === asByzantium.me)!;
    expect(mine.faction).toBe('Byzantium');
    expect(ownHeirLine(asByzantium)).toBe('The Heir: nobody holds him (not trained, or lost)');
    mine.vp_sources.push([{ kind: 'FactionGoal', of: mine.id }, 2]);
    expect(ownHeirLine(asByzantium)).toBe('The Heir: alive, held by you');
    mine.vp_sources = [];
    const other = asByzantium.players.find((p) => p.team !== asByzantium.team)!;
    other.vp_sources.push([{ kind: 'FactionGoal', of: mine.id }, 2]);
    expect(ownHeirLine(asByzantium)).toMatch(/^The Heir: alive, held by .+/);
    expect(ownHeirLine(view)).toBeNull();
  });
});
