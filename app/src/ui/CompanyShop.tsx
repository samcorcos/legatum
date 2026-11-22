// Byzantium's shop in the city panel: hire a Mercenary Company from a met empire it is not at War with. Each empire sells only
// its own special unit, so the shop is one compact row per seller; the Gold goes to that seller in full. Rows are priced and
// refused by the core's own rules (see `purchasePreviews`), so a row that cannot be pressed says why in one line under it.
import { RULES, specialUnitEdge } from '../data';
import { companyName, companyStats, hireOrder, shopRows, type ShopRow } from '../company';
import { empireOfPlayer } from '../empires';
import { toggleHire } from '../orders';
import { withArticle } from '../resources';
import { useGame } from '../store';
import { playerColor } from '../theme';
import type { CityView, PlayerView } from '../types';
import { Amounts } from './Amounts';
import { PortraitRoundel } from './premium';
import { ItemArt } from './ItemArt';
import { Icon, signed } from './bits';
import { Tip } from './Tip';
import { ONE_PURCHASE, type HirePreview } from './productionPurchases';
import './CompanyShop.css';

function Row({ city, view, row, preview }: { city: CityView; view: PlayerView; row: ShopRow; preview: HirePreview }) {
  const editOrders = useGame((s) => s.editOrders);
  const { offer, faction } = row;
  const seller = empireOfPlayer(view, offer.seller);
  const name = companyName(faction, offer.unit);
  const crest = <PortraitRoundel faction={faction} size={22} color={playerColor(offer.seller)} />;
  // A seller that cannot field its special unit yet: one muted line saying why, nothing to press.
  if (offer.unavailable) {
    return <div className="company-row unavailable" data-testid="company-row" data-seller={offer.seller} data-unit={offer.unit}>
      {crest}
      <p className="company-unavailable" data-testid="company-unavailable">{`${seller}: ${offer.unavailable}`}</p>
    </div>;
  }
  const label = `Hire ${withArticle(name)} Company from ${seller}`;
  const blocked = !preview.selected && !!preview.reason;
  // "One purchase a turn" is the same for every other row: the section says it once, so the rows stay quiet.
  const oneAtATime = preview.reason === ONE_PURCHASE;
  const left = preview.gold - offer.price;
  const income = view.economy.gold_per_turn.total;
  const afterIncome = income - RULES.company.upkeep;
  // Core `rules/cities.rs::pay_gold` turns a Company when treasury + this turn's net is negative.
  const unpaid = afterIncome < 0 && left >= 0 ? Math.floor(left / -afterIncome) + 1 : null;
  const mutiny = unpaid !== null && unpaid <= RULES.company.turns;
  const lackGold = !!preview.reason?.startsWith('Not enough Gold');
  const needId = `company-why-${city.id}-${offer.seller}-${offer.unit}`;
  // Where the button would be: the shortfall in words and an icon (never colour alone); any other refusal is a line under the row.
  const short = lackGold ? `Need ${offer.price - preview.gold} more` : 'Cannot hire';
  const dead = blocked && !oneAtATime;
  return <div className={`company-row${preview.selected ? ' chosen' : ''}${blocked ? ' blocked' : ''}`} data-testid="company-row" data-seller={offer.seller} data-unit={offer.unit}>
    {crest}
    <span className="company-id">
      <span className="company-seller-name">{seller}</span>
      <span className="company-unit"><ItemArt item={{ kind: 'Unit', unit: offer.unit }} faction={faction} size={20} /> <span className="company-name">{name}</span></span>
      <span className="company-stats"><Amounts text={`${companyStats(faction, offer.unit)} · ${specialUnitEdge(faction)}`} /></span>
    </span>
    <span className="company-price"><Amounts text={`${offer.price} Gold`} /></span>
    <Tip placement="left" content={<>
      <p><Amounts text={`${label} for ${offer.price} Gold. The Gold goes to ${seller} in full.`} /></p>
      <p><Amounts text={`It joins ${city.name} next turn, serves ${RULES.company.turns} turns, then goes home. Upkeep ${RULES.company.upkeep} Gold a turn; if you cannot pay it, the Company turns barbarian where it stands.`} /></p>
      {unpaid !== null && !mutiny && <p>At your current income, its contract ends before the treasury runs dry.</p>}
      {!preview.selected && <p><Amounts text={preview.reason ?? `Costs ${offer.price} of your ${preview.gold} Gold, leaving ${left}.`} /></p>}
    </>}>
      <button className={`btn-buy company-hire${preview.selected ? ' active' : ''}${dead ? ' short' : ''}`} aria-pressed={preview.selected} aria-label={`${label} for ${offer.price} Gold`}
        aria-describedby={dead ? needId : 'company-one-purchase'}
        disabled={view.finished || blocked}
        onClick={() => editOrders((orders) => toggleHire(orders, hireOrder(city.id, offer)), preview.selected ? undefined : 'purchase')}>
        {preview.selected ? <><Icon name="ui-check" size={13} className="ui-mark" /> Hiring</>
          : dead ? <span className="company-why" id={needId}><Icon name="ui-alert" size={12} className="ui-mark" /> {short}</span> : 'Hire'}
      </button>
    </Tip>
    {dead && !lackGold && <p className="company-why-long"><Amounts text={preview.reason!} /></p>}
    {mutiny && <p className="company-impact danger"><Amounts text={`At your current income, treasury cannot pay at the end of turn ${view.turn + unpaid! - 1}; an unpaid Company turns barbarian where it stands.`} /></p>}
  </div>;
}

