// The board-game rule, enforced (user, 2026-11-23, on Byzantium's "Thin levies" card: "this should just say
// 'you can field at most one military unit per city, plus one more'; all the other stuff is rules nuance").
// A card, sheet row or tip states the rule a player needs in a short line; exceptions and edge cases live in
// the Guide (`GuideContent.tsx`, or a sheet row's `strengthDetail`/`priceDetail`). Style guide: legatum-ui,
// "Game text: plain language". The plain-language reviewer aims for about 15 words; this check fails the
// clear cases so an overlong explainer can't slip in unreviewed.
import { expect, test } from 'vitest';
import { BUILDING_STATS, CLASS_NOTE, TECH_STATS, THIN_LEVIES, UNIT_STATS, SIEGE_HEADLINE, ARTILLERY_HEADLINE } from '../data';
import { FACTION_SHEETS } from '../factionSheets';

/** Most words a card, sheet row or tip may hold. */
const MAX_WORDS = 25;
/** Most sentences a faction sheet row may hold (one rule, two at most). */
const MAX_SHEET_SENTENCES = 2;

const words = (s: string) => s.trim().split(/\s+/).length;
const sentences = (s: string) => (s.match(/[.!?](?:\s|$)/g) ?? []).length;

function explainers(): [string, string][] {
  const out: [string, string][] = [];
  for (const [faction, sheet] of Object.entries(FACTION_SHEETS)) {
    sheet.pairs.forEach((p, i) => {
      if (!(p.strength)) {} else { out.push([`${faction} sheet row ${i + 1} strength`, p.strength]); }
      if (!(p.price)) {} else { out.push([`${faction} sheet row ${i + 1} price`, p.price]); }
    });
    if (!(sheet.lose)) {} else { out.push([`${faction} sheet lose`, sheet.lose]); }
    if (!(sheet.counteredBy)) {} else { out.push([`${faction} sheet countered by`, sheet.counteredBy]); }
  }
  for (const [kind, s] of Object.entries(UNIT_STATS)) out.push([`${kind} unit card`, s.blurb]);
  for (const [kind, s] of Object.entries(BUILDING_STATS)) out.push([`${kind} building card`, s.blurb]);
  for (const [tech, s] of Object.entries(TECH_STATS)) out.push([`${tech} technology card`, s.blurb]);
  for (const [cls, note] of Object.entries(CLASS_NOTE)) out.push([`${cls} class note`, note]);
  out.push(['Thin levies', THIN_LEVIES], ['Siege tip', SIEGE_HEADLINE], ['Indirect fire tip', ARTILLERY_HEADLINE]);
  return out;
}

test('every card, sheet row and tip is a short rule, with the fine print in the Guide', () => {
  const long = explainers().filter(([, text]) => words(text) > MAX_WORDS).map(([where, text]) => `${where} (${words(text)} words): ${text}`);
  expect(long).toEqual([]);
});

test('a faction sheet row says one rule, two sentences at most', () => {
  const long = explainers().filter(([where]) => where.includes(' sheet ')).filter(([, text]) => sentences(text) > MAX_SHEET_SENTENCES).map(([where, text]) => `${where}: ${text}`);
  expect(long).toEqual([]);
});

test('Thin levies is the headline only (user, 2026-11-23)', () => {
  expect(THIN_LEVIES).toBe('Thin levies: you can field at most one military unit per city, plus one more.');
});
