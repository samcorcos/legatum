import type { Faction } from '../types';
import { RANK_EDGE, RANK_GOLD, RANK_INK, RANK_LAMBDA, RANK_PATCH, rankLevel, rankStrokes } from '../rankInsignia';

/** The same engraved rank badge in panels, tooltips, reports and combat cards. */
export function RankMark({ level, faction, compact = false }: { level: number; faction?: Faction; compact?: boolean }) {
  const rank = rankLevel(level, faction);
  if (!rank) return null;
  const { chevrons, rockers, device } = rankStrokes(rank);
  const label = rank === 6 ? 'Spartan Veteran, level 6' : rank === 5 && faction !== 'Sparta' ? 'Veteran, level 5' : `Level ${rank}`;
  return <svg className="rank-insignia" width={compact ? 18 : 24} height={compact ? 26 : 34} viewBox="0 0 24 34" role="img" aria-label={label}>
    <path d={RANK_PATCH} fill={RANK_INK} stroke={RANK_EDGE} strokeWidth=".8" />
    {[...chevrons, ...rockers].map((d, i) => <g key={i}>
      <path d={d} fill="none" stroke={RANK_EDGE} strokeWidth="3.1" strokeLinecap="round" strokeLinejoin="round" />
      <path d={d} fill="none" stroke={RANK_GOLD} strokeWidth="1.35" strokeLinecap="round" strokeLinejoin="round" />
    </g>)}
    {device && <>
      <path d={RANK_LAMBDA} fill="none" stroke={RANK_INK} strokeWidth="4" strokeLinecap="round" strokeLinejoin="round" />
      <path d={RANK_LAMBDA} fill="none" stroke={RANK_GOLD} strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round" />
    </>}
  </svg>;
}
