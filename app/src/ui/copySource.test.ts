// The copy lint over source: the phrase rules marked `source: true` in test/copyRules.ts, run over the app's
// and the core's code with comments stripped, so mechanics-speak can't reach a screen the rendered corpus
// (ui/textArtifacts.test.tsx) doesn't cover: component JSX, tooltips, notices, report lines, refusal reasons
// and breakdown labels from crates/core. A hit names the file and line.
import { readdirSync, readFileSync, statSync } from 'node:fs';
import { join, relative } from 'node:path';
import { expect, it } from 'vitest';
import { BANNED, EXCEPTIONS } from '../test/copyRules';

const APP_SRC = join(__dirname, '..');
const CORE_SRC = join(__dirname, '../../../crates/core/src');

function files(dir: string, ext: RegExp, skip: RegExp): string[] {
  return readdirSync(dir).flatMap((name) => {
    const path = join(dir, name);
    if (statSync(path).isDirectory()) return skip.test(`${name}/`) ? [] : files(path, ext, skip);
    return ext.test(name) && !skip.test(name) ? [path] : [];
  });
}

/** Source with comments blanked out (line numbers kept). */
function code(text: string): string {
  return text
    .replace(/\/\*[\s\S]*?\*\//g, (m) => m.replace(/[^\n]/g, ' '))
    .replace(/(^|[\s;{}(),])\/\/.*$/gm, '$1');
}

/** The text a line can show a player: its string literals, and for TSX the JSX text after a tag. Code (Rust's `Vec::retain`, a `multiplier` variable) is not text. */
function textOf(line: string, rust: boolean): string {
  const literals = rust ? line.match(/"(?:[^"\\]|\\.)*"/g) : line.match(/'(?:[^'\\]|\\.)*'|"(?:[^"\\]|\\.)*"|`(?:[^`\\]|\\.)*`/g);
  const jsx = rust ? [] : [...line.matchAll(/>([^<>{}]{3,})/g)].map((m) => m[1]);
  return [...(literals ?? []), ...jsx].join(' | ');
}

function hits(paths: string[], rust: boolean): string[] {
  const out: string[] = [];
  for (const path of paths) {
    const lines = code(readFileSync(path, 'utf8')).split('\n');
    lines.forEach((source, i) => {
      const line = textOf(source, rust);
      for (const r of BANNED.filter((b) => b.source)) {
        const m = r.pattern.exec(line);
        if (m && !EXCEPTIONS.some((e) => e.rule.toLowerCase() === m[0].toLowerCase() && line.includes(e.text))) {
          out.push(`${relative(join(APP_SRC, '../..'), path)}:${i + 1} | banned "${m[0]}" (${r.reason}; write ${r.instead}) | ${source.trim().slice(0, 140)}`);
        }
      }
    });
  }
  return out;
}

it('no app source string uses banned mechanics-speak', () => {
  const app = files(APP_SRC, /\.(ts|tsx)$/, /\.test\.tsx?$|^wasm-pkg\/$|^test\/$|\.d\.ts$|^copyRules\.ts$|^playwright\.config\.ts$/);
  const found = hits(app, false);
  expect(found, found.join('\n')).toEqual([]);
});

it('no core string that reaches the player uses banned mechanics-speak', () => {
  const core = files(CORE_SRC, /\.rs$/, /tests?\.rs$|^tests\/$/);
  const found = hits(core, true);
  expect(found, found.join('\n')).toEqual([]);
});
