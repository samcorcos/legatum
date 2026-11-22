// The game lobby: a staging room like any 4X's. A seats table (who plays, which
// faction, which team, how hard each AI plays), game settings and the victory targets beside it.
// Hosting a network game, the same screen is the room: every edit is
// sent to the server, and network players take the open seats and Ready up before the host starts.
import { useEffect, useMemo, useRef, useState, type ReactNode } from 'react';
import { useGame } from '../store';
import { listAutosaves } from '../platform/autosaveStore';
import { ReplaceGameDialog } from '../ui/ReplaceGameDialog';
import { rememberedSeat, rememberSeat } from '../remote';
import { factionColor } from '../theme';
import { Tip } from '../ui/Tip';
import { HexBackdrop } from './HexBackdrop';
import {
  activeCount,
  aiAlliancesOn,
  buildSetup,
  mapSizeLabel,
  newLobby,
  setSlot,
  slotColorFactions,
  problems,
  rememberDifficulty,
  levelOf,
  teams,
  type Lobby,
  type LobbyMode,
} from '../lobby';
import { closeStaging, configFromLobby, everyoneReady, lobbyFromStaging, withOccupants, stagingEnabled, startStaging, settingsAcknowledged, StagingSettingsWriter, type StagingConfig } from '../staging';
import { HOLD_TURNS, turnsWord } from '../victoryHold';
import { Amount } from '../ui/Amounts';
import { SeatsTable } from './lobby/SeatsTable';
import { Settings } from './lobby/Settings';
import { randomSeed } from './lobby/randomSeed';
import { rememberPlayerName, savedPlayerName } from '../platform/online';
import { HOST_AWAY } from '../platform/relayLobby';
import '../lobby.css';

const TITLES: Record<LobbyMode, string> = { solo: 'Single Player', hotseat: 'Hot Seat', host: 'Host Game' };
function forgetHostRoom(roomId: string | undefined) {
  const saved = rememberedSeat();
  if (roomId && saved?.seat === 0 && saved.roomId === roomId) rememberSeat(null);
}


/** An online host's first seat carries the name typed for the game list. */
function hostNamed(lobby: Lobby, mode: LobbyMode): Lobby {
  const name = mode === 'host' ? savedPlayerName() : null;
  return name ? setSlot(lobby, 0, { name }) : lobby;
}

