// STAND-IN archetype classifier for tools/film/fake-run.mjs only. The real vocabulary comes from
// the strategy-diversity-metrics classifier in crates/sim once it lands; the evolution run logs
// those names and every reader just displays whatever names the log contains.
//
// Rule: at about a third of the game (turn clamp(turns/3, 25, 70)), compare each empire with the
// others in the same game (z-scores over the 8 seats) on four telemetry signals, and label it by
// its strongest lean. No lean of at least 0.6 standard deviations is "Balanced".
import { snapshotAtOrBefore } from "./archive.mjs";

export const ARCHETYPE_ORDER = ["Rush", "Wide", "Science", "Economic", "Balanced"];

const z = (xs) => {
  const m = xs.reduce((a, b) => a + b, 0) / xs.length;
  const sd = Math.sqrt(xs.reduce((a, b) => a + (b - m) ** 2, 0) / xs.length);
  return xs.map((x) => (sd ? (x - m) / sd : 0));
};

export function classifyGame(telemetry, turns) {
  const turn = Math.max(25, Math.min(70, Math.round(turns / 3)));
  const snap = snapshotAtOrBefore(telemetry, turn);
  const rows = snap.players;
  const signal = {
    Rush: z(rows.map((p) => p.army_strength / Math.max(1, p.population))),
    Wide: z(rows.map((p) => p.cities)),
    Science: z(rows.map((p) => p.science / Math.max(1, p.population))),
    Economic: z(rows.map((p) => p.gold)),
  };
  const out = {};
  rows.forEach((p, i) => {
    let archetype = "Balanced", score = 0.6;
    for (const a of ARCHETYPE_ORDER.slice(0, 4)) if (signal[a][i] > score) { archetype = a; score = signal[a][i]; }
    if (!p.alive && p.cities === 0) archetype = "Balanced";
    out[p.player] = {
      archetype,
      class_turn: snap.turn,
      f_cities: p.cities,
      f_army: p.army_strength,
      f_science: p.science,
      f_gold: p.gold,
      f_population: p.population,
    };
  });
  return out;
}
