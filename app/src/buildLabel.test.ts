import { describe, expect, it } from 'vitest';
import { formatBuildLabel } from './buildLabel';

describe('formatBuildLabel', () => {
  it('uses the release label when one is set', () => {
    expect(formatBuildLabel('Playtest 7a', 'playtest-6.3-12-gabcdef1')).toBe('Playtest 7a');
  });
  it('names an exactly tagged build after its playtest', () => {
    expect(formatBuildLabel('', 'playtest-6.3')).toBe('Playtest 6.3');
    expect(formatBuildLabel('', 'playtest-7a')).toBe('Playtest 7a');
  });
  it('marks builds past a playtest tag as dev builds', () => {
    expect(formatBuildLabel('', 'playtest-6.3-12-gabcdef123')).toBe('Dev · after 6.3 (abcdef1)');
  });
  it('falls back when there is no playtest tag', () => {
    expect(formatBuildLabel('', 'abcdef123')).toBe('Dev (abcdef1)');
    expect(formatBuildLabel('', '')).toBe('Dev build');
  });
  it('names the package version in a production build with no playtest tag', () => {
    expect(formatBuildLabel('', 'abcdef123', '0.1.0')).toBe('Version 0.1.0 (abcdef1)');
    expect(formatBuildLabel('', '', '0.1.0')).toBe('Version 0.1.0');
    expect(formatBuildLabel('', 'playtest-7a', '0.1.0')).toBe('Playtest 7a');
  });
});
