import type { ReactNode } from 'react';
import type { Breakdown, CityYields, Faction, Yields } from '../types';
import { iconUrl, resourceArtUrl } from '../icons';
import { YIELD_COLORS } from '../theme';
import { Crest, FactionSeal } from '../brand/ornaments';
import { wonderArt, wonderLineLabel, wonderOfLabel } from '../wonders';
import { FoodDecadencePointer, OwnDecadenceNote, decadenceLinePercent, isDecadenceLine } from './DecadenceNote';

export { YIELD_KEYS, YIELD_LABEL, type YieldKey } from './yieldKeys';
import { YIELD_KEYS, YIELD_LABEL, type YieldKey } from './yieldKeys';

/** An icon from the set, painted in `color` (the SVGs use currentColor, which <img> can't set). */
export function MaskIcon({ url, size, color, className, label }: { url: string; size: number; color?: string; className?: string; label?: string }) {
  return (
    <span
      role={label ? 'img' : undefined}
      aria-label={label}
      className={`mask-icon ${className ?? ''}`}
      style={{ width: size, height: size, backgroundColor: color ?? 'currentColor', WebkitMaskImage: `url("${url}")`, maskImage: `url("${url}")` }}
    />
  );
}

/** Good and bad for you: an arrow, never colour alone (the label is for screen readers). */
const TONE_ICON = { good: 'ui-trend-up', bad: 'ui-trend-down' } as const;
export function ToneIcon({ tone, size = 11 }: { tone: 'good' | 'bad' | 'neutral'; size?: number }) {
  return tone === 'neutral' ? <span aria-hidden="true">·</span> : <Icon name={TONE_ICON[tone]} size={size} className="ui-mark" />;
}

export function YieldIcon({ k, size = 14 }: { k: YieldKey; size?: number }) {
  const url = iconUrl(`yield-${k}`);
  return url ? <MaskIcon url={url} size={size} color={YIELD_COLORS[k]} className="yield-icon" label={YIELD_LABEL[k]} /> : null;
}

export function Icon({ name, size = 16, fallback, className, color, label }: { name: string; size?: number; fallback?: ReactNode; className?: string; color?: string; label?: string }) {
  if (name.startsWith('resource-')) {
    const url = resourceArtUrl(name.slice('resource-'.length));
    if (url) return <img src={url} width={size} height={size} className={`resource-art-icon ${className ?? ''}`} alt={label ?? ''} aria-hidden={label ? undefined : true} />;
  }
  const url = iconUrl(name);
  if (url) return <MaskIcon url={url} size={size} color={color} className={`icon ${className ?? ''}`} label={label} />;
  return <>{fallback ?? null}</>;
}

/** A faction's mark by size: the heraldry glyph (tinted) below 40 px, the raster crest from 40 px. */
export function FactionMark({ faction, size = 18, color }: { faction: Faction; size?: number; color?: string }) {
  if (size >= 40) return <Crest faction={faction} size={size} color={color} />;
  return <FactionSeal faction={faction} size={size} color={color ?? 'var(--accent)'} />;
}

export function signed(n: number) {
  return n > 0 ? `+${n}` : n < 0 ? `−${Math.abs(n)}` : '0';
}

export function BreakdownTable({ b, title, unit, plain, signedLabels }: { b: Breakdown; title?: string; unit?: string; plain?: boolean; signedLabels?: string[] }) {
  return (
    <div className="bd">
      {title && <div className="bd-title">{title}</div>}
      {b.parts.length === 0 && <div className="bd-row muted">Nothing yet</div>}
      {b.parts.map(([label, v], i) => {
        // Decadence raises what growth needs: a cost, so it reads as one (red), not as a neutral "+23".
        const cost = isDecadenceLine(label) && v > 0;
        return (
          <div className="bd-row" key={i}>
            <span className={cost ? 'neg' : ''}>{cost ? `Decadence: growth needs +${v} Food` : (wonderOfLabel(label) ? wonderLineLabel(wonderOfLabel(label)!) : label)}</span>
            <span className={v < 0 || cost ? 'neg' : ''}>{(i === 0 || plain) && v >= 0 && !signedLabels?.includes(label) ? v : signed(v)}</span>
          </div>
        );
      })}
      <div className="bd-row bd-total">
        <span>Total{unit ? ` ${unit}` : ''}</span>
        <span>{b.total}</span>
      </div>
    </div>
  );
}

