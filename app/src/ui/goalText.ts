import { factionGoalVp } from '../data';

/** The Yam Relay Station's build card in plain words: what it scores, what it opens, and the one-per-empire rule. */
export const hallCardText = () => `+${factionGoalVp('Mongols')} VP while you hold it · every empire crosses all your land at Peace · one per empire, rebuilt only if destroyed`;

/** The core's reason for a Station that already stands (here or in another of your cities): the card reads as Built, not as a wait. */
export const hallStands = (reason: string) => reason.startsWith('The Yam Relay Station already stands in');

/** Every goal measure caps at the scoring target, the Ruhr Foundry count included ("5 of 5", never "7 of 5"; 8.9.23 review). */
export const cappedGoalLabel = (label: string): string => label.replace(/^([\d,]+) of ([\d,]+)\b/, (whole, held: string, target: string) =>
  Number(held.replace(/,/g, '')) > Number(target.replace(/,/g, '')) ? `${target} of ${target}` : whole);