export function LobbyScreen() {
  const mode = useGame((s) => s.setupMode);
  const status = useGame((s) => s.net?.status);
  const startGame = useGame((s) => s.startGame);
  const hostNewGame = useGame((s) => s.hostNewGame);
  const goto = useGame((s) => s.goto);
  const set = useGame((s) => s.set);
  const busy = useGame((s) => s.busy);
  const error = useGame((s) => s.error);
  const [replacing, setReplacing] = useState<{ label: string; turn: number } | null>(null);
  const checking = useRef(false);
  const [restoringRoomId, setRestoringRoomId] = useState(() => {
    const saved = rememberedSeat();
    return mode === 'host' && status?.is_local && saved?.seat === 0 && saved.token ? saved.roomId : undefined;
  });
  const room = mode === 'host' && (!!restoringRoomId || stagingEnabled(status));
  const [draft, setDraft] = useState<{ lobby: Lobby | null; writeSettings: boolean }>(() => ({
    lobby: restoringRoomId ? null : hostNamed(newLobby(mode, randomSeed()), mode),
    writeSettings: !restoringRoomId,
  }));
  const { lobby, writeSettings } = draft;
  const setLobby = (next: Lobby) => {
    // The host's own name is the one the game list shows next time.
    if (mode === 'host' && next.slots[0]?.name !== lobby?.slots[0]?.name) rememberPlayerName(next.slots[0]?.name ?? '');
    const changedLevel = lobby && (levelOf(next) !== levelOf(lobby) || next.slots.some((slot, i) => slot.kind === 'ai' && levelOf(slot) !== levelOf(lobby.slots[i] ?? lobby)));
    if (!rememberDifficulty(next) && changedLevel) set({ error: 'AI level could not be saved on this device. It will reset when you leave this lobby.' });
    setDraft({
      lobby: next,
      writeSettings: writeSettings || JSON.stringify(configFromLobby(next)) !== JSON.stringify(lobby && configFromLobby(lobby)),
    });
  };
  const { staging, synced, stop } = useStagingHost(room, lobby, writeSettings);
  useEffect(() => {
    if (lobby || !restoringRoomId) return;
    if (staging?.room_id === restoringRoomId && staging.seed !== undefined) {
      // Wait for the host-only snapshot. Posting public settings or new defaults would
      // replace the seed; copying guests' choices into config would reset their Ready flags.
      setDraft({ lobby: lobbyFromStaging(staging, false), writeSettings: false });
    } else if (!staging && status && !status.staging &&
      // A room may have started during reload. Let its credential recover that handoff;
      // staging_closed/no_room clears the credential if there is no game seat to recover.
      (!status.has_game || rememberedSeat()?.roomId !== restoringRoomId)) {
      forgetHostRoom(restoringRoomId);
      // Discard the client's in-memory credential too, before claiming a new host seat.
      useGame.getState().exitStaging();
      if (stagingEnabled(status)) useGame.getState().enterStaging(true);
      setRestoringRoomId(undefined);
      setDraft({ lobby: hostNamed(newLobby(mode, randomSeed()), mode), writeSettings: true });
      set({ error: null });
    }
  }, [lobby, mode, restoringRoomId, staging, status, set]);

  const back = async () => {
    // Read the live state so repeated clicks cannot queue another room deletion.
    if (useGame.getState().busy) return;
    if (room) {
      const closingRoomId = useGame.getState().staging?.room_id ?? restoringRoomId ?? rememberedSeat()?.roomId;
      set({ busy: 'Closing the game room…', error: null });
      useGame.getState().exitStaging();
      // Drain settings and close before showing Host Game again: a delayed DELETE would
      // otherwise close the next room. Keep the existing best-effort handling of close errors.
      await stop().then(() => closeStaging()).then(() => forgetHostRoom(closingRoomId)).catch(() => {});
      set({ busy: null });
    }
    goto('title');
  };

  if (!lobby) {
    return (
      <LobbyFrame title={TITLES[mode]} onBack={back} backDisabled={!!busy} error={error}>
        <div className="waiting-host staging-wait">
          <div className="spinner" />
          Restoring the game room…
        </div>
      </LobbyFrame>
    );
  }

  // What everyone sees: network players' own names and factions in their seats.
  const shown = staging ? withOccupants(lobby, staging) : lobby;
  const blockers = problems(lobby);
  if (room && !staging) blockers.push('Opening the game room…');
  else if (room && !synced) blockers.push('Saving the latest settings…');
  else if (room && staging && !everyoneReady(staging)) blockers.push('Waiting for every online seat to be taken and Ready.');

  const start = () => {
    if (blockers.length || busy) return;
    if (room) {
      set({ busy: 'Starting…', error: null });
      startStaging().catch((e) => set({ busy: null, error: `Could not start: ${e instanceof Error ? e.message : String(e)}` }));
      return;
    }
    const setup = buildSetup(lobby);
    if (mode === 'host') { hostNewGame(setup); return; }
    // A saved game is only replaced after asking; the new game's first save is what replaces it.
    if (checking.current) return;
    checking.current = true;
    void listAutosaves().then(([newest]) => {
      checking.current = false;
      if (newest) setReplacing({ label: newest.faction || newest.label, turn: newest.turn });
      else startGame(setup, { training: lobby.training === true });
    });
  };

  // The Start button lives in a dock below the scrolling lobby, never on top of its settings.
  const dock = (
    <>
      <div className="lobby-dock-status" role="status">
        {blockers.map((p) => (
          <div key={p} className="lobby-problem">
            {p}
          </div>
        ))}
      </div>
      <button className="btn big primary lobby-go" disabled={blockers.length > 0 || !!busy} onClick={start}>
        {busy ? busy : mode === 'host' && !room ? 'Host the game' : 'Start game'}
        <span className="btn-sub">
          {activeCount(lobby)} empires · {mapSizeLabel(lobby.mapSize).toLowerCase()} map{lobby.barbarians === false ? ' · no barbarians' : ''}{aiAlliancesOn(lobby) ? ' · AI empires allied in pairs' : ''}
        </span>
      </button>
      {replacing && <ReplaceGameDialog saved={replacing} onCancel={() => setReplacing(null)}
        onStart={() => { setReplacing(null); startGame(buildSetup(lobby), { training: lobby.training === true }); }} />}
    </>
  );

  return (
    <LobbyFrame title={TITLES[mode]} onBack={back} backDisabled={!!busy} error={error} dock={dock}>
      <div className="lobby-body">
        <SeatsTable lobby={lobby} shown={shown} onChange={setLobby} room={staging ? { state: staging, guest: false } : undefined} />
        <aside className="lobby-side">
          <Settings lobby={lobby} onChange={setLobby} />
          <Victory lobby={shown} />
        </aside>
      </div>
    </LobbyFrame>
  );
}

