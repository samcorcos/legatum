import { ownerName as ownerNameOf } from '../barbarians';
import type { BattleForecast, CityAssaultForecast, Faction, Hex, Order, PlayerView, UnitKind, UnitView } from '../types';
import { attacksInMelee, itemName, RULES } from '../data';
import { distance, hexEq } from '../hex';
import { isCalmRouteNote } from '../path';
import { BreakdownTable } from './bits';
import { VpSeal } from './vpParts';
import { capitalMark } from '../capitalMark';
import { playerColor } from '../theme';
import { useGame } from '../store';
import { cityFireStatus } from '../format';
import { CITY_HEAL, cityTileDefender, siegeTier, exactOdds, splitCounterattackLoss, outriderBlocks } from '../combat';
import { attackAftermath } from '../aftermath';
import { assaultSummary, attackerRisk, occupationLockWarning, cityEntryWarning, cityFireOutcome, cityShotTarget, cityStrikeWarning, forecastWinLabel, garrisonHeadline, wallsLine } from './forecastWarnings';
import { usePlacedCard, type CardAnchor } from './usePlacedCard';
import { forecastEraHeadline, outclasses, unitEra } from '../map/unitEra';
import { EraTab } from './EraTab';
import { RankMark } from './RankMark';

/**
 * Who defends a target hex, as the viewer knows it. On a city's tile the city defends while it has
 * HP and its garrison only at 0 HP (core `combat::city_tile_defender`); elsewhere a military unit
 * first, then any unit.
 */
function defenderAt(view: PlayerView, target: Hex) {
  const city = view.cities.find((c) => hexEq(c.pos, target));
  const garrison = view.units.find((u) => hexEq(u.pos, target) && u.military);
  const unit = city
    ? (cityTileDefender(city.hp, !!garrison) === 'Garrison' ? garrison : undefined)
    : garrison ?? view.units.find((u) => hexEq(u.pos, target));
  const owner = unit?.owner ?? city?.owner ?? 0;
  const name = unit && (!city || unit.military) ? itemName({ kind: 'Unit', unit: unit.kind }, view.players.find((p) => p.id === unit.owner)?.faction) : city?.name ?? 'Target';
  return { city, unit, owner, name };
}

/** The target's name for titles ("Attack Sunken India"), never raw coordinates. */
export function forecastTargetName(view: PlayerView, target: Hex): string | null {
  const d = defenderAt(view, target);
  return d.unit || d.city ? d.name : null;
}

/**
 * Everything the forecast says, with a walled city's shot folded into the headline numbers
 * (playtest-6 H1, pillar 5): the attacker's HP result, the odds and the verdict all count it.
 */
