import { describe, expect, it } from 'vitest';
import type { PlayerView } from '../types';
import { meetingsForTurn, planMoments, heldMoment, type Moment } from './moments';
import { asTheirs, whereLine } from './MeetingTakeover';
import { FACTION_SHEETS } from '../factionSheets';
import { GREETING } from './greetings';
import { FACTION_INFO } from '../data';

const player = (id: number, team: number, faction: string) => ({ id, team, faction, name: `P${id}`, alive: true, is_human: id === 0, vp: 0, vp_sources: [], cities: 1, faction_goal_done: false });
const viewWith = (met: number[], turn = 12) => ({
  turn, me: 0, team: 0,
  players: [player(0, 0, 'Athens'), player(1, 1, 'Rome'), player(2, 2, 'Egypt'), player(3, 2, 'Sparta')],
  cities: [
    { id: 1, owner: 0, name: 'Athenai', pos: { q: 0, r: 0 } },
    { id: 2, owner: 1, name: 'Roma', pos: { q: 6, r: -3 } },
  ],
  units: [],
  diplomacy: { met, relations: [] },
}) as unknown as PlayerView;

describe('first contact', () => {
  it('makes one held card per empire met this turn, never for one already met', () => {
    const cards = meetingsForTurn(viewWith([1]), viewWith([1, 2]), 5);
    expect(cards).toHaveLength(1);
    const [card] = cards;
    expect(card.meeting!.team).toBe(2);
    expect(card.meeting!.members.map((m) => m.faction)).toEqual(['Egypt', 'Sparta']);
    expect(heldMoment(card)).toBe(true);
  });

  it('says nothing without the earlier view (a loaded game, a replayed card)', () => {
    expect(meetingsForTurn(null, viewWith([1, 2]), 0)).toEqual([]);
  });

  it('places a met empire by its nearest known city', () => {
    const [card] = meetingsForTurn(viewWith([]), viewWith([1]), 0);
    expect(card.meeting!.where).toMatchObject({ kind: 'city', name: 'Roma', from: 'Athenai' });
    expect(whereLine(card.meeting!.where)).toMatch(/^Their city Roma lies \d+ tiles [a-z-]+ of Athenai\.$/);
    expect(whereLine(null)).toBeNull();
    expect(whereLine({ at: { q: 0, r: 0 }, kind: 'unit', name: 'Scout', tiles: 0, direction: 'N', from: null })).toBe('You have seen their Scout.');
    expect(whereLine({ at: { q: 0, r: 0 }, kind: 'unit', name: 'Scout', tiles: 0, direction: 'N', from: 'Athenai' })).toBe('Their Scout was seen in Athenai.');
  });

  it('plays every meeting after the era-up and score check, and never folds one away', () => {
    const base = { owner: 0, faction: 'Rome', partner: false, at: null, title: '', effect: '', lore: '', details: [], deltas: [] } as const;
    const card = (id: string, extra: Partial<Moment>): Moment => ({ ...base, id, eventIndex: Number(id.length), tier: 'major', ...extra }) as Moment;
    const meets = meetingsForTurn(viewWith([]), viewWith([1, 2]), 50);
    const others = ['a', 'bb', 'ccc', 'dddd'].map((id) => card(id, {}));
    const eraUp = card('era', { tier: 'legendary', eraUp: {} as Moment['eraUp'] });
    const plan = planMoments([...others, ...meets, eraUp]);
    expect(plan.shown.slice(0, 3).map((m) => !(m.eraUp) ? !(m.meeting) ? m.id : `meet${m.meeting.team}` : 'era')).toEqual(['era', 'meet1', 'meet2']);
    expect(plan.shown.filter((m) => m.meeting)).toHaveLength(2);
    expect(plan.collapsed.some((m) => m.meeting)).toBe(false);
  });

  it('gives every people a greeting in its own voice', () => {
    for (const people of Object.keys(FACTION_INFO)) {
      const g = GREETING[people as keyof typeof GREETING];
      expect(g, people).toBeTruthy();
      expect(g.line.split(/\s+/).length, people).toBeLessThanOrEqual(25);
      expect(!!g.native === !!g.meaning, people).toBe(true);
    }
  });

  it('turns every sheet line about a people into a line about them', () => {
    expect(asTheirs('Cities other than your capital send you no Gold.')).toBe('Cities other than their capital send them no Gold.');
    expect(asTheirs('You buy buildings at 175%.')).toBe('They buy buildings at 175%.');
    expect(asTheirs("A city that is building doesn't grow: switch it to Grow to grow it.")).toBe("A city that is building doesn't grow.");
    expect(asTheirs('It lets you move, and if you win it costs you 5 Gold.')).toBe('It lets them move, and if they win it costs them 5 Gold.');
    for (const sheet of Object.values(FACTION_SHEETS)) for (const p of sheet.pairs) for (const line of [p.strength, p.price]) {
      if (!(line)) {} else { expect(asTheirs(line)).not.toMatch(/\b(you|your)\b/i); }
    }
  });
});
