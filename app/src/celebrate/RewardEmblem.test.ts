import { expect, it } from 'vitest';
import { readdirSync, readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';

const dir = fileURLToPath(new URL('../assets/rewards/', import.meta.url));
const KINDS = ['camp', 'hall', 'ruin', 'captured', 'founded', 'growth', 'milestone', 'greatwork', 'tech'];

it('every emblem kind has an SVG that is one currentColor ink with prefixed ids', () => {
  const files = readdirSync(dir).filter((f) => f.endsWith('.svg')).sort();
  expect(files).toEqual(KINDS.map((k) => `emblem-${k}.svg`).sort());
  for (const kind of KINDS) {
    const svg = readFileSync(`${dir}emblem-${kind}.svg`, 'utf8');
    expect(svg).toContain('fill="currentColor"');
    expect(svg).not.toMatch(/fill="#(?!000|fff)[0-9a-f]+"/i);
    for (const id of svg.matchAll(/ id="([^"]+)"/g)) expect(id[1].startsWith(`emblem-${kind}-`)).toBe(true);
  }
});
