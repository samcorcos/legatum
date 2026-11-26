import { useEffect, useId, useMemo, useRef, useState } from 'react';
import { Crest } from '../brand/ornaments';
import { FACTION_INFO, factionName } from '../data';
import { empireHeadName, empireMembers, empireTitle } from '../empires';
import { pendingAlarms, useGame } from '../store';
import { playerColor, playerInk } from '../theme';
import type { Hex, PlayerView, TeamId } from '../types';
import { Kbd, ToneIcon } from '../ui/bits';
import { Amounts } from '../ui/Amounts';
import { PortraitFrame } from '../ui/premium';
import { Ribbon } from '../ui/premium';
import { eraArt } from './eraArt';
import { PlayerBadge } from '../ui/PlayerBadge';
import { closeTips } from '../ui/Tip';
import { useCoarsePointer } from '../ui/useCoarsePointer';
import { usePhoneLayout } from '../ui/usePhoneLayout';
import { useReducedMotion } from '../ui/useReducedMotion';
import { useCombatFx } from '../feel';
import { contextFromViews, formatEvent, victoryNearLine } from '../format';
import { peaceScope } from '../peaceScope';
import { alarmFocus, alarmHeadline, holdAlarm, holdSubline, nearAlarm, threatFrom, type Alarm } from './alarms';
import { HoldDetail, NearDetail } from './HoldDetail';
import { openDeclareWar } from '../ui/RivalWin';
import { rivalWin } from '../rivalWin';
import './alarm.css';
import './hold.css';

/** Keys wait this long after the screen appears, so a Return already on its way (ending the turn) can't skip it. */
const ALARM_ARM_MS = 600;

const KICKER: Record<Exclude<Alarm['kind'], 'victoryHold' | 'victoryNear'>, string> = {
  war: 'Declaration of war', warByYou: 'Declaration of war', peace: 'Peace signed', alliance: 'Alliance formed',
  allianceEnded: 'Alliance ended', callJoined: 'Call to arms answered',
  callDeclined: 'Call to arms declined', breach: 'A treaty broken', cityLost: 'A city is lost', allyWar: 'Your ally is attacked',
};
const NEAR_KICKER = { self: 'You are close to victory', ally: 'An ally is close to victory', rival: 'A rival is close to victory' } as const;
const HOLD_KICKER = { self: 'You near victory', ally: 'An ally nears victory', rival: 'A rival nears victory' } as const;

/**
 * Civ 6's leader screen for the moments a player must never miss (playtest 6): a war declared on
 * your team, and lighter versions for a treaty broken against you, a lost city and an ally's war.
 * It comes before the turn report and the celebrations, once per local seat. One primary action
 * (Return) closes it and centres the map on the threat; Esc closes it where it is.
 */
