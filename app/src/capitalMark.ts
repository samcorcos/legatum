// The star that marks an original capital for the whole game, even after it is captured.
// It follows the scoring rule (core `City::original_capital`), not the current capital (`is_capital`),
// which moves to a replacement city that scores nothing. One place decides the mark and its words.
import { empireOfPlayer } from './empires';
import { playerInk } from './theme';
import type { CityView, PlayerId, PlayerView } from './types';

export interface CapitalMark {
  /** The empire that founded the city: whose ink the star wears. */
  founder: PlayerId;
  ink: string;
  /** You founded it. */
  own: boolean;
  /** A teammate founded it. */
  teammate: boolean;
  /**
   * Who holds its 1 VP now (core `scoring::capitals_held`, the view's `capitals_taken`): `you` when you hold
   * the city, `teammate` when a teammate does, otherwise null. The point is held, not kept: it moves with the city.
   */
  scored: 'you' | 'teammate' | null;
  /** You or a teammate hold it now, so you cannot capture it until a rival takes it. */
  heldByTeam: boolean;
}

type MarkView = Pick<PlayerView, 'me' | 'team' | 'players'>;
type MarkCity = Pick<CityView, 'id' | 'owner' | 'original_capital' | 'original_capital_of'>;

/** The mark for a city, or null when it is not an original capital (a replacement capital gets none). */
export function capitalMark(view: MarkView, city: MarkCity): CapitalMark | null {
  if (!city.original_capital) return null;
  const founder = city.original_capital_of ?? city.owner;
  const founderTeam = view.players.find((p) => p.id === founder)?.team;
  const own = founder === view.me;
  const teammate = !own && founderTeam === view.team;
  const took = (p: { capitals_taken?: [number, string][] }) => !!p.capitals_taken?.some(([id]) => id === city.id);
  const me = view.players.find((p) => p.id === view.me);
  const scored = me && took(me) ? 'you' : view.players.some((p) => p.team === view.team && took(p)) ? 'teammate' : null;
  const heldByTeam = view.players.find((p) => p.id === city.owner)?.team === view.team;
  return { founder, ink: playerInk(founder), own, teammate, scored, heldByTeam };
}

/** "Switzerland's", "Mongols'". */
const possessive = (name: string) => (name.endsWith('s') ? `${name}'` : `${name}'s`);

/** The one plain line every surface prints for a mark. */
export function capitalLine(view: Pick<MarkView, 'players'>, mark: CapitalMark): string {
  if (mark.own) return 'Your original capital. A rival scores 1 VP while it holds it.';
  const empire = empireOfPlayer(view, mark.founder);
  if (mark.teammate) return `Your teammate's original capital (${empire}). A rival scores 1 VP while it holds it.`;
  if (mark.scored === 'you') return `${possessive(empire)} original capital. You score 1 VP while you hold it.`;
  if (mark.scored === 'teammate') return `${possessive(empire)} original capital. Your teammate scores 1 VP while holding it.`;
  return `${possessive(empire)} original capital. Hold it to score 1 VP.`;
}

/** Convenience for surfaces that hold a city: its line, or null for any other city. */
export function capitalLineOf(view: MarkView, city: MarkCity): string | null {
  const mark = capitalMark(view, city);
  return mark && capitalLine(view, mark);
}

/** What the map draws: a filled star in the founder's ink, outlined while you hold its point. */
export interface CapitalStar { ink: string; scored: boolean }

export function capitalStar(view: MarkView, city: MarkCity): CapitalStar | null {
  const mark = capitalMark(view, city);
  return mark && { ink: mark.ink, scored: mark.scored === 'you' };
}

/** Whether a city is a current capital that is not an original one: no star, and capturing it scores nothing. */
export function isReplacementCapital(city: Pick<CityView, 'visible' | 'is_capital' | 'original_capital'>): boolean {
  return city.visible && city.is_capital && !city.original_capital;
}

/** The founder's original capital, if the viewer knows where it is (it may be held by anyone). */
function originalOf(view: Pick<PlayerView, 'cities'>, owner: PlayerId) {
  return view.cities.find((c) => c.original_capital && c.original_capital_of === owner);
}

/** Kicker hover for a replacement capital; null for any other city. */
export function replacementKickerLine(view: Pick<PlayerView, 'cities'>, city: CityView): string | null {
  if (!isReplacementCapital(city)) return null;
  const original = originalOf(view, city.owner);
  return `Capital since ${original ? `${original.name} fell` : 'the original capital was lost'}. Capturing it scores no VP.`;
}

/** Tile-card line for a replacement capital; null for any other city. */
export function replacementTileLine(view: Pick<PlayerView, 'cities' | 'players' | 'me'>, city: CityView): string | null {
  if (!isReplacementCapital(city)) return null;
  const original = originalOf(view, city.owner);
  if (!original) return 'Capturing it scores no VP: it is not an original capital.';
  const whose = city.owner === view.me ? 'your' : possessive(empireOfPlayer(view, city.owner));
  return `Capturing it scores no VP: ${whose} original capital is ${original.name}.`;
}
