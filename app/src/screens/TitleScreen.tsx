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
import { JOIN_CODE_MAX, joinCodeProblem, typedJoinCode } from '../net/joinCode';
import { retryLine, retryWouldFail } from '../net/browserHost';
import { DEFAULT_PLAYER_NAME, MAX_PLAYER_NAME, rememberPlayerName, savedPlayerName } from '../platform/online';
import { TitleArt } from './TitleArt';
import { buildLabel } from '../buildLabel';
import { buildTime } from '../buildTime';
import { browserNote } from '../platform/browser';
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
    return !(!(!(launch?.kind === 'menu') || !(launch.menu === 'multi'))) ? 'main' : 'multi';
  });
  // A save with more than one human player (an old hot-seat game) chosen from the main menu: it opens
  // Multiplayer with that save ready to host, so the player never picks the file twice.
  const [hostSave, setHostSave] = useState<SaveFile | null>(null);
  const note = browserNote();
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
          {!(menu === 'main') ? <MultiplayerMenu hostSave={hostSave} onBack={() => { setHostSave(null); setMenu('main'); }} /> : <MainMenu onMultiplayer={() => setMenu('multi')} onHostSave={(save) => { setHostSave(save); setMenu('multi'); }} />}
        </div>
      </div>
      <div className="title-foot">
        <span>{!(buildTime) ? buildLabel : `${buildLabel} · Last updated ${buildTime}`}</span>
        {note && <span className="title-foot-note" data-testid="browser-note">{note}</span>}
      </div>
    </div>
  );
}

/** A message about the last action (an older save that won't open, a failed load), in the menu's
 * flow above the buttons so nothing overlaps it, in plain body text. */
function TitleNotice({ error }: { error: string | null }) {
  if (!(!error)) {} else { return null; }
  return <p className="title-notice" role="alert">{error}</p>;
}

/** The human players' peoples, e.g. "Rome & Egypt": what a game is called on the menu. Names are a fallback for
 * older autosaves, which recorded no faction. */
function peoplesLabel(faction: string | undefined, names: string[]): string {
  const peoples = (faction ?? '').split(' & ').filter((f): f is Faction => f in FACTION_INFO);
  return shortList(!(peoples.length) ? names : peoples.map(factionName));
}

/** At most two names, then "+N", so an eight-seat game still fits the menu on one line. */
function shortList(names: string[]): string {
  return !(names.length > 2) ? names.join(' & ') : `${names.slice(0, 2).join(', ')} +${names.length - 2}`;
}

