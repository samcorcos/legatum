// Coach questions (designer, 2026-11-15): Hard plus "why did you do that?". With the toggle on,
// End Turn asks the engine what Hard would have ordered in his seat (`crates/ai/src/why.rs`); a
// material difference comes back as one question, shown at the start of the next turn. Answers are
// kept in memory for the session (`unwrittenAnswers`). The toggle, the asked keys and the last category
// live in the save file wrapper, never in the game state, so hashes and rules do not change.
import { buildLabel } from './buildLabel';
import type { Order } from './types';

export type WhyCategory = 'war_peace' | 'settle' | 'production' | 'purchase' | 'window'
  | 'garrison' | 'border_army' | 'war_reserve' | 'great_work';

export interface WhyQuestion {
  turn: number;
  category: WhyCategory;
  key: string;
  materiality: number;
  text: string;
  evidence: unknown;
}

/** What the app remembers between turns (saved with the game). */
export interface WhyMemory {
  enabled: boolean;
  /** Keys of the standing facts already asked about. */
  asked: string[];
  /** The last question's turn and category. */
  last: [number, WhyCategory] | null;
  /** Categories the player skipped, once per unanswered question. */
  skipped: WhyCategory[];
}

export const NO_WHY: WhyMemory = { enabled: false, asked: [], last: null, skipped: [] };

export interface WhyEngine {
  whyReview(player: number, orders: Order[], asked: { keys: string[]; last: [number, WhyCategory] | null; skipped: WhyCategory[] }): { question: WhyQuestion | null; candidates: WhyQuestion[]; hard_orders: Order[] };
  trainingGameId(): string;
}

/** After a question is shown, remember it so it is never asked twice and never twice in a row by category. */
export function remember(memory: WhyMemory, q: WhyQuestion): WhyMemory {
  return { ...memory, asked: memory.asked.includes(q.key) ? memory.asked : [...memory.asked, q.key], last: [q.turn, q.category] };
}

export interface WhyAnswerLine {
  type: 'answer';
  game: string;
  turn: number;
  category: WhyCategory;
  question: string;
  evidence: unknown;
  /** His own words, or 'skipped'. */
  answer: string;
  build: string;
}

export function answerLine(game: string, q: WhyQuestion, answer: string | null): WhyAnswerLine {
  const text = (answer ?? '').trim();
  return {
    type: 'answer', game, turn: q.turn, category: q.category, question: q.text,
    evidence: q.evidence, answer: text || 'skipped', build: buildLabel,
  };
}

const unwritten: string[] = [];

/** Keep one answer as a JSON line for this session. */
export function writeAnswer(line: WhyAnswerLine): void {
  unwritten.push(JSON.stringify(line));
}

export const unwrittenAnswers = (): readonly string[] => unwritten;
