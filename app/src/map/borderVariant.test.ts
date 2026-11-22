import { afterEach, describe, expect, it, vi } from 'vitest';

async function load(search: string, stored: string | null) {
  vi.resetModules();
  const store = new Map<string, string>(stored ? [['legatum.dev.borders', stored]] : []);
  vi.stubGlobal('location', { search });
  vi.stubGlobal('localStorage', { getItem: (k: string) => store.get(k) ?? null, setItem: (k: string, v: string) => void store.set(k, v) });
  return { mod: await import('./borderVariant'), store };
}

afterEach(() => vi.unstubAllGlobals());

describe('the dev border-look switch', () => {
  it('shows the default look when nothing is chosen', async () => {
    const { mod } = await load('', null);
    expect(mod.getBorderVariant()).toBe(mod.DEFAULT_BORDER_VARIANT);
  });
  it('takes ?borders= from the URL, remembers it, and ignores unknown values', async () => {
    const a = await load('?borders=b', null);
    expect(a.mod.getBorderVariant()).toBe('b');
    expect(a.store.get('legatum.dev.borders')).toBe('b');
    const bad = await load('?borders=zzz', null);
    expect(bad.mod.getBorderVariant()).toBe(bad.mod.DEFAULT_BORDER_VARIANT);
  });
  it('falls back to the remembered choice, and switches at runtime', async () => {
    const { mod, store } = await load('', 'current');
    expect(mod.getBorderVariant()).toBe('current');
    mod.setBorderVariant('a');
    expect(mod.getBorderVariant()).toBe('a');
    expect(store.get('legatum.dev.borders')).toBe('a');
  });
});
