import { Icon } from '../ui/bits';
import { Amount, Amounts } from '../ui/Amounts';
import { feel } from '../feel';
import { useGame } from '../store';
import type { Moment } from './moments';
import type { UnlockCard, UnlockTile } from './unlockCard';
import { itemArtUrl, slugUrl, unlockEntryArt } from '../unlockArt';
import { resourceArtUrl } from '../icons';
import { costInFactionYield, costYield, isGreatWork } from '../data';
import { parseUpgradeEffect } from '../ui/unlockCardModel';

/** A tile's painted miniature (null while unpainted: the tile keeps its icon). */
const tileArt = (t: UnlockTile) => t.icon.startsWith('resource-')
  ? resourceArtUrl(t.icon.slice('resource-'.length), 256)
  : unlockEntryArt(t.item, t.name, t.tech, t.faction, 256);

const tilePair = (t: UnlockTile): [string | null, string | null] | null => {
  const upgrade = t.kind === 'Effect' ? parseUpgradeEffect(t.name) : null;
  if (upgrade) return [itemArtUrl({ kind: 'Unit', unit: upgrade.from }, t.faction, 128), itemArtUrl({ kind: 'Unit', unit: upgrade.to }, t.faction, 128)];
  // A merged upgrade rule shows the units it makes.
  if (t.upgrades.length) return [itemArtUrl({ kind: 'Unit', unit: t.upgrades[0].to }, t.faction, 128), t.upgrades[1] ? itemArtUrl({ kind: 'Unit', unit: t.upgrades[1].to }, t.faction, 128) : null];
  if (/^Farms and Pastures upgrade/.test(t.name)) return [slugUrl('improvement-irrigated-farm', 128), slugUrl('improvement-ranch', 128)];
  return null;
};

function TileIllustration({ tile, multiple }: { tile: UnlockTile; multiple: boolean }) {
  const pair = tilePair(tile);
  if (pair?.some(Boolean)) return <span className="unlock-art-pair" aria-hidden="true">{pair.map((url, n) => url && <img key={n} src={url} alt="" draggable={false} />)}</span>;
  const art = tileArt(tile);
  return art ? <img className="unlock-art" src={art} alt="" draggable={false} />
    : <span className="unlock-frame"><Icon name={tile.icon} size={multiple ? 30 : 40} /></span>;
}

/** Open the tech panel to pick what to research next. */
export function chooseResearch(): void {
  useGame.getState().set({ reportOpen: false, techOpen: true });
}

/** The card's primary action (Return): open the city that builds the unlock first, else pick research, else nothing. */
export function activateUnlock(card: UnlockCard): void {
  const s = useGame.getState();
  if (card.primary === 'show' && card.show) {
    s.set({ reportOpen: false, techOpen: false });
    s.select({ kind: 'city', id: card.show.city }, true);
  } else if (card.primary === 'research') chooseResearch();
}

const turnsText = (n: number) => `${n} turn${n === 1 ? '' : 's'}`;
const kindLabel = (t: UnlockTile) => t.item?.kind === 'Building' && isGreatWork(t.item.building)
  ? 'World wonder' : t.kind === 'Effect' ? (parseUpgradeEffect(t.name) ? 'Upgrade' : 'Rule') : t.kind;

/**
 * Civ 6's "you unlocked something" moment for a technology you finished (legatum-ui). Centred over the
 * map; the scrim lets clicks through, so End Turn and the map stay usable. It waits for Return (the
 * primary action) or Esc (later) and never times out.
 */
export function UnlockMomentCard({ moment, dismiss }: { moment: Moment; dismiss(): void }) {
  const card = moment.unlock!;
  const items = card.tiles.filter((tile) => tile.item);
  // One item plus resource reveals keeps the large painting, but only on a tall window (celebrate.css, min-height 701px): on a short one every row must be on screen.
  const soleItem = items.length === 1 && card.tiles.every((tile) => tile === items[0] || tile.icon.startsWith('resource-'));
  return <>
    <div className="unlock-scrim" aria-hidden="true" />
    <article key={moment.id} className="moment-card unlock" data-primary={card.primary} data-dense={card.tiles.length > 2 ? 'true' : undefined} data-tier={moment.tier} data-tech={card.tech} aria-label={`${moment.title}: technology discovered`}>
      <div className="unlock-crest"><span className="unlock-rays" aria-hidden="true" />
        <span className="unlock-seal"><Icon name={card.icon} size={30} /></span></div>
      <div className="unlock-kicker">{card.kicker}</div>
      <h2 className="unlock-title">{moment.title}</h2>
      <p className="unlock-quote">“{card.quote}”</p>
      <div className="unlock-rule" aria-hidden="true"><svg width="10" height="10" viewBox="-5.5 -5.5 11 11"><path d="M0 -5L.95 -.95L5 0L.95 .95L0 5L-.95 .95L-5 0L-.95 -.95Z" fill="var(--accent)" /></svg></div>
      <div className="unlock-body">
        <div className="unlock-label">Unlocks</div>
        <ul className="unlock-tiles" data-count={card.tiles.length} data-hero={soleItem ? 'true' : undefined} aria-label="Unlocks">
          {card.tiles.map((t, i) => <li key={`${t.key ?? t.name}-${i}`} className={`unlock-tile ${t.kind.toLowerCase()}${t.icon.startsWith('unit-') ? ' unit-art' : ''}`}>
            <span className="unlock-kind">{kindLabel(t)}</span>
            <TileIllustration tile={t} multiple={card.tiles.length > 1} />
            <span className="unlock-text">
              <span className="unlock-name" title={t.name}>{t.name}</span>
              {t.kind !== 'Effect' && (t.cost !== null || t.turns !== null || t.upgradeFrom) && <span className="unlock-cost">{t.cost !== null && <Amount n={costInFactionYield(t.faction, t.cost)} unit={costYield(t.faction)} />}{t.turns !== null && t.inCity && ` · ${turnsText(t.turns)} in ${t.inCity}`}{t.upgradeFrom && `${t.cost !== null || t.turns !== null ? ' · ' : ''}From ${t.upgradeFrom}`}</span>}
              {t.sentence && <span className="unlock-sentence" title={t.sentence}><Amounts text={t.sentence} /></span>}
              {t.notes.length > 0 && <span className="unlock-cost unlock-notes">{t.notes.join(' · ').replace(/^./, (c) => c.toUpperCase())}</span>}
            </span>
          </li>)}
        </ul>
      {card.next && <p className="unlock-next">Research continues: <b>{card.next.name}</b>{card.next.turns !== null && ` · ${turnsText(card.next.turns)}`}{card.next.unlocks && ` · unlocks ${card.next.unlocks}`}</p>}
      {card.chooseNext && <p className="unlock-next" data-testid="unlock-choose-next"><b>No research is set.</b> Choose the next one under Technologies.</p>}
      </div>
      <footer className="unlock-foot">
        <button className="btn primary" data-primary-action onClick={() => { feel.select(); dismiss(); activateUnlock(card); }}>
          {card.primary === 'show' && card.show ? <>Show {card.show.itemName} in {card.show.cityName}</> : card.primary === 'research' ? 'Choose research' : 'Continue'}{' '}
          <span className="kbd">Return</span></button>
        {card.chooseNext && card.primary === 'show' && <button className="btn quiet" onClick={() => { feel.select(); dismiss(); chooseResearch(); }}>Choose research <span className="kbd">T</span></button>}
        {card.primary !== 'continue' && <button className="btn quiet" onClick={() => { feel.select(); dismiss(); }}>Later <span className="kbd">Esc</span></button>}
      </footer>
    </article>
  </>;
}
