// A hired Mercenary Company in the unit panel: whose it was, how long it serves and what it costs. Public facts
// (`contract_until`, `origin`), so a rival's Company shows the same countdown the map badge does.
import { COMPANY_UNIT_RULE, contractLeft, contractLine } from '../company';
import type { PlayerView, UnitView } from '../types';
import { Amounts } from './Amounts';
import { Icon } from './bits';

/** The status in the panel's head, where it is seen at once: "Company (Spartan): 7 turns of contract left, upkeep 2 Gold a turn". */
export function CompanyTag({ unit, view }: { unit: UnitView; view: PlayerView }) {
  const left = contractLeft(unit, view);
  if (left === null) return null;
  return <div className={`up-company-tag${left === 0 ? ' last' : ''}`} data-testid="company-tag" data-left={left}>
    <Icon name={left === 0 ? 'ui-alert' : 'yield-gold'} size={13} className="ui-mark" /> <Amounts text={contractLine(unit, view)} />
  </div>;
}

/** The rule behind it, among the panel's notes (stated once: the head line carries the numbers). */
export function CompanyNote({ unit, view }: { unit: UnitView; view: PlayerView }) {
  if (contractLeft(unit, view) === null) return null;
  return <div className="up-company" data-testid="company-note">
    <Icon name="yield-gold" size={16} className="up-company-mark" />
    <p className="up-company-rule"><Amounts text={COMPANY_UNIT_RULE} /></p>
  </div>;
}
