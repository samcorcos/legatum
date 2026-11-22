// The race panel's holdings (round-2 variant C): per empire, a token for every VP source it holds,
// its own goal as a dashed crest until done, and a last row for what nobody holds. Pointing at or
// focusing a token opens that source's card (its rule and every empire's public number); the arrow
// keys flip through every way to score. Titles are outlined (live, can be lost), kept points solid.
import { createPortal } from 'react-dom';
import { useCallback, useEffect, useLayoutEffect, useRef, useState, type HTMLAttributes, type MouseEvent, type RefObject } from 'react';
import { CrestMedallion, Star, roman } from '../brand/ornaments';
import { DECADENCE_FOOD_RULE, POWER_RULE, decadenceCurveText, decadenceRules, type DecadenceRules } from '../data';
import { Icon } from './bits';
import { RingMedallion } from './ringMedallion';
import { Amount } from './Amounts';
import { sakokuChip, sakokuLine } from '../sakoku';
import { useGame } from '../store';
import { PortraitRoundel } from './premium';
import { SourceCard, empireVars } from './vpParts';
import type { Source, Token, VpEmpire, VpModel, VpRow } from '../vpModel';
import './vpHoldings.css';

/** Hover intent: sweeping the pointer across the panel opens nothing; resting on a token does. */
export const HOVER_INTENT_MS = 120;
const CARD_ID = 'vp-token-card';

interface Active { id: string; el: HTMLElement; pinned: boolean }

/** The one open card, and the props that make a token open it (hover, focus, tap, arrow keys, Esc). */
export function useTokenCards(model: VpModel | null, panel: RefObject<HTMLElement | null>) {
  const [active, setActive] = useState<Active | null>(null);
  const [tabKey, setTabKey] = useState<string | null>(null);
  const timer = useRef<number | undefined>(undefined);
  const lastPointer = useRef<string>('mouse');
  const activeRef = useRef(active);
  const defaultKey = defaultTokenKey(model);
  activeRef.current = active;

  const clear = () => { window.clearTimeout(timer.current); timer.current = undefined; };
  const open = useCallback((id: string, el: HTMLElement, pinned = false) => { clear(); setActive({ id, el, pinned }); }, []);
  const close = useCallback(() => { clear(); setActive(null); }, []);
  useEffect(() => clear, []);

  // A pinned card (a tap) closes on a press anywhere outside the panel or the card.
  useEffect(() => {
    if (!active?.pinned) return;
    const away = (e: PointerEvent) => {
      const t = e.target as Node;
      if (panel.current?.contains(t) || (t instanceof Element && t.closest(`#${CARD_ID}`))) return;
      close();
    };
    document.addEventListener('pointerdown', away, true);
    return () => document.removeEventListener('pointerdown', away, true);
  }, [active?.pinned, close, panel]);

  // The card follows its source out of the model (an eliminated empire's goal, a changed turn): drop it if it is gone.
  useEffect(() => { if (active && !model?.byId.has(active.id)) close(); }, [active, model, close]);

  const flip = (from: string, step: 1 | -1, owner?: string) => {
    const ids = model?.all.map((s) => s.id) ?? [];
    const at = ids.indexOf(from);
    if (at < 0) return;
    const next = ids[(at + step + ids.length) % ids.length];
    const els = [...(panel.current?.querySelectorAll<HTMLElement>(`[data-source="${CSS.escape(next)}"]`) ?? [])];
    // The same empire's token when it has one, else the first token of the source.
    const target = els.find((el) => el.dataset.owner === owner) ?? els[0];
    const folded = target?.closest('details');
    if (folded) folded.open = true;
    target?.focus();
  };

  const tokenProps = (key: string, sourceId: string, owner?: number): HTMLAttributes<HTMLElement> & Record<string, unknown> => ({
    'data-token': key,
    'data-source': sourceId,
    'data-owner': owner === undefined ? undefined : String(owner),
    tabIndex: (tabKey ?? defaultKey) === key ? 0 : -1,
    'aria-describedby': active?.id === sourceId ? CARD_ID : undefined,
    onPointerDown: (e) => { lastPointer.current = e.pointerType; },
    onPointerEnter: (e) => {
      if (e.pointerType === 'touch') return;
      const el = e.currentTarget;
      clear();
      // Warm: once a card is open, moving to the next token swaps it at once.
      if (activeRef.current) open(sourceId, el, false);
      else timer.current = window.setTimeout(() => open(sourceId, el), HOVER_INTENT_MS);
    },
    onPointerLeave: (e) => {
      if (e.pointerType === 'touch') return;
      clear();
      timer.current = window.setTimeout(() => { if (!activeRef.current?.pinned && document.activeElement !== activeRef.current?.el) close(); }, 90);
    },
    onFocus: (e) => { setTabKey(key); open(sourceId, e.currentTarget, activeRef.current?.pinned ?? false); },
    onBlur: (e) => {
      const to = e.relatedTarget as Node | null;
      if (to && panel.current?.contains(to)) return;
      if (!activeRef.current?.pinned) close();
    },
    onClick: (e) => {
      // A tap pins the card (there is no hover); a click leaves the hover card open.
      if (lastPointer.current === 'touch' || lastPointer.current === 'pen') {
        if (activeRef.current?.id === sourceId && activeRef.current.pinned) close();
        else open(sourceId, e.currentTarget, true);
      } else open(sourceId, e.currentTarget, activeRef.current?.pinned ?? false);
    },
    onKeyDown: (e) => {
      if (e.key === 'ArrowRight' || e.key === 'ArrowDown') { e.preventDefault(); flip(sourceId, 1, e.currentTarget.dataset.owner); }
      else if (e.key === 'ArrowLeft' || e.key === 'ArrowUp') { e.preventDefault(); flip(sourceId, -1, e.currentTarget.dataset.owner); }
      else if (e.key === 'Escape' && activeRef.current) { e.preventDefault(); e.stopPropagation(); close(); }
    },
  });

  const reset = () => { close(); setTabKey(null); };
  return { active, tokenProps, close, reset };
}

