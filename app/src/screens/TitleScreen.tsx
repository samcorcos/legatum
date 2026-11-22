import { CALENDAR_NOTE, calendarLabel } from '../data';
import { useEffect, useId, useLayoutEffect, useRef, useState } from 'react';
import { parseAutosave, useGame, type SaveFile } from '../store';
import { listAutosaves, type AutosaveRecord } from '../platform/autosaveStore';
import { ReplaceGameDialog } from '../ui/ReplaceGameDialog';
import { pickSaveFile } from '../ui/GameMenu';
import { DecoRule, Fan, Wordmark } from '../brand/ornaments';
import { openGuide } from '../ui/guideState';
import { HintSettings } from '../ui/FirstGameHints';
import { HapticsSettings } from '../ui/HapticsSettings';
import { SoundSettings } from '../ui/SoundSettings';
import { CombatFxSettings } from '../ui/CombatFxSettings';
import { cleanRoomName } from '../net/roomName';
import { retryLine, retryWouldFail } from '../net/browserHost';
import { DEFAULT_PLAYER_NAME, MAX_PLAYER_NAME, rememberPlayerName, savedPlayerName } from '../platform/online';
import { TitleArt } from './TitleArt';
import { usePerfOverlay } from '../ui/PerfOverlay';
import { buildLabel } from '../buildLabel';
import { PRIVACY_URL, uploadsGames } from '../platform/gameUpload';
import { isOlderVersionMessage, olderAutosaveMessage, saveHasCurrentRules } from '../saveErrors';
import { MIN_SEATS } from '../lobby';
import { MAX_PLAYERS } from '../gameSetup';

/** Autosaves (by save time) that failed to open as an older version's save, so Continue can't repeat the failure. */
const olderAutosaves = new Set<string>();
const unreadableAutosaves = new Set<string>();

/** Main menu: Single Player, Multiplayer (Hot Seat, Host Game, Join Game), Continue, Load Game. */
export function TitleScreen() {
  const [menu, setMenu] = useState<'main' | 'multi'>('main');
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
        <span>Explore · expand · build · compete</span>
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
  const continueAutosave = useGame((s) => s.continueAutosave);
  const [records, setRecords] = useState<AutosaveRecord[] | null>(null);
  useEffect(() => {
    let live = true;
    void listAutosaves().then((list) => { if (live) setRecords(list); });
    return () => { live = false; };
  }, []);
  const [, refresh] = useState(0);
  // Newest first. A turn that will not open is skipped, so Continue offers the turn before it, never a dead end.
  const slots = (records ?? []).map((record) => ({ record, save: parseAutosave(record.text) }));
  const usable = slots.filter((slot) => slot.save && !unreadableAutosaves.has(slot.record.id));
  const auto = usable[0]?.save ?? null;
  const skipped = auto && slots[0].record !== usable[0].record ? slots[0].record.turn : null;
  const olderAuto = !!auto && (!saveHasCurrentRules(auto.game) || olderAutosaves.has(auto.savedAt));
  const unreadableAuto = slots.length > 0 && !auto;
  const continueAuto = async (record: AutosaveRecord, save: SaveFile) => {
    await continueAutosave(record, save);
    const failure = useGame.getState().error;
    if (isOlderVersionMessage(failure)) olderAutosaves.add(save.savedAt);
    else if (failure) {
      unreadableAutosaves.add(record.id);
      if (slots.some((slot) => slot.save && !unreadableAutosaves.has(slot.record.id))) set({ error: null });
    }
    if (failure) refresh((n) => n + 1);
  };
  // A file chosen with Load Game replaces the saved game in progress, so it asks first, like a new game does.
  const [chosen, setChosen] = useState<{ save: SaveFile; saved: { label: string; turn: number } } | null>(null);
  const askThenLoad = async (save: SaveFile) => {
    const [newest] = await listAutosaves();
    if (newest) setChosen({ save, saved: { label: newest.faction || newest.label, turn: newest.turn } });
    else await loadSave(save, true);
  };
  // While a message shows, starting a new game is the way forward, so it takes the primary slot.
  const newGamePrimary = !views && records !== null && (!auto || !!error || olderAuto || unreadableAuto);
  const continuePrimary = !views && !!auto && !error && !olderAuto && !unreadableAuto;
  return (
    <>
      {chosen && <ReplaceGameDialog file saved={chosen.saved} onCancel={() => setChosen(null)}
        onStart={() => { const { save } = chosen; setChosen(null); void loadSave(save, true); }} />}
      <TitleNotice error={error ?? (olderAuto ? olderAutosaveMessage(auto?.playtest) : skipped !== null ? `Your latest save (turn ${skipped}) could not be opened. Continue opens turn ${auto?.turn} instead.` : null)} />
      <div className="title-actions">
        <ResumeButton />
        <button className={`btn big ${newGamePrimary ? 'primary' : ''}`} onClick={() => openSetup('solo')}>
          Single Player
          <span className="btn-sub">You against {MIN_SEATS - 1} to {MAX_PLAYERS - 1} AI empires</span>
        </button>
        <button className="btn big" onClick={onMultiplayer}>
          Multiplayer
          <span className="btn-sub">{online?.relay ? 'Hot seat on this computer, or a game online' : 'Hot seat on this computer'}</span>

        </button>
        <button className={`btn big ${continuePrimary ? 'primary' : ''}`} disabled={!auto || !!busy || olderAuto || unreadableAuto} onClick={() => auto && usable[0] && continueAuto(usable[0].record, auto)}>
          {olderAuto ? 'Saved game (older version)' : unreadableAuto ? 'Saved game (unreadable)' : 'Continue'}
          <span className="btn-sub">
            {olderAuto ? 'Cannot continue with these rules' : unreadableAuto ? "This save file is damaged"
              : auto ? `${auto.names.join(' & ')} · turn ${auto.turn} · ${calendarLabel(auto.turn)} · ${new Date(auto.savedAt).toLocaleString()}` : records === null ? 'Looking for your game…' : 'No autosave yet'}
          </span>
        </button>
        <button className="btn big quiet" onClick={() => pickSaveFile(askThenLoad, (m) => set({ error: m }))}>
          Load Game…
        </button>
        <button className="btn big quiet" onClick={() => { set({ error: null }); goto('scenarios'); }}>
          Scenarios
        </button>
        <button className="btn big quiet" onClick={openGuide}>How to Play <span className="btn-sub">The field guide · ?</span></button>
        <Settings />
      </div>
      {uploadsGames() && (
        <p className="title-hosted-note title-upload-note">
          Games you play here are uploaded, without player names, to help improve the AI.{' '}
          <a href={PRIVACY_URL} target="_blank" rel="noreferrer">Privacy</a>
        </p>
      )}
    </>
  );
}

