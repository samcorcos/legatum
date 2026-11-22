// Saving a scenario run where Claude can read it afterwards (after every turn, and when it ends):
// - desktop (Tauri): ~/LegatumData/scenarios/<id>-<timestamp>.json, through the native command
//   `scenario_save_run` (app/src-tauri/src/lib.rs);
// - web or phone served by legatum-server: the server's saves directory, <saves>/scenarios/;
// - a static web host: a download of the same file.
import { isTauri, invoke } from '@tauri-apps/api/core';
import { runFileName, type ScenarioRun } from '../scenarios/scenarios';
import { HOSTED } from './hosted';

export interface SavedRun {
  where: 'disk' | 'server' | 'download';
  /** Where the file landed, when known. */
  path?: string;
}

async function serverMode(): Promise<boolean> {
  if (HOSTED) return false;
  try {
    const status = await fetch('/api/status', { signal: AbortSignal.timeout(5000), cache: 'no-store' });
    if (!status.ok || !status.headers.get('content-type')?.includes('application/json')) return false;
    return (await status.json() as { mode?: string }).mode === 'server';
  } catch {
    return false;
  }
}

function download(json: string, name: string): void {
  const url = URL.createObjectURL(new Blob([json], { type: 'application/json' }));
  const link = document.createElement('a');
  link.href = url;
  link.download = name;
  try { link.click(); } finally { setTimeout(() => URL.revokeObjectURL(url), 1000); }
}

// Writes go one at a time, in order: a slow checkpoint must never land on top of the finished run.
let writes: Promise<unknown> = Promise.resolve();
function inOrder<T>(write: () => Promise<T>): Promise<T> {
  const next = writes.then(write, write);
  writes = next.catch(() => undefined);
  return next;
}

async function saveNow(run: ScenarioRun): Promise<SavedRun> {
  const json = JSON.stringify(run);
  const name = runFileName(run);
  if (isTauri()) {
    const path = await invoke<string>('scenario_save_run', { name, json });
    return { where: 'disk', path };
  }
  if (await serverMode()) {
    const response = await fetch('/api/scenario-runs', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: json,
      signal: AbortSignal.timeout(30000),
    });
    if (!response.ok) throw new Error(`The server could not save the run (HTTP ${response.status}).`);
    const saved = await response.json() as { path?: string };
    return { where: 'server', path: saved.path };
  }
  download(json, name);
  return { where: 'download', path: name };
}

export function saveScenarioRun(run: ScenarioRun): Promise<SavedRun> {
  return inOrder(() => saveNow(run));
}

/** Save a run still being played, after each turn, so the orders survive the app closing mid-run.
 * Same file name as the finished run, which replaces it. Desktop and server only: a static host
 * would download a file every turn. */
export function checkpointScenarioRun(run: ScenarioRun): Promise<void> {
  return inOrder(async () => {
    if (isTauri() || await serverMode()) await saveNow(run);
  });
}
