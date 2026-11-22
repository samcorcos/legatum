import { describe, expect, it } from 'vitest';
import { JOIN_CODE_MAX, joinCodeProblem, typedJoinCode } from './joinCode';

describe('join code text', () => {
  it('keeps letters and digits only, up to the limit', () => {
    expect(typedJoinCode('wolf 42!')).toBe('wolf42');
    expect(typedJoinCode('é-ü_9')).toBe('9');
    expect(typedJoinCode('a'.repeat(30))).toHaveLength(JOIN_CODE_MAX);
  });

  it('accepts an empty box (an open game) and 4 or more characters, and says why a short code is not enough', () => {
    expect(joinCodeProblem('')).toBeNull();
    expect(joinCodeProblem('abcd')).toBeNull();
    expect(joinCodeProblem('abc')).toMatch(/at least 4/);
  });
});
