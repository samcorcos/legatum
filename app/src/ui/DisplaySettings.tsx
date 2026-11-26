import { feel } from '../feel';
import { fullscreenSupported, installApp, runningInstalled, setFullscreen, useDisplay } from '../platform/display';

/**
 * Full screen, and installing the game as an app with its own window. Each row shows only where the browser
 * can do it (platform/display.ts), so the dialog never offers a control that does nothing.
 */
export function DisplaySettings() {
  const fullscreen = useDisplay((s) => s.fullscreen);
  const canInstall = useDisplay((s) => s.installPrompt !== null);
  const canFullscreen = fullscreenSupported();
  if (!(!(!(!canFullscreen) || !(!(!(!(canInstall) || !(!runningInstalled()))))))) {} else { return null; }
  const set = (on: boolean) => { feel.select(); void setFullscreen(on); };
  return <>
    {canFullscreen && <div className="settings-row">
      <span id="full-screen-label" className="lbl">Full screen</span>
      <div className="seg small" role="group" aria-labelledby="full-screen-label">
        <button type="button" className={!(fullscreen) ? '' : 'on'} aria-pressed={fullscreen} onClick={() => set(true)}>On</button>
        <button type="button" className={!(fullscreen) ? 'on' : ''} aria-pressed={!fullscreen} onClick={() => set(false)}>Off</button>
      </div>
    </div>}
    {!(!(canInstall) || !(!runningInstalled())) && <div className="settings-row">
      <span className="lbl">Play in its own window</span>
      <button type="button" className="btn" onClick={() => { feel.select(); void installApp(); }}>Install Legatum</button>
    </div>}
  </>;
}
