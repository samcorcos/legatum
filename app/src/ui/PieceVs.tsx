// A faction piece's comparison with the standard item it replaces, for the surfaces other than the faction sheet
// (the sheet folds it into the piece card's own numbers): a one-line summary for a build card and a short list for
// a hovercard. Both render `pieceCompare.ts`, so they read the same everywhere. Never colour alone: an arrow too.
import { addsText, deltaText, diffText, inGoodFirstOrder, onlyAddsRules, ruleClause, sameText, the, type PieceComparison, type Tone } from '../pieceCompare';
import { Amounts } from './Amounts';
import { ToneIcon } from './bits';

/** "vs Walls: ▲ +5 city strength ▼ +20 Production": only the changes, for a build card. */
export function PieceVsLine({ comparison: c }: { comparison: PieceComparison }) {
  if (c.mode === 'adds') return <span className="prod-vs" data-testid="piece-vs"><Amounts text={addsText(c)} /></span>;
  if (c.same) return <span className="prod-vs same" data-testid="piece-vs"><Amounts text={sameText(c)} /></span>;
  const items = [
    ...inGoodFirstOrder(c.diffs).map((d) => ({ key: d.id, tone: d.tone, text: deltaText(d) })),
    ...c.rules.map((r, i) => ({ key: `r${i}`, tone: r.tone, text: ruleClause(r) })),
  ];
  return (
    <span className="prod-vs" data-testid="piece-vs">
      <span className="prod-vs-label">{onlyAddsRules(c) ? `Same as ${the(c.against)}, plus` : `vs ${c.against}:`}</span>
      {items.map((it) => (
        <span key={it.key}>
          {' '}
          <span className={`prod-vs-item ${it.tone}`} data-tone={it.tone}><ToneIcon tone={it.tone} size={10} /><Amounts text={it.text} /></span>
        </span>
      ))}
    </span>
  );
}

/** The hovercard's block: what the piece replaces and each difference with the standard item's number. */
export function PieceVsBlock({ comparison: c }: { comparison: PieceComparison }) {
  const rows: { key: string; tone: Tone; text: string }[] = [
    ...c.diffs.map((d) => ({ key: d.id, tone: d.tone, text: diffText(d, c.against) })),
    ...c.rules.map((r, i) => ({ key: `r${i}`, tone: r.tone, text: r.text })),
  ];
  return (
    <div className="uc-piece" data-testid="uc-piece">
      <div className="uc-vs">{c.mode === 'adds' ? 'Stands beside' : 'Replaces'} <b>{c.against}</b></div>
      {c.mode === 'adds' ? <p className="uc-piece-note">{`A city can have both it and ${the(c.against)}.`}</p>
        : c.same ? <p className="uc-piece-note">{sameText(c)}</p>
          : <ul className="uc-piece-list">{rows.map((r) => <li key={r.key} className={r.tone}><ToneIcon tone={r.tone} size={11} /><Amounts text={r.text} /></li>)}</ul>}
    </div>
  );
}
