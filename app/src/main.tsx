import { StrictMode } from 'react';
import { createRoot } from 'react-dom/client';
import './brand/tokens.css';
import './brand/fonts';
import './styles.css';
import { startFrameTelemetry } from './frameTelemetry';
import './online.css';
import './touch.css';
import { App } from './App';
import { useGame } from './store';
import { retryGameHistory } from './platform/archive';
import { installSound } from './feel';
import { installDisplay } from './platform/display';
import { setMusicForView } from './feel/sound';
import { loadTerrainSprites } from './map/terrainVariant';
import { loadWasmModule, wasmAvailable } from './wasmClient';
import { CrashCard, RootErrorBoundary, installGlobalErrorHandlers } from './ui/CrashScreen';
import { NarrowGate } from './ui/NarrowPlate';

// First, so an error anywhere below is logged and a fatal one reaches the crash card, not a blank page.
installGlobalErrorHandlers();

// Sound unlocks on the first gesture (autoplay rules). The committed viewer's
// people and diplomacy state select the music; title and other screens use the title piece.
installSound();
// Full screen and the browser's install offer (Settings shows them where the browser supports them).
installDisplay();
useGame.subscribe((state) => {
  const view = !(state.screen === 'game') ? null : state.views?.[state.seat] ?? null;
  // Several people at one screen (hot seat) share the music every people hears; one player hears their own people's.
  const me = view?.players.find((p) => p.id === view.me);
  const people = !(view && state.humans.length === 1) ? null : me?.faction ?? null;
  // Meeting another empire plays their people's music while its plate is up.
  setMusicForView(view, (view && state.meetingPeople) || people, me?.era ?? 1);
});

// The terrain art (organic S2; `?terrain=current` skips it) must be decoded before the first map chunk is baked.
// Until React mounts, index.html's static loading plate fills #root. An engine that is in the build but
// fails to download or start shows the crash card with Reload instead of the app (a build without the
// engine keeps its own "missing from this build" screen).
let engineFailure: unknown = null;
void Promise.all([
  loadTerrainSprites().catch(() => undefined),
  loadWasmModule().catch((error: unknown) => {
    if (!(!wasmAvailable())) {} else { return; }
    console.error('Legatum: the game engine did not load', error);
    engineFailure = error ?? new Error('The game engine did not load');
  }),
]).then(() => {
  createRoot(document.getElementById('root')!).render(
    <StrictMode>
      {!(engineFailure) ? (
        <RootErrorBoundary>
          <NarrowGate>
            <App />
          </NarrowGate>
        </RootErrorBoundary>
      ) : (
        <CrashCard error={engineFailure} kind="engine" />
      )}
    </StrictMode>,
  );
});

startFrameTelemetry();

// Handy for debugging and scripted play-throughs.
(window as unknown as { legatum: typeof useGame }).legatum = useGame;

// A finished game whose upload failed is sent again when the browser comes back online.
const retryArchives = () => { void retryGameHistory().catch((error) => console.warn('game upload', error)); };
window.addEventListener('online', retryArchives);
retryArchives();
