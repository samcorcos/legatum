import { CALENDAR_NOTE, calendarLabel, FACTION_INFO, factionName } from '../data';
import type { Faction } from '../types';
import { useEffect, useRef, useState } from 'react';
import { currentCampaign, needsOnlineHost, parseAutosave, useGame, type SaveFile } from '../store';
import { listAutosaves, type AutosaveRecord } from '../platform/autosaveStore';
import { ReplaceGameDialog } from '../ui/ReplaceGameDialog';
import { pickSaveFile } from '../ui/GameMenu';
import { Fan, Wordmark } from '../brand/ornaments';
import { openGuide } from '../ui/guideState';
import { SettingsDialog } from '../ui/SettingsDialog';
import { cleanRoomName } from '../net/roomName';
import { JOIN_CODE_MAX, joinCodeProblem, typedJoinCode } from '../net/joinCode';
import { retryLine, retryWouldFail } from '../net/browserHost';
import { DEFAULT_PLAYER_NAME, MAX_PLAYER_NAME, rememberPlayerName, savedPlayerName } from '../platform/online';
import { TitleArt } from './TitleArt';
import { buildLabel } from '../buildLabel';
import { buildTime } from '../buildTime';
import { PRIVACY_URL, uploadsGames } from '../platform/gameUpload';
import { isOlderVersionMessage, loadErrorMessage, olderAutosaveMessage, parseSaveFile, saveHasCurrentRules, saveRulesVersion } from '../saveErrors';
import { buildForSave, isBehind, navigate, newestUrl, pageLaunch, takeHandedOffSave } from '../platform/versions';
import { useSiteBuilds } from '../ui/useSiteBuilds';
import { MIN_SEATS } from '../lobby';
import { MAX_PLAYERS } from '../gameSetup';

/** Autosaves (by save time) that failed to open as an older version's save, so Continue can't repeat the failure. */
const olderAutosaves = new Set<string>();
const unreadableAutosaves = new Set<string>();
/** Why a game with more than one human player cannot carry on in a build with no online play. */
const OTHER_PLAYERS_OFFLINE = 'This game has other players. Online play is unavailable in this version.';

/** The launch another build opened this page with (platform/versions.ts) is acted on once. */
let launchHandled = false;

