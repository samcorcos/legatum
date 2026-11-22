import type { Faction, Hex, Order, PlayerView, UnitKind } from '../types';
import { TERRAIN_YIELDS, FACTION_ADJECTIVE, BUILDING_NAME, WORK_RADIUS, defenseBonus, eraName, moveCost, itemName, improvementName, isGreatWork, unitEraTier } from '../data';
import { improvementKindFor, isDormant } from '../improvements';
import { distance, hexEq, tileIndex } from '../hex';
import { isCalmRouteNote } from '../path';
import { empireOfPlayer, placeName } from '../empires';
import { Icon, YIELD_KEYS, YIELD_LABEL, YieldIcon } from './bits';
import { terrainYields } from '../settlement';
import { RESOURCE_INFO, bonusYieldName, empireHolds, isStrategic, lackedResources, resourceBenefitLines, tileYields } from '../resources';
import { playerColor, playerInk } from '../theme';
import { cityOccupiedLine, cityProductionWarning, occupationLockWarning, occupationTileLine } from './forecastWarnings';
import { productionProgress } from '../cityProgress';
import { Amount, Amounts } from './Amounts';
import { cityFireStatus } from '../format';
import { usePlacedCard, type CardAnchor } from './usePlacedCard';
import { BARBARIAN_INK, campAt, CAMP_CLEAR_GOLD, isBarbarian } from '../barbarians';
import { useGame } from '../store';
import { buildingIconName, factionGoalVp, wonderTileText } from '../data';
import { heirLines } from './HeirNote';
import { contractLine, isCompany } from '../company';
import { heirHoverLine, heirTag, isHeir } from '../heir';
import { atWar } from '../diplomacy';
import { WONDER_INFO, cityWonders, holdingCity, wonderArt, wonderBonusParts, wonderDiscoveryLine, wonderYieldWord } from '../wonders';
import { FactionSeal } from '../brand/ornaments';
import { RankMark } from './RankMark';
import { sleepOrderText } from '../orders';
import { capitalMark, replacementTileLine } from '../capitalMark';
import { CapitalNote, ReplacementNote } from './CapitalStar';

/** "Mongol Scout", but a faction's own piece already carries its name ("Spartan Hoplite"). */
const ownedName = (adjective: string, unit: string) => unit.startsWith(`${adjective} `) ? unit : `${adjective} ${unit}`;
export const tileDefenceText = (bonus: number) => `+${bonus} defence`;

/** Delayed hover card describing a tile. */
/** A stable empty list: a fresh `[]` from the store selector re-renders forever. */
const NO_ORDERS: Order[] = [];