export const TILE_PRIORITY_INFO = {
  Balanced: 'Tile score: 4 × Food + 3 × Production + 2 × Gold.',
  Growth: 'Tile score: 8 × Food + 2 × Production + Gold.',
  Production: 'Tile score: 3 × Food + 8 × Production + Gold.',
};

/** Why the city chooses its next claim: said once per breakdown, not under every yield. */
export const tileFocusNoteText = (focus: CityYields['focus'], reason: string) =>
  `This city works every owned tile within its work radius. Its ${focus} focus steers its next claim (${reason}). ${TILE_PRIORITY_INFO[focus]}`;

function TileFocusNote({ yields }: { yields: CityYields }) {
  return (
    <div className="bd-note">
      {tileFocusNoteText(yields.focus, yields.focus_reason)}
    </div>
  );
}

/** The decadence explanation once per breakdown, whenever one of the city's lines is a Decadence line. */
function CityDecadenceNote({ sources }: { sources: CityYields['sources'] }) {
  const line = sources.find((s) => isDecadenceLine(s.label));
  return line ? <OwnDecadenceNote fallback={decadenceLinePercent(line.label)} /> : null;
}

export function YieldSourceTable({ yields, k, context = true }: { yields: CityYields; k: YieldKey; context?: boolean }) {
  const rows = yields.sources.filter((s) => s.yields[k] !== 0);
  return (
    <div className="bd">
      <div className="bd-title">
        <YieldIcon k={k} /> {YIELD_LABEL[k]}
      </div>
      {context && <TileFocusNote yields={yields} />}
      {rows.length === 0 && <div className="bd-row muted">No sources</div>}
      {rows.map((s, i) => {
        // A natural wonder's bonus is a named line of its own: its painting, "Grand Canyon +25%", the amount with its icon.
        const wonder = wonderOfLabel(s.label);
        return (
          <div className={`bd-row${wonder ? ' bd-wonder' : ''}`} key={i} data-wonder={wonder ?? undefined}>
            <span>{wonder ? <><img src={wonderArt(wonder, 'b')} width={24} height={24} alt="" />{wonderLineLabel(wonder)}</> : s.label}</span>
            <span className={s.yields[k] < 0 ? 'neg' : ''}>{signed(s.yields[k])}{wonder && <> <YieldIcon k={k} size={12} /></>}</span>
          </div>
        );
      })}
      {yields.sources
        .filter((s) => YIELD_KEYS.every((y) => s.yields[y] === 0))
        .map((s, i) => (
          <div className="bd-row muted" key={`z${i}`}>
            <span>{s.label}</span>
            <span>0</span>
          </div>
        ))}
      <div className="bd-row bd-total">
        <span>Total per turn</span>
        <span>{yields.total[k]}</span>
      </div>
      {context && (k === 'food' ? <FoodDecadencePointer /> : <CityDecadenceNote sources={rows} />)}
    </div>
  );
}

export function Bar({ value, max, color, className }: { value: number; max: number; color: string; className?: string }) {
  const f = max > 0 ? Math.max(0, Math.min(1, value / max)) : 0;
  return (
    <div className={`bar ${className ?? ''}`}>
      <div className="bar-fill" style={{ width: `${f * 100}%`, background: color }} />
    </div>
  );
}

/** A requirement not met: an empty ring (the × is only for close and remove). */
export function NotMetMark({ size = 11, className = '' }: { size?: number; className?: string }) {
  return <span className={`not-met-mark ${className}`} role="img" aria-label="missing" style={{ width: size - 2, height: size - 2 }} />;
}

export function Kbd({ children }: { children: ReactNode }) {
  return <kbd className="kbd">{children}</kbd>;
}

/**
 * What a building changes in a city: every yield now → after, then the sources of only the yields
 * that change. The shared tile-focus context appears once.
 */
export function YieldsAfter({ now, after }: { now: Yields; after: CityYields }) {
  const changed = YIELD_KEYS.filter((k) => after.total[k] !== now[k]);
  return (
    <div className="bd">
      <div className="bd-title">After completion</div>
      <div className="bd-note">At today's population.</div>
      {YIELD_KEYS.map((k) => (
        <div className={`bd-row ${changed.includes(k) ? '' : 'muted'}`} key={k}>
          <span><YieldIcon k={k} size={12} /> {YIELD_LABEL[k]}</span>
          <span>{now[k]} → {after.total[k]}</span>
        </div>
      ))}
      <TileFocusNote yields={after} />
      {changed.map((k) => <YieldSourceTable key={k} yields={after} k={k} context={false} />)}
      <CityDecadenceNote sources={after.sources} />
    </div>
  );
}
