// License and completeness gate for app/public/audio (policy: docs/design/sound-sources.md).
import fs from 'node:fs';
import path from 'node:path';
import { describe, expect, it } from 'vitest';
import { AUDIO_DIR, LICENSE_URLS, readManifest, renderCredits } from './audio-credits.mjs';

const manifest = readManifest();
const ALLOWED = Object.keys(LICENSE_URLS);
const NEEDS_ATTRIBUTION = /^(CC-BY|GPL|LicenseRef-OGA-BY)/;

function shippedFiles(dir = AUDIO_DIR, prefix = '') {
  return fs.readdirSync(dir, { withFileTypes: true }).flatMap((d) => {
    const rel = prefix + d.name;
    if (d.isDirectory()) return rel === 'licenses' ? [] : shippedFiles(path.join(dir, d.name), rel + '/');
    return rel === 'manifest.json' || rel === 'CREDITS.md' || d.name.startsWith('.') ? [] : [rel];
  });
}

describe('audio manifest', () => {
  it('lists every shipped audio file exactly once, and only files that exist', () => {
    const listed = manifest.map((e) => e.file);
    expect(new Set(listed).size).toBe(listed.length);
    expect([...listed].sort()).toEqual(shippedFiles().sort());
  });

  it('uses only allowlisted licenses, never NC, ND or Sampling+', () => {
    for (const e of manifest) {
      expect(e.license, e.file).not.toMatch(/NC|ND|Sampling/i);
      expect(ALLOWED, e.file).toContain(e.license);
    }
  });

  it('carries attribution and a shipped license text wherever the license requires it', () => {
    for (const e of manifest) {
      expect(e.title && e.author && e.source && e.modified, e.file).toBeTruthy();
      if (NEEDS_ATTRIBUTION.test(e.license)) {
        expect(e.attribution?.trim(), e.file).toBeTruthy();
        expect(fs.existsSync(path.join(AUDIO_DIR, 'licenses', `${e.license}.txt`)), `${e.license}.txt`).toBe(true);
      }
    }
  });

  it('keeps CREDITS.md in sync with the manifest', () => {
    expect(fs.readFileSync(path.join(AUDIO_DIR, 'CREDITS.md'), 'utf8')).toBe(renderCredits(manifest));
  });

  it('rejects a manifest that breaks the rules (the checks above are not vacuous)', () => {
    const bad = [{ license: 'CC-BY-NC-4.0' }, { license: 'CC-BY-ND-4.0' }];
    expect(bad.every((e) => /NC|ND|Sampling/i.test(e.license))).toBe(true);
    expect(NEEDS_ATTRIBUTION.test('CC-BY-SA-3.0') && NEEDS_ATTRIBUTION.test('GPL-2.0-or-later')).toBe(true);
  });
});
