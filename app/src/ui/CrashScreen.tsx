// The last line of defence: a calm full-screen card when the app itself breaks, instead of a blank page.
// Three ways in: a render error anywhere under the root (RootErrorBoundary), an unhandled error or
// rejection that `isFatal` recognises (installGlobalErrorHandlers), and an engine that failed to download
// or start (main.tsx renders the card in place of the app).
//
// This file must keep working when the rest of the app is broken, so it imports nothing from the game
// (no store, no data, no platform code). It reads the autosaves straight from the browser's database and
// from the old localStorage slot, the way platform/autosaveStore.ts writes them: that module's database
// name, store name, record shape and skipped-slots key are duplicated below on purpose. It names the file
// the way platform/files.ts does.
import { Component, useEffect, useRef, useState, type ErrorInfo, type KeyboardEvent as ReactKeyboardEvent, type ReactNode } from 'react';
import { buildLabel } from '../buildLabel';
import './crash.css';

/** store.ts AUTOSAVE_KEY (the single slot older builds wrote). Duplicated on purpose: this screen must not import the store. */
export const CRASH_AUTOSAVE_KEY = 'legatum.autosave.v1';
/** platform/autosaveStore.ts AUTOSAVE_DB, its saves store, and SKIPPED_SLOTS_KEY. */
const AUTOSAVE_DB = 'legatum-autosave-v1';
const SKIPPED_KEY = 'legatum.autosave.skipped.v1';
/** Per tab: the turn the card was last shown for and how many times, so a repeat can offer the turn before. */
const CRASH_MARKER_KEY = 'legatum.crash.v1';

export type CrashKind = 'crash' | 'engine';

interface StoredSave { id: string; turn: number | null; savedAt: string; text: string }

/** Every record in the autosave database. Empty when there is none or it cannot be read; never creates the database. */
function readDatabase(): Promise<StoredSave[]> {
  return new Promise((resolve) => {
    const none = () => resolve([]);
    const timer = setTimeout(none, 1500);
    const done = (found: StoredSave[]) => { clearTimeout(timer); resolve(found); };
    try {
      if (typeof indexedDB === 'undefined') return done([]);
      const request = indexedDB.open(AUTOSAVE_DB);
      // The database does not exist yet: stop it being created empty.
      request.onupgradeneeded = () => { try { request.transaction?.abort(); } catch { /* already gone */ } };
      request.onerror = () => done([]);
      request.onblocked = () => done([]);
      request.onsuccess = () => {
        const db = request.result;
        try {
          if (!db.objectStoreNames.contains('saves')) { db.close(); return done([]); }
          const all = db.transaction('saves', 'readonly').objectStore('saves').getAll();
          all.onsuccess = () => {
            db.close();
            done((all.result as Partial<StoredSave & { turn: unknown }>[]).filter((r) => typeof r?.text === 'string' && typeof r.id === 'string')
              .map((r) => ({ id: r.id!, turn: Number.isInteger(r.turn) ? (r.turn as number) : null, savedAt: typeof r.savedAt === 'string' ? r.savedAt : '', text: r.text! })));
          };
          all.onerror = () => { db.close(); done([]); };
        } catch { try { db.close(); } catch { /* closed */ } done([]); }
      };
    } catch {
      done([]);
    }
  });
}

const timeOf = (save: StoredSave) => { const t = Date.parse(save.savedAt); return Number.isFinite(t) ? t : -1; };

/** Every autosave this browser holds, newest first, minus the slots the player stepped back from. */
async function readAutosaves(): Promise<StoredSave[]> {
  const saves = await readDatabase();
  try {
    const text = window.localStorage.getItem(CRASH_AUTOSAVE_KEY);
    if (text) {
      let turn: number | null = null;
      let savedAt = '';
      try {
        const parsed = JSON.parse(text) as { turn?: unknown; savedAt?: unknown };
        if (typeof parsed?.turn === 'number' && Number.isInteger(parsed.turn)) turn = parsed.turn;
        if (typeof parsed?.savedAt === 'string') savedAt = parsed.savedAt;
      } catch { /* A damaged save is still worth keeping as a file. */ }
      // The old slot also holds a save the database refused: take it when it is newer than what the database has.
      if (!saves.length || (savedAt && !saves.some((s) => s.savedAt === savedAt))) saves.push({ id: `legacy:${turn}`, turn, savedAt, text });
    }
  } catch { /* storage blocked */ }
  let skipped: unknown = [];
  try { skipped = JSON.parse(window.localStorage.getItem(SKIPPED_KEY) ?? '[]'); } catch { /* none */ }
  return saves.filter((s) => !(Array.isArray(skipped) && skipped.includes(s.id))).sort((a, b) => timeOf(b) - timeOf(a) || (b.turn ?? 0) - (a.turn ?? 0));
}