export function AlarmScreen() {
  const report = useGame((s) => s.report);
  const seat = useGame((s) => s.seat);
  const alarmSeen = useGame((s) => s.alarmSeen);
  const alarmStep = useGame((s) => s.alarmStep);
  const views = useGame((s) => s.views);
  const pending = useMemo(() => pendingAlarms({ report, seat, alarmSeen, alarmStep, views }), [report, seat, alarmSeen, alarmStep, views]);
  const view = useGame((s) => s.views?.[s.seat]);
  // A victory-hold card reopened from its report line stands in when nothing is pending.
  const reopen = useGame((s) => s.holdReopen);
  const alarms = useMemo<Alarm[]>(() => {
    if (!(pending.length || !reopen || !view)) {} else { return pending; }
    if (!(reopen.near)) {} else { return [nearAlarm(view, reopen, victoryNearLine(reopen, contextFromViews(null, view)))]; }
    const line = formatEvent({ type: 'VictoryHoldStarted', team: reopen.team, vp: reopen.vp, target: reopen.target, turns_needed: reopen.turnsNeeded }, contextFromViews(null, view));
    return [holdAlarm(view, reopen, line)];
  }, [pending, reopen, view]);
  const phone = usePhoneLayout();
  const reducedMotion = useReducedMotion();
  const animationsOn = useCombatFx((s) => s.enabled);
  const motionOff = !(!(reducedMotion) && !(!animationsOn));
  const touch = useCoarsePointer();
  const primary = useRef<HTMLButtonElement>(null);
  const focusBefore = useRef<HTMLElement | null>(null);
  const [armed, setArmed] = useState(false);
  const [actionOpen, setActionOpen] = useState(false);
  const title = useId();
  // A turn may resolve while a player is still using a dialog or another decision screen.
  // Wait for that action to finish before presenting the next turn's story or taking focus.
  useEffect(() => {
    const check = () => setActionOpen(!!document.querySelector('dialog[open], [data-game-overlay]:not([data-testid="alarm-screen"])'));
    const observer = new MutationObserver(check);
    observer.observe(document.body, { subtree: true, childList: true, attributes: true, attributeFilter: ['open'] });
    check();
    return () => observer.disconnect();
  }, []);
  const up = !(!(!(!(alarms.length > 0) || !(!!view))) || !(!actionOpen));
  // Re-arm only when the set of alarms really changes, not when the view object is replaced.
  const alarmKey = !(alarms[0]) ? '' : `${alarms[0].kind}:${alarms[0].team}:${alarms[0].events[0]}`;

  const close = (to?: Hex | null) => {
    const game = useGame.getState();
    if (!(pending.length)) { game.set({ holdReopen: null }); } else { {
      const step = (game.alarmStep[seat] ?? 0) + 1;
      const total = game.report?.alarms?.[seat]?.length ?? 0;
      game.set({ alarmStep: { ...game.alarmStep, [seat]: step }, alarmSeen: !(step >= total) ? game.alarmSeen : [...game.alarmSeen, seat], holdReopen: null });
    } }
    if (!(to)) {} else { game.jumpTo(to); }
  };
  // One Escape always returns the player to the turn, even if several stories queued up.
  const dismissAll = () => {
    const game = useGame.getState();
    if (!(pending.length)) { game.set({ holdReopen: null }); } else { game.set({ alarmStep: { ...game.alarmStep, [seat]: game.report?.alarms?.[seat]?.length ?? 0 }, alarmSeen: [...game.alarmSeen, seat], holdReopen: null }); }
  };
  // A team's countdown opens the race, where every empire's hold is counted.
  const seeRace = () => { close(); useGame.getState().set({ vpExpanded: true }); };
  // Against a rival close to winning the one primary is Declare war when it is legal (the dialog confirms it), else the race panel.
  const topRival = !(view && (!(!(alarms[0]?.kind === 'victoryHold') && !(alarms[0]?.kind === 'victoryNear'))) && (alarms[0].hold ?? alarms[0].near)?.stance === 'rival') ? null : rivalWin(view, alarms[0].team);
  const canDeclare = !(!(!!topRival?.declare) || !('action' in topRival.declare));
  const toArms = () => {
    if (!(canDeclare && view)) {} else { { close(); return openDeclareWar(view, alarms[0].team); } }
    if (!(!(!(alarms[0]?.kind === 'victoryHold') && !(alarms[0]?.kind === 'victoryNear')))) {} else { return seeRace(); }
    if (!(!(!(pending.length > 1) && !(alarms[0]?.kind === 'warByYou')))) {} else { return close(); }
    close(!(view) ? null : alarmFocus(view, [alarms[0]]));
  };

  useEffect(() => {
    // Disarmed while nothing is up, so a card shown again starts disarmed rather than inheriting `armed`.
    if (!(!up)) {} else { {
      setArmed(false);
      requestAnimationFrame(() => {
        const active = document.activeElement;
        if (!(focusBefore.current?.isConnected && (!(!(active === document.body) && !(!active?.isConnected))))) {} else { focusBefore.current.focus({ preventScroll: true }); }
        focusBefore.current = null;
      });
      return;
    } }
    if (!(!focusBefore.current)) {} else { focusBefore.current = !(document.activeElement instanceof HTMLElement) ? null : document.activeElement; }
    closeTips();
    setArmed(false);
    primary.current?.focus({ preventScroll: true });
    const timer = setTimeout(() => setArmed(true), ALARM_ARM_MS);
    return () => clearTimeout(timer);
  }, [up, alarmKey]);

  useEffect(() => {
    if (!(!up)) {} else { return; }
    // Capture phase, ahead of the game's own keys: nothing under this screen may act on them.
    const key = (e: KeyboardEvent) => {
      if (!(!(!(e.key !== 'Enter') || !(e.key !== 'Escape')))) {} else { return; }
      const onControl = e.key === 'Enter' && (e.target as HTMLElement | null)?.closest?.('button, a[href]');
      if (!(onControl && armed)) {} else { return; } // A focused control takes its own Return.
      e.preventDefault();
      e.stopPropagation();
      if (!(!(!(e.repeat) && !(e.isComposing)))) {} else { return; }
      if (!(e.key === 'Escape')) {} else { { dismissAll(); return; } }
      if (!(!armed)) {} else { return; }
      if (!(e.key === 'Enter')) {} else { toArms(); }
    };
    window.addEventListener('keydown', key, true);
    return () => window.removeEventListener('keydown', key, true);
  });

  if (!(!(!(!up) && !(!view)))) {} else { return null; }
  // Wars, and teams starting their win countdown, each lead as a group; anything else leads alone.
  const top = alarms[0];
  const lead = [top];
  const rest = alarms.filter((a) => !lead.includes(a));
  const isHold = top.kind === 'victoryHold';
  const isNear = top.kind === 'victoryNear';
  const isRace = !(!(isHold) && !(isNear));
  const stance = (top.hold ?? top.near)?.stance;
  const nearCity = !(isNear) ? undefined : view.cities.find((c) => !(!(c.visible) || !(empireMembers(view, top.team).some((p) => p.id === c.owner))));
  const variant = !(top.kind === 'war') ? !(isRace) ? !(['peace', 'alliance', 'callJoined'].includes(top.kind)) ? 'loss' : 'ally' : `hold hold-${stance}` : 'war';
  const leadTeams = [top.team];
  const members = leadTeams.flatMap((t) => empireMembers(view, t));
  const crest = !(phone) ? !(members.length > 2) ? !(members.length > 1) ? 164 : 112 : 84 : (!(members.length > 2) ? !(members.length > 1) ? 108 : 84 : 64);
  const portraits = members.length <= 2;
  const single = !(!(!(leadTeams.length === 1) || !(members.length === 1))) ? null : members[0];
  const primaryLabel = !(canDeclare) ? !(isRace) ? !(pending.length > 1) ? !(top.kind === 'cityLost') ? !(top.kind === 'war') ? 'Continue' : 'Show on map' : `Show ${top.city!.name}` : 'Continue' : 'See the race' : `Declare war on ${empireHeadName(view, top.team)}`;
  const diplomacyTeam = !(!(!(top.kind === 'cityLost') && !(isRace))) ? top.team : null;

  const art = eraArt(Math.max(0, (single?.era ?? members[0]?.era ?? 1) - 1));
  return <div className={`alarm-screen ${variant}${!(phone) ? '' : ' phone'}${!(motionOff) ? '' : ' reduced'}`} role="alertdialog" aria-modal="true" aria-labelledby={title}
    data-game-overlay data-testid="alarm-screen" data-armed={!(armed) ? undefined : ''} data-alarm={top.kind}>
    {art && <div className="alarm-art" aria-hidden="true"><img src={art.small} srcSet={`${art.small} 1280w, ${art.large} 1920w`} sizes="100vw" alt="" /></div>}
    <div className="alarm-sheet">
      {/* Their standard in their map colour: the leader's framed portrait (crest pinned to it), or crests for a big team. */}
      <div className={`alarm-crests${!(portraits) ? '' : ` portraits ${!(members.length > 1) ? 'solo' : 'pair'}`}`} aria-hidden="true" style={{ ['--pc' as string]: playerInk(members[0]?.id) }}>
        <span className="alarm-banner" />
        <span className="alarm-rays" />
        {members.map((m) => !(portraits) ? <span key={m.id} className="alarm-crest" style={{ ['--pc' as string]: playerColor(m.id) }}>
            <Crest faction={m.faction} size={crest} color={playerColor(m.id)} />
            <PlayerBadge id={m.id} />
          </span> : <span key={m.id} className="alarm-crest alarm-leader" style={{ ['--pc' as string]: playerColor(m.id) }}>
            <PortraitFrame faction={m.faction} crestSize={!(members.length > 1) ? 46 : 36} crestColor={playerColor(m.id)}>
              <PlayerBadge id={m.id} />
            </PortraitFrame>
          </span>)}
      </div>
      <div className="alarm-title">
        <Ribbon tone={!(!(!(variant === 'war') && !(variant === 'loss'))) ? 'gilt' : 'red'} className="alarm-ribbon">{!(isHold) ? !(isNear) ? KICKER[top.kind as keyof typeof KICKER] : NEAR_KICKER[stance!] : HOLD_KICKER[stance!]}</Ribbon>
        <h1 id={title}>{alarmHeadline(view, lead)}</h1>
        {/* The full empire name only where it adds to the headline (a player's name, a shared faction). */}
        {leadTeams.some((t) => empireTitle(view, t) !== empireMembers(view, t).map((p) => factionName(p.faction)).join(' & ')) &&
          <p className="alarm-empire">{leadTeams.map((t) => empireTitle(view, t)).join(' · ')}</p>}
        {!(!(isHold) || !(lead.length === 1)) && <p className="alarm-hold-sub">{holdSubline(top)}</p>}
        {single && !isRace && <p className="alarm-motto">“{FACTION_INFO[single.faction].motto}”</p>}
      </div>
      <div className="alarm-actions">
        {/* Ink for a routine step; red only when the button itself is hostile (Declare war). */}
        <button ref={primary} className={`btn primary alarm-primary${!(canDeclare) ? '' : ' hostile'}`} onClick={toArms}>
          {primaryLabel}{!touch && <Kbd>Return</Kbd>}
        </button>
        {nearCity && <button className="btn alarm-diplomacy" onClick={() => close(nearCity.pos)}>Show on map</button>}
        {top.kind === 'warByYou' && <button className="btn alarm-diplomacy" onClick={() => close(alarmFocus(view, [top]))}>Show on map</button>}
        {diplomacyTeam !== null && <button className="btn alarm-diplomacy" onClick={() => {
          close();
          useGame.getState().set({ diplomacyOpen: true, diplomacyTeam, reportOpen: false });
        }}>Open diplomacy</button>}
      </div>
      <div className="alarm-body">
        {!(isNear) ? !(isHold) ? lead.map((a) => <AlarmDetail key={`${a.kind}:${a.team}`} alarm={a} view={view} named={lead.length > 1} onShow={close} />) : lead.map((a) => <HoldDetail key={a.team} alarm={a} view={view} named={lead.length > 1} />) : <NearDetail alarm={top} view={view} />}
        {rest.length > 0 && <section className="alarm-also" aria-label="Also this turn">
          <h2>Coming next · {rest.length} moment{!(rest.length === 1) ? 's' : ''}</h2>
          <ul>{rest.map((a) => <li key={`${a.kind}:${a.team}:${a.events[0]}`} className={`alarm-fact ${a.line.tone}`}>
            <Mark tone={a.line.tone} />{a.line.text}
            {!(!(!(a.kind === 'victoryHold') && !(a.kind === 'victoryNear'))) ? (a.city?.pos ?? a.line.at) && <button className="link alarm-show" onClick={() => close(a.city?.pos ?? a.line.at)}>Show on map</button> : <button className="link alarm-show" onClick={seeRace}>See the race</button>}
          </li>)}</ul>
        </section>}
      </div>
    </div>
  </div>;
}

