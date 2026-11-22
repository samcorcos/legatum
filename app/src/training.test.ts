import { describe, expect, it, vi } from 'vitest';
import { TrainingDriver, cardText, chipText, reportLine, type TrainingEngine, type TrainingProgress, type TrainingUi } from './training';
import type { PlayerView } from './types';

const battle = (name: string, q: number, done: boolean, changed: boolean) =>
  ({ seat: 2, name, at: { q, r: 4 }, kind: 'attack' as const, done, changed });

const progress = (battles: ReturnType<typeof battle>[], current: number | null): TrainingProgress =>
  ({ turn: 30, battles, current, done: current === null, rollouts: 0 });

/** A viewer who knows Lugano (q 5) and Thun (q 9) but not Sion (q 20). */
const view = {
  cities: [
    { name: 'Lugano', pos: { q: 5, r: 4 } },
    { name: 'Thun', pos: { q: 9, r: 4 } },
  ],
} as unknown as PlayerView;

describe('Training mode text', () => {
  it('the chip counts the battles still being studied', () => {
    const p = progress([battle('Lugano', 5, true, false), battle('Thun', 9, false, false), battle('Sion', 20, false, false)], 1);
    expect(chipText(p)).toBe('Training: studying 2 battles');
  });

  it('the card names the battle, its place in the list and the seconds spent, never an unseen place', () => {
    const ui: TrainingUi = { progress: progress([battle('Lugano', 5, true, false), battle('Thun', 9, false, false), battle('Sion', 20, false, false)], 1), startedAt: 0, capMs: 15_000, waiting: true };
    expect(cardText(ui, view, 6_200)).toMatchObject({ title: 'Training: the AI is studying 3 battles', line: 'Attack on Thun (2 of 3) · 6 of 15 s' });
    const hidden = { ...ui, progress: { ...ui.progress, current: 2 } };
    expect(cardText(hidden, view, 1000).line).toBe('A battle out of sight (3 of 3) · 1 of 15 s');
  });

  it('the report line lists the battles where the plan changed, with the first known one on the map', () => {
    const p = progress([battle('Lugano', 5, true, true), battle('Thun', 9, true, true), battle('Sion', 20, true, false)], null);
    const line = reportLine(p, view)!;
    expect(line.text).toBe('Training: the AI changed its plan in 2 of 3 battles (Lugano, Thun)');
    expect(line.at).toEqual({ q: 5, r: 4 });
    const kept = reportLine(progress([battle('Lugano', 5, true, false)], null), view)!;
    expect(kept.text).toBe('Training: the AI studied its attack on Lugano and kept its first plan');
    expect(kept.at).toEqual({ q: 5, r: 4 });
    const unseen = reportLine(progress([battle('Sion', 20, true, false)], null), view)!;
    expect(unseen.text).toBe('Training: the AI studied a battle out of sight and kept its first plan');
    expect(unseen.at).toBeNull();
  });
});

describe('Training search never holds the turn', () => {
  const p0: TrainingProgress = { turn: 44, battles: [battle('Bellinzona', 28, false, false)], current: 0, done: false, rollouts: 0 };
  const engine = (step: () => TrainingProgress | null): TrainingEngine => ({
    trainingBegin: () => p0,
    trainingStep: step,
    trainingFinish: () => null,
    trainingGameId: () => 'game-1',
  });
  it('a search that never finishes settles at the cap', async () => {
    const driver = new TrainingDriver(engine(() => ({ ...p0 })), p0, 100, () => {});
    await expect(driver.finish()).resolves.toBeNull();
    expect(driver.done).toBe(true);
  });

  it('a WASM search that throws settles on Hard', async () => {
    vi.spyOn(console, 'warn').mockImplementation(() => {});
    const driver = new TrainingDriver(engine(() => { throw new Error('boom'); }), p0, 15_000, () => {});
    await expect(driver.finish()).resolves.toBeNull();
    expect(driver.done).toBe(true);
  });
});
