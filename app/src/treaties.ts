// Plain one-line definitions of the diplomatic states, shown first wherever a player meets them
// (offer cards, Offer buttons, relation headers, the Guide), with the full rules one click further.
// Numbers come from the core snapshot (rules.generated.json).
import { RULES } from './data';

const { Alliance, guaranteeTurns } = RULES.treaty;

/** Alliance Gold terms: upkeep per owned city and the early-exit cost. */
export function allianceTerms(cities = 1) {
  return {
    upkeep: Alliance.upkeep * Math.max(1, cities),
    breachCost: Alliance.breachCost,
    protectedTurns: Alliance.protectedTurns,
    betrayalTurns: Alliance.betrayalTurns,
  };
}

export const ALLIANCE_SHARES = 'Allies defend each other at once, and each ally chooses whether to join the other’s wars. They share sight and can cross each other’s borders.';

export const TREATY_DEFINITION = {
  Peace: `You can’t attack each other. Peace signed after a War starts a ${guaranteeTurns}-turn truce; neither side can declare War or be pulled into War with the other during it. After it ends, a declaration starts War at once.`,
  Alliance: `${ALLIANCE_SHARES} Each ally pays Gold a turn for its cities, but nothing while at War with an empire close to winning.`,
  War: `You can attack each other, and so can the allies who joined each side. A Peace treaty ends only the war between its two teams once both agree, then starts a ${guaranteeTurns}-turn truce for that pair. Other wars continue.`,
} as const;

export type DefinedTreaty = keyof typeof TREATY_DEFINITION;

export const treatyDefinition = (kind: string | null | undefined): string | null =>
  kind && kind in TREATY_DEFINITION ? TREATY_DEFINITION[kind as DefinedTreaty] : null;

/** Alliance terms and the peace guarantee, for copy that states them. */
export const TREATY_TERMS = { Alliance, guaranteeTurns };
