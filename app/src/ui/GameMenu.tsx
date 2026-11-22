import { useEffect, useMemo, useRef, useState } from 'react';
import { hostedSaveText, saveFile, useGame, type SaveFile } from '../store';
import { openGuide } from './guideState';
import { HintSettings } from './FirstGameHints';
import { HapticsSettings } from './HapticsSettings';
import { SoundSettings } from './SoundSettings';
import { CombatFxSettings } from './CombatFxSettings';
import { loadFromFile, saveToFile } from '../platform/files';
import { checkSaveText } from '../platform/saveLimits';
import { loadErrorMessage, parseSaveFile } from '../saveErrors';
import { exportRetainedHistories } from '../platform/archive';
import { currentActivePlaySeconds, startActivePlayClock } from '../activePlayTime';
import { archivePlaySeconds, formatPlayDuration } from '../playTime';

export function saveGameToFile() {
  const { client, views } = useGame.getState();
  if (!client || !views) return;
  try {
    const s = saveFile(client, views);
    // Serialize the envelope only; s.game stays an opaque string.
    saveToFile(JSON.stringify(s), s.turn);
  } catch (e) {
    useGame.getState().set({ error: String(e) });
  }
}

/** The online host's "Save to file…": the game as the host engine holds it now, so it can be hosted again. */
export async function saveHostedGameToFile() {
  const { set, views } = useGame.getState();
  try {
    const text = hostedSaveText();
    if (!text || !views?.length) return set({ error: 'There is no game to save yet.' });
    await saveToFile(text, views[0].turn);
  } catch (e) {
    set({ error: String(e) });
  }
}

export async function pickSaveFile(onLoad: (s: SaveFile) => void | Promise<void>, onError: (msg: string) => void) {
  try {
    const contents = await loadFromFile();
    if (contents === null) return;
    checkSaveText(contents);
    const s = parseSaveFile(contents);
    await onLoad(s);
  } catch (e) {
    onError(loadErrorMessage(e));
  }
}