/** Main menu: Single Player, online Multiplayer, Continue, Load Game. */
export function TitleScreen() {
  const [menu, setMenu] = useState<'main' | 'multi'>(() => {
    const launch = pageLaunch();
    return launch?.kind === 'menu' && launch.menu === 'multi' ? 'multi' : 'main';
  });
  // A save with more than one human player (an old hot-seat game) chosen from the main menu: it opens
  // Multiplayer with that save ready to host, so the player never picks the file twice.
  const [hostSave, setHostSave] = useState<SaveFile | null>(null);
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
        <div className="title-menu">
          {menu === 'main'
            ? <MainMenu onMultiplayer={() => setMenu('multi')} onHostSave={(save) => { setHostSave(save); setMenu('multi'); }} />
            : <MultiplayerMenu hostSave={hostSave} onBack={() => { setHostSave(null); setMenu('main'); }} />}
        </div>
      </div>
      <div className="title-foot">
        <span>{buildTime ? `${buildLabel} · Last updated ${buildTime}` : buildLabel}</span>
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

/** The human players' peoples, e.g. "Rome & Egypt": what a game is called on the menu. Names are a fallback for
 * older autosaves, which recorded no faction. */
function peoplesLabel(faction: string | undefined, names: string[]): string {
  const peoples = (faction ?? '').split(' & ').filter((f): f is Faction => f in FACTION_INFO);
  return shortList(peoples.length ? peoples.map(factionName) : names);
}

/** At most two names, then "+N", so an eight-seat game still fits the menu on one line. */
function shortList(names: string[]): string {
  return names.length > 2 ? `${names.slice(0, 2).join(', ')} +${names.length - 2}` : names.join(' & ');
}

function ResumeButton() {
  const goto = useGame((s) => s.goto);
  const views = useGame((s) => s.views);
  if (!views) return null;
  const peoples = shortList([...new Set(views[0].players.filter((p) => p.is_human).map((p) => p.faction))].map(factionName));
  return (
    <button className="btn big primary title-resume" onClick={() => goto('game')}>
      Resume
      <span className="btn-sub" title={CALENDAR_NOTE}>{peoples ? `${peoples} · ` : ''}Turn {views[0].turn} · {calendarLabel(views[0].turn)}</span>
    </button>
  );
}

function MainMenu({ onMultiplayer, onHostSave }: { onMultiplayer: () => void; onHostSave: (save: SaveFile) => void }) {
  const online = useGame((s) => s.online);
  const openSetup = useGame((s) => s.openSetup);
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
  const builds = useSiteBuilds();
  // An autosave made with other rules continues in the build that made it, while the site still serves it.
  const elsewhere = auto && !saveHasCurrentRules(auto.game) ? buildForSave(builds, saveRulesVersion(auto.game), auto.build) : null;
  const olderAuto = !!auto && !elsewhere && (!saveHasCurrentRules(auto.game) || olderAutosaves.has(auto.savedAt));
  // New games start on the newest version; this tab may be an older one (finishing a game, or open since a deploy).
  const behind = !views && isBehind(builds);
  const startSolo = () => (behind ? navigate(newestUrl({ menu: 'solo' })) : openSetup('solo'));
  const openMultiplayer = () => (behind ? navigate(newestUrl({ menu: 'multi' })) : onMultiplayer());
  const unreadableAuto = slots.length > 0 && !auto;
  // While a game is open, Resume returns to it. Continue shows beside it only for a different saved game.
  const client = useGame((s) => s.client);
  const autoIsOpen = !!views && (!slots.length || client?.kind !== 'remote' && slots[0].record.campaign === currentCampaign());
  // A game with other players carries on only as an online host: Continue takes it there.
  const autoNeedsHost = !!auto && needsOnlineHost(auto);
  const autoBlocked = autoNeedsHost && !online?.relay;
  const continueAuto = async (record: AutosaveRecord, save: SaveFile) => {
    if (needsOnlineHost(save) && online?.relay) return onHostSave(save);
    await continueAutosave(record, save);
    const failure = useGame.getState().error;
    if (isOlderVersionMessage(failure)) olderAutosaves.add(save.savedAt);
    else if (failure && !needsOnlineHost(save)) {
      unreadableAutosaves.add(record.id);
      if (slots.some((slot) => slot.save && !unreadableAutosaves.has(slot.record.id))) set({ error: null });
    }
    if (failure) refresh((n) => n + 1);
  };
  // A page another build opened: continue its autosave, open the save it handed over, or start what was asked.
  useEffect(() => {
    const launch = pageLaunch();
    if (!launch || launchHandled || launch.kind === 'menu' && launch.menu === 'multi') return;
    if (launch.kind === 'continue') {
      if (records === null) return;
      launchHandled = true;
      const slot = usable.find((s) => s.record.campaign === launch.campaign);
      if (slot?.save) void continueAuto(slot.record, slot.save);
    } else if (launch.kind === 'open') {
      launchHandled = true;
      void takeHandedOffSave().then((text) => {
        if (!text) return;
        try {
          void loadSave(parseSaveFile(text), true);
        } catch (e) {
          set({ error: loadErrorMessage(e) });
        }
      });
    } else if (launch.kind === 'join') {
      if (!online?.relay) return;
      launchHandled = true;
      void useGame.getState().joinOnline(online.relay, launch.room).then((problem) => { if (problem) set({ error: problem }); });
    } else {
      // Never sent on again from here: the page that asked already chose this version.
      launchHandled = true;
      openSetup('solo');
    }
  });
  // A file chosen with Load Game replaces the saved game in progress, so it asks first, like a new game does.
  const [chosen, setChosen] = useState<{ save: SaveFile; saved: { label: string; turn: number } } | null>(null);
  const askThenLoad = async (save: SaveFile) => {
    if (needsOnlineHost(save)) {
      if (online?.relay) onHostSave(save);
      else set({ error: OTHER_PLAYERS_OFFLINE });
      return;
    }
    const [newest] = await listAutosaves();
    if (newest) setChosen({ save, saved: { label: newest.faction || newest.label, turn: newest.turn } });
    else await loadSave(save, true);
  };
  // While a message shows, starting a new game is the way forward, so it takes the primary slot.
  const newGamePrimary = !views && records !== null && (!auto || !!error || olderAuto || unreadableAuto || autoBlocked);
  const continuePrimary = !views && !!auto && !error && !olderAuto && !unreadableAuto && !autoBlocked;
  // Carrying on a game has one place: the top of the menu, as Resume or as the primary Continue.
  const continueButton = autoIsOpen ? null : (
    <button className={`btn big ${continuePrimary ? 'primary' : ''}`} disabled={!auto || !!busy || olderAuto || unreadableAuto || autoBlocked} onClick={() => auto && usable[0] && continueAuto(usable[0].record, auto)}>
      {olderAuto ? 'Saved game (older version)' : unreadableAuto ? 'Saved game (unreadable)' : 'Continue'}
      <span className="btn-sub">
        {olderAuto ? 'Cannot continue with these rules' : unreadableAuto ? "This save file is damaged"
          // The reason (online play is unavailable) is said once, under Multiplayer.
          : autoBlocked ? 'This game has other players'
          : auto && autoNeedsHost ? `${peoplesLabel(usable[0].record.faction, auto.names)} · Turn ${auto.turn} · host it online, the others rejoin`
          : auto && elsewhere ? `${peoplesLabel(usable[0].record.faction, auto.names)} · Turn ${auto.turn} · opens in the version it started on`
          : auto ? `${peoplesLabel(usable[0].record.faction, auto.names)} · Turn ${auto.turn} · ${calendarLabel(auto.turn)} · ${new Date(auto.savedAt).toLocaleString(undefined, { dateStyle: 'short', timeStyle: 'short' })}` : records === null ? 'Looking for your game…' : 'No autosave yet'}
      </span>
    </button>
  );
  return (
    <>
      {chosen && <ReplaceGameDialog file saved={chosen.saved} onCancel={() => setChosen(null)}
        onStart={() => { const { save } = chosen; setChosen(null); void loadSave(save, true); }} />}
      <TitleNotice error={error ?? (olderAuto ? olderAutosaveMessage(auto?.playtest) : skipped !== null ? `Your latest save (turn ${skipped}) could not be opened. Continue opens turn ${auto?.turn} instead.` : null)} />
      <div className="title-actions">
        <ResumeButton />
        {continuePrimary && continueButton}
        <button className={`btn big ${newGamePrimary ? 'primary' : ''}`} onClick={startSolo}>
          Single Player
          <span className="btn-sub">{behind ? 'New games start on the newest version' : `You against ${MIN_SEATS - 1} to ${MAX_PLAYERS - 1} AI empires`}</span>
        </button>
        <button className="btn big" disabled={!online?.relay} onClick={openMultiplayer}>
          Multiplayer
          <span className="btn-sub">{online?.relay ? 'Host or join a game online' : 'Online play is unavailable in this version'}</span>

        </button>
        {!continuePrimary && continueButton}
        <button className="btn big quiet" onClick={() => pickSaveFile(askThenLoad, (m) => set({ error: m }))}>
          Load Game…
        </button>
        <button className="btn big quiet" onClick={openGuide}>How to Play</button>
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

function MultiplayerMenu({ hostSave, onBack }: { hostSave: SaveFile | null; onBack: () => void }) {
  const views = useGame((s) => s.views);
  const goto = useGame((s) => s.goto);
  const error = useGame((s) => s.error);
  const online = useGame((s) => s.online);
  const refusal = useGame((s) => s.hostRefusal);
  // Host Game runs the game in this tab and opens its online room.
  const [hosting, setHosting] = useState<'idle' | 'starting' | 'failed'>('idle');
  // Host from a save has its own busy and failed state, so a retry stays on the button the player pressed.
  const [resuming, setResuming] = useState<'idle' | 'starting' | 'failed'>('idle');
  const busy = hosting === 'starting' || resuming === 'starting';
  const [hostError, setHostError] = useState<string | null>(null);
  const [name, setName] = useState(() => savedPlayerName() ?? DEFAULT_PLAYER_NAME);
  // The game list drops symbols and emoji; say so before the name is used.
  // An optional code that locks the game: guests type it once to take a seat. It stays in this tab.
  const [code, setCode] = useState('');
  const codeNote = joinCodeProblem(code);
  const typed = name.trim();
  const shown = cleanRoomName(typed);
  const listedAs = typed && shown !== typed ? (shown ? `Shown in the game list as “${shown}”.` : 'Listed as “Game” and four characters.') : null;
  // After a refusal that pressing Host Game again cannot fix, Host Game is not the next action: Join Game is.
  const hostCannotRetry = hosting === 'failed' && !!refusal && retryWouldFail(refusal.kind);
  const hostPrimary = hosting === 'failed' && !views && !hostCannotRetry;
  // A save brought from the main menu makes hosting it the next action.
  const savePrimary = !views && (!!hostSave || resuming === 'failed') && !hostPrimary;
  // The doors are equal until the game knows which one this player needs: after a refusal Host Game cannot fix,
  // Join is the way on.
  const joinPrimary = !views && !busy && !savePrimary && hostCannotRetry;
  const hostOnline = async () => {
    if (codeNote) return;
    const who = name.trim().slice(0, MAX_PLAYER_NAME) || DEFAULT_PLAYER_NAME;
    rememberPlayerName(who);
    setHosting('starting');
    setResuming('idle');
    setHostError(null);
    const failure = await useGame.getState().hostOnline(who, code || undefined);
    if (failure) {
      setHostError(failure);
      setHosting('failed');
    }
  };

  const hostFromSave = () => {
    if (codeNote) return;
    const who = name.trim().slice(0, MAX_PLAYER_NAME) || DEFAULT_PLAYER_NAME;
    rememberPlayerName(who);
    setHostError(null);
    setHosting('idle');
    const host = async (save: SaveFile) => {
      setResuming('starting');
      const failure = await useGame.getState().hostOnlineFromSave(who, save, code || undefined);
      if (failure) {
        setHostError(failure);
        setResuming('failed');
      }
    };
    // The save chosen on the main menu is hosted as it is; otherwise pick the file here.
    if (hostSave) void host(hostSave);
    else void pickSaveFile(host, (message) => { setHostError(message); setResuming('failed'); });
  };

  return (
    <>
      <TitleNotice error={error ?? hostError} />
      <div className="title-actions">
        <ResumeButton />
        {online?.relay ? (
          <>
            <label className="host-name mp-name">
              {/* The note replaces the label while it shows, so it adds no height to a menu that is already tight. */}
              {listedAs === null ? <span>Your name</span> : <span className="host-name-note">{listedAs}</span>}
              <input
                className="lobby-input"
                aria-label="Your name"
                value={name}
                maxLength={MAX_PLAYER_NAME}
                disabled={busy}
                onChange={(e) => { setName(e.target.value); rememberPlayerName(e.target.value); }}
              />
            </label>
            {/* Two doors, side by side: one player hosts, the others join. */}
            <div className="mp-doors">
              <section className="mp-door" aria-labelledby="mp-host-head">
                <h3 id="mp-host-head" className="mp-door-head">Host</h3>
                <p className="mp-door-lede">Set up the game. Others join you.</p>
                <label className="host-name host-code">
                  <span>Lock with a code</span>
                  <input
                    className="lobby-input"
                    aria-label="Join code (optional)"
                    placeholder="Optional"
                    value={code}
                    maxLength={JOIN_CODE_MAX}
                    autoComplete="off"
                    autoCapitalize="off"
                    spellCheck={false}
                    disabled={busy}
                    onChange={(e) => setCode(typedJoinCode(e.target.value))}
                    onKeyDown={(e) => { if (e.key === 'Enter' && !busy) void hostOnline(); }}
                  />
                </label>
                <button className={`btn big mp-door-main ${hostPrimary ? 'primary' : ''}`} disabled={busy || !!codeNote} onClick={hostOnline}>
                  Host Game
                  <span className="btn-sub">
                    {hosting === 'starting' ? <><span className="spinner" /> Opening your game…</>
                      : codeNote ? codeNote
                      : hosting === 'failed' ? (refusal ? retryLine(refusal.kind, refusal.retryAfterS) : 'Try again') : 'Start a new game'}
                  </span>
                </button>
                <button className={`btn big ${savePrimary ? 'primary' : ''}`} disabled={busy || !!codeNote} onClick={hostFromSave}>
                  {hostSave ? 'Host this save' : 'Host from a save…'}
                  <span className="btn-sub">
                    {resuming === 'starting' ? <><span className="spinner" /> Opening your game…</>
                      : codeNote ? codeNote
                      : resuming === 'failed' ? 'Try again'
                      : hostSave ? `${peoplesLabel(undefined, hostSave.names)} · Turn ${hostSave.turn}` : 'Carry on a saved game'}
                  </span>
                </button>
              </section>
              <section className="mp-door" aria-labelledby="mp-join-head">
                <h3 id="mp-join-head" className="mp-door-head">Join</h3>
                <p className="mp-door-lede">Take a seat in a game someone is hosting.</p>
                <button className={`btn big mp-door-main ${joinPrimary ? 'primary' : ''}`} disabled={busy} onClick={() => goto('online')}>
                  Join Game
                  <span className="btn-sub">See the open games</span>
                </button>
              </section>
            </div>
          </>
        ) : (
          <p className="title-hosted-note">Online play is unavailable in this version.</p>
        )}
        <button className="btn quiet" onClick={onBack}>
          ← Back
        </button>
      </div>
    </>
  );
}

/** Settings open in a modal over the title, so the menu never changes size. */
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