export function forecastModel(f: BattleForecast, ranged: boolean, view: PlayerView, unit: UnitView, target: Hex, orders: Order[], assault: CityAssaultForecast | null = null) {
  const { city, unit: defUnit, owner: defOwner, name: defName } = defenderAt(view, target);
  const faction = view.players.find((p) => p.id === unit.owner)?.faction;
  const unitLabel = itemName({ kind: 'Unit', unit: unit.kind }, faction);
  // The shot goes to the weakest of your units in range: fold it in only when that is this one.
  const shotTarget = city && f.city_strike ? cityShotTarget(view, city) : null;
  const elsewhere = shotTarget && shotTarget.id !== unit.id ? shotTarget : null;
  const fire = elsewhere ? null : cityFireOutcome(f, faction, unit.kind);
  const otherMelee = ranged ? [] : orders.filter((o) => (o.type === 'MoveUnit' || o.type === 'MoveUnitPath')
    && o.unit !== unit.id && hexEq(o.to, target) && view.units.some((u) => u.id === o.unit && u.owner === unit.owner));
  const attackers = otherMelee.length + 1;
  const fullCounter = fire ? fire.attack : f.expected_damage_to_attacker;
  const sharedCounter = Math.trunc(fullCounter / attackers);
  // At a city the headline chance is the one that matters: capturing it this turn, from core's pooled assault.
  const win = city && assault
    ? assault.can_capture ? assault.capture_chance : assault.garrison ? assault.garrison.destroyed_chance : fire?.win ?? f.win_chance
    : fire?.win ?? f.win_chance;
  const winLabel = city && assault
    ? assault.can_capture ? 'capture the city' : assault.garrison ? 'destroy the garrison' : 'drop it to 0 HP'
    : forecastWinLabel(f, !!city, ranged, !attacksInMelee(unit.kind));
  const loss = attackers > 1 && !fire
    ? splitCounterattackLoss(f.defender_strength.total, f.attacker_strength.total, f.attacker_hp, attackers)
    : fire?.loss ?? f.loss_chance;
  const taken = (fire?.fire ?? 0) + sharedCounter;
  const dealt = fire?.dealt ?? f.expected_damage_to_defender;
  const verdict = win >= 80 ? 'Decisive' : win >= 40 ? 'Favorable' : loss >= 40 ? 'Costly' : dealt > taken ? 'Advantage' : 'Risky';
  const tone = verdict === 'Decisive' || verdict === 'Favorable' || verdict === 'Advantage' ? 'good' : 'bad';
  const risk = fire
    ? fire.canDie ? `Risky: your ${unitLabel} may be destroyed (${fire.loss > 0 ? `${fire.loss}%` : 'under 1%'} chance, city fire included).` : null
    : attackers > 1
      ? loss > 0 ? `Risky: your ${unitLabel} may be destroyed (${loss}% chance, counterattack split ${attackers} ways).` : null
      : attackerRisk(f, unitLabel);
  const strike = city ? cityStrikeWarning(f, city.name, unitLabel, fire?.fire) : null;
  const hpAfter = fire && attackers === 1 ? fire.hpAfter : Math.max(0, f.attacker_hp - taken);
  // Sparta kills heal (see `killHeals`); a melee winner advances onto the defender's tile, so what
  // waits there next turn is part of the headline (p881 horseman).
  const heals = faction === 'Sparta' && !!defUnit?.military && !(city && city.hp > 0);
  // The forecast assumes a fortified defender stays fortified. A ranged unit that fires first loses
  // it (core `rules/battle.rs`: an attacker's fortify is cleared), and the p881 Archer did.
  const fortified = defUnit && !ranged ? f.defender_strength.parts.find(([label, amount]) => label === 'Fortified' && amount > 0)?.[1] ?? 0 : 0;
  const unfortified = fortified > 0
    ? exactOdds(f.attacker_strength.total, f.defender_strength.total - fortified, Math.max(1, f.attacker_hp - (fire?.fire ?? 0)), f.defender_hp, false).win
    : null;
  const aftermath = ranged ? null : attackAftermath(view, unit, target, f, hpAfter, heals, win > 0 || (unfortified ?? 0) > 0, unfortified);
  // Occupation lock experiment: a capture that can happen this turn also locks the capturing unit and freezes the city.
  const lockWarning = city && assault?.can_capture && assault.capture_chance > 0 ? occupationLockWarning(view, unit, city) : null;
  const elsewhereName = elsewhere ? itemName({ kind: 'Unit', unit: elsewhere.kind }, view.players.find((p) => p.id === elsewhere.owner)?.faction) : null;
  return {
    city, defUnit, defOwner, defName, unitLabel, fire, win, winLabel, loss, taken, dealt, verdict, tone, risk, strike,
    // The whole-fight block earns its room when a garrison stands under the city or other attacks are pooled; a lone attack on an empty city is already the card above.
    assault: city && assault && (assault.garrison || assault.attacks > 1) ? assault : null,
    counterText: otherMelee.length > 0 ? `${otherMelee.length} other${otherMelee.length === 1 ? '' : 's'} also attacking: you take about −${sharedCounter} instead of −${fullCounter}.` : null,
    ownerName: ownerNameOf(view, defOwner),
    defMax: city && !(defUnit && defUnit.military) ? city.max_hp : 100,
    hpAfter, aftermath, lockWarning,
    fortify: fortified > 0 && unfortified !== null && defUnit
      ? { bonus: fortified, win: unfortified, shoots: defUnit.ranged > 0 }
      : null,
    elsewhereText: city && elsewhere ? `${city.name} fires at the weakest unit in range: likely your ${elsewhereName} (${elsewhere.hp} HP), not this ${unitLabel}.` : null,
    // Only walled cities fire: say so when attacking one that can't, so no shot is expected.
    quiet: !!city && !city.walls && cityFireStatus(view, city) !== null,
  };
}

