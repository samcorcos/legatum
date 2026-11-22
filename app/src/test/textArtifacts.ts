// Player-facing text must be built from structured data, never patched up from prose. This is the
// guard's detector: a few patterns that only ever mean a join went wrong (",." after deleting a
// phrase, "undefined" from a missing field, an empty "· ·" slot). Used by ui/textArtifacts.test.tsx.
import { BANNED, EXCEPTIONS, GLOSSARY, MAX_SEMICOLONS, MAX_SENTENCE_WORDS } from './copyRules';

const PATTERNS: [string, RegExp][] = [
  ['punctuation run ",." ", ." ";." ":." ",," ",;"', /,\s*\.|[;:]\s*\.|,,|,\s*;|;\s*,/],
  ['doubled full stop (not an ellipsis)', /(?<!\.)\.\.(?!\.)/],
  ['space before , . ; :', / [,.;:](?!\d)/],
  ['empty brackets', /\(\s*\)|\[\s*\]/],
  ['leftover placeholder', /\bundefined\b|\bNaN\b|\bnull\b|\[object Object\]|\bInfinity\b/],
  ['empty separator slot', /·\s*·/],
  ['doubled space', /[^\S ]{2,}/],
  // A leading "·" is allowed: a continuation span ("· 7 of 10 VP") reads after its sibling's name.
  ['starts with punctuation', /^[\s]*[,.;:)\]/&|]/],
  // A trailing "—" is the empty-value mark ("Buy—") and a trailing "+" a details toggle, not dangling separators.
  ['ends with a separator', /[,;·/&|\-–]\s*$/],
];

/** Every artifact found in one block of text, each as "reason: text". */
export function artifactsIn(block: string): string[] {
  // A lone dash is the deliberate empty-value mark ("—" for no value), not a dangling separator.
  if (/^[\s\-–—]+$/.test(block)) return [];
  // A fraction split across elements ("7" then "/10") reads as one: its halves are not artifacts.
  if (/^\/\d+$|^\d+\/$/.test(block.trim())) return [];
  const text = block.replace(/ /g, ' ');
  const found: string[] = [];
  for (const [reason, re] of PATTERNS) {
    const m = re.exec(text);
    if (m) found.push(`${reason} [${JSON.stringify(text.slice(Math.max(0, m.index - 12), m.index + m[0].length + 12))}]`);
  }
  return found;
}

const decode = (s: string) => s.replace(/&quot;/g, '"').replace(/&#x27;|&#39;/g, "'").replace(/&lt;/g, '<').replace(/&gt;/g, '>').replace(/&nbsp;/g, ' ').replace(/&amp;/g, '&');

const BLOCK_TAGS = '(?:p|div|li|ul|ol|dl|dt|dd|h[1-6]|tr|td|th|table|section|article|header|footer|button|summary|details|dialog|nav|aside|figure|figcaption|br|select|option|label|form|blockquote)';

/** The text blocks of rendered markup: block elements split the text, inline elements join it, icons count as one glyph, decorative text (aria-hidden marks like a list dot) is not read, and title/aria-label/alt attributes are blocks of their own. */
export function htmlBlocks(html: string): string[] {
  const attrs = [...html.matchAll(/\s(?:title|aria-label|alt)="([^"]*)"/g)].map((m) => decode(m[1]));
  const text = html
    .replace(/<!--[\s\S]*?-->/g, '')
    .replace(/<(?:style|script)[\s\S]*?<\/(?:style|script)>/g, '')
    .replace(/<svg[\s\S]*?<\/svg>/g, '□')
    .replace(/<img[^>]*>/g, '□')
    .replace(/<(\w+)\b[^>]*aria-hidden="true"[^>]*>[^<]*<\/\1>/g, '')
    .replace(new RegExp(`</?${BLOCK_TAGS}\\b[^>]*>`, 'g'), '\n')
    .replace(/<[^>]+>/g, '');
  return [...decode(text).split('\n'), ...attrs].map((b) => b.trim()).filter(Boolean);
}

/** Check labelled blocks and return every finding as a readable line ("label | reason | text"). */
export function auditBlocks(label: string, blocks: Iterable<string>): string[] {
  const out: string[] = [];
  for (const block of blocks) for (const reason of artifactsIn(block)) out.push(`${label} | ${reason} | ${block.length > 160 ? `${block.slice(0, 160)}…` : block}`);
  return out;
}

// ---- The copy lint (user, 2026-10-17): plain, natural, in-theme words. Rules live in ./copyRules.ts; the
// style guide is "Game text: plain language" in the legatum-ui skill.

const words = (s: string) => s.split(/\s+/).filter((w) => /[\p{L}\p{N}]/u.test(w));
/** Sentences of a block: split after . ! ? (not inside a number like 1.5 or an ellipsis) and at line breaks. */
const sentencesOf = (block: string): string[] =>
  block.split(/(?<=[.!?])\s+(?=[\p{Lu}\p{N}"“(+−-])|\n+/u).map((s) => s.trim()).filter(Boolean);

/** Every plain-language problem in one block of player text, each as "reason: excerpt". */
export function copyIssuesIn(block: string): string[] {
  const text = block.replace(/ /g, ' ');
  const found: string[] = [];
  const near = (i: number, len: number) => JSON.stringify(text.slice(Math.max(0, i - 20), i + len + 20));
  const excused = (rule: string) => EXCEPTIONS.some((e) => e.rule.toLowerCase() === rule.toLowerCase() && text.includes(e.text));
  for (const r of BANNED) {
    const m = r.pattern.exec(text);
    if (m && !excused(m[0])) found.push(`banned "${m[0]}" (${r.reason}; write ${r.instead}) [${near(m.index, m[0].length)}]`);
  }
  for (const g of GLOSSARY) for (const syn of g.synonyms) {
    const m = syn.exec(text);
    if (m && !excused(m[0])) found.push(`"${m[0]}" is another name for ${g.canonical} [${near(m.index, m[0].length)}]`);
  }
  const doubled = /\b([\p{L}]+)\s+\1\b/iu.exec(text);
  if (doubled && !excused('doubled')) found.push(`doubled word "${doubled[0]}" [${near(doubled.index, doubled[0].length)}]`);
  for (const s of sentencesOf(text)) {
    const n = words(s).length;
    if (n > MAX_SENTENCE_WORDS && !excused('long')) found.push(`sentence of ${n} words (at most ${MAX_SENTENCE_WORDS}) [${JSON.stringify(s.slice(0, 90))}…]`);
    const semis = (s.match(/;/g) ?? []).length;
    if (semis >= MAX_SEMICOLONS && !excused('semicolons')) found.push(`${semis} semicolons in one sentence (one idea per sentence) [${JSON.stringify(s.slice(0, 90))}…]`);
  }
  return found;
}

/** The copy lint over labelled blocks, one line per finding ("label | reason | text"). */
export function auditCopy(label: string, blocks: Iterable<string>): string[] {
  const out: string[] = [];
  for (const block of blocks) for (const reason of copyIssuesIn(block)) out.push(`${label} | ${reason}`);
  return out;
}
