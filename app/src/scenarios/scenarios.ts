// Scenarios: preset training positions with one objective (repo scenarios/*.json, format in
// crates/wasm/src/scenario.rs). The engine prepares and scores them; this module lists them,
// remembers each one's best result on this device, and builds the run record that is saved
// when a scenario ends (format "legatum-scenario-run", read by tools/scenarios/compare.py).
import type { Faction, Order } from '../types';

export type ScenarioOutcome = 'Playing' | 'Won' | 'Lost' | 'Complete';

/** `legatum_core::scenario::Status`, from `GameClient.scenarioStatus()`. */
export interface ScenarioStatus {
  turn: number;
  start_turn: number;
  end_turn: number;
  turns_left: number;
  outcome: ScenarioOutcome;
  reason: string;
  score: number;
  conditions: { label: string; progress: string; met: boolean; keep: boolean }[];
  score_lines: { label: string; value: number; weight: number; points: number }[];
  /** The cities the objective is about (`legatum_core::scenario::objective_targets`). */
  targets?: ScenarioTarget[];
}

type ScenarioTargetKind = 'Take' | 'Hold' | 'Contain';

/** `legatum_core::scenario::Target`: marked on the map from turn 1, even in fog. */
export interface ScenarioTarget {
  kind: ScenarioTargetKind;
  city: number;
  name: string;
  owner: number;
  pos: { q: number; r: number };
  /** e.g. "6 tiles north-east of Tessel Market"; empty for the seat's own cities. */
  bearing: string;
  /** Tiles from the seat's nearest city (0 for its own). */
  distance: number;
}

/** The target the objective bar points at: a city to take, else the nearest city of the
 *  empire to contain, else the city to hold that the objective or briefing names first (the
 *  threatened one when every city must be held), else the first city to hold. */
export function leadTarget(targets: ScenarioTarget[] | undefined, text = ''): ScenarioTarget | undefined {
  const of = (kind: ScenarioTargetKind) => (targets ?? []).filter((t) => t.kind === kind).sort((a, b) => a.distance - b.distance || a.city - b.city);
  const mention = (t: ScenarioTarget) => text.search(new RegExp(`\\b${t.name.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')}\\b`));
  const named = of('Hold').map((t) => ({ t, at: mention(t) })).filter((m) => m.at >= 0).sort((a, b) => a.at - b.at)[0]?.t;
  return of('Take')[0] ?? of('Contain')[0] ?? named ?? of('Hold')[0];
}

/** One line naming the target and where it is, for the objective bar and briefing. */
export function targetText(t: ScenarioTarget): string {
  const verb = t.kind === 'Take' ? 'Take' : t.kind === 'Hold' ? 'Hold' : 'Contain';
  return t.bearing ? `${verb} ${t.name}: ${t.bearing}` : `${verb} ${t.name}`;
}

export interface ScenarioBaseline {
  runs: number;
  difficulty: string;
  median: number;
  p10: number;
  p90: number;
  /** Share of runs that met the objective, in percent; null for score-only scenarios. */
  win_percent: number | null;
  /** Median turns the runs that met the objective took, from the first turn of the window. */
  median_turns?: number | null;
  measured_at: string;
}

/** The Scenario Lab card (`legatum_wasm::scenario::LabCard`): what the scenario asks a human. */
interface LabCard {
  /** The question it answers, in plain words. */
  question: string;
  /** Two plain lines setting the scene. */
  blurb: string;
  /** What the feedback box asks, when it differs from the question. */
  ask?: string;
  /** The seat's era when the scenario starts (0 is Ancient), for the card's painting. */
  era?: number;
  /** A change the scenario waits for; while set it is built but not listed (delete it when the change lands). */
  requires?: string;
  /** Who the opponent is when it is not the plain Hard AI (a scripted rusher), shown on the briefing. */
  opponent?: string;
  order?: number;
}

export interface ScenarioFile {
  format: 'legatum-scenario';
  version: 1;
  id: string;
  revision: number;
  title: string;
  faction: Faction;
  seat: number;
  difficulty: string;
  minutes: number;
  briefing: string;
  tests?: string;
  objective: { summary: string; turns: number; win?: unknown[]; keep?: unknown[]; end_on_win?: boolean; score: unknown[] };
  predictions: { good_play: string; ai_play: string; ai_score: { median: number; low: number; high: number; win_percent: number } };
  source?: string;
  baseline?: ScenarioBaseline | null;
  /** The human player's own result from this position (war scenarios cut from playtest games). */
  human?: { met: boolean; turns: number; units_lost: number; cities_lost: number; summary: string } | null;
  /** Present on every scenario the Lab lists. */
  lab?: LabCard;
  state: { encoding: string; hash: string; data: string };
}