/** The newest autosave's raw text and its turn, or null when there is none or storage is blocked. */
export async function readAutosaveText(): Promise<{ text: string; turn: number | null } | null> {
  const [newest] = await readAutosaves();
  return newest ? { text: newest.text, turn: newest.turn } : null;
}

/** "Reload on the previous turn": hide the newest slot, so the one before it is what Continue offers after the reload. */
async function stepBack(): Promise<void> {
  const [newest] = await readAutosaves();
  if (!newest) return;
  try {
    const skipped = JSON.parse(window.localStorage.getItem(SKIPPED_KEY) ?? '[]') as unknown;
    window.localStorage.setItem(SKIPPED_KEY, JSON.stringify([...(Array.isArray(skipped) ? skipped : []), newest.id]));
  } catch { /* blocked: Reload still works */ }
}

/** Counts how many times this card has shown for this turn in this tab (a Reload keeps the tab's count). */
function noteCrash(turn: number | null): number {
  try {
    const kept = JSON.parse(window.sessionStorage.getItem(CRASH_MARKER_KEY) ?? 'null') as { turn?: unknown; count?: unknown } | null;
    const count = kept && kept.turn === turn && typeof kept.count === 'number' ? kept.count + 1 : 1;
    window.sessionStorage.setItem(CRASH_MARKER_KEY, JSON.stringify({ turn, count }));
    return count;
  } catch {
    return 1;
  }
}

/** The save's file name, as Save to file names it in a browser (platform/files.ts). */
export const autosaveFileName = (turn: number | null) => (turn === null ? 'legatum-autosave.legatum.json' : `legatum-turn-${turn}.legatum.json`);

/** Writes the autosave to a file the player keeps. Returns false when there is nothing to save or it failed. */
export async function downloadAutosave(): Promise<boolean> {
  const save = await readAutosaveText();
  if (!save) return false;
  try {
    const url = URL.createObjectURL(new Blob([save.text], { type: 'application/json' }));
    const a = document.createElement('a');
    a.href = url;
    a.download = autosaveFileName(save.turn);
    document.body.appendChild(a);
    try { a.click(); } finally {
      a.remove();
      setTimeout(() => URL.revokeObjectURL(url), 1000);
    }
    return true;
  } catch (error) {
    console.error('Could not save the autosave to a file', error);
    return false;
  }
}

/** What "Copy error details" copies: the error, the build and the browser. No game state, no player names. */
export function errorDetails(error: unknown, kind: CrashKind, now = new Date()): string {
  const err = error instanceof Error ? error : null;
  const message = err ? `${err.name}: ${err.message}` : (() => { try { return String(error); } catch { return 'Unknown error'; } })();
  let build = 'unknown';
  try { build = buildLabel; } catch { /* keep "unknown" */ }
  let agent = 'unknown';
  try { agent = navigator.userAgent; } catch { /* keep "unknown" */ }
  return [
    `Legatum error report (${kind === 'engine' ? 'engine did not load' : 'app error'})`,
    `Error: ${message}`,
    '',
    `Build: ${build}`,
    `Time: ${now.toISOString()}`,
    `Browser: ${agent}`,
    ...(err?.stack ? ['', 'Stack:', err.stack] : []),
    ...(err && 'cause' in err && err.cause !== undefined ? ['', `Cause: ${String(err.cause)}`] : []),
  ].join('\n');
}

async function copyText(text: string): Promise<boolean> {
  try {
    await navigator.clipboard.writeText(text);
    return true;
  } catch {
    try {
      const area = document.createElement('textarea');
      area.value = text;
      area.setAttribute('readonly', '');
      area.style.position = 'fixed';
      area.style.opacity = '0';
      document.body.appendChild(area);
      area.select();
      const ok = document.execCommand('copy');
      area.remove();
      return ok;
    } catch {
      return false;
    }
  }
}

export interface CrashHandlers {
  reload: () => void;
  save: () => Promise<boolean>;
  /** Hides the newest autosave so Continue offers the turn before it, then reloads. */
  previous: () => Promise<void>;
  copy: (text: string) => Promise<boolean>;
}

