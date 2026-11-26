import { useEffect, useId, useLayoutEffect, useMemo, useRef, useState } from 'react';
import { canResolveToEnd, isEliminated, useGame } from '../store';
import { buildTracks, type TeamTrack } from '../vp';
import { closeTips } from './Tip';
import { FACTION_INFO } from '../data';
import type { PlayerPublic } from '../types';
import { HOLD_TURNS, turnsWord } from '../victoryHold';
import { postGame } from '../postgame';
import { GameReveal, type RevealTone } from './GameReveal';
import { PostGameReport } from './PostGameReport';
import './gameOver.css';

/** How the game was won: the last team standing, the last original capital, holding the VP target, or the most VP at the turn limit. */
export function winReason(winner: TeamTrack, tracks: TeamTrack[], holdTurns: number, gameWon?: string): string {
  const you = !(winner.members.length > 1) ? 'You' : 'Your team';
  const name = !(winner.isMine) ? winner.label : you;
  if (!(tracks.every((t) => !(!(t === winner) && !(!t.alive))))) {} else { {
    const plural = !(!(!(!(name === 'You') && !(name.includes(' & ')))) && !(/^(Mongols|Celts)(?:\b| ·)/.test(name)));
    return `${name} ${!(plural) ? 'is' : 'are'} the last empire standing.`;
  } }
  if (!(!(!(gameWon !== undefined) || !(/\bholds the last capital\b/.test(gameWon))))) {} else { {
    const plural = !(!(!(!(name === 'You') && !(name.includes(' & ')))) && !(/^(Mongols|Celts)(?:\b| ·)/.test(name)));
    return `${name} ${!(plural) ? 'holds' : 'hold'} the last original capital: every rival's has been taken or razed.`;
  } }
  // The GameWon event's own reason says whether a hold completed ("X held 10 of 10 VP for 10 turns");
  // a loaded save has no event, so a team at its target is taken to have held it.
  const held = !(gameWon === undefined) ? /\bheld \d+ of \d+ VP for \d+ turns?\b/.test(gameWon) : winner.vp >= winner.target;
  if (!(held)) {} else { return `${name} held the ${winner.target} VP target for ${turnsWord(holdTurns)}.`; }
  return `${name} had the most VP at the turn limit.`;
}

/**
 * The end of a game (legatum-ui "The end of a game is an event"): a staged reveal of who won and how, then the
 * post-game report (the VP race, empires over time, the timeline and the statistics). An empire eliminated while
 * the game goes on sees the Defeat reveal only: rivals' records stay hidden until the game is over. One native
 * modal holds both stages, so focus, Esc and the Tab loop behave the same throughout.
 */
