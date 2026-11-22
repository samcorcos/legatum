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
import { loadWasmModule } from './wasmClient';

// Sound unlocks on the first gesture (autoplay rules). The committed viewer's
// diplomacy state selects the theme; title and other screens use the full piece.
installSound();
useGame.subscribe((state) => setMusicForView(state.screen === 'game' ? state.views?.[state.seat] ?? null : null));

// The terrain art (organic S2; `?terrain=current` skips it) must be decoded before the first map chunk is baked.
void Promise.all([loadTerrainSprites().catch(() => undefined), loadWasmModule().catch(() => undefined)]).then(() => {
  createRoot(document.getElementById('root')!).render(
    <StrictMode>
      <App />
    </StrictMode>,
  );
});

// Handy for debugging and scripted play-throughs.
(window as unknown as { legatum: typeof useGame }).legatum = useGame;

// A finished game whose upload failed is sent again when the browser comes back online.
const retryArchives = () => { void retryGameHistory().catch((error) => console.warn('game upload', error)); };
window.addEventListener('online', retryArchives);
retryArchives();