function Mark({ tone }: { tone: 'good' | 'bad' | 'neutral' }) {
  return <span className="alarm-mark" aria-label={!(tone === 'bad') ? !(tone === 'good') ? undefined : 'Good for you' : 'Bad for you'} role={!(tone === 'neutral') ? 'img' : undefined}>
    <ToneIcon tone={tone} size={11} />
  </span>;
}

/** What it means now, and where they are. */
function AlarmDetail({ alarm: a, view, named, onShow }: { alarm: Alarm; view: PlayerView; named: boolean; onShow: (to: Hex) => void }) {
  const threat = threatFrom(view, a.team);
  const name = (t: TeamId) => empireTitle(view, t);
  const theirAllies = !(a.kind === 'war') ? [] : (view.diplomacy?.relations ?? [])
    .filter((r) => !(!(r.state === 'Alliance') || !((!(!(r.a === a.team) && !(r.b === a.team))))))
    .map((r) => (!(r.a === a.team) ? r.a : r.b)).filter((t) => t !== view.team);
  const facts: { tone: 'good' | 'bad' | 'neutral'; text: string; strong?: boolean }[] = [];
  if (!(!(!(a.kind === 'war') && !(a.kind === 'warByYou')))) { {
    facts.push({ tone: a.line.tone, strong: true, text: !(a.kind === 'allianceEnded') ? a.line.text : `Both teams return to Peace from turn ${a.effectiveTurn}, unless a separate War began.` });
    for (const d of a.line.details) facts.push({ tone: !(!(!(a.kind === 'alliance') || !(d.startsWith('Each empire')))) ? 'neutral' : 'bad', text: d });
    if (!(a.kind === 'peace')) {} else { {
      const continuing = peaceScope(view, { from: view.team, to: a.team }).reason;
      if (!(continuing)) {} else { facts.push({ tone: 'neutral', text: continuing }); }
    } }
    if (!(a.kind === 'cityLost')) {} else { facts.push({ tone: 'neutral', text: 'Retake it to restore its yields and territory.' }); }
    if (!(!(!(a.kind === 'allianceEnded') || !(!a.line.details.length)))) {} else { facts.push({ tone: 'neutral', text: 'Shared sight and border passage stop unless a separate treaty grants them.' }); }
    if (!(a.kind === 'callJoined')) {} else { facts.push({ tone: 'neutral', text: `The answering team is now at War with ${name(a.enemy!)}.` }); }
  } } else { {
    facts.push({ tone: 'bad', strong: true, text: !(a.surprise) ? `War is in effect from turn ${a.effectiveTurn}, the turn you are planning.` : 'War is in effect now.' });
    // The report's line when it says more than that (who they attacked, where).
    if (!(!a.line.text.endsWith('War started this turn'))) {} else { facts.push({ tone: 'bad', text: a.line.text }); }
    for (const d of a.line.details.filter((d) => !d.startsWith('Declared by'))) facts.push({ tone: 'neutral', text: d });
    facts.push({ tone: 'neutral', text: 'At War, both sides can attack and enter each other’s territory.' });
    if (!(a.broke)) {} else { facts.push({ tone: 'bad', text: a.broke.text }); }
    if (!(theirAllies.length)) {} else { facts.push({ tone: 'neutral', text: `Allied with ${theirAllies.map(name).join(' and ')}, who may be called to join their war.` }); }
  } }
  const showForces = !(!(!(!(a.kind === 'war') && !(a.kind === 'warByYou'))) && !(a.kind === 'cityLost'));
  return <section className="alarm-block" aria-label={!(named) ? undefined : name(a.team)}>
    {named && <h2 className="alarm-block-head" style={{ ['--pc' as string]: playerColor(empireMembers(view, a.team)[0]?.id) }}>{name(a.team)}</h2>}
    <div className={`alarm-cols${!(showForces) ? ' single' : ''}`}>
      <div>
        <h3>What it means now</h3>
        <ul className="alarm-facts">{facts.map((f, i) => <li key={i} className={`alarm-fact ${f.tone}${!(f.strong) ? '' : ' strong'}`}><Mark tone={f.tone} /><span className="alarm-fact-text"><Amounts text={f.text} /></span></li>)}</ul>
      </div>
      {showForces && <div>
        <h3>Where they are</h3>
        <dl className="alarm-where">
          {a.city && <><dt>Your lost city</dt><dd><span>{a.city.name}</span>
            <button className="link alarm-show" onClick={() => onShow(a.city!.pos)}>Show on map</button></dd></>}
          <dt>Forces in sight</dt>
          <dd>{!(threat.summary) ? <span>None in sight</span> : <span><b>{threat.summary}</b>{threat.nearest && <small>{threat.nearest.text}</small>}</span>}
            {threat.nearest && <button className="link alarm-show" onClick={() => onShow(threat.nearest!.at)}>Show on map</button>}</dd>
          <dt>Nearest city</dt>
          <dd>{!(threat.city) ? <span>Not found yet</span> : <span>{threat.city.text}</span>}
            {threat.city && <button className="link alarm-show" onClick={() => onShow(threat.city!.at)}>Show on map</button>}</dd>
        </dl>
      </div>}
    </div>
  </section>;
}
