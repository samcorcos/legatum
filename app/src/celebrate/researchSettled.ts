import type { Moment } from './moments';

/**
 * A held card's "Choose research" is decided when the turn's story is planned, from the view after the turn. The
 * player can pick research before the card shows (it waits behind the open research screen), so once research is
 * chosen the card drops that call: its primary becomes Continue (or Show, for an unlock) and Return never reopens
 * the research screen.
 */
export function settleResearch(moment: Moment | null, researchChosen: boolean): Moment | null {
  if (!moment || !researchChosen) return moment;
  const { unlock, eraUp } = moment;
  if (eraUp?.primary === 'research') return { ...moment, eraUp: { ...eraUp, primary: 'continue', researchSet: true } };
  if (unlock?.chooseNext) return { ...moment, unlock: { ...unlock, chooseNext: false, primary: unlock.primary === 'research' ? 'continue' : unlock.primary } };
  return moment;
}
