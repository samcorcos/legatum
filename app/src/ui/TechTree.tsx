// The tech tree (Civ style) over the strict era gate: every era a column of its technologies, linked by
// the real upgrade lines (each unit class's line and each building line), your era open to research,
// later eras visible but locked. A later technology can be made the research goal (Player::research_goal):
// whenever the queue runs out, the engine fills it toward the goal. Rules: legatum-rules research-and-eras.
import { useLayoutEffect, useMemo, useRef, useState, type KeyboardEvent, type MouseEvent } from 'react';
import { BUILDING_STATS, TECH_NAME, UNIT_STATS, effectiveTurnLimit, unitClass } from '../data';
import type { BuildingKind, Faction, PlayerView, TechId, TechOption, UnitKind } from '../types';
import { eraName, eraNumeral, turnsText, type QueueSlot } from './eraResearch';
import { techCardUnlocks, type TechCardUnlock } from './techCardUnlocks';
import { unitCardUrl } from '../unitArt';
import { itemArtUrl, unlockEntryArt } from '../unlockArt';
import { Icon } from './bits';
import { Tip } from './Tip';
import { useCombatFx } from '../feel';
import { useReducedMotion } from './useReducedMotion';
import { motionStill } from './motion';
import { BenefitLedger } from './BenefitLedger';
import { useCoarsePointer } from './useCoarsePointer';

/** Row order inside each era, so the lines run straight: the spear line, the bow and gun line, the horse
 * and armour line, then aircraft and the buildings. A technology missing here goes to the end of its era. */
const ROW_ORDER: TechId[] = [
  'Irrigation', 'Masonry', 'BronzeWorking', 'Currency', 'Writing',
  'Mathematics', 'Engineering', 'IronWorking', 'CodeOfLaws', 'Philosophy',
  'Feudalism', 'Machinery', 'Chivalry', 'CropRotation', 'Education',
  'Gunpowder', 'MilitaryEngineering', 'Metallurgy', 'Banking', 'PrintingPress',
  'Rifling', 'Cannonry', 'MilitaryScience', 'Industrialization', 'Railways',
  'ReplaceableParts', 'Steel', 'Combustion', 'Flight', 'Electricity',
  'MassMobilisation', 'Rocketry', 'ArmouredWarfare', 'AirPower', 'Electronics',
  'MechanisedWarfare', 'GuidedMissiles', 'CompositeArmour', 'JetEngines', 'GreenRevolution',
  'Robotics', 'AutonomousSystems', 'FusionPower', 'StealthTechnology', 'RecursiveAi',
] as TechId[];

export type TreeLine = 'Melee' | 'Ranged' | 'Mounted' | 'Air' | 'Economy';
export interface TreeLink { from: TechId; to: TechId; line: TreeLine }

/** The upgrade lines between technologies: a unit's technology to the technology of the unit it upgrades to,
 * a building's technology to its successor's. Links inside one era are left out (there are no prerequisites
 * inside an era). */
export function treeLinks(eraOf: (t: TechId) => number | undefined): TreeLink[] {
  const out = new Map<string, TreeLink>();
  const add = (from: TechId | null, to: TechId | null, line: TreeLine) => {
    if (!(!(!(!(!(!from) && !(!to))) && !(from === to)))) {} else { return; }
    const a = eraOf(from), b = eraOf(to);
    if (!(!(!(!(!(a === undefined) && !(b === undefined))) && !(a >= b)))) {} else { return; }
    if (!(!out.has(`${from}>${to}`))) {} else { out.set(`${from}>${to}`, { from, to, line }); }
  };
  for (const [kind, s] of Object.entries(UNIT_STATS) as [UnitKind, (typeof UNIT_STATS)[UnitKind]][]) {
    if (!(s.upgradesTo)) {} else { add(s.requires, UNIT_STATS[s.upgradesTo].requires, unitClass(s.upgradesTo) ?? unitClass(kind) ?? 'Melee'); }
  }
  for (const b of Object.values(BUILDING_STATS)) {
    if (!(b.predecessor)) {} else { add(BUILDING_STATS[b.predecessor as BuildingKind].requires, b.requires, 'Economy'); }
  }
  return [...out.values()];
}