/** Host side of the staging room: open it, keep its settings in sync, and sit in the host seat. */
function useStagingHost(room: boolean, lobby: Lobby | null, writeSettings: boolean) {
  const staging = useGame((s) => s.staging);
  const mySeat = useGame((s) => s.stagingSeat);
  const config = useMemo(() => lobby ? configFromLobby(lobby) : null, [lobby]);
  const [acknowledged, setAcknowledged] = useState<StagingConfig | null>(null);
  const writer = useRef<StagingSettingsWriter | null>(null);
  useEffect(() => {
    if (!room) return;
    useGame.getState().enterStaging(true);
    writer.current = new StagingSettingsWriter((saved) => {
      const me = saved.seats[0];
      const g = useGame.getState();
      if (g.stagingSeat !== 0) g.takeStagingSeat(0, me.name);
      g.updateStagingSeat({ name: me.name, faction: me.faction });
      setAcknowledged(saved);
      g.set({ error: null });
    }, (e) => useGame.getState().set({ error: `Could not open the game room: ${e instanceof Error ? e.message : String(e)}` }));
    return () => { void writer.current?.stop(); };
  }, [room]);
  useEffect(() => {
    if (!room || !config) return;
    // A private snapshot is already saved. Only new rooms and host edits need a POST.
    if (writeSettings) writer.current?.update(config);
    else {
      // A reload can land after the settings POST but before the host's seat update.
      // Reconcile only our reservation; reposting config would reset guests' readiness.
      const host = useGame.getState().staging?.occupants[0];
      const me = config.seats[0];
      if (host && (host.name !== (me.name.trim() || 'Player') || host.faction !== me.faction)) {
        useGame.getState().updateStagingSeat({ name: me.name, faction: me.faction });
      }
      setAcknowledged(config);
    }
  }, [room, config, writeSettings]);
  return {
    staging: room && mySeat === 0 ? staging : null,
    synced: !!config && settingsAcknowledged(config, acknowledged, staging),
    stop: () => writer.current?.stop() ?? Promise.resolve(),
  };
}

export function LobbyFrame({ title, onBack, backDisabled = false, error, dock, children }: { title: string; onBack: () => void; backDisabled?: boolean; error: string | null; dock?: ReactNode; children: ReactNode }) {
  return (
    <div className="setup-screen lobby-screen">
      <HexBackdrop />
      <div className="lobby-inner">
        <div className="setup-top">
          <button className="btn quiet" onClick={onBack} disabled={backDisabled}>
            ← Back
          </button>
          <h2>{title}</h2>
          <div />
        </div>
        {error === HOST_AWAY ? (
          <div className="toast bad inline host-away" role="status">
            <div>{error}</div>
            <div className="host-away-wait"><span className="spinner" /> Waiting for the host to come back…</div>
          </div>
        ) : error && <div className="toast bad inline" role="alert">{error}</div>}
        {children}
      </div>
      {dock && (
        <footer className="lobby-dock">
          <div className="lobby-dock-inner">{dock}</div>
        </footer>
      )}
    </div>
  );
}


export function Victory({ lobby }: { lobby: Lobby }) {
  const ts = teams(lobby);
  const colored = slotColorFactions(lobby);
  return (
    <section className="lobby-card">
      <div className="lobby-card-head">
        <h3>Victory</h3>
      </div>
      <p className="lobby-victory-lede">
        A team wins by reaching its VP target and holding it for {HOLD_TURNS} turns{lobby.turnLimit === null ? ". No turn limit." : `; otherwise the most VP at turn ${lobby.turnLimit}.`} Teammates share vision and victory.
      </p>
      <div className="team-targets">
        {ts.map((t) => (
          <div key={t.lobbyTeam} className="team-target">
            <span className="team-name">Team {t.lobbyTeam + 1}</span>
            <span className="team-members">
              {t.slots.map((s) => (
                <span key={s} className="team-member" style={{ ['--pc' as string]: colored[s] ? factionColor(colored[s]!) : '#3a3f47' }}>
                  {lobby.slots[s].name || `Seat ${s + 1}`}
                </span>
              ))}
            </span>
            <Tip content={`${t.slots.length === 1 ? 'A one-empire team needs 10 VP.' : `A team needs 10 VP per empire: ${t.slots.length} empires need ${t.target}.`} Hold it for ${turnsWord(HOLD_TURNS)} to win.`}>
              <b className="team-vp"><Amount n={t.target} unit="VP" /></b>
            </Tip>
          </div>
        ))}
      </div>
    </section>
  );
}
