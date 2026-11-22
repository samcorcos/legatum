import { describe, expect, it } from 'vitest';
import { hasAmounts, splitAmounts, yieldParts } from './amountParts';

/** "+3 Science" shows as [+3 Science]; plain words stay as they are. */
const show = (text: string) => splitAmounts(text).map((s) => 'amount' in s ? `[${s.amount.sign}${s.amount.n} ${s.amount.unit}]` : s.text).join('');

describe('splitAmounts', () => {
  it('renders missing label text as empty, including the undefined text from the turn-200 crash', () => {
    expect(splitAmounts(undefined)).toEqual([]);
    expect(splitAmounts(null)).toEqual([]);
    expect(hasAmounts(undefined)).toBe(false);
    expect(hasAmounts(null)).toBe(false);
  });

  it('finds signed amounts of each unit', () => {
    expect(show('+3 Science')).toBe('[+3 Science]');
    expect(show('+2 Food, +1 Production')).toBe('[+2 Food], [+1 Production]');
    expect(show('+2 VP once per empire')).toBe('[+2 VP] once per empire');
    expect(show('costs 10 Iron')).toBe('costs [10 Iron]');
    expect(show('Siege Engines cost 10 Coal')).toBe('Siege Engines cost [10 Coal]');
  });

  it('normalises minus signs and keeps "/turn" and "per turn" as text', () => {
    expect(show('Upkeep -1 Gold/turn')).toBe('Upkeep [−1 Gold]/turn');
    expect(show('Upkeep −3 Gold per turn')).toBe('Upkeep [−3 Gold] per turn');
    expect(show('-2 Food, +6 Production; one per empire')).toBe('[−2 Food], [+6 Production]; one per empire');
  });

  it('handles unsigned costs, ranges and thousands separators', () => {
    expect(show('Costs 146 Production')).toBe('Costs [146 Production]');
    expect(show('between 10–20 Gold')).toBe('between [10–20 Gold]');
    expect(show('820/1,500 Science from ruins')).toBe('[820/1,500 Science] from ruins');
    expect(show('Gold 120 → 90')).toBe('Gold 120 → 90');
    expect(show('Stored 120 → 90 Gold, debt')).toBe('Stored [120 → 90 Gold], debt');
    expect(show('Net income +3 → −1 Gold per turn')).toBe('Net income [+3 → −1 Gold] per turn');
  });

  it('leaves names and non-amounts alone', () => {
    for (const text of ['Science victory', 'Gold project', 'Production is the city’s', 'Upgrade Gold costs -50%', 'Population 5 required',
      '25% of Production becomes Science', 'T10 Gold', 'Gold plus Production generated per turn', 'Unlocks the 300-Production Great Work',
      '+10 city strength', '+12 against city HP', '−35 HP', 'Largest Economy', 'Iron and Coal are strategic']) {
      expect(hasAmounts(text), text).toBe(false);
      expect(show(text)).toBe(text);
    }
  });

  it('does not take a number that belongs to a longer token', () => {
    expect(show('turn 12 Gold')).toBe('turn [12 Gold]');
    expect(show('T12 Gold')).toBe('T12 Gold');
    expect(show('ring-2 Gold')).toBe('ring-2 Gold');
  });

  it('round-trips: segments rebuild the text, with signs normalised', () => {
    const text = 'Upgrade from Laboratory; Population 5 required; +4 Science, then +2 Gold';
    expect(show(text)).toBe('Upgrade from Laboratory; Population 5 required; [+4 Science], then [+2 Gold]');
  });
});

describe('yieldParts', () => {
  it('lists nonzero yields in order with signs', () => {
    expect(yieldParts({ food: -2, production: 6, gold: 0, science: 0 })).toEqual([
      { sign: '−', n: '2', unit: 'Food' }, { sign: '+', n: '6', unit: 'Production' }]);
    expect(yieldParts({ food: 2, gold: 1 }, { signed: false })).toEqual([{ sign: '', n: '2', unit: 'Food' }, { sign: '', n: '1', unit: 'Gold' }]);
  });
});