/** A scenario as shipped: its file and the exact JSON text the engine loads. */
export interface ScenarioEntry {
  file: ScenarioFile;
  text: string;
}

/**
 * Every scenario the Lab can list: the ones cut by `tools/scenarios/cut.sh` (`scenarios/lab/`) and
 * the war scenarios cut from playtest games (`scenarios/war/`). Only files with a Lab card are
 * listed; the rest are for the simulator.
 */
const raw = import.meta.glob(['../../../scenarios/lab/*.json', '../../../scenarios/war/*.json'], { query: '?raw', import: 'default', eager: true }) as Record<string, string>;

/**
 * Tests may shorten a scenario (e.g. its turn window) before it loads, through
 * `globalThis.legatumScenarioOverrides = { [id]: { objective: { turns: 1 } } }` set by an init script.
 */
function withOverrides(text: string): string {
  const overrides = (globalThis as { legatumScenarioOverrides?: Record<string, { objective?: Partial<ScenarioFile['objective']> }> }).legatumScenarioOverrides;
  if (!overrides) return text;
  const file = JSON.parse(text) as ScenarioFile;
  const o = overrides[file.id];
  if (!o) return text;
  return JSON.stringify({ ...file, objective: { ...file.objective, ...o.objective } });
}

function parseAll(texts: string[]): ScenarioEntry[] {
  return texts
    .map((t) => withOverrides(t))
    .map((text) => ({ text, file: JSON.parse(text) as ScenarioFile }))
    .filter((e) => e.file.format === 'legatum-scenario' && e.file.lab)
    .sort((a, b) => (a.file.lab?.order ?? 1000) - (b.file.lab?.order ?? 1000) || a.file.title.localeCompare(b.file.title));
}

/** The scenarios the Lab lists, in display order: those with a card that wait for no change. */
export function parseEntries(texts: string[]): ScenarioEntry[] {
  return parseAll(texts).filter((e) => !e.file.lab?.requires);
}

export function scenarioCatalog(): ScenarioEntry[] {
  return parseEntries(Object.values(raw));
}

/** Scenarios built and measured but held back until a change lands (`lab.requires`), for tests. */
export function heldBackCatalog(): ScenarioEntry[] {
  return parseAll(Object.values(raw)).filter((e) => e.file.lab?.requires);
}

/** A question that has no scenario yet: its card is listed but cannot be started. */
export interface SoonSlot {
  id: string;
  title: string;
  question: string;
  blurb: string;
  /** What has to land first, in plain words. */
  waitingFor: string;
}

/** Slots held for questions waiting on a feature. Delete a slot when its scenario ships. */
export const SOON_SLOTS: SoonSlot[] = [];

// ---- Best results on this device ----

const RECORDS_KEY = 'legatum.scenarios.v1';

export interface ScenarioRecord {
  revision: number;
  best: number;
  bestOutcome: ScenarioOutcome;
  /** Fewest turns a winning run took (absent until one wins). */
  bestTurns?: number;
  plays: number;
  lastAt: string;
}

export function readRecords(): Record<string, ScenarioRecord> {
  try {
    const v = JSON.parse(localStorage.getItem(RECORDS_KEY) ?? '{}') as unknown;
    return v && typeof v === 'object' ? v as Record<string, ScenarioRecord> : {};
  } catch {
    return {};
  }
}

/** Merge one finished run into the records. A new revision of a scenario starts a fresh record. */
export function nextRecords(records: Record<string, ScenarioRecord>, id: string, revision: number, status: ScenarioStatus, at: string): Record<string, ScenarioRecord> {
  const old = records[id]?.revision === revision ? records[id] : undefined;
  const better = !old || status.score > old.best;
  const taken = turnsTaken(status);
  const bestTurns = status.outcome === 'Won' ? Math.min(old?.bestTurns ?? taken, taken) : old?.bestTurns;
  return {
    ...records,
    [id]: {
      revision,
      best: better ? status.score : old.best,
      bestOutcome: better ? status.outcome : old.bestOutcome,
      ...(bestTurns === undefined ? {} : { bestTurns }),
      plays: (old?.plays ?? 0) + 1,
      lastAt: at,
    },
  };
}