export const tokenKey = (t: Token) => `${t.owner.id}:${t.source.id}`;

function defaultTokenKey(model: VpModel | null): string | null {
  if (!model) return null;
  const rows = [...model.rows].sort((a, b) => Number(b.isMine) - Number(a.isMine));
  for (const r of rows) {
    const first = r.tokens[0] ?? r.pendingGoals[0];
    if (first) return tokenKey(first);
  }
  return model.open[0] ? `open:${model.open[0].id}` : null;
}

/** The card beside the race panel for the open token: pointer-transparent, never covers the panel. */
export function TokenCard({ model, active, panel }: { model: VpModel | null; active: Active | null; panel: RefObject<HTMLElement | null> }) {
  const ref = useRef<HTMLDivElement>(null);
  const [pos, setPos] = useState<{ top: number; left: number } | null>(null);
  const s = active ? model?.byId.get(active.id) : undefined;
  useLayoutEffect(() => {
    const card = ref.current, p = panel.current;
    if (!active || !card || !p) { setPos(null); return; }
    const anchor = active.el.getBoundingClientRect(), pr = (p.closest('.right-sidebar') ?? p).getBoundingClientRect();
    const hud = document.querySelector('.topbar')?.getBoundingClientRect().bottom ?? 52;
    const h = card.offsetHeight, w = card.offsetWidth;
    const top = Math.max(hud + 8, Math.min(anchor.top - 24, window.innerHeight - h - 8));
    setPos({ top, left: Math.max(8, pr.left - w - 14) });
  }, [active, panel, s]);
  if (!active || !s || !model) return null;
  return createPortal(
    <div ref={ref} id={CARD_ID} className="vhc-pop" role="tooltip" style={pos ? { top: pos.top, left: pos.left } : { top: 0, left: 0, visibility: 'hidden' }}>
      <SourceCard model={model} s={s} />
      <div className="vhc-pop-hint"><kbd className="kbd">←</kbd><kbd className="kbd">→</kbd> every way to score <kbd className="kbd">Esc</kbd></div>
    </div>,
    document.body,
  );
}

type TokenProps = ReturnType<ReturnType<typeof useTokenCards>['tokenProps']>;

const heldLabel = (s: Source, owner?: VpEmpire, of?: VpEmpire) =>
  `${s.name}, +${s.vp} VP, ${s.held ? 'held: can be captured' : s.live ? 'live: can be lost' : 'kept for good'}${owner ? `, ${owner.isMe ? 'yours' : owner.name}` : ''}${s.held && of && of !== owner ? `, ${of.isMe ? 'your' : `${of.name}’s`} goal` : ''}`;

