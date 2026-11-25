import { awayLine, awayPlayers } from '../awayPlayers';
import { TrainingCard } from '../ui/Training';
import { WhyCard } from '../ui/WhyCard';
import { CelebrationProvider, MomentLane } from '../celebrate/MomentLane';
import { AlarmScreen } from '../celebrate/AlarmScreen';
import { PeopleIntro } from '../celebrate/PeopleIntro';
import { lazy, Suspense, useEffect, useState } from 'react';
import { canUse3d, needsFlatMap, useMapView } from '../map3d/mapPrefs';
import { useGame } from '../store';
import { MapCanvas } from '../map/MapCanvas';

// The 3D map loads only for players who switch it on (three.js stays out of the 2D download).
const Map3DView = lazy(() => import('../map3d/Map3DView'));
import { Minimap } from '../map/Minimap';
import { PingControl } from '../ui/PingControl';
import { TopBar } from '../ui/TopBar';
import { VpTrack } from '../ui/VpTrack';
import { UnitPanel } from '../ui/UnitPanel';
import { AirWing } from '../ui/AirWing';
import { CityPanel } from '../ui/CityPanel';
import { SeatBar } from '../ui/SeatBar';
import { TechPanel } from '../ui/TechPanel';
import { ObjectiveSheet } from '../ui/ObjectiveSheet';
import { CitiesOverview } from '../ui/CitiesOverview';
import { DiplomacyPanel } from '../ui/DiplomacyPanel';
import { DeclareWarPrompt } from '../ui/DeclareWarPrompt';
import { GoalActionDialog, useGoalActionKey } from '../ui/GoalActions';
import { FreeTechPicker, FreeTechsPrompt } from '../ui/FreeTechs';
import { DecisionNotice } from '../ui/DecisionNotice';
import { SaveNotice } from '../ui/SaveNotice';
import { TurnFailureDialog } from '../ui/TurnFailureDialog';
import { EraNotice } from '../ui/EraNotice';
import { TurnReport } from '../ui/TurnReport';
import { GameOver } from '../ui/GameOver';
import { plainOrderError } from '../format';
import { handleAircraftKey, handleGameKey } from './gameKeyHandler';
import { handleOverlayKey, notePointerDown, returnFocusAfterPointer } from '../gameplayKeys';
import { playerColor } from '../theme';
import { Advisor } from '../ui/Advisor';
import { ReclaimSeatButton } from '../ui/ReclaimSeatButton';
import '../ui/sidebar.css';

export function GameErrorToast() {
  const error = useGame((s) => s.error);
  const set = useGame((s) => s.set);
  if (!error) return null;
  return (
    <div className={`toast game-error-toast ${error.startsWith('Save updated:') ? 'good' : 'bad'}`} role="alert">
      <span className="toast-message">{plainOrderError(error)}</span>
      <button type="button" className="toast-close" aria-label="Dismiss" onClick={() => set({ error: null })}>×</button>
    </div>
  );
}

