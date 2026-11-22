import { expect, it } from 'vitest';
import { sakokuChip, sakokuLine, sakokuStatus, sakokuTarget } from './sakoku';

const open = { sakoku_turns: null, faction_goal_done: false };

it('the countdown is 30 turns closed at Standard, from the core rules', () => {
  expect(sakokuTarget()).toBe(30);
});

it('an open country has no public status; a closed one has its count, and the VP holds after the countdown', () => {
  expect(sakokuStatus(open)).toBeNull();
  expect(sakokuStatus({ sakoku_turns: undefined, faction_goal_done: false })).toBeNull();
  const counting = sakokuStatus({ sakoku_turns: 12, faction_goal_done: false })!;
  expect(counting).toEqual({ turns: 12, target: 30, left: 18, holding: false });
  expect(sakokuLine(counting)).toBe('Closed 12 of 30 turns · holds 2 VP in 18');
  expect(sakokuChip(counting)).toBe('12/30');
  const holding = sakokuStatus({ sakoku_turns: 33, faction_goal_done: true })!;
  expect(holding.holding).toBe(true);
  expect(sakokuLine(holding)).toBe('Closed 33 turns · holding 2 VP');
  expect(sakokuChip(holding)).toBe('holding');
});
