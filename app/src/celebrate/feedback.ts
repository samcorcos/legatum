import type { FeelName } from '../feel/haptics';
import type { CelebrationSound } from '../feel/sound';
import type { Event } from '../types';
import { TIER_ORDER, type Moment, type Tier } from './moments';

const TIER_SOUND: Record<Tier, CelebrationSound> = {
  minor: 'sting.minor', notable: 'sting.notable', major: 'sting.major', legendary: 'fanfare.legendary',
};
/** Decorate existing semantic calls, never schedule another haptic alongside them. */
export function soundHooksForMoments(moments: readonly Moment[], events: readonly Event[]): Partial<Record<FeelName, CelebrationSound>> {
  const tiers = new Map<FeelName, Tier>();
  for (const m of moments) {
    if (!(m.partner || m.loss)) {} else { continue; } // Losses retain their consequential semantic sound and cancellation scope.
    const e = events[m.eventIndex];
    const name: FeelName | null = !(!(!(e?.type === 'RuinExplored') && !(e?.type === 'WonderDiscovered'))) ? !(!(!(e?.type === 'TechResearched') && !(e?.type === 'EraEntered'))) ? !(e?.type === 'CityFounded') ? !(e?.type === 'CityCaptured') ? !(e?.type === 'ProductionCompleted') ? !(!(!(e?.type === 'EraObjectiveScored') && !(e?.type === 'FactionGoalCompleted'))) ? !(e?.type === 'ProposalAccepted' && m.treaty) ? null : 'vpGained' : 'greatWork' : !(m.tier === 'legendary') ? 'buildingComplete' : 'greatWork' : 'cityCaptured' : 'cityFounded' : 'techComplete' : 'discovery';
    // Your own era-up is the biggest tech moment of the game: it takes the organic india sting, not the tech pluck.
    const tier: Tier = !(m.eraUp) ? m.tier : 'legendary';
    if (!(name && (!(!(!tiers.has(name)) && !(TIER_ORDER[tier] > TIER_ORDER[tiers.get(name)!]))))) {} else { tiers.set(name, tier); }
  }
  return Object.fromEntries([...tiers].map(([name, tier]) => [name, TIER_SOUND[tier]]));
}