export function GameScreen() {
  const views = useGame((s) => s.views);
  const seat = useGame((s) => s.seat);
  const selection = useGame((s) => s.selection);
  const busy = useGame((s) => s.busy);
  const training = useGame((s) => s.training);
  const resolvingToEnd = useGame((s) => s.resolvingToEnd);
  const stopResolving = useGame((s) => s.stopResolving);
  const conn = useGame((s) => (s.client?.kind === 'remote' ? s.net?.conn : null));
  const away = useGame((s) => awayLine(awayPlayers(s), 'until'));
  const credentialRejected = useGame((s) => s.client?.kind === 'remote' && s.net?.credentialRejected);

  useEffect(() => {
    window.addEventListener('keydown', handleGameKey);
    // Capture phase: Return and Escape close what covers the map before any panel sees them.
    window.addEventListener('keydown', handleOverlayKey, true);
    // Capture phase, after the overlay handler: aircraft keys beat map panning and the ground-unit keys (gameKeyHandler.ts).
    window.addEventListener('keydown', handleAircraftKey, true);
    window.addEventListener('pointerdown', notePointerDown, true);
    // Capture phase: runs before the control's own click handler (see gameplayKeys.ts).
    window.addEventListener('click', returnFocusAfterPointer, true);
    return () => {
      window.removeEventListener('keydown', handleGameKey);
      window.removeEventListener('keydown', handleOverlayKey, true);
      window.removeEventListener('keydown', handleAircraftKey, true);
      window.removeEventListener('pointerdown', notePointerDown, true);
      window.removeEventListener('click', returnFocusAfterPointer, true);
    };
  }, []);

  useGoalActionKey();
  const mapKind = useMapView((m) => m.kind);
  const [can3d] = useState(canUse3d);
  const picking = useGame(needsFlatMap);
  const show3d = can3d && mapKind === '3d' && !picking;
  if (!views) return null;
  const view = views[seat];
  const unit = selection?.kind === 'unit' ? view.units.find((u) => u.id === selection.id) : null;
  const city = selection?.kind === 'city' ? view.cities.find((c) => c.id === selection.id) : null;
  const plane = selection?.kind === 'air' ? view.aircraft?.find((a) => a.id === selection.id) : null;
  const airCity = plane ? view.cities.find((c) => c.id === plane.city) : null;

  return (
    <CelebrationProvider><div className="game" style={{ ['--seat' as string]: playerColor(view.me) }}>
      {show3d ? <Suspense fallback={<MapCanvas />}><Map3DView /></Suspense> : <MapCanvas />}
      <div className="seat-frame" />
      <TopBar />
      <MomentLane />
      <ObjectiveSheet />
      <aside className="right-sidebar paper-grain" aria-label="Empire sidebar"><SeatBar race={<VpTrack />} /><Minimap /><PingControl /></aside>
      <div className="command-docks">
      <div className="left-dock" data-overlay-slot="selection-dock">
        {selection && <button className="btn quiet selection-close touch-only" onClick={() => useGame.getState().select(null)}>Close selection</button>}
        {unit && <UnitPanel unit={unit} view={view} />}
        {plane && airCity && <AirWing city={airCity} view={view} planeId={plane.id} />}
        {city && <CityPanel city={city} view={view} />}
      </div>
      </div>
      <TurnReport />
      <DiplomacyPanel />
      <DeclareWarPrompt />
      <GoalActionDialog />
      <FreeTechPicker />
      <DecisionNotice />
      <SaveNotice />
      <TurnFailureDialog />
      <EraNotice />
      <TechPanel />
      <CitiesOverview />
      <GameOver />
      <AlarmScreen />
      <PeopleIntro />
      <FreeTechsPrompt />
      <Advisor />
      <WhyCard />
      {busy && (
        <div className={`busy${training?.waiting ? ' training-wait' : ''}`}>
          <div className="busy-card" role="status">
            {training?.waiting ? <TrainingCard /> : <><div className="spinner" />{busy}</>}
            {resolvingToEnd && <button className="btn" onClick={stopResolving}>Stop after this turn</button>}
          </div>
        </div>
      )}
      {credentialRejected ? (
        <div className="conn-banner">
          Your saved seat credential is no longer valid.
          <ReclaimSeatButton />
          <button className="btn quiet" onClick={() => useGame.getState().leaveOnline()}>Leave</button>
        </div>
      ) : conn === 'taken' ? (
        <div className="conn-banner">
          This seat is now being played in another window.
          <button className="btn" onClick={() => useGame.getState().rejoinOnline()}>
            Rejoin here
          </button>
          <button className="btn quiet" onClick={() => useGame.getState().leaveOnline()}>
            Leave
          </button>
        </div>
      ) : conn === 'open' && away ? (
        <div className="conn-banner away-banner" role="status" data-testid="away-banner">
          <span className="conn-dot" aria-hidden /> {away}
        </div>
      ) : (
        conn &&
        conn !== 'open' &&
        conn !== 'closed' && (
          <div className="conn-banner">
            <div className="spinner" /> Lost the connection to the host. Reconnecting… your orders are kept.
          </div>
        )
      )}
      {conn !== 'taken' && <GameErrorToast />}
    </div></CelebrationProvider>
  );
}
