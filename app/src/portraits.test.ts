// @vitest-environment node
// Every playable faction uses a paired people portrait (one man and one woman, unnamed), with no
// single-leader fallback.
import { existsSync, readdirSync, readFileSync } from 'node:fs';
import { describe, expect, it } from 'vitest';
import { FACTIONS } from './data';

const art = new URL('./assets/art/', import.meta.url).pathname;
// The full-size `people-<id>.png` masters live outside git: one counts as present when it is in the tree or listed in the
// art-sources manifest (tools/art/sources-manifest.json, fetched with tools/art/fetch-sources.py).
const archived = new Set<string>((JSON.parse(readFileSync(new URL('../../tools/art/sources-manifest.json', import.meta.url), 'utf8')) as { files: { path: string }[] }).files.map((f) => f.path));

describe('faction portraits', () => {
  it('gives every playable faction a people portrait set with its prompt', () => {
    for (const f of FACTIONS) {
      const id = f.toLowerCase();
      for (const file of [`people-${id}.png`, `people-${id}-512.png`, `people-${id}-360.jpg`, `people-${id}.prompt.txt`]) {
        expect(existsSync(`${art}${file}`) || archived.has(`app/src/assets/art/${file}`), `${f} is missing ${file}`).toBe(true);
      }
    }
  });

  it('keeps no single-leader portrait art', () => {
    expect(readdirSync(art).filter((n) => n.startsWith('leader-'))).toEqual([]);
  });
});
