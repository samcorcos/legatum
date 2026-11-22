import { StrictMode } from 'react';
import { createRoot } from 'react-dom/client';
import './brand/tokens.css';
import './brand/fonts';
import './styles.css';
import './online.css';
import './touch.css';
import { App } from './App';
import { useGame } from './store';
import { retryGameHistory } from './platform/archive';
import { installSound } from './feel';
import { setMusicForView } from './feel/sound';
import { loadTerrainSprites } from './map/terrainVariant';
import { loadWasmModule, wasmAvailable } from './wasmClient';
import { CrashCard, RootErrorBoundary, installGlobalErrorHandlers } from './ui/CrashScreen';
import { NarrowGate } from './ui/NarrowPlate';

// First, so an error anywhere below is logged and a fatal one reaches the crash card, not a blank page.
installGlobalErrorHandlers();

// Sound unlocks on the first gesture (autoplay rules). The committed viewer's
// diplomacy state selects the theme; title and other screens use the full piece.
installSound();
useGame.subscribe((state) => setMusicForView(state.screen === 'game' ? state.views?.[state.seat] ?? null : null));

// The terrain art (organic S2; `?terrain=current` skips it) must be decoded before the first map chunk is baked.
// Until React mounts, index.html's static loading plate fills #root. An engine that is in the build but
// fails to download or start shows the crash card with Reload instead of the app (a build without the
// engine keeps its own "missing from this build" screen).
let engineFailure: unknown = null;
void Promise.all([
  loadTerrainSprites().catch(() => undefined),
  loadWasmModule().catch((error: unknown) => {
    if (!wasmAvailable()) return;
    console.error('Legatum: the game engine did not load', error);
    engineFailure = error ?? new Error('The game engine did not load');
  }),
]).then(() => {
  createRoot(document.getElementById('root')!).render(
    <StrictMode>
      {engineFailure ? (
        <CrashCard error={engineFailure} kind="engine" />
      ) : (
        <RootErrorBoundary>
          <NarrowGate>
            <App />
          </NarrowGate>
        </RootErrorBoundary>
      )}
    </StrictMode>,
  );
});

// Handy for debugging and scripted play-throughs.
(window as unknown as { legatum: typeof useGame }).legatum = useGame;

// A finished game whose upload failed is sent again when the browser comes back online.
const retryArchives = () => { void retryGameHistory().catch((error) => console.warn('game upload', error)); };
window.addEventListener('online', retryArchives);
retryArchives();
