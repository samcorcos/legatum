// The premium building blocks, extracted from the faction sheet (the reference for the whole game's
// visual bar): an engraved leader portrait frame, round portrait roundels, icon medallion rows
// with a rising or falling arrow, the unique-piece card, the star divider and the parchment grain
// (class `paper-grain`). New screens compose these instead of inventing their own treatments.
import type { CSSProperties, ReactNode } from 'react';
import { buildingStatsFor, costInFactionYield, costYield, itemIcon, unitStatsFor } from '../data';
import { diffText, diffWas, pieceComparison, sameText, addsText, type DiffId, type PieceComparison, type Tone } from '../pieceCompare';
import { itemArtUrl } from '../unlockArt';
import { Crest, FactionSeal, Star, leaderPortraitUrl } from '../brand/ornaments';
import type { Faction, ProductionItem, Yields } from '../types';
import { inkOfColor } from '../theme';
import { Icon, ToneIcon, YIELD_KEYS, YieldIcon } from './bits';
import { Amounts } from './Amounts';
import './premium.css';

/** Engraved rule with the house four-point star at its centre. */
export function StarDivider({ className }: { className?: string }) {
  return (
    <div className={`sd ${className ?? ''}`} aria-hidden="true">
      <span />
      <Star size={9} />
      <span />
    </div>
  );
}

/** The four star studs that sit on a portrait frame's corners. */
function PortraitCorners() {
  return (
    <>
      {['tl', 'tr', 'bl', 'br'].map((c) => (
        <Star key={c} size={11} className={`pf-corner ${c}`} />
      ))}
    </>
  );
}

/**
 * A leader's engraved frame: the house double rule in ink, a muted-gilt hairline inside it, a star
 * stud at each corner and the faction's raster crest pinned over the lower-right corner. Size comes
 * from `--pw`/`--ph` on the frame or any ancestor (120 x 150 by default). `fallen` greys the paint.
 * With no faction the children fill the frame (the Random "?" mark).
 */
export function PortraitFrame({
  faction,
  crest = true,
  crestSize = 42,
  crestColor,
  fallen,
  className,
  style,
  children,
}: {
  faction: Faction | null;
  crest?: boolean;
  crestSize?: number;
  crestColor?: string;
  fallen?: boolean;
  className?: string;
  style?: CSSProperties;
  children?: ReactNode;
}) {
  const url = faction ? leaderPortraitUrl(faction) : null;
  return (
    <figure className={`pf ${fallen ? 'fallen' : ''} ${faction ? '' : 'pf-empty'} ${className ?? ''}`} style={style}>
      {url && <img key={faction} src={url} alt="" draggable={false} />}
      {children}
      <PortraitCorners />
      {faction && crest && (
        <span className="pf-crest">
          <Crest faction={faction} size={crestSize} color={crestColor} />
        </span>
      )}
    </figure>
  );
}

/** A round oil-painting thumbnail in an ink ring and a paper ring, the faction's seal pinned at its lower right. */
export function PortraitRoundel({ faction, size = 38, color, fallen, className }: { faction: Faction | 'Random'; size?: number; color?: string; fallen?: boolean; className?: string }) {
  const style = { '--rs': `${size}px`, ...(color ? { '--ring': inkOfColor(color) } : {}) } as CSSProperties;
  if (faction === 'Random') {
    return (
      <span className={`pf-roundel ${className ?? ''}`} style={style}>
        <span className="random-mark">?</span>
      </span>
    );
  }
  const url = leaderPortraitUrl(faction);
  return (
    <span className={`pf-roundel ${fallen ? 'fallen' : ''} ${className ?? ''}`} style={style}>
      {url && <img src={url} alt="" loading="lazy" draggable={false} />}
      <span className="pf-roundel-seal" aria-hidden="true">
        <FactionSeal faction={faction} size={Math.round(size * 0.34)} color="var(--accent)" />
      </span>
    </span>
  );
}

/** A banner strip with swallowtail ends, for a name or a verdict under a portrait: ink, red or gilt. */
export function Ribbon({ tone = 'ink', children, className }: { tone?: 'ink' | 'red' | 'gilt'; children: ReactNode; className?: string }) {
  return (
    <span className={`rb ${tone} ${className ?? ''}`}>
      <span className="rb-text">{children}</span>
    </span>
  );
}

/** The ring-and-icon medallion: green with a rising arrow, red with a falling one. */
function Medallion({ icon, tone, sign }: { icon: string; tone: 'good' | 'bad'; sign?: string }) {
  return (
    <span className={`md ${tone}`}>
      <Icon name={icon} size={20} />
      <span className="md-sign">
        <Icon name={tone === 'good' ? 'ui-trend-up' : 'ui-trend-down'} size={9} label={sign ?? (tone === 'good' ? 'Gain' : 'Cost')} />
      </span>
    </span>
  );
}

/** A medallion, ink text and a tinted hairline under both: no filled box. */
export function MedallionRow({
  icon,
  tone,
  sign,
  className,
  children,
}: {
  icon: string;
  tone: 'good' | 'bad';
  sign?: string;
  className?: string;
  children: ReactNode;
}) {
  return (
    <div className={`mr ${tone} ${className ?? ''}`}>
      <Medallion icon={icon} tone={tone} sign={sign} />
      <div className="mr-text">{typeof children === 'string' ? <Amounts text={children} /> : children}</div>
    </div>
  );
}

interface PieceStat {
  /** Which number this is, so a comparison can mark it (`pieceCompare.ts` `DiffId`). */
  id: DiffId;
  icon?: string;
  yieldKey?: (typeof YIELD_KEYS)[number];
  value: string;
  label?: string;
}

