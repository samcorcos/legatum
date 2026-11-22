// The last line of defence: a calm full-screen card when the app itself breaks, instead of a blank page.
// Three ways in: a render error anywhere under the root (RootErrorBoundary), an unhandled error or
// rejection that `isFatal` recognises (installGlobalErrorHandlers), and an engine that failed to download
// or start (main.tsx renders the card in place of the app).
//
// This file must keep working when the rest of the app is broken, so it imports nothing from the game
// (no store, no data, no platform code): it reads the autosave straight from localStorage under the key
// the store writes (store.ts AUTOSAVE_KEY) and names the file the way platform/files.ts does.
// When autosaves move to IndexedDB, `readAutosaveText` must read from there too (keep it a plain,
// dependency-free read, and keep the localStorage fallback for saves written by older builds).
import { Component, useEffect, useRef, useState, type ErrorInfo, type KeyboardEvent as ReactKeyboardEvent, type ReactNode } from 'react';
import { buildLabel } from '../buildLabel';
import './crash.css';

/** store.ts AUTOSAVE_KEY. Duplicated on purpose: this screen must not import the store. */
export const CRASH_AUTOSAVE_KEY = 'legatum.autosave.v1';

export type CrashKind = 'crash' | 'engine';

/** The raw autosave text and its turn, or null when there is none or storage is blocked. */
export function readAutosaveText(): { text: string; turn: number | null } | null {
  try {
    const text = window.localStorage.getItem(CRASH_AUTOSAVE_KEY);
    if (!text) return null;
    let turn: number | null = null;
    try {
      const parsed = JSON.parse(text) as { turn?: unknown };
      if (typeof parsed?.turn === 'number' && Number.isInteger(parsed.turn)) turn = parsed.turn;
    } catch { /* A damaged save is still worth keeping as a file. */ }
    return { text, turn };
  } catch {
    return null;
  }
}

/** The save's file name, as Save to file names it in a browser (platform/files.ts). */
export const autosaveFileName = (turn: number | null) => (turn === null ? 'legatum-autosave.legatum.json' : `legatum-turn-${turn}.legatum.json`);

/** Writes the autosave to a file the player keeps. Returns false when there is nothing to save or it failed. */
export async function downloadAutosave(): Promise<boolean> {
  const save = readAutosaveText();
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
    `Build: ${build}`,
    `Time: ${now.toISOString()}`,
    `Browser: ${agent}`,
    '',
    `Error: ${message}`,
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
  copy: (text: string) => Promise<boolean>;
}

const DEFAULT_HANDLERS: CrashHandlers = {
  reload: () => window.location.reload(),
  save: downloadAutosave,
  copy: copyText,
};

/** The full-screen card. Reload is the one primary and has focus; Tab stays inside the card. */
export function CrashCard({ error, kind = 'crash', handlers }: { error: unknown; kind?: CrashKind; handlers?: Partial<CrashHandlers> }) {
  const act = { ...DEFAULT_HANDLERS, ...handlers };
  const cardRef = useRef<HTMLDivElement>(null);
  const reloadRef = useRef<HTMLButtonElement>(null);
  const [hasSave] = useState(() => readAutosaveText() !== null);
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

  const engine = kind === 'engine';
  const heading = engine ? 'The game could not load' : 'Something went wrong';
  const sentence = engine
    ? 'The game engine did not finish downloading. Check your connection and reload.'
    : hasSave
      ? 'The game hit an error. Your last finished turn is saved in this browser.'
      : 'The game hit an error. Reload to start again.';

  return (
    <main className="crash-screen">
      <div
        ref={cardRef}
        className="crash-card"
        role="alertdialog"
        aria-modal="true"
        aria-labelledby="crash-title"
        aria-describedby="crash-text"
        data-testid="crash-screen"
        data-kind={kind}
        onKeyDown={onKeyDown}
      >
        <p className="crash-kicker">Legatum</p>
        <h1 id="crash-title">{heading}</h1>
        <p id="crash-text" className="crash-text">{sentence}</p>
        <div className="crash-actions">
          <button ref={reloadRef} type="button" className="btn primary crash-reload" onClick={() => act.reload()}>
            Reload <span className="kbd">Return</span>
          </button>
          {hasSave && <button type="button" className="btn" onClick={() => void onSave()}>Save to file</button>}
          <button type="button" className="btn" onClick={() => void onCopy()}>Copy error details</button>
        </div>
        {(saveNote || copyNote) && (
          <p className="crash-note" role="status">{[saveNote, copyNote].filter(Boolean).join(' ')}</p>
        )}
        {manual && <textarea className="crash-details" readOnly value={manual} aria-label="Error details" rows={6} onFocus={(e) => e.currentTarget.select()} />}
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
