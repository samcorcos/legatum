import { useEffect, useId, useLayoutEffect, useRef, useState, type KeyboardEvent, type PointerEvent, type ReactNode } from 'react';
import { createPortal } from 'react-dom';
import type { Faction, ProductionItem } from '../types';
import { unitPanelUrl, unitThumbUrl } from '../unitArt';
import { Amount, Amounts, YieldAmounts } from './Amounts';
import { Icon } from './bits';
import { claimTip, noteTipClosed, releaseTip, tipPinnedElsewhere, tipWarm, type TipSlot } from './Tip';
import type { TechCardUnlock } from './techCardUnlocks';
import { PieceVsBlock } from './PieceVs';
import { unlockHover, type BuildingHover, type EffectHover, type HoverStat, type UnitHover } from './unlockCardModel';
import { unlockEntryArt } from '../unlockArt';
import { WhereToBuildLine } from './WhereToBuild';
import './unlockCard.css';

/** The hovercard waits this long on the first hover so a sweep across the cards shows nothing. */
const UNLOCK_HOVER_DELAY = 160;
const GAP = 10;

type Box = { left: number; top: number; right: number; bottom: number };

/**
 * Where the card goes: beside the tech card it describes (its right, else its left), centred on the hovered row and
 * kept inside the viewport. It never covers the tech card, so it never covers the text it explains.
 */
function placeUnlockCard(card: Box, row: Box, size: { width: number; height: number }, viewport: { width: number; height: number }): { left: number; top: number; side: 'right' | 'left' } {
  const room = { right: viewport.width - 8 - (card.right + GAP), left: card.left - GAP - 8 };
  const side = room.right >= size.width || room.right >= room.left ? 'right' : 'left';
  const left = side === 'right' ? card.right + GAP : card.left - GAP - size.width;
  const centre = (row.top + row.bottom) / 2 - size.height / 2;
  return {
    side,
    left: Math.max(16, Math.min(viewport.width - size.width - 16, left)),
    top: Math.max(16, Math.min(viewport.height - size.height - 16, centre)),
  };
}

type Mode = 'hover' | 'pinned' | null;

/**
 * One row of a tech card's unlock list, with the hovercard that sells it: the large art, what it is for, what it beats
 * and how it compares. Hover (after a short delay) or keyboard focus shows it; a tap or Enter pins it so touch players
 * can read it; Escape or an outside press closes it. A mouse click still reaches the tech card (it researches).
 */