/** One held source. Solid = kept for good, outlined = live (a title, or a live goal). */
export function HeldToken({ t, props }: { t: Token; props: TokenProps }) {
  const { source: s, owner: e } = t;
  // A held goal's token also shows him on the map when he is in view: a click pans there.
  const locate = s.held && s.heldAt ? s.heldAt : undefined;
  const click = props.onClick as ((ev: MouseEvent<HTMLElement>) => void) | undefined;
  return <button type="button" className={`vhc-tok k-${s.kind}${s.live ? ' live' : ''}`} style={empireVars(e)} aria-label={`${heldLabel(s, e, t.of)}${locate ? `. Click to show ${s.faction === 'Mongols' ? 'it' : 'him'} on the map` : ''}`} data-held={s.held || undefined} data-locate={locate ? `${locate.q},${locate.r}` : undefined} {...props}
    onClick={locate ? (ev) => { click?.(ev); useGame.getState().jumpTo(locate); } : click}>
    {s.kind === 'goal' && !s.held ? <CrestMedallion faction={t.of?.faction ?? e.faction} color={t.of?.ink ?? e.ink} size={15} /> : s.kind === 'era' ? <b className="vhc-era-n">{roman(s.era!.era)}</b> : <Icon name={s.icon} size={14} />}
    {s.vp > 1 && <small>+{s.vp}</small>}
  </button>;
}

/** An empire's own goal, not done yet: its crest in a dashed ring. */
export function PendingGoal({ t, props }: { t: Token; props: TokenProps }) {
  const { source: s, owner: e } = t;
  return <span className={`vhc-goal-wrap${e.isMe && s.progress ? ' mine' : ''}`}>
    <button type="button" className="vhc-goal-open" style={empireVars(e)} aria-label={`${s.name}, +${s.vp} VP, not yet${e.isMe ? `, yours, ${s.progress?.label ?? ''}` : `, ${e.name}`}`} {...props}>
      <CrestMedallion faction={e.faction} color={e.ink} size={13} />
    </button>
    {e.isMe && s.progress && <small className="vhc-goal-progress">{s.progress.label}</small>}
  </span>;
}

/** A Great Work or entered era nobody holds yet. */
function OpenToken({ s, props }: { s: Source; props: TokenProps }) {
  return <button type="button" className="vhc-open-tok" aria-label={`${s.name}, +${s.vp} VP, nobody holds it yet`} {...props}>
    {s.kind === 'era' ? <span className="vhc-era-open"><b className="vhc-era-n">{roman(s.era!.era)}</b></span> : <RingMedallion icon={s.icon} tone="hollow" size={19} />}
    {s.vp > 1 && <small>+{s.vp}</small>}
  </button>;
}

export function NobodyRow({ model, tokenProps }: { model: VpModel; tokenProps: ReturnType<typeof useTokenCards>['tokenProps'] }) {
  const worth = model.open.reduce((a, s) => a + s.vp, 0);
  return <div className="vhc-nobody" aria-label={`Nobody yet: ${worth} VP unclaimed`}>
    <span className="vhc-nobody-mark"><Star size={8} /></span>
    <span className="vhc-nobody-who"><span className="vhc-name">Nobody yet</span><span className="vhc-worth"><Amount n={worth} unit="VP" /> open</span></span>
    <span className="vhc-holds">
      {model.open.length
        ? model.open.map((s) => <OpenToken key={s.id} s={s} props={tokenProps(`open:${s.id}`, s.id)} />)
        : <span className="muted vhc-none">every open era and wonder is held</span>}
    </span>
  </div>;
}

/** The plain rule for the capitals victory, said once in the race panel. */
export const CAPITALS_RULE = "Win by holding the last capital: every rival's original capital taken or razed.";

/** Whether a team still holds an original capital of its own: a star and the word, never colour alone. Public, like every row's VP. */
export function CapitalChip({ row, nameOf }: { row: VpRow; nameOf: (e: VpEmpire) => string }) {
  const who = row.isMine ? (row.members.length > 1 ? 'Your team' : 'You') : row.label;
  const detail = row.members.length > 1 ? ` ${row.members.map((m) => `${nameOf(m)}: ${m.capitalStands ? 'standing' : 'fallen'}`).join('; ')}.` : '';
  const title = row.capitalStands
    ? `${who}: an original capital of its own still stands.${detail}`
    : `${who}: every original capital of its own has been taken or razed, so it cannot win by holding the last capital unless it retakes one. It can still win by VP.`;
  return <span className={`vhc-capital${row.capitalStands ? '' : ' fallen'}`} title={title} data-testid="capital-chip" data-team={row.team} data-stands={row.capitalStands ? 'true' : 'false'}
    aria-label={`${who}: original capital ${row.capitalStands ? 'standing' : 'fallen'}`}>
    <Star size={9} />{row.capitalStands ? 'Capital' : 'Capital fallen'}
  </span>;
}

