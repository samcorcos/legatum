import type { PlayerView, TechUnlock } from '../types';
import { factionItemName, INDIA_CAPITAL_RADIUS, INDIA_NETWORK_RADIUS, INDIA_NETWORK_PERCENT_PER_CITY, INDIA_YIELD_PERCENT_PER_HEX, greatWorkRules, itemDescription, isGreatWork, itemName } from '../data';
import { firstBuildForecast } from '../research';
import { BreakdownTable, Tip, YieldSourceTable } from './index';
import { YIELD_KEYS, YIELD_LABEL } from './bits';
import { Amount, Amounts } from './Amounts';
import type { AmountUnit } from './amountParts';
import { WhereToBuild } from './WhereToBuild';

export function UnlockDetail({ view, unlock, turns }: { view: PlayerView; unlock: TechUnlock; turns: number | null }) {
  const build = firstBuildForecast(view, unlock, turns);
  const benefits = YIELD_KEYS.filter((k) => unlock.yields[k]).map((k) => `${unlock.yields[k] > 0 ? "+" : ""}${unlock.yields[k]} ${YIELD_LABEL[k]}`);
  if (unlock.city_strength) benefits.push(`+${unlock.city_strength} city strength`);
  if (unlock.unit_strength) benefits.push(`${unlock.unit_strength} melee strength`);
  if (unlock.unit_ranged) benefits.push(`${unlock.unit_ranged} ranged, range ${unlock.unit_range}, ${unlock.unit_moves} moves`);
  if (unlock.siege_bonus) benefits.push(`+${unlock.siege_bonus} ranged against city HP`);
  if (unlock.item.kind === 'Building' && isGreatWork(unlock.item.building)) benefits.push('+2 VP while its tile is held');
  const faction = view.players.find((p) => p.id === view.me)?.faction;
  if (unlock.item.kind === 'Building' && unlock.item.building === 'WarShrine') benefits.push(itemDescription(unlock.item, faction));
  if (unlock.item.kind === 'Building' && unlock.item.building === 'Roadworks') benefits.push('roads to every city within 8 tiles');
  // Faction rules that change an ordinary item (Mongol commerce, Byzantine Science, Germany's Markets);
  // replacements already show their own name and numbers.
  const factionNote = faction && unlock.item.kind === 'Building' && unlock.item.building !== 'WarShrine' && !factionItemName(faction, unlock.item.building)
    && itemDescription(unlock.item, faction) !== itemDescription(unlock.item, undefined) ? itemDescription(unlock.item, faction) : null;
  return <div className="research-unlock">
    <p><strong>{unlock.name}</strong>{benefits.length > 0 && <> · <Amounts text={benefits.join(', ')} /></>}</p>
    {unlock.locked_note ? <p className="research-locked" data-testid="research-locked">{unlock.locked_note}</p> : <p>{unlock.cost_breakdown ? <Tip content={<BreakdownTable b={unlock.cost_breakdown} title="Production cost" />}><span><Amount n={unlock.cost} unit="Production" /></span></Tip> : <Amount n={unlock.cost} unit="Production" />} · {unlock.buy_cost === null ? 'Cannot be bought' : <><Amount n={unlock.buy_cost} unit="Gold" /> to buy from zero progress</>}
      {unlock.upgrade_cost !== null && <> · {unlock.upgrade_from ?? 'Unit'} upgrade: <Amount n={unlock.upgrade_cost} unit="Gold" /> in your territory</>}</p>}
    {!unlock.locked_note && <WhereToBuild item={unlock.item} preview />}
    {factionNote && <p className="research-faction-note"><Amounts text={factionNote} /></p>}
    {faction === 'India' && YIELD_KEYS.some((k) => unlock.yields[k]) && <p className="research-faction-note">India: each other Indian city within {INDIA_NETWORK_RADIUS} tiles adds {INDIA_NETWORK_PERCENT_PER_CITY}% Food and Production. Corruption cuts city yields by {INDIA_YIELD_PERCENT_PER_HEX}% per tile beyond {INDIA_CAPITAL_RADIUS} from the capital, up to 100%. The city forecasts below include both.</p>}
    {unlock.predecessor && <p>Requires {itemName({ kind: 'Building', building: unlock.predecessor }, faction)}; it keeps that building's faction bonuses.</p>}
    {unlock.item.kind === 'Building' && isGreatWork(unlock.item.building) && <p><Amounts text={greatWorkRules(faction)} /></p>}
    {!!unlock.min_population && <p>Requires population {unlock.min_population} when starting or buying. Completed upgrades survive population loss.</p>}
    {/* The headline forecast; its research + build split and assumptions are the breakdown (pillar 5). */}
    {!unlock.locked_note && <p className="research-first-build">{build && turns !== null
      ? <Tip content={<div className="bd">
          <div className="bd-title">First {unlock.name} ≈ turn {view.turn + build.totalTurns - 1}</div>
          <div className="bd-row"><span>Research</span><span>{turns} turn{turns === 1 ? '' : 's'}</span></div>
          <div className="bd-row"><span>Build in {build.city}</span><span>{build.buildTurns} turn{build.buildTurns === 1 ? '' : 's'}</span></div>
          <div className="bd-note">Starts right after research from zero Production, at today's output.</div>
        </div>}><span><strong>First {unlock.name} ≈ turn {view.turn + build.totalTurns - 1}</strong> in {build.city}</span></Tip>
      : <span className="muted">No city can build it yet (needs an eligible city with Production).</span>}</p>}
    {!unlock.locked_note && unlock.cities && unlock.cities.length > 0 && <details><summary>City forecasts and requirements</summary><ul>{unlock.cities.map((c) => <li key={c.city}>
      {view.cities.find((city) => city.id === c.city)?.name}: {c.blocker ?? `${c.build_turns ?? '—'} turns to build from zero Production`}
      {!!unlock.min_population && (view.cities.find((city) => city.id === c.city)?.population ?? 0) < unlock.min_population && c.turns_to_eight_at_gate != null && <><br />Once the city reaches population {unlock.min_population} with every Food building and no Food stored, it takes {c.turns_to_eight_at_gate} more turns to reach population 8. The count leaves out the research and buildings still missing, and assumes the city claims no new tiles.</>}
      {!c.blocker && unlock.item.kind === 'Building' && <><br />At today's population, after completion: {YIELD_KEYS.map((k) => <Tip key={k} content={<YieldSourceTable yields={c.yields_after} k={k} />}><span><Amount n={c.yields_after.total[k]} unit={YIELD_LABEL[k] as AmountUnit} /> · </span></Tip>)}<br />Food surplus: {c.food_surplus_after} after feeding the current population.
        {unlock.yields.food > 0 && c.turns_to_eight_after !== undefined && <><br />With this building installed at today's population and stored Food: {(view.cities.find((city) => city.id === c.city)?.population ?? 0) < 6 && c.turns_to_six_after !== undefined && <>{c.turns_to_six_after === null ? 'population 6 needs more Food' : `${c.turns_to_six_after} turns to population 6`}; </>}{c.turns_to_eight_after == null ? 'population 8 needs more Food' : `${c.turns_to_eight_after} turns to population 8`}. Excludes research and construction time; holds territory fixed and uses balanced citizen priorities.</>}</>}
    </li>)}</ul></details>}
  </div>;
}
