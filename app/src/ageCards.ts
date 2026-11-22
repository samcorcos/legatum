// World Age cards as the player reads them. Everything here echoes the core's rule (`objectives::Objective`); the app adds no rule.
import type { PlayerView } from './types';

export type CardRule = { kind: string; [field: string]: unknown };

/** One plain line on what a card asks of you: the benefit is always 1 VP, kept for good. */
export function cardHint(kind: string | undefined): string {
  switch (kind) {
    case 'Veterans': return 'Counts levels your units gain in battle from now on; levels they already have do not count';
    case 'HumbleTheMighty': return 'Counts captures you make from now on; capitals and wonder cities do not count';
    case 'Breach': return 'Counts captures you make from now on; capitals and wonder cities do not count';
    case 'Cities': return 'Counts cities you gain from now on, founded or captured';
    default: return '';
  }
}

/** A card's short name for compact places: "Veterans", "Humble the mighty", "Breach", "Dominion". */
export const cardName = (text: string): string => text.split(':')[0];

/** A card's name and its plain ask, for places where the ask is not on screen nearby: "Dominion, gain 4 cities". */
export const cardNameAndAsk = (text: string): string => (text.includes(':') ? `${cardName(text)}, ${text.slice(text.indexOf(':') + 1).trim()}` : text);

/** The meter of a card that counts something done in play: how far the Age's own count has come, in words. */
export function cardMeter(objective: CardRule, current: number, target: number): { value: number; target: number; label: string } | null {
  switch (objective.kind) {
    case 'Veterans': return { value: current, target, label: `Promotions earned since the Age began: ${Math.min(current, target)} of ${target}` };
    case 'Breach': return { value: current, target, label: `Walled cities captured since the Age began: ${Math.min(current, target)} of ${target}` };
    case 'HumbleTheMighty': return { value: current, target, label: `Cities captured from an empire ahead of you on VP since the Age began: ${Math.min(current, target)} of ${target}` };
    case 'Population': return { value: current, target, label: `Population grown since the Age began: ${Math.min(current, target)} of ${target}` };
    case 'Cities': return { value: current, target, label: `Cities gained since the Age began: ${Math.min(current, target)} of ${target}` };
    case 'Territory': return { value: current, target, label: `Tiles of territory gained since the Age began: ${Math.min(current, target)} of ${target}` };
    default: return null;
  }
}

/** The turn and card with which you scored an Age (null if you have not). */
export function scoredCard(view: PlayerView, era: number): { turn: number; slot: number } | null {
  const s = view.eras?.find((e) => e.era === era)?.scored_by.find((x) => x.player === view.me);
  return s ? { turn: s.turn, slot: s.slot } : null;
}