export function UnlockItem({ entry, faction, className, children }: { entry: TechCardUnlock; faction: Faction; className: string; children: ReactNode }) {
  const id = useId();
  const li = useRef<HTMLLIElement>(null);
  const timer = useRef<ReturnType<typeof setTimeout> | null>(null);
  const modeRef = useRef<Mode>(null);
  const lastPointer = useRef<string>('mouse');
  const [mode, setMode] = useState<Mode>(null);
  const [host, setHost] = useState<Element | null>(null);
  const token = useRef<TipSlot>({ close: () => {}, mode: () => null }).current;

  const cancel = () => { if (timer.current) { clearTimeout(timer.current); timer.current = null; } };
  const open = (next: Exclude<Mode, null>) => {
    cancel();
    if (!li.current) return;
    claimTip(token);
    modeRef.current = next;
    setHost(li.current.closest('dialog') ?? document.body);
    setMode(next);
  };
  const close = () => {
    cancel();
    const was = modeRef.current;
    modeRef.current = null;
    setMode(null);
    releaseTip(token);
    if (was === 'hover') noteTipClosed();
  };
  token.close = close;
  token.mode = () => modeRef.current;

  const hoverIn = () => {
    if (modeRef.current || tipPinnedElsewhere(token)) return;
    if (tipWarm(token)) open('hover');
    else { cancel(); timer.current = setTimeout(() => open('hover'), UNLOCK_HOVER_DELAY); }
  };
  const hoverOut = () => { cancel(); if (modeRef.current === 'hover') close(); };

  useEffect(() => {
    if (!mode) return;
    const dismiss = (e: KeyboardEvent | globalThis.KeyboardEvent) => {
      if (e.key !== 'Escape') return;
      close();
      e.preventDefault();
      e.stopPropagation();
    };
    const outside = (e: globalThis.PointerEvent) => {
      if (!li.current?.contains(e.target as Node) && !document.getElementById(id)?.contains(e.target as Node)) close();
    };
    const scrolled = () => close();
    document.addEventListener('keydown', dismiss, true);
    document.addEventListener('pointerdown', outside, true);
    document.addEventListener('scroll', scrolled, true);
    window.addEventListener('resize', scrolled);
    return () => {
      document.removeEventListener('keydown', dismiss, true);
      document.removeEventListener('pointerdown', outside, true);
      document.removeEventListener('scroll', scrolled, true);
      window.removeEventListener('resize', scrolled);
    };
  }, [mode]);
  useEffect(() => () => { cancel(); releaseTip(token); }, [token]);

  return <li ref={li} className={className} tabIndex={0} data-enter-activates aria-describedby={mode ? id : undefined}
    data-unlock-hover={entry.kind.toLowerCase()}
    onPointerEnter={(e: PointerEvent) => { if (e.pointerType === 'mouse') hoverIn(); }}
    onPointerLeave={(e: PointerEvent) => { if (e.pointerType === 'mouse') hoverOut(); }}
    // Pressing is a decision about the card (it researches): the hover steps aside. A touch tap pins instead.
    onPointerDown={(e: PointerEvent) => { lastPointer.current = e.pointerType; cancel(); if (e.pointerType === 'mouse' && modeRef.current === 'hover') close(); }}
    onClick={(e) => {
      if (lastPointer.current === 'mouse') return;
      e.stopPropagation();
      if (modeRef.current === 'pinned') close(); else open('pinned');
    }}
    onFocus={(e) => { if (e.target === e.currentTarget && e.currentTarget.matches(':focus-visible') && !modeRef.current) open('hover'); }}
    onBlur={(e) => { if (!e.currentTarget.contains(e.relatedTarget as Node) && modeRef.current === 'hover') close(); }}
    onKeyDown={(e) => {
      if (e.target !== e.currentTarget || (e.key !== 'Enter' && e.key !== ' ')) return;
      // Enter or Space pins the card open (a pinned card is closed by Escape, the same key or another press).
      e.preventDefault();
      e.stopPropagation();
      if (modeRef.current === 'pinned') close(); else open('pinned');
    }}>
    {children}
    {mode && host && createPortal(<UnlockCard id={id} anchor={li} mode={mode} entry={entry} faction={faction} onClose={close} />, host)}
  </li>;
}

function UnlockCard({ id, anchor, mode, entry, faction, onClose }: { id: string; anchor: { current: HTMLElement | null }; mode: Exclude<Mode, null>; entry: TechCardUnlock; faction: Faction; onClose: () => void }) {
  const ref = useRef<HTMLDivElement>(null);
  const [pos, setPos] = useState<{ left: number; top: number; side: string } | null>(null);
  const hover = unlockHover(entry, faction);
  useLayoutEffect(() => {
    const el = ref.current, row = anchor.current;
    if (!el || !row) return;
    const card = row.closest('.era-card') ?? row;
    setPos(placeUnlockCard(card.getBoundingClientRect(), row.getBoundingClientRect(), el.getBoundingClientRect(), { width: window.innerWidth, height: window.innerHeight }));
  }, [anchor, entry, mode]);
  const pinned = mode === 'pinned';
  return <div ref={ref} id={id} role={pinned ? 'dialog' : 'tooltip'} aria-label={pinned ? `${entry.label}: details` : undefined}
    className={`unlock-card ${pinned ? 'pinned' : 'hovering'}`} data-testid="unlock-card" data-side={pos?.side} data-type={hover.type}
    style={{ left: pos?.left ?? -9999, top: pos?.top ?? -9999, opacity: pos ? 1 : 0 }}>
    {pinned && <button type="button" className="unlock-card-close" aria-label="Close details" onClick={(e) => { e.stopPropagation(); onClose(); }}><Icon name="ui-cross" size={11} /></button>}
    {hover.type === 'unit' && <UnitBody h={hover} faction={faction} />}
    {hover.type === 'building' && <BuildingBody h={hover} icon={entry.icon} art={unlockEntryArt(entry.unlock?.item, entry.label, entry.tech, faction, 256)} item={entry.unlock?.item} />}
    {hover.type === 'effect' && <EffectBody h={hover} faction={faction} art={unlockEntryArt(null, entry.label, entry.tech, faction, 256)} />}
  </div>;
}

