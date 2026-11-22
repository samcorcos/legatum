// @vitest-environment happy-dom
// The Age score check (user, 2026-11-22): every player tallies up when a World Age begins. The track shows who is ahead,
// the sheet shows each way to score as boxes with how to win the next point, and it plays after the opener's era-up.
import { act } from 'react';
import { createRoot, type Root } from 'react-dom/client';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { FixtureClient } from '../test/fixtureEngine';
import { useGame } from '../store';
import { withScoreCheck, withWorldAges } from '../test/vpV2Fixture';
import { momentsForTurn, planMoments } from './moments';
import { scoreCheckFor } from './scoreCheckCards';
import { ScoreCheck } from './ScoreCheck';
import { EraUpTakeover } from './EraUpTakeover';
import type { PlayerView } from '../types';

let host: HTMLElement, root: Root, base: PlayerView;
beforeEach(async () => {
  vi.stubGlobal('IS_REACT_ACT_ENVIRONMENT', true);
  useGame.setState(useGame.getInitialState(), true);
  const client = new FixtureClient();
  await client.newGame(await client.standardSetup('42', 'India', 'Byzantium', 'Ada', 'Bo'));
  base = structuredClone(client.view(0));
  host = document.body.appendChild(document.createElement('div'));
  root = createRoot(host);
});
afterEach(() => { act(() => root.unmount()); host.remove(); vi.unstubAllGlobals(); });

describe('the score check model', () => {
  it('puts every empire on the track at its VP and says where you stand', () => {
    const view = withScoreCheck(base);
    const card = scoreCheckFor(view, 3, view.me);
    expect(card.track).toHaveLength(11);
    expect(card.track[5].pieces.map((p) => p.faction)).toEqual(['Mongols']);
    expect(card.track[3].pieces.some((p) => p.isMine)).toBe(true);
    expect(card.standing).toBe('You: 3 VP, tied 2nd with Celts and Switzerland.');
    expect(card.leader).toBe('Leader: Mongols, 5 VP.');
    expect(card.opener).toBe('You reached the Medieval era first.');
    expect(card.hold).toBeNull();
  });

  it('fills your sheet: the scored Age, the new card, the capitals and the held wonder', () => {
    const view = withScoreCheck(base);
    const card = scoreCheckFor(view, 3, view.me);
    const ages = card.lines.find((l) => l.key === 'ages')!;
    expect(ages.boxes.slice(0, 3).map((b) => b.state)).toEqual(['kept', 'new', 'later']);
    // A box still to come names its Age, as the heading does; the row terms are just the VP.
    expect(ages.boxes[2].title).toBe('Renaissance: not begun yet');
    expect(ages.terms).toBe('1 VP each');
    expect(ages.how).toBe('New card: Breach, capture a city that has Walls');
    expect(ages.fresh).toBe(true);
    expect(ages.open).toBe(1);
    const capitals = card.lines.find((l) => l.key === 'capitals')!;
    expect(capitals.boxes.every((b) => b.state === 'open')).toBe(true);
    expect(capitals.boxes).toHaveLength(view.players.filter((p) => p.team !== view.team).length);
    expect(capitals.how).toBe('Take a rival’s original capital');
    const wonders = card.lines.find((l) => l.key === 'wonders')!;
    // One box per wonder, two cells wide (2 VP), drawing the wonder's icon; capitals draw their star.
    const held = wonders.boxes.filter((b) => b.state === 'held');
    expect(held).toHaveLength(1);
    expect(held[0]).toMatchObject({ span: 2, icon: expect.stringMatching(/^building-/) });
    expect(capitals.boxes.every((b) => b.star)).toBe(true);
    const goal = card.lines.find((l) => l.key === 'goal')!;
    expect(goal.boxes.reduce((n, b) => n + (b.span ?? 1), 0)).toBe(2);
    expect(goal.boxes[0].icon).toBeTruthy();
    expect(wonders.detail).toContain('Great Library');
  });

  it('names a rival holding its target, and how soon it wins', () => {
    const view = withScoreCheck(base);
    const mongols = view.standings.find((s) => view.players.find((p) => p.id === s.members[0])?.faction === 'Mongols')!;
    mongols.vp = 10;
    mongols.victory_hold = { turns_held: 6, turns_needed: 10 };
    const card = scoreCheckFor(view, 3, view.me);
    expect(card.hold).toBe('Mongols holds 10 VP and wins in 4 turns unless someone stops them.');
    expect(card.track[10].pieces.map((p) => p.faction)).toEqual(['Mongols']);
  });
});

