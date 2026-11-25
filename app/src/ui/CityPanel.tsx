import { bearing, empireOfPlayer } from '../empires';
import { rivalBuilders as rivalsOnWonder, rivalBuilderText } from '../wonderRace';
import { useEffect, useLayoutEffect, useMemo, useRef, useState, type ReactNode } from 'react';
import { purchasePreviews, tilePurchasePreview } from './productionPurchases';
import { useGame } from '../store';
import { AUTOMATION_NO_CHOICE, autoPickReason, cityAutomationFocus, FOCUS_CYCLE, focusLabel, setCityAutomation } from '../automation';
import type { CityBuildFocus } from '../types';
import type { BuildingKind, CityView, Faction, Order, PlayerView, ProductionItem, ProductionOption, Resource, ImprovementKind, UnavailableOption } from '../types';
import { comparisonSentence, pieceAgainst, pieceComparison } from '../pieceCompare';
import { PieceVsLine } from './PieceVs';
import { razeLossLines, razeLosses, razeWorkText } from '../razeLosses';
import { BUILDING_NAME, BUILDING_STATS, TERRAIN_NAME, FIRST_GROWTH, GROWTH_BASE, GROWTH_PER_POP, WORK_RADIUS, projectPercent, costInFactionYield, costYield, isGrow, greatWorkRules, greatWorkRequirement, GREAT_WORKS, isGreatWork, isProductionBuilding, TECH_NAME, CAPTURE_RESISTANCE_TURNS, itemCardText, itemDescription, itemSummary, moreText, eraScaled, itemKey, itemName as baseItemName, IMPROVEMENT_INFO, sameItem, buildingFamily, RULES, HORSE_MOUNTED, wallsUpgradeName } from '../data';
import { effectiveProduction, errorsWhere, orderCity, productionQueue, queueAfterBuild, setProductionQueue, toggleBuy } from '../orders';
import { distance, hexEq, tileIndex } from '../hex';
import { WONDER_INFO, WONDER_PERCENT, cityWonders, wonderArt, wonderBonusSentence, wonderYieldWord } from '../wonders';
import { improvementGain, improvementName, improvementShortTitle, improvementTitle, improvementUpgradeYields, improvementWhere, isDormant } from '../improvements';
import { playerInk, YIELD_COLORS } from '../theme';
import { ItemArt } from './ItemArt';
import { BreakdownTable, Tip, YieldIcon, YieldSourceTable } from './index';
import { YieldsAfter } from './bits';
import { CityYieldHover } from './CityYieldHover';
import { OwnDecadenceNote, decadenceLinePercent, isDecadenceLine } from './DecadenceNote';
import { TILE_PRIORITY_INFO, YIELD_KEYS, YIELD_LABEL, signed } from './bits';
import { CITY_HEAL_RULE, CITY_STRIKE_RULE, cityFireStatus, cityHealStatus, cityStrengthRule, upkeepText } from '../format';
import { Icon, ToneIcon } from './bits';
import { PlayerBadge } from './PlayerBadge';
import { cityOccupiedLine, cityProductionWarning, FORECAST_ASSUMPTIONS, waitingTag } from './forecastWarnings';
import { pendingDecisions } from '../turnFlow';
import { closeTips } from './Tip';
import { CapitalNote, CapitalStarGlyph, KickerDot } from './CapitalStar';
import { capitalLine, capitalMark, replacementKickerLine } from '../capitalMark';
import { hallCardText, hallStands } from './goalText';
import { GOAL_CITY_ICON, TOMB_CITY_NOTE, goalCitiesAt, goalCityLine, tombWorking } from '../goalCities';
import { isPermanentlyUnavailable, nestBuildingUpgrades, progressText, unavailableText, unitLines, withUnavailable, type BuildingLineNode } from './productionChoices';
import { feel } from '../feel';
import { answerDecision } from '../notice';
import { RESOURCE_INFO, isStrategic, needText, nileYields, supplyPerMine, supplyRuleText, withArticle } from '../resources';
import { CostChip } from './CostChip';
import { Amount, Amounts, Quantity } from './Amounts';
import { ResourceReason, openResourceChip } from './ResourceReason';
import type { AmountUnit } from './amountParts';
import { ForeignCityActions } from './ForeignCityActions';
import { CityBorders } from './CityBorders';
import { ProgressBar } from './ProgressBar';
import { claimSummary, growthProgress, territoryCounts, yieldsText } from '../cityProgress';
import { revealBelowStickyHead, revealIdleProductionPicker } from './cityPanelScroll';
import { LevyNote } from './Levy';
import { CompanyShop } from './CompanyShop';
import { canHire } from '../company';
import { WhereToBuild, WhereToBuildLine } from './WhereToBuild';
import { TILE_RELEASE_RULE } from './cityTerritoryText';

const lowerFirst = (text: string) => text.charAt(0).toLowerCase() + text.slice(1);
const upperFirst = (text: string) => text.charAt(0).toUpperCase() + text.slice(1);

/** What each automatic-build focus does, for the select's tooltip. */
const FOCUS_TIP: Record<CityBuildFocus, string> = {
  Military: 'Trains the newest military units your strategic resources allow, never outdated ones. Spends no Gold.',
  Economy: 'Builds buildings and improvements, never units. Spends no Gold.',
  Gold: 'Runs the Gold project and builds nothing else.',
  Science: 'Runs the Science project and builds nothing else.',
  Food: 'Runs the Food project and builds nothing else.',
};

/** How Build and + Queue work, and what happens to progress when you switch (one rule, one wording). */
const BUILD_RULE = 'Build replaces the current item. Its progress stays banked with it in this city and resumes when you build it again; it never moves to another item. Production not yet assigned (overflow, or stored while idle) joins whatever starts next. + Queue or Shift-click adds to the queue. Greyed-out items can\'t be built here yet: hover one for why.';
const EGYPT_BUILD_RULE = 'Build replaces the current item. Its progress stays banked with it in this city and resumes when you build it again. Surplus Food builds ordinary buildings; their progress never moves to another item and excess progress is lost. Unassigned Production joins units, Settlers, improvements and wonders, but never an ordinary building. + Queue or Shift-click adds to the queue. Greyed-out items show why they cannot start yet.';

/**
 * Why reaching this population unlocks the item (legatum-ui: never let a change happen
 * unannounced, no hidden math). The Settler is the one item that costs population directly;
 * anything else gated by population (upgrade buildings) just needs a city of that size.
 */
const unlockReason = (item: ProductionItem, name: string, population: number): string =>
  item.kind === 'Unit' && item.unit === 'Settler'
    ? `Settlers cost 1 population, so a city must reach size ${population} before it can build one.`
    : `${name} needs a city of population ${population} or more to build.`;

/** Your own city's capture bounty shows from this many Gold (playtest-6 H4: "~1 Gold" was noise). */
const OWN_BOUNTY_SHOWN = 25;

