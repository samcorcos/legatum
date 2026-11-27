import { useEffect, useRef, useState, type ReactNode } from 'react';
import { clampDealGold, dealsOn, offerGoldForecast, withDealGold } from '../deals';
import { incomingOffers, queueDiplomacy, relationWith, withLocalDiplomacy } from '../diplomacy';
import { empireMembers, empireTitle } from '../empires';
import { useGame } from '../store';
import { pendingDecisions } from '../turnFlow';
import type { DiplomaticAction, PeaceProposal, PlayerView, TeamId } from '../types';
import { DealGold, goldTerm, offerForecast, offerHeadline, respondToOffer, warLines } from '../ui/DiplomacyPanel';
import { PortraitFrame } from '../ui/premium';
import { peaceScope } from '../peaceScope';
import { allianceTerms } from '../treaties';
import { useCelebration } from './MomentLane';
import { eraArt } from './eraArt';
import { PEACE_OFFER, WAR_REPLY } from './greetings';
import './peaceTalks.css';

/** Everything on the plate comes from the current seat's public view. */
export function warFacts(view: PlayerView, team: TeamId): string[] {
  const relation = relationWith(view, team);
  const owns = (id: number, t: number) => view.players.find((p) => p.id === id)?.team === t;
  const held = view.cities.filter((c) => c.visible && c.founder !== undefined && owns(c.owner, view.team) && owns(c.founder, team));
  const lost = view.cities.filter((c) => c.visible && c.founder !== undefined && owns(c.owner, team) && owns(c.founder, view.team));
  return [
    ...(relation?.state === 'War' ? [`At War since turn ${relation.effective_turn}`] : []),
    ...held.map((c) => `You hold ${c.name}, once theirs`),
    ...lost.map((c) => `${empireTitle(view, team)} holds ${c.name}, once yours`),
  ];
}

export function armyLine(view: PlayerView, team: TeamId, extras: TeamId[] = []): string | null {
  const army = (t: TeamId) => {
    const members = view.players.filter((p) => p.team === t && p.alive);
    return members.length && members.every((p) => p.army !== undefined)
      ? members.reduce((n, p) => n + p.army!, 0) : null;
  };
  if (!view.standings.some((s) => s.team === view.team) || !view.standings.some((s) => s.team === team)) return null;
  const ours = army(view.team), theirs = army(team);
  if (ours === null || theirs === null) return null;
  const more = [...new Set(extras)].filter((t) => t !== view.team && t !== team).flatMap((t) => {
    const points = army(t);
    return points !== null && view.standings.some((s) => s.team === t) ? [`${empireTitle(view, t)} ${points}`] : [];
  });
  return `Army points: you ${ours}, ${empireTitle(view, team)} ${theirs}${more.length ? `, ${more.join(', ')}` : ''}`;
}

function Plate({ view, team, mode, children, onClose }: { view: PlayerView; team: TeamId; mode: 'offer' | 'terms' | 'war'; children: ReactNode; onClose(): void }) {
  const lead = empireMembers(view, team)[0];
  const mine = view.players.find((p) => p.id === view.me);
  const art = eraArt((mine?.era ?? 1) - 1);
  useEffect(() => {
    if (!lead) return;
    useGame.setState({ meetingPeople: lead.faction });
    return () => useGame.setState({ meetingPeople: null });
  }, [lead?.faction]);
  useEffect(() => {
    const key = (e: KeyboardEvent) => {
      if (e.key !== 'Escape' || e.defaultPrevented) return;
      e.preventDefault(); e.stopPropagation(); onClose();
    };
    window.addEventListener('keydown', key, true);
    return () => window.removeEventListener('keydown', key, true);
  }, [onClose]);
  return <div className={`era-takeover peace-talks ${mode}`} role="dialog" aria-modal="true" aria-label={mode === 'war' ? `Declare War on ${empireTitle(view, team)}?` : `Peace talks with ${empireTitle(view, team)}`}
    data-testid={mode === 'war' ? 'war-takeover' : 'peace-talks'} data-game-overlay>
    <div className="et-art" aria-hidden="true">{art && <img src={art.small} srcSet={`${art.small} 1280w, ${art.large} 1920w`} sizes="100vw" alt="" />}</div>
    <div className="pt-wash" aria-hidden="true" />
    <div className="pt-layout">{children}</div>
  </div>;
}

