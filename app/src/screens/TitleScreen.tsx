import { CALENDAR_NOTE, calendarLabel } from '../data';
import { useEffect, useId, useLayoutEffect, useRef, useState } from 'react';
import { readAutosave, useGame, type SaveFile } from '../store';
import { pickSaveFile } from '../ui/GameMenu';
import { DecoRule, Fan, Wordmark } from '../brand/ornaments';
import { InviteLink } from '../ui/InviteLink';
import { openGuide } from '../ui/guideState';
import { HintSettings } from '../ui/FirstGameHints';
import { HapticsSettings } from '../ui/HapticsSettings';
import { SoundSettings } from '../ui/SoundSettings';
import { CombatFxSettings } from '../ui/CombatFxSettings';
import { canQuit, quitApp } from '../tauri';
import { DEFAULT_PLAYER_NAME, MAX_PLAYER_NAME, online as onlineApi, openHostedWindow, rememberPlayerName, savedPlayerName } from '../platform/online';
import { TitleArt } from './TitleArt';
import { usePerfOverlay } from '../ui/PerfOverlay';
import { buildLabel } from '../buildLabel';
import { HOSTED, SOURCE_URL } from '../platform/hosted';
import { isOlderVersionMessage, olderAutosaveMessage, saveHasCurrentRules } from '../saveErrors';

/** Autosaves (by save time) that failed to open as an older version's save, so Continue can't repeat the failure. */
const olderAutosaves = new Set<string>();
const unreadableAutosaves = new Set<string>();

/** Main menu: Single Player, Multiplayer (Hot Seat, Host Game, Join Game), Continue, Load Game. */
export function TitleScreen() {
  const lan = useGame((s) => s.lan);
  // Someone who opened a host's network address came to join: start on the multiplayer menu.
  const [menu, setMenu] = useState<'main' | 'multi'>(lan && !lan.status.is_local ? 'multi' : 'main');
  return (
    <div className="title-screen">
      <TitleArt />
      <div className="title-frame" aria-hidden>
        <i />
        <i />
        <i />
        <i />
      </div>
      <div className="title-inner">
        <Fan width={112} />
        <h1 className="title-logo">
          <Wordmark height={112} />
        </h1>
        <div className="title-kicker">
          <DecoRule width={72} />
          <span>A Contest Through History</span>
          <DecoRule width={72} flip />
        </div>
        <p className="title-tag">Build a civilization. Contest its legacy.</p>
        <div className="title-menu">
          {menu === 'main' ? <MainMenu onMultiplayer={() => setMenu('multi')} /> : <MultiplayerMenu onBack={() => setMenu('main')} />}
        </div>
      </div>
      <div className="title-foot">
        <span>{buildLabel}</span>
        <span>
          {lan ? `Network game server · ${lan.status.is_local ? 'this computer is the host' : 'connected to a host'}` : 'Explore · expand · build · compete'}
        </span>
      </div>
    </div>
  );
}

/** A message about the last action (an older save that won't open, a failed load), in the menu's
 * flow above the buttons so nothing overlaps it, in plain body text. */
function TitleNotice({ error }: { error: string | null }) {
  if (!error) return null;
  return <p className="title-notice" role="alert">{error}</p>;
}

function ResumeButton() {
  const goto = useGame((s) => s.goto);
  const views = useGame((s) => s.views);
  if (!views) return null;
  return (
    <button className="btn big primary title-resume" onClick={() => goto('game')}>
      Resume
      <span className="btn-sub" title={CALENDAR_NOTE}>Turn {views[0].turn} · {calendarLabel(views[0].turn)}</span>
    </button>
  );
}