export function CityPanel({ city, view, peek = false }: { city: CityView; view: PlayerView; peek?: boolean }) {
  const [razeConfirm, setRazeConfirm] = useState(false);
  useEffect(() => {
    if (!razeConfirm) return;
    const close = (event: KeyboardEvent) => { if (event.key === 'Escape') { event.preventDefault(); setRazeConfirm(false); } };
    window.addEventListener('keydown', close);
    return () => window.removeEventListener('keydown', close);
  }, [razeConfirm]);
  const [phoneChoice, setPhoneChoice] = useState('');
  const wonderWhere = (tile: CityView['pos']) => { const n = distance(city.pos, tile); return `${n} tile${n === 1 ? '' : 's'} ${bearing(city.pos, tile)}`; };
  /** An improvement names its tile ("Mine — Iron, 2 tiles NE"); every other item keeps its own name. */
  const itemName = (item: ProductionItem, faction?: Faction): string => item.kind === 'Improvement'
    ? improvementTitle(item, city.pos, view.tiles[tileIndex(item.tile, view.width, view.height)]?.resource)
    : item.kind === 'Wonder'
      ? `${baseItemName(item, faction)} · ${wonderWhere(item.tile)}`
    : baseItemName(item, faction);
  /** Upgrade Walls states this city's own gain: +5 city strength scaled to the owner's era (core `data::era_scaled`). */
  const wallsGain = (): number => eraScaled(BUILDING_STATS.UpgradeWalls.cityStrength, (view.players.find((p) => p.id === city.owner)?.era ?? 1) - 1);
  const isUpgradeWalls = (item: ProductionItem): boolean => item.kind === 'Building' && item.building === 'UpgradeWalls';
  const summaryOf = (item: ProductionItem, faction?: Faction): string => isUpgradeWalls(item) ? `+${wallsGain()} city strength for the next level` : itemSummary(item, faction);
  const descriptionOf = (item: ProductionItem, faction?: Faction): string => isUpgradeWalls(item)
    ? `The next level adds ${wallsGain()} city strength. You can add one level for each era you have reached.`
    : itemDescription(item, faction);
  /** The production meter's label is one line, so an improvement or a placed site uses the short form ("Quarry, Stone NE", "Yam Relay Station W"). */
  const meterName = (item: ProductionItem): string => item.kind === 'Improvement'
    ? improvementShortTitle(item, city.pos, view.tiles[tileIndex(item.tile, view.width, view.height)]?.resource)
    : item.kind === 'Wonder' ? `${baseItemName(item, owner?.faction)} ${bearing(city.pos, item.tile)}`
    : item.kind === 'Building' && item.building === 'UpgradeWalls' ? wallsUpgradeName(city.walls_level ?? Number(city.walls), itemName({ kind: 'Building', building: 'Walls' }, owner?.faction))
    : itemName(item, owner?.faction);
  const improve = useGame((s) => s.improve);
  const place = useGame((s) => s.place);
  useEffect(() => {
    if (place?.city !== city.id) return;
    const cancel = (event: KeyboardEvent) => {
      if (event.key !== 'Escape' || event.defaultPrevented) return;
      event.preventDefault();
      event.stopImmediatePropagation();
      useGame.getState().set({ place: null });
    };
    window.addEventListener('keydown', cancel, true);
    return () => window.removeEventListener('keydown', cancel, true);
  }, [place?.city, city.id]);
  const applyPlacePick = useRef<() => void>(() => {});
  useEffect(() => applyPlacePick.current(), [place?.pick]);
  const setImproveHover = (hover: CityView['pos'] | null) => { const s = useGame.getState(); if (s.improve.hover?.q !== hover?.q || s.improve.hover?.r !== hover?.r) s.set({ improve: { ...s.improve, hover } }); };
  useEffect(() => () => { const s = useGame.getState(); if (s.improve.hover || s.improve.pick) s.set({ improve: { hover: null, pick: null } }); }, []);
  /** Why the card just clicked can't be built. A queue confirmation never inserts a line: it shows on the Queue button itself. */
  const [notice, setNotice] = useState<{ city: number; key: string; text: string; tone: 'good' | 'bad' } | null>(null);
  const orders = useGame((s) => s.orders[s.seat]);
  const buildFocus = cityAutomationFocus(city.id, view, orders);
  const pendingAuto = orders.findLast((o) => o.type === 'SetCityAutomation' && o.city === city.id);
  const pendingBuild = orders.some((o) => (o.type === 'SetProduction' || o.type === 'SetProductionQueue') && o.city === city.id);
  const autoPick = pendingAuto?.type === 'SetCityAutomation' && pendingAuto.picked ? pendingAuto.picked
    : !pendingBuild && buildFocus === city.detail?.build_automation ? city.detail?.last_auto_pick : null;
  const errors = useGame((s) => s.errors[s.seat]);
  const client = useGame((s) => s.client);
  const editOrders = useGame((s) => s.editOrders);
  // Hovering the territory bar highlights the tile its next growth claims.
  const territory = useGame((s) => s.territory);
  const setTerritory = (patch: Partial<typeof territory>) => { const s = useGame.getState(); s.set({ territory: { ...s.territory, ...patch } }); };
  const setTerritoryOpen = (open: boolean) => setTerritory({ open, claimHover: territory.claimHover });
  const setClaimHover = (claimHover: boolean) => setTerritory({ claimHover });
  useEffect(() => () => { const s = useGame.getState(); if (s.territory.open || s.territory.claimHover) s.set({ territory: { open: false, claimHover: false } }); }, []);
  const owner = view.players.find((p) => p.id === city.owner);
  const fire = city.visible ? cityFireStatus(view, city) : null;
  const heal = city.visible ? cityHealStatus(city) : null;
  // Your own city's bounty is what a captor would take: noise until it is worth defending over.
  const bounty = !city.visible || city.capture_bounty === undefined || city.capture_bounty === null
    || (city.owner === view.me && city.capture_bounty < OWN_BOUNTY_SHOWN) ? null : city.capture_bounty;
  const col = playerInk(city.owner);
  const d = city.detail;
  const purchases = useMemo(() => purchasePreviews(client, city, view, orders, errors), [client, city, view, orders, errors]);
  const errs = errorsWhere(orders, errors, (o: Order) => orderCity(o) === city.id);
  const panel = useRef<HTMLDivElement>(null);
  const placementReturnScroll = useRef<number | null>(null);
  useLayoutEffect(() => {
    if (place || placementReturnScroll.current === null || !panel.current) return;
    panel.current.scrollTop = placementReturnScroll.current;
    placementReturnScroll.current = null;
  }, [place]);
  /** The Queue button just pressed says "Queued #2" for a moment, in place (no layout change). */
  const [flash, setFlash] = useState<{ city: number; key: string; label: string } | null>(null);
  const flashTimer = useRef<ReturnType<typeof setTimeout> | undefined>(undefined);
  useEffect(() => () => clearTimeout(flashTimer.current), []);
  // Queueing grows the queue list above the build cards. Whatever the pointer was on keeps its place: the panel
  // scrolls by exactly how far that card moved (or, if the card left the list, the card after it), so a second
  // click in a row lands where the first one did.
  const anchor = useRef<{ el: HTMLElement; next: HTMLElement | null; top: number; nextTop: number; at: number } | null>(null);
  const keepInPlace = (from: Element | null | undefined) => {
    const el = from?.closest<HTMLElement>('.production-choice, .wtb-row');
    if (!el || !panel.current) { anchor.current = null; return; }
    const cards = [...panel.current.querySelectorAll<HTMLElement>('.production-choice, .wtb-row')];
    const next = cards[cards.indexOf(el) + 1] ?? null;
    anchor.current = { el, next, top: el.getBoundingClientRect().top, nextTop: next?.getBoundingClientRect().top ?? 0, at: Date.now() };
  };
  useLayoutEffect(() => {
    const a = anchor.current;
    if (!a) return;
    anchor.current = null;
    const scroller = panel.current;
    if (!scroller || Date.now() - a.at > 250) return;
    const moved = a.el.isConnected ? a.el.getBoundingClientRect().top - a.top : a.next?.isConnected ? a.next.getBoundingClientRect().top - a.nextTop : 0;
    if (Math.abs(moved) >= 1) scroller.scrollTop += moved;
  });
  // The sticky head's height, published for the sticky "where to build" heads that stack under it.
  useLayoutEffect(() => {
    const scroller = panel.current, head = scroller?.querySelector<HTMLElement>('.cp-head');
    if (!scroller || !head) return;
    const publish = () => scroller.style.setProperty('--cp-head-h', `${head.offsetHeight}px`);
    publish();
    if (typeof ResizeObserver === 'undefined') return;
    const observer = new ResizeObserver(publish);
    observer.observe(head);
    return () => observer.disconnect();
  }, [city.id]);
  // An idle city with nothing queued: choosing is the next decision, so show the picker heading and as many
  // cards as fit below the sticky head. Leave taller panels at their current position when three cards fit.
  useLayoutEffect(() => {
    const scroller = panel.current, dd = city.detail;
    if (city.owner !== view.me || !scroller || !dd || dd.producing || dd.production_queue?.length) return;
    if (orders.some((o) => (o.type === 'SetProduction' || o.type === 'SetProductionQueue') && o.city === city.id)) return;
    revealIdleProductionPicker(scroller);
  }, [city.id, city.owner, view.me]);
  // A production warning that appears while the panel is scrolled to the build list (Ready with an automated
  // city that has nothing to pick) would sit half under the sticky head: scroll it whole into view once.
  const shownWarning = useRef<string | null>(null);
  useLayoutEffect(() => {
    const scroller = panel.current, warning = scroller?.querySelector<HTMLElement>('.production-warning') ?? null;
    const key = warning ? `${city.id}:${warning.textContent}` : null;
    if (key === shownWarning.current) return;
    shownWarning.current = key;
    if (scroller && warning) revealBelowStickyHead(scroller, warning);
  });
  // The star marks the original capital for the whole game (the scoring rule), not the current capital. In the common case, your own
  // founding city, the line is a tooltip so it costs the build list no room; any other original capital prints it under the head.
  const capMark = capitalMark(view, city);
  const capKicker = (placement: 'right' | 'bottom') => capMark
    ? <Tip content={capitalLine(view, capMark)} placement={placement}><CapitalStarGlyph mark={capMark} size={11} /></Tip> : <KickerDot />;
  // A replacement capital says what it is on hover: it is the current capital, but capturing it scores nothing.
  const replacedLine = replacementKickerLine(view, city);
  const kickerLabel = city.owner === view.me ? <>Your {city.visible && city.is_capital ? 'capital' : 'city'} <PlayerBadge id={city.owner} /></>
    : <>{city.visible && city.is_capital ? 'Capital' : 'City'} of <PlayerBadge id={city.owner} /> {owner ? empireOfPlayer(view, owner.id) : 'the unknown'}</>;
  // The sticky head: name, three numbers and, for a city you can see into, its yields row, so scrolling to the build
  // cards never hides what the city makes (the 1000x600 laptop opened an idle city with the yields scrolled away).
  const headWith = (yieldsRow?: ReactNode) => (
    <div className="cp-head">
      <div className="cp-kicker">
        {capKicker('right')}
        {replacedLine ? <Tip content={replacedLine} placement="right"><span data-testid="city-replacement">{kickerLabel}</span></Tip> : kickerLabel}
        {capKicker('bottom')}
      </div>
      <div className="cp-name">{city.name}</div>
      {city.visible && <div className="cp-meta">
        <span className="cp-population">
          <span className="cp-pop">{city.population}</span> population
        </span>
        <Tip content={cityStrengthRule(city, owner?.faction, owner ? owner.era - 1 : 0)}>
          <span className="lbl">Strength</span>
          <b>{city.strength}</b>
        </Tip>
        <Tip content={`City HP. ${CITY_HEAL_RULE}`}>
          <span className="lbl">HP</span>
          <b>{city.hp}</b>
          <span className="muted"> / {city.max_hp}</span>
        </Tip>
      </div>}
      {yieldsRow}
    </div>
  );
  const head = headWith();

  // Egypt's Royal Tomb work: none of its cities builds or grows (core `economy`), and every panel says so.
  const tombStopped = city.owner === view.me && tombWorking(view);
  // Healing, fire and bounty scroll with the list instead of riding in the sticky head: the head stays a name and three numbers, so the build cards keep the room.
  const status = (
    <>
      {capMark && !(capMark.own && city.owner === view.me) && <div className="cp-capital"><CapitalNote view={view} mark={capMark} testId="city-capital" /></div>}
      {/* Rome's Colonia and Egypt's tomb: public goal cities, named with whose goal it is and who scores now. */}
      {goalCitiesAt(view, city.id).map((g) => <div key={g.kind} className="cp-capital cp-goal-city" data-testid={`city-goal-${g.kind}`}>
        <Icon name={GOAL_CITY_ICON[g.kind]} size={16} /><span>{goalCityLine(view, g)}</span>
      </div>)}
      {tombStopped && <div className="cp-capital cp-goal-city cp-tomb-work" data-testid="city-tomb-work"><Icon name="goal-royal-tomb" size={16} /><span>{TOMB_CITY_NOTE}</span></div>}
      {(fire || heal || bounty !== null || city.greece_captive_at_risk) && <div className="cp-status">
        {heal && <Tip as="div" className="cp-healing-wrap" content={CITY_HEAL_RULE}>
          <ProgressBar className="cp-healing" testId="city-heal" name="City HP healing"
            label={heal.blocked ? 'Healing' : `Healing · ${heal.text.replace(/^Heals /, '')}`} color="var(--good)" tone={heal.blocked ? 'paused' : 'good'}
            value={city.hp} max={city.max_hp} turns={heal.blocked ? heal.text : undefined} />
        </Tip>}
        {fire && <Tip content={CITY_STRIKE_RULE}>
          <span className={`cp-fire ${fire.fires ? 'fires' : 'quiet'}`} data-testid="city-fire">{fire.text}</span>
        </Tip>}
        {bounty !== null && <Tip content="A Switzerland city pays its captor treasury × this city's share of the empire's population, doubled for a capital, at most the whole treasury.">
          {/* From the viewer's side: a prize to take from a rival, a loss to defend at home. */}
          <span className={`cp-bounty ${owner?.team === view.team ? 'risk' : 'gain'}`}>
            <span aria-hidden="true"><ToneIcon tone={owner?.team === view.team ? 'bad' : 'good'} /> </span>Capture bounty ~<Amount n={bounty} unit="Gold" />
          </span>
        </Tip>}
        {city.greece_captive_at_risk && <span className="cp-captive-risk">If a rival captures this city: +1 free technology for them</span>}
      </div>}
    </>
  );

  if (!d) {
    const teammate = owner?.team === view.team;
    if (peek) return teammate ? <p className="phone-selection-note">Managed by {owner?.name}</p>
      : <ForeignCityActions city={city} view={view} compact />;
    return (
      <div className="panel city-panel" style={{ ['--pc' as string]: col }}>
        {head}
        {status}
        {(teammate || !city.visible) && <div className="hint">
          {teammate ? (
            <>
              Managed by {owner?.name}.{' '}
            </>
          ) : (
            'City and owner are current; other details are out of sight.'
          )}
        </div>}
        {!teammate && <ForeignCityActions city={city} view={view} />}
      </div>
    );
  }

  const queue = productionQueue(orders, city.id, d.producing, d.production_queue);
  const razeOrdered = orders.some((o) => o.type === 'Raze' && o.city === city.id);
  const razeGold = city.population * 10;
  const losses = razeLosses(view, city);
  const lossLines = razeLossLines(losses);
  const razeWorks = [...losses.works, ...losses.goals].map(razeWorkText);
  // One compact block: the confirm replaces the idle row in place, so a captured city's build list sits no lower than it must.
  const razeControl = d.captured && city.owner === view.me && <section className="cp-block cp-raze" data-testid="city-raze">
    <div className="cp-sub">Raze</div>
    {d.raze_blocker && <p role="note" className="prod-blocked">{d.raze_blocker}</p>}
    {razeConfirm && !razeOrdered ? <div role="alertdialog" aria-label={`Raze ${city.name}?`} aria-modal="true" data-testid="raze-confirm">
      <p>Raze {city.name}? It destroys:</p>
      <ul className="raze-loses">
        {lossLines.length ? lossLines.map((line, i) => <li key={line} className={i < razeWorks.length ? 'raze-vp' : undefined}>{line}</li>) : <li>the city</li>}
      </ul>
      <p>Its tiles are released and you gain {razeGold} Gold.</p>
      <div className="raze-buttons">
        <button className="btn" onClick={() => setRazeConfirm(false)}>Cancel</button>
        <button className="btn danger confirm" autoFocus onClick={() => {
          editOrders((list) => [...list.filter((o) => orderCity(o) !== city.id), { type: 'Raze', city: city.id }], 'commit');
          setRazeConfirm(false);
        }}>Confirm Raze</button>
      </div>
    </div> : <div className="raze-row">
      <p>Burns the city{razeWorks.length ? ` and destroys ${razeWorks.join(' and ')}` : ''}: its tiles are released; you gain {razeGold} Gold.</p>
      {razeOrdered ? <button className="btn" onClick={() => editOrders((list) => list.filter((o) => !(o.type === 'Raze' && o.city === city.id)))}>Raze ordered · Undo</button>
        : <button className="btn danger" disabled={view.finished || !!d.raze_blocker} onClick={() => setRazeConfirm(true)}>Raze city</button>}
    </div>}
  </section>;
  const pendingQueue = orders.some((o) => (o.type === 'SetProductionQueue' || o.type === 'SetProduction') && o.city === city.id);
  const producing = effectiveProduction(orders, city.id, d.producing);
  const waitingQueue = !producing && queue.length > 0;
  /** A rival completed the wonder this city was building: the line names that, not the queue's next item. */
  const raceLost = !producing && !pendingQueue ? d.wonder_lost ?? null : null;
  /** A waiting queue banks Production only up to its first item's cost (core `bank_cap`). */
  const waitingCap = waitingQueue && !raceLost ? d.options.find((o) => sameItem(o.item, queue[0]))?.cost ?? d.idle_production_cap : null;
  const buildChanged = pendingQueue && !(sameItem(producing, d.producing) || (!producing && !d.producing));
  const pendingItem = buildChanged ? producing : null;
  const option = producing ? d.options.find((o) => sameItem(o.item, producing)) : null;
  const yields = buildChanged ? producing ? option?.yields ?? d.yields : d.idle_yields ?? d.yields : d.yields;
  const surplus = yields.total.food - 2 * city.population;
  // The Nile (core `economy::nile`): while an Egyptian city builds anything but Grow, its surplus Food is already
  // converted in the yields, so the Production total is the progress from Food and the converted Food is on its own line.
  const egyptBuilding = owner?.faction === 'Egypt' && !!producing && !isGrow(producing);
  const egyptProgress = Math.max(0, yields.total.production);
  /** A project's per-turn result: Germany's Gold or Science, or how soon Grow grows the city. */
  const projectRate = (item: ProductionItem & { kind: 'Project' }, y: typeof yields): string => {
    if (item.project === 'Grow') {
      const gain = y.total.food - 2 * city.population;
      if (gain <= 0) return 'No surplus Food';
      const turns = Math.max(1, Math.ceil((d.growth_threshold - d.food_stored) / gain));
      return `Grows in ${turns} turn${turns === 1 ? '' : 's'}`;
    }
    return `+${Math.floor(Math.max(0, y.total.production) * projectPercent(owner?.faction, item.project) / 100)} ${item.project}/turn`;
  };
  const growOption = owner?.faction === 'Egypt' ? d.options.find((o) => isGrow(o.item)) ?? null : null;
  const nileSurplus = egyptBuilding ? -(yields.sources.find((src) => src.label.startsWith('The Nile: surplus Food builds'))?.yields.food ?? 0) : 0;
  const turnsToGrow = buildChanged
    ? d.resistance > 0 || egyptBuilding ? null
      : d.food_stored + surplus >= d.growth_threshold ? 1
      : surplus > 0 ? Math.max(1, Math.ceil((d.growth_threshold - d.food_stored) / surplus)) : null
    : d.turns_to_grow;
  const storedProduction = pendingItem ? option?.production_stored ?? 0 : d.production_stored;
  const productionWarning = cityProductionWarning(city, orders, owner?.faction, view);
  // Occupation lock: a frozen city's build list is replaced by this note.
  const occupiedLine = cityOccupiedLine(view, city);
  const growth = growthProgress(city, buildChanged ? { turnsToGrow, surplus } : undefined);
  const unlockItems = growth?.unlocks ?? [];
  const unlockNames = unlockItems.map((item) => itemName(item, owner?.faction)).join(', ');
  // One rule, one wording (legatum-ui): "Grows to N · unlocks X" everywhere this is shown
  // (the turn report and the growth celebration moment use the same "unlocks X" phrase).
  // Population is territory: the growth bar is the territory bar, and each growth claims a tile.
  const claim = claimSummary(city, view);
  // The rule behind the claim lives in this phrase's tooltip: ring order, then the focus (an auto focus).
  const focusReason = yields.focus_reason.split(';')[0];
  const claimTip = claim ? <Tip content={<span data-testid="claim-rule">
    <Amounts text={`Ring ${claim.ring}. ${claim.worked ? `Worked as soon as it is claimed: ${claim.effect}.` : claim.handoffTo ? `Beyond ${WORK_RADIUS} tiles from this city: ${claim.handoffTo} takes over and works it at the end of the turn.` : `Beyond ${WORK_RADIUS} tiles from the city: claimed but not worked. It keeps rivals out.`} `} />
    {`Claims the nearest ring first, then the best tile for your focus (${yields.focus}${focusReason.startsWith('building') ? `, from ${focusReason}` : `, ${focusReason}`}). `}
    {`${yields.focus} is this city's focus, and you never set it by hand. It follows what the city builds: military units and world wonders favour Production, a Settler at population 1 favours Food. ${TILE_PRIORITY_INFO[yields.focus]}`}
    {pendingQueue && ' Preview after Ready.'}
  </span>}>{claim.text}</Tip> : null;
  // Under the bar: what the claim adds, as icons with counts (or that it only blocks rivals).
  const claimFoot = growth?.state === 'starving' || !claim ? undefined : claim.worked
    ? <><span className="cp-next-tile">Next tile</span>{YIELD_KEYS.filter((k) => claim.yields[k] !== 0).map((k) => <span key={k} className="cp-claim-yield"><Quantity plus value={claim.yields[k]} unit={YIELD_LABEL[k] as AmountUnit} /></span>)}
        {YIELD_KEYS.every((k) => claim.yields[k] === 0) && <span className="muted">no yield</span>}</>
    : <span className="muted">Next tile · {claim.effect}</span>;
  const growthLabel = growth?.state === 'starving' ? 'Starving'
    : <span className="cp-grow-label">Grows to {city.population + 1}{claimTip && <> · {claimTip}</>}{unlockItems.length > 0 && <> · <Tip content={unlockItems.map((item) => unlockReason(item, itemName(item, owner?.faction), city.population + 1)).join(' ')}>unlocks {unlockNames}</Tip></>}</span>;
  const growthLabelText = growth?.state === 'starving' ? 'Starving' : `Grows to ${city.population + 1}${claim ? ` · ${claim.text}` : ''}${unlockNames ? ` · unlocks ${unlockNames}` : ''}`;
  const counts = territoryCounts(d);
  const ownedWonders = cityWonders(view, city);
  const growthTurns = tombStopped && d.resistance === 0 ? 'Paused'
    : egyptBuilding && d.resistance === 0 ? 'Food builds instead'
    : growth?.state === 'resisting' ? 'Paused'
    : growth?.state === 'starving' ? `−1 population in ${growth.turns}`
    : growth?.state === 'growing' ? `${growth.turns} turn${growth.turns === 1 ? '' : 's'}`
    : growth?.state === 'shortage' ? 'Short of Food' : 'Not growing';
  const growthDecadence = d.growth_breakdown?.parts.find(([label]) => isDecadenceLine(label));
  const growthTip = (
    <div className="bd">
      <div className="bd-title">Growth</div>
      <div className="bd-row"><span>Food per turn</span><span>{yields.total.food + nileSurplus}</span></div>
      <div className="bd-row"><span>Eaten (2 per population)</span><span className="neg">−{2 * city.population}</span></div>
      <div className="bd-row bd-total"><span>Surplus</span><span>{surplus + nileSurplus}</span></div>
      {egyptBuilding && <div className="bd-row"><span>Goes into the {producing ? itemName(producing, owner?.faction) : 'build'}</span><span>−{nileSurplus} Food, +{egyptProgress} progress</span></div>}
      {egyptBuilding && <div className="bd-row bd-total"><span>Left to grow</span><span>{surplus}</span></div>}
      <div className="bd-row"><span>Stored</span><span><Amount n={`${d.food_stored}/${d.growth_threshold}`} unit="Food" /></span></div>
      <div className="bd-note"><Amounts text={
        growth?.state === 'starving' ? `Stored Food runs out and ${city.name} loses 1 population in ${growth.turns} turn${growth.turns === 1 ? '' : 's'}. Build a Farm or a Granary to add Food.`
          : growth?.state === 'resisting' ? `No growth during resistance (${d.resistance} turns).`
          : tombStopped ? TOMB_CITY_NOTE
          : egyptBuilding ? `While ${city.name} is building, its surplus Food goes into the build and the city doesn't grow. Choose Grow to grow it instead.`
          : `Grows at ${d.growth_threshold} Food. The first growth costs ${FIRST_GROWTH} Food. After that, each growth costs ${GROWTH_BASE} Food plus ${GROWTH_PER_POP} for every population above 1. Leftover Food carries over, and a city grows at most 1 population a turn. Each growth claims one more tile.`
      } /></div>
      {unlockItems.map((item) => <div className="bd-note" key={itemKey(item)}>{unlockReason(item, itemName(item, owner?.faction), city.population + 1)}</div>)}
      {d.growth_breakdown && <BreakdownTable b={d.growth_breakdown} title="Growth threshold" unit="Food" />}
      {growthDecadence && <OwnDecadenceNote fallback={decadenceLinePercent(growthDecadence[0])} />}
    </div>
  );
  const faction = owner?.faction;
  // The Mongols's Yam Relay Station in its own city: a greyed card too, so the Station reads as built where it is bought.
  const hallHere = city.owner === view.me && faction === 'Mongols' && d.buildings.includes('YamRelayStation');
  const unavailable: UnavailableOption[] = [...(d.unavailable ?? []), ...(hallHere && !(d.unavailable ?? []).some((u) => sameItem(u.item, { kind: 'Building', building: 'YamRelayStation' })) ? [{ item: { kind: 'Building' as const, building: 'YamRelayStation' as const }, cost: 0, reason: `The Yam Relay Station already stands in ${city.name}`, production_banked: 0 }] : [])];
  const listOptions = d.options;
  const listUnavailable = unavailable;
  const permanentLine = (items: UnavailableOption[], options: ProductionOption[] = []) => {
    const excluded = items.filter((u) => isPermanentlyUnavailable(u, faction));
    const hasAztecHorse = faction === 'Aztecs' && [...items.map((u) => u.item), ...options.map((o) => o.item), ...d.options.map((o) => o.item)].some((item) => item.kind === 'Unit' && HORSE_MOUNTED.includes(item.unit));
    if ((!excluded.length && !hasAztecHorse) || !faction) return null;
    if (hasAztecHorse || (faction === 'Aztecs' && excluded.some((u) => u.item.kind === 'Unit' && HORSE_MOUNTED.includes(u.item.unit))))
      return 'Aztecs train no horse-mounted units (Early Tanks, Tanks and later armour allowed)';
    const rules = [...new Set(excluded.map((u) => u.reason))];
    return `Unavailable by faction rule: ${rules.join('; ')}`;
  };
  const banks = d.production_banked ?? [];
  const costOf = (item: ProductionItem) => d.options.find((o) => sameItem(o.item, item))?.cost ?? unavailable.find((u) => sameItem(u.item, item))?.cost ?? null;
  const bankOf = (item: ProductionItem) => banks.find((b) => sameItem(b.item, item))?.progress ?? 0;
  const whyUnavailable = (u: UnavailableOption) => unavailableText(u, city.name, city.population, growth, itemName(u.item, faction));
  const changeQueue = (items: ProductionItem[]) => editOrders((o) => setProductionQueue(o, city.id, items), 'queue');
  const append = (item: ProductionItem, from?: Element | null) => {
    const next = [...queue, item];
    keepInPlace(from);
    changeQueue(next);
    const count = next.filter((queued) => sameItem(queued, item)).length;
    setNotice(null);
    setFlash({ city: city.id, key: itemKey(item), label: count > 1 ? `Queued ×${count}` : `Queued #${next.length}` });
    clearTimeout(flashTimer.current);
    flashTimer.current = setTimeout(() => setFlash(null), 1500);
  };
  /** True when it replaced the current build: that shows the G17 notice, which must not be hidden by the next decision. */
  const buildNow = (item: ProductionItem, from?: Element | null): boolean => {
    setNotice(null);
    keepInPlace(from);
    const next = queueAfterBuild(queue, producing, item);
    // Misclick guard: replacing the current build names both, with Undo (G17).
    if (producing && !sameItem(producing, item)) {
      answerDecision(`${city.name} now builds ${itemName(item, faction)} instead of ${itemName(producing, faction)}.`, (o) => setProductionQueue(o, city.id, next), undefined, 'queue');
      return true;
    }
    changeQueue(next);
    return false;
  };
  /** Buying names the city, with Undo (G17): a buy in the wrong city is easy to miss before End Turn. */
  const buy = (item: ProductionItem, price: number | null) => answerDecision(
    `Buying ${itemName(item, faction)} in ${city.name}${price === null ? '' : ` for ${price} Gold`}.`, (ord) => toggleBuy(ord, city.id, item), undefined, 'purchase');
  /** A greyed-out item never adds an order; clicking it says why, on the card (never a silent bounce). */
  const refuse = (u: UnavailableOption) => {
    const why = whyUnavailable(u);
    feel.invalid();
    setNotice({ city: city.id, key: itemKey(u.item), tone: 'bad', text: `Can't build ${itemName(u.item, faction)} yet: ${lowerFirst(why.short)}.${why.growth ? ` ${why.growth}.` : ''}` });
  };
  // Switching banks the old item's progress with it; the new item starts from its own bank.
  const oldItem = d.producing;
  // Said only when progress moves somewhere: an old build's banks, or a new one resuming or carrying some.
  const switchNote = buildChanged && ((oldItem && oldItem.kind !== 'Project' && d.production_stored > 0) || (option?.production_stored ?? 0) > 0) ? upperFirst([
    oldItem && oldItem.kind !== 'Project' && d.production_stored > 0 ? `${city.name} keeps the ${d.production_stored} ${faction === 'Egypt' && oldItem.kind === 'Building' ? 'Food-funded progress' : 'Production'} already put into the ${itemName(oldItem, faction)}` : null,
    !producing ? 'choose a new build' : producing.kind !== 'Project' && option ? `${itemName(producing, faction)} ${option.production_banked > 0 ? 'resumes' : 'starts'} at ${option.production_stored}/${option.cost}` : null,
  ].filter(Boolean).join('; ')) : '';
  // No build because the city can't make what it banked (e.g. a Settler after the city shrank to 1).
  const parked = !producing && !waitingQueue ? banks.find((b) => unavailable.some((u) => sameItem(u.item, b.item))) : undefined;
  const parkedText = (() => {
    if (!parked) return null;
    const u = unavailable.find((x) => sameItem(x.item, parked.item))!;
    const why = whyUnavailable(u);
    const until = why.growth?.startsWith(`${city.name} grows to`) ? why.growth : `${city.name} can build it again (${lowerFirst(why.short)}${why.growth ? `; ${why.growth}` : ''})`;
    return `${city.name} keeps the ${parked.progress} ${faction === 'Egypt' && parked.item.kind === 'Building' ? 'Food-funded progress' : 'Production'} already put into the ${itemName(parked.item, faction)} until ${until}.`;
  })();
  // The Banked line skips what another line already states (the resumed build, the parked item).
  const shownBanks = banks.filter((b) => !(buildChanged && sameItem(b.item, producing)) && b !== parked);
  /** Build completes this decision; the Cities list keeps this panel reachable. */
  const advanceAfterBuild = () => {
    const s = useGame.getState();
    const pending = pendingDecisions(view, s.orders[s.seat], s.errors[s.seat], s);
    if (pending.some((decision) => decision.kind === 'city' && decision.id === city.id)) return;
    closeTips();
    document.querySelector<HTMLElement>('[data-game-map]')?.focus({ preventScroll: true });
    s.nextDecision();
    if (!pending.length) s.select(null);
  };
  // Tile-chosen builds (the Yam Relay Station, world wonders) are placed the way Civ places a district:
  // one card per building, whose Build, Queue or Buy lights the eligible tiles on the map to click.
  const siteOptions = (building: BuildingKind) => d.options.filter((o) => o.item.kind === 'Wonder' && o.item.building === building);
  const siteChosen = (o: ProductionOption) => queue.some((item) => sameItem(item, o.item)) || (orders ?? []).some((x) => x.type === 'BuyItem' && x.city === city.id && sameItem(x.item, o.item));
  /** One building's card: the tile already in the queue, else any eligible tile, standing for them all. */
  const siteCard = (building: BuildingKind) => { const all = siteOptions(building); return all.find(siteChosen) ?? all[0]; };
  /** "Plains, Wheat, 3 tiles SW": the chosen tile, as the card names it. */
  const sitePlace = (tile: CityView['pos']) => { const t = view.tiles[tileIndex(tile, view.width, view.height)]; return [t?.terrain ? TERRAIN_NAME[t.terrain] : null, t?.resource ? RESOURCE_INFO[t.resource].name : null, wonderWhere(tile)].filter(Boolean).join(', '); };
  const yamCard = listOptions.some((o) => o.item.kind === 'Wonder' && o.item.building === 'YamRelayStation') ? siteCard('YamRelayStation') : undefined;
  const placing = (building: BuildingKind) => place?.city === city.id && place.building === building;
  const startPlace = (building: BuildingKind, intent: 'build' | 'queue' | 'buy') => {
    setImproveHover(null);
    const s = useGame.getState();
    const cancel = placing(building) && place!.intent === intent;
    if (!cancel) placementReturnScroll.current = panel.current?.scrollTop ?? 0;
    s.set({ place: cancel ? null : { city: city.id, building, intent, pick: null } });
    if (!cancel) s.jumpTo(city.pos);
  };
  /** "Buy a tile" opens the same placement mode: the map prices every buyable tile and a click on one queues the purchase. */
  const startBuyTile = () => {
    setImproveHover(null);
    const s = useGame.getState();
    placementReturnScroll.current = panel.current?.scrollTop ?? 0;
    s.set({ place: { city: city.id, building: null, intent: 'tile', pick: null } });
    s.jumpTo(city.pos);
  };
  const buyTile = (slot: 'row' | 'inline' | 'territory') => city.owner === view.me ? <CityBorders city={city} view={view} onChoose={startBuyTile} slot={slot} /> : null;
  // A lit tile clicked on the map becomes the order the placement was opened for (run by the effect above the early return).
  applyPlacePick.current = () => {
    if (!place?.pick || place.city !== city.id || !place.building) return;
    const o = siteOptions(place.building).find((x) => x.item.kind === 'Wonder' && hexEq(x.item.tile, place.pick!));
    useGame.getState().set({ place: null });
    if (!o) return;
    if (place.intent === 'queue') append(o.item);
    else if (place.intent === 'buy') buy(o.item, null);
    else buildNow(o.item);
  };
  // Short of a strategic resource, the newest buildings that need it yield nothing: say so beside the building list.
  const idleBuildings = d.idle_buildings ?? [];
  const noSupply = (resource: Resource) => !view.supply?.find((x) => x.resource === resource)?.supplied.total;
  const idleNote = idleBuildings.length > 0 && <div className="idle-building-note" role="note">
    <Icon name="ui-alert" size={12} className="ui-mark" />{' '}
    {idleBuildings.map((x) => `The ${itemName({ kind: 'Building', building: x.building }, owner?.faction)} is idle: ${noSupply(x.resource) ? `you have no ${x.resource}` : `you are short of ${x.resource}`}. It yields nothing until you get ${noSupply(x.resource) ? 'some' : 'more'}.`).join(' ')}
  </div>;
  if (peek) {
    const choice = d.options.find((o) => itemKey(o.item) === phoneChoice) ?? option
      ?? d.options.find((o) => !o.resource_blocker) ?? d.options[0];
    const choiceBlocked = !!choice?.resource_blocker && !sameItem(choice.item, producing);
    return <div className="phone-primary-actions phone-build-choice">
      {razeControl}
      <select aria-label={`Production for ${city.name}`} value={choice ? itemKey(choice.item) : ''} disabled={view.finished || !choice}
        onChange={(event) => { setPhoneChoice(event.target.value); feel.select(); }}>
        {/* The current build says so, so the closed picker names what the city is making (playtest-6 M2). */}
        {d.options.map((o) => <option key={itemKey(o.item)} value={itemKey(o.item)} disabled={!!o.resource_blocker && !sameItem(o.item, producing)}>
          {sameItem(o.item, producing) ? 'Building: ' : ''}{itemName(o.item, owner?.faction)} · {o.item.kind === 'Project' ? projectRate(o.item, o.yields) : `${o.turns > 0 ? `${o.turns} turns` : 'No output'} · ${costInFactionYield(owner?.faction ?? 'India', o.cost)} ${costYield(owner?.faction ?? 'India')}`}{o.supply_need ? ` · ${needText(o.supply_need)}` : ''}{o.resource_blocker && !sameItem(o.item, producing) ? ` · ${o.resource_blocker}` : ''}
        </option>)}
      </select>
      <button className="btn primary" data-primary-action disabled={view.finished || !choice || choiceBlocked}
        aria-label={choice ? `Build ${itemName(choice.item, owner?.faction)}` : 'Build'}
        aria-description={choiceBlocked ? choice!.resource_blocker! : undefined}
        onClick={() => { if (choice && !buildNow(choice.item)) advanceAfterBuild(); }}><span>{choice?.item.kind === 'Project' ? 'Run project' : 'Build'}</span></button>
      {choiceBlocked && <small className="phone-action-why prod-blocked" role="note"><ResourceReason text={choice!.resource_blocker!} resource={choice!.supply_need!} /></small>}
    </div>;
  }
  const industryFood = (item: ProductionItem) =>
    owner?.faction === 'Germany' && (item.kind === 'Building' || item.kind === 'Wonder') && isProductionBuilding(item.building);
  const renderOption = (o: ProductionOption, keeps?: string) => {
    // A tile-chosen build not yet placed stands for every eligible tile, so it goes by its plain name.
    const name = o.item.kind === 'Building' && o.item.building === 'UpgradeWalls' ? wallsUpgradeName(city.walls_level ?? Number(city.walls), itemName({ kind: 'Building', building: 'Walls' }, owner?.faction)) : o.item.kind === 'Wonder' && !siteChosen(o) ? baseItemName(o.item, owner?.faction) : itemName(o.item, owner?.faction);
    if (o.item.kind === 'Project') {
      return <div className="production-choice project-choice" key={itemKey(o.item)} data-item={itemKey(o.item)}>
        <Tip as="div" placement="right" summary={descriptionOf(o.item, owner?.faction)} content={descriptionOf(o.item, owner?.faction)}>
          <button className="prod" aria-label={`Run ${name}`} onClick={() => buildNow(o.item)} disabled={view.finished}>
            <span className="prod-icons"><ItemArt item={o.item} faction={owner?.faction} size={20} /></span>
            <span className="prod-name">{name}</span>
            <span className="prod-cost">{projectRate(o.item, o.yields)}</span>
            <span className="prod-go">{sameItem(producing, o.item) ? 'Active' : isGrow(o.item) ? 'Grow →' : 'Run project →'}</span>
          </button>
        </Tip>
        <EffectMore effect={summaryOf(o.item, owner?.faction)}><Amounts text={moreText(descriptionOf(o.item, owner?.faction), summaryOf(o.item, owner?.faction))} /></EffectMore>
      </div>;
    }
    const positions = queue.flatMap((item, i) => sameItem(item, o.item) ? [i + 1] : []);
    const active = !!producing && positions.includes(1);
    const chosen = pendingQueue && positions.length > 0;
    const unlock = view.research.unlocks.find((u) => sameItem(u.item, o.item));
    const after = unlock?.cities?.find((c) => c.city === city.id);
    const purchase = purchases.options.get(itemKey(o.item))!;
    const { cost: price, reason, selected: selectedBuy } = purchase;
    const duplicate = (o.item.kind === 'Building' || o.item.kind === 'Improvement' || o.item.kind === 'Wonder') && positions.length > 0;
    // G11: an item short of its strategic resource can't start now, but can wait behind other work.
    const resourceBlocked = !!o.resource_blocker && !active;
    const headOnly = queue.length === 0;
    // The card shows the item's own progress: the active build's, or what an earlier switch banked.
    const wasActive = sameItem(o.item, d.producing);
    const progress = wasActive && !buildChanged ? progressText(d.production_stored, 0, o.cost, true)
      : wasActive ? progressText(d.production_stored, d.production_stored, o.cost, false)
        : (o.production_banked ?? 0) > 0 ? progressText(o.production_stored, o.production_banked, o.cost, false) : null;
    const key = itemKey(o.item);
    // G53: every build card shows its ongoing cost. Gold/turn after sits in the tooltip, not the
    // card, to keep the card's fixed set of rows from growing per item (legatum-ui: must fit at
    // 1000x600).
    const upkeepLine = o.upkeep ? `${upkeepText(o.upkeep)}. Gold/turn ${signed(view.economy.gold_per_turn.total)} → ${signed(o.upkeep.gold_per_turn_after)} after building.` : null;
    const imp = o.item.kind === 'Improvement' ? o.item : null;
    const site = o.item.kind === 'Wonder' ? o.item : null;
    // Met rivals on the same wonder, with their public pace, fastest first (before the choice, not after the loss).
    const rivalBuilders = site && isGreatWork(site.building) ? rivalsOnWonder(view, site.building).map(rivalBuilderText) : [];
    // A tile-chosen build's card stands for every eligible tile until one is in the queue: its buttons open the placement.
    const unplaced = !!site && !siteChosen(o);
    const placingHere = !!site && placing(site.building);
    const tile = imp?.tile ?? (unplaced ? undefined : site?.tile);
    const picked = !!tile && !!improve.pick && hexEq(improve.pick, tile);
    // Map-first: a row lights its tile on the map while hovered or focused.
    const mapHover = tile ? { onPointerEnter: () => setImproveHover(tile), onPointerMove: () => setImproveHover(tile), onPointerLeave: () => setImproveHover(null), onFocus: () => setImproveHover(tile), onBlur: () => setImproveHover(null) } : {};
    const preview = o.improvement;
    const info = imp ? IMPROVEMENT_INFO[imp.improvement] : null;
    const techKnown = !!info && view.research.known.includes(info.upgradeTech);
    // "+2 Production when built" (the number already includes a known upgrade), "+2 Production and +1 Iron supply when built" on the improvement of a strategic deposit,
    // or "+1 Food when built · Later: Irrigated Farm with Crop Rotation, +1 Food".
    const impGain = imp && preview ? nileYields(owner?.faction, improvementGain(preview.resource, view.research.known)) : null;
    const supplyGain = imp && preview && isStrategic(preview.resource) ? `+${supplyPerMine(view.players.find((p) => p.id === view.me)?.faction)} ${RESOURCE_INFO[preview.resource].name} supply` : '';
    const gainYields = impGain && YIELD_KEYS.some((k) => impGain[k]) ? yieldsText(impGain) : '';
    const gains = [gainYields, supplyGain].filter(Boolean).join(' and ');
    // The card copy already leads `detail`, so a site's effect adds only what this tile gives up.
    const effect = site && o.wonder && !unplaced ? `Replaces ${yieldsText(o.wonder.lost)} on this tile${view.tiles[tileIndex(site.tile, view.width, view.height)]?.improvement ? ' and removes its improvement' : ''}.` : imp && preview && info ? [
      gains ? `${gains} when built` : null,
      ...(techKnown || !YIELD_KEYS.some((k) => nileYields(owner?.faction, improvementUpgradeYields(preview.resource))[k]) ? [] : [`Later: ${info.upgradedName} with ${TECH_NAME[info.upgradeTech]}, ${yieldsText(nileYields(owner?.faction, improvementUpgradeYields(preview.resource)))}`]),
    ].filter(Boolean).join(' · ') : null;
    const against = owner ? pieceAgainst(owner.faction, o.item) : null;
    // A piece that renames its standard item (the Alpine Redoubt) has one quote, so only a distinct item gives a second price.
    const againstCost = against && !sameItem(against, o.item) ? costOf(against) : null;
    // The gap is between the two prices this city quotes now (faction prices included), not the static tables.
    const vs = owner ? pieceComparison(owner.faction, o.item, againstCost !== null ? { now: o.cost, was: againstCost } : null) : null;
    const description = descriptionOf(o.item, owner?.faction);
    // Upgrade Walls' text is computed for this city, so its static card text would repeat it.
    const cardText = isUpgradeWalls(o.item) ? description : itemCardText(o.item, owner?.faction);
    const fullCopy = cardText === description ? description : `${cardText} ${description}`;
    const hallDetail = o.item.kind === 'Building' && o.item.building === 'YamRelayStation' ? hallCardText() : null;
    const detail = [fullCopy, effect, hallDetail, site && isGreatWork(site.building) ? `Only one empire can complete it. ${rivalBuilders.length ? `Known builders: ${rivalBuilders.join(', ')}.` : 'No known rival builder.'}` : null].filter(Boolean).join(' ');
    const cardLineBase = site && isGreatWork(site.building) ? `${summaryOf(o.item, owner?.faction).replace(/^World wonder: /, '').replace(/^whoever holds its tile scores (\+\d+ VP)$/, '$1 to its tile holder')} · one in the world${rivalBuilders.length ? `, also building: ${rivalBuilders.join(', ')}` : ''}` : [keeps, gains || summaryOf(o.item, owner?.faction)].filter(Boolean).join(' · ');
    const cardLine = industryFood(o.item) ? `${cardLineBase} · −1 Food` : cardLineBase;
    const card = <>
        <button className={`prod ${active ? 'active' : ''} ${chosen ? 'pending' : ''}`} aria-label={`Build ${name}`} aria-pressed={active}
        aria-disabled={resourceBlocked || undefined}
        aria-description={resourceBlocked ? `${o.resource_blocker}${o.supply_need ? ` Open ${o.supply_need} supply.` : ''}` : undefined}
        title={imp || site ? detail : undefined}
        disabled={view.finished} onClick={(event) => { if (unplaced) { startPlace(site!.building, event.shiftKey ? 'queue' : 'build'); return; } if (resourceBlocked) { if (headOnly) { if (o.supply_need) openResourceChip(o.supply_need); return; } if (!duplicate && queue.length < 50) append(o.item, event.currentTarget); return; } if (event.shiftKey) { if (!duplicate && queue.length < 50) append(o.item, event.currentTarget); } else if (!buildNow(o.item, event.currentTarget)) advanceAfterBuild(); }}>
        <span className="prod-icons"><ItemArt item={o.item} faction={owner?.faction} size={20} />{imp && preview && <Icon name={`resource-${preview.resource.toLowerCase()}`} size={18} label={RESOURCE_INFO[preview.resource].name} />}</span>
        <span className="prod-name">{imp ? <><span className="prod-what">{improvementName(imp.improvement, preview?.resource)}</span><span className="prod-where">{improvementWhere(imp, city.pos)}</span></> : site ? <><span className="prod-what">{baseItemName(site, owner?.faction)}</span>{unplaced && !isGreatWork(site.building) && <span className="prod-where place-where">{placingHere ? 'Click a lit tile on the map' : 'Choose its tile on the map'}</span>}{!unplaced && <><span className="prod-where">{sitePlace(site.tile)}</span>{o.wonder && <span className="prod-where site-cost"><Amounts text={`gives up ${yieldsText(o.wonder.lost)}`} /></span>}</>}</> : name} {positions.length > 1
          ? <span className="queue-badge queue-count" title={`In the queue ${positions.length} times (positions ${positions.join(', ')})`}>×{positions.length}</span>
          : positions.map((n) => <span className="queue-badge" key={n} title={`Position ${n} in the queue`}>{n}</span>)}</span>
        <span className="prod-cost">{progress ? <span className="prod-progress">{progress}</span> : <>{costInFactionYield(owner?.faction ?? 'India', o.cost)} <YieldIcon k={costYield(owner?.faction ?? 'India') === 'Food' ? 'food' : 'production'} size={11} /></>}{o.supply_need && <span className="prod-supply-need" title={`${needText(o.supply_need)} for as long as it exists. ${supplyRuleText(o.supply_need, view.players.find((p) => p.id === view.me)?.faction)}`}><Amounts text={needText(o.supply_need)} /></span>}</span>
        <span className="prod-turns">{o.turns > 0 ? `${o.turns} turn${o.turns === 1 ? '' : 's'}` : 'No output'}</span>
        {o.item.kind === 'Unit' && <span className={`prod-upkeep ${o.upkeep ? o.upkeep.free ? 'free' : 'paid' : 'none'}`}><Amounts text={upkeepText(o.upkeep)} />{o.upkeep && o.upkeep.gold_per_turn_after < 0 && <span className="prod-upkeep-after" title="Your total Gold/turn once this unit exists"> → <Amounts text={`${signed(o.upkeep.gold_per_turn_after)} Gold/turn`} /></span>}</span>}
        {imp && <span className="prod-upkeep none">No upkeep</span>}
        <span className="prod-go" title={chosen ? 'Chosen: it starts when the turn ends' : undefined}>{placingHere ? 'Cancel' : resourceBlocked ? headOnly ? 'Blocked' : 'Queue →' : chosen ? active ? 'Chosen' : 'Queued' : tombStopped ? 'Queue' : 'Build →'}</span>
      </button>
    </>;
    return <div className={`production-choice ${picked ? ' picked' : ''}${resourceBlocked ? ' supply-blocked' : ''}${site ? ' wonder-site' : ''}`} key={key} data-item={key} data-picked={picked || undefined}
      ref={picked ? (el) => el?.scrollIntoView?.({ block: 'nearest' }) : undefined} {...mapHover}>
      {imp || site ? <div className="tip-trigger">{card}</div> : <Tip as="div" placement="right" summary={description} content={<><p><Amounts text={`${detail} Build time if started now, at current yields; queued items may change it.`} /></p>{vs && <p><Amounts text={comparisonSentence(vs)} /></p>}{upkeepLine && <p><Amounts text={upkeepLine} /></p>}{after && o.item.kind === 'Building' && <><YieldsAfter now={d.yields.total} after={after.yields_after} /><p>Food left over after feeding the population: {after.food_surplus_after}.</p></>}</>}>{card}</Tip>}
      <div className="production-secondary">
        <button className={`queue-add ${flash?.key === key && flash.city === city.id ? 'queued' : ''}`} aria-label={`Queue ${name}`} title={duplicate ? 'Already in the queue' : resourceBlocked && headOnly ? 'Queue it behind another build: it starts once the resource is there' : 'Add to the end of the production queue'} disabled={view.finished || duplicate || queue.length >= 50 || (resourceBlocked && headOnly)} onClick={(event) => unplaced ? startPlace(site!.building, 'queue') : append(o.item, event.currentTarget)}>
          {flash?.key === key && flash.city === city.id ? <><Icon name="ui-check" size={11} className="ui-mark" /> {flash.label}</> : '+ Queue'}
        </button>
        {!site || !isGreatWork(site.building) ? <>
        <Tip content={`${reason ? `${reason}${reason.endsWith('.') ? ' ' : '. '}` : price !== null && !selectedBuy ? `Costs ${price} of your ${purchase.gold} Gold, leaving ${purchase.gold - price}. ` : ''}${purchase.breakdown}`}>
          <button className={`btn-buy ${selectedBuy ? 'active' : ''}`} aria-pressed={selectedBuy}
            aria-label={`Buy ${name}${price === null ? '' : ` for ${price} Gold`}`}
            aria-describedby={`purchase-status-${city.id}`}
            disabled={view.finished || (!selectedBuy && !!reason)} onClick={() => unplaced && !selectedBuy ? startPlace(site!.building, 'buy') : selectedBuy ? editOrders((ord) => toggleBuy(ord, city.id, o.item)) : buy(o.item, price)}>
            {/* G16: the price is a spend chip; a chosen buy keeps it, with ✓ (clicking again cancels). */}
            {selectedBuy ? <Icon name="ui-check" size={13} className="ui-mark" label="Buying" /> : 'Buy'} {price === null ? '—' : resourceBlocked && !selectedBuy ? null : <CostChip spends={[selectedBuy ? { unit: 'Gold', amount: price, from: purchase.gold, to: purchase.gold - price } : { unit: 'Gold', amount: price }]} />}
          </button>
        </Tip>
        </> : null}
      </div>
      {resourceBlocked && <div className="production-blocker prod-blocked"><Icon name="ui-alert" size={11} className="ui-mark" /><Amounts text={o.resource_blocker!} /></div>}
      <EffectMore effect={cardLine}><Amounts text={moreText(detail, cardLine, summaryOf(o.item, owner?.faction))} />{vs && <PieceVsLine comparison={vs} />}</EffectMore>
      {reason === "Walls must be built with Production; they can't be bought." && o.item.kind === 'Building' && buildingFamily(o.item.building) === 'Walls' &&
        <div className="production-refusal"><Amounts text={reason} /></div>}
      {o.item.kind === 'Building' && <WhereToBuild item={o.item} here={city.id} onEdit={keepInPlace} />}
      {flash?.key === key && flash.city === city.id && <span className="sr-only" role="status">{`${name}: ${flash.label} in the queue`}</span>}
    </div>;
  };
  /** The newer unit of a line that its strategic resource holds back: one line under the unit the city can build now. */
  const renderBetter = (o: ProductionOption) => {
    const name = itemName(o.item, owner?.faction);
    const key = itemKey(o.item);
    const short = o.supply_need ? `${name} needs ${o.supply_need}` : `${name} can't start yet`;
    const headOnly = queue.length === 0;
    const flashed = flash?.key === key && flash.city === city.id;
    return <div className="upgrade-line unit-better" key={key} data-better={key}>
      <Tip as="span" className="upgrade-detail" placement="right" summary={o.resource_blocker!} content={<p><Amounts text={`${o.resource_blocker} ${itemDescription(o.item, owner?.faction)}`} /></p>}>
        <ItemArt item={o.item} faction={owner?.faction} size={14} />
        <span>{o.supply_need ? <ResourceReason text={short} resource={o.supply_need} /> : short}</span>
      </Tip>
      <button className="upgrade-queue" aria-label={`Queue ${name}`} disabled={view.finished || headOnly || queue.length >= 50}
        title={headOnly ? 'Nothing to queue behind yet. Start another build first.' : `Add to the end of the queue. It starts when you have spare ${o.supply_need ?? 'supply'}.`}
        onClick={(event) => append(o.item, event.currentTarget)}>{flashed ? <><Icon name="ui-check" size={11} className="ui-mark" /> {flash.label}</> : '+ Queue'}</button>
    </div>;
  };
  /** A researched item this city can't choose yet: greyed out in its section, with the reason. */
  const renderUnavailable = (u: UnavailableOption) => {
    const name = u.item.kind === 'Building' && u.item.building === 'UpgradeWalls' ? wallsUpgradeName(city.walls_level ?? Number(city.walls), itemName({ kind: 'Building', building: 'Walls' }, faction)) : itemName(u.item, faction);
    const why = whyUnavailable(u);
    const wallsOnly = u.item.kind === 'Building' && buildingFamily(u.item.building) === 'Walls';
    const buyReason = wallsOnly ? "Walls must be built with Production; they can't be bought." : `Can't buy ${name} yet: ${lowerFirst(why.short)}.`;
    const key = itemKey(u.item);
    return <div className="production-choice unavailable" key={key} data-item={key} data-unavailable="true">
      <Tip as="div" placement="right" summary={why.full} content={<p><Amounts text={`${why.full} ${descriptionOf(u.item, faction)}`} /></p>}>
        <button className="prod" aria-label={`Build ${name}`} aria-disabled="true" aria-description={why.line} disabled={view.finished} onClick={() => refuse(u)}>
          <ItemArt item={u.item} faction={owner?.faction} size={20} />
          <span className="prod-name">{name}</span>
          {/* One statement of why not: a clicked card says it in red below, so the grey line steps aside. */}
          <span className="prod-cost">{u.reason.startsWith('Already built') || hallStands(u.reason) ? <><Icon name="ui-check" size={11} className="ui-mark" /> Built</> : u.production_banked > 0 ? <span className="prod-progress">{u.production_banked}/{u.cost} banked</span> : <>{costInFactionYield(owner?.faction ?? 'India', u.cost)} <YieldIcon k={costYield(owner?.faction ?? 'India') === 'Food' ? 'food' : 'production'} size={11} /></>}</span>
          <span className="prod-go">{u.reason.startsWith('Already built') || hallStands(u.reason) ? 'Built' : isPermanentlyUnavailable(u, faction) ? 'Never available' : 'Not yet'}</span>
        </button>
      </Tip>
      <div className="production-secondary">
        <button className="queue-add" aria-label={`Queue ${name}`} aria-disabled="true" disabled={view.finished} onClick={() => refuse(u)}>+ Queue</button>
        {!(u.item.kind === 'Wonder' && isGreatWork(u.item.building)) && <Tip content={buyReason}>
          <button className="btn-buy" aria-label={`Buy ${name}`} aria-description={wallsOnly ? buyReason : undefined} disabled>Buy</button>
        </Tip>}
      </div>
      {/* The reason sits in one place: the clicked refusal replaces the grey line where it stood, never below the yield line. */}
      {notice?.key === key && notice.city === city.id
        ? <div className={`production-notice ${notice.tone}`} role="status"><Amounts text={notice.text} /></div>
        : <div className="production-blocker prod-unavailable"><Amounts text={why.line} /></div>}
      <EffectMore effect={`${summaryOf(u.item, faction)}${industryFood(u.item) ? ' · −1 Food' : ''}`}><Amounts text={moreText(descriptionOf(u.item, faction), summaryOf(u.item, faction))} /></EffectMore>
      {wallsOnly && <div className="production-refusal"><Amounts text={buyReason} /></div>}
      {u.item.kind === 'Building' && <WhereToBuild item={u.item} here={city.id} onEdit={keepInPlace} hideNone />}
    </div>;
  };

  const queueUpgrade = (node: BuildingLineNode, ancestors: ProductionItem[]) => {
    const upgrade = node.entry.unavailable;
    if (!upgrade) return null;
    const parentItem = [...ancestors].reverse().find((item) => item.kind === 'Building' && item.building === upgrade.prerequisite_building);
    if (!parentItem) return null;
    const parentName = itemName(parentItem, faction);
    const path = [...ancestors, upgrade.item];
    const missing = path.filter((item) => item.kind === 'Building' && !d?.buildings.includes(item.building) && !queue.some((queued) => sameItem(queued, item)));
    const next = [...queue, ...missing];
    if (next.length === queue.length) return;
    changeQueue(next);
    setNotice(null);
    setFlash({ city: city.id, key: itemKey(upgrade.item), label: `Queued after ${parentName}` });
    clearTimeout(flashTimer.current);
    flashTimer.current = setTimeout(() => setFlash(null), 1500);
  };
  function renderUpgradeBranch(node: BuildingLineNode, ancestors: ProductionItem[]): ReactNode {
    const locked = node.entry.unavailable;
    if (!locked) return renderBuildingNode(node, ancestors);
    const parent = ancestors[ancestors.length - 1];
    const name = itemName(locked.item, faction);
    const parentName = itemName(parent, faction);
    const path = [...ancestors, locked.item];
    const missing = path.filter((item) => item.kind === 'Building' && !d?.buildings.includes(item.building) && !queue.some((queued) => sameItem(queued, item)));
    return <div className="upgrade-branch" key={itemKey(locked.item)}>
      <div className="upgrade-line">
        <Tip as="span" className="upgrade-detail" placement="right" summary={`${name} · ${locked.cost} Production`} content={<><p><Amounts text={`${descriptionOf(locked.item, faction)} ${whyUnavailable(locked).full}`} /></p><WhereToBuildLine item={locked.item} /></>}>
          <ItemArt item={locked.item} faction={owner?.faction} size={14} />
          <span>Leads to: {name} <span style={{ whiteSpace: 'nowrap' }}>({locked.cost} <YieldIcon k="production" size={11} />)</span></span>
        </Tip>
        <button className="upgrade-queue" disabled={view.finished || missing.length === 0 || queue.length + missing.length > 50} aria-label={`Queue ${name} after ${parentName}`} title={queue.length + missing.length > 50 ? 'Production queue is full' : undefined} onClick={() => queueUpgrade(node, ancestors)}>+ Queue after {parentName}</button>
      </div>
      {node.upgrades.length > 0 && <div className="upgrade-children">{node.upgrades.map((child) => renderUpgradeBranch(child, path))}</div>}
    </div>;
  }
  function renderBuildingNode(node: BuildingLineNode, ancestors: ProductionItem[] = []): ReactNode {
    const item = node.entry.row?.item ?? node.entry.unavailable!.item;
    const path = [...ancestors, item];
    return <div className="building-line" key={itemKey(item)}>
      {node.entry.row ? renderOption(node.entry.row) : renderUnavailable(node.entry.unavailable!)}
      {node.upgrades.length > 0 && <div className="upgrade-nest" aria-label={`Upgrades after ${itemName(item, faction)}`}>
        {node.upgrades.map((upgrade) => renderUpgradeBranch(upgrade, path))}
      </div>}
    </div>;
  }
  const renderBuildingPicker = (options: ProductionOption[], locked: UnavailableOption[]) =>
    nestBuildingUpgrades(withUnavailable(options, (o) => o.item, locked.filter((u) => !isPermanentlyUnavailable(u, faction)))).map((node) => renderBuildingNode(node));

  /** Improvements: every worked, known resource tile this city can still improve, with what it adds. Resources lie dormant until then. */
  const improvementOptions = listOptions.filter((o) => o.item.kind === 'Improvement');
  // The same check as the map's faded medallions and the tile card: owned, known and unimproved.
  const dormant = d.territory.filter((h) => { const t = view.tiles[tileIndex(h, view.width, view.height)]; return !!t && isDormant(t, view.me, owner?.faction); }).length;
  const improvements = (city.owner === view.me && improvementOptions.length > 0) ? <section aria-label="Improvements" data-testid="improvements-group">
    <Tip as="div" className="cp-sub" content={`A resource yields its bonus only once its tile is improved. A strategic deposit supplies its resource as soon as it is inside your borders. Its improvement (${['Mine', 'SaltpetreWorks', 'OilWell', 'QuartzMine'].map((k) => withArticle(IMPROVEMENT_INFO[k as ImprovementKind].name)).join(', ')}) adds its tile's yield and 1 more supply. Pick a tile here or click it on the map. Improvements upgrade by themselves with technology; a released tile keeps its improvement, and a captured city's land keeps it.`}>
      Improvements <span className="cp-sub-note">· {dormant} resource tile{dormant === 1 ? '' : 's'} unimproved</span>
    </Tip>
    <div className={`prod-list ${producing ? '' : 'choosing'}`}>{improvementOptions.map((o) => renderOption(o))}</div>
  </section> : null;
  // Byzantium's shop: only its own cities, never a rival's or a foreign city's panel.
  const hire = city.owner === view.me && canHire(view) ? <CompanyShop city={city} view={view} hires={purchases.hires} /> : null;

  /** Idle with nothing queued: the control sits on the "Choose what ... builds" line, so it never scrolls out of view above the cards and costs no extra row. */
  const idleChoice = city.owner === view.me && !producing && queue.length === 0;
  const autoSelect = <select id={`city-automation-${city.id}`} aria-label={`Automate ${city.name}`} title={buildFocus ? buildFocus === 'Food' && owner?.faction === 'Egypt' ? 'Runs Grow and builds nothing else.' : FOCUS_TIP[buildFocus] : 'Pick a focus: the city picks its builds or runs a project. Spends no Gold.'} value={buildFocus ?? ''}
    disabled={view.finished} onChange={(e) => editOrders((list) => setCityAutomation(list, city.id, (e.target.value || null) as CityBuildFocus | null))}>
    <option value="">Off</option>{FOCUS_CYCLE.map((f) => <option key={f} value={f}>{focusLabel(f, owner?.faction)}</option>)}
  </select>;
  const autoReason = buildFocus && productionWarning !== AUTOMATION_NO_CHOICE ? <p className="cp-automation-reason">{autoPick
    ? autoPickReason(autoPick, buildFocus, city, view)
    : buildFocus === 'Military' || buildFocus === 'Economy' ? `Your queue goes first. When it empties, the city picks ${buildFocus === 'Economy' ? 'an' : 'a'} ${buildFocus} build.` : buildFocus === 'Food' && owner?.faction === 'Egypt' ? 'Your queue goes first. When it empties, the city runs Grow.' : `Your queue goes first. When it empties, the city runs its ${focusLabel(buildFocus, owner?.faction)}.`}</p> : null;
  const automationRow = city.owner === view.me ? <section className="cp-automation" aria-label="City building automation">
    <label htmlFor={`city-automation-${city.id}`}>Automate <kbd className="kbd" title="I: on or off with your last focus. Shift+I: next focus.">I</kbd></label>
    {autoSelect}
    {autoReason}
  </section> : null;

  const wonderBlocker = (building: BuildingKind): string => {
    const winner = view.great_works?.find((w) => w.completed === building);
    if (winner) return `${baseItemName({ kind: 'Building', building })} was completed by ${view.players.find((p) => p.id === winner.player)?.faction ?? 'another empire'}.`;
    if (view.great_works?.some((w) => w.player === view.me && w.completed)) return 'Your empire has already completed its world wonder.';
    const other = view.cities.find((c) => c.owner === view.me && c.id !== city.id && c.detail && [c.detail.producing, ...(c.detail.production_queue ?? [])].some((item) => item?.kind === 'Wonder'));
    if (other) return `A world wonder is already being built or queued in ${other.name}.`;
    const tech = BUILDING_STATS[building].requires;
    if (tech && !view.research.known.includes(tech)) return `Needs ${TECH_NAME[tech]}.`;
    const required = greatWorkRequirement(building);
    if (required) {
      if (!d.buildings.includes(required[0])) return `Build ${baseItemName({ kind: 'Building', building: required[0] })} in this city first.`;
      const count = view.cities.filter((c) => c.owner === view.me && c.detail?.buildings.includes(required[0])).length;
      if (count < required[1]) return `Needs ${required[1]} cities with ${baseItemName({ kind: 'Building', building: required[0] })}; you have ${count}.`;
    }
    return 'No eligible owned land tile within 3 tiles of this city.';
  };
  const wonderCards = GREAT_WORKS.map((building) => {
    const option = listOptions.find((o) => o.item.kind === 'Wonder' && o.item.building === building);
    if (option) return renderOption(siteCard(building));
    const name = baseItemName({ kind: 'Building', building }, faction);
    return <div className="production-choice unavailable wonder-locked" key={building} data-unavailable="true" data-wonder={building}>
      <button className="prod" aria-label={`${name} unavailable`} disabled><ItemArt item={{ kind: 'Building', building }} faction={faction} size={20} /><span className="prod-name">{name}</span><span className="prod-cost">{costInFactionYield(faction ?? 'India', BUILDING_STATS[building].cost)} <YieldIcon k={costYield(faction ?? 'India') === 'Food' ? 'food' : 'production'} size={11} /></span><span className="prod-go">Not yet</span></button>
      {industryFood({ kind: 'Building', building }) && <div className="prod-effect"><Amounts text={`+${BUILDING_STATS[building].yields.production} Production · −1 Food`} /></div>}
      <div className="production-blocker prod-unavailable">{wonderBlocker(building)}</div>
    </div>;
  });

  if (!peek && place?.city === city.id) {
    if (place.intent === 'tile') return <section className="panel city-panel city-placement" aria-label="Choose a tile to buy" data-testid="city-placement">
      <p className="city-placement-kicker">{city.name} · Buy a tile</p>
      <p className="city-placement-instruction">Click a lit tile to buy it for {city.name}</p>
      <p className="city-placement-note"><Amounts text={`You have ${tilePurchasePreview(city, view, orders, errors).funds} Gold. A greyed price costs more than that.`} /></p>
      <button type="button" className="btn city-placement-cancel" onClick={() => useGame.getState().set({ place: null })}>Cancel <kbd>Esc</kbd></button>
    </section>;
    const name = baseItemName({ kind: 'Building', building: place.building! }, faction);
    return <section className="panel city-panel city-placement" aria-label="Choose a build tile" data-testid="city-placement">
      <p className="city-placement-kicker">{city.name} · {place.intent === 'buy' ? 'Buy' : place.intent === 'queue' ? 'Queue' : 'Build'} {name}</p>
      <p className="city-placement-instruction">Choose a tile for {name}: click a highlighted tile near {city.name}</p>
      <p className="city-placement-note">Hover a highlighted tile to see what it gives up.</p>
      <button type="button" className="btn city-placement-cancel" onClick={() => useGame.getState().set({ place: null })}>Cancel <kbd>Esc</kbd></button>
    </section>;
  }

  return (
    <div ref={panel} className="panel city-panel" style={{ ['--pc' as string]: col }}>
      {headWith(<>
      <div className="cp-yields" aria-label="City yields per turn" data-testid="city-yields">
        {YIELD_KEYS.map((k) => owner?.faction === 'Egypt' && k === 'production' ? (
          // The Nile: Egypt has no Production, so this cell says what its surplus Food builds instead.
          <Tip key={k} content={`Egypt has no Production. Surplus Food builds what ${city.name} is working on, or grows it if you choose Grow.`} className="cp-yield" placement="top">
            <span><YieldIcon k="food" size={15} /> <b data-testid="city-nile-builds">{egyptBuilding ? egyptProgress : 0}</b></span>
            <span className="cp-yield-l">Builds</span>
          </Tip>
        ) : (
          <Tip key={k} content={<YieldSourceTable yields={yields} k={k} />} summary={<CityYieldHover yields={yields} k={k} eaten={k === 'food' ? 2 * city.population : undefined} />} className="cp-yield" placement="top">
            {/* Food is what is left after the population eats, so a starving city reads negative (8.9.23 review: a gross "+2 Food" beside "losing population"). */}
            <span><YieldIcon k={k} size={15} /> <b className={k === 'food' && surplus < 0 ? 'neg' : undefined} data-testid={k === 'food' ? 'city-food-net' : undefined}>{k === 'food' ? signed(surplus) : yields.total[k]}</b></span>
            <span className="cp-yield-l">{YIELD_LABEL[k]}</span>
          </Tip>
        ))}
      </div>
      </>)}
      {status}
      {!idleChoice && automationRow}
      {city.owner === view.me && (view.aircraft ?? []).some((a) => a.city === city.id) && <div className="city-hangar" aria-label="City hangar">
        <b>Hangar {(view.aircraft ?? []).filter((a) => a.city === city.id).length} / {RULES.airBaseCap}</b>
        {(view.aircraft ?? []).filter((a) => a.city === city.id).map((a) => <button key={a.id} className="btn quiet" onClick={() => useGame.getState().select({ kind: 'air', id: a.id }, true)}>{baseItemName({ kind: 'Unit', unit: a.kind }, owner?.faction)} · {a.hp} HP</button>)}
      </div>}
      {city.owner !== view.me && city.visible && (view.air_sightings ?? []).some((a) => a.city === city.id) && <div className="city-hangar" aria-label="Visible rival hangar">
        <b>Hangar in sight</b>
        {(view.air_sightings ?? []).filter((a) => a.city === city.id).map((a, i) => <span key={i}>{baseItemName({ kind: 'Unit', unit: a.kind }, owner?.faction)}</span>)}
      </div>}
      {d.resistance > 0 && (
        <Tip as="div" className="cp-block cp-resist" content="A captured city resists: no Food, Production, growth or border growth until resistance ends.">
          <ProgressBar testId="city-resistance" label="Resistance after capture" tone="paused" color={YIELD_COLORS.production}
            value={Math.max(0, CAPTURE_RESISTANCE_TURNS - d.resistance)} max={CAPTURE_RESISTANCE_TURNS}
            turns={`${d.resistance} turn${d.resistance === 1 ? '' : 's'} left`} />
        </Tip>
      )}
      <div className="cp-progress" data-testid="city-progress">
        <div className="cp-progress-cell cp-territory-bar" onPointerEnter={() => setClaimHover(true)} onPointerLeave={() => setClaimHover(false)}
          onFocus={() => setClaimHover(true)} onBlur={() => setClaimHover(false)}>
          <Tip as="div" content={growthTip}>
            <ProgressBar testId="city-growth" label={growthLabel} name={growthLabelText} color={YIELD_COLORS.food}
              value={d.food_stored} max={d.growth_threshold} amount={`${d.food_stored}/${d.growth_threshold}`} foot={claimFoot ?? <span />}
              turns={growthTurns} tone={growth?.state === 'starving' ? 'bad' : growth?.state === 'growing' ? 'neutral' : 'paused'} />
          </Tip>
        </div>
      </div>
      {ownedWonders.length > 0 && <div className="cp-wonders" data-testid="city-wonders">
        {ownedWonders.map((w) => (
          <Tip key={`${w.hex.q},${w.hex.r}`} as="div" className="cp-wonder" content={<div className="bd"><div className="bd-note">{wonderBonusSentence(w.wonder)} It follows the tile: lose the tile and the bonus goes with it.</div></div>}>
            <img src={wonderArt(w.wonder, 'b')} width={26} height={26} alt="" />
            <span><b>{WONDER_INFO[w.wonder].name}</b> +{WONDER_PERCENT}% {wonderYieldWord(w.wonder)}{w.amount ? <> (<b>+{w.amount}</b> <YieldIcon k={WONDER_INFO[w.wonder].yield} size={13} />)</> : null}</span>
          </Tip>
        ))}
      </div>}
      <div className="cp-economy-row">
        <details className="cp-economy" open={territory.open} onToggle={(e) => setTerritoryOpen(e.currentTarget.open)}><summary>
          <span data-testid="city-tiles">Tiles</span>{' '}
          <span className="cp-economy-borders" data-testid="city-territory">{counts.worked} worked · {counts.owned} owned</span>
        </summary>
          <div className="cp-territory">
            <Tip as="div" className="hint" content={<div className="bd"><div className="bd-note">A city works every tile it owns within {WORK_RADIUS} tiles. Each population above 1 claims one more tile next to its border, nearest first and the best by its focus within a ring. An out-of-reach tile goes to another city of yours that can work it, if there is one. Otherwise it keeps rivals out. {TILE_RELEASE_RULE}</div></div>}>
              Works every tile it owns within {WORK_RADIUS} tiles{counts.owned > counts.worked ? ` · ${counts.owned - counts.worked} claimed, not worked` : ''}
            </Tip>
          </div>
        </details>
        {city.owner === view.me && <button className="link cp-compare" onClick={() => useGame.getState().set({ citiesOpen: true })}>Compare cities <kbd>C</kbd></button>}
      </div>
      {/* A building city has its picker far below the fold, so the tile purchase sits right under the tile count (an idle city keeps it in the picker's head). */}
      {!idleChoice && !occupiedLine && buyTile('territory')}

      <div className="cp-block">
        {/* The active item shares this row, keeping at least five choices visible at 1280x800. */}
        <ProgressBar className="cp-production" testId="city-production" name={egyptBuilding ? 'Built from Food' : 'Production'} color={YIELD_COLORS.production}
          label={<span className="cp-now-line"><span className="lbl">{tombStopped ? 'Waiting for the tomb' : 'Now producing'}</span>{producing ? <>
            <span className="cp-now-item"><ItemArt item={producing} faction={owner?.faction} size={16} /><span className="cp-now-name">{meterName(producing)}</span></span>
          </> : raceLost ? <>
            <span className="cp-now-item"><ItemArt item={{ kind: 'Building', building: raceLost.building }} faction={owner?.faction} size={16} /><span className="cp-now-name">{BUILDING_NAME[raceLost.building]}</span></span>
            <span className="cp-now-tag" data-testid="wonder-lost-tag">lost to {empireOfPlayer(view, raceLost.winner)}</span>
          </> : waitingQueue ? <>
            <span className="cp-now-item"><ItemArt item={queue[0]} faction={owner?.faction} size={16} /><span className="cp-now-name">{meterName(queue[0])}</span></span>
            <span className="cp-now-tag">{waitingTag(d) ?? 'waiting'}</span>
          </> : null}{city.owner === view.me && queue.length > 0 && (producing || (waitingQueue && !raceLost)) && (() => {
            // One click to stop the active build (user, 2026-11-01): the next queued item that can start takes over, or the city goes idle.
            const name = itemName(producing ?? queue[0], owner?.faction);
            const rest = queue.slice(1);
            const blockedWhy = (item: ProductionItem): string | null => {
              const o = d.options.find((x) => sameItem(x.item, item));
              if (!o) return 'it is not available now';
              if (item.kind === 'Unit' && item.unit === 'Settler' && city.population < 2) return 'a Settler needs population 2';
              return o.resource_blocker ?? null;
            };
            const first = rest.findIndex((item) => blockedWhy(item) === null);
            const after = first > 0 ? [rest[first], ...rest.filter((_, k) => k !== first)] : rest;
            const next = first >= 0 ? rest[first] : null;
            const tip = `Stop building ${name}: its progress stays banked${next ? `, and ${itemName(next, owner?.faction)} starts${first > 0 ? ` (${itemName(rest[0], owner?.faction)} cannot start: ${blockedWhy(rest[0])})` : ''}` : rest.length ? `. Nothing queued can start yet: ${itemName(rest[0], owner?.faction)} ${blockedWhy(rest[0])}` : ''}`;
            return <Tip content={tip} placement="top" className="cp-now-stop-tip"><button className="queue-remove cp-now-stop" data-testid="production-stop" aria-label={`Stop building ${name}`}
              disabled={view.finished} onClick={() => changeQueue(after)}><Icon name="ui-cross" size={12} /></button></Tip>;
          })()}</span>}
          turns={<span className={`cp-now-turns${tombStopped ? ' wrap' : ''}`}>
            {tombStopped ? (producing ? <>{d.production_stored}/{d.production_cost ?? option?.cost ?? '—'} stored · waits for the tomb</> : 'Nothing queued') : <>
            {producing && pendingQueue && <><span className="pending-dot" aria-hidden="true" /><span className="pending-label" title="Pending until Ready">pending</span>{' '}</>}
            {buildChanged
              ? !producing ? '—' : producing.kind === 'Project' ? projectRate(producing, yields) : option
                ? <>{option.production_stored}/{option.cost} stored · {egyptBuilding ? `+${egyptProgress} from Food` : <Amount sign="+" n={yields.total.production} unit="Production" />}/turn · {option.turns > 0 ? `${option.turns} turn${option.turns === 1 ? '' : 's'}` : '—'}</>
                : '—'
              : d.producing?.kind === 'Project' ? projectRate(d.producing, yields)
              : d.producing
                ? <>{d.production_stored}/{d.production_cost} stored · {egyptBuilding ? `+${egyptProgress} from Food` : <Amount sign="+" n={yields.total.production} unit="Production" />}/turn · {d.production_stall ? 'Stalled' : `${d.turns_to_complete ?? '—'} turn${d.turns_to_complete === 1 ? '' : 's'}`}</>
                : raceLost ? (d.production_stored > 0 ? `Stopped · ${d.production_stored} Production held for the next build` : 'Stopped · choose a new build')
                : waitingCap !== null ? `${d.production_stored}/${waitingCap} held` : `${d.production_stored}/${d.idle_production_cap} stored`}
            </>}
          </span>}
          value={storedProduction} max={waitingCap !== null ? Math.max(1, waitingCap) : pendingQueue ? producing ? option?.cost ?? d.production_cost ?? 1 : d.idle_production_cap : d.production_cost ?? d.idle_production_cap} />

        {switchNote && <div className="production-switch" data-testid="production-switch" role="status"><Amounts text={`${switchNote}.`} /></div>}
        {egyptBuilding && !tombStopped && <div className="production-switch" data-testid="egypt-construction" role="status">{`${city.name}'s ${nileSurplus} surplus Food goes into the ${producing ? itemName(producing, owner?.faction) : 'build'} this turn, so ${city.name} doesn't grow.`}{growOption && <>{' '}<button className="btn quiet" data-testid="egypt-grow-instead" disabled={view.finished} onClick={() => buildNow(growOption.item)}>Grow instead →</button></>}</div>}
        {parkedText && <div className="production-choose" data-testid="production-choose"><b>Choose a new build.</b> <Amounts text={parkedText} /></div>}
        {productionWarning && <Tip as="div" className="production-warning" content={FORECAST_ASSUMPTIONS}><Amounts text={productionWarning} /></Tip>}
        {shownBanks.length > 0 && <Tip as="div" className="cp-banked" content="Progress stays with the item you switched away from, in this city. It resumes when you build that item again and never moves to another item. Buying an item is a separate full-price purchase and leaves its banked progress alone.">
          <span data-testid="city-banked"><span className="lbl">Banked</span> {shownBanks.map((b) => `${itemName(b.item, faction)} ${b.progress}/${costOf(b.item) ?? '?'}`).join(' · ')}</span>
        </Tip>}
      </div>

      {/* The production queue: its own section under "Now producing", never hidden, with visible reorder and remove controls. */}
      {city.owner === view.me && <section className="cp-block production-queue" data-testid="production-queue" aria-label="Production queue">
        {/* The first item is the one in "Now producing" above, so it is not listed twice; the rows start at #2 and the up arrow on #2 swaps it in. */}
        {raceLost && queue.length > 0
          // The stopped city lists its whole waiting queue: nothing above is "now producing" it.
          ? <div data-testid="wonder-lost-queue"><p className="queue-empty"><span className="lbl">Queue</span> {queue.map((item) => itemName(item, owner?.faction)).join(' → ')} · waits for your choice</p>
            <div className="cp-resume"><button className="btn" disabled={view.finished} onClick={() => changeQueue(queue)}>Resume {itemName(queue[0], owner?.faction)}</button></div></div>
          : queue.length === 0
          ? <p className="queue-empty"><span className="lbl">Queue</span> Empty{!occupiedLine && ' · add from the list below'}</p>
          : queue.length === 1
            ? <p className="queue-empty"><span className="lbl">Queue</span> Nothing after this{!occupiedLine && ' · add from the list below'}</p>
          : <>
            <div className="queue-head"><span className="lbl">Next in queue</span><span className="queue-count-note">{queue.length - 1} item{queue.length === 2 ? '' : 's'} · builds in this order</span></div>
            <ol aria-label="Production queue" start={2}>
              {queue.map((item, i) => {
                if (i === 0) return null;
                const name = itemName(item, owner?.faction);
                const queued = d.options.find((o) => sameItem(o.item, item));
                const turns = queued?.turns;
                const waitsFor = queued?.resource_blocker ? queued.supply_need : null;
                const note = bankOf(item) > 0 ? `${bankOf(item)}/${costOf(item) ?? '?'} banked` : null;
                return <li key={`${i}-${itemKey(item)}`}>
                  <span className="queue-badge">{i + 1}</span>
                  <ItemArt item={item} faction={owner?.faction} size={18} />
                  <span className="queue-text">
                    <span className="queue-name">{name}</span>
                    <span className="queue-sub">{note && <><span className="queue-note">{note}</span><span aria-hidden="true"> · </span></>}<span className="queue-turns" title={waitsFor ? queued!.resource_blocker! : undefined}>{waitsFor ? <>needs <ResourceReason text={waitsFor} resource={waitsFor} /></> : turns && turns > 0 ? `${turns} turn${turns === 1 ? '' : 's'}` : '—'}</span></span>
                  </span>
                  <button className="queue-move" aria-label={`Move ${name} up`} title={`Move ${name} up in the queue`} disabled={view.finished} onClick={() => { const next = [...queue]; [next[i - 1], next[i]] = [next[i], next[i - 1]]; changeQueue(next); }}><Icon name="ui-chevron-up" size={14} /></button>
                  <button className="queue-move" aria-label={`Move ${name} down`} title={`Move ${name} down in the queue`} disabled={view.finished || i === queue.length - 1} onClick={() => { const next = [...queue]; [next[i], next[i + 1]] = [next[i + 1], next[i]]; changeQueue(next); }}><Icon name="ui-chevron-down" size={14} /></button>
                  <button className="queue-remove" aria-label={`Remove ${name} from queue`} title={`Remove ${name} from the queue`} disabled={view.finished} onClick={() => changeQueue(queue.filter((_, j) => i !== j))}><Icon name="ui-cross" size={12} /></button>
                </li>;
              })}
            </ol>
          </>}
      </section>}

      {razeControl}
      {occupiedLine ? <><p className="hint warn" data-testid="city-occupied">{occupiedLine}. Food, Gold and Science continue.</p></> : <>
      <div className="cp-block production-choose-block">
        {/* How Build and Queue differ is one hover away, so the choices start right under the title. */}
        <Tip as="div" className="cp-choose-title" content={owner?.faction === 'Egypt' ? EGYPT_BUILD_RULE : BUILD_RULE}>{buildFocus && idleChoice && productionWarning !== AUTOMATION_NO_CHOICE ? `Chooses at Ready · ${focusLabel(buildFocus, owner?.faction)}` : tombStopped ? 'Queue for after the tomb' : `Choose what ${city.name} builds`}</Tip>
      </div>
      {tombStopped && <p className="hint" data-testid="city-tomb-picker">You can still buy with Gold.</p>}
      <div className="cp-buyline">
        <div className="purchase-status" id={`purchase-status-${city.id}`} role="status">
          <Tip content="Gold available for purchases this turn."><span><Amount n={purchases.gold} unit="Gold" /> ({signed(view.economy.gold_per_turn.total)}/turn) to spend</span></Tip>
        </div>
        {idleChoice && <div className="cp-automation-inline" role="group" aria-label="City building automation">
          <label htmlFor={`city-automation-${city.id}`}>Automate <kbd className="kbd" title="I: on or off with your last focus. Shift+I: next focus.">I</kbd></label>{autoSelect}
        </div>}
      </div>
      {idleChoice && buyTile('row')}
      {idleChoice && autoPick && autoReason && <div className="cp-block cp-auto-note">{autoReason}</div>}


      {growOption && <><div className="cp-sub">Growth</div><div className="prod-list">{renderOption(growOption)}</div></>}
      <div className="cp-sub"><span>Units</span>{idleChoice && buyTile('inline')}</div>
      <LevyNote />
      <div className={`prod-list ${producing ? '' : 'choosing'}`}>
        {withUnavailable(unitLines(listOptions, owner?.faction ?? 'India'), (line) => line.main.item, listUnavailable.filter((u) => u.item.kind === 'Unit' && !isPermanentlyUnavailable(u, faction))).map((r) => r.row ? <div key={itemKey(r.row.main.item)}>
          {renderOption(r.row.main, r.row.keeps)}
          {r.row.better && renderBetter(r.row.better)}
          {r.row.older.length > 0 && <details className="unit-line"><summary>Also: {r.row.older.map((o) => itemName(o.item, owner?.faction)).join(', ')}</summary>{r.row.older.map((o) => renderOption(o))}</details>}
        </div> : renderUnavailable(r.unavailable))}{permanentLine(listUnavailable.filter((u) => u.item.kind === 'Unit'), listOptions.filter((o) => o.item.kind === 'Unit')) && <p className="prod-faction-rule" role="note">{permanentLine(listUnavailable.filter((u) => u.item.kind === 'Unit'), listOptions.filter((o) => o.item.kind === 'Unit'))}</p>}
      </div>
      {hire}
      <div className="cp-sub">Buildings</div>
      {idleNote}
      <div className={`prod-list ${producing ? '' : 'choosing'}`}>{renderBuildingPicker(listOptions.filter((o) => o.item.kind === 'Building'), listUnavailable.filter((u) => u.item.kind === 'Building'))}</div>
      {permanentLine(listUnavailable.filter((u) => u.item.kind === 'Building'), listOptions.filter((o) => o.item.kind === 'Building')) && <p className="prod-faction-rule" role="note">{permanentLine(listUnavailable.filter((u) => u.item.kind === 'Building'), listOptions.filter((o) => o.item.kind === 'Building'))}</p>}
      {improvements}

      {owner?.faction === 'Mongols' && <section aria-label="Yam Relay Station"><div className="cp-sub">Yam Relay Station · one per empire</div>{yamCard && <div className="prod-list">{renderOption(yamCard)}</div>}{!yamCard && <p className="hint">{(() => {
        const hall = view.courier_halls?.find((h) => h.of === city.owner);
        if (hall) return hall.holder === city.owner ? `The Yam Relay Station already stands in ${view.cities.find((c) => c.id === hall.city)?.name ?? 'your city'}.` : `The Yam Relay Station stands in ${view.cities.find((c) => c.id === hall.city)?.name ?? 'a rival city'}; retake the city instead.`;
        const building = view.cities.find((c) => c.owner === city.owner && c.id !== city.id && c.detail && [c.detail.producing, ...(c.detail.production_queue ?? [])].some((item) => item?.kind === 'Wonder' && item.building === 'YamRelayStation'));
        return building ? `The Yam Relay Station is already being built in ${building.name}.` : 'No eligible tile is available for the Yam Relay Station here.';
      })()}</p>}</section>}
      <section className="cp-great-works" aria-label="World wonders">
        <h3 className="cp-great-works-head">World wonders · held for <Amount sign="+" n={2} unit="VP" /></h3>
        <Tip as="div" className="hint" content={greatWorkRules(owner?.faction)}>Wonders are built with Production only. Each is built once in the world: choose Build, then click a lit tile on the map.</Tip>
        <div className="prod-list">{wonderCards}</div>
      </section>
      {/* Projects come last, as in Civ 6: the fallback when nothing else is worth building (Sam, 2026-11-22). */}
      {d.options.some((o) => o.item.kind === 'Project' && !isGrow(o.item)) && <><div className="cp-sub">Projects</div><div className="prod-list">{d.options.filter((o) => o.item.kind === 'Project' && !isGrow(o.item)).map((o) => renderOption(o))}</div></>}

      </>}

      <div className="cp-foot">
        <Tip content="Buildings in this city">
          <span className="lbl">Buildings</span>{' '}
          {d.buildings.length ? d.buildings.map((b, i) => {
            const idle = idleBuildings.find((x) => x.building === b);
            const name = itemName({ kind: 'Building', building: b }, owner?.faction);
            return <span key={b}>{i > 0 && ', '}{name}{idle && <span className="idle-building"> (Idle: {noSupply(idle.resource) ? 'no' : 'short of'} {idle.resource})</span>}</span>;
          }) : <span className="muted">none</span>}
        </Tip>
      </div>
      {errs.map((m, i) => (
        <div key={i} className="order-error">
          {m}
        </div>
      ))}
    </div>
  );
}

/** A build card's yield and its More toggle share one line; the toggle stays put when the text opens below it. */
function EffectMore({ effect, children }: { effect: string; children: ReactNode }) {
  return <details className="production-more">
    {/* More floats first, so the yield wraps around it and More never sits alone on a line. */}
    <summary><span className="more-toggle"><span className="more-closed">More</span><span className="more-open">Less</span></span><span className="prod-effect" title={effect}><Amounts text={effect} /></span></summary>
    <div className="production-more-text">{children}</div>
  </details>;
}