/** "If it destroys the Archer, your Horseman moves into its tile": the tile it advances into, with the risk word (p881 horseman). */
export function AftermathLine({ m }: { m: ReturnType<typeof forecastModel> }) {
  const a = m.aftermath;
  if (!a) return null;
  return (
    <div className={`forecast-shield forecast-aftermath${a.severe ? ' severe' : ''}`} data-testid="forecast-aftermath" data-risk={a.risk} role={a.severe ? 'alert' : undefined}>
      {a.line}
    </div>
  );
}

/** The assault block's words for this model (null when the target is not a city with a pooled assault). */
function assaultText(m: ReturnType<typeof forecastModel>, view: PlayerView) {
  if (!m.city || !m.assault) return null;
  const g = m.assault.garrison;
  const garrisonName = g ? itemName({ kind: 'Unit', unit: g.kind }, view.players.find((p) => p.id === g.owner)?.faction) : null;
  return assaultSummary(m.assault, m.city.name, garrisonName);
}

/**
 * The whole fight at a city in one place (user, 2026-10-19, T187 Lugano): the city, the garrison under it, whether it is
 * captured this turn and what your unit is left with. Every number is core's (`CityAssaultForecast`), pooled with the
 * attacks you have queued on the same city.
 */
function AssaultBlock({ m, view }: { m: ReturnType<typeof forecastModel>; view: PlayerView }) {
  const t = assaultText(m, view);
  if (!t) return null;
  return (
    <div className="forecast-assault" data-testid="forecast-assault" data-capture={m.assault!.capture_chance} data-attacks={m.assault!.attacks}>
      <div className="forecast-assault-title">{t.title}</div>
      {t.rows.map((r) => (
        <div key={r.key} className={`forecast-assault-row ${r.key}`} data-testid={`forecast-assault-${r.key}`}>
          <span className="forecast-assault-name">{r.label}</span>
          <span className="forecast-assault-hp">{r.hp} → <b>{r.after}</b> HP{r.note ? <em className={r.key === 'city' ? 'carry' : undefined} data-testid={r.key === 'city' ? 'forecast-assault-carried' : undefined}> {r.note}</em> : null}</span>
        </div>
      ))}
      <div className={`forecast-assault-capture ${t.captureTone}`} data-testid="forecast-assault-capture">{t.capture}</div>
    </div>
  );
}

/** The kill chance assumes a fortified defender stays fortified: say so, and what happens if it loses the bonus. */
export function FortifyLine({ m }: { m: ReturnType<typeof forecastModel> }) {
  const fortify = m.fortify;
  if (!fortify) return null;
  const fires = fortify.shoots && fortify.win !== m.win;
  // With the chance already in the aftermath line (a 0% forecast), the note only points at it.
  const pointer = m.win === 0 && !!m.aftermath;
  return (
    <div className="forecast-shield forecast-fortify" data-testid="forecast-fortify">
      Assumes the {m.defName} stays fortified (+{fortify.bonus} Defense).
      {fires && (pointer
        ? ` Firing loses it: chance above.`
        : ` ${/^[AEIOU]/.test(m.defName) ? 'An' : 'A'} ${m.defName} that fires first loses the +${fortify.bonus}: ${fortify.win}% to destroy it.`)}
    </div>
  );
}

