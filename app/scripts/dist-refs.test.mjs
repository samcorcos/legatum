// tools/release/check-dist-refs.mjs (every file the web build points to is in it) and carry-assets.mjs (earlier
// builds' files stay on the site, so an open tab keeps its art after a deploy).
import { execFileSync, spawnSync } from 'node:child_process';
import { mkdtempSync, mkdirSync, writeFileSync, existsSync, readFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join, resolve } from 'node:path';
import { describe, expect, it } from 'vitest';
import { refsIn, resolveRef } from '../../tools/release/check-dist-refs-lib.mjs';

const tools = resolve(__dirname, '../../tools/release');

function site(files) {
  const dir = mkdtempSync(join(tmpdir(), 'dist-refs-'));
  mkdirSync(join(dir, 'assets'));
  for (const [name, text] of Object.entries(files)) writeFileSync(join(dir, name), text);
  return dir;
}

const check = (dir) => spawnSync(process.execPath, [join(tools, 'check-dist-refs.mjs'), dir], { encoding: 'utf8' });

describe('refsIn', () => {
  it('finds art, chunks, preloads, styles and page links', () => {
    const js = 'const a=new URL(`tank-AbC_12xy.webp`,import.meta.url).href,b=new URL("city-Q.png", import.meta.url);'
      + 'import("./lazy-X1.js");import{x}from"./dep-Y.js";__vite__mapDeps(["assets/lazy-Z.css"]);const k={"./assets/units/tank.png":a};';
    expect(refsIn(js, '.js')).toEqual(['tank-AbC_12xy.webp', 'city-Q.png', './lazy-X1.js', './dep-Y.js', 'assets/lazy-Z.css']);
    expect(refsIn('a{background:url(./paper-1.webp)}b{src:url("font-2.woff2")}', '.css')).toEqual(['./paper-1.webp', 'font-2.woff2']);
    expect(refsIn('<script src="./assets/index-1.js"></script><a href="https://x.org">', '.html')).toEqual(['./assets/index-1.js', 'https://x.org']);
  });

  it('resolves from the file that names it, and "assets/" from the site root', () => {
    expect(resolveRef('tank-1.webp', 'assets/')).toBe('assets/tank-1.webp');
    expect(resolveRef('assets/lazy-Z.css', 'assets/')).toBe('assets/lazy-Z.css');
    expect(resolveRef('./assets/index-1.js', '')).toBe('assets/index-1.js');
  });
});

describe('check-dist-refs', () => {
  it('passes when every reference names a file', () => {
    const dir = site({ 'index.html': '<script src="./assets/index-1.js"></script>', 'assets/index-1.js': 'new URL(`tank-1.webp`,import.meta.url)', 'assets/tank-1.webp': 'x' });
    const r = check(dir);
    expect(r.status, r.stderr).toBe(0);
  });

  it('fails on a picture the build points to but does not contain (the 2026-11-22 broken art)', () => {
    const dir = site({ 'index.html': '<script src="./assets/index-1.js"></script>', 'assets/index-1.js': 'new URL(`tank-1.png`,import.meta.url)', 'assets/tank-1.webp': 'x' });
    const r = check(dir);
    expect(r.status).toBe(1);
    expect(r.stderr).toContain('assets/index-1.js -> tank-1.png');
  });
});

describe('carry-assets', () => {
  it('copies another build\'s images and fonts without replacing this build\'s files, and records them', () => {
    const dist = site({ 'index.html': '<p>', 'assets/tank-NEW.webp': 'new' });
    const old = site({ 'index.html': '<p>', 'assets/tank-OLD.png': 'old', 'assets/tank-NEW.webp': 'other', 'assets/index-OLD.js': 'js' });
    execFileSync(process.execPath, [join(tools, 'carry-assets.mjs'), dist, '--from-dir', old]);
    expect(readFileSync(join(dist, 'assets/tank-OLD.png'), 'utf8')).toBe('old');
    expect(readFileSync(join(dist, 'assets/tank-NEW.webp'), 'utf8')).toBe('new');
    expect(existsSync(join(dist, 'assets/index-OLD.js'))).toBe(false);
    const history = JSON.parse(readFileSync(join(dist, 'asset-history.json'), 'utf8')).files;
    expect(Object.keys(history)).toEqual(['assets/tank-NEW.webp', 'assets/tank-OLD.png']);
  });
});