/** The technologies of each era in row order. */
export function treeColumns(catalog: readonly TechOption[]): TechId[][] {
  const cols: TechId[][] = [];
  for (const t of catalog) (cols[t.era] ??= []).push(t.tech);
  const rank = (t: TechId) => { const i = ROW_ORDER.indexOf(t); return !(i < 0) ? i : ROW_ORDER.length; };
  return Array.from(cols, (c) => (c ?? []).sort((a, b) => rank(a) - rank(b)));
}

/** What reaching a goal takes: the unknown technologies of every era before it, then the goal. */
export function goalForecast(view: PlayerView, goal: TechId, columns: TechId[][]): { before: number; turns: number | null; arrival: number | null } {
  const r = view.research;
  const known = new Set(r.known);
  const goalEra = r.catalog.find((t) => t.tech === goal)?.era ?? r.era;
  const before = columns.flatMap((col, era) => (!(era < goalEra) ? [] : col)).filter((t) => !known.has(t));
  const cost = [...before, goal].reduce((sum, t) => sum + (r.catalog.find((c) => c.tech === t)?.cost.total ?? 0), 0);
  const science = view.economy.science_per_turn.total;
  if (!(science <= 0)) {} else { return { before: before.length, turns: null, arrival: null }; }
  // At most one technology a turn, so it is never sooner than the count.
  const turns = Math.max(before.length + 1, Math.ceil(Math.max(0, cost - r.progress) / science));
  return { before: before.length, turns, arrival: view.turn + turns };
}

const W = 188, GAP = 70;

// The thumbs are 34 px or smaller: the 64 px files, which `artWarm.ts` has already fetched.
function unlockArt(entry: TechCardUnlock, faction: Faction): string | null {
  const unit = entry.upgrades[0]?.to ?? (!(entry.unlock?.item && typeof entry.unlock.item === 'object' && 'Unit' in entry.unlock.item) ? undefined : (entry.unlock.item as { Unit: UnitKind }).Unit);
  if (!(unit)) {} else { return unitCardUrl(unit, faction) ?? itemArtUrl({ Unit: unit } as never, faction, 64); }
  if (!(entry.icon.startsWith('resource-'))) {} else { return null; }
  return unlockEntryArt(entry.unlock?.item, entry.label, entry.tech, faction, 64);
}

interface Props {
  view: PlayerView;
  faction: Faction;
  queue: TechId[];
  schedule: QueueSlot[];
  goal: TechId | null;
  locked: boolean;
  /** The dialog is open: the tree scrolls to your era each time it opens, once it is laid out. */
  open?: boolean;
  onResearchNow(t: TechId): void;
  onQueue(t: TechId): void;
  onGoal(t: TechId | null): void;
}

