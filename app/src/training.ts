// Training mode (designer, 2026-11-07): AI seats at the Training level search their battles.
// The search starts when a turn begins, while the human plans, in short WASM slices. End Turn
// waits for it (at most the cap) behind a progress card; the turn report then says what it
// changed. Every searched battle, the human's own battle positions and their realised outcomes
// become training log lines (`crates/ai/src/training.rs`), kept in memory for the session.
import type { Hex, PlayerView } from './types';
import type { ReportLine } from './format';

export type BattleKind = 'attack' | 'defend' | 'field';

export interface TrainingBattle {
  seat: number;
  name: string;
  at: Hex;
  kind: BattleKind;
  done: boolean;
  changed: boolean;
}

export interface TrainingProgress {
  turn: number;
  battles: TrainingBattle[];
  current: number | null;
  done: boolean;
  rollouts: number;
}

export interface TrainingFinish {
  turn: number;
  progress: TrainingProgress;
  lines: string[];
}

/** What the engine client offers for Training mode (the local WASM client). */
export interface TrainingEngine {
  trainingBegin(capMs: number): TrainingProgress | null;
  trainingStep(maxRollouts: number, elapsedMs: number): TrainingProgress | null;
  trainingFinish(elapsedMs: number): TrainingFinish | null;
  trainingGameId(): string;
}

/** The search's time cap per turn. `localStorage['legatum.training.capMs']` overrides it. */
export const TRAINING_CAP_MS = 15_000;

export function trainingCapMs(): number {
  try {
    const v = Number(globalThis.localStorage?.getItem('legatum.training.capMs'));
    if (Number.isFinite(v) && v >= 1000 && v <= 600_000) return v;
  } catch { /* Storage can be unavailable. */ }
  return TRAINING_CAP_MS;
}

/** What the UI shows: the search's progress, and whether End Turn is waiting for it. */
export interface TrainingUi {
  progress: TrainingProgress;
  startedAt: number;
  capMs: number;
  /** The human ended the turn: the progress card is up. */
  waiting: boolean;
}

const plural = (n: number, one: string, many = `${one}s`) => `${n} ${n === 1 ? one : many}`;

/** A battle's name as this viewer may see it: never a place he doesn't know. */
export function battleName(b: TrainingBattle, view: PlayerView | null): string | null {
  if (!view) return null;
  const known = view.cities.some((c) => (c.pos.q === b.at.q && c.pos.r === b.at.r) || b.name === `near ${c.name}`);
  return known ? b.name : null;
}

export function chipText(p: TrainingProgress): string {
  const left = p.battles.filter((b) => !b.done).length;
  return p.done ? `Training: studied ${plural(p.battles.length, 'battle')}` : `Training: studying ${plural(left, 'battle')}`;
}

/** What the AI is doing at a battle, from its own side: "Attack on Lugano", "Defence of Thun". */
export function battleAction(b: TrainingBattle, view: PlayerView | null): string {
  const name = battleName(b, view);
  if (name === null) return 'A battle out of sight';
  if (b.kind === 'attack') return `Attack on ${name}`;
  if (b.kind === 'defend') return `Defence of ${name}`;
  return name === 'open field' ? 'Battle in the open field' : `Battle ${name}`;
}

/** The progress card: "Training: the AI is studying 3 battles" and "Attack on Lugano (2 of 3) · 6 of 15 s". */
export function cardText(ui: TrainingUi, view: PlayerView | null, now: number): { title: string; line: string; fraction: number } {
  const p = ui.progress;
  const n = p.battles.length;
  const i = p.current ?? n - 1;
  const b = p.battles[Math.max(0, i)];
  const action = b ? battleAction(b, view) : '';
  const secs = Math.max(0, Math.round((now - ui.startedAt) / 1000));
  const done = p.battles.filter((x) => x.done).length;
  // Battles finished or the share of the time cap used, whichever is further along: it can step
  // when a battle finishes, then runs on time.
  const fraction = Math.min(1, Math.max(n ? done / n : 1, (now - ui.startedAt) / ui.capMs));
  return {
    title: `Training: the AI is studying ${plural(n, 'battle')}`,
    line: p.done ? 'Finishing the turn' : `${action} (${Math.min(n, i + 1)} of ${n}) · ${Math.min(secs, Math.round(ui.capMs / 1000))} of ${Math.round(ui.capMs / 1000)} s`,
    fraction,
  };
}

/** The turn report's line: "Training: the AI changed its plan in 2 of 3 battles (Lugano, Thun)". */
export function reportLine(p: TrainingProgress, view: PlayerView): ReportLine | null {
  const n = p.battles.length;
  if (!n) return null;
  const changed = p.battles.filter((b) => b.changed);
  const named = changed.map((b) => battleName(b, view)).filter((x): x is string => x !== null);
  const unseen = changed.length - named.length;
  const places = [...named, ...(unseen ? [unseen === 1 ? 'one out of sight' : `${unseen} out of sight`] : [])];
  const only = p.battles[0];
  const text = changed.length
    ? `Training: the AI changed its plan in ${changed.length} of ${plural(n, 'battle')} (${places.join(', ')})`
    : n === 1
      ? `Training: the AI studied ${battleAction(only, view).replace(/^A /, 'a ').replace(/^(Attack|Defence|Battle)/, (w) => `its ${w.toLowerCase()}`)} and kept its first plan`
      : `Training: the AI studied ${n} battles and kept its first plan in all of them`;
  const first = (changed.length ? changed : p.battles).find((b) => battleName(b, view) !== null) ?? null;
  return {
    category: 'combat',
    text,
    details: ['Training is a research tool. The AI studies each battle using the true positions of nearby units, including ones it cannot see, before choosing its orders. Every battle is saved.'],
    at: first ? first.at : null,
    // Shown under Mine and Team too: it is about the AI he is playing against.
    players: [view.me, ...new Set(p.battles.map((b) => b.seat))],
    weight: 0,
    tone: 'neutral',
  };
}

