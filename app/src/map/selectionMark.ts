// The selection mark, shared by the 2D and 3D maps: a bright white tile edge on a dark halo, a breathing wash
// inside it, a ring round the unit, and a ring that bursts out from the tile when the selection changes. White is
// no empire's colour (they are all muted), stays apart from the advisor's gilt ring and the red route and enemy
// marks, and needs no colour vision; the dark halo keeps it visible on pale parchment, snow and sea alike.

/** The mark's bright ink. */
export const SELECT_INK = '#ffffff';
/** The dark halo under the bright ink. */
export const SELECT_HALO = '#0b1924';
/** How long the burst lasts when the selection changes, in ms. */
export const SELECT_BURST_MS = 700;

/** The breathing wash's strength, 0..1, on the selection clock (ms). A still map holds it at 0.5. */
export function selectionBreath(t: number, still: boolean): number {
  return still ? 0.5 : 0.5 + 0.5 * Math.sin(t / 300);
}

/** The burst ring at `ms` after the selection changed: its scale (1 = the tile edge) and opacity, or null once it is over. */
export function selectionBurst(ms: number): { scale: number; alpha: number } | null {
  if (!(ms >= 0) || ms >= SELECT_BURST_MS) return null;
  const k = ms / SELECT_BURST_MS;
  return { scale: 1 + 1.4 * (1 - Math.pow(1 - k, 3)), alpha: 1 - k };
}
