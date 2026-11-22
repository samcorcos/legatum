// The ringed medallion used by the premium pass beyond the faction sheet: a round icon in a tone
// ring (good, bad, neutral ink, gilt for a won title, a dashed hollow one for what is still open),
// with an optional arrow sign or a pinned badge (a holder's crest, a check). It builds on the
// faction sheet's `.md` language (premium.tsx) and takes the same icon names, plus the reward
// emblems (`emblem-captured`) and the treaty glyphs (`treaty-*`: the house star).
import type { CSSProperties, ReactNode } from 'react';
import { emblemUrl } from '../celebrate/RewardEmblem';
import type { Emblem } from '../celebrate/moments';
import { Star } from '../brand/ornaments';
import type { Wonder } from '../types';
import { wonderArt } from '../wonders';
import { Icon, MaskIcon } from './bits';
import './ringMedallion.css';

export type RingTone = 'good' | 'bad' | 'neutral' | 'gilt' | 'hollow';

/** An icon by name, including the reward emblems (`emblem-*`) and treaty glyphs (`treaty-*`) the plain Icon lacks. */
export function Glyph({ name, size, color }: { name: string; size: number; color?: string }) {
  // An era's numeral seal ("era-numeral:III") and a natural wonder's own painting ("wonder:MountFuji"), for the report lines.
  if (name.startsWith('era-numeral:')) return <span className="rm-numeral" style={{ fontSize: Math.round(size * 0.78) }}>{name.slice(12)}</span>;
  if (name.startsWith('wonder:')) return <img className="rm-wonder" src={wonderArt(name.slice(7) as Wonder, 'b')} alt="" width={size} height={size} />;
  if (name.startsWith('treaty-')) return <Star size={size * 0.8} className="rm-star" />;
  if (name.startsWith('emblem-')) {
    const url = emblemUrl(name.slice(7) as Emblem);
    return url ? <MaskIcon url={url} size={size} color={color} className="icon" /> : null;
  }
  return <Icon name={name} size={size} color={color} />;
}

/**
 * A round medallion: `icon` inside a ring in the tone's colour. `sign` adds the faction sheet's
 * rising or falling arrow (good and bad only, never colour alone). `badge` pins a small node at
 * the lower right (a holder's crest, a check).
 */
export function RingMedallion({ icon, tone, size = 26, sign, iconColor, badge, className }: {
  icon: string; tone: RingTone; size?: number; sign?: string; iconColor?: string; badge?: ReactNode; className?: string;
}) {
  const arrow = tone === 'good' || tone === 'bad';
  return (
    <span className={`rm ${tone} ${className ?? ''}`} style={{ '--rm': `${size}px` } as CSSProperties}>
      <Glyph name={icon} size={Math.round(size * 0.68)} color={iconColor} />
      {arrow && <span className="rm-sign"><Icon name={tone === 'good' ? 'ui-trend-up' : 'ui-trend-down'} size={Math.max(7, Math.round(size * 0.3))} label={sign ?? (tone === 'good' ? 'Good for you' : 'Bad for you')} /></span>}
      {badge && <span className="rm-badge">{badge}</span>}
    </span>
  );
}

export type TreatyKind = 'Peace' | 'Alliance' | 'War';

/**
 * A treaty's emblem, built from the house star: a ring around the star, two interlocked rings for
 * an Alliance, and a red ring with a cross for War. No new art.
 */
export function TreatyEmblem({ kind, size = 40 }: { kind: TreatyKind; size?: number }) {
  const bad = kind === 'War';
  const label = kind;
  return (
    <span className={`te ${kind.toLowerCase()}${bad ? ' bad' : ''}${size < 22 ? ' sm' : ''}`} style={{ '--te': `${size}px` } as CSSProperties} role="img" aria-label={`${label} emblem`}>
      <span className="te-ring a" />
      {kind === 'Alliance' && <span className="te-ring b" />}
      <Star size={Math.round(size * 0.38)} className="te-star" />
      <span className="te-sign"><Icon name={bad ? 'ui-cross' : 'ui-check'} size={Math.max(8, Math.round(size * 0.26))} /></span>
    </span>
  );
}
