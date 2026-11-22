// tools/release/keep-builds.mjs: every build stays playable at v/<id>/ and is listed in versions.json, so a game
// always finishes on the version it started on (app/src/platform/versions.ts reads the list).
import { execFile, spawnSync } from 'node:child_process';
import { mkdtempSync, mkdirSync, writeFileSync, existsSync, readFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { dirname, join, resolve } from 'node:path';
import { describe, expect, it } from 'vitest';
import { startStaticServer } from '../e2e-hosted/serve.mjs';

const tools = resolve(__dirname, '../../tools/release');
const env = { ...process.env, NO_PROXY: '127.0.0.1', no_proxy: '127.0.0.1' };

function build(id) {
  const dir = mkdtempSync(join(tmpdir(), 'keep-builds-'));
  const files = {
    'index.html': `<!doctype html><html><head><meta charset="UTF-8" /><script type="module" src="./assets/index-${id}.js"></script></head><body></body></html>`,
    [`assets/index-${id}.js`]: `new URL(\`tank-${id}.webp\`,import.meta.url)`,
    [`assets/tank-${id}.webp`]: 'art',
    [`terrain/grass-${id}.webp`]: 'grass',
    '404.html': '<html><body>There is nothing at this address</body></html>',
  };
  for (const [name, text] of Object.entries(files)) {
    mkdirSync(dirname(join(dir, name)), { recursive: true });
    writeFileSync(join(dir, name), text);
  }
  return dir;
}

const keep = (dist, id, rules, from) => new Promise((ok, fail) => execFile(process.execPath,
  [join(tools, 'keep-builds.mjs'), dist, '--id', id, '--rules', String(rules), '--protocol', '1', '--label', `Version 0.1.0 · web ${id}`, ...(from ? ['--from', from] : [])],
  { env }, (e, stdout, stderr) => (e ? fail(new Error(stderr || String(e))) : ok(stdout))));
const carry = (dist, from) => new Promise((ok, fail) => execFile(process.execPath, [join(tools, 'carry-assets.mjs'), dist, '--from', from],
  { env }, (e, stdout, stderr) => (e ? fail(new Error(stderr || String(e))) : ok(stdout))));
const versions = (dist) => JSON.parse(readFileSync(join(dist, 'versions.json'), 'utf8'));

describe('keep-builds', () => {
  it('publishes the build at v/<id>/ with a base at the site root, and the refs check follows the base', async () => {
    const dist = build('aaa1111');
    await keep(dist, 'aaa1111', 49);
    const page = readFileSync(join(dist, 'v/aaa1111/index.html'), 'utf8');
    expect(page).toMatch(/<head>\n {4}<base href="\.\.\/\.\.\/" \/>/);
    const list = versions(dist);
    expect(list.builds.map((b) => [b.id, b.rules, b.retiredAt])).toEqual([['aaa1111', 49, null]]);
    expect(list.publicFiles).toEqual(['terrain/grass-aaa1111.webp']);
    const r = spawnSync(process.execPath, [join(tools, 'check-dist-refs.mjs'), dist], { encoding: 'utf8' });
    expect(r.status, r.stderr).toBe(0);
  });

  it('keeps the live site\'s earlier builds and the root files they used, and retires the one it replaces', async () => {
    const first = build('aaa1111');
    await keep(first, 'aaa1111', 49);
    const live = await startStaticServer(0, first);
    const next = build('bbb2222');
    try {
      await carry(next, live.url);
      const out = await keep(next, 'bbb2222', 50, live.url);
      expect(out).toContain('kept 1 earlier builds');
    } finally {
      await live.close();
    }
    const list = versions(next);
    expect(list.builds.map((b) => [b.id, b.rules])).toEqual([['bbb2222', 50], ['aaa1111', 49]]);
    expect(list.builds[0].retiredAt).toBeNull();
    expect(typeof list.builds[1].retiredAt).toBe('string');
    expect(existsSync(join(next, 'v/aaa1111/index.html'))).toBe(true);
    // The old build's script, art and root file are all still on the site.
    expect(existsSync(join(next, 'assets/index-aaa1111.js'))).toBe(true);
    expect(existsSync(join(next, 'terrain/grass-aaa1111.webp'))).toBe(true);
    const r = spawnSync(process.execPath, [join(tools, 'check-dist-refs.mjs'), next], { encoding: 'utf8' });
    expect(r.status, r.stderr).toBe(0);
  });

  it('drops a build retired longer ago than --keep-days', async () => {
    const first = build('aaa1111');
    await keep(first, 'aaa1111', 49);
    const list = versions(first);
    list.builds.push({ id: 'old0000', label: '', rules: 48, protocol: 1, deployedAt: '2026-01-01T00:00:00Z', retiredAt: '2026-01-02T00:00:00Z' });
    writeFileSync(join(first, 'versions.json'), JSON.stringify(list));
    const live = await startStaticServer(0, first);
    const next = build('bbb2222');
    try {
      await keep(next, 'bbb2222', 50, live.url);
    } finally {
      await live.close();
    }
    expect(versions(next).builds.map((b) => b.id)).toEqual(['bbb2222', 'aaa1111']);
  });
});