export function TileCard({ view, hex, anchor, routeWarning, blockedReason, reinforcement, note, refusal }: { view: PlayerView; hex: Hex; anchor?: CardAnchor; routeWarning?: string; blockedReason?: string; reinforcement?: string; note?: string; refusal?: string }) {
  const placed = usePlacedCard<HTMLDivElement>(anchor);
  const selection = useGame((st) => st.selection);
  const orders = useGame((st) => st.orders[st.seat]) ?? NO_ORDERS;
  const t = view.tiles[tileIndex(hex, view.width, view.height)];
  if (!t?.terrain) {
    if (t?.landmark) return <div className="tilecard" ref={placed.ref} style={placed.style}>
      <div className="tilecard-head"><b>{BUILDING_NAME[t.landmark.building]}</b><span className="muted">{placeName(view, hex)}</span></div>
      <div className="tilecard-row">This wonder occupies its tile. Its two points belong to whoever holds the tile.</div>
    </div>;
    // Byzantium's Heir is public wherever he stands: an unexplored tile that holds him still answers the hover, with only what is known.
    const lone = view.units.find((u) => isHeir(u) && hexEq(u.pos, hex));
    if (!lone) return null;
    return <div className="tilecard" ref={placed.ref} style={placed.style}>
      <div className="tilecard-head"><b>{heirTag(view, lone) ?? 'Heir'}</b><span className="muted">{placeName(view, hex)}</span></div>
      <div className="tilecard-row" data-testid="heir-tile"><Amounts text={heirHoverLine(view, lone, factionGoalVp('Byzantium')) ?? ''} /></div>
      <div className="tilecard-row muted">Unexplored: his position is public, the land around him is not.</div>
    </div>;
  }
  // The same faction-adjusted yields the map's yield icons count; the base terrain is the breakdown.
  const faction = view.players.find((p) => p.id === view.me)?.faction;
  const base = TERRAIN_YIELDS[t.terrain];
  // Worked-tile yields include the resource the viewer knows (the view hides unrevealed ones).
  const y0 = faction ? tileYields(faction, t) : base;
  const terrainOnly = faction ? terrainYields(faction, t.terrain) : base;
  const adjusted = YIELD_KEYS.some((k) => terrainOnly[k] !== base[k]);
  const nile = faction === 'Egypt';
  const resource = t.resource ?? null;
  const mc = moveCost(t.terrain);
  const def = defenseBonus(t.terrain);
  const owner = t.owner !== null ? view.players.find((p) => p.id === t.owner) : null;
  const units = view.units.filter((u) => hexEq(u.pos, hex));
  const city = view.cities.find((c) => hexEq(c.pos, hex));
  const camp = campAt(view, hex);
  // A barbarian camp or unit on an owned tile: it yields nothing to its owner (Civ 6).
  const barbarianHeld = !!owner && !city && (!!camp || units.some((u) => isBarbarian(u.owner)));
  // A rival-held Heir under your selected military unit: moving onto his tile captures him.
  const heir = units.find((u) => u.kind === 'Heir');
  const mover = selection?.kind === 'unit' ? view.units.find((u) => u.id === selection.id && u.owner === view.me && u.military) : undefined;
  const team = (id: number) => view.players.find((p) => p.id === id)?.team === view.team;
  const capturable = !!heir && !!mover && !team(heir.owner) && !isBarbarian(heir.owner) && atWar(view, heir.owner);
  const productionWarning = city?.visible ? cityProductionWarning(city, orders, faction, view) : null;
  const lockWarning = occupationLockWarning(view, mover, city);
  const fire = city?.visible ? cityFireStatus(view, city) : null;
  // One status line, from the same check as the map's faded medallion and the picker's count.
  const kind = resource ? improvementKindFor(resource) : null;
  // Which of your cities builds it: the one whose territory holds the tile, else the nearest.
  const holds = (c: (typeof view.cities)[number]) => !!c.detail?.territory.some((h) => hexEq(h, hex));
  const owningCity = t.owner === view.me ? view.cities.find((c) => c.owner === view.me && holds(c)) : undefined;
  const outOfReach = !!owningCity && distance(owningCity.pos, hex) > WORK_RADIUS;
  const recipient = owningCity && outOfReach ? view.cities
    .filter((c) => c.owner === view.me && c.id !== owningCity.id)
    .map((c) => ({ city: c, distance: distance(c.pos, hex) }))
    .filter(({ distance: d }) => d <= WORK_RADIUS)
    .sort((a, b) => a.distance - b.distance || a.city.id - b.city.id)[0]?.city : undefined;
  const builder = view.cities.filter((c) => c.owner === view.me)
    .sort((x, y) => Number(holds(y)) - Number(holds(x)) || distance(x.pos, hex) - distance(y.pos, hex))[0];
  const dormantNote = resource && kind && isDormant(t, view.me, faction)
    ? `${isStrategic(resource) ? `Its ${bonusYieldName(resource)} bonus is dormant` : 'Dormant'}: build the ${improvementName(kind)}${builder ? ` from ${builder.name}'s production list` : " from a city's production list"}`
    : null;
  // The next action for the tile's resource: improve it (yours), claim it (nobody's) or who holds it.
  const ownerName = owner ? empireOfPlayer(view, owner.id) : 'another empire';
  const resourceStatus = !resource || !kind ? null
    : t.improvement ? `${improvementName(t.improvement.kind, !!t.improvement.upgraded)}: the resource is yielding its bonus`
    : t.owner === null ? (isStrategic(resource) ? 'Unclaimed: bring it inside your borders' : 'Unclaimed: bring it inside your borders to improve it')
    : t.owner !== view.me ? (isStrategic(resource) ? `${empireHolds([ownerName.split(' · ')[0]])} this deposit.${lackedResources(view.supply).has(resource) ? ` You have no ${RESOURCE_INFO[resource].name} yet: take it or find another.` : ''}` : `In ${ownerName} territory`)
    : dormantNote;
  return (
    <div className="tilecard" ref={placed.ref} style={placed.style}>
      {t.landmark ? <div className="tilecard-lead" data-testid="tile-landmark"><Icon name={buildingIconName(t.landmark.building)} size={46} /><div><b className="tilecard-lead-name">{BUILDING_NAME[t.landmark.building]}</b><span className="tilecard-lead-kind">Wonder tile · {owner ? empireOfPlayer(view, owner.id) : 'unclaimed'}</span></div></div> : t.wonder ? <WonderLead view={view} hex={hex} t={t} /> : resource ? (
        <div className="tilecard-lead" data-testid="tile-resource">
          <Icon name={`resource-${resource.toLowerCase()}`} size={46} label={RESOURCE_INFO[resource].name} />
          <div>
            <b className="tilecard-lead-name">{RESOURCE_INFO[resource].name}</b>
            <span className={`tilecard-lead-kind ${isStrategic(resource) ? 'strategic' : 'bonus'}`}>{isStrategic(resource) ? 'Strategic' : 'Bonus'} resource</span>
            {city ? null : <span className="muted tilecard-lead-place">{placeName(view, hex)}</span>}
          </div>
        </div>
      ) : (
        <div className="tilecard-head">
          <b>{t.terrain}</b>
          {/* Where it is, by the nearest known city: raw coordinates meant nothing to players. */}
          {city ? null : <span className="muted">{placeName(view, hex)}</span>}
        </div>
      )}
      {resource && !t.wonder && !t.landmark && (
        <div className={`resource-line ${isStrategic(resource) ? 'strategic' : 'bonus'}`}>
          <ul className="tilecard-benefits">
            {resourceBenefitLines(resource, view.research.known, view.players.find((p) => p.id === view.me)?.faction).map((line) => <li key={line}><Amounts text={line} /></li>)}
          </ul>
          {resourceStatus && <div className={`tilecard-status ${isDormant(t, view.me, faction) && !isStrategic(resource) ? 'bad' : ''}`} data-testid="tile-improvement"><Amounts text={resourceStatus} /></div>}
        </div>
      )}
      {routeWarning && <div className={isCalmRouteNote(routeWarning) ? 'tilecard-warn route-note' : 'tilecard-warn'} role={isCalmRouteNote(routeWarning) ? undefined : 'alert'} data-testid="route-warning">{routeWarning}</div>}
      {blockedReason && <div className="tilecard-row blocked" role="status" data-testid="tile-blocked-reason"><Amounts text={blockedReason} /></div>}
      {reinforcement && <div className="tilecard-row good" role="status" data-testid="reinforcement-note"><Amounts text={reinforcement} /></div>}
      {refusal && <div className="tilecard-row blocked" role="status" data-testid="tile-refusal"><Icon name="ui-alert" size={13} className="ui-mark" /> <Amounts text={refusal} /></div>}
      {note && <div className="tilecard-row" role="status" data-testid="tile-note"><Amounts text={note} /></div>}
      {t.landmark && <div className="tilecard-row" data-testid="tile-wonder-yield"><Amounts text={isGreatWork(t.landmark.building) ? wonderTileText(t.landmark.building) : 'This landmark follows its tile owner.'} /> The tile gives no Food, Production, Gold or Science while the landmark stands.</div>}
      {(resource || t.wonder) && <div className="tilecard-terrain"><b>{t.terrain}</b></div>}
      <div className="tilecard-yields">
        {YIELD_KEYS.filter((k) => y0[k] > 0).map((k) => (
          <span key={k}>
            <YieldIcon k={k} /> {y0[k]}
          </span>
        ))}
        {YIELD_KEYS.every((k) => y0[k] === 0) && <span className="muted">No yield</span>}
      </div>
      {(adjusted || nile) && (
        <div className="tilecard-row muted" data-testid="tile-base-yields">
          Base {t.terrain}: <Amounts text={YIELD_KEYS.filter((k) => base[k] > 0).map((k) => `${base[k]} ${YIELD_LABEL[k]}`).join(', ') || 'none'} /> · {nile ? 'The Nile: double Food, no Production' : `${faction} terrain rule applied`}
        </div>
      )}
      <div className="tilecard-row">
        {mc === null ? 'Impassable to land units' : t.road ? `Road · move cost 1 (${mc} off road)` : `Move cost ${mc}`}
        {def > 0 && ` · ${tileDefenceText(def)}`}
      </div>
      {owner && (
        <div className="tilecard-row">
          <span className="swatch" style={{ background: playerColor(owner.id) }} />
          <FactionSeal faction={owner.faction} size={14} color={playerInk(owner.id)} /> {owner.id === view.me ? 'Your territory' : empireOfPlayer(view, owner.id)}
        </div>
      )}
      {owningCity && outOfReach && <div className="tilecard-row muted" data-testid="tile-work-status">
        {recipient
          ? `${owningCity.name} owns this tile beyond its ${WORK_RADIUS}-tile work radius. ${recipient.name} will take it over and work it after the next turn.`
          : `Striped: ${owningCity.name} owns this tile beyond its ${WORK_RADIUS}-tile work radius. It gives no yield. It blocks rivals.`}
      </div>}
      {barbarianHeld && <div className="tilecard-row bad">Barbarians hold the tile: it yields nothing.</div>}
      {(camp || units.some((u) => isBarbarian(u.owner))) && faction === 'Celts' && <div className="tilecard-row good">Barbarians never attack you.</div>}
      {city && view.players.find((p) => p.id === view.me)?.faction === 'Celts' && owner?.id !== view.me && city.founder !== view.me && <div className="tilecard-row bad">Celts sack this city when they capture it.</div>}
      {t.ruin && <div className="tilecard-row ruin"><Icon name="ruin" size={15} color="currentColor" /> ancient ruin: unexplored</div>}
      {camp && (
        <div className="tilecard-row barbarian-camp">
          <span className="swatch" style={{ background: BARBARIAN_INK.blood }} /> <b>Barbarian camp</b>
          {!camp.visible && <span className="muted"> · last seen</span>}
          <div className="muted">Clear it with a military unit for <Amount n={CAMP_CLEAR_GOLD} unit="Gold" /> once its guard is gone.</div>
        </div>
      )}
      {city && <div className="tilecard-row">{city.name}{city.visible ? <> · pop {city.population} · strength {city.strength}{city.capture_bounty !== undefined && city.capture_bounty !== null && <> · Capture bounty: ~<Amount n={city.capture_bounty} unit="Gold" /></>}</> : ' · out of sight'}</div>}
      {city?.greece_captive_tech && <div className="tilecard-row">Capture it: +1 free technology</div>}
      {city && (() => {
        const mark = capitalMark(view, city);
        const replaced = replacementTileLine(view, city);
        return mark ? <div className="tilecard-row"><CapitalNote view={view} mark={mark} testId="tile-capital" /></div>
          : replaced && <div className="tilecard-row"><ReplacementNote text={replaced} testId="tile-replacement" /></div>;
      })()}
      {/* Production is private (pillar 6): only your own city's banner meter gets this tooltip line. */}
      {city && city.visible && city.owner === view.me && (() => {
        const p = productionProgress(city);
        if (!p) return null;
        const name = p.item ? itemName(p.item, faction) : null;
        const text = cityOccupiedLine(view, city) ?? (p.state === 'idle' ? 'No production chosen'
          : p.state === 'stalled' ? p.stallText
          : p.state === 'waiting' ? `Queued: ${name} · ${p.stored}/${p.needed} stored`
          : `Producing ${name}: ${p.stored}/${p.needed}${p.turns !== null ? `, ${p.turns} turn${p.turns === 1 ? '' : 's'}` : ''}`);
        return <div className="tilecard-row" data-testid="tile-city-production">{text}</div>;
      })()}
      {fire && <div className={`tilecard-row${fire.fires && view.players.find((p) => p.id === city?.owner)?.team !== view.team ? ' bad' : ''}`} data-testid="tile-city-fire">{fire.text}</div>}
      {lockWarning && <div className="tilecard-warn" role="alert" data-testid="occupation-lock-warning">{lockWarning}</div>}
      {productionWarning && <div className="tilecard-row production-warning"><Amounts text={productionWarning} /></div>}
      {heir && <div className={`tilecard-row${capturable ? ' good' : ''}`} data-testid={capturable ? 'heir-capture' : 'heir-tile'}>
        {capturable ? <Amounts text={`Capture the Heir (+${factionGoalVp('Byzantium')} VP)`} /> : <Amounts text={heirHoverLine(view, heir, factionGoalVp('Byzantium')) ?? heirLines(heir, view)[0].text} />}
      </div>}
      {units.map((u) => (
        <div className="tilecard-row tilecard-unit" key={u.id}>
          {(() => {
            const player = view.players.find((p) => p.id === u.owner);
            const unit = itemName({ kind: 'Unit', unit: u.kind }, player?.faction);
            const name = isBarbarian(u.owner) ? `Barbarian ${unit}` : u.owner === view.me ? `Your ${unit}` : ownedName(player ? FACTION_ADJECTIVE[player.faction as Faction] : 'Rival', unit);
            return <><span className="swatch" style={{ background: playerColor(u.owner) }} />
              {player ? <FactionSeal faction={player.faction} size={14} color={playerInk(u.owner)} /> : <Icon name="ui-cross" size={14} color={faction === 'Celts' ? 'currentColor' : BARBARIAN_INK.blood} />}
              {' '}{u.military && <RankMark level={u.level ?? 0} faction={player?.faction} compact />}{name}{u.military && ` · ${eraName(unitEraTier(u.kind as UnitKind))} era`} · {u.hp} HP · strength {u.strength}</>;
          })()}
          {u.owner === view.me && occupationTileLine(view, u) && <div data-testid="tile-lock"><Icon name="mark-lock" size={12} className="ui-mark" /> {occupationTileLine(view, u)}</div>}
          {u.fortify > 0 && ` · resting defence +${u.fortify}`}
          {u.standing_order === 'Heal' && ` · ${sleepOrderText(u, true).toLowerCase()}`}
          {isCompany(u) && <div className="muted" data-testid="tile-company"><Amounts text={contractLine(u, view)} /></div>}
        </div>
      ))}
      {!t.visible && <div className="tilecard-row muted">Out of sight: owner and city are current; other details may be old.</div>}
    </div>
  );
}