export function GameMenu() {
  const [open, setOpen] = useState(false);
  const [activeSeconds, setActiveSeconds] = useState(0);
  const ref = useRef<HTMLDivElement>(null);
  const trigger = useRef<HTMLButtonElement>(null);
  const loadSave = useGame((s) => s.loadSave);
  const set = useGame((s) => s.set);
  const goto = useGame((s) => s.goto);
  const hash = useGame((s) => s.lastHash);
  const hasReport = useGame((s) => !!s.report);
  const kind = useGame((s) => s.client?.kind);
  const leaveOnline = useGame((s) => s.leaveOnline);
  const hostHere = useGame((s) => !!s.net?.status.is_local);
  const client = useGame((s) => s.client);
  const turn = useGame((s) => s.views?.[s.seat]?.turn);
  const archivedSeconds = useMemo(() => archivePlaySeconds(client?.archive?.()), [client, turn]);
  useEffect(() => {
    const clock = startActivePlayClock();
    const tick = () => setActiveSeconds(currentActivePlaySeconds());
    tick();
    const timer = window.setInterval(tick, 1000);
    return () => { window.clearInterval(timer); clock.dispose(); };
  }, []);
  useEffect(() => {
    if (!open) return;
    const close = (e: PointerEvent) => {
      if (!ref.current?.contains(e.target as Node)) setOpen(false);
    };
    window.addEventListener('pointerdown', close);
    ref.current?.querySelector<HTMLElement>('.menu-pop button')?.focus();
    return () => window.removeEventListener('pointerdown', close);
  }, [open]);
  return (
    <div className="menu" ref={ref} data-game-overlay={open ? '' : undefined}
      onKeyDown={(e) => {
        if (!open) return;
        e.stopPropagation();
        if (e.key === 'Escape') {
          e.preventDefault();
          setOpen(false);
          trigger.current?.focus();
        }
      }}
      onBlur={(e) => {
        // Safari and other WebKit views do not focus a <button> on click: mousedown moves focus
        // from the previously focused element to <body>, so blur fires with relatedTarget null in the
        // middle of every click on a menu item. Closing on that blur (even after a rAF check of
        // activeElement) unmounts the item before mouseup and the click never fires. Only keyboard
        // focus moving to a real element outside the menu closes it here; a mouse press outside is the
        // window pointerdown listener, and Escape is above.
        const to = e.relatedTarget;
        if (to instanceof Node && !e.currentTarget.contains(to)) setOpen(false);
      }}
    >
      <button ref={trigger} className="icon-btn" onClick={() => setOpen(!open)} aria-label="Menu" title="Menu" aria-expanded={open} aria-controls={open ? 'game-menu' : undefined}>
        <svg width="18" height="18" viewBox="0 0 18 18">
          <path d="M3 5h12M3 9h12M3 13h12" stroke="currentColor" strokeWidth="1.8" strokeLinecap="round" />
        </svg>
      </button>
      {open && (
        <div className="menu-pop" id="game-menu">
          {kind !== 'remote' && <p className="menu-foot" aria-label="Active play time">Active play: {formatPlayDuration(archivedSeconds + activeSeconds)}</p>}
          {kind !== 'remote' && (
            <>
              <button onClick={() => (saveGameToFile(), setOpen(false))}>Save to file…</button>
              <button
                onClick={() => {
                  setOpen(false);
                  pickSaveFile((save) => loadSave(save, true), (m) => set({ error: `${m} Your current game is unchanged.` }));
                }}
              >
                Load from file…
              </button>
              <hr />
            </>
          )}
          {kind === 'remote' && hostHere && (
            <>
              <button onClick={() => (saveHostedGameToFile(), setOpen(false))}>Save to file…</button>
              <hr />
            </>
          )}
          <HintSettings />
          <HapticsSettings />
          <SoundSettings />
          <CombatFxSettings />
          <hr />
          <button onClick={() => (trigger.current?.focus(), setOpen(false), openGuide())}>How to Play (?)</button>
          <button onClick={() => (trigger.current?.focus(), set({ techOpen: true }), setOpen(false))}>Technologies (T)</button>
          <button onClick={() => (trigger.current?.focus(), set({ citiesOpen: true }), setOpen(false))}>Cities (C)</button>
          {/* A loaded save has no report until the next turn resolves: don't offer a no-op. */}
          <button disabled={!hasReport} onClick={() => (trigger.current?.focus(), set({ reportOpen: true, reportTurn: null }), setOpen(false))}>Turn report (L)</button>
          <hr />
          {kind === 'remote' ? (
            hostHere ? (
              <button onClick={() => {
                setOpen(false);
                if (window.confirm('End this game for everyone? Other players go back to the title, and the game cannot be resumed.')) leaveOnline();
              }}>End the game for everyone</button>
            ) : (
              <button onClick={() => (leaveOnline(), setOpen(false))}>Leave the game (free this seat)</button>
            )
          ) : (
            <button onClick={() => (goto('title'), setOpen(false))}>Exit to title</button>
          )}
          <div className="menu-foot">
            {kind !== 'remote' ? 'Autosaved each turn' : hostHere ? 'Online game · you host it: keep this tab open, or save to a file' : 'Online game · the host keeps the game'}
          </div>
          {/* Troubleshooting only: reachable, but out of the player's way. */}
          <details className="menu-advanced">
            <summary>Advanced</summary>
            {kind === 'wasm' && <>
              <button onClick={() => {
                setOpen(false);
                void exportRetainedHistories(useGame.getState().client?.archive?.()).catch((e) => set({ error: String(e) }));
              }}>Export game history</button>
            </>}
            <div className="menu-foot">
              {kind === 'remote' && 'Online player'}
              {hash && <div className="mono">state {hash.slice(0, 10)}…</div>}
            </div>
          </details>
        </div>
      )}
    </div>
  );
}
