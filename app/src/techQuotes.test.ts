import { expect, it } from 'vitest';
import { TECHS, TECH_NAME } from './data';
import { TECH_QUOTE } from './techQuotes';

it('every technology has its own one-line quote, with no attribution', () => {
  expect(Object.keys(TECH_QUOTE).sort()).toEqual([...TECHS].sort());
  const lines = Object.values(TECH_QUOTE);
  expect(new Set(lines).size).toBe(TECHS.length);
  for (const tech of TECHS) {
    const line = TECH_QUOTE[tech];
    expect(line.length, TECH_NAME[tech]).toBeGreaterThan(15);
    expect(line.length, TECH_NAME[tech]).toBeLessThan(75);
    expect(line, TECH_NAME[tech]).toMatch(/[.!?]$/);
    expect(line, TECH_NAME[tech]).not.toMatch(/["“”—–]/);
  }
});
