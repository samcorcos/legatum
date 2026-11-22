// @vitest-environment happy-dom
// Faction goals increment 5 in the UI: the Yam Relay Station's token in the race panel, its goal-card
// note, the open-road note under a unit, the build card's words, and Peace's border line.
import { act } from 'react';
import { createRoot, type Root } from 'react-dom/client';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { FixtureClient } from '../test/fixtureEngine';
import { useGame } from '../store';
import { VpTrack } from './VpTrack';
import { UnitPanel } from './UnitPanel';
import { hallCardText, hallStands } from './goalText';
import { peaceBorders } from './DiplomacyPanel';
import { buildVpModel } from '../vpModel';
import { canEnter } from '../diplomacy';
import { tileIndex } from '../hex';
import type { PlayerView } from '../types';

let host: HTMLElement, root: Root, view: PlayerView, client: FixtureClient;
const glass = () => view.players.find((p) => p.faction === 'Mongols')!;
const rival = () => view.players.find((p) => p.team !== view.team && p.id !== glass().id)!;
const hallCity = () => view.cities.find((c) => c.owner === view.me)!;

beforeEach(async () => {
  vi.stubGlobal('IS_REACT_ACT_ENVIRONMENT', true);
  vi.useFakeTimers();
  useGame.setState(useGame.getInitialState(), true);
  client = new FixtureClient();
  await client.newGame(await client.standardSetup('42', 'India', 'Mongols', 'Ada', 'Bo'));
  client.endTurn({});
  view = structuredClone(client.view(0));
  host = document.body.appendChild(document.createElement('div'));
  root = createRoot(host);
});
afterEach(() => { act(() => root.unmount()); host.remove(); document.querySelectorAll('.vhc-pop').forEach((n) => n.remove()); vi.useRealTimers(); vi.unstubAllGlobals(); });

/** The Mongols's Station stands in my city, so I hold it. */
const iHold = () => {
  view.courier_halls = [{ of: glass().id, holder: view.me, city: hallCity().id }];
  view.players.find((p) => p.id === view.me)!.vp_sources.push([{ kind: 'FactionGoal', of: glass().id }, 2]);
};

describe('the Station in the race panel', () => {
  const render = () => {
    useGame.setState({ client, views: [view], humans: [0], orders: [[]], errors: [[]], seat: 0 });
    act(() => root.render(<div className="game"><div className="seatbar" /><VpTrack /></div>));
  };

  it('puts the goal token in the holder’s row wearing the Station’s own icon, and says where the Station stands', () => {
    iHold();
    render();
    const tok = host.querySelector<HTMLElement>(`.panel.vp [data-source="goal-${glass().id}"][data-owner="${view.me}"]`)!;
    expect(tok).not.toBeNull();
    expect(tok.getAttribute('aria-label')).toMatch(/held: can be captured.*show it on the map/);
    act(() => tok.focus());
    const card = document.querySelector('.vhc-pop')!;
    expect(card.querySelector('h3')!.textContent).toBe('The Yam Network');
    expect(card.querySelector('.vh-held')!.textContent).toContain(`You hold the Yam Relay Station in ${hallCity().name} · in view`);
    expect(card.textContent).toContain('whoever holds its tile scores');
  });

  it('a click on the token shows the Station’s city on the map', () => {
    iHold();
    const jumpTo = vi.fn();
    render();
    useGame.setState({ jumpTo });
    act(() => { host.querySelector<HTMLElement>(`.panel.vp [data-source="goal-${glass().id}"][data-owner="${view.me}"]`)!.click(); });
    expect(jumpTo).toHaveBeenCalledWith(hallCity().pos);
  });

  it('says last seen for a remembered city, and that nobody holds an unbuilt or sacked Station', () => {
    iHold();
    hallCity().visible = false;
    expect(buildVpModel(view).goals.find((g) => g.only === glass().id)!.holdNote).toBe(`You hold the Yam Relay Station in ${hallCity().name} · last seen`);
    view.courier_halls = [];
    view.players.find((p) => p.id === view.me)!.vp_sources = [];
    const g = buildVpModel(view).goals.find((x) => x.only === glass().id)!;
    expect(g.holdNote).toBe('Nobody holds the Yam Relay Station: it is not built, its tile is unclaimed, or it was destroyed');
    expect(g.heldAt).toBeUndefined();
    expect(g.own).toBe('open');
  });

  it('follows the Station to a captor and keeps its card in the holder’s row when the Mongols has fallen', () => {
    view.courier_halls = [{ of: glass().id, holder: rival().id, city: hallCity().id }];
    rival().vp_sources.push([{ kind: 'FactionGoal', of: glass().id }, 2]);
    glass().alive = false;
    const m = buildVpModel(view);
    const g = m.goals.find((x) => x.only === glass().id)!;
    expect(g.owners).toEqual([rival().id]);
    const row = m.rows.find((r) => r.members.some((e) => e.id === rival().id))!;
    expect(row.tokens.find((t) => t.source.id === g.id)?.of?.faction).toBe('Mongols');
  });
});