function People({ view, team, own = false }: { view: PlayerView; team: TeamId; own?: boolean }) {
  const members = own ? view.players.filter((p) => p.id === view.me) : empireMembers(view, team).slice(0, 2);
  if (!members.length) return null;
  return <div className="pt-people">
    <div className="pt-portraits">{members.map((member) => <PortraitFrame key={member.id} faction={member.faction} className="pt-portrait" crestSize={36} />)}</div>
    <span className="pt-banner">{empireTitle(view, team)}{own ? ' · You' : ''}</span>
  </div>;
}

function Top({ view, team, war = false }: { view: PlayerView; team: TeamId; war?: boolean }) {
  const facts = warFacts(view, team);
  return <header className="pt-top">
    <span className="pt-kicker">{war ? 'A declaration of War' : 'Peace talks'} · Turn {view.turn}</span>
    {facts.length > 0 && <div className="pt-facts">{facts.map((fact) => <span key={fact}>{fact}</span>)}</div>}
  </header>;
}

function OfferPlate({ view, proposal, close }: { view: PlayerView; proposal: PeaceProposal; close(): void }) {
  const primary = useRef<HTMLButtonElement>(null);
  useEffect(() => { primary.current?.focus({ preventScroll: true }); }, [proposal.id]);
  const team = proposal.from;
  const actions = view.diplomacy.actions.filter((a) => a.order.type === 'RespondToProposal' && a.order.proposal === proposal.id);
  const accept = actions.find((a) => a.order.type === 'RespondToProposal' && a.order.accept);
  const decline = actions.find((a) => a.order.type === 'RespondToProposal' && !a.order.accept);
  const forecast = offerForecast(proposal, view, accept);
  const gold = goldTerm(proposal, view)[0];
  const lead = empireMembers(view, team)[0];
  const respond = (action: DiplomaticAction | undefined) => {
    if (!action || action.blocked_reason) return;
    const s = useGame.getState();
    respondToOffer(proposal, action, withLocalDiplomacy(view, s.humans, s.orders));
    close();
  };
  return <Plate view={view} team={team} mode="offer" onClose={close}>
    <Top view={view} team={team} />
    <main className="pt-main"><People view={view} team={team} />
      <div className="pt-content">
        <h1>{offerHeadline(proposal, view)}</h1>
        {lead && <blockquote>“{PEACE_OFFER[lead.faction]}”</blockquote>}
        <div className="pt-ledger">
          <section><h2>{empireTitle(view, team)} gives</h2><p>Peace, guaranteed 10 turns</p>{gold?.tone === 'gain' && <p className="gain">{gold.text}</p>}</section>
          <section><h2>You give</h2><p>Peace, guaranteed 10 turns</p>{gold?.tone === 'cost' && <p className="cost">{gold.text}</p>}</section>
        </div>
        {forecast.filter((line) => line.icon !== 'yield-gold' && !line.text.startsWith('Peace guaranteed') && line.text !== 'Costs no Gold').map((line) => <p className="pt-detail" key={line.text}>{line.text}</p>)}
        <div className="pt-actions">
          {accept && !accept.blocked_reason && <button ref={primary} className="btn primary" onClick={() => respond(accept)}>Accept Peace</button>}
          {decline && <button className="btn quiet" onClick={() => respond(decline)}>Refuse, fight on</button>}
          <button ref={!accept || accept.blocked_reason ? primary : undefined} className="btn quiet" onClick={close}>Decide later</button>
        </div>
        {accept?.blocked_reason && <p className="pt-blocked">{accept.blocked_reason}</p>}
        <p className="pt-footnote">Answer by turn {proposal.respond_by}. Peace starts next turn. {peaceScope(view, proposal).reason}</p>
      </div>
    </main>
  </Plate>;
}

