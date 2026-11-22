import { afterEach, expect, it, vi } from 'vitest';
import { onWonderArtLoad, wonderImage } from './wonderArt';

afterEach(() => vi.unstubAllGlobals());

it('notifies the map when a natural-wonder image finishes loading or fails', () => {
  const created: { src: string; onload: null | (() => void); onerror: null | (() => void) }[] = [];
  vi.stubGlobal('Image', class {
    src = '';
    onload: null | (() => void) = null;
    onerror: null | (() => void) = null;
    complete = false;
    naturalWidth = 0;
    constructor() { created.push(this); }
  });
  const redraw = vi.fn();
  const off = onWonderArtLoad(redraw);
  wonderImage('MountFuji');
  expect(created).toHaveLength(1);
  expect(created[0].src).toContain('mount-fuji-b-128.png');
  created[0].onload?.();
  expect(redraw).toHaveBeenCalledTimes(1);
  off();
  wonderImage('VictoriaFalls');
  created[1].onerror?.();
  expect(redraw).toHaveBeenCalledTimes(1);
});