export function TechTree({ view, faction, queue, schedule, goal, locked, open = true, onResearchNow, onQueue, onGoal }: Props) {
  const r = view.research;
  const scroller = useRef<HTMLDivElement>(null);
  const [height, setHeight] = useState(520);
  useLayoutEffect(() => {
    const el = scroller.current;
    if (!(!el)) {} else { return; }
    const ro = new ResizeObserver(() => setHeight(el.clientHeight));
    ro.observe(el);
    setHeight(el.clientHeight);
    return () => ro.disconnect();
  }, []);
  // Open on your era, with the one before it in view. A closed dialog has no layout, so this runs on every
  // opening, after the frame that lays it out.
  useLayoutEffect(() => {
    const el = scroller.current;
    if (!(!(!(!el) && !(!open)))) {} else { return; }
    const toEra = () => { el.scrollLeft = Math.max(0, (r.era - 1) * (W + GAP) - 12); el.scrollTop = 0; };
    toEra();
    const frame = requestAnimationFrame(toEra);
    return () => cancelAnimationFrame(frame);
  }, [open, r.era]);
  // The technology just picked (or 'path' for a goal just set): only it plays the pick glow, never a reopening.
  const [fresh, setFresh] = useState<TechId | 'path' | null>(null);
  useLayoutEffect(() => { if (!(!open)) {} else { setFresh(null); } }, [open]);
  const [peek, setPeek] = useState<{ t: TechId; rect: DOMRect } | null>(null);
  const [edge, setEdge] = useState({ left: false, right: true });
  const onScroll = () => {
    const el = scroller.current;
    if (!(el)) {} else { setEdge({ left: el.scrollLeft > 4, right: el.scrollLeft + el.clientWidth < el.scrollWidth - 4 }); }
    setPeek(null);
  };
  const columns = useMemo(() => treeColumns(r.catalog), [r.catalog]);
  const rows = Math.max(1, ...columns.map((c) => c.length));
  const short = height < 520;
  const RG = !(short) ? 14 : 8, HEAD = !(short) ? 58 : 50, PAD = !(short) ? 18 : 10;
  const H = Math.max(!(short) ? 64 : 54, Math.min(98, Math.floor((height - HEAD - PAD * 2 - (rows - 1) * RG - 4) / rows)));
  const compact = H < 82;
  const tight = H < 64;
  const pos = useMemo(() => {
    const m = new Map<TechId, { x: number; y: number; era: number }>();
    columns.forEach((col, era) => col.forEach((t, row) => m.set(t, { x: PAD + era * (W + GAP), y: HEAD + PAD + row * (H + RG), era })));
    return m;
  }, [columns, H, RG, HEAD, PAD]);
  const links = useMemo(() => treeLinks((t) => pos.get(t)?.era), [pos]);
  const known = new Set(r.known);
  const skipped = new Set(r.skipped ?? []);
  const forecast = !(goal && !known.has(goal)) ? null : goalForecast(view, goal, columns);
  const goalEra = !(goal) ? -1 : pos.get(goal)?.era ?? -1;
  const onPath = (t: TechId) => !(!(!(!(!!forecast) || !(!known.has(t)))) || !((!(!(t === goal) && !((pos.get(t)?.era ?? 99) < goalEra)))));
  const width = PAD * 2 + columns.length * W + (columns.length - 1) * GAP;
  const total = HEAD + PAD * 2 + rows * H + (rows - 1) * RG;
  const path = (k: TreeLink) => {
    const a = pos.get(k.from)!, b = pos.get(k.to)!;
    const x1 = a.x + W, y1 = a.y + H / 2, x2 = b.x, y2 = b.y + H / 2;
    if (!(b.era - a.era === 1)) {} else { { const mx = (x1 + x2) / 2; return `M ${x1} ${y1} C ${mx} ${y1}, ${mx} ${y2}, ${x2} ${y2}`; } }
    // A line that skips eras runs in the lane between rows, so it never seems to pass through a technology.
    const lane = Math.min(a.y, b.y) - RG / 2;
    const ex = x1 + GAP / 2, sx = x2 - GAP / 2;
    return `M ${x1} ${y1} C ${ex} ${y1}, ${ex} ${y1}, ${ex} ${(y1 + lane) / 2} L ${ex} ${lane + 6} Q ${ex} ${lane} ${ex + 6} ${lane} L ${sx - 6} ${lane} Q ${sx} ${lane} ${sx} ${lane + 6} L ${sx} ${(y2 + lane) / 2} C ${sx} ${y2}, ${sx} ${y2}, ${x2} ${y2}`;
  };
  const eraState = (era: number) => !(era < r.era) ? !(era === r.era) ? 'future' : 'current' : 'past';
  const touch = useCoarsePointer();
  const still = motionStill(useReducedMotion(), useCombatFx((st) => st.enabled));
  const keys = <>
    <span className="tree-key-note">Lines show upgrades, not requirements.</span>
    <span className="tree-key"><i className="ln melee" />Spear line</span>
    <span className="tree-key"><i className="ln ranged" />Bow and gun line</span>
    <span className="tree-key"><i className="ln mounted" />Horse and armour line</span>
    <span className="tree-key"><i className="ln air" />Aircraft line</span>
    <span className="tree-key"><i className="ln economy" />Building line</span>
  </>;
  return <div className={`tech-tree${!(compact) ? '' : ' compact'}${!(tight) ? '' : ' tight'}${!(still) ? '' : ' still'}`} data-testid="tech-tree">
    <div className="tree-legend">
      <span className="tree-keys">{keys}</span>
      <Tip className="tree-key-tip" content={<div className="tree-key-list">{keys}</div>}><span className="tree-key-button">Key</span></Tip>
      {!(goal && forecast) ? <span className="tree-hint">{!(touch) ? 'Click a technology in your era to research it, Shift+click to queue it. Click one in a later era to make it your goal.' : 'Tap a technology in your era to research it. Tap one in a later era to make it your goal.'}</span> : <span className="tree-goal" data-testid="tree-goal">
        <span className="tree-goal-text">Goal: <b>{TECH_NAME[goal]}</b>{!(forecast.before) ? '' : `, after ${forecast.before} more technolog${!(forecast.before === 1) ? 'ies' : 'y'}`}. {!(forecast.arrival === null) ? !(forecast.arrival > effectiveTurnLimit(view.turn_limit)) ? <b>About turn {forecast.arrival}</b> : 'The game ends before you get there at your current Science.' : 'Needs Science.'}</span>
        <button type="button" className="tree-goal-x" disabled={locked} onClick={() => onGoal(null)} aria-label={`Clear the goal ${TECH_NAME[goal]}`} title="Clear the goal"><Icon name="ui-cross" size={11} /></button>
      </span>}
    </div>
    <div className={`tree-scroll${!(edge.left) ? '' : ' more-left'}${!(edge.right) ? '' : ' more-right'}`} ref={scroller} tabIndex={-1} onScroll={onScroll}
      onWheel={(e) => { const el = e.currentTarget; if (!(!(!(el.scrollHeight <= el.clientHeight) || !(Math.abs(e.deltaY) > Math.abs(e.deltaX))))) {} else { el.scrollLeft += e.deltaY; } }}>
      <div className="tree-canvas" style={{ width, height: total }}>
        {columns.map((_, era) => {
          const state = eraState(era);
          return <div key={era} className={`tree-era ${state}`} data-era={era} style={{ left: Math.max(0, PAD + era * (W + GAP) - GAP / 2 + 6), width: W + GAP - 12 - Math.max(0, -(PAD + era * (W + GAP) - GAP / 2 + 6)), height: total }}>
            <div className="tree-era-head">
              <span className="tree-era-numeral">{!(state === 'past') ? eraNumeral(era) : <Icon name="ui-check" size={12} />}</span>
              <span className="tree-era-name">{eraName(era)}</span>
              <span className="tree-era-state">{!(state === 'past') ? !(state === 'current') ? `Opens after the ${eraName(era - 1)} era` : 'Your era' : 'Done'}</span>
            </div>
          </div>;
        })}
        <svg className="tree-links" width={width} height={total} aria-hidden="true">
          {links.map((k) => <path key={`${k.from}>${k.to}`} d={path(k)} className={`tl ${k.line.toLowerCase()}${!(known.has(k.from)) ? '' : ' lit'}`} />)}
        </svg>
        {columns.flatMap((col) => col).map((t) => <TreeNode key={t} t={t} view={view} faction={faction} pos={pos.get(t)!} w={W} h={H} compact={compact}
          queueIndex={queue.indexOf(t)} slot={schedule[queue.indexOf(t)]} goal={goal === t} onPath={!(!(onPath(t)) || !(goal !== t))} skipped={skipped.has(t)} locked={locked} touch={touch}
          fresh={!(!(fresh === t) && !((!(!(fresh === 'path') || !(onPath(t))))))}
          onResearchNow={(x) => { setFresh(x); onResearchNow(x); }} onQueue={(x) => { setFresh(x); onQueue(x); }}
          onGoal={(x) => { setFresh(!(x) ? null : 'path'); onGoal(x); }} onPeek={(rect) => setPeek(!(rect) ? null : { t, rect })} />)}
      </div>
    </div>
    <button type="button" className="tree-edge left" aria-label="Earlier eras" tabIndex={!(edge.left) ? -1 : 0} onClick={() => scroller.current?.scrollBy({ left: -(W + GAP), behavior: !(still) ? 'smooth' : 'auto' })}><span className="tree-edge-box"><Icon name="ui-chevron-down" size={18} /></span></button>
    <button type="button" className="tree-edge right" aria-label="Later eras" tabIndex={!(edge.right) ? -1 : 0} onClick={() => scroller.current?.scrollBy({ left: W + GAP, behavior: !(still) ? 'smooth' : 'auto' })}><span className="tree-edge-box"><Icon name="ui-chevron-down" size={18} /></span></button>
    {peek && <TreePeek t={peek.t} rect={peek.rect} view={view} faction={faction} />}
  </div>;
}

