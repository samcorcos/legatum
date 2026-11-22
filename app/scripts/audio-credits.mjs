// Renders app/public/audio/CREDITS.md from manifest.json. The in-game Credits (Guide) reads the
// same manifest. Run: node app/scripts/audio-credits.mjs   (audio-manifest.test.mjs checks it's current)
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

export const AUDIO_DIR = path.join(path.dirname(fileURLToPath(import.meta.url)), '../public/audio');

export const LICENSE_URLS = {
  'CC0-1.0': 'https://creativecommons.org/publicdomain/zero/1.0/',
  'CC-BY-3.0': 'https://creativecommons.org/licenses/by/3.0/',
  'CC-BY-4.0': 'https://creativecommons.org/licenses/by/4.0/',
  'CC-BY-SA-3.0': 'https://creativecommons.org/licenses/by-sa/3.0/',
  'CC-BY-SA-4.0': 'https://creativecommons.org/licenses/by-sa/4.0/',
  'GPL-2.0-or-later': 'https://www.gnu.org/licenses/old-licenses/gpl-2.0.html',
  'GPL-3.0-or-later': 'https://www.gnu.org/licenses/gpl-3.0.html',
  'LicenseRef-OGA-BY-3.0': 'https://static.opengameart.org/OGA-BY-3.0.txt',
  'LicenseRef-Legatum-Original': '',
};

export function renderCredits(manifest) {
  const lines = [
    '# Audio credits',
    '',
    'Generated from `manifest.json` by `app/scripts/audio-credits.mjs`; do not edit by hand.',
    'License texts are in `licenses/`.',
    '',
  ];
  const required = manifest.filter((e) => e.license !== 'CC0-1.0');
  if (required.length) {
    lines.push('## Required attributions', '');
    for (const e of required) lines.push(`- ${e.attribution}`);
    lines.push('');
  }
  lines.push('## Files', '');
  const eclipse = manifest.filter((e) => e.file === 'music/atlas.m4a' || e.file === 'music/war.m4a');
  if (eclipse.length === 2 && eclipse[0].source === eclipse[1].source) {
    lines.push(`Both \`music/atlas.m4a\` and \`music/war.m4a\` are cut from the same CC0 work, "${eclipse[0].title}" by ${eclipse[0].author}.`, '');
  }
  for (const e of manifest) {
    const url = LICENSE_URLS[e.license];
    lines.push(`- \`${e.file}\` — "${e.title}" by ${e.author} (${e.source}), ${e.license}${url ? ` (${url})` : ''}. ${capital(e.modified)}.`);
  }
  lines.push('');
  return lines.join('\n');
}

const capital = (s) => s.charAt(0).toUpperCase() + s.slice(1);

export function readManifest() {
  return JSON.parse(fs.readFileSync(path.join(AUDIO_DIR, 'manifest.json'), 'utf8'));
}

if (process.argv[1] && fileURLToPath(import.meta.url) === path.resolve(process.argv[1])) {
  fs.writeFileSync(path.join(AUDIO_DIR, 'CREDITS.md'), renderCredits(readManifest()));
  console.log('wrote', path.join(AUDIO_DIR, 'CREDITS.md'));
}
