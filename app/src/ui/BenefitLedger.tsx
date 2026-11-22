import type { Faction } from '../types';
import { itemArtUrl, slugUrl, unlockEntryArt } from '../unlockArt';
import { resourceArtUrl } from '../icons';
import { Amount, Amounts } from './Amounts';
import { signedAmount, type AmountUnit } from './amountParts';
import { Icon } from './bits';
import { UnlockItem } from './UnlockHover';
import { parseUpgradeEffect } from './unlockCardModel';
import { unitCardUrl } from '../unitArt';
import type { TechCardUnlock } from './techCardUnlocks';
import { YIELD_LABEL } from './yieldKeys';

function heading(entry: TechCardUnlock): string {
  if (entry.wonder) return 'World wonder';
  if (entry.kind !== 'Effect') return entry.kind;
  return parseUpgradeEffect(entry.label) ? 'Upgrade' : 'Rule';
}

/** A card keeps the point of a long rule on one line; its detail card has the exact rule. */
function line(entry: TechCardUnlock): string {
  const label = entry.locked ?? entry.label;
  if (entry.kind !== 'Effect') return label;
  return label
    .replace(/^Reveals (.+?) on the map(?: · .*)?$/, 'Reveals $1')
    .replace(/^Farms and Pastures upgrade: .*\(\+1 Food\)$/, 'Farm/Pasture +1 Food')
    .replace(/^Mines upgrade: .*\(\+1 Production\)$/, 'Mines +1 Production')
    .replace(/^Mines upgrade: Deep Mine \(\+1 Gold on Copper, \+1 Production on Iron and Coal\)$/, 'Mines +1 Production, Copper +1 Gold')
    .replace(/^Quarries upgrade: .*\(\+1 Production\)$/, 'Quarries +1 Production')
    .replace(/^Siege units now ignore (.+)$/, 'Ignore $1')
    .replace(/^Units can buy Siege: they ignore enemy Walls$/, 'Buy Siege: ignore Walls')
    .replace(/^Units that start a turn on a road get \+2 moves$/, 'Road start: +2 moves')
    // The full line stays in the hover; the card keeps each rule to two lines.
    .replace(/^(\+1 \w+) for each Workshop, .+$/, 'Workshops and upgrades $1')
    .replace(/^(\+1 \w+) for each Library, .+$/, 'Libraries and upgrades $1')
    .replace(/^Land units cost 10% less Production$/, 'Land units 10% cheaper')
    .replace(/^Factories and Research Labs need no Coal or Oil$/, 'Factories and Labs need no Coal or Oil')
    .replace(/^Reveal the whole map to your team; units still need sight$/, 'Reveal the whole map to your team')
    .replace(/^Each Research Lab gains (.+)$/, 'Research Labs $1')
    .replace(/^Upgrade Gold costs [-−]50%$/, 'Upgrade Gold −50%');
}

function LedgerArt({ entry, faction }: { entry: TechCardUnlock; faction: Faction }) {
  const upgrade = entry.kind === 'Effect' ? parseUpgradeEffect(entry.label) : null;
  // One upgrade shows the unit and what it becomes; a merged rule shows the units it makes.
  const units = upgrade ? [upgrade.from, upgrade.to] : entry.upgrades.length === 1 ? [entry.upgrades[0].from, entry.upgrades[0].to] : entry.upgrades.map((u) => u.to);
  const pair = units.length
    ? units.map((unit) => unitCardUrl(unit, faction) ?? itemArtUrl({ kind: 'Unit', unit }, faction, 128))
    : /^Farms and Pastures upgrade/.test(entry.label)
      ? [slugUrl('improvement-irrigated-farm', 128), slugUrl('improvement-ranch', 128)]
      : null;
  const resource = entry.icon.startsWith('resource-') ? entry.icon.slice('resource-'.length) : null;
  const art = resource ? resourceArtUrl(resource, 64) : unlockEntryArt(entry.unlock?.item, entry.label, entry.tech, faction, 128);
  return <span className={`era-unlock-art${pair ? ' pair' : ''}`} data-art={entry.kind.toLowerCase()} aria-hidden="true">
    {pair ? pair.map((url, i) => url && <img src={url} key={i} alt="" draggable={false} />)
      : art ? <img src={art} alt="" draggable={false} /> : <Icon name={entry.icon} size={28} />}
  </span>;
}

/** One illustrated row per benefit. Full descriptions remain in each row's focusable detail card. */
export function BenefitLedger({ entries, faction }: { entries: TechCardUnlock[]; faction: Faction }) {
  const items = entries.filter((entry) => entry.kind !== 'Effect');
  const soleItem = items.length === 1 && entries.every((entry) => entry === items[0] || entry.icon.startsWith('resource-'));
  return <ul className="era-unlocks benefit-ledger" data-benefits={entries.length} data-hero={soleItem ? 'true' : undefined} aria-label="Everything this technology unlocks">
    {entries.map((entry, index) => <UnlockItem key={`${entry.kind}-${entry.label}-${index}`} entry={entry} faction={faction}
      className={`era-unlock ${entry.kind.toLowerCase()}${entry.locked ? ' locked' : ''}${entry.notes.length ? ' has-notes' : ''}${entry.upgrades.length ? ' upgrade-rule' : ''}`}>
      <span className="ledger-heading">{heading(entry)}</span>
      <LedgerArt entry={entry} faction={faction} />
      <span className="era-unlock-text">
        <span className="era-unlock-label" title={entry.locked ?? entry.label}><Amounts text={line(entry)} /></span>
        {!entry.locked && (entry.yields.length > 0 || entry.strength) && <span className={`era-unlock-gain${entry.kind === 'Unit' ? ' unit-line' : ''}`}>
          {entry.yields.map(([k, v]) => <Amount key={k} sign={signedAmount(v)} n={Math.abs(v)} unit={YIELD_LABEL[k] as AmountUnit} />)}
          {entry.strength && <span>{[entry.strength, ...entry.notes].join(' · ')}</span>}
        </span>}
      </span>
      {entry.upgradeFrom && <span className="ledger-upgrade" title={`upgrades from ${entry.upgradeFrom}`}>From {entry.upgradeFrom}</span>}
    </UnlockItem>)}
  </ul>;
}
