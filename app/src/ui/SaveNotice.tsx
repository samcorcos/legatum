import { useGame } from '../store';
import { saveGameToFile } from './GameMenu';
import './SaveNotice.css';

/** The browser would not keep the latest autosave: one calm notice per game, with Save to file as its one action. */
export function SaveNotice() {
  const shown = useGame((s) => s.autosaveNotice);
  const set = useGame((s) => s.set);
  if (!shown) return null;
  return <div className="toast save-notice" role="status" data-testid="save-notice">
    <span className="toast-message">This game could not be saved in this browser. Save it to a file to keep it.</span>
    <button className="btn" onClick={() => { saveGameToFile(); set({ autosaveNotice: false }); }}>Save to file</button>
    <button type="button" className="toast-close" aria-label="Dismiss" onClick={() => set({ autosaveNotice: false })}>×</button>
  </div>;
}
