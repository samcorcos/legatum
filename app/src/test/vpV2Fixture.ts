// A victory view for UI tests and screenshots with staged World Ages. Render-only; the numbers are the ones the core
// would send for such a game, except that each Age here carries three cards (a real game draws one) so the lists and
// sheets are exercised with more than one line.
import { decadencePercent, decadenceRules } from '../data';
import type { EraView, ObjectiveProgress, PlayerView, RuleSwitches } from '../types';

const V2_ERAS: EraView[] = [
  {
    era: 2, name: 'Classical', entered_by: [], scored_by: [],
    objectives: [
      { slot: 0, family: 'Expand', text: 'Prosperity: grow your population by 9', target: 9, objective: { kind: 'Population', n: 9 } },
      { slot: 1, family: 'Expand', text: 'Gain 4 cities', target: 4, objective: { kind: 'Cities', n: 4 } },
      { slot: 2, family: 'Fight', text: 'Breach: capture a walled city', target: 1, objective: { kind: 'Breach' } },
    ],
  },
  {
    era: 3, name: 'Medieval', entered_by: [], scored_by: [],
    objectives: [
      { slot: 0, family: 'Expand', text: 'Frontier: gain 14 tiles of territory', target: 14, objective: { kind: 'Territory', n: 14 } },
      { slot: 1, family: 'Expand', text: 'Grow your population by 40', target: 40, objective: { kind: 'Population', n: 40 } },
      { slot: 2, family: 'Fight', text: 'Humble the mighty: capture a city from an empire ahead of you on VP', target: 1, objective: { kind: 'HumbleTheMighty' } },
    ],
  },
  {
    era: 4, name: 'Renaissance', entered_by: [], scored_by: [],
    objectives: [
      { slot: 0, family: 'Build', text: 'Add a Bank to 3 more cities', target: 3, objective: { kind: 'Buildings', building: 'Clearinghouse', cities: 3 } },
      { slot: 1, family: 'Expand', text: 'Gain 100 tiles of territory', target: 100, objective: { kind: 'Territory', n: 100 } },
      { slot: 2, family: 'Fight', text: 'Veterans: earn 4 promotions in battle', target: 4, objective: { kind: 'Veterans', n: 4 } },
    ],
  },
  {
    era: 5, name: 'Industrial', entered_by: [], scored_by: [],
    objectives: [
      { slot: 0, family: 'Build', text: 'Add a Forge to 4 more cities', target: 4, objective: { kind: 'Buildings', building: 'MachineShop', cities: 4 } },
      { slot: 1, family: 'Expand', text: 'Gain 8 cities', target: 8, objective: { kind: 'Cities', n: 8 } },
      { slot: 2, family: 'Fight', text: 'Breach: capture a walled city', target: 1, objective: { kind: 'Breach' } },
    ],
  },
  {
    era: 6, name: 'Modern', entered_by: [], scored_by: [],
    objectives: [
      { slot: 0, family: 'Expand', text: 'Frontier: gain 24 tiles of territory', target: 24, objective: { kind: 'Territory', n: 24 } },
      { slot: 1, family: 'Expand', text: 'Grow your population by 85', target: 85, objective: { kind: 'Population', n: 85 } },
      { slot: 2, family: 'Fight', text: 'Veterans: earn 4 promotions in battle', target: 4, objective: { kind: 'Veterans', n: 4 } },
    ],
  },
  {
    era: 7, name: 'Future', entered_by: [], scored_by: [],
    objectives: [
      { slot: 0, family: 'Build', text: 'Add a Research Lab to 4 more cities', target: 4, objective: { kind: 'Buildings', building: 'InferenceCore', cities: 4 } },
      { slot: 1, family: 'Expand', text: 'Gain 160 tiles of territory', target: 160, objective: { kind: 'Territory', n: 160 } },
      { slot: 2, family: 'Fight', text: 'Humble the mighty: capture a city from an empire ahead of you on VP', target: 1, objective: { kind: 'HumbleTheMighty' } },
    ],
  },
];

