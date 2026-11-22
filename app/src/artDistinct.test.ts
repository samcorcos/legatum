// @vitest-environment node
// Every icon, medallion and miniature must stay distinct from the others in its set at display size, in greyscale and
// for colour-blind players (legatum-art; user, 2026-10-11: "Coal and iron look way too similar... All the icons,
// miniatures, etc need to be visually distinct"). tools/art/distinctness.py renders each set at its drawn size
// (headless Chromium, tools/art/render-sets.mjs), scores every pair, and fails when a new asset is as alike as an
// existing one in its set. Thresholds and the accepted-pair list live in tools/art/distinctness.json.
import { spawnSync } from 'node:child_process';
import { describe, expect, it } from 'vitest';

const root = new URL('../../', import.meta.url).pathname;
const run = (args: string[]) =>
  spawnSync('python3', ['tools/art/distinctness.py', ...args], { cwd: root, encoding: 'utf8', env: { ...process.env, LEGATUM_NODE: process.execPath }, maxBuffer: 64 * 1024 * 1024 });

describe('art distinctness', () => {
  it('the metric tells a copy from a different picture', () => {
    const r = spawnSync('python3', ['tools/art/test_distinctness.py'], { cwd: root, encoding: 'utf8' });
    expect(r.status, r.stderr + r.stdout).toBe(0);
  });

  it('no icon, medallion or miniature is too like another in its set', () => {
    const r = run(['--render', '--check']);
    // On failure the output lists every pair at or over its set's threshold, worst first, with the scores.
    expect(r.status, `${r.stdout}\n${r.stderr}`).toBe(0);
    expect(r.stdout).toMatch(/sets, \d+ pair comparisons/);
  }, 900_000);
});
