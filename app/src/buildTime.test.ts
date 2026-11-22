import { describe, expect, it } from 'vitest';
import { formatBuildTime } from './buildTime';

describe('formatBuildTime', () => {
  it('shows the build time in the given time zone', () => {
    expect(formatBuildTime('2026-11-22T21:39:00Z', 'America/Chicago')).toBe('22 Nov 2026 15:39');
    expect(formatBuildTime('2026-11-22T21:39:00Z', 'UTC')).toBe('22 Nov 2026 21:39');
    expect(formatBuildTime('2026-09-03T08:05:00Z', 'UTC')).toBe('3 Sep 2026 08:05');
  });
  it('shows nothing for a missing or bad time', () => {
    expect(formatBuildTime('')).toBe('');
    expect(formatBuildTime('not a time')).toBe('');
  });
});
