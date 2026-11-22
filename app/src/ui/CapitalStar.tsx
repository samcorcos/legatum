import { capitalLine, type CapitalMark } from '../capitalMark';
import { Amounts } from './Amounts';
import type { PlayerView } from '../types';

/** The four-point house star, with the narrow waist of the map's filled mark or the wider one of its outlined (scored) form. */
const starPath = (r: number, waist: number) => {
  const k = waist * r;
  return `M0 ${-r}L${k} ${-k}L${r} 0L${k} ${k}L0 ${r}L${-k} ${k}L${-r} 0L${-k} ${-k}Z`;
};

/**
 * An original capital's star as the map draws it: filled in the founder's ink, outlined while you
 * hold it yourself (its 1 VP is yours while you do). The shape differs, so colour is never the only cue.
 */
export function CapitalStarGlyph({ mark, size = 12 }: { mark: CapitalMark; size?: number }) {
  const scored = mark.scored === 'you';
  return (
    <svg className="capital-star" width={size} height={size} viewBox="-6 -6 12 12" role="img" aria-label={scored ? 'Original capital, its VP held by you' : 'Original capital'}>
      <path d={starPath(5.4, scored ? 0.45 : 0.3)} fill={scored ? 'none' : mark.ink} stroke={mark.ink} strokeWidth={scored ? 1.1 : 0.6} strokeLinejoin="round" />
    </svg>
  );
}

/** The plain ornament on every other city's panel kicker: the star is reserved for original capitals. */
export function KickerDot() {
  return <span className="kicker-dot" aria-hidden="true" />;
}

/** The star and the one plain line about it. */
export function CapitalNote({ view, mark, testId }: { view: Pick<PlayerView, 'players'>; mark: CapitalMark; testId?: string }) {
  return (
    <div className="capital-note" data-testid={testId}>
      <CapitalStarGlyph mark={mark} />
      <span><Amounts text={capitalLine(view, mark)} /></span>
    </div>
  );
}

/** A plain note about a replacement capital: no star, and what capturing it is worth. */
export function ReplacementNote({ text, testId }: { text: string; testId?: string }) {
  return <div className="capital-note replacement" data-testid={testId}><span><Amounts text={text} /></span></div>;
}