function TermsPlate({ view, team, close }: { view: PlayerView; team: TeamId; close(): void }) {
  const sendButton = useRef<HTMLButtonElement>(null);
  useEffect(() => { sendButton.current?.focus({ preventScroll: true }); }, [team]);
  const orders = useGame((s) => s.orders[s.seat] ?? []);
  const [gold, setGold] = useState(0);
  const action = view.diplomacy.actions.find((a) => a.order.type === 'ProposeTreaty' && a.order.kind === 'Peace' && a.order.to === team);
  const forecast = offerGoldForecast(view, gold, orders);
  const blocked = action?.blocked_reason ?? forecast?.error ?? (!action ? 'Peace is unavailable now.' : null);
  const send = () => {
    if (!action || blocked) return;
    useGame.getState().editOrders((current) => queueDiplomacy(current, withDealGold(action.order, clampDealGold(gold)), view));
    close();
  };
  return <Plate view={view} team={team} mode="terms" onClose={close}>
    <Top view={view} team={team} />
    <main className="pt-terms-main">
      <People view={view} team={view.team} own />
      <div className="pt-term"><h1>You give</h1><div className="pt-term-box"><p>Peace, guaranteed 10 turns</p>{gold > 0 && <p className="cost">{gold} Gold</p>}</div></div>
      <div className="pt-term"><h1>You demand</h1><div className="pt-term-box"><p>Peace, guaranteed 10 turns</p>{gold < 0 && <p className="gain">{-gold} Gold</p>}</div></div>
      <People view={view} team={team} />
    </main>
    <div className="pt-add"><h2>Add a term</h2>{dealsOn(view) ? <DealGold view={view} orders={orders} gold={gold} onChange={setGold} /> : <p>Peace has no Gold term in this game.</p>}</div>
    <footer className="pt-bottom"><div><p>{warFacts(view, team).join(' · ')}{armyLine(view, team) ? ` · ${armyLine(view, team)}` : ''}</p><p>{empireTitle(view, team)} answers at the end of this turn.</p>{blocked && <p className="pt-blocked">{blocked}</p>}</div>
      <div className="pt-actions"><button className="btn quiet" onClick={close}>Cancel</button><button ref={sendButton} className="btn primary" disabled={!!blocked} onClick={send}>Send terms to {empireTitle(view, team)}</button></div>
    </footer>
  </Plate>;
}

export function WarPlate({ view, team, attackText, confirmLabel = 'Declare War', onConfirm, onClose }: {
  view: PlayerView; team: TeamId; attackText?: string; confirmLabel?: string; onConfirm(): void; onClose(): void;
}) {
  const action = view.diplomacy.actions.find((a) => a.order.type === 'DeclareWar' && a.order.against === team && !!a.order.breach === (relationWith(view, team)?.state === 'Alliance'));
  const keep = useRef<HTMLButtonElement>(null);
  useEffect(() => { keep.current?.focus({ preventScroll: true }); }, [team]);
  useEffect(() => {
    const key = (e: KeyboardEvent) => {
      if (e.defaultPrevented || e.repeat || e.metaKey || e.ctrlKey || e.altKey || e.shiftKey) return;
      if (e.key.toLowerCase() === 'd' || (e.key === 'Enter' && !(e.target instanceof Element && e.target.closest('button')))) {
        e.preventDefault(); e.stopPropagation(); onClose();
      }
    };
    window.addEventListener('keydown', key, true);
    return () => window.removeEventListener('keydown', key, true);
  }, [onClose]);
  const lead = empireMembers(view, team)[0];
  const lines = action && !action.blocked_reason ? [
    attackText ?? (action.effective_turn === view.turn ? 'War starts now: attacks you order this turn resolve this turn.' : `War begins turn ${action.effective_turn}.`),
    ...warLines(action, view).map((line) => line.text),
    ...(action.cost > 0 ? [`Breaking the Alliance costs ${action.cost} Gold.`] : []),
    ...(action.cost === 0 ? ['Costs no Gold.'] : []),
    ...(action.betrayal ? [`Betrayal: ${empireTitle(view, team)} will refuse an Alliance for ${allianceTerms(view.cities.filter((c) => c.owner === view.me).length).betrayalTurns} turns.`] : []),
  ] : [];
  const armies = armyLine(view, team, [...(action?.new_wars ?? []).flatMap((w) => [w.team, w.enemy]), ...(action?.calls ?? []).flatMap((w) => [w.team, w.enemy])]);
  return <Plate view={view} team={team} mode="war" onClose={onClose}>
    <Top view={view} team={team} war />
    <main className="pt-main"><People view={view} team={team} /><div className="pt-content">
      <h1>Declare War on {empireTitle(view, team)}?</h1>
      {lead && <blockquote>“{WAR_REPLY[lead.faction]}”</blockquote>}
      <section className="pt-war-box"><h2>What happens</h2>{lines.map((line) => <p key={line}>{line}</p>)}
        {armies && <p className="pt-army">{armies}</p>}
      </section>
      {action?.blocked_reason && <p className="pt-blocked">{action.blocked_reason}</p>}
      {!action && <p className="pt-blocked">War is unavailable now.</p>}
      <div className="pt-actions">{action && !action.blocked_reason && <button className="btn primary" onClick={onConfirm}>{confirmLabel}</button>}
        <button ref={keep} className="btn quiet" onClick={onClose}>Keep the Peace</button></div>
    </div></main>
  </Plate>;
}