export function saveRecord(id: string, revision: number, status: ScenarioStatus): void {
  try {
    localStorage.setItem(RECORDS_KEY, JSON.stringify(nextRecords(readRecords(), id, revision, status, new Date().toISOString())));
  } catch {
    // Storage blocked: the run file still has the result.
  }
}

// ---- The run record ----

interface ScenarioRunTurn {
  /** The turn these orders were given for. */
  turn: number;
  /** The human seat's orders, exactly as submitted. */
  orders: Order[];
  /** State hash after the turn resolved. */
  hash: string;
  score: number;
  outcome: ScenarioOutcome;
  turns_left: number;
  /** Milliseconds the player took over this turn, from its start to ending it. */
  ms: number;
  /** The seat's notable report lines for this turn ("what happened"). */
  events: { text: string; tone: string; category: string; weight: number; mine: boolean }[];
}

/** What the player says after a scenario (Fun and Fair run 1 to 5). */
export type FeltLike = 'too-easy' | 'about-right' | 'too-hard' | 'confusing';

export const FELT_CHOICES: { id: FeltLike; label: string }[] = [
  { id: 'too-easy', label: 'Too easy' },
  { id: 'about-right', label: 'About right' },
  { id: 'too-hard', label: 'Too hard' },
  { id: 'confusing', label: 'Confusing' },
];

export interface ScenarioFeedback {
  fun: number | null;
  fair: number | null;
  felt: FeltLike | null;
  text: string;
  at: string;
}

export interface ScenarioRun {
  format: 'legatum-scenario-run';
  version: 1;
  scenario: { id: string; revision: number; title: string; question: string; seat: number; faction: Faction; state_hash: string; turns: number };
  player: 'human';
  client: string;
  started_at: string;
  finished_at: string | null;
  initial_state: ScenarioFile['state'];
  /** Objective status before the first turn. */
  initial: ScenarioStatus;
  turns: ScenarioRunTurn[];
  outcome: ScenarioStatus | null;
  score: number | null;
  /** The player left before the objective was decided. */
  abandoned?: boolean;
  /** The player's answers on the feedback card. */
  feedback?: ScenarioFeedback | null;
  /** The engine's full owner history (setup, every player's orders, hashes, telemetry). */
  archive: unknown;
}

export function newRun(entry: ScenarioEntry, initial: ScenarioStatus, client: string, now = new Date().toISOString()): ScenarioRun {
  const f = entry.file;
  return {
    format: 'legatum-scenario-run',
    version: 1,
    scenario: { id: f.id, revision: f.revision, title: f.title, question: f.lab?.question ?? '', seat: f.seat, faction: f.faction, state_hash: f.state.hash, turns: f.objective.turns },
    player: 'human',
    client,
    started_at: now,
    finished_at: null,
    initial_state: f.state,
    initial,
    turns: [],
    outcome: null,
    score: null,
    archive: null,
  };
}

/** A file name for a run: `<id>-<UTC start time>.json`, safe on every platform. One run keeps one
 * name from its first turn to its feedback, so each turn's checkpoint is replaced by the next. */
export function runFileName(run: ScenarioRun): string {
  const stamp = run.started_at.replace(/\.\d+Z$/, 'Z').replace(/[:]/g, '-');
  return `${run.scenario.id.replace(/[^A-Za-z0-9_-]/g, '_')}-${stamp}.json`;
}

/** Turns a run has taken so far, counted from the window's first turn. */
function turnsTaken(status: Pick<ScenarioStatus, 'turn' | 'start_turn'>): number {
  return status.turn - status.start_turn;
}

/** The kinds of scenario the comparison words differently. */
function kindOf(f: ScenarioFile): 'race' | 'hold' | 'score' {
  const hold = (f.objective.win ?? []).includes('HoldStartingCities');
  if (f.objective.end_on_win) return 'race';
  if (hold && !f.objective.end_on_win) return 'hold';
  return 'score';
}

const tries = (b: ScenarioBaseline, pct: number) => `${Math.round((pct * b.runs) / 100)} of ${b.runs} tries`;

/** The Hard AI's result from the same start, in one line for the card and the briefing. */
export function aiResultLine(f: ScenarioFile): string {
  const b = f.baseline;
  if (!b) return 'Hard AI: not measured yet';
  const kind = kindOf(f);
  if (kind === 'race') {
    const pct = b.win_percent ?? 0;
    if (pct === 0) return `Hard AI: failed all ${b.runs} tries`;
    return `Hard AI: ${b.median_turns != null ? `did it in ${turnsWord(b.median_turns)}` : 'did it'}, ${tries(b, pct)}`;
  }
  if (kind === 'hold') return `Hard AI: held every city in ${tries(b, b.win_percent ?? 0)}`;
  return `Hard AI: a typical score of ${b.median}`;
}