/** This session's training log lines (the latest 5,000), also on `globalThis.legatumTrainingLog`. */
export const unwrittenLog: string[] = [];
(globalThis as { legatumTrainingLog?: string[] }).legatumTrainingLog = unwrittenLog;

/** Keep log lines for this session. `_gameId` names the game the lines belong to. */
export function writeTrainingLog(_gameId: string, lines: string[]): void {
  if (!lines.length) return;
  unwrittenLog.push(...lines);
  if (unwrittenLog.length > 5000) unwrittenLog.splice(0, unwrittenLog.length - 5000);
}

/** The hard watchdog: this long past the cap, the turn goes ahead on Hard's orders regardless. */
export const TRAINING_WATCHDOG_MS = 2000;

/**
 * One turn's search. `onProgress` fires as it advances; `finish()` resolves once the searched
 * orders are in the engine, or once the search failed or ran past the cap plus the watchdog,
 * when the training seats play Hard's orders (the engine's pending orders). It never waits
 * forever (playtest 8.9.27, T44: a failed search start left End Turn waiting).
 */
export class TrainingDriver {
  private cancelled = false;
  /** The search is over: its orders are in (`applied`) or it fell back to Hard (`null`). */
  private settled = false;
  private applied: TrainingFinish | null = null;
  private wakers: (() => void)[] = [];
  private urgent = false;
  readonly started = performance.now();

  constructor(
    private engine: TrainingEngine,
    readonly progress0: TrainingProgress,
    private capMs: number,
    private onProgress: (p: TrainingProgress) => void,
  ) {
    this.schedule();
    this.armWatchdog();
  }

  get done(): boolean { return this.settled; }
  get result(): TrainingFinish | null { return this.applied; }

  cancel() { this.cancelled = true; this.wake(); }

  /** End Turn: stop yielding to the planner and wait for the result. */
  finish(): Promise<TrainingFinish | null> {
    this.urgent = true;
    if (this.settled || this.cancelled) return Promise.resolve(this.applied);
    return new Promise((resolve) => this.wakers.push(() => resolve(this.applied)));
  }

  private wake() {
    const w = this.wakers;
    this.wakers = [];
    w.forEach((f) => f());
  }

  private elapsed() { return performance.now() - this.started; }

  private complete(f: TrainingFinish | null) {
    if (this.cancelled || this.settled) return;
    this.settled = true;
    this.applied = f;
    if (f) {
      this.onProgress(f.progress);
      writeTrainingLog(this.engine.trainingGameId(), f.lines);
    }
    this.wake();
  }

  /** Play Hard's orders this turn, with a warning in the console and the training log. */
  private fallBack(reason: string, error?: unknown) {
    if (this.cancelled || this.settled) return;
    console.warn(`legatum: training search ${reason}; the AI plays Hard this turn`, error ?? '');
    try {
      writeTrainingLog(this.engine.trainingGameId(), [JSON.stringify({ type: 'fallback', turn: this.progress0.turn, reason })]);
    } catch { /* The log is best effort; the turn must go ahead. */ }
    this.complete(null);
  }

  /** At the cap plus `TRAINING_WATCHDOG_MS`, settle on Hard's orders whatever the search is doing. */
  private armWatchdog() {
    const check = () => {
      if (this.cancelled || this.settled) return;
      const held = (globalThis as { legatumTrainingHold?: boolean }).legatumTrainingHold;
      const left = this.capMs + TRAINING_WATCHDOG_MS - this.elapsed();
      if (held || left > 0) {
        setTimeout(check, held ? 100 : Math.max(10, left));
        return;
      }
      this.fallBack(`gave no answer by the cap + ${TRAINING_WATCHDOG_MS / 1000} s`);
    };
    setTimeout(check, Math.max(10, this.capMs + TRAINING_WATCHDOG_MS));
  }

  /** Short slices on the main thread, yielding so planning stays smooth. */
  private schedule() {
    const slice = () => {
      if (this.cancelled || this.settled) return;
      // The screen tour holds the search to photograph the card (`legatumTrainingHold`).
      if ((globalThis as { legatumTrainingHold?: boolean }).legatumTrainingHold) {
        setTimeout(slice, 100);
        return;
      }
      try {
        const budget = this.urgent ? 60 : 12;
        const t0 = performance.now();
        let p: TrainingProgress | null = null;
        do {
          p = this.engine.trainingStep(2, this.elapsed());
        } while (p && !p.done && performance.now() - t0 < budget);
        if (!p || p.done || this.elapsed() >= this.capMs) {
          this.complete(this.engine.trainingFinish(this.elapsed()));
          return;
        }
        this.onProgress(p);
      } catch (e) {
        this.fallBack('failed', e);
        return;
      }
      setTimeout(slice, this.urgent ? 4 : 24);
    };
    setTimeout(slice, 50);
  }
}
