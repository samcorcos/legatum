import type { SettlementSite } from '../types';
import { WONDER_INFO, WONDER_PERCENT, wonderYieldWord } from '../wonders';
import { useState } from 'react';
import type { Yields } from '../types';
import { YieldIcon, signed } from '../ui/bits';
import { Amounts } from '../ui/Amounts';
import { usePlacedCard, type CardAnchor } from '../ui/usePlacedCard';
import { bandRange, bandTitle, qualityLines, qualityParts, settleSummary, siteVerdict, SETTLE_BANDS } from './settleAdvice';

/** Unclaimed natural wonders a city here could own: each with its bonus, as a block of its own under the quality lines. */
function NearbyWonders({ site }: { site: SettlementSite }) {
  const near = site.wonders;
  if (!near.length) return null;
  return <ul className="settlement-wonders" data-testid="settle-wonders">{near.map((n, i) => (
    <li key={i}><b>{WONDER_INFO[n.wonder].name}</b> {n.distance === 0 ? 'on the site' : `${n.distance} ${n.distance === 1 ? 'tile' : 'tiles'} away`}: the city that owns it gets <b>+{WONDER_PERCENT}% {wonderYieldWord(n.wonder)}</b> <YieldIcon k={WONDER_INFO[n.wonder].yield} size={13} /></li>
  ))}</ul>;
}

/** The hover: one plain sentence, the key numbers, then the relative half (top-three marker, travel). */
function SettlementSummary({ site, faction }: { site: SettlementSite; faction?: string }) {
  const verdict = siteVerdict(site, faction);
  return <>
    <p className="settlement-summary" data-settle-band={site.band} title={bandTitle(site)}><i className={`settle-swatch band-${site.band.toLowerCase()}`} aria-hidden="true" /><span><b>{verdict.head}:</b> {verdict.rest}</span></p>
    <ul className="settlement-reasons" aria-label="Key numbers for this spot">{qualityLines(site).map((line) => <li key={line.id} data-settle-line={line.id}><Amounts text={line.text} /></li>)}</ul>
    <NearbyWonders site={site} />
    <p className="settlement-route" data-settle-route>{settleSummary(site)}</p>
  </>;
}

const YieldLine = ({ yields }: { yields: Yields }) => <span className="settlement-yields">{(['food', 'production', 'gold', 'science'] as const).filter((k) => yields[k] !== 0).map((k) => <span key={k}><YieldIcon k={k} /> {yields[k]}</span>)}</span>;
export function SettlementDetails({ site, faction }: { site: SettlementSite; faction?: string }) {
  const [expanded, setExpanded] = useState(false);
  return <>
    <SettlementSummary site={site} faction={faction} />
    <details className="settlement-breakdown" open={expanded} onToggle={(e) => setExpanded(e.currentTarget.open)}>
    <summary>Details</summary>
    <div className="settlement-score"><b>Site quality: {site.quality} ({site.band}, {bandRange(site.band)})</b></div>
    <p className="muted">Quality is a fixed scale, the same on every map and for every Settler of your people: {SETTLE_BANDS.map((b) => `${b} ${bandRange(b)}`).join(', ')}.</p>
    <dl className="settlement-math" aria-label="Site quality calculation">{qualityParts(site, faction).map((part) => <div key={part.id + part.label}><dt>{part.label}</dt><dd>{signed(part.points)}</dd></div>)}</dl>
    <p className="muted">For this Settler, travel and threats change the score to {site.score}; the band does not move.</p>
    <dl className="settlement-math" aria-label="Suitability calculation">{site.breakdown.filter((part) => part.value !== 0 || part.label.startsWith('India local network') || part.label.startsWith('Corruption:')).map((part, i) => <div key={i}><dt>{part.label}</dt><dd>{signed(part.value)}</dd></div>)}</dl>
    <p>Center terrain <YieldLine yields={site.center_yields} /></p>
    <p>Founding border: {site.ring_one.length ? <>{site.ring_one.length} tiles worked from turn one <YieldLine yields={site.ring_one.reduce((sum, t) => ({ food: sum.food + t.yields.food, production: sum.production + t.yields.production, gold: sum.gold + t.yields.gold, science: sum.science + t.yields.science }), { food: 0, production: 0, gold: 0, science: 0 })} /></> : 'no known free tile in the initial border.'}</p>
    <p className="muted">A city works every tile it owns within 3, and each population claims one more, nearest first. Terrain yields only; city income is added separately.</p>
    <p>Land within 3 tiles (outer ring at half) <YieldLine yields={site.potential_yields} /></p>
    {Object.values(site.terrain_bonus).some(Boolean) && <p>Faction terrain bonus <YieldLine yields={site.terrain_bonus} /></p>}
    <p>{site.nearest_city === null ? 'Your first city' : `${site.nearest_city} hexes to your nearest city`} · {site.travel_turns === null ? 'No known route' : `${site.travel_turns} travel turns`}</p>
    <p>Border overlap: {site.own_border_tiles} own · {site.foreign_tiles} foreign tiles. Foreign yields excluded; founding can reassign your initial ring.</p>
    <p>{site.threats ? `${site.threats} known enemy units can attack here next turn.` : 'Outside known enemy attack reach.'}</p>
    <p className="muted">{site.unknown_tiles} unknown tiles within 3. Known legal site; unseen cities or changed borders may block founding. Travel assumes no new blockers.</p>
    </details>
  </>;
}
export function SettlementCard({ site, anchor, faction }: { site: SettlementSite; anchor: CardAnchor; faction?: string }) {
  const placed = usePlacedCard<HTMLDivElement>(anchor);
  return <div className="tilecard settlement-card" ref={placed.ref} style={placed.style} aria-label="Settlement suitability">
    <SettlementSummary site={site} faction={faction} />
    <p className="muted">Click the tile for details.</p>
  </div>;
}