describe('the open road under a unit', () => {
  it('notes it while standing on a Station holder’s land at Peace, and not on your own or an ordinary rival’s', () => {
    const unit = view.units.find((u) => u.owner === view.me && u.military)!;
    const at = tileIndex(unit.pos, view.width, view.height);
    view.tiles[at].owner = rival().id;
    useGame.setState({ client, views: [view], humans: [0], orders: [[]], errors: [[]], seat: 0, selection: { kind: 'unit', id: unit.id } });
    act(() => root.render(<UnitPanel unit={unit} view={view} />));
    expect(host.querySelector('[data-testid="open-road-note"]')).toBeNull();
    expect(host.textContent).toContain('Closed foreign territory');
    view.courier_halls = [{ of: glass().id, holder: rival().id, city: hallCity().id }];
    act(() => root.render(<UnitPanel unit={unit} view={{ ...view }} />));
    expect(host.querySelector('[data-testid="open-road-note"]')!.textContent).toContain('lets every empire cross their land at Peace');
    expect(host.textContent).not.toContain('Closed foreign territory');
    expect(canEnter(view, unit.kind, rival().id)).toBe(true);
  });
});

describe('the Station’s words', () => {
  it('the build card says what it scores, what it opens and the one-per-empire rule; a standing Station reads Built', () => {
    expect(hallCardText()).toBe('+2 VP while you hold it · every empire crosses all your land at Peace · one per empire, rebuilt only if destroyed');
    expect(hallStands('The Yam Relay Station already stands in Far Mile')).toBe(true);
    expect(hallStands('The Yam Relay Station stands in Far Mile; retake the city instead')).toBe(false);
    expect(hallStands('The Yam Relay Station is already being built in Far Mile')).toBe(false);
  });

  it('Peace says whose borders the Station opens', () => {
    const relation = view.diplomacy.relations.find((r) => r.a === view.team || r.b === view.team)!;
    const other = view.players.find((p) => p.team === (relation.a === view.team ? relation.b : relation.a))!;
    expect(peaceBorders(relation, view)).toBe('closed borders');
    view.courier_halls = [{ of: glass().id, holder: other.id, city: hallCity().id }];
    expect(peaceBorders(relation, view)).toBe('their land is open to you (their Yam Relay Station), yours stays closed');
    view.courier_halls = [{ of: glass().id, holder: view.me, city: hallCity().id }];
    expect(peaceBorders(relation, view)).toBe('your land is open to them (your Yam Relay Station), theirs stays closed');
    view.courier_halls = [{ of: glass().id, holder: view.me, city: hallCity().id }, { of: other.id, holder: other.id, city: 99 }];
    expect(peaceBorders(relation, view)).toBe('borders open both ways (Yam Relay Stations)');
  });
});