/** The section: a header with the contract terms once, then one row per seller (or the reason there is none). */
export function CompanyShop({ city, view, hires }: { city: CityView; view: PlayerView; hires: Map<string, HirePreview> }) {
  const editOrders = useGame((s) => s.editOrders);
  const rows = shopRows(view);
  const pending = [...hires.values()].some((h) => h.reason === ONE_PURCHASE);
  const income = view.economy.gold_per_turn.total;
  const count = rows.filter((r) => !r.offer.unavailable).length;
  return <section aria-label="Hire a Mercenary Company" data-testid="company-shop" className="company-shop">
    <Tip as="div" className="cp-sub" content={<><p><Amounts text={`Pay a met empire that is not at War with you ${RULES.company.hirePercent}% of its special unit's Production cost, in Gold. The Gold goes to that empire. The Company joins this city next turn.`} /></p>
      <p><Amounts text={`It serves ${RULES.company.turns} turns, then goes home. It costs ${RULES.company.upkeep} Gold a turn, cannot be upgraded and does not count against thin levies. Everyone sees its seller and its countdown.`} /></p></>}>
      Hire a Mercenary Company <span className="cp-sub-note">· {count === 0 ? 'none for hire' : `${count} for hire`}</span>
    </Tip>
    <p className="company-gets" data-testid="company-gets">Each empire sells only its own special unit. It arrives as that empire's version, with its stats, for {RULES.company.turns} turns.</p>
    <p className="company-rule"><Amounts text={`${RULES.company.turns}-turn contract · upkeep ${RULES.company.upkeep} Gold a turn each (Gold/turn ${signed(income)} → ${signed(income - RULES.company.upkeep)} after a hire) · does not count against thin levies`} /></p>
    {pending && <p className="company-one" id="company-one-purchase" role="status">A city has one purchase a turn: cancel the current one to choose another. <button className="link" onClick={() => editOrders((ord) => ord.filter((o) => !((o.type === 'HireCompany' || o.type === 'BuyItem') && o.city === city.id)))}>Cancel it</button></p>}
    {rows.length === 0
      ? <p className="company-empty" data-testid="company-empty">Nothing to hire yet: it takes an empire you have met and are not at War with.</p>
      : <div className="company-offers">{rows.map((r) => <Row key={r.offer.seller} city={city} view={view} row={r} preview={hires.get(`${r.offer.seller}:${r.offer.unit}`)!} />)}</div>}
  </section>;
}