const DEFAULT_HANDLERS: CrashHandlers = {
  reload: () => window.location.reload(),
  save: downloadAutosave,
  previous: async () => { await stepBack(); window.location.reload(); },
  copy: copyText,
};

/** The full-screen card. Reload is the one primary and has focus; Tab stays inside the card. */
export function CrashCard({ error, kind = 'crash', handlers }: { error: unknown; kind?: CrashKind; handlers?: Partial<CrashHandlers> }) {
  const act = { ...DEFAULT_HANDLERS, ...handlers };
  const cardRef = useRef<HTMLDivElement>(null);
  const reloadRef = useRef<HTMLButtonElement>(null);
  const detailsRef = useRef<HTMLTextAreaElement>(null);
  // What is saved, and whether this is the second time the card shows for the same turn (then the turn before is on offer).
  const [saved, setSaved] = useState<{ any: boolean; again: boolean; earlier: boolean } | null>(null);
  const counted = useRef(false);
  const [saveNote, setSaveNote] = useState<string | null>(null);
  const [copyNote, setCopyNote] = useState<string | null>(null);
  const [manual, setManual] = useState<string | null>(null);

  useEffect(() => {
    reloadRef.current?.focus();
    // Focus that escapes the card (a click on the backdrop, a stray script) comes back to Reload.
    const keep = (event: FocusEvent) => {
      if (cardRef.current && event.target instanceof Node && !cardRef.current.contains(event.target)) reloadRef.current?.focus();
    };
    document.addEventListener('focusin', keep);
    return () => document.removeEventListener('focusin', keep);
  }, []);

  useEffect(() => {
    if (counted.current) return;
    counted.current = true;
    void readAutosaves().then((all) => setSaved({ any: all.length > 0, again: kind === 'crash' && all.length > 0 && noteCrash(all[0].turn) >= 2, earlier: all.length > 1 }));
  }, [kind]);

  // When copying failed, the text to copy by hand takes focus (its onFocus selects it all).
  useEffect(() => {
    if (manual) detailsRef.current?.focus();
  }, [manual]);

  const onKeyDown = (event: ReactKeyboardEvent<HTMLDivElement>) => {
    // Return always means Reload, unless another button of the card has focus (then that button acts).
    if (event.key === 'Enter' && !(event.target instanceof HTMLButtonElement) && !(event.target instanceof HTMLTextAreaElement)) {
      event.preventDefault();
      act.reload();
      return;
    }
    if (event.key !== 'Tab' || !cardRef.current) return;
    const stops = [...cardRef.current.querySelectorAll<HTMLElement>('button:not([disabled]), textarea')];
    if (!stops.length) return;
    const at = stops.indexOf(document.activeElement as HTMLElement);
    const next = event.shiftKey ? (at <= 0 ? stops.length - 1 : at - 1) : (at === -1 || at === stops.length - 1 ? 0 : at + 1);
    event.preventDefault();
    stops[next].focus();
  };

  const onSave = async () => {
    const ok = await act.save();
    setSaveNote(ok ? 'Saved. Load it later with Load Game on the title screen.' : 'Could not save the file.');
  };
  const onCopy = async () => {
    const text = errorDetails(error, kind);
    if (await act.copy(text)) { setCopyNote('Copied.'); setManual(null); }
    else { setCopyNote('Could not copy. Select the text below and copy it.'); setManual(text); }
  };

  const hasSave = !!saved?.any;
  const engine = kind === 'engine';
  const heading = engine ? 'The game could not load' : 'Something went wrong';
  const goBack = !engine && !!saved?.again && saved.earlier;
  const sentence = engine
    ? 'The game did not finish loading. Check your connection, then reload the page.'
    : goBack
      ? 'The game hit the same error again. Choose Reload on the previous turn, then Continue, to go back one turn.'
      : hasSave
      ? 'The game hit an error. Your last finished turn is saved in this browser. Reload, then choose Continue.'
      : 'The game hit an error. Reload to start again.';

  return (
    <main className="crash-screen">
      <div
        ref={cardRef}
        className={`crash-card${goBack ? ' wide' : ''}`}
        role="alertdialog"
        aria-modal="true"
        aria-labelledby="crash-title"
        aria-describedby="crash-text"
        data-testid="crash-screen"
        data-kind={kind}
        onKeyDown={onKeyDown}
      >
        <h1 id="crash-title">{heading}</h1>
        <p id="crash-text" className="crash-text">{sentence}</p>
        <div className="crash-actions">
          <button ref={reloadRef} type="button" className="btn primary crash-reload" onClick={() => act.reload()}>
            Reload <span className="kbd">Return</span>
          </button>
          {hasSave && <button type="button" className="btn" onClick={() => void onSave()}>Save to file</button>}
          {goBack && <button type="button" className="btn" onClick={() => void act.previous()}>Reload on the previous turn</button>}
        </div>
        <p className="crash-more"><button type="button" className="btn quiet" onClick={() => void onCopy()}>Copy error details</button></p>
        <div role="status">
          {saveNote && <p className="crash-note">{saveNote}</p>}
          {copyNote && <p className="crash-note">{copyNote}</p>}
        </div>
        {manual && <textarea ref={detailsRef} className="crash-details" readOnly value={manual} aria-label="Error details" rows={6} onFocus={(e) => e.currentTarget.select()} />}
      </div>
    </main>
  );
}

