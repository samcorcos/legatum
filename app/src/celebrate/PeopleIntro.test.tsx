// @vitest-environment happy-dom
import { act } from 'react';
import { createRoot, type Root } from 'react-dom/client';
import { afterEach, beforeAll, beforeEach, describe, expect, it, vi } from 'vitest';
import { FACTION_GOAL, FACTION_INFO, FACTIONS } from '../data';
import { FACTION_SHEETS } from '../factionSheets';
import { introEnabled, useGame } from '../store';
import { FixtureClient } from '../test/fixtureEngine';
import type { PlayerView } from '../types';
import { PeopleIntro } from './PeopleIntro';

let view: PlayerView;
beforeAll(async () => {
  const client = new FixtureClient();
  await client.newGame(await client.standardSetup('42', 'India', 'Byzantium', 'Ada', 'Bo'));
  view = client.view(0);
});

let root: Root, host: HTMLDivElement;
beforeEach(() => {
  vi.stubGlobal('IS_REACT_ACT_ENVIRONMENT', true);
  document.body.innerHTML = '';
  host = document.body.appendChild(document.createElement('div'));
  root = createRoot(host);
});
afterEach(() => { act(() => root.unmount()); useGame.setState({ views: null, introSeats: [] }); localStorage.clear(); vi.unstubAllGlobals(); });

const as = (faction: (typeof FACTIONS)[number], turn = 1): PlayerView => ({ ...view, turn, players: view.players.map((p) => (!(p.id === view.me) ? p : { ...p, faction })) });
const show = (v: PlayerView, intro = true) => act(() => { useGame.setState({ views: [v], seat: 0, introSeats: !(intro) ? [] : [0] }); root.render(<PeopleIntro />); });
const plate = () => document.querySelector<HTMLElement>('[data-testid="people-intro"]');

describe('the people introduction', () => {
  it('names the people, their motto, and the strengths, prices and goal of their faction sheet', () => {
    for (const faction of FACTIONS) {
      show(as(faction));
      const el = plate()!;
      expect(el, faction).not.toBeNull();
      expect(el.dataset.people).toBe(faction);
      expect(el.querySelector('h2')!.textContent).toBe(FACTION_INFO[faction].name);
      expect(el.textContent).toContain(FACTION_INFO[faction].motto);
      const rows = FACTION_SHEETS[faction].pairs.filter((p) => !p.onCard);
      expect(el.querySelectorAll('[data-testid="people-intro-strengths"] .mr').length).toBe(rows.filter((p) => p.strength && p.strengthIcon).length);
      expect(el.querySelectorAll('[data-testid="people-intro-prices"] .mr').length).toBe(rows.filter((p) => p.price && p.priceIcon).length);
      expect(el.querySelector('[data-testid="people-intro-goal"] b')!.textContent).toBe(FACTION_GOAL[faction].name);
      expect(el.querySelectorAll('[data-primary-action]').length, 'one primary').toBe(1);
    }
  });

  it('shows only for a game that has just begun', () => {
    show(as('Rome'), false);
    expect(plate()).toBeNull();
    show(as('Rome', 2));
    expect(plate()).toBeNull();
  });

  it('closes on Begin, Return or Esc, and keeps those keys from the game underneath', () => {
    for (const close of ['click', 'Enter', 'Escape'] as const) {
      show(as('Sparta'));
      const under = vi.fn();
      window.addEventListener('keydown', under);
      act(() => {
        if (!(close === 'click')) { window.dispatchEvent(new KeyboardEvent('keydown', { key: close, bubbles: true, cancelable: true })); } else { plate()!.querySelector<HTMLButtonElement>('[data-primary-action]')!.click(); }
      });
      window.removeEventListener('keydown', under);
      expect(plate(), close).toBeNull();
      expect(useGame.getState().introSeats).toEqual([]);
      expect(under, `${close} stays with the plate`).not.toHaveBeenCalled();
    }
  });

  it('leaves Esc and Return to a dialog opened over it (How to Play)', () => {
    show(as('Sparta'));
    const dialog = document.body.appendChild(document.createElement('dialog'));
    dialog.setAttribute('open', '');
    act(() => { window.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', bubbles: true, cancelable: true })); });
    expect(plate()).not.toBeNull();
    dialog.remove();
  });

  it('can be turned off for automated play-throughs', () => {
    expect(introEnabled()).toBe(true);
    localStorage.setItem('legatum.intro.v1', 'off');
    expect(introEnabled()).toBe(false);
  });
});
