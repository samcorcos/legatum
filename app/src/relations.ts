import type { PlayerView, Relation, TeamId } from './types';
import { empireTitle } from './empires';

export interface RelationsEmpire {
  team: TeamId;
  name: string;
  known: boolean;
}

export interface RelationsCell {
  a: TeamId;
  b: TeamId;
  state: Relation['state'] | 'Unknown' | 'Self';
  title: string;
  effectiveTurn?: number;
}

/** Public relations, masked until the viewer has met both empires. */
export function publicRelationsModel(view: PlayerView): { empires: RelationsEmpire[]; cells: RelationsCell[][] } {
  const known = new Set([view.team, ...(view.diplomacy?.met ?? [])]);
  const unknownNumbers = new Map(view.standings.filter((s) => !known.has(s.team)).map((s, i) => [s.team, i + 1]));
  const label = (team: TeamId) => known.has(team) ? (team === view.team ? 'Your empire' : empireTitle(view, team))
    : `Unknown empire ${unknownNumbers.get(team) ?? ''}`.trim();
  const empires = view.standings
    .filter((standing) => standing.team === view.team || view.players.some((p) => p.team === standing.team && p.alive))
    .map((standing) => {
      const met = known.has(standing.team);
      return { team: standing.team, name: label(standing.team), known: met };
    });
  const relations = view.diplomacy?.relations ?? [];
  const cell = (a: RelationsEmpire, b: RelationsEmpire): RelationsCell => {
    if (a.team === b.team) return { a: a.team, b: b.team, state: 'Self', title: `${a.name}` };
    if (!a.known || !b.known) return { a: a.team, b: b.team, state: 'Unknown', title: 'Unknown relation' };
    const relation = relations.find((r) => (r.a === a.team && r.b === b.team) || (r.a === b.team && r.b === a.team));
    if (!relation) return { a: a.team, b: b.team, state: 'Unknown', title: 'Unknown relation' };
    const title = `${a.name} and ${b.name}: ${relation.state === 'War' ? 'at War' : relation.state === 'Alliance' ? 'allied' : 'at Peace'} since turn ${relation.effective_turn}.`;
    return { a: a.team, b: b.team, state: relation.state, title, effectiveTurn: relation.effective_turn };
  };
  return { empires, cells: empires.map((a) => empires.map((b) => cell(a, b))) };
}
