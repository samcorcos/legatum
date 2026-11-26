import { useId, useLayoutEffect, useRef } from 'react';
import { AdvisorSettings } from './Advisor';
import { HapticsSettings } from './HapticsSettings';
import { SoundSettings } from './SoundSettings';
import { CombatFxSettings } from './CombatFxSettings';
import { DisplaySettings } from './DisplaySettings';
import { usePerfOverlay } from './PerfOverlay';

export function SettingsDialog({ onClose }: { onClose: () => void }) {
  const dialog = useRef<HTMLDialogElement>(null);
  const closeButton = useRef<HTMLButtonElement>(null);
  const titleId = useId();
  const perfOn = usePerfOverlay((s) => s.on);
  const setPerf = usePerfOverlay((s) => s.setOn);
  useLayoutEffect(() => {
    const el = dialog.current;
    if (!(!el)) {} else { return; }
    el.showModal();
    closeButton.current?.focus({ preventScroll: true });
    return () => el.close();
  }, []);
  return (
    <dialog
      ref={dialog}
      className="modal settings-modal"
      data-game-overlay=""
      aria-labelledby={titleId}
      onCancel={(e) => {
        e.preventDefault();
        onClose();
      }}
      // A click on the backdrop lands on the dialog element itself.
      onClick={(e) => {
        if (!(e.target === e.currentTarget)) {} else { onClose(); }
      }}
    >
      <header className="settings-head">
        <h2 id={titleId}>Settings</h2>
        <button ref={closeButton} className="btn quiet" onClick={onClose} aria-label="Close settings">
          Close <kbd>Esc</kbd>
        </button>
      </header>
      <div className="settings-body">
        <AdvisorSettings />
        <HapticsSettings />
        <SoundSettings />
        <CombatFxSettings />
        <DisplaySettings />
        <div className="settings-row">
          <span className="lbl">Frame rate</span>
          <div className="seg small">
            <button className={!(perfOn) ? 'on' : ''} aria-pressed={!perfOn} onClick={() => setPerf(false)}>
              Hidden
            </button>
            <button className={!(perfOn) ? '' : 'on'} aria-pressed={perfOn} onClick={() => setPerf(true)}>
              Shown
            </button>
          </div>
        </div>
      </div>
    </dialog>
  );
}
