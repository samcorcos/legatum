import type { Emblem } from './moments';

const urls = import.meta.glob<string>('../assets/rewards/emblem-*.svg', { eager: true, query: '?url', import: 'default' });
export const emblemUrl = (kind: Emblem) => Object.entries(urls).find(([path]) => path.endsWith(`emblem-${kind}.svg`))?.[1] ?? null;

const LABEL: Record<Emblem, string> = {
  camp: 'A broken standard over cold embers', hall: 'The Yam Relay Station’s compass rose', ruin: 'A broken arch above buried stones', captured: 'A gate with the banner raised',
  founded: 'A cornerstone on its tile', growth: 'A sprout rising from its tile', milestone: 'A four-point star in a laurel',
  greatwork: 'A monument in a laurel', tech: 'A four-point star within its orbits',
};

/**
 * A reward's own engraved emblem inside the card's medallion ring (the ring is the player's colour, the engraving
 * is ink). It stands where the faction crest stood, so every reward kind reads differently at a glance.
 */
export function RewardEmblem({ kind, size, color }: { kind: Emblem; size: number; color: string }) {
  const url = emblemUrl(kind);
  if (!url) return null;
  return (
    <span className="medallion moment-emblem" data-emblem={kind} style={{ width: size, height: size, ['--pc' as string]: color }}>
      <span role="img" aria-label={LABEL[kind]} className="moment-emblem-art" style={{ WebkitMaskImage: `url("${url}")`, maskImage: `url("${url}")` }} />
    </span>
  );
}