/** Sparta heal only by kills: say so when this attack can destroy military (not a city with HP). */
function killHeals(view: PlayerView, unit: UnitView, m: ReturnType<typeof forecastModel>): boolean {
  return view.players.find((p) => p.id === unit.owner)?.faction === 'Sparta'
    && !!m.defUnit?.military && !(m.city && m.city.hp > 0);
}

/** Civ-style combat preview shown when hovering an enemy with a military unit selected. */
export function ForecastCard({
  forecast: f,
  anchor,
  ranged,
  view,
  unit,
  target,
  routeWarning,
  bowNote,
  assault,
}: {
  forecast: BattleForecast;
  anchor?: CardAnchor;
  ranged: boolean;
  view: PlayerView;
  unit: UnitView;
  target: Hex;
  /** The previewed route may halt before the attack (path.ts:routeRiskText). */
  routeWarning?: string;
  /** Why a unit that could also shoot (the Keshig) has no shot at this target: said beside the charge, never silently. */
  bowNote?: string;
  /** The whole fight at an enemy city with your other queued attacks on it (core `forecast_city_assault`). */
  assault?: CityAssaultForecast | null;
}) {
  const orders = useGame((s) => s.orders[s.seat]);
  const m = forecastModel(f, ranged, view, unit, target, orders, assault);
  const { city, fire, strike } = m;
  const spartaKill = killHeals(view, unit, m);
  const placed = usePlacedCard<HTMLDivElement>(anchor);
  const faction = view.players.find((p) => p.id === unit.owner)?.faction;
  const defFaction = view.players.find((p) => p.id === m.defOwner)?.faction;
  const walls = city ? wallsLine(f, city.name, m.unitLabel, siegeTier(view.research.known), m.dealt) : null;
  const outriderEscapes = !ranged && !!m.defUnit
    && outriderBlocks(unit.kind as UnitKind, m.defUnit.kind as UnitKind, distance(unit.pos, target));
  return (
    <div className="forecast" ref={placed.ref} style={placed.style} data-verdict={m.verdict}>
      <div className={`forecast-head ${m.tone}`}>
        <span className="forecast-kind">{ranged ? 'Ranged strike' : 'Attack'}</span>
        <span className="forecast-verdict">{m.verdict}</span>
      </div>
      {city && capturePointFor(view, city, unit.owner) && !ranged &&
        // The point a capture scores leads the card, in gilt under the header: it can decide the attack (8.9.23 review: it sat in the grey footnotes).
        <div className="forecast-vp" data-testid="forecast-capital-vp"><VpSeal vp={1} className="inline" /><span>+1 VP while you hold {city.name}</span></div>}
      {outriderEscapes && <div className="forecast-risk forecast-outrider" data-testid="forecast-outrider">If the {m.defName} moves this turn, it gets away.</div>}
      {m.aftermath?.severe && <AftermathLine m={m} />}
      {m.defUnit?.military && outclasses(m.defUnit.kind as UnitKind, defFaction, unit.kind as UnitKind, faction, f.defender_strength.total, f.attacker_strength.total) && (
        // A newer, stronger unit leads the card (user, 2026-10-11: he attacked a Phalangite with a Hoplite without noticing).
        <div className="forecast-era bad" data-testid="forecast-era">
          {forecastEraHeadline(m.defUnit.kind as UnitKind, defFaction, unit.kind as UnitKind, f.defender_strength.total, f.attacker_strength.total)}
        </div>
      )}
      <div className="forecast-sides">
        <Side
          name={m.unitLabel}
          level={unit.level}
          faction={faction}
          owner="You"
          era={unit.military ? unitEra(unit.kind as UnitKind) : undefined}
          color={playerColor(unit.owner)}
          strength={f.attacker_strength.total}
          hp={f.attacker_hp}
          max={100}
          after={m.hpAfter}
          fire={fire?.fire}
          attack={fire ? fire.attack : undefined}
        />
        <div className="forecast-vs">vs</div>
        <Side
          name={m.defName}
          level={m.defUnit?.level}
          faction={defFaction}
          owner={m.ownerName}
          era={m.defUnit?.military ? unitEra(m.defUnit.kind as UnitKind) : undefined}
          color={playerColor(m.defOwner)}
          strength={f.defender_strength.total}
          hp={f.defender_hp}
          max={m.defMax}
          after={m.assault && m.city && !m.defUnit?.military ? m.assault.city_hp_after : Math.max(0, f.defender_hp - m.dealt)}
          right
        />
      </div>
      {m.aftermath && !m.aftermath.severe && <AftermathLine m={m} />}
      {city && m.assault && <AssaultBlock m={m} view={view} />}
      {m.lockWarning && <div className="forecast-risk bad" role="alert" data-testid="forecast-occupation-lock">{m.lockWarning}</div>}
      {city && !m.assault && f.garrison && <div className="forecast-shield" data-testid="forecast-garrison">{garrisonHeadline(f, view, city.name, ranged)}</div>}
      {city && walls && <div className={`forecast-shield forecast-walls${walls.tone === 'good' ? ' good' : ''}`} data-testid="forecast-walls">{walls.text}</div>}
      {city && city.hp > 0 && m.dealt > 0 && <div className="forecast-foot forecast-note" data-testid="forecast-city-heal">Damaging {city.name} stops its +{CITY_HEAL} HP end-of-turn heal this turn.</div>}
      {m.counterText && <div className="forecast-foot forecast-counter" data-testid="forecast-counter-split">{m.counterText}</div>}
      {/* The city's shot changes the outcome, so it is a red risk row up top, never a footnote. */}
      {fire && city && strike && <div className="forecast-risk bad forecast-city-fire" data-testid="forecast-city-fire" role="alert">
        City fire: {city.name} {f.walls && f.walls.covered > 0 ? 'still ' : ''}fires first, about −{fire.fire} HP to your {m.unitLabel}{strike.deadly ? ', and it could destroy it' : ''}. <span className="forecast-included">Included in your unit’s HP loss above.</span>
      </div>}
      {routeWarning && <div className={isCalmRouteNote(routeWarning) ? 'forecast-shield route-note' : 'forecast-risk bad'} role={isCalmRouteNote(routeWarning) ? undefined : 'alert'} data-testid="route-warning">{routeWarning}</div>}
      {bowNote && <div className="forecast-shield forecast-bow-note" data-testid="forecast-shot-why">{bowNote}</div>}
      {m.risk && <div className="forecast-risk bad" role="alert">{m.risk}</div>}
      <div className="forecast-odds">
        <div>
          <b className="good">{m.win}%</b> {m.winLabel}
        </div>
        <div>
          <b className={m.loss > 0 ? 'bad' : ''}>{m.loss}%</b> lose your unit
        </div>
      </div>
      <FortifyLine m={m} />
      <div className="forecast-bd">
        <BreakdownTable b={f.attacker_strength} title="Your strength" />
        <BreakdownTable b={f.defender_strength} title="Their strength" />
      </div>
      <div className="forecast-foot forecast-dealt">
        Expected damage: you deal <b>{m.dealt}</b>, take <b>{m.taken}</b>
        {fire ? <> ({fire.fire} from city fire{ranged ? '' : `, ${fire.attack} in the attack`})</> : ranged ? ' (no retaliation)' : ''}. Rolls 80–120%.
      </div>
      {city && f.sack_gold !== undefined && <div className="forecast-foot" data-testid="forecast-sack">Sack: +{f.sack_gold} Gold ({RULES.celtsSackGoldPerPopulation} per population)</div>}
      {fire && strike && <details className="forecast-foot forecast-strike"><summary>How city fire works</summary><span className="forecast-strike-math">It fires at the weakest unit in range. {strike.details}{ranged ? '' : ' Your attack then fights at the reduced HP.'}</span></details>}
      {strike && !fire && <div className="forecast-foot forecast-strike" data-testid="forecast-city-fire-elsewhere">
        {m.elsewhereText ?? strike.text} <details><summary>How city fire works</summary><span className="forecast-strike-math">{strike.details}</span></details>
      </div>}
      {city && !strike && m.quiet && <div className="forecast-foot" data-testid="forecast-no-city-fire">{city.name} cannot fire back: it has no Walls.</div>}
      {spartaKill && <div className="forecast-foot forecast-note">A kill heals your surviving attackers +100 HP, shared with Spartan units that damaged this target this turn.</div>}
      {!ranged && <div className="forecast-foot forecast-note">Based on visible positions. Incoming units can change the defender and fight without terrain or fortify bonuses.</div>}
      {/* With the whole-fight block the card already says whether the city is captured; the capture rules fold away (they scroll open when wanted). */}
      {city && !m.assault && <div className={`forecast-foot${ranged || !attacksInMelee(unit.kind) ? '' : ' forecast-note'}`}>{cityEntryWarning(unit, ranged)}</div>}
      {city && m.assault && <details className="forecast-foot forecast-strike forecast-note" data-testid="forecast-capture-rules"><summary>How capture works</summary><span className="forecast-strike-math">{cityEntryWarning(unit, ranged)}</span></details>}
      {!ranged && <div className="forecast-foot forecast-pursuit forecast-note">
        {outriderEscapes
          ? `If it rides away, your ${m.unitLabel} stops where it is and needs orders. If it stays, your ${m.unitLabel} attacks once.`
          : `One attack this turn, then your ${m.unitLabel} stays where it fought and needs orders.`}
        {!city && m.defUnit && !outriderEscapes ? ` It targets the ${m.defName}, not its tile: if the ${m.defName} moves this turn, your ${m.unitLabel} pursues it and attacks where it stops, if it can reach it.` : ''}
      </div>}
    </div>
  );
}