// ---------- Window-level errors ----------

type FatalListener = (error: unknown) => void;
const listeners = new Set<FatalListener>();
let pendingFatal: { error: unknown } | null = null;

/** Show the crash card for this error (the root boundary listens). Before the boundary mounts it waits. */
export function reportFatal(error: unknown) {
  if (!listeners.size) { pendingFatal ??= { error }; return; }
  for (const listen of listeners) listen(error);
}

function onFatal(listen: FatalListener): () => void {
  listeners.add(listen);
  if (pendingFatal) { const { error } = pendingFatal; pendingFatal = null; listen(error); }
  return () => { listeners.delete(listen); };
}

/** Lazy-loaded code that failed to download after a new release (each browser words it differently). */
const CHUNK_FAILED = /Failed to fetch dynamically imported module|error loading dynamically imported module|Importing a module script failed|Unable to preload CSS/i;

/**
 * Whether an unhandled error leaves the game unusable. An allowlist: anything not recognised is only
 * logged, so a stray rejection from a background task never covers a game that still works.
 * - The engine trapped (a Rust panic or out-of-bounds access surfaces as WebAssembly.RuntimeError):
 *   its memory can no longer be trusted, so playing on would be playing a broken game.
 * - Part of the app failed to download (a lazy chunk after a release): a reload fetches the new one.
 */
export function isFatal(error: unknown): boolean {
  if (typeof WebAssembly !== 'undefined' && typeof WebAssembly.RuntimeError === 'function' && error instanceof WebAssembly.RuntimeError) return true;
  const message = error instanceof Error ? error.message : typeof error === 'string' ? error : '';
  return CHUNK_FAILED.test(message);
}

let installed = false;

/** Logs every unhandled error and rejection, and routes the fatal ones to the crash card. */
export function installGlobalErrorHandlers(target: Window = window) {
  if (installed) return;
  installed = true;
  target.addEventListener('error', (event: ErrorEvent) => {
    const error = event.error ?? event.message;
    if (isFatal(error)) { console.error('Legatum: fatal error', error); reportFatal(error); }
    else console.warn('Legatum: unhandled error', error);
  });
  target.addEventListener('unhandledrejection', (event: PromiseRejectionEvent) => {
    console.warn('Legatum: unhandled rejection', event.reason);
    if (isFatal(event.reason)) reportFatal(event.reason);
  });
}

/** Test seam: forget the install and any waiting fatal error. */
export function resetCrashStateForTests() {
  installed = false;
  pendingFatal = null;
  listeners.clear();
}

/** The root error boundary: a render error anywhere below, or a fatal window error, shows the card. */
export class RootErrorBoundary extends Component<{ children: ReactNode; handlers?: Partial<CrashHandlers> }, { error: unknown; failed: boolean }> {
  state = { error: null as unknown, failed: false };
  private stop: (() => void) | null = null;

  static getDerivedStateFromError(error: unknown) {
    return { error, failed: true };
  }

  componentDidMount() {
    this.stop = onFatal((error) => this.setState((s) => (s.failed ? s : { error, failed: true })));
  }

  componentWillUnmount() {
    this.stop?.();
  }

  componentDidCatch(error: unknown, info: ErrorInfo) {
    console.error('Legatum: the screen failed to draw', error, info.componentStack);
  }

  render() {
    return this.state.failed ? <CrashCard error={this.state.error} handlers={this.props.handlers} /> : this.props.children;
  }
}
