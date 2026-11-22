// Battle result cues: after a turn resolves, the hexes where fighting happened show what it cost
// ("−24", "Destroyed") for a few seconds, so the outcome reads on the map, not only in the report.
import { attackerUnit, type Event, type Hex, type PlayerView } from '../types';
import { hexEq } from '../hex';

export interface MapCue {
  hex: Hex;
  text: string;
  /** good: the player's side came out ahead here; bad: the player lost something. */
  tone: 'good' | 'bad' | 'neutral';
  /** Filled in per frame: opacity and upward drift in px. */
  alpha: number;
  rise: number;
}

export const CUE_HOLD_MS = 3500;
export const CUE_FADE_MS = 1500;

/**
 * One cue per hex (damage, or "Destroyed", which wins), from this viewer's visible events.
 */
export function battleCues(events: Event[], view: PlayerView): MapCue[] {
  const team = new Set(view.players.filter((p) => p.team === view.team).map((p) => p.id));
  const cues: MapCue[] = [];
  const put = (hex: Hex, text: string, mine: boolean, strong = false) => {
    const tone = mine ? 'bad' : 'good';
    const at = cues.findIndex((c) => hexEq(c.hex, hex));
    if (at >= 0) {
      if (strong) cues[at] = { hex, text, tone, alpha: 1, rise: 0 };
      return;
    }
    cues.push({ hex, text, tone, alpha: 1, rise: 0 });
  };
  for (const e of events) {
    if (e.type === 'Battle') {
      const defenderOwner = e.defender.owner;
      if (e.damage_to_defender > 0) put(e.at, `−${e.damage_to_defender}`, team.has(defenderOwner));
      // A city's shot (cities fire back) takes no damage, so only a unit attacker gets a cue.
      const attacker = view.units.find((u) => u.id === attackerUnit(e.attacker));
      if (attacker && e.damage_to_attacker > 0) put(attacker.pos, `−${e.damage_to_attacker}`, team.has(e.attacker_owner));
    } else if (e.type === 'UnitDestroyed') {
      put(e.at, 'Destroyed', team.has(e.owner), true);
    }
  }
  return cues;
}

/** A clicked combat line's arrow: attacker (when the viewer saw it) to target, then it fades. */
export interface ReportArrow {
  from: Hex | null;
  to: Hex;
  alpha: number;
}
export const ARROW_HOLD_MS = 3000;
export const ARROW_FADE_MS = 800;

/** The arrow as drawn `elapsed` ms after the line was clicked; null once it has faded. */
export function arrowAt(arrow: { from: Hex | null; to: Hex }, elapsed: number): ReportArrow | null {
  if (elapsed > ARROW_HOLD_MS + ARROW_FADE_MS) return null;
  const alpha = elapsed <= ARROW_HOLD_MS ? 1 : 1 - (elapsed - ARROW_HOLD_MS) / ARROW_FADE_MS;
  return { from: arrow.from, to: arrow.to, alpha };
}

/**
 * The cues as drawn `elapsed` ms after the turn resolved (reduced motion: no drift). With combat animations on, each
 * cue waits for its blow to land: `delay` gives the ms from the turn's start (0 when the hex has none).
 */
export function cuesAt(cues: MapCue[], elapsed: number, reducedMotion: boolean, delay?: (hex: Hex) => number): MapCue[] {
  const out: MapCue[] = [];
  for (const c of cues) {
    const e = elapsed - (delay?.(c.hex) ?? 0);
    if (e < 0 || e > CUE_HOLD_MS + CUE_FADE_MS) continue;
    const alpha = e <= CUE_HOLD_MS ? 1 : 1 - (e - CUE_HOLD_MS) / CUE_FADE_MS;
    // A delayed cue pops in: a quick lift as its blow lands, then the usual slow drift.
    const pop = delay && e < 160 ? (1 - e / 160) * -6 : 0;
    out.push({ ...c, alpha, rise: reducedMotion ? 0 : Math.min(10, e / 60) + pop });
  }
  return out;
}