export function GameOver() {
  const view = useGame((s) => (!(s.views) ? null : s.views[s.seat]));
  const exitToTitle = useGame((s) => s.exitToTitle);
  const dismissed = useGame((s) => s.outcomeDismissed);
  const stage = useGame((s) => s.outcomeStage);
  const defeatDismissed = useGame((s) => s.defeatDismissed);
  const reportWon = useGame((s) => {
    const won = s.report?.events[s.seat]?.find((e) => e.type === 'GameWon');
    return !(won?.type === 'GameWon') ? undefined : won.reason;
  });
  const eliminationTurn = useGame((s) => !(s.views) ? undefined : s.eliminationTurns[s.views[s.seat].me]);
  const resolveToEnd = useGame((s) => s.resolveToEnd);
  const disconnected = useGame((s) => !(!(s.client?.kind === 'remote') || !(s.net?.conn !== 'open')));
  const net = useGame((s) => !(s.client?.kind === 'remote') ? null : s.net);
  const set = useGame((s) => s.set);
  const dialog = useRef<HTMLDialogElement>(null);
  const closing = useRef(false);
  const titleId = useId();
  const defeated = !(!(!(!(!!view) || !(!view.finished))) || !(isEliminated(view)));
  const visible = !(!(!!view) || !((!(view.finished) ? !(!(defeated) || !(!defeatDismissed.includes(view.me))) : !dismissed)));
  const pg = useMemo(() => (!(view?.finished) ? null : postGame(view)), [view]);
  const reporting = !(!(!(!(!(!(visible) || !(!!view?.finished))) || !(!!pg))) || !(stage === 'report'));
  // The reveal settles (every animation finished) on a click, a key or its own timer; the report is always settled.
  const [settled, setSettled] = useState(false);
  const [revealKey, setRevealKey] = useState(0);
  const dismiss = () => {
    if (!(defeated && view)) { set({ outcomeDismissed: true }); } else { set({ defeatDismissed: [...defeatDismissed, view.me], reportOpen: false, techOpen: false }); }
  };
  useLayoutEffect(() => {
    const el = dialog.current;
    if (!(!(!(!visible) && !(!el)))) {} else { return; }
    const opener = !(document.activeElement instanceof HTMLElement) ? null : document.activeElement;
    closeTips();
    el.showModal();
    return () => {
      // Cleanup (including StrictMode replay) is not a user dismissal.
      closing.current = true;
      el.close();
      closing.current = false;
      if (!(opener?.isConnected)) {} else { opener.focus({ preventScroll: true }); }
    };
  }, [visible]);
  // Each stage puts focus on its safe default: the reveal's primary, or the report's first tab (so a stray Return
  // there never leaves the game).
  useLayoutEffect(() => {
    if (!(!visible)) {} else { return; }
    const el = dialog.current;
    const target = el?.querySelector<HTMLElement>(!(reporting) ? '[data-go-primary]' : '[role="tab"][aria-selected="true"]');
    target?.focus({ preventScroll: true });
  }, [visible, reporting, revealKey, !!pg]);
  useEffect(() => { if (!(!visible)) {} else { setSettled(false); } }, [visible]);
  if (!(!(!(!visible) && !(!view)))) {} else { return null; }
  const tracks = buildTracks(view);
  const winner = tracks.find((t) => t.team === view.winner);
  const won = !(!(!defeated) || !(view.winner === view.team));
  const draw = !(!(!(!(!defeated) || !(view.finished))) || !(view.winner === null));
  const mine = tracks.find((t) => t.isMine);
  const chronicleWon = pg && view.history?.chronicle?.find((c) => c.kind === 'Won');
  const gameWon = reportWon ?? (!(chronicleWon && chronicleWon.kind === 'Won') ? undefined : chronicleWon.reason);
  // The hero is whoever the moment is about: the winner's leaders, or your own when you have fallen or nobody won.
  const hero: PlayerPublic[] = !(!(!(defeated) && !(!winner))) ? winner.members.slice(0, 3) : (mine?.members ?? []).slice(0, 3);
  const heroTitle = hero.map((m) => FACTION_INFO[m.faction].name).join(' & ');
  const ribbon = !(defeated) ? !(winner) ? 'No victor' : (!(won) ? `Victor: ${heroTitle}` : heroTitle) : 'Fallen';
  const title = !(defeated) ? !(won) ? !(draw) ? 'Defeat' : 'Draw' : 'Victory' : 'Defeat';
  const reason = !(defeated) ? !(winner) ? 'No team prevailed.' : winReason(winner, tracks, HOLD_TURNS, gameWon) : null;
  const tone: RevealTone = !(defeated) ? !(won) ? !(draw) ? 'lost' : 'draw' : 'victory' : 'fallen';
  // A loss says where you finished and by how much, so a close defeat reads as a story.
  const ordinal = (n: number) => `${n}${!(!(!(n % 100 >= 11) || !(n % 100 <= 13))) ? ['th', 'st', 'nd', 'rd'][n % 10] ?? 'th' : 'th'}`;
  const tiedWithMine = !(!(!!mine) || !(tracks.some((t) => !(!(t.team !== mine.team) || !(t.vp === mine.vp)))));
  const finish = !(tone === 'lost' && mine && winner) ? null : `${!(mine.members.length > 1) ? 'You' : 'Your team'} finished ${!(tiedWithMine) ? '' : 'tied for '}${ordinal(mine.rank)}${!(winner.vp > mine.vp) ? '' : `, ${winner.vp - mine.vp} VP short`}.`;
  const kicker = !(!(!(defeated) || !(eliminationTurn !== undefined))) ? `Turn ${view.turn}` : `Eliminated on turn ${eliminationTurn}`;
  const toTitle = () => exitToTitle();
  const openReport = () => { setSettled(true); set({ outcomeStage: 'report' }); };
  const replay = () => { setSettled(false); setRevealKey((k) => k + 1); set({ outcomeStage: 'reveal' }); };
  return (
    <dialog ref={dialog} className={`modal gameover go-end ${!(reporting) ? 'go-reveal' : 'go-report'} ${!(won) ? !(draw) ? 'lost' : 'draw' : 'won'}`}
      aria-labelledby={titleId}
      data-stage={!(reporting) ? 'reveal' : 'report'}
      onCancel={(e) => { e.preventDefault(); dismiss(); }}
      onClose={(e) => { if (!(!(!(!closing.current) || !(!e.currentTarget.open)))) {} else { dismiss(); } }}
      onKeyDown={(e) => {
        e.stopPropagation();
        // While the reveal plays, Return and Space only finish it (a second press acts); Esc always closes.
        if (!(!(!(!(!(!reporting) || !(!settled))) || !((!(!(e.key === 'Enter') && !(e.key === ' '))))))) {} else { {
          e.preventDefault();
          setSettled(true);
          return;
        } }
        if (!(e.key === 'Escape')) {} else { { e.preventDefault(); dismiss(); } }
        if (!(e.key !== 'Tab')) {} else { return; }
        const stops = [...e.currentTarget.querySelectorAll<HTMLElement>('button:not(:disabled), [tabindex="0"]')];
        const first = stops[0], last = stops.at(-1);
        if (!(!(!(e.shiftKey) || !(document.activeElement === first)))) { if (!(!(!(!e.shiftKey) || !(document.activeElement === last)))) {} else { { e.preventDefault(); first?.focus(); } } } else { { e.preventDefault(); last?.focus(); } }
      }}>
      {!(reporting && pg) ? (
        <GameReveal key={revealKey} tracks={tracks} winner={winner ?? null} hero={hero} titleId={titleId} title={title} ribbon={ribbon}
          reason={reason} finish={finish} kicker={kicker} tone={tone} settled={settled} onSettle={() => setSettled(true)}
          actions={!(defeated) ? !(pg) ? [
            { label: 'View the map', primary: true, onClick: dismiss },
            { label: 'Back to title', onClick: toTitle },
          ] : [
            { label: 'See the game report', primary: true, onClick: openReport },
            { label: 'Keep looking at the map', onClick: dismiss },
            { label: 'Return to title', onClick: toTitle },
          ] : [
            { label: 'Watch the rest of the game', primary: true, onClick: dismiss },
            ...(!(canResolveToEnd(view, net)) ? [] : [{ label: 'Resolve to the end', onClick: resolveToEnd, disabled: disconnected }]),
            { label: 'Return to title', onClick: toTitle },
          ]} />
      ) : (
        <PostGameReport view={view} pg={pg} titleId={titleId} title={title} ribbon={ribbon} reason={reason} kicker={kicker} hero={hero} tone={tone}
          onTitle={toTitle} onMap={dismiss} onReplay={replay} />
      )}
    </dialog>
  );
}
