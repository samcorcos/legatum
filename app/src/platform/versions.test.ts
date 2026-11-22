import 'fake-indexeddb/auto';
import { afterEach, describe, expect, it } from 'vitest';
import { buildForRoom, buildForSave, fetchVersions, handOffSave, isBehind, parseVersions, readLaunch, takeHandedOffSave, type SiteBuild } from './versions';

const b = (id: string, rules: number, label = `Version 0.1.0 · web ${id}`): SiteBuild => ({ id, label, rules, protocol: 1, deployedAt: '2026-11-22T00:00:00Z', retiredAt: null });
// Newest first, as keep-builds.mjs writes them.
const site = [b('ccc3333', 50), b('bbb2222', 50), b('aaa1111', 49)];

describe('site builds', () => {
  it('reads versions.json and ignores what is not a build', () => {
    expect(parseVersions({ builds: [{ id: 'ccc3333', rules: 50, protocol: 1, label: 'x', deployedAt: 'd', retiredAt: null }, { id: 'Bad id', rules: 1 }, null] }))
      .toEqual([{ id: 'ccc3333', rules: 50, protocol: 1, label: 'x', deployedAt: 'd', retiredAt: null }]);
    expect(parseVersions({})).toBeNull();
    expect(parseVersions({ builds: [] })).toBeNull();
  });

  it('fetches the list from the site root, and gives null when it is missing', async () => {
    const asked: string[] = [];
    const ok = await fetchVersions(async (url) => { asked.push(url); return { ok: true, json: async () => ({ builds: [{ id: 'aaa1111', rules: 49 }] }) }; });
    expect(ok?.[0].id).toBe('aaa1111');
    expect(asked[0]).toMatch(/\/versions\.json$/);
    expect(await fetchVersions(async () => ({ ok: false, json: async () => null }))).toBeNull();
    expect(await fetchVersions(async () => { throw new Error('offline'); })).toBeNull();
  });

  it('a tab is behind only when the newest build is another one', () => {
    expect(isBehind(site, 'bbb2222')).toBe(true);
    expect(isBehind(site, 'ccc3333')).toBe(false);
    expect(isBehind(null, 'bbb2222')).toBe(false);
    expect(isBehind(site, null)).toBe(false);
  });

  it('opens a save in the build that made it, else the newest with its rules', () => {
    expect(buildForSave(site, 50, 'bbb2222', 'aaa1111')?.id).toBe('bbb2222');
    expect(buildForSave(site, 50, 'gone000', 'aaa1111')?.id).toBe('ccc3333');
    expect(buildForSave(site, 50, undefined, 'aaa1111')?.id).toBe('ccc3333');
    expect(buildForSave(site, 49, undefined, 'ccc3333')?.id).toBe('aaa1111');
    // Rules the site no longer serves, or a save this build already is: nothing to open elsewhere.
    expect(buildForSave(site, 48, undefined, 'ccc3333')).toBeNull();
    expect(buildForSave(site, 50, 'ccc3333', 'ccc3333')).toBeNull();
    expect(buildForSave(null, 49)).toBeNull();
  });

  it('joins an online game on the build its host runs', () => {
    const room = { build: 'Version 0.1.0 · web aaa1111', rules_version: 49, protocol: 1 };
    expect(buildForRoom(site, room, 'ccc3333')?.id).toBe('aaa1111');
    expect(buildForRoom(site, { ...room, protocol: 2 }, 'ccc3333')).toBeNull();
    expect(buildForRoom(site, { ...room, build: 'Playtest 9' }, 'ccc3333')).toBeNull();
    expect(buildForRoom(site, room, 'aaa1111')).toBeNull();
  });
});

describe('handing a save to another build', () => {
  it('passes the text once, and ignores a stale one', async () => {
    expect(await handOffSave('{"format":"legatum-save"}')).toBe(true);
    expect(await takeHandedOffSave()).toBe('{"format":"legatum-save"}');
    expect(await takeHandedOffSave()).toBeNull();
    await handOffSave('old');
    expect(await takeHandedOffSave(Date.now() + 11 * 60_000)).toBeNull();
  });
});

describe('the launch another build asked for', () => {
  const g = globalThis as unknown as { location?: unknown; history?: unknown };
  afterEach(() => { delete g.location; delete g.history; });
  const at = (search: string) => {
    const replaced: string[] = [];
    g.location = { search, pathname: '/v/aaa1111/', hash: '' };
    g.history = { state: null, replaceState: (_s: unknown, _t: string, url: string) => replaced.push(url) };
    return replaced;
  };

  it('reads it and takes it out of the address, keeping other parameters', () => {
    const replaced = at('?continue=c-1&perf=1');
    expect(readLaunch()).toEqual({ kind: 'continue', campaign: 'c-1' });
    expect(replaced).toEqual(['/v/aaa1111/?perf=1']);
    at('?open=save');
    expect(readLaunch()).toEqual({ kind: 'open' });
    at('?join=room-7');
    expect(readLaunch()).toEqual({ kind: 'join', room: 'room-7' });
    at('?menu=multi');
    expect(readLaunch()).toEqual({ kind: 'menu', menu: 'multi' });
    const none = at('?menu=other');
    expect(readLaunch()).toBeNull();
    expect(none).toEqual([]);
  });
});