/** The turn story owns the screen first; each pending Peace offer then opens once for this seat. */
export function PeaceTalks() {
  const view = useGame((s) => s.views?.[s.seat]);
  const seat = useGame((s) => s.seat);
  const client = useGame((s) => s.client);
  const warPrompt = useGame((s) => s.warPrompt);
  const diplomacyOpen = useGame((s) => s.diplomacyOpen);
  const request = useGame((s) => s.diplomacyTakeover);
  const orders = useGame((s) => s.orders);
  const errors = useGame((s) => s.errors);
  const humans = useGame((s) => s.humans);
  const story = useCelebration();
  const [automatic, setAutomatic] = useState<number | null>(null);
  const seen = useRef(new Set<string>());
  const lastTurn = useRef(view?.turn);
  useEffect(() => { seen.current.clear(); setAutomatic(null); }, [client]);
  useEffect(() => { setAutomatic(null); }, [seat]);
  useEffect(() => {
    if (view && lastTurn.current !== undefined && lastTurn.current !== view.turn) {
      setAutomatic(null);
      useGame.getState().set({ diplomacyTakeover: null });
    }
    lastTurn.current = view?.turn;
  }, [view?.turn]);
  const pending = view && !view.finished ? pendingDecisions(view, orders[seat] ?? [], errors[seat] ?? [], { humans, orders, errors }) : [];
  const offers = view ? incomingOffers(view).filter((p) => (p.kind ?? 'Peace') === 'Peace' && pending.some((d) => d.kind === 'diplomacy' && d.id === p.id)) : [];
  useEffect(() => {
    if (automatic !== null && !offers.some((p) => p.id === automatic)) setAutomatic(null);
    if (request?.mode === 'offer' && !offers.some((p) => p.id === request.id)) useGame.getState().set({ diplomacyTakeover: null });
  }, [automatic, request, offers]);
  useEffect(() => {
    if (!view || request || automatic !== null || story.holding || story.active || warPrompt || diplomacyOpen || document.querySelector('dialog[open]')) return;
    const next = offers.find((p) => !seen.current.has(`${seat}:${p.id}`));
    if (next) { seen.current.add(`${seat}:${next.id}`); setAutomatic(next.id); }
  }, [view, request, automatic, story.holding, story.active, seat, offers, warPrompt, diplomacyOpen]);
  const close = () => { setAutomatic(null); useGame.getState().set({ diplomacyTakeover: null }); };
  if (!view || view.finished) return null;
  if (request?.mode === 'terms') return <TermsPlate key={`${seat}:${request.team}`} view={view} team={request.team} close={close} />;
  if (request?.mode === 'war') return <WarPlate view={view} team={request.team} onClose={close} onConfirm={() => {
    const action = view.diplomacy.actions.find((a) => a.order.type === 'DeclareWar' && a.order.against === request.team && !!a.order.breach === (relationWith(view, request.team)?.state === 'Alliance'));
    if (!action || action.blocked_reason) return;
    useGame.getState().editOrders((current) => queueDiplomacy(current, action.order, view));
    close();
  }} />;
  const id = request?.mode === 'offer' ? request.id : automatic;
  const offer = id === null ? undefined : offers.find((p) => p.id === id);
  return offer ? <OfferPlate view={view} proposal={offer} close={close} /> : null;
}
