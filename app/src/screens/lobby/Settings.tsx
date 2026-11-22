import { offeredLevels, DIFFICULTY_INFO, MAP_SIZES, MAX_TURN_LIMIT, levelOf, mapSizeLabel, notes, setDifficulty, setMapSize, type AiLevel, type Lobby, type MapSizeId, validTurnLimit } from '../../lobby';
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
      {notes(lobby).map((n) => (
        <div key={n} className="lobby-note">
          {n}
        </div>
      ))}
      <Segmented label="Turn limit" value={lobby.turnLimit === null ? 'Off' : 'On'} options={[
        { id: 'Off', label: 'Off', sub: 'Play until victory' },
        { id: 'On', label: 'On', sub: 'Most VP at the limit' },
      ]} onChange={change && ((value: 'Off' | 'On') => change({ turnLimit: value === 'Off' ? null : (lobby.turnLimit ?? 300) }))} />
      {lobby.turnLimit !== null && <div className="lobby-field">
        <label className="lbl" htmlFor="turn-limit">Maximum turns</label>
        <input id="turn-limit" type="number" min={1} max={MAX_TURN_LIMIT} step={1}
          value={lobby.turnLimit || ''} readOnly={!change} aria-invalid={!validTurnLimit(lobby.turnLimit)}
          onChange={(e) => change?.({ turnLimit: Number(e.target.value) })} />
      </div>}
      <Segmented label="Barbarians" value={lobby.barbarians === false ? 'Off' : 'On'} options={[
        { id: 'On', label: 'On', sub: 'Camps raid cities' },
        { id: 'Off', label: 'Off', sub: 'No barbarians' },
      ]} onChange={change && ((value: 'On' | 'Off') => change({ barbarians: value === 'On' }))} />
      {anyAi && onChange && (
        <Segmented
          label="AI level (all AI empires)"
          value={levelOf(lobby)}
          options={offeredLevels().map((d) => ({ id: d, label: d, sub: LEVEL_SUB[d], tip: DIFFICULTY_INFO[d] }))}
          onChange={(d) => onChange(setDifficulty(lobby, d))}
        />
      )}
      {change && lobby.mode !== 'host' && (
        <Segmented label="Coach questions" value={lobby.training ? 'On' : 'Off'} options={[
          { id: 'Off', label: 'Off', sub: 'Play normally' },
          { id: 'On', label: 'On', sub: 'Asks why you differ from Hard' },
        ]} onChange={(value: 'On' | 'Off') => onChange?.(value === 'On' ? { ...setDifficulty(lobby, 'Hard'), training: true } : { ...lobby, training: false })} />
      )}
      {change && lobby.mode !== 'host' && lobby.training && <div className="muted lobby-level-note" data-testid="training-note">The AI empires play Hard. In the background Hard also plays your seat; when you do something very different from it, a question asks why and you answer in a text box. Answers are saved on this computer.</div>}
      {anyAi && levelOf(lobby) === 'Training' && <div className="muted lobby-level-note" data-testid="ai-level-note">{DIFFICULTY_INFO.Training}</div>}
      <div className="lobby-field">
        <div className="lbl">Map seed</div>
        <div className="seed">
          <input
            value={lobby.seed}
            readOnly={!change}
            onChange={(e) => change?.({ seed: e.target.value.trim() })}
            className={onChange && !seedOk ? 'bad' : ''}
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
    </section>
    </>
  );
}
