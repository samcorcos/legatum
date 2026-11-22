import { isTauri, invoke } from '@tauri-apps/api/core';
import { rememberedSeat } from '../remote';
import { HOSTED } from './hosted';

interface History {
  metadata: { game_id: string; turn_count: number; finished_at: string | null; outcome: string };
}
interface Pending { id: string; revision: string; bytes: ArrayBuffer; gzip: boolean }
const DB = 'legatum-game-archive-v1';
let tail = Promise.resolve();

/** Capture the last turns when another game replaces an unfinished local session. */
export function abandonGameHistory(json: string): Promise<void> {
  const recording = JSON.parse(json) as History;
  if (recording.metadata.finished_at !== null) return recordGameHistory(json);
  recording.metadata.finished_at = new Date().toISOString();
  recording.metadata.outcome = 'abandoned';
  return recordGameHistory(JSON.stringify(recording));
}

/** Lockdown Mode and some in-app browsers expose no IndexedDB: histories then go straight to the host. */
const hasDatabase = () => typeof indexedDB !== 'undefined';

async function database(): Promise<IDBDatabase> {
  if (!hasDatabase()) throw new Error('this browser keeps no local storage for game history');
  return new Promise((resolve, reject) => {
    const request = indexedDB.open(DB, 1);
    request.onupgradeneeded = () => request.result.createObjectStore('games', { keyPath: 'id' });
    request.onsuccess = () => resolve(request.result);
    request.onerror = () => reject(request.error);
  });
}

async function transaction<T>(mode: IDBTransactionMode, run: (store: IDBObjectStore) => IDBRequest<T>): Promise<T> {
  const db = await database();
  return new Promise((resolve, reject) => {
    const tx = db.transaction('games', mode);
    const request = run(tx.objectStore('games'));
    tx.oncomplete = () => { db.close(); resolve(request.result); };
    tx.onerror = tx.onabort = () => { db.close(); reject(tx.error ?? request.error); };
  });
}

async function deliver(json: string): Promise<boolean> {
  if (isTauri()) {
    // Native compression; plugin-fs writes only inside the configured archive scope.
    // Native finalization serializes all app/server/sim writers with the same index lock.
    const staged = await invoke<{ ticket: string; path: string; bytes: number[] } | null>('archive_stage', { json });
    // The native archive is off (its folder could not be opened): keep the history on this device.
    if (!staged) return false;
    const { writeFile } = await import('@tauri-apps/plugin-fs');
    await writeFile(staged.path, new Uint8Array(staged.bytes));
    await invoke('archive_commit', { ticket: staged.ticket });
    return true;
  }
  // The hosted web build has no archive server: histories stay on this device (Export game history).
  if (HOSTED) return false;
  const status = await fetch('/api/status', { signal: AbortSignal.timeout(5000), cache: 'no-store' });
  if (status.status === 404) return false;
  // Static hosts often return index.html for unknown paths.
  if (!status.ok) throw new Error(`Archive server returned HTTP ${status.status}`);
  if (!status.headers.get('content-type')?.includes('application/json')) return false;
  const host = await status.json() as { mode?: string };
  if (host.mode !== 'server') return false;
  const token = rememberedSeat()?.token;
  const response = await fetch('/api/archive', {
    method: 'POST', headers: { 'Content-Type': 'application/json', ...(token ? { Authorization: `Bearer ${token}` } : {}) },
    body: json, signal: AbortSignal.timeout(30000),
  });
  if (!response.ok) throw new Error(`Game history is kept on this device, but the archive upload failed (HTTP ${response.status}). Use Export game history, or try again when connected.`);
  return true;
}

async function flush(): Promise<void> {
  if (!hasDatabase()) return;
  const pending = await transaction<Pending[]>('readonly', (s) => s.getAll());
  const errors: unknown[] = [];
  for (const item of pending) {
    try {
      if (!await deliver(await unpack(item))) break;
      await acknowledge(item);
    } catch (error) { errors.push(error); }
  }
  if (errors.length) throw errors[0];
}

/** Another tab can checkpoint this game while its earlier upload is in flight. */
async function acknowledge(item: Pending): Promise<void> {
  const db = await database();
  return new Promise((resolve, reject) => {
    const tx = db.transaction('games', 'readwrite');
    const store = tx.objectStore('games');
    const request = store.get(item.id);
    request.onsuccess = () => {
      if ((request.result as Pending | undefined)?.revision === item.revision) store.delete(item.id);
    };
    tx.oncomplete = () => { db.close(); resolve(); };
    tx.onerror = tx.onabort = () => { db.close(); reject(tx.error ?? request.error); };
  });
}

async function pack(id: string, json: string): Promise<Pending> {
  const blob = new Blob([json], { type: 'application/json' });
  const gzip = typeof CompressionStream !== 'undefined';
  const bytes = gzip ? await new Response(blob.stream().pipeThrough(new CompressionStream('gzip'))).arrayBuffer() : await blob.arrayBuffer();
  const revision = globalThis.crypto?.randomUUID?.() ?? `${Date.now()}-${Math.random()}`;
  return { id, revision, bytes, gzip };
}

async function unpack(item: Pending): Promise<string> {
  const blob = new Blob([item.bytes]);
  return item.gzip ? new Response(blob.stream().pipeThrough(new DecompressionStream('gzip'))).text() : blob.text();
}

/** Capture JSON before queueing, retain failures across reloads, retry in turn order. */
export function recordGameHistory(json: string): Promise<void> {
  const recording = JSON.parse(json) as History;
  const operation = tail.then(async () => {
    const pending = await pack(recording.metadata.game_id, json);
    try {
      await transaction('readwrite', (s) => s.put(pending));
    } catch (error) {
      // A denied/full browser database must not prevent a functioning native/server archive.
      if (await deliver(json)) return;
      throw new Error(`Could not keep game history: ${String(error)}. Export game history before leaving.`);
    }
    await flush();
  });
  tail = operation.catch(() => {});
  return operation;
}

export function retryGameHistory(): Promise<void> {
  const operation = tail.then(flush);
  tail = operation.catch(() => {});
  return operation;
}

/** Static-host fallback: gzip when available, plain portable JSON on older browsers. */
export async function exportGameHistory(json: string): Promise<void> {
  const recording = JSON.parse(json) as History;
  let blob = new Blob([json], { type: 'application/json' });
  let extension = 'json';
  if (typeof CompressionStream !== 'undefined') {
    blob = await new Response(blob.stream().pipeThrough(new CompressionStream('gzip'))).blob();
    extension = 'json.gz';
  }
  const url = URL.createObjectURL(blob);
  const link = document.createElement('a');
  link.href = url;
  link.download = `${recording.metadata.game_id}.tape.${extension}`;
  try { link.click(); } finally { setTimeout(() => URL.revokeObjectURL(url), 1000); }
}

/** Export retained histories, including games played before the current session. */
export async function exportRetainedHistories(current?: string): Promise<void> {
  await tail;
  const pending = await transaction<Pending[]>('readonly', (s) => s.getAll()).catch(() => []);
  const games = new Map(await Promise.all(pending.map(async (p) => [p.id, await unpack(p)] as const)));
  if (current) games.set((JSON.parse(current) as History).metadata.game_id, current);
  for (const json of games.values()) await exportGameHistory(json);
}
