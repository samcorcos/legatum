// @vitest-environment happy-dom
import { act } from 'react';
import { createRoot, type Root } from 'react-dom/client';
import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import { FixtureClient } from '../test/fixtureEngine';
import { VpTrack } from './VpTrack';
import { HOVER_INTENT_MS } from './VpHoldings';
import { useGame } from '../store';
import type { PlayerView, VpSource } from '../types';
import { withV2 } from '../test/vpV2Fixture';
import { decadenceCurveText, decadencePercent } from '../data';

let host: HTMLElement, root: Root, view: PlayerView;
beforeEach(async () => {
  vi.stubGlobal('IS_REACT_ACT_ENVIRONMENT', true);
  vi.useFakeTimers();
  useGame.setState(useGame.getInitialState(), true);
  const client = new FixtureClient();
  await client.newGame(await client.standardSetup('42', 'India', 'Byzantium', 'Ada', 'Bo'));
  const base = client.view(0);
  const v2 = withV2(base, { era: 2, scored: [2] });
  view = {
    ...v2,
    players: v2.players.map((p) => p.id === v2.me ? { ...p, vp_sources: [...p.vp_sources, [{ kind: 'GreatWork', of: p.id }, 2] as [VpSource, number]] } : p),
    faction_goal: { value: 0, target: 1, label: 'The Imperial Diwan is not built', live: true, done: false },
  };
  useGame.setState({ client, views: [view], humans: [0], orders: [[]], errors: [[]], seat: 0 });
  host = document.body.appendChild(document.createElement('div'));
  root = createRoot(host);
  act(() => root.render(<div className="game"><div className="seatbar" /><VpTrack /></div>));
});
afterEach(() => { act(() => root.unmount()); host.remove(); document.querySelectorAll('.vhc-pop').forEach((n) => n.remove()); vi.useRealTimers(); vi.unstubAllGlobals(); });

const tokens = () => [...host.querySelectorAll<HTMLElement>('.panel.vp [data-source]')];
const card = () => document.querySelector('.vhc-pop');
const cardTitle = () => card()?.querySelector('h3')?.textContent ?? null;

it('each empire shows a token for everything it holds: kept solid, held ones outlined, its own goal dashed until done', () => {
  const mine = host.querySelector('.vp-row.mine')!;
  const held = [...mine.querySelectorAll('.vhc-tok')];
  expect(held.map((t) => t.getAttribute('data-source'))).toEqual(['era-2', 'GreatWork']);
  expect(held.map((t) => t.classList.contains('live'))).toEqual([false, true]);
  expect(held[0].getAttribute('aria-label')).toMatch(/kept for good, yours$/);
  expect(held[1].getAttribute('aria-label')).toMatch(/held: can be captured, yours$/);
  // An era token wears its Roman numeral.
  expect(held[0].textContent).toBe('II');
  expect(held[0].getAttribute('aria-label')).toMatch(/Classical, \+1 VP, kept for good/);
  expect(mine.querySelectorAll('.vhc-goal-open').length).toBeGreaterThan(0);
  expect(mine.querySelector('.vhc-goal-open')!.getAttribute('aria-label')).toMatch(/not yet/);
  expect(mine.querySelector('.vhc-goal-progress')?.textContent).toBe('The Imperial Diwan is not built');
});

it('the last row lists the entered eras nobody holds, with what they are worth', () => {
  const nobody = host.querySelector('.vhc-nobody')!;
  const open = [...nobody.querySelectorAll('.vhc-open-tok')].map((t) => t.getAttribute('data-source'));
  // Era 2 is scored, era 3 is entered by a rival and scored by nobody, era 4 is entered by nobody.
  expect(open).toEqual(['era-3']);
  expect(nobody.querySelector('.vhc-worth')!.textContent).toBe('1 VP open');
});

it('focusing a token opens its card with the rule and every empire’s number; blur closes it', () => {
  const tok = host.querySelector<HTMLElement>(`.vp-row.mine [data-source="GreatWork"]`)!;
  act(() => tok.focus());
  expect(cardTitle()).toBe('World wonder');
  expect(card()!.querySelector('.vh-card.own-mine')).not.toBeNull();
  act(() => tok.blur());
  expect(card()).toBeNull();
});

it('the own faction token names the goal and its card explains the rule and current progress', () => {
  const token = host.querySelector<HTMLElement>(`.vp-row.mine [data-source="goal-${view.me}"]`)!;
  expect(token.getAttribute('aria-label')).toContain('The Imperial Diwan');
  expect(host.querySelector('.vhc-goal-progress')?.textContent).toContain('The Imperial Diwan is not built');
  act(() => token.focus());
  expect(cardTitle()).toBe('The Imperial Diwan');
  expect(card()!.textContent).toContain('Build a Revenue Office in every owned city');
  expect(card()!.textContent).toContain('The Imperial Diwan is not built');
});