/** A legacy Sakoku scenario's public vow, with its countdown and full rule. */
export function ClosedChip({ members }: { members: VpEmpire[] }) {
  const closed = members.filter((m) => m.sakoku);
  if (!closed.length) return null;
  return <>{closed.map((m) => {
    const st = m.sakoku!;
    const title = `${m.isMe ? 'You have' : `${m.name} has`} closed the country (Sakoku): ${(() => { const l = sakokuLine(st); return l.charAt(0).toLowerCase() + l.slice(1); })()}. While closed it can't declare war or ally, learns nothing from abroad, and nobody may cross its land at Peace.`;
    return <span key={m.id} className={`vhc-closed${st.holding ? ' holding' : st.left === 0 ? ' blocked' : ''}`} title={title} data-testid="closed-chip" data-owner={m.id}
      aria-label={`${m.isMe ? 'Your country is' : `${m.name} is`} closed: ${sakokuLine(st)}`}>
      <Icon name="ui-sakoku" size={11} />{sakokuChip(st)}
    </span>;
  })}</>;
}

/** How far ahead of the average empire a row's empire is ("+60%", neutral: the cost is the red chip beside it), with the one rate it causes in the tooltip. */
export function LeadChip({ members, rules = decadenceRules() }: { members: VpEmpire[]; rules?: DecadenceRules }) {
  if (rules.decadenceSlope === 0) return null;
  const ahead = members.filter((m) => m.lead > 0);
  if (!ahead.length) return null;
  const top = ahead.reduce((a, b) => (b.lead > a.lead ? b : a));
  const says = (m: VpEmpire) => {
    const who = m.isMe ? 'Your' : `${m.name}'s`, its = m.isMe ? 'your' : 'its';
    return `${who} Power is ${m.lead}% above the average empire's. ${m.decadence > 0
      ? `The further ahead an empire is, the more its people grow complacent, so ${its} cities grow ${m.decadence}% slower and lose ${m.decadence}% of their Production, Gold and Science.`
      : `That is too little to cut any yield yet.`}`;
  };
  const title = ahead.map(says).join(' ') + ` ${POWER_RULE} ${decadenceCurveText(rules)}`;
  return <span className="vhc-lead" title={title} data-testid="lead-chip" aria-label={`Power ${top.lead}% above the average empire`}>
    +{top.lead}%
  </span>;
}