describe('when it plays', () => {
  it('every player gets it when an Age begins, right after the opener’s own era-up', () => {
    const view = withScoreCheck(base);
    const moments = momentsForTurn([{ type: 'EraEntered', player: view.me, era: 3, first: true }, { type: 'AgeBegan', era: 3, first: view.me }], view.me, base, view);
    const { shown } = planMoments(moments);
    expect(shown.map((m) => (m.eraUp ? 'era-up' : m.scoreCheck ? 'score-check' : m.title))).toEqual(['era-up', 'score-check']);
    // A rival opened the Age: the score check alone, first.
    const rival = view.players.find((p) => p.team !== view.team)!.id;
    const theirs = planMoments(momentsForTurn([{ type: 'AgeBegan', era: 3, first: rival }], view.me, base, view)).shown;
    expect(theirs).toHaveLength(1);
    expect(theirs[0].scoreCheck?.opener).toBe('Mongols reached the Medieval era first.');
  });

  it('the era-up no longer repeats the points in a World Ages game', () => {
    const view = withWorldAges(base);
    useGame.setState({ views: [view], seat: 0 });
    const m = momentsForTurn([{ type: 'EraEntered', player: 0, era: 2, first: false }], 0, base, view)[0];
    act(() => root.render(<EraUpTakeover moment={m} dismiss={vi.fn()} />));
    expect(host.querySelector('[data-testid=era-up-objectives]')).toBeNull();
    expect(host.textContent).not.toContain('Victory points open now');
  });
});

describe('shown again from the report', () => {
  it('marks the newest Age begun as new and says the sheet is the score now', () => {
    const view = withScoreCheck(base);
    // Replaying the Classical Age (II) on turn 92, after the Medieval Age (III) has begun.
    const card = { ...scoreCheckFor(view, 2, view.me), replay: true };
    const ages = card.lines.find((l) => l.key === 'ages')!;
    expect(ages.boxes.slice(0, 3).map((b) => b.state)).toEqual(['kept', 'new', 'later']);
    expect(ages.how).toBe('New card: Breach, capture a city that has Walls');
    const m = { ...momentsForTurn([{ type: 'AgeBegan', era: 2, first: view.me }], view.me, base, view)[0] };
    m.scoreCheck = card;
    act(() => root.render(<ScoreCheck moment={m} dismiss={vi.fn()} />));
    expect(host.querySelector('.sc-kicker')?.textContent).toBe('The Classical Age · the score now · Turn 92');
  });

  it('never says an older card is open when none is', () => {
    const card = scoreCheckFor(withScoreCheck(base), 3, 0);
    const ages = card.lines.find((l) => l.key === 'ages')!;
    expect(ages.detail).toBe('Cards never expire: this one stays open until you score it.');
  });
});

describe('the takeover', () => {
  it('shows the track, your sheet and one primary that closes it', () => {
    const view = withScoreCheck(base);
    const m = momentsForTurn([{ type: 'AgeBegan', era: 3, first: view.me }], view.me, base, view)[0];
    const dismiss = vi.fn();
    act(() => root.render(<ScoreCheck moment={m} dismiss={dismiss} />));
    const t = host.querySelector('[data-testid=score-check]')!;
    expect(t.querySelector('.sc-kicker')?.textContent).toBe('The Medieval Age begins · Turn 92');
    expect(t.querySelectorAll('[data-testid=score-track] li')).toHaveLength(11);
    expect([...t.querySelectorAll('.sc-row[data-line] .sc-name b')].map((b) => b.textContent)).toEqual(['Age cards', 'Rival capitals', 'World wonders', 'Faction goal']);
    expect(t.querySelector('.sc-row.fresh')?.getAttribute('data-line')).toBe('ages');
    // The boxes draw the thing that scores: the wonder's icon, the capital's star.
    expect(t.querySelector('.sc-row[data-line=wonders] .sc-box.held .mask-icon')).not.toBeNull();
    expect(t.querySelector('.sc-row[data-line=capitals] .sc-box svg path')).not.toBeNull();
    expect(t.querySelector('.sc-row[data-line=wonders] .sc-box.held')?.getAttribute('data-span')).toBe('2');
    expect(t.querySelectorAll('[data-primary-action]')).toHaveLength(1);
    act(() => t.querySelector<HTMLButtonElement>('[data-primary-action]')!.click());
    expect(dismiss).toHaveBeenCalledOnce();
  });
});