/** A price as the faction pays it: Production, or surplus Food for Egypt. */
function costStat(faction: Faction, cost: number): Omit<PieceStat, 'id'> {
  const y = costYield(faction);
  return { icon: y === 'Food' ? 'yield-food' : 'yield-production', value: String(costInFactionYield(faction, cost)), label: y };
}

/** The piece's numbers as icon-number pairs, from the same stat functions its description reads. */
function pieceStats(item: ProductionItem, faction: Faction): PieceStat[] {
  if (item.kind === 'Unit') {
    const u = unitStatsFor(faction, item.unit);
    const stats: PieceStat[] = [
      { id: 'cost', ...costStat(faction, u.cost) },
      { id: 'moves', icon: 'mark-move', value: String(u.moves), label: u.moves === 1 ? 'move' : 'moves' },
      { id: 'strength', icon: 'unit-spearman', value: String(u.strength), label: 'strength' },
    ];
    if (u.ranged) stats.push({ id: 'ranged', icon: 'unit-archer', value: String(u.ranged), label: `ranged · range ${u.range}` });
    return stats;
  }
  if (item.kind !== 'Building') return [];
  const b = buildingStatsFor(faction, item.building);
  const stats: PieceStat[] = [{ id: 'cost', ...costStat(faction, b.cost) }];
  for (const k of YIELD_KEYS) {
    const n = (b.yields as Partial<Yields>)[k];
    if (n) stats.push({ id: k, yieldKey: k, value: n > 0 ? `+${n}` : `−${Math.abs(n)}`, label: k[0].toUpperCase() + k.slice(1) });
  }
  if (b.cityStrength) stats.push({ id: 'cityStrength', icon: 'fortify', value: `+${b.cityStrength}`, label: 'city strength' });
  return stats;
}

/** A faction's unique piece as a card: its own icon in an engraved frame, the replaced item struck through, its numbers as icon-number pairs (each that differs from the standard item shows that item's number, in green when better and red when worse), then how it works differently and its rule text. */
export function PieceCard({
  faction,
  piece,
  name,
  factionName,
  replaces,
  text,
}: {
  faction: Faction;
  piece: ProductionItem;
  name: string;
  factionName: string;
  replaces?: string | null;
  text: string;
}) {
  const replacedIcon = replaces ? itemIcon(piece) : null;
  const art = itemArtUrl(piece, faction, 256);
  const comparison = pieceComparison(faction, piece);
  // The price stays first; after it the gains lead and the losses come last (the order of `inGoodFirstOrder`).
  const rank = (id: DiffId) => { const tone = comparison?.diffs.find((x) => x.id === id)?.tone; return tone === 'good' ? 0 : tone === 'bad' ? 2 : 1; };
  const stats = pieceStats(piece, faction).map((st, i) => ({ st, i })).sort((a, b) => (a.st.id === 'cost' ? -1 : b.st.id === 'cost' ? 1 : rank(a.st.id) - rank(b.st.id) || a.i - b.i)).map(({ st }) => st);
  // A difference with no pair of its own (a number the piece lost) still reads, as a line.
  const unpaired = comparison?.diffs.filter((d) => !stats.some((st) => st.id === d.id)) ?? [];
  return (
    <div className="pc">
      <span className={`pc-icon${art ? ' painted' : ''}`}>
        {art ? <img src={art} alt="" width={256} height={256} draggable={false} /> : <Icon name={itemIcon(piece, faction)} size={40} />}
      </span>
      <div className="pc-head">
        <div className="pc-title">
          <span className="pc-name">{name}</span>
          <span className="pc-replaces">
            {replacedIcon ? (
              <>
                <span className="pc-struck"><Icon name={replacedIcon} size={14} /></span>
                replaces {replaces}
              </>
            ) : (
              <>
                <FactionSeal faction={faction} size={14} color="var(--accent)" />
                only {factionName} can build it
              </>
            )}
          </span>
        </div>
        <div className="pc-stats">
          {stats.map((st) => {
            const d = comparison?.diffs.find((x) => x.id === st.id);
            return (
              <span className={`pc-stat${d ? ` ${d.tone}` : ''}`} key={st.id} data-diff={d ? d.tone : undefined}>
                {st.yieldKey ? <YieldIcon k={st.yieldKey} size={15} /> : <Icon name={st.icon!} size={15} />}
                <b>{st.value}</b>
                {st.label && <small>{st.label}</small>}
                {d && <small className="pc-was"> ({comparison!.against}: {diffWas(d)})</small>}
                {d && <ToneIcon tone={d.tone} size={10} />}
              </span>
            );
          })}
        </div>
      </div>
      {comparison && <PieceVsNotes comparison={comparison} unpaired={unpaired} />}
      {text && <p className="pc-text"><Amounts text={text} /></p>}
    </div>
  );
}

/** What the numbers can't show about how a piece differs: rule differences with an arrow and a colour, a number it lost, "same as" for a rename and "not a replacement" for an addition. */
function PieceVsNotes({ comparison, unpaired = [] }: { comparison: PieceComparison; unpaired?: PieceComparison['diffs'] }) {
  const notes: { text: string; tone: Tone | null }[] = comparison.mode === 'adds' ? [{ text: addsText(comparison), tone: null }]
    : comparison.same ? [{ text: sameText(comparison), tone: null }]
    : [...unpaired.map((d) => ({ text: diffText(d, comparison.against), tone: d.tone })), ...comparison.rules.map((r) => ({ text: r.text, tone: r.tone }))];
  if (notes.length === 0) return null;
  return (
    <p className="pc-vs">
      {notes.map((n, i) => (
        <span key={i} className={`pc-vs-note${n.tone ? ` ${n.tone}` : ''}`}>
          {n.tone && <ToneIcon tone={n.tone} size={10} />}
          <Amounts text={n.text} />
        </span>
      ))}
    </p>
  );
}
