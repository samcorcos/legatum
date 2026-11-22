import { describe, expect, it } from 'vitest';
import { cardHint, cardMeter, cardName, cardNameAndAsk } from './ageCards';

describe('World Age cards', () => {
  it('names a card shortly', () => {
    expect(cardName('Veterans: earn 4 promotions in battle (a promotion is a level a unit gains from combat experience after this Age began)')).toBe('Veterans');
    expect(cardName('Humble the mighty: capture a city from an empire ahead of you on VP or, if you lead, from the rival with the most VP')).toBe('Humble the mighty');
    expect(cardName('Dominion: gain 3 cities')).toBe('Dominion');
  });

  it('words the Veterans meter, never a count above the target', () => {
    expect(cardMeter({ kind: 'Veterans', n: 4 }, 2, 4)?.label).toBe('Promotions earned since the Age began: 2 of 4');
    expect(cardMeter({ kind: 'Veterans', n: 4 }, 6, 4)?.label).toBe('Promotions earned since the Age began: 4 of 4');
    expect(cardHint('Veterans')).toContain('levels your units gain in battle');
  });

  it('words the meters of the capture kinds', () => {
    expect(cardMeter({ kind: 'Breach' }, 0, 1)?.label).toBe('Walled cities captured since the Age began: 0 of 1');
    expect(cardMeter({ kind: 'Population', n: 9 }, 6, 9)?.label).toBe('Population grown since the Age began: 6 of 9');
    expect(cardMeter({ kind: 'Cities', n: 4 }, 3, 4)?.label).toBe('Cities gained since the Age began: 3 of 4');
    expect(cardNameAndAsk('Dominion: gain 4 cities')).toBe('Dominion, gain 4 cities');
  });
});
