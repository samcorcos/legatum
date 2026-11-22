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
   * Who has scored its +1 VP. The point is per captor, not per team: `you` when your own capture scored it,
   * `teammate` when only a teammate's did (you can still score it yourself), otherwise null.
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
  if (mark.own) return 'Your original capital. Each rival that captures it scores +1 VP.';
  const empire = empireOfPlayer(view, mark.founder);
  if (mark.teammate) return `Your teammate's original capital (${empire}). Each rival that captures it scores +1 VP.`;
  if (mark.scored === 'you') return `${possessive(empire)} original capital. You scored its +1 VP when you took it; taking it again scores nothing.`;
  if (mark.scored === 'teammate') return mark.heldByTeam
    ? `${possessive(empire)} original capital. Your teammate scored it. If a rival takes it, you can score it by capturing it yourself.`
    : `${possessive(empire)} original capital. Your teammate has scored it. You can still score it by capturing it.`;
  return `${possessive(empire)} original capital. Capture it to score +1 VP, kept for good. Each empire scores it once.`;
}

/** Convenience for surfaces that hold a city: its line, or null for any other city. */
export function capitalLineOf(view: MarkView, city: MarkCity): string | null {
  const mark = capitalMark(view, city);
  return mark && capitalLine(view, mark);
}

/** What the map draws: a filled star in the founder's ink, outlined only once you yourself have scored it. */
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
