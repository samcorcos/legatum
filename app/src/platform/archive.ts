
interface History {
  metadata: { game_id: string; turn_count: number; finished_at: string | null; outcome: string };
}
interface Stored { id: string; bytes: ArrayBuffer; gzip: boolean }
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

/** Lockdown Mode and some in-app browsers expose no IndexedDB. */
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

async function pack(id: string, json: string): Promise<Stored> {
  const blob = new Blob([json], { type: 'application/json' });
  const gzip = typeof CompressionStream !== 'undefined';
  const bytes = gzip ? await new Response(blob.stream().pipeThrough(new CompressionStream('gzip'))).arrayBuffer() : await blob.arrayBuffer();
  return { id, bytes, gzip };
}

async function unpack(item: Stored): Promise<string> {
  const blob = new Blob([item.bytes]);
  return item.gzip ? new Response(blob.stream().pipeThrough(new DecompressionStream('gzip'))).text() : blob.text();
}

/** Keep a game's history in this browser's database, in turn order. A newer checkpoint replaces the older one. */
export function recordGameHistory(json: string): Promise<void> {
  const recording = JSON.parse(json) as History;
  const operation = tail.then(async () => {
    const stored = await pack(recording.metadata.game_id, json);
    try {
      await transaction('readwrite', (s) => s.put(stored));
    } catch (error) {
      throw new Error(`Could not keep game history: ${String(error)}. Export game history before leaving.`);
    }
  });
  tail = operation.catch(() => {});
  return operation;
}

/** Download one history: gzip when available, plain portable JSON on older browsers. */
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
  const stored = await transaction<Stored[]>('readonly', (s) => s.getAll()).catch(() => []);
  const games = new Map(await Promise.all(stored.map(async (p) => [p.id, await unpack(p)] as const)));
  if (current) games.set((JSON.parse(current) as History).metadata.game_id, current);
  for (const json of games.values()) await exportGameHistory(json);
}