it('pointing at a token opens its card after a short rest, and leaving closes it', () => {
  const tok = host.querySelector<HTMLElement>(`.vp-row:not(.mine) [data-source="era-2"]`)!;
  act(() => { tok.dispatchEvent(new PointerEvent('pointerover', { bubbles: true, pointerType: 'mouse' })); });
  expect(card()).toBeNull();
  act(() => { vi.advanceTimersByTime(HOVER_INTENT_MS + 5); });
  expect(cardTitle()).toBe('Age II: Classical');
  expect(card()!.querySelector('.vh-card.own-mine')).not.toBeNull();
  act(() => { tok.dispatchEvent(new PointerEvent('pointerout', { bubbles: true, pointerType: 'mouse' })); vi.advanceTimersByTime(200); });
  expect(card()).toBeNull();
});

it('a sweep across tokens opens nothing; the arrow keys flip through every way to score; Esc closes', () => {
  const first = host.querySelector<HTMLElement>(`.vp-row.mine [data-source="era-2"]`)!;
  act(() => first.focus());
  const order = [...new Set(tokens().map((t) => t.dataset.source!))];
  const firstTitle = cardTitle();
  expect(cardTitle()).toBe(firstTitle);
  act(() => { document.activeElement!.dispatchEvent(new KeyboardEvent('keydown', { key: 'ArrowRight', bubbles: true })); });
  const second = cardTitle();
  expect(second).not.toBe(firstTitle);
  act(() => { document.activeElement!.dispatchEvent(new KeyboardEvent('keydown', { key: 'ArrowLeft', bubbles: true })); });
  expect(cardTitle()).toBe(firstTitle);
  expect(order.length).toBeGreaterThan(4);
  expect(order).toContain('era-2');
  act(() => { document.activeElement!.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', bubbles: true })); });
  expect(card()).toBeNull();
});

it('every source has a token somewhere, so the arrows can reach all of them', () => {
  const present = new Set(tokens().map((t) => t.dataset.source));
  expect(present.has('GreatWork')).toBe(true);
  // An era is a way to score once an empire has entered it.
  for (const e of view.eras.filter((x) => x.entered_by.length)) expect(present.has(`era-${e.era}`)).toBe(true);
  expect(present.has('era-4')).toBe(false);
  for (const p of view.players.filter((q) => q.alive)) expect(present.has(`goal-${p.id}`)).toBe(true);
});

it('the panel has one Tab stop among its tokens (roving), and never says anything is a suggestion', () => {
  expect(tokens().filter((t) => t.tabIndex === 0)).toHaveLength(1);
  expect(host.textContent).not.toMatch(/fastest|route|recommend|suggest/i);
});

it('a team holding its target shows an unmistakable countdown above the rows: yours gilt, a rival’s red, both with a meter', () => {
  const rivalTeam = view.standings.find((s) => s.team !== view.team)!;
  const mineTeam = view.standings.find((s) => s.team === view.team)!;
  const held = { ...view, standings: view.standings.map((s) => (s === rivalTeam ? { ...s, victory_hold: { turns_held: 3, turns_needed: 10 } } : s === mineTeam ? { ...s, victory_hold: { turns_held: 9, turns_needed: 10 } } : s)) };
  act(() => useGame.setState({ views: [held] }));
  const strips = [...host.querySelectorAll('.vhc-hold')];
  expect(strips.map((s) => s.classList.contains('mine'))).toEqual([true, false]);
  expect(strips[0].textContent).toMatch(/Your team wins in 1 turn/);
  expect(strips[1].textContent).toMatch(/wins in 7 turns/);
  expect(strips[1].getAttribute('aria-label')).toMatch(/wins in 7 turns: holding/);
  expect(strips[1].classList.contains('last')).toBe(false);
  expect(strips[0].classList.contains('last')).toBe(true);
  const meter = strips[1].querySelector('[role="progressbar"]')!;
  expect(meter.getAttribute('aria-valuenow')).toBe('3');
  expect(meter.getAttribute('aria-valuemax')).toBe('10');
  expect(meter.querySelectorAll('i.on')).toHaveLength(3);
  expect(host.querySelectorAll('.vp-row.holding')).toHaveLength(2);
});

it('a closed country’s count is public: a chip in its row, with the whole rule in its tooltip, and nothing for an open one', () => {
  expect(host.querySelector('[data-testid="closed-chip"]')).toBeNull();
  const rival = view.players.find((p) => p.id !== view.me && p.alive)!;
  const closed = { ...view, players: view.players.map((p) => (p.id === rival.id ? { ...p, sakoku_turns: 12 } : p)) };
  act(() => useGame.setState({ views: [closed] }));
  const chip = host.querySelector<HTMLElement>('[data-testid="closed-chip"]')!;
  expect(chip).not.toBeNull();
  expect(chip.textContent).toBe('12/30');
  expect(chip.getAttribute('aria-label')).toContain('closed: Closed 12 of 30 turns · holds 2 VP in 18');
  expect(chip.getAttribute('title')).toContain("can't declare war or ally");
  expect(chip.closest('.vp-row')).not.toBeNull();
  const holding = { ...closed, players: closed.players.map((p) => (p.id === rival.id ? { ...p, sakoku_turns: 31, faction_goal_done: true } : p)) };
  act(() => useGame.setState({ views: [holding] }));
  expect(host.querySelector('[data-testid="closed-chip"]')!.textContent).toBe('holding');
  expect(host.querySelector('[data-testid="closed-chip"]')!.classList.contains('holding')).toBe(true);
});

it('every row says whether its capital stands, in words, and the rule is stated once', () => {
  const chips = () => [...host.querySelectorAll<HTMLElement>('[data-testid="capital-chip"]')];
  expect(chips().length).toBe(host.querySelectorAll('.vhc-row').length);
  expect(chips().every((c) => c.textContent === 'Capital' && c.dataset.stands === 'true')).toBe(true);
  expect(host.querySelectorAll('.vhc-rule').length).toBe(1);
  expect(host.querySelector('.vhc-rule')!.textContent).toBe("Win by holding the last capital: every rival's original capital taken or razed.");
  const rival = view.players.find((p) => p.team !== view.team && p.alive)!;
  const fallen = { ...view, players: view.players.map((p) => (p.id === rival.id ? { ...p, capital_stands: false } : p)) };
  act(() => useGame.setState({ views: [fallen] }));
  const lost = chips().filter((c) => c.dataset.stands === 'false');
  expect(lost.map((c) => c.dataset.team)).toEqual([String(rival.team)]);
  expect(lost[0].textContent).toBe('Capital fallen');
  expect(lost[0].getAttribute('aria-label')).toContain('original capital fallen');
  expect(lost[0].getAttribute('title')).toContain('can still win by VP');
  expect(chips().filter((c) => c.dataset.stands === 'true').length).toBe(chips().length - 1);
});

it('an empire above the average shows its lead and the one rate it cuts every yield by, with the curve in the tooltip', () => {
  const rate = (lead: number) => decadencePercent(lead, { decadenceSlope: 45 });
  const view = withV2(structuredClone(useGame.getState().views![0]), { era: 2, lead: 60, rivalLead: 30 });
  act(() => useGame.setState({ views: [view] }));
  const mine = host.querySelector('.vp-row.mine [data-testid="lead-chip"]')!;
  expect(mine.textContent).toBe('+60%');
  expect(mine.getAttribute('aria-label')).toBe('Power 60% above the average empire');
  const title = mine.getAttribute('title')!;
  expect(title).toContain(`Your Power is 60% above the average empire's.`);
  expect(title).toContain(`the more its people grow complacent, so your cities grow ${rate(60)}% slower and lose ${rate(60)}% of their Production, Gold and Science`);
  expect(title).toContain(decadenceCurveText({ decadenceSlope: 45 }));
  expect(title).toMatch(/Power is an empire's total yields/);
  expect(title).not.toMatch(/Gold income/);
  const chips = [...host.querySelectorAll('.vp-row:not(.mine) [data-testid="lead-chip"]')];
  expect(chips.map((c) => c.textContent)).toEqual(['+30%']);
  // The rate is its own red chip with an arrow (colour is never alone): every yield, not Gold.
  const cut = host.querySelector('.vp-row.mine [data-testid="decadence-chip"]')!;
  expect(cut.textContent).toBe(`${rate(60)}%`);
  expect(cut.querySelector('.icon')).not.toBeNull();
  expect(cut.getAttribute('aria-label')).toBe(`Decadence: growth ${rate(60)}% slower, Production, Gold and Science cut by ${rate(60)}%`);
  expect(cut.getAttribute('title')).toMatch(/grow \d+% slower \(smaller families\) and lose \d+% of their Production \(sloth\), Gold \(luxury\) and Science \(complacency\)/);
  expect([...host.querySelectorAll('.vp-row:not(.mine) [data-testid="decadence-chip"]')].map((c) => c.textContent)).toEqual([`${rate(30)}%`]);
  // At or below the average there is no chip.
  act(() => useGame.setState({ views: [withV2(structuredClone(useGame.getState().views![0]), { era: 2 })] }));
  expect(host.querySelector('[data-testid="lead-chip"]')).toBeNull();
  expect(host.querySelector('[data-testid="decadence-chip"]')).toBeNull();
});

it('after the game ends the race shows final standings without countdowns, rival warnings or era objectives', () => {
  const finished = {
    ...structuredClone(useGame.getState().views![0]),
    finished: true,
    winner: view.team,
  };
  act(() => useGame.setState({ views: [finished] }));

  expect(host.querySelectorAll('.vp-row')).toHaveLength(finished.standings.length);
  expect(host.querySelector('.rival-win')).toBeNull();
  expect(host.querySelector('.vhc-holds-list')).toBeNull();
  expect(host.querySelector('.era-hud')).toBeNull();
  expect(host.querySelector('.vhc-nobody')).toBeNull();
  expect(host.querySelector('.vp-foot')).toBeNull();
  expect(host.textContent).not.toMatch(/wins this turn|Next: .* era/);
});
