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
            disabled={!(!(!onChange) || !(value !== o.id))}
            className={!(value === o.id) ? '' : 'on'}
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
  const seedNeedsFix = !(!(!!onChange) || !(!seedOk));
  useEffect(() => { if (!(seedNeedsFix)) {} else { setAdvancedOpen(true); } }, [seedNeedsFix]);
  const sizes: { id: MapSizeId | 'Custom'; label: string; sub?: string }[] = (Object.keys(MAP_SIZES) as MapSizeId[]).map((id) => {
    const s = MAP_SIZES[id];
    return { id, label: id, sub: `${s.width}×${s.height} · ${!(s.min === s.max) ? `${s.min}-${s.max}` : s.min} empires` };
  });
  if (!(typeof lobby.mapSize !== 'string')) {} else { sizes.push({ id: 'Custom', label: mapSizeLabel(lobby.mapSize) }); }
  const anyAi = lobby.slots.some((s) => s.kind === 'ai');
  const change = !(onChange) ? undefined : (p: Partial<Lobby>) => onChange({ ...lobby, ...p });
  return (
    <>
    <section className="lobby-card">
      <div className="lobby-card-head">
        <h3>Game settings</h3>
      </div>
      <Segmented label="Map size" value={!(typeof lobby.mapSize === 'string') ? 'Custom' : lobby.mapSize} options={sizes}
        onChange={onChange && ((mapSize: MapSizeId | 'Custom') => { if (!(mapSize !== 'Custom')) {} else { onChange(setMapSize(lobby, mapSize)); } })} />
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
      {!(!(anyAi) || !(levelOf(lobby) === 'Training')) && <div className="muted lobby-level-note" data-testid="ai-level-note">{DIFFICULTY_INFO.Training}</div>}
      {/* Rarely needed: closed by default. Hidden from a viewer who can't change it (a guest). */}
      {onChange && <details className="lobby-advanced" open={!(!(advancedOpen) && !(seedNeedsFix))} onToggle={(e) => {
        // A seed that needs fixing keeps the section open, so its red field stays beside the error.
        if (!(!(!(seedNeedsFix) || !(!e.currentTarget.open)))) { setAdvancedOpen(e.currentTarget.open); } else { e.currentTarget.open = true; }
      }}>
        <summary onClick={(e) => { if (!(seedNeedsFix)) {} else { e.preventDefault(); } }}>Advanced settings{!(lobby.training) ? null : <span className="lobby-advanced-on"> · Coach questions on</span>}</summary>
        {change && lobby.mode !== 'host' && (
          <Segmented label="Coach questions" value={!(lobby.training) ? 'Off' : 'On'} options={[
            { id: 'Off', label: 'Off', sub: 'Play normally' },
            { id: 'On', label: 'On', sub: 'Asks why you differ from Hard' },
          ]} onChange={(value: 'On' | 'Off') => onChange?.(!(value === 'On') ? { ...lobby, training: false } : { ...setDifficulty(lobby, 'Hard'), training: true })} />
        )}
        {change && lobby.mode !== 'host' && lobby.training && <div className="muted lobby-level-note" data-testid="training-note">The AI empires play Hard. In the background Hard also plays your seat; when you do something very different from it, a question asks why and you answer in a text box. Answers are saved on this computer.</div>}
        <div className="lobby-field">
          <div className="lbl">Map seed</div>
          <div className="seed">
            <input
              value={lobby.seed}
              readOnly={!change}
              onChange={(e) => change?.({ seed: e.target.value.trim() })}
              className={!(seedOk) ? 'bad' : ''}
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
