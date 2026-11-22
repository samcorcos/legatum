import { useHaptics } from '../feel';
import { hapticsAvailable } from '../feel/haptics';

/** Only where the device can vibrate: on a desktop the toggle would do nothing. */
export function HapticsSettings() {
  const { enabled, setEnabled } = useHaptics();
  if (!hapticsAvailable()) return null;
  return <div className="hint-settings" onKeyDown={(e) => e.stopPropagation()}>
    <label><input type="checkbox" checked={enabled} onChange={(e) => setEnabled(e.target.checked)} />Haptics</label>
  </div>;
}