/** The wonder leads its tile: the painted vignette, the bonus in plain words, who holds it, the discovery reward. */
function WonderLead({ view, hex, t }: { view: PlayerView; hex: Hex; t: PlayerView['tiles'][number] }) {
  const wonder = t.wonder!;
  const info = WONDER_INFO[wonder];
  const holder = holdingCity(view, hex);
  const mine = holder ? cityWonders(view, holder).find((w) => w.wonder === wonder) : null;
  const owner = t.owner !== null ? view.players.find((p) => p.id === t.owner) : null;
  const place = placeName(view, hex);
  return (
    <div className="tilecard-wonder-lead" data-testid="natural-wonder">
      <div className="tilecard-lead">
        <img src={wonderArt(wonder, 'a')} width={56} height={56} alt="" />
        <div>
          <b className="tilecard-lead-name">{info.name}</b>
          <span className="tilecard-lead-kind wonder">Natural wonder{!t.visible && ' · last seen'}</span>
          {place && <span className="muted tilecard-lead-place">{place}</span>}
        </div>
      </div>
      <ul className="tilecard-benefits">
        <li data-testid="wonder-bonus">{wonderBonusParts(wonder).lead}<b>{wonderBonusParts(wonder).amount}</b><YieldIcon k={info.yield} />.</li>
        <li data-testid="wonder-owner" className={t.owner === null ? undefined : 'good'}>
          {t.owner === null ? "Unclaimed: grow a city's borders over it to own it."
            : holder ? <>Held by your city {holder.name}{mine?.amount ? <>: <b>+{mine.amount} {wonderYieldWord(wonder)}</b><YieldIcon k={info.yield} /> each turn</> : ''}.</>
            : <>Held by {owner ? empireOfPlayer(view, owner.id) : 'another empire'}.</>}
        </li>
        <li data-testid="wonder-discovery" className="muted">{wonderDiscoveryLine(wonder, !!t.wonder_found)}</li>
      </ul>
    </div>
  );
}