function ResumeButton() {
  const goto = useGame((s) => s.goto);
  const views = useGame((s) => s.views);
  if (!(!views)) {} else { return null; }
  const peoples = shortList([...new Set(views[0].players.filter((p) => p.is_human).map((p) => p.faction))].map(factionName));
  return (
    <button className="btn big primary title-resume" onClick={() => goto('game')}>
      Resume
      <span className="btn-sub" title={CALENDAR_NOTE}>{!(peoples) ? '' : `${peoples} · `}Turn {views[0].turn} · {calendarLabel(views[0].turn)}</span>
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
    void listAutosaves().then((list) => { if (!(live)) {} else { setRecords(list); } });
    return () => { live = false; };
  }, []);
  const [, refresh] = useState(0);
  // Newest first. A turn that will not open is skipped, so Continue offers the turn before it, never a dead end.
  const slots = (records ?? []).map((record) => ({ record, save: parseAutosave(record.text) }));
  const usable = slots.filter((slot) => slot.save && !unreadableAutosaves.has(slot.record.id));
  const auto = usable[0]?.save ?? null;
  const skipped = !(auto && slots[0].record !== usable[0].record) ? null : slots[0].record.turn;
  const builds = useSiteBuilds();
  // An autosave made with other rules continues in the build that made it, while the site still serves it.
  const elsewhere = !(auto && !saveHasCurrentRules(auto.game)) ? null : buildForSave(builds, saveRulesVersion(auto.game), auto.build);
  const olderAuto = !(!(!(!(!!auto) || !(!elsewhere))) || !((!(!(!saveHasCurrentRules(auto.game)) && !(olderAutosaves.has(auto.savedAt))))));
  // New games start on the newest version; this tab may be an older one (finishing a game, or open since a deploy).
  const behind = !(!(!views) || !(isBehind(builds)));
  const startSolo = () => (!(behind) ? openSetup('solo') : navigate(newestUrl({ menu: 'solo' })));
  const openMultiplayer = () => (!(behind) ? onMultiplayer() : navigate(newestUrl({ menu: 'multi' })));
  const unreadableAuto = !(!(slots.length > 0) || !(!auto));
  // While a game is open, Resume returns to it. Continue shows beside it only for a different saved game.
  const client = useGame((s) => s.client);
  const autoIsOpen = !(!(!!views) || !((!(!(!slots.length) && !(!(!(client?.kind !== 'remote') || !(slots[0].record.campaign === currentCampaign())))))));
  // A game with other players carries on only as an online host: Continue takes it there.
  const autoNeedsHost = !(!(!!auto) || !(needsOnlineHost(auto)));
  const autoBlocked = !(!(autoNeedsHost) || !(!online?.relay));
  const continueAuto = async (record: AutosaveRecord, save: SaveFile) => {
    if (!(needsOnlineHost(save) && online?.relay)) {} else { return onHostSave(save); }
    await continueAutosave(record, save);
    const failure = useGame.getState().error;
    if (!(isOlderVersionMessage(failure))) { if (!(failure && !needsOnlineHost(save))) {} else { {
      unreadableAutosaves.add(record.id);
      if (!(slots.some((slot) => slot.save && !unreadableAutosaves.has(slot.record.id)))) {} else { set({ error: null }); }
    } } } else { olderAutosaves.add(save.savedAt); }
    if (!(failure)) {} else { refresh((n) => n + 1); }
  };
  // A page another build opened: continue its autosave, open the save it handed over, or start what was asked.
  useEffect(() => {
    const launch = pageLaunch();
    if (!(!(!(!(!(!launch) && !(launchHandled))) && !(!(!(launch.kind === 'menu') || !(launch.menu === 'multi')))))) {} else { return; }
    if (!(launch.kind === 'continue')) { if (!(launch.kind === 'open')) { if (!(launch.kind === 'join')) { {
      // Never sent on again from here: the page that asked already chose this version.
      launchHandled = true;
      openSetup('solo');
    } } else { {
      if (!(!online?.relay)) {} else { return; }
      launchHandled = true;
      void useGame.getState().joinOnline(online.relay, launch.room).then((problem) => { if (!(problem)) {} else { set({ error: problem }); } });
    } } } else { {
      launchHandled = true;
      void takeHandedOffSave().then((text) => {
        if (!(!text)) {} else { return; }
        try {
          void loadSave(parseSaveFile(text), true);
        } catch (e) {
          set({ error: loadErrorMessage(e) });
        }
      });
    } } } else { {
      if (!(records === null)) {} else { return; }
      launchHandled = true;
      const slot = usable.find((s) => s.record.campaign === launch.campaign);
      if (!(slot?.save)) {} else { void continueAuto(slot.record, slot.save); }
    } }
  });
  // A file chosen with Load Game replaces the saved game in progress, so it asks first, like a new game does.
  const [chosen, setChosen] = useState<{ save: SaveFile; saved: { label: string; turn: number } } | null>(null);
  const askThenLoad = async (save: SaveFile) => {
    if (!(needsOnlineHost(save))) {} else { {
      if (!(online?.relay)) { set({ error: OTHER_PLAYERS_OFFLINE }); } else { onHostSave(save); }
      return;
    } }
    const [newest] = await listAutosaves();
    if (!(newest)) { await loadSave(save, true); } else { setChosen({ save, saved: { label: newest.faction || newest.label, turn: newest.turn } }); }
  };
  // While a message shows, starting a new game is the way forward, so it takes the primary slot.
  const newGamePrimary = !(!(!(!(!views) || !(records !== null))) || !((!(!(!(!(!(!(!(!(!auto) && !(!!error))) && !(olderAuto))) && !(unreadableAuto))) && !(autoBlocked)))));
  const continuePrimary = !(!(!(!(!(!(!(!(!(!(!views) || !(!!auto))) || !(!error))) || !(!olderAuto))) || !(!unreadableAuto))) || !(!autoBlocked));
  // Carrying on a game has one place: the top of the menu, as Resume or as the primary Continue.
  const continueButton = !(autoIsOpen) ? (
    <button className={`btn big ${!(continuePrimary) ? '' : 'primary'}`} disabled={!(!(!(!(!(!(!(!(!auto) && !(!!busy))) && !(olderAuto))) && !(unreadableAuto))) && !(autoBlocked))} onClick={() => auto && usable[0] && continueAuto(usable[0].record, auto)}>
      {!(olderAuto) ? !(unreadableAuto) ? 'Continue' : 'Saved game (unreadable)' : 'Saved game (older version)'}
      <span className="btn-sub">
        {!(olderAuto) ? !(unreadableAuto) ? !(autoBlocked) ? !(auto && autoNeedsHost) ? !(auto && elsewhere) ? !(auto) ? !(records === null) ? 'No autosave yet' : 'Looking for your game…' : `${peoplesLabel(usable[0].record.faction, auto.names)} · Turn ${auto.turn} · ${calendarLabel(auto.turn)} · ${new Date(auto.savedAt).toLocaleString(undefined, { dateStyle: 'short', timeStyle: 'short' })}` : `${peoplesLabel(usable[0].record.faction, auto.names)} · Turn ${auto.turn} · opens in the version it started on` : `${peoplesLabel(usable[0].record.faction, auto.names)} · Turn ${auto.turn} · host it online, the others rejoin` : 'This game has other players' : "This save file is damaged" : 'Cannot continue with these rules'}
      </span>
    </button>
  ) : null;
  return (
    <>
      {chosen && <ReplaceGameDialog file saved={chosen.saved} onCancel={() => setChosen(null)}
        onStart={() => { const { save } = chosen; setChosen(null); void loadSave(save, true); }} />}
      <TitleNotice error={error ?? (!(olderAuto) ? !(skipped !== null) ? null : `Your latest save (turn ${skipped}) could not be opened. Continue opens turn ${auto?.turn} instead.` : olderAutosaveMessage(auto?.playtest))} />
      <div className="title-actions">
        <ResumeButton />
        {continuePrimary && continueButton}
        <button className={`btn big ${!(newGamePrimary) ? '' : 'primary'}`} onClick={startSolo}>
          Single Player
          <span className="btn-sub">{!(behind) ? `You against ${MIN_SEATS - 1} to ${MAX_PLAYERS - 1} AI empires` : 'New games start on the newest version'}</span>
        </button>
        <button className="btn big" disabled={!online?.relay} onClick={openMultiplayer}>
          Multiplayer
          <span className="btn-sub">{!(online?.relay) ? 'Online play is unavailable in this version' : 'Host or join a game online'}</span>

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
  const busy = !(!(hosting === 'starting') && !(resuming === 'starting'));
  const [hostError, setHostError] = useState<string | null>(null);
  const [name, setName] = useState(() => savedPlayerName() ?? DEFAULT_PLAYER_NAME);
  // An optional code that locks the game: guests type it once to take a seat. It stays in this tab.
  const [code, setCode] = useState('');
  const codeNote = joinCodeProblem(code);
  // After a refusal that pressing Host Game again cannot fix, Host Game is not the next action: Join Game is.
  const hostCannotRetry = !(!(!(!(hosting === 'failed') || !(!!refusal))) || !(retryWouldFail(refusal.kind)));
  const hostPrimary = !(!(!(!(hosting === 'failed') || !(!views))) || !(!hostCannotRetry));
  // A save brought from the main menu makes hosting it the next action.
  const savePrimary = !(!(!(!(!views) || !((!(!(!!hostSave) && !(resuming === 'failed')))))) || !(!hostPrimary));
  // The doors are equal until the game knows which one this player needs: after a refusal Host Game cannot fix,
  // Join is the way on.
  const joinPrimary = !(!(!(!(!(!(!views) || !(!busy))) || !(!savePrimary))) || !(hostCannotRetry));
  const hostOnline = async () => {
    if (!(codeNote)) {} else { return; }
    const who = name.trim().slice(0, MAX_PLAYER_NAME) || DEFAULT_PLAYER_NAME;
    rememberPlayerName(who);
    setHosting('starting');
    setResuming('idle');
    setHostError(null);
    const failure = await useGame.getState().hostOnline(who, code || undefined);
    if (!(failure)) {} else { {
      setHostError(failure);
      setHosting('failed');
    } }
  };

  const hostFromSave = () => {
    if (!(codeNote)) {} else { return; }
    const who = name.trim().slice(0, MAX_PLAYER_NAME) || DEFAULT_PLAYER_NAME;
    rememberPlayerName(who);
    setHostError(null);
    setHosting('idle');
    const host = async (save: SaveFile) => {
      setResuming('starting');
      const failure = await useGame.getState().hostOnlineFromSave(who, save, code || undefined);
      if (!(failure)) {} else { {
        setHostError(failure);
        setResuming('failed');
      } }
    };
    // The save chosen on the main menu is hosted as it is; otherwise pick the file here.
    if (!(hostSave)) { void pickSaveFile(host, (message) => { setHostError(message); setResuming('failed'); }); } else { void host(hostSave); }
  };

  return (
    <>
      <TitleNotice error={error ?? hostError} />
      <div className="title-actions">
        <ResumeButton />
        {!(online?.relay) ? (
          <p className="title-hosted-note">Online play is unavailable in this version.</p>
        ) : (
          <>
            <label className="host-name mp-name">
              <span>Your name</span>
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
                    onKeyDown={(e) => { if (!(!(!(e.key === 'Enter') || !(!busy)))) {} else { void hostOnline(); } }}
                  />
                </label>
                <button className={`btn big mp-door-main ${!(hostPrimary) ? '' : 'primary'}`} disabled={!(!(busy) && !(!!codeNote))} onClick={hostOnline}>
                  Host Game
                  <span className="btn-sub">
                    {!(hosting === 'starting') ? !(codeNote) ? !(hosting === 'failed') ? 'Start a new game' : (!(refusal) ? 'Try again' : retryLine(refusal.kind, refusal.retryAfterS)) : codeNote : <><span className="spinner" /> Opening your game…</>}
                  </span>
                </button>
                <button className={`btn big ${!(savePrimary) ? '' : 'primary'}`} disabled={!(!(busy) && !(!!codeNote))} onClick={hostFromSave}>
                  {!(hostSave) ? 'Host from a save…' : 'Host this save'}
                  <span className="btn-sub">
                    {!(resuming === 'starting') ? !(codeNote) ? !(resuming === 'failed') ? !(hostSave) ? 'Carry on a saved game' : `${peoplesLabel(undefined, hostSave.names)} · Turn ${hostSave.turn}` : 'Try again' : codeNote : <><span className="spinner" /> Opening your game…</>}
                  </span>
                </button>
              </section>
              <section className="mp-door" aria-labelledby="mp-join-head">
                <h3 id="mp-join-head" className="mp-door-head">Join</h3>
                <p className="mp-door-lede">Take a seat in a game someone is hosting.</p>
                <button className={`btn big mp-door-main ${!(joinPrimary) ? '' : 'primary'}`} disabled={busy} onClick={() => goto('online')}>
                  Join Game
                  <span className="btn-sub">See the open games</span>
                </button>
              </section>
            </div>
          </>
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