/** The large image slot: a framed vignette. Units show their painted miniature; buildings take painted art when it exists. */
function Art({ children, size = 'large' }: { children: ReactNode; size?: 'large' | 'pair' }) {
  return <div className={`uc-art ${size}`} aria-hidden="true">{children}</div>;
}

export function UnitFigure({ kind, faction, size }: { kind: UnitHover['kind']; faction: Faction; size: number }) {
  const url = size > 80 ? unitPanelUrl(kind, faction) : unitThumbUrl(kind, faction);
  return url ? <img className="uc-figure" src={url} width={size} height={size} alt="" /> : <Icon name={`unit-${kind.toLowerCase()}`} size={Math.round(size * 0.6)} />;
}

function StatRow({ s }: { s: HoverStat }) {
  const fmt = (n: number) => <>{n}</>;
  return <div className="uc-stat" data-stat={s.id}>
    <dt>{s.label}</dt>
    <dd>
      {s.from !== undefined && s.from !== s.value && <><span className="uc-from">{fmt(s.from)}</span><span className="uc-arrow" aria-hidden="true">→</span></>}
      <b>{fmt(s.value)}</b>
      {s.delta && <span className={`uc-delta ${s.delta.tone}`}>{s.delta.text}</span>}
    </dd>
  </div>;
}

function UnitBody({ h, faction }: { h: UnitHover; faction: Faction }) {
  return <>
    <div className="uc-top">
      <Art><UnitFigure kind={h.kind} faction={faction} size={150} /></Art>
      <div className="uc-head">
        <h5 className="uc-name">{h.name}</h5>
        <div className="uc-tags">{h.classLabel && <span className="uc-class" data-testid="uc-class">{h.classLabel}</span>}<span className="uc-era">{h.era} era</span></div>
        <p className="uc-flavour" data-testid="uc-flavour">{h.flavour}</p>
      </div>
    </div>
    {h.piece && <PieceVsBlock comparison={h.piece} />}
    {h.matchup && <div className="uc-matchup" data-testid="uc-matchup">
      <p>{h.matchup.line}</p>
      <div className="uc-chips">
        {h.matchup.beats.length > 0 && <span className="uc-chip good"><b>Beats</b> {h.matchup.beats.join(', ')}</span>}
        {h.matchup.weakTo.length > 0 && <span className="uc-chip bad"><b>Weak to</b> {h.matchup.weakTo.join(', ')}</span>}
      </div>
    </div>}
    {h.rule && <p className="uc-rule"><Amounts text={h.rule} /></p>}
    <div className="uc-compare">
      {h.replaces && <div className="uc-vs">Compared with <b>{h.replaces.name}</b></div>}
      <dl className="uc-stats">{h.stats.map((s) => <StatRow key={s.id} s={s} />)}</dl>
    </div>
    {h.replaces && <div className="uc-upgrade" data-testid="uc-upgrade">
      <span className="uc-pair" aria-hidden="true"><UnitFigure kind={h.replaces.kind} faction={faction} size={40} /><span className="uc-arrow">→</span><UnitFigure kind={h.kind} faction={faction} size={40} /></span>
      <span className="uc-upgrade-text">{h.upgradeCost !== null
        ? <>Upgrade from {h.replaces.name}: <Amount n={h.upgradeCost} unit="Gold" /> in your territory</>
        : <>Upgrades from {h.replaces.name}</>}</span>
    </div>}
    {h.locked ? <div className="uc-foot"><span className="uc-locked" data-testid="uc-locked">{h.locked}</span></div> : <div className="uc-foot">
      <span><span className="uc-foot-label">Build</span> <Amount n={h.cost} unit="Production" /></span>
      {h.buyCost !== null && <span><span className="uc-foot-label">Buy</span> <Amount n={h.buyCost} unit="Gold" /></span>}
    </div>}
  </>;
}