/** Your best result, in one line for the card. */
export function bestResultLine(f: ScenarioFile, record?: ScenarioRecord): string {
  if (!record || record.revision !== f.revision) return 'Not played yet';
  const met = record.bestOutcome === 'Won' || record.bestOutcome === 'Complete';
  if (kindOf(f) === 'race' && record.bestTurns !== undefined) return `Your best: ${turnsWord(record.bestTurns)}`;
  return `Your best: ${met ? OUTCOME_TITLE[record.bestOutcome].toLowerCase() : 'failed'}, score ${record.best}`;
}

export interface AiComparison {
  /** What you did, in a plain sentence. */
  you: string;
  /** What the Hard AI did from the same start. */
  ai: string;
  /** How the two compare, when they can be compared. */
  verdict?: string;
}

/** The result card's "you against the Hard AI" lines. */
export function compareToAi(f: ScenarioFile, status: ScenarioStatus): AiComparison {
  const b = f.baseline;
  const won = status.outcome === 'Won' || status.outcome === 'Complete';
  const kind = kindOf(f);
  const n = turnsTaken(status);
  if (kind === 'race') {
    const you = won ? `You took ${turnsWord(n)}.` : `You did not meet the objective in ${turnsWord(f.objective.turns)}.`;
    if (!b) return { you, ai: 'The Hard AI has not played this start yet.' };
    const pct = b.win_percent ?? 0;
    const ai = pct === 0
      ? `The Hard AI failed all ${b.runs} tries from the same start.`
      : `The Hard AI ${b.median_turns != null ? `took ${turnsWord(b.median_turns)}` : 'met the objective'} from the same start, and did it in ${tries(b, pct)}.`;
    let verdict: string | undefined;
    if (won && pct > 0 && b.median_turns != null) {
      const d = b.median_turns - n;
      verdict = d > 0 ? `That is ${turnsWord(d)} faster than the Hard AI.` : d < 0 ? `That is ${turnsWord(-d)} slower than the Hard AI.` : 'That matches the Hard AI.';
    } else if (won && pct === 0) {
      verdict = 'You did what the Hard AI could not.';
    } else if (!won && pct > 0) {
      verdict = 'The Hard AI managed it where you did not.';
    }
    return { you, ai, verdict };
  }
  if (kind === 'hold') {
    const you = won ? `You held every city for all ${turnsWord(f.objective.turns)}.` : status.reason;
    if (!b) return { you, ai: 'The Hard AI has not played this start yet.' };
    const pct = b.win_percent ?? 0;
    const ai = `The Hard AI held every city in ${tries(b, pct)} from the same start.`;
    const verdict = won && pct < 50 ? 'You held where the Hard AI usually fell.' : !won && pct >= 50 ? 'The Hard AI usually held this position.' : undefined;
    return { you, ai, verdict };
  }
  const you = `You scored ${status.score}.`;
  if (!b) return { you, ai: 'The Hard AI has not played this start yet.' };
  return { you, ai: `${versusBaseline(status.score, b)} (median ${b.median}).`, verdict: undefined };
}

/** Where a finished run's score falls among the AI baseline runs, in words. */
export function versusBaseline(score: number, baseline: ScenarioBaseline): string {
  if (score > baseline.p90) return `Better than 90% of the Hard AI's ${baseline.runs} tries`;
  if (score > baseline.median) return `Better than the Hard AI's median of ${baseline.median}`;
  if (score === baseline.median) return `Level with the Hard AI's median of ${baseline.median}`;
  if (score >= baseline.p10) return `Below the Hard AI's median of ${baseline.median}`;
  return `Below 90% of the Hard AI's ${baseline.runs} tries`;
}

/** "Turn 23 of 36" and what is left, for the objective bar. */
export function turnsText(status: ScenarioStatus): string {
  if (status.outcome !== 'Playing') return `Ended on turn ${status.turn}`;
  return `${status.turns_left} turn${status.turns_left === 1 ? '' : 's'} left`;
}

export const OUTCOME_TITLE: Record<ScenarioOutcome, string> = {
  Playing: 'In progress',
  Won: 'Objective met',
  Lost: 'Objective failed',
  Complete: 'Window complete',
};

export const turnsWord = (n: number) => `${n} turn${n === 1 ? '' : 's'}`;
