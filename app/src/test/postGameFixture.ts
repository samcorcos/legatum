// Current-rules copies of two recorded end states for the end-of-game screens: the Swiss win
// (turn 203, held 10 VP for 10 turns) and a game lost to the AI. The original fixtures remain
// untouched. The copies retain core per-turn history, with archive and AI memory stripped.
// Read in Node (Playwright specs) only; the JSON is passed to the app as an opaque save string.
import { readFileSync } from 'node:fs';

export type PostGameFixture = 'swiss-win' | 'rival-win';

export function postGameSave(f: PostGameFixture): { game: string; turn: number } {
  const game = readFileSync(new URL(`./fixtures/postgame-${f}.current.json`, import.meta.url), 'utf8');
  // Only the top-level turn is read; the string itself stays exact (it holds u64 words).
  const turn = (JSON.parse(game) as { turn: number }).turn;
  return { game, turn };
}