function MultiplayerMenu({ onBack }: { onBack: () => void }) {
  const views = useGame((s) => s.views);
  const goto = useGame((s) => s.goto);
  const openSetup = useGame((s) => s.openSetup);
  const error = useGame((s) => s.error);
  const online = useGame((s) => s.online);
  const refusal = useGame((s) => s.hostRefusal);
  // Online play: Host Game runs the game in this tab and opens its game room. A build with no relay has
  // no game list, so it offers hot seat only.
  const [hosting, setHosting] = useState<'idle' | 'starting' | 'failed'>('idle');
  // Host from a save has its own busy and failed state, so a retry stays on the button the player pressed.
  const [resuming, setResuming] = useState<'idle' | 'starting' | 'failed'>('idle');
  const busy = hosting === 'starting' || resuming === 'starting';
  const [hostError, setHostError] = useState<string | null>(null);
  const [name, setName] = useState(() => savedPlayerName() ?? DEFAULT_PLAYER_NAME);
  // The game list drops symbols and emoji; say so before the name is used.
  const typed = name.trim();
  const shown = cleanRoomName(typed);
  const listedAs = typed && shown !== typed ? (shown ? `Shown in the game list as “${shown}”.` : 'The game list will show “Game” and four characters.') : null;
  // After a refusal that pressing Host Game again cannot fix, Host Game is not the next action: Join Game is.
  const hostCannotRetry = hosting === 'failed' && !!refusal && retryWouldFail(refusal.kind);
  const hostPrimary = hosting === 'failed' && !views && !hostCannotRetry;
  const joinPrimary = !views && !busy && resuming !== 'failed' && !hostPrimary;
  const hostOnline = async () => {
    const who = name.trim().slice(0, MAX_PLAYER_NAME) || DEFAULT_PLAYER_NAME;
    rememberPlayerName(who);
    setHosting('starting');
    setResuming('idle');
    setHostError(null);
    const failure = await useGame.getState().hostOnline(who);
    if (failure) {
      setHostError(failure);
      setHosting('failed');
    }
  };

  const hostFromSave = () => {
    const who = name.trim().slice(0, MAX_PLAYER_NAME) || DEFAULT_PLAYER_NAME;
    rememberPlayerName(who);
    setHostError(null);
    setHosting('idle');
    void pickSaveFile(async (save) => {
      setResuming('starting');
      const failure = await useGame.getState().hostOnlineFromSave(who, save);
      if (failure) {
        setHostError(failure);
        setResuming('failed');
      }
    }, (message) => { setHostError(message); setResuming('failed'); });
  };

  return (
    <>
      <TitleNotice error={error ?? hostError} />
      <div className="title-actions">
        <ResumeButton />
        <button className="btn big" onClick={() => openSetup('hotseat')}>
          Hot Seat
          <span className="btn-sub">Several players take turns planning on this computer</span>
        </button>
        {online?.relay ? (
          <>
            <label className="host-name">
              {/* The note replaces the label while it shows, so it adds no height to a menu that is already tight. */}
              {listedAs === null ? <span>Your name in the game list</span> : <span className="host-name-note">{listedAs}</span>}
              <input
                className="lobby-input"
                aria-label="Your name in the game list"
                value={name}
                maxLength={MAX_PLAYER_NAME}
                disabled={busy}
                onChange={(e) => setName(e.target.value)}
                onKeyDown={(e) => { if (e.key === 'Enter' && !busy) void hostOnline(); }}
              />
            </label>
            <button className={`btn big ${hostPrimary ? 'primary' : ''}`} disabled={busy} onClick={hostOnline}>
              Host Game
              <span className="btn-sub">
                {hosting === 'starting' ? <><span className="spinner" /> Opening your game…</>
                  : hosting === 'failed' ? (refusal ? retryLine(refusal.kind, refusal.retryAfterS) : 'Try again') : 'Open a game that others can find and join'}
              </span>
            </button>
            <button className={`btn big ${resuming === 'failed' && !views ? 'primary' : ''}`} disabled={busy} onClick={hostFromSave}>
              Host from a save…
              <span className="btn-sub">
                {resuming === 'starting' ? <><span className="spinner" /> Opening your game…</>
                  : resuming === 'failed' ? 'Try again' : 'Carry on an online game from its save file'}
              </span>
            </button>
            <button className={`btn big ${joinPrimary ? 'primary' : ''}`} disabled={busy} onClick={() => goto('online')}>
              Join Game
              <span className="btn-sub">See the games that are open now</span>
            </button>
          </>
        ) : (
          <p className="title-hosted-note">This version has no online play. Hot seat still works.</p>
        )}
        <button className="btn quiet" onClick={onBack}>
          ← Back
        </button>
      </div>
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