/**
 * Phone and tablet: the forecast's headline where you confirm (playtest-6 H2): verdict, both HP
 * results (city fire included) and the odds, in a few lines above Confirm. The full card follows.
 */
export function ForecastSummary({ forecast: f, ranged, view, unit, target, assault }: { forecast: BattleForecast; ranged: boolean; view: PlayerView; unit: UnitView; target: Hex; assault?: CityAssaultForecast | null }) {
  const orders = useGame((s) => s.orders[s.seat]);
  const m = forecastModel(f, ranged, view, unit, target, orders, assault);
  const spartaKill = killHeals(view, unit, m);
  return (
    <div className="forecast-summary" data-testid="forecast-summary" data-verdict={m.verdict}>
      <div className="forecast-summary-line">
        <b className={`forecast-summary-verdict ${m.tone}`}>{m.verdict}</b>
        <span>You <RankMark level={unit.level ?? 0} faction={view.players.find((p) => p.id === unit.owner)?.faction} compact /> <b>{f.attacker_hp} → {m.hpAfter}</b> HP</span>
        <span>{m.defName} <RankMark level={m.defUnit?.level ?? 0} faction={view.players.find((p) => p.id === m.defOwner)?.faction} compact /> <b>{f.defender_hp} → {Math.max(0, f.defender_hp - m.dealt)}</b></span>
      </div>
      <div className="forecast-summary-line muted">
        <span><b className="good">{m.win}%</b> {m.winLabel}</span>
        <span><b className={m.loss > 0 ? 'bad' : ''}>{m.loss}%</b> lose your unit</span>
      </div>
      {m.aftermath && <div className={`forecast-summary-shield forecast-aftermath${m.aftermath.severe ? ' severe' : ''}`} data-testid="forecast-aftermath" data-risk={m.aftermath.risk}>{m.aftermath.line}</div>}
      {m.fortify && <div className="forecast-summary-shield forecast-fortify" data-testid="forecast-fortify">Assumes the {m.defName} stays fortified (+{m.fortify.bonus} Defense).</div>}
      {m.city && m.assault && <div className={`forecast-summary-shield forecast-assault-capture ${assaultText(m, view)!.captureTone}`} data-testid="forecast-assault">{assaultText(m, view)!.capture}</div>}
      {m.lockWarning && <div className="forecast-summary-shield" data-testid="forecast-occupation-lock">{m.lockWarning}</div>}
      {m.city && !m.assault && f.garrison && <div className="forecast-summary-shield">{garrisonHeadline(f, view, m.city.name, ranged)}</div>}
      {/* The touch card has no room to spare: it names the walls only when Siege changes the fight; the numbers above already show the halving. */}
      {m.city && f.walls && f.walls.covered > 0 && <div className="forecast-summary-shield" data-testid="forecast-walls">{wallsLine(f, m.city.name, m.unitLabel, siegeTier(view.research.known))!.text}</div>}
      {m.counterText && <div className="forecast-summary-shield">{m.counterText}</div>}
      {m.fire && m.city && <div className="forecast-summary-fire bad">City fire: {m.city.name} {f.walls && f.walls.covered > 0 ? 'still ' : ''}fires first, about −{m.fire.fire} HP</div>}
      {spartaKill && <div className="forecast-summary-line">A kill heals your surviving attackers +100 HP, shared.</div>}
    </div>
  );
}