function MainMenu({ onMultiplayer }: { onMultiplayer: () => void }) {
  const online = useGame((s) => s.online);
  const openSetup = useGame((s) => s.openSetup);
  const goto = useGame((s) => s.goto);
  const loadSave = useGame((s) => s.loadSave);
  const set = useGame((s) => s.set);
  const error = useGame((s) => s.error);
  const busy = useGame((s) => s.busy);
  const views = useGame((s) => s.views);
  const [auto, setAuto] = useState<SaveFile | null>(null);
  const [checkingAuto, setCheckingAuto] = useState(true);
  const [, refresh] = useState(0);
  const olderAuto = !!auto && (!saveHasCurrentRules(auto.game) || olderAutosaves.has(auto.savedAt));
  const unreadableAuto = !!auto && unreadableAutosaves.has(auto.savedAt);
  const continueAuto = async (save: SaveFile) => {
    await loadSave(save);
    const failure = useGame.getState().error;
    if (isOlderVersionMessage(failure)) olderAutosaves.add(save.savedAt);
    else if (failure) unreadableAutosaves.add(save.savedAt);
    if (failure) refresh((n) => n + 1);
  };
  useEffect(() => {
    let active = true;
    void readAutosave().then((save) => {
      if (active) { setAuto(save); setCheckingAuto(false); }
    });
    return () => { active = false; };
  }, []);
  // While a message shows, starting a new game is the way forward, so it takes the primary slot.
  const newGamePrimary = !views && (!auto || !!error || olderAuto || unreadableAuto);
  const continuePrimary = !views && !!auto && !error && !olderAuto && !unreadableAuto;
  return (
    <>
      <TitleNotice error={error ?? (olderAuto ? olderAutosaveMessage(auto?.playtest) : null)} />
      <div className="title-actions">
        <ResumeButton />
        <button className={`btn big ${newGamePrimary ? 'primary' : ''}`} onClick={() => openSetup('solo')}>
          Single Player
          <span className="btn-sub">You against five AI empires</span>
        </button>
        <button className="btn big" onClick={onMultiplayer}>
          Multiplayer
          <span className="btn-sub">{HOSTED ? 'Hot seat on this computer' : online?.relay ? 'Hot seat on this computer, or a game online' : 'Hot seat on this computer, or a game on your network'}</span>
        </button>
        <button className={`btn big ${continuePrimary ? 'primary' : ''}`} disabled={!auto || !!busy || olderAuto || unreadableAuto} onClick={() => auto && continueAuto(auto)}>
          {olderAuto ? 'Saved game (older version)' : unreadableAuto ? 'Saved game (unreadable)' : 'Continue'}
          <span className="btn-sub">
            {olderAuto ? 'Cannot continue with these rules' : unreadableAuto ? "This save file is damaged"
              : auto ? `${auto.names.join(' & ')} · turn ${auto.turn} · ${calendarLabel(auto.turn)} · ${new Date(auto.savedAt).toLocaleString()}` : checkingAuto ? 'Checking autosaves…' : 'No autosave yet'}
          </span>
        </button>
        <button className="btn big quiet" onClick={() => pickSaveFile((save) => loadSave(save, true), (m) => set({ error: m }))}>
          Load Game…
        </button>
        <button className="btn big quiet" onClick={() => { set({ error: null }); goto('scenarios'); }}>
          Scenarios
        </button>
        <button className="btn big quiet" onClick={openGuide}>How to Play <span className="btn-sub">The field guide · ?</span></button>
        <Settings />
        {canQuit() && (
          <button className="btn big quiet" onClick={() => quitApp().catch((e) => set({ error: `Could not quit: ${String(e)}` }))}>
            Quit
          </button>
        )}
      </div>
    </>
  );
}

