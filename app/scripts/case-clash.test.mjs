// Two tracked files whose paths differ only in letter case are one file on a Mac's default disk, so the app
// stops building there (2026-11-22: celebrate/ScoreCheck.tsx and celebrate/scoreCheck.ts).
import { execFileSync } from 'node:child_process';
import { resolve } from 'node:path';
import { describe, expect, it } from 'vitest';

describe('tracked files', () => {
  it('have no two paths that differ only in letter case', () => {
    const files = execFileSync('git', ['ls-files'], { cwd: resolve(__dirname, '../..'), encoding: 'utf8' }).split('\n').filter(Boolean);
    // A TypeScript module is imported without its extension, so './Foo' and './foo' clash even as Foo.tsx and foo.ts.
    const key = (f) => f.toLowerCase().replace(/\.tsx?$/, '');
    const seen = new Map();
    const clashes = [];
    for (const f of files) {
      const k = key(f);
      if (seen.has(k) && seen.get(k) !== f) clashes.push(`${seen.get(k)} / ${f}`);
      else seen.set(k, f);
    }
    expect(clashes).toEqual([]);
  });
});