function Side({
  name,
  level,
  faction,
  owner,
  color,
  strength,
  hp,
  max,
  after,
  fire,
  attack,
  right,
  era,
}: {
  name: string;
  level?: number;
  faction?: Faction;
  owner: string;
  color: string;
  strength: number;
  hp: number;
  max: number;
  after: number;
  /** The city's shot, when it is folded into this side's result. */
  fire?: number;
  attack?: number;
  right?: boolean;
  /** The unit's era (0-6): the same tab its miniature wears on the map. */
  era?: number;
}) {
  const shot = Math.min(fire ?? 0, hp);
  return (
    <div className={`fside ${right ? 'right' : ''}`}>
      <div className="fside-name" style={{ color }}>
        <RankMark level={level ?? 0} faction={faction} compact />{name}
      </div>
      <div className="fside-owner">{owner}{era !== undefined && <> <EraTab era={era} /></>}</div>
      <div className="fside-str">{strength}</div>
      <div className="hpbar">
        <div className="hp-after" style={{ width: `${(after / max) * 100}%` }} />
        <div className="hp-lost" style={{ width: `${((hp - after - shot) / max) * 100}%` }} />
        {shot > 0 && <div className="hp-lost hp-fire" style={{ width: `${(shot / max) * 100}%` }} />}
      </div>
      <div className="fside-hp" data-testid={right ? undefined : 'forecast-attacker-hp'}>
        {hp} → <b>{after}</b> HP
      </div>
      {fire !== undefined && <div className="fside-split">−{fire} city fire{attack ? `, −${attack} attack` : ''}</div>}
    </div>
  );
}

/** Whether capturing this city gives the unit's owner the original-capital VP to hold: it is a rival of the founder and does not hold the city already (core `scoring::capitals_held`). */
function capturePointFor(view: PlayerView, city: { id: number; owner: number; original_capital?: boolean; original_capital_of?: number | null }, captor: number): boolean {
  const mark = capitalMark(view, city);
  if (!mark || mark.own || mark.teammate || city.owner === captor) return false;
  const captorTeam = view.players.find((p) => p.id === captor)?.team;
  return captorTeam === undefined || view.players.find((p) => p.id === mark.founder)?.team !== captorTeam;
}
