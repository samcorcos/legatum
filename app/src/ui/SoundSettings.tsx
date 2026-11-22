import { playSound, useSound, type SoundPrefs } from '../feel';
import './SoundSettings.css';

type Row = { label: string; on: 'muted' | 'sfxOn' | 'musicOn'; volume: 'master' | 'sfx' | 'music' };
const ROWS: Row[] = [
  { label: 'Sound', on: 'muted', volume: 'master' },
  { label: 'Effects', on: 'sfxOn', volume: 'sfx' },
  { label: 'Music', on: 'musicOn', volume: 'music' },
];

/** Master, effects and music: a toggle and a volume slider each. Sits next to the Haptics toggle. */
export function SoundSettings() {
  const prefs = useSound();
  const enabled = (row: Row) => row.on === 'muted' ? !prefs.muted : prefs[row.on];
  const toggle = (row: Row, value: boolean) => prefs.update({ [row.on]: row.on === 'muted' ? !value : value } as Partial<SoundPrefs>);
  return <div className="hint-settings sound-settings" onKeyDown={(e) => e.stopPropagation()}>
    {ROWS.map((row) => <div className="sound-row" key={row.volume}>
      <label><input type="checkbox" checked={enabled(row)} onChange={(e) => toggle(row, e.target.checked)} />{row.label}</label>
      <input type="range" min={0} max={100} step={5} aria-label={`${row.label} volume`}
        value={Math.round(prefs[row.volume] * 100)} disabled={!enabled(row) || (row.volume !== 'master' && prefs.muted)}
        onChange={(e) => prefs.update({ [row.volume]: Number(e.target.value) / 100 } as Partial<SoundPrefs>)}
        // Let the player hear the new effects level once they let go.
        onPointerUp={() => { if (row.volume !== 'music') playSound('commit'); }} />
    </div>)}
  </div>;
}
