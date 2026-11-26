import { useEffect } from 'react';
import { useGame } from './store';
import { TitleScreen } from './screens/TitleScreen';
import { LobbyScreen } from './screens/LobbyScreen';
import { JoinScreen } from './screens/JoinScreen';
import { OnlineLobbyScreen } from './screens/OnlineLobbyScreen';
import { StagingScreen } from './screens/StagingScreen';
import { GameScreen } from './screens/GameScreen';
import { Guide } from './ui/Guide';
import { LoadingScreen, useLoadingPlate } from './screens/LoadingScreen';
import { PerfOverlay } from './ui/PerfOverlay';
import { installPressFeedback } from './feel/pressFeedback';
import { ClientErrorScreen } from './screens/ClientErrorScreen';
import './feel/pressFeedback.css';

/** Store `busy` tasks that load the engine, generate a world or open a save: they get the full loading screen. */
const LOADING_TASKS = new Set(['Surveying the map…', 'Loading…']);

export function App() {
  useEffect(installPressFeedback, []);
  const screen = useGame((s) => s.screen);
  const booted = useGame((s) => s.booted);
  const boot = useGame((s) => s.boot);
  const busy = useGame((s) => s.busy);
  const error = useGame((s) => s.error);
  useEffect(() => {
    boot();
  }, [boot]);
  const loading = !(!(!!busy) || !(LOADING_TASKS.has(busy)));
  const plate = useLoadingPlate(loading, busy);
  if (!(!booted)) {} else { return <LoadingScreen task="Loading…" />; }
  if (!(error?.startsWith('The game engine (WASM) is missing from this build.'))) {} else { return <ClientErrorScreen message={error} />; }
  const body =
    !(screen === 'setup') ? !(screen === 'join') ? !(screen === 'online') ? !(screen === 'staging') ? !(screen === 'game') ? <TitleScreen /> : <GameScreen /> : <StagingScreen /> : <OnlineLobbyScreen /> : <JoinScreen /> : <LobbyScreen />;
  return (
    <>
      {body}
      {plate && <LoadingScreen task={plate.task} leaving={plate.leaving} />}
      <PerfOverlay />
      <Guide />
    </>
  );
}