/** The rate that lead costs: every yield of the row's empire cut by this percent (a red chip with a down arrow, since colour is never alone). */
export function DecadenceChip({ members, rules = decadenceRules() }: { members: VpEmpire[]; rules?: DecadenceRules }) {
  if (rules.decadenceSlope === 0) return null;
  const cut = members.filter((m) => m.decadence > 0);
  if (!cut.length) return null;
  const top = cut.reduce((a, b) => (b.decadence > a.decadence ? b : a));
  const title = cut.map((m) => `${m.isMe ? 'Your' : `${m.name}'s`} cities grow ${m.decadence}% slower (smaller families) and lose ${m.decadence}% of their Production (sloth), Gold (luxury) and Science (complacency).`).join(' ')
    + ` ${DECADENCE_FOOD_RULE}`;
  return <span className="vhc-decadence" title={title} data-testid="decadence-chip" aria-label={`Decadence: growth ${top.decadence}% slower, Production, Gold and Science cut by ${top.decadence}%`}>
    <Icon name="ui-trend-down" size={9} className="ui-mark" />{top.decadence}%
  </span>;
}

/** What a race row says about the relation with its empire: a pin on the roundel, the words in its tooltip and name. */
export interface RowRelation { kind: 'war' | 'peace' | 'alliance'; declared?: boolean; isNew?: boolean; text: string; title: string }

/** The pin on a roundel: war a red cross, peace a green check, an alliance the house star in gilt. Never colour alone. */
function RelationPin({ r }: { r: RowRelation }) {
  return <span className={`vhc-rel ${r.kind}${r.declared ? ' declared' : ''}${r.isNew ? ' new' : ''}`} title={r.title} data-relation={r.text} aria-hidden="true">
    {r.kind === 'alliance' ? <Star size={8} /> : <Icon name={r.kind === 'war' ? 'ui-cross' : 'ui-check'} size={8} />}
  </span>;
}

/** The roundels of a row's empires, each a button into diplomacy, with the relation pinned. */
export function RowWho({ members, relation, onOpen, nameOf }: { members: VpEmpire[]; relation: RowRelation | null; onOpen: () => void; nameOf: (e: VpEmpire) => string }) {
  return <span className="vhc-who">
    {members.map((m, i) => (
      <button key={m.id} type="button" className="vp-player icon-btn vhc-roundel" aria-label={`Diplomacy with ${nameOf(m)}${m.aiLevel ? ` · AI ${m.aiLevel}` : ''}${relation ? `: ${relation.text}` : ''}`} title={`${m.aiLevel ? `AI · ${m.aiLevel}. ` : ''}${relation ? `${relation.text}. ` : ''}Open diplomacy`} onClick={onOpen}>
        <PortraitRoundel faction={m.faction} size={22} color={m.ink} />
        {relation && i === members.length - 1 && <RelationPin r={relation} />}
      </button>
    ))}
  </span>;
}

/** The countdown wording, one place: "You win in 7 turns" / "Mongols Kin wins in 1 turn" / "wins this turn". */
export function holdWords(row: VpRow, { short = false }: { short?: boolean } = {}): string {
  const left = row.hold!.left;
  const when = left === 0 ? 'this turn' : `in ${left} turn${left === 1 ? '' : 's'}`;
  if (short) return left === 0 ? 'wins now' : `wins in ${left}`;
  return `${row.isMine ? (row.members.length > 1 ? 'Your team wins' : 'You win') : `${row.label} wins`} ${when}`;
}

/** The countdown with the fact first, for a narrow strip that must never cut it: "Wins in 7 turns", "You win in 1 turn". */
function holdHeadline(row: VpRow): string {
  const left = row.hold!.left;
  const when = left === 0 ? 'this turn' : `in ${left} turn${left === 1 ? '' : 's'}`;
  return `${row.isMine ? (row.members.length > 1 ? 'Your team wins' : 'You win') : 'Wins'} ${when}`;
}

/** One pip per turn of the hold: filled for turns held, empty for turns still to hold. */
function HoldMeter({ row }: { row: VpRow }) {
  const h = row.hold!;
  return <span className="vhc-hold-meter" role="progressbar" aria-label={`${row.label}: holds the target ${h.turnsHeld} of ${h.turnsNeeded} turns`} aria-valuemin={0} aria-valuemax={h.turnsNeeded} aria-valuenow={Math.min(h.turnsHeld, h.turnsNeeded)}>
    {Array.from({ length: h.turnsNeeded }, (_, i) => <i key={i} className={i < h.turnsHeld ? 'on' : ''} />)}
  </span>;
}

/**
 * Teams holding their VP target are counting down to the win: the race says so above its rows,
 * in the empire's own colour, a red-ringed alarm when it is a rival's and gilt when it is yours.
 * Two strips at most in the race; the sheet shows them all.
 */
/** `named`: the headline carries the empire's name (a wide strip), instead of leaving it to the line under it. */
export function HoldStrips({ model, limit = 2, named = false }: { model: VpModel; limit?: number; named?: boolean }) {
  if (!model.holds.length) return null;
  const shown = model.holds.slice(0, limit);
  return <div className="vhc-holds-list" role="group" aria-label="Teams holding their victory target">
    {shown.map((row) => (
      <div key={row.team} className={`vhc-hold ${row.isMine ? 'mine' : 'rival'}${row.hold!.left <= 2 ? ' last' : ''}`} role="status" aria-label={`${holdWords(row)}: holding ${row.vp} of ${row.target} VP`} style={empireVars(row.members[0])}>
        <span className="vhc-hold-icon">{row.isMine ? <Star size={9} /> : <Icon name="ui-alert" size={12} className="ui-mark" />}</span>
        <span className="vhc-hold-text">
          <b>{named ? holdWords(row) : holdHeadline(row)}</b>
          <small>{row.isMine || named ? `Holding ${row.vp} of ${row.target} VP` : `${row.label} · holding ${row.vp} of ${row.target} VP`}</small>
        </span>
        <HoldMeter row={row} />
      </div>
    ))}
    {model.holds.length > shown.length && <div className="vhc-hold-more">+{model.holds.length - shown.length} more holding</div>}
  </div>;
}