function BuildingBody({ h, icon, art, item }: { h: BuildingHover; icon: string; art: string | null; item?: ProductionItem }) {
  return <>
    <div className="uc-top">
      <Art>{art ? <img className="uc-figure uc-hero" src={art} alt="" /> : <span className="uc-glyph"><Icon name={icon} size={84} /></span>}</Art>
      <div className="uc-head">
        <h5 className="uc-name">{h.name}</h5>
        <div className="uc-tags"><span className="uc-class">{h.tag}</span>{h.upgradesFrom && <span className="uc-era">Upgrades {h.upgradesFrom}</span>}</div>
        <p className="uc-flavour" data-testid="uc-flavour">{h.flavour}</p>
      </div>
    </div>
    {h.piece && <PieceVsBlock comparison={h.piece} />}
    <div className="uc-gives" data-testid="uc-gives">
      <div className="uc-vs">What it gives</div>
      <p className="uc-gives-line">{h.yields.length > 0 ? <YieldAmounts yields={Object.fromEntries(h.yields)} /> : <Amounts text={h.rule} />}{h.yields.length > 0 && h.cityStrength > 0 ? `, +${h.cityStrength} city strength` : ''}</p>
      <p className="uc-why" data-testid="uc-why"><b>Why build it:</b> {h.why}</p>
    </div>
    {item && !h.locked && <WhereToBuildLine item={item} />}
    {h.locked ? <div className="uc-foot"><span className="uc-locked" data-testid="uc-locked">{h.locked}</span></div> : <div className="uc-foot">
      <span><span className="uc-foot-label">Build</span> <Amount n={h.cost} unit="Production" /></span>
      {h.buyCost !== null ? <span><span className="uc-foot-label">Buy</span> <Amount n={h.buyCost} unit="Gold" /></span> : <span><span className="uc-foot-label">Buy</span> not for sale</span>}
      {h.minPopulation ? <span><span className="uc-foot-label">Needs</span> population {h.minPopulation}</span> : null}
    </div>}
  </>;
}

function EffectBody({ h, faction, art }: { h: EffectHover; faction: Faction; art: string | null }) {
  return <>
    <div className="uc-top effect">
      {h.upgrade
        ? <Art size="pair"><span className="uc-pair big"><span className="uc-pair-unit"><UnitFigure kind={h.upgrade.from.kind} faction={faction} size={112} /><small>{h.upgrade.from.name}</small></span><span className="uc-arrow big" aria-hidden="true">→</span><span className="uc-pair-unit"><UnitFigure kind={h.upgrade.to.kind} faction={faction} size={112} /><small>{h.upgrade.to.name}</small></span></span></Art>
        : <Art>{art ? <img className="uc-figure uc-hero" src={art} alt="" /> : <span className="uc-glyph"><Icon name={h.icon} size={72} /></span>}</Art>}
      <div className="uc-head">
        <h5 className="uc-name effect-text"><Amounts text={h.text} /></h5>
        <div className="uc-tags"><span className="uc-class">Effect</span></div>
        <p className="uc-why" data-testid="uc-why">{h.why}</p>
      </div>
    </div>
  </>;
}
