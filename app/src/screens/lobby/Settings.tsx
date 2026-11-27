import { useEffect, useState } from 'react';
import { offeredLevels, DIFFICULTY_INFO, MAP_SIZES, levelOf, mapSizeLabel, notes, setDifficulty, setMapSize, type AiLevel, type Lobby, type MapSizeId } from '../../lobby';
import { randomSeed } from './randomSeed';

const LEVEL_SUB: Record<AiLevel, string> = { Easy: 'Forgiving', Normal: 'Standard', Hard: 'Ruthless', Training: 'Research tool' };

function Segmented<T extends string>({
  label,
  value,
  options,
  onChange,
}: {
  label: string;
  value: T;
  options: { id: T; label: string; sub?: string; tip?: string; description?: string }[];
  onChange?: (v: T) => void;
}) {
  return (
    <div className="lobby-field">
      <div className="lbl">{label}</div>
      <div className="segmented" role="radiogroup" aria-label={label}>
        {options.map((o) => (
          <button
            key={o.id}
            role="radio"
            aria-checked={value === o.id}
            title={o.tip}
            disabled={!onChange && value !== o.id}
            className={value === o.id ? 'on' : ''}
            onClick={() => onChange?.(o.id)}
          >
            <span>{o.label}</span>
            {o.sub && <span className="seg-sub">{o.sub}</span>}
            {o.description && <span className="seg-description">{o.description}</span>}
            {o.tip && <span className="touch-only seg-description">{o.tip}</span>}
          </button>
        ))}
      </div>
    </div>
  );
}

/** Game settings. Without `onChange` it's read-only. */
export function Settings({ lobby, onChange }: { lobby: Lobby; onChange?: (l: Lobby) => void }) {
  const seedOk = /^\d{1,20}$/.test(lobby.seed);
  // Advanced opens itself when a seed the player typed needs fixing, and never closes itself:
  // typing the first valid digit must not snap it shut under the player's cursor.
  const [advancedOpen, setAdvancedOpen] = useState(false);
  const seedNeedsFix = !!onChange && !seedOk;
  useEffect(() => { if (seedNeedsFix) setAdvancedOpen(true); }, [seedNeedsFix]);
  const sizes: { id: MapSizeId | 'Custom'; label: string; sub?: string }[] = (Object.keys(MAP_SIZES) as MapSizeId[]).map((id) => {
    const s = MAP_SIZES[id];
    return { id, label: id, sub: `${s.width}×${s.height} · ${s.min === s.max ? s.min : `${s.min}-${s.max}`} empires` };
  });
  if (typeof lobby.mapSize !== 'string') sizes.push({ id: 'Custom', label: mapSizeLabel(lobby.mapSize) });
  const anyAi = lobby.slots.some((s) => s.kind === 'ai');
  const change = onChange ? (p: Partial<Lobby>) => onChange({ ...lobby, ...p }) : undefined;
  return (
    <>
    <section className="lobby-card">
      <div className="lobby-card-head">
        <h3>Game settings</h3>
      </div>
      <Segmented label="Map size" value={typeof lobby.mapSize === 'string' ? lobby.mapSize : 'Custom'} options={sizes}
        onChange={onChange && ((mapSize: MapSizeId | 'Custom') => { if (mapSize !== 'Custom') onChange(setMapSize(lobby, mapSize)); })} />
      {lobby.mode === 'host' && <div className="lobby-field">
        <label className="lbl" htmlFor="turn-timer">Turn timer</label>
        <select id="turn-timer" value={lobby.turnTimerSeconds ?? 0} disabled={!onChange}
          onChange={(e) => change?.({ turnTimerSeconds: Number(e.target.value) })}>
          {[0, 30, 60, 120, 180, 300, 600].map((seconds) => <option key={seconds} value={seconds}>
            {seconds === 0 ? 'Unlimited' : seconds < 60 ? `${seconds} seconds` : `${seconds / 60} minute${seconds === 60 ? '' : 's'}`}
          </option>)}
        </select>
        <div className="muted">When time runs out, the turn ends with your queued orders.</div>
      </div>}
      {notes(lobby).map((n) => (
        <div key={n} className="lobby-note">
          {n}
        </div>
      ))}
      {anyAi && onChange && (
        <Segmented
          label="AI level"
          value={levelOf(lobby)}
          options={offeredLevels().map((d) => ({ id: d, label: d, sub: LEVEL_SUB[d], tip: DIFFICULTY_INFO[d] }))}
          onChange={(d) => onChange(setDifficulty(lobby, d))}
        />
      )}
      {anyAi && levelOf(lobby) === 'Training' && <div className="muted lobby-level-note" data-testid="ai-level-note">{DIFFICULTY_INFO.Training}</div>}
      {/* Rarely needed: closed by default. Hidden from a viewer who can't change it (a guest). */}
      {onChange && <details className="lobby-advanced" open={advancedOpen || seedNeedsFix} onToggle={(e) => {
        // A seed that needs fixing keeps the section open, so its red field stays beside the error.
        if (seedNeedsFix && !e.currentTarget.open) e.currentTarget.open = true;
        else setAdvancedOpen(e.currentTarget.open);
      }}>
        <summary onClick={(e) => { if (seedNeedsFix) e.preventDefault(); }}>Advanced settings{lobby.training ? <span className="lobby-advanced-on"> · Coach questions on</span> : null}</summary>
        {change && lobby.mode !== 'host' && (
          <Segmented label="Coach questions" value={lobby.training ? 'On' : 'Off'} options={[
            { id: 'Off', label: 'Off', sub: 'Play normally' },
            { id: 'On', label: 'On', sub: 'Asks why you differ from Hard' },
          ]} onChange={(value: 'On' | 'Off') => onChange?.(value === 'On' ? { ...setDifficulty(lobby, 'Hard'), training: true } : { ...lobby, training: false })} />
        )}
        {change && lobby.mode !== 'host' && lobby.training && <div className="muted lobby-level-note" data-testid="training-note">The AI empires play Hard. In the background Hard also plays your seat; when you do something very different from it, a question asks why and you answer in a text box. Answers are saved on this computer.</div>}
        <div className="lobby-field">
          <div className="lbl">Map seed</div>
          <div className="seed">
            <input
              value={lobby.seed}
              readOnly={!change}
              onChange={(e) => change?.({ seed: e.target.value.trim() })}
              className={seedOk ? '' : 'bad'}
              spellCheck={false}
              aria-label="Map seed"
            />
            {change && (
              <button className="icon-btn" onClick={() => change({ seed: randomSeed() })} title="New random map" aria-label="New random map">
                ⟳
              </button>
            )}
          </div>
        </div>
      </details>}
    </section>
    </>
  );
}