export interface V2Options {
  /** Your era (1-based). Eras up to it are entered. The first rival is in era 3, so Ages up to the later of the two have begun. */
  era?: number;
  /** Overreach: your lead over the average empire in percent (default 0: no decadence), and the first rival's. The rates derive from `data.ts` (and `rules`). */
  lead?: number;
  rivalLead?: number;
  /** Decadence settings a game changes (`view.rule_experiments`). */
  rules?: RuleSwitches;
  /** Eras you have scored. */
  scored?: number[];
  /** Your progress, per era and slot: [current, target]. */
  progress?: Record<string, [number, number]>;
}

export function withV2(base: PlayerView, o: V2Options = {}): PlayerView {
  const view = structuredClone(base);
  view.rule_experiments = { ...view.rule_experiments, ...((o.lead || o.rivalLead) ? { decadence_slope: 45 } : {}), ...o.rules };
  const era = o.era ?? 2, scored = o.scored ?? [];
  const rival = view.players.find((p) => p.team !== view.team && p.alive)!;
  const others = view.players.filter((p) => p.alive && p.id !== view.me);
  view.players = view.players.map((p) => {
    const pe = p.id === view.me ? era : p.id === rival.id ? 3 : 1;
    const lead = p.id === view.me ? o.lead ?? 0 : p.id === rival.id ? o.rivalLead ?? 0 : 0;
    const rules = decadenceRules(view.rule_experiments);
    return {
      ...p, era: pe, economy: 40, army: 40, advancement: pe * 1000 + 3, power: 100 + lead, lead_percent: lead,
      decadence_percent: decadencePercent(lead, rules),
    };
  });
  // An Age is in the view from the turn the first empire enters its era.
  view.eras = V2_ERAS.filter((ev) => ev.era <= Math.max(era, 3)).map((ev) => ({
    ...structuredClone(ev),
    entered_by: [...(ev.era <= era ? [view.me] : []), ...(ev.era <= 3 ? [rival.id] : []), ...(ev.era === 2 && others[1] ? [others[1].id] : [])],
    scored_by: [
      ...(scored.includes(ev.era) ? [{ player: view.me, slot: 1, turn: 40 }] : []),
      ...(ev.era === 2 ? [{ player: rival.id, slot: 0, turn: 37 }] : []),
    ],
  }));
  const prog: Record<string, [number, number]> = o.progress ?? { '2:0': [6, 9], '2:1': [2, 4], '2:2': [0, 1], '3:0': [6, 14], '3:1': [11, 40], '3:2': [0, 1] };
  view.objective_progress = view.eras.flatMap((ev) => ev.objectives.map((ob): ObjectiveProgress => {
    const [current, target] = prog[`${ev.era}:${ob.slot}`] ?? [0, ob.target];
    return { era: ev.era, slot: ob.slot, current, target, entered: true, era_scored: scored.includes(ev.era) };
  }));
  view.players = view.players.map((p) => {
    const sources = p.vp_sources.filter(([s]) => s.kind !== 'EraObjective');
    const eraSources = view.eras.filter((ev) => ev.scored_by.some((s) => s.player === p.id)).map((ev) => [{ kind: 'EraObjective' as const, era: ev.era }, 1] as [{ kind: 'EraObjective'; era: number }, number]);
    return { ...p, vp_sources: [...sources, ...eraSources] };
  });
  return view;
}

/**
 * A World Age game on top of `withV2(base, { era: 3 })`: two open Ages (Classical and Medieval), each with the one Veterans card,
 * your progress 2 of 4 and 1 of 4, and a rival who has scored Classical.
 */
export function withWorldAges(base: PlayerView): PlayerView {
  const view = withV2(base, { era: 3, scored: [] });
  view.eras = view.eras.filter((e) => e.era <= 3).map((e) => ({
    ...e,
    objectives: [{ slot: 0, family: 'Fight' as const, text: 'Veterans: earn 4 promotions in battle (a promotion is a level a unit gains from combat experience after this Age began)', target: 4, objective: { kind: 'Veterans', n: 4 } }],
  }));
  view.objective_progress = [
    { era: 2, slot: 0, current: 2, target: 4, entered: true, era_scored: false, ready: false },
    { era: 3, slot: 0, current: 1, target: 4, entered: true, era_scored: false, ready: false },
  ];
  return view;
}