function MultiplayerMenu({ onBack }: { onBack: () => void }) {
  const lan = useGame((s) => s.lan);
  const views = useGame((s) => s.views);
  const goto = useGame((s) => s.goto);
  const openSetup = useGame((s) => s.openSetup);
  const refresh = useGame((s) => s.refreshStatus);
  const error = useGame((s) => s.error);
  const online = useGame((s) => s.online);
  const st = lan?.status;
  // Online play from the desktop app: starting a game opens it in this window. A build with no relay
  // has no lobby to list a game in, so it shows the network-game buttons instead.
  const [hosting, setHosting] = useState<'idle' | 'starting' | 'failed'>('idle');
  const [name, setName] = useState(() => savedPlayerName() ?? DEFAULT_PLAYER_NAME);
  const hostOnline = async () => {
    const who = name.trim().slice(0, MAX_PLAYER_NAME) || DEFAULT_PLAYER_NAME;
    rememberPlayerName(who);
    setHosting('starting');
    try {
      const game = await onlineApi().hostStart(who);
      openHostedWindow(game.url);
    } catch {
      setHosting('failed');
    }
  };

  // Poll while waiting for the host, and keep seat info fresh.
  const onLan = !!st;
  const hasGame = !!st?.has_game;
  useEffect(() => {
    if (!onLan) return;
    const t = setInterval(() => refresh(), hasGame ? 4000 : 2000);
    return () => clearInterval(t);
  }, [refresh, onLan, hasGame]);

  const hostSub = HOSTED
    ? 'Not available on this website'
    : !st
    ? 'Start the network host on this computer first'
    : !st.is_local
      ? 'Only the host computer can start a network game'
      : st.has_game
        ? 'Replaces the current network game'
        : 'Players on your network join your game';
  // Staging room (servers that implement it report `staging`): an open room takes precedence.
  const room = st?.staging ?? null;
  const openSeats = room ? room.config.seats.filter((s, i) => s.kind === 'open' && !room.occupants[i]?.connected).length : 0;
  const canJoin = !!room || !!st?.has_game;
  const joinSub = HOSTED
    ? 'Not available on this website'
    : !st
    ? "Open the host's network address in your browser"
    : room
      ? `Game room open · ${openSeats} open seat${openSeats === 1 ? '' : 's'}`
      : st.has_game
        ? `Turn ${st.turn} · ${st.seats.map((s) => s.name).join(' & ')}`
        : 'Waiting for the host to start a game…';

  return (
    <>
      <TitleNotice error={error ?? (hosting === 'failed' ? 'Could not open your game. Check your internet connection and try again.' : null)} />
      <div className="title-actions">
        <ResumeButton />
        <button className="btn big" onClick={() => openSetup('hotseat')}>
          Hot Seat
          <span className="btn-sub">Several players take turns planning on this computer</span>
        </button>
        {online?.relay && !st ? (
          <>
            <label className="host-name">
              <span>Your name in the game list</span>
              <input
                className="lobby-input"
                value={name}
                maxLength={MAX_PLAYER_NAME}
                disabled={hosting === 'starting'}
                onChange={(e) => setName(e.target.value)}
                onKeyDown={(e) => { if (e.key === 'Enter' && hosting !== 'starting') void hostOnline(); }}
              />
            </label>
            <button className={`btn big ${hosting === 'failed' && !views ? 'primary' : ''}`} disabled={hosting === 'starting'} onClick={hostOnline}>
              Host Game
              <span className="btn-sub">
                {hosting === 'starting' ? <><span className="spinner" /> Opening your game…</>
                  : hosting === 'failed' ? 'Try again' : 'Open a game that others can find and join'}
              </span>
            </button>
            <button className={`btn big ${!views && hosting === 'idle' ? 'primary' : ''}`} disabled={hosting === 'starting'} onClick={() => goto('online')}>
              Join Game
              <span className="btn-sub">See the games that are open now</span>
            </button>
          </>
        ) : (
          <>
            <button className="btn big" disabled={!st?.is_local} onClick={() => openSetup('host')}>
              Host Game
              <span className="btn-sub">{hostSub}</span>
            </button>
            <button className={`btn big ${canJoin && !views ? 'primary' : ''}`} disabled={!canJoin} onClick={() => goto(room ? 'staging' : 'join')}>
              Join Game
              <span className="btn-sub">{joinSub}</span>
            </button>
          </>
        )}
        {st && !canJoin && !st.is_local && (
          <div className="waiting-host">
            <div className="spinner" />
            Waiting for the host to start a game…
          </div>
        )}
        {HOSTED && (
          <p className="title-hosted-note">
            A network game needs the host program running on one player’s computer.{' '}
            <a href={SOURCE_URL} target="_blank" rel="noreferrer">How to host a game</a>
          </p>
        )}
        <button className="btn quiet" onClick={onBack}>
          ← Back
        </button>
      </div>
      {st?.is_local && st.lan_urls.length > 0 && (
        <div className="title-invite">
          <InviteLink urls={st.lan_urls} />
        </div>
      )}
    </>
  );
}

/** Settings open in a modal over the title, so the menu never changes size: first-game hints and the frame-rate readout. */
function Settings() {
  const [open, setOpen] = useState(false);
  const trigger = useRef<HTMLButtonElement>(null);
  const close = () => {
    setOpen(false);
    trigger.current?.focus({ preventScroll: true });
  };
  return (
    <div className="title-settings">
      <button ref={trigger} className="btn big quiet title-divided" onClick={() => setOpen(true)} aria-haspopup="dialog" aria-expanded={open}>
        Settings
      </button>
      {open && <SettingsDialog onClose={close} />}
    </div>
  );
}

function SettingsDialog({ onClose }: { onClose: () => void }) {
  const dialog = useRef<HTMLDialogElement>(null);
  const closeButton = useRef<HTMLButtonElement>(null);
  const titleId = useId();
  const perfOn = usePerfOverlay((s) => s.on);
  const setPerf = usePerfOverlay((s) => s.setOn);
  useLayoutEffect(() => {
    const el = dialog.current;
    if (!el) return;
    el.showModal();
    closeButton.current?.focus({ preventScroll: true });
    return () => el.close();
  }, []);
  return (
    <dialog
      ref={dialog}
      className="modal settings-modal"
      aria-labelledby={titleId}
      onCancel={(e) => {
        e.preventDefault();
        onClose();
      }}
      // A click on the backdrop lands on the dialog element itself.
      onClick={(e) => {
        if (e.target === e.currentTarget) onClose();
      }}
    >
      <header className="settings-head">
        <h2 id={titleId}>Settings</h2>
        <button ref={closeButton} className="btn quiet" onClick={onClose} aria-label="Close settings">
          Close <kbd>Esc</kbd>
        </button>
      </header>
      <div className="settings-body">
        <HintSettings />
        <HapticsSettings />
        <SoundSettings />
        <CombatFxSettings />
        <div className="settings-row">
          <span className="lbl">Frame rate</span>
          <div className="seg small">
            <button className={perfOn ? '' : 'on'} aria-pressed={!perfOn} onClick={() => setPerf(false)}>
              Hidden
            </button>
            <button className={perfOn ? 'on' : ''} aria-pressed={perfOn} onClick={() => setPerf(true)}>
              Shown
            </button>
          </div>
        </div>
      </div>
    </dialog>
  );
}