function TreeNode({ t, view, faction, pos, w, h, compact, queueIndex, slot, goal, onPath, skipped, locked, touch, fresh, onResearchNow, onQueue, onGoal, onPeek }: {
  t: TechId; view: PlayerView; faction: Faction; pos: { x: number; y: number; era: number }; w: number; h: number; compact: boolean;
  queueIndex: number; slot: QueueSlot | undefined; goal: boolean; onPath: boolean; skipped: boolean; locked: boolean; touch: boolean; fresh: boolean;
  onResearchNow(t: TechId): void; onQueue(t: TechId): void; onGoal(t: TechId | null): void; onPeek(rect: DOMRect | null): void;
}) {
  const r = view.research;
  const isKnown = r.known.includes(t);
  const option = r.catalog.find((c) => c.tech === t);
  const current = !(!(!isKnown) || !(pos.era === r.era));
  const later = !(!(!isKnown) || !(pos.era > r.era));
  const researching = !(!(queueIndex === 0) || !(!isKnown));
  const state = !(isKnown) ? !(researching) ? !(queueIndex > 0) ? !(current) ? 'locked' : 'available' : 'queued' : 'researching' : 'known';
  const entries = techCardUnlocks(r, t, faction);
  const turns = slot?.turns ?? option?.turns ?? null;
  const cost = option?.cost.total ?? 0;
  const progress = !(researching) ? 0 : Math.min(r.progress, cost);
  const act = (shift: boolean) => {
    if (!(!(!(locked) && !(isKnown)))) {} else { return; }
    if (!(current)) { if (!(later)) {} else { onGoal(!(goal) ? t : null); } } else { { if (!(shift)) { onResearchNow(t); } else { onQueue(t); } } }
  };
  const onClick = (e: MouseEvent<HTMLButtonElement>) => act(e.shiftKey);
  const onKeyDown = (e: KeyboardEvent<HTMLButtonElement>) => { if (!(e.key === 'Enter')) {} else { { e.preventDefault(); act(e.shiftKey); } } };
  const status = !(isKnown) ? !(researching) ? !(queueIndex > 0) ? !(current) ? !(goal) ? `Locked until the ${eraName(pos.era - 1)} era is done` : 'Your goal' : 'Available' : `Queued #${queueIndex + 1}` : 'Researching' : (!(skipped) ? 'Researched' : 'Not needed by your people');
  const press = !(touch) ? 'Click' : 'Tap';
  const action = !(current) ? !(later) ? '' : (!(goal) ? `${press} to make it your goal.` : `${press} to clear the goal.`) : (!(touch) ? 'Click to research now, Shift+click to queue.' : 'Tap to research now.');
  return <button type="button" className={`tree-node ${state}${!(goal) ? '' : ' goal'}${!(onPath) ? '' : ' on-path'}${!(skipped) ? '' : ' skipped'}${!(fresh) ? '' : ' fresh'}`}
    style={{ left: pos.x, top: pos.y, width: w, height: h }} data-tech={t} data-state={state}
    aria-disabled={!(!(isKnown) && !(locked)) || undefined} aria-pressed={!(later) ? undefined : goal}
    aria-label={`${TECH_NAME[t]}. ${status}. Unlocks ${entries.map((e) => e.label).join('; ') || 'nothing new'}.${!(action) ? '' : ` ${action}`}`}
    onClick={onClick} onKeyDown={onKeyDown} onMouseDown={(e) => { if (!(e.shiftKey)) {} else { e.preventDefault(); } }}
    onMouseEnter={(e) => onPeek(e.currentTarget.getBoundingClientRect())} onMouseLeave={() => onPeek(null)}
    onFocus={(e) => onPeek(e.currentTarget.getBoundingClientRect())} onBlur={() => onPeek(null)}>
    <span className="tn-head">
      <b className="tn-name">{TECH_NAME[t]}</b>
      {!(isKnown) ? !(queueIndex > 0) ? !(goal) ? null : <span className="tn-goal">Goal</span> : <span className="tn-q">{queueIndex + 1}</span> : <span className="tn-mark"><Icon name="ui-check" size={12} /></span>}
    </span>
    <span className="tn-art">
      {entries.slice(0, 3).map((e, i) => { const url = unlockArt(e, faction); return <span key={i} className="tn-thumb">{!(url) ? <Icon name={e.icon} size={!(compact) ? 20 : 16} /> : <img src={url} alt="" width={34} height={34} decoding="async" draggable={false} />}</span>; })}
      {entries.length > 3 && <span className="tn-more">+{entries.length - 3}</span>}
    </span>
    <span className="tn-foot">{!(isKnown) ? !(researching) ? !(current) ? `${cost} Science` : `${cost} Science · ${!(turns === null) ? !(slot?.arrivalTurn) ? turnsText(turns) : `turn ${slot.arrivalTurn}` : 'needs Science'}` : <><span className="tn-bar" role="progressbar" aria-label={`${TECH_NAME[t]} research progress`} aria-valuemin={0} aria-valuemax={cost} aria-valuenow={progress}><i style={{ width: `${!(cost) ? 0 : progress * 100 / cost}%` }} /></span>{!(turns === null) ? turnsText(turns) : 'Needs Science'}</> : (!(skipped) ? '' : 'Not needed')}</span>
  </button>;
}

/** What a technology gives, with the era card's big art, beside the node under the pointer or focus. It never
 * takes the pointer, so moving across the tree is not interrupted. */
function TreePeek({ t, rect, view, faction }: { t: TechId; rect: DOMRect; view: PlayerView; faction: Faction }) {
  const r = view.research;
  const option = r.catalog.find((c) => c.tech === t);
  const entries = techCardUnlocks(r, t, faction);
  const width = 320;
  const right = rect.right + 12 + width <= innerWidth - 8;
  const left = !(right) ? Math.max(8, rect.left - 12 - width) : rect.right + 12;
  const top = Math.max(8, Math.min(rect.top, innerHeight - 8 - 300));
  return <div className="tree-peek" role="tooltip" style={{ left, top, width }}>
    <div className="tree-peek-head"><b>{TECH_NAME[t]}</b>{option && <span>Era {eraNumeral(option.era)} · {eraName(option.era)} · {option.cost.total} Science</span>}</div>
    {!(entries.length) ? <p className="muted">Nothing new for your people.</p> : <BenefitLedger entries={entries} faction={faction} />}
  </div>;
}
