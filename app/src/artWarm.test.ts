import { afterEach, describe, expect, it, vi } from 'vitest';
import { smallArtUrls, warmImages } from './artWarm';

afterEach(() => vi.unstubAllGlobals());

describe('background fetch of the small paintings', () => {
  it('lists only the small files: unit cards and thumbs, unlock and resource paintings at 64 px', () => {
    const urls = smallArtUrls();
    expect(urls.length).toBeGreaterThan(150);
    for (const url of urls) expect(url, url).toMatch(/-(64|card)[.-]/);
    expect(new Set(urls).size).toBe(urls.length);
  });

  it('fetches every file, four at a time, and survives a file that fails', async () => {
    let open = 0, most = 0;
    const made: { src: string; onload?: () => void; onerror?: () => void }[] = [];
    vi.stubGlobal('Image', class {
      onload?: () => void; onerror?: () => void; decoding = ''; private url = '';
      get src() { return this.url; }
      set src(v: string) { this.url = v; open++; most = Math.max(most, open); made.push(this); setTimeout(() => { open--; (v.includes('bad') ? this.onerror : this.onload)?.(); }, 1); }
    });
    const urls = Array.from({ length: 11 }, (_, i) => (i === 3 ? 'bad.png' : `a${i}.png`));
    await warmImages(urls, (cb) => setTimeout(cb, 0));
    expect(made.map((m) => m.src).sort()).toEqual([...urls].sort());
    expect(most).toBeLessThanOrEqual(4);
  });
});
