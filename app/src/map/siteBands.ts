// The Settle lens's absolute scale. A band is a fixed range of a site's quality (core
// `settlement::site_quality`), so "Good" means the same on every map and in every game.
// The floors come from core through rules.generated.json; nothing here re-derives a band
// from a site: the core's score carries its `band`.
import { RULES } from '../data';
import { SITE_BANDS, type SiteBand } from '../types';

export { SITE_BANDS };
export type { SiteBand };

/** Lowest quality of Fair, Good, Great and Exceptional. */
export const BAND_FLOORS: readonly number[] = RULES.siteBands.floors;

export const siteBandOf = (quality: number): SiteBand => SITE_BANDS[BAND_FLOORS.filter((floor) => quality >= floor).length];

/** The quality range of a band, in the words the legend and the hover use. */
export function bandRange(band: SiteBand): string {
  const i = SITE_BANDS.indexOf(band);
  if (i === 0) return `under ${BAND_FLOORS[0]}`;
  if (i === SITE_BANDS.length - 1) return `${BAND_FLOORS[i - 1]} or more`;
  return `${BAND_FLOORS[i - 1]} to ${BAND_FLOORS[i] - 1}`;
}
