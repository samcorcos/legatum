import { OBSERVER_SPEEDS, canStepBack, type ObserverPerspective, type ObserverSpeed } from '../observer';
import { useGame } from '../store';
import { empireOfPlayer } from '../empires';
import { Tip } from './Tip';
import './observerBar.css';

export function ObserverBar() {
  const observer = useGame((s) => s.observer);
  const view = useGame((s) => (s.views ? s.views[s.seat] : null));
  const togglePlay = useGame((s) => s.observerTogglePlay);
  const stepBack = useGame((s) => s.observerStepBack);
  const stepForward = useGame((s) => s.observerStepForward);
  const setSpeed = useGame((s) => s.observerSetSpeed);
  const setPerspective = useGame((s) => s.observerSetPerspective);

  if (!observer.active || !view) return null;

  const backEnabled = canStepBack(observer);
  const forwardDisabled = view.finished && observer.cursor >= observer.history.length - 1;
  const playDisabled = view.finished && observer.cursor >= observer.history.length - 1;
  const perspectiveValue = observer.perspective === 'god' ? 'god' : String(observer.perspective);

  const onPerspectiveChange = (raw: string) => {
    const next: ObserverPerspective = raw === 'god' ? 'god' : Number(raw);
    setPerspective(next);
  };

  return (
    <div className="observer-bar paper-grain" role="toolbar" aria-label="Observer playback controls" data-testid="observer-bar">
      <div className="obs-transport">
        <Tip hoverOnly placement="top" content="Step back 1 turn (←)">
          <button
            type="button"
            className="btn quiet obs-btn"
            onClick={stepBack}
            disabled={!backEnabled}
            aria-label="Step back 1 turn"
            data-testid="obs-step-back"
          >
            ◀
          </button>
        </Tip>
        <Tip hoverOnly placement="top" content={observer.playing ? 'Pause (Space)' : 'Play (Space)'}>
          <button
            type="button"
            className={`btn obs-btn obs-play${observer.playing ? '' : ' primary'}`}
            onClick={togglePlay}
            disabled={playDisabled}
            aria-label={observer.playing ? 'Pause' : 'Play'}
            aria-pressed={observer.playing}
            data-testid="obs-play-pause"
          >
            {observer.playing ? '⏸' : '▶'}
          </button>
        </Tip>
        <Tip hoverOnly placement="top" content="Step forward 1 turn (→)">
          <button
            type="button"
            className="btn quiet obs-btn"
            onClick={stepForward}
            disabled={forwardDisabled}
            aria-label="Step forward 1 turn"
            data-testid="obs-step-forward"
          >
            ▶|
          </button>
        </Tip>
      </div>

      <span className="obs-sep" aria-hidden="true" />

      <label className="obs-speed-wrap">
        <span className="sr-only">Playback speed</span>
        <select
          className="obs-select obs-speed"
          aria-label="Playback speed"
          value={observer.speed}
          onChange={(e) => setSpeed(Number(e.target.value) as ObserverSpeed)}
          data-testid="obs-speed"
        >
          {OBSERVER_SPEEDS.map((spd) => (
            <option key={spd} value={spd}>{spd}×</option>
          ))}
        </select>
      </label>

      <span className="obs-sep" aria-hidden="true" />

      <label className="obs-view-wrap">
        <span className="sr-only">View perspective</span>
        <select
          className="obs-select obs-perspective"
          aria-label="View perspective"
          value={perspectiveValue}
          onChange={(e) => onPerspectiveChange(e.target.value)}
          data-testid="obs-perspective"
        >
          <option value="god">God view</option>
          {view.players.map((p) => (
            <option key={p.id} value={String(p.id)}>
              {empireOfPlayer(view, p.id)}{p.alive ? '' : ' (eliminated)'}
            </option>
          ))}
        </select>
      </label>
    </div>
  );
}
