// Where the game's autosaves live: the browser's own database, a few turns deep, with the old single
// localStorage slot as the fallback. Nothing here knows about the game: a record is text plus the little the
// title screen needs to show it. ui/CrashScreen.tsx keeps a tiny duplicate of the read side (it must not
// use anything from the game), so keep AUTOSAVE_DB, the store names and the record shape in step with it.
//
// - Rolling: each campaign (one game) keeps its newest AUTOSAVE_SLOTS turns, one record per turn. Writing a turn drops
//   that game's records for later turns: they belong to a line of play the player went back from.
// - Compact: the newest PLAIN_SLOTS records keep their text (Continue, the crash screen and older builds read only
//   those); older ones are gzipped into `gz` with no `text`, about a tenth of the size. "Load an earlier turn" reads them.
// - Atomic per record: the new record is written first, the old ones pruned after, so a failed or cut-off
//   write never costs the saves that were already there. Another game's records go only after the new
//   game's first record is in.
// - No database (private mode, blocked): the old single localStorage slot, as before.
// - Saves written by older builds sit in that localStorage slot. They are copied into the database the
//   first time they are read, and the old key is removed only after a newer database write has succeeded.

export interface AutosaveRecord {
  /** `${campaign}:${turn}`: writing the same turn again replaces that slot. */
  id: string;
  /** One game's records share a campaign. */
  campaign: string;
  turn: number;
  /** ISO time of the save. */
  savedAt: string;
  /** What the title screen calls the game, e.g. the players' names. */
  label: string;
  /** The human players' peoples, e.g. "Athens": what the replace question calls the game. Absent in older saves. */
  faction?: string;
  /** The save file's text, exactly as Save to file writes it. */
  text: string;
}
export type AutosaveDraft = Omit<AutosaveRecord, 'id'>;
/** An older turn as stored: its text gzipped into `gz`. Older builds and the crash screen skip it (no `text`). */
type CompactRecord = Omit<AutosaveRecord, 'text'> & { gz: ArrayBuffer };
/** One kept turn of a game, for "Load an earlier turn": what the list shows, without the save's text. */
export type AutosaveTurn = Omit<AutosaveRecord, 'text'>;
export type SaveBackend = 'database' | 'local';

export const AUTOSAVE_SLOTS = 10;
/** The newest records of a game that keep their text uncompressed. */
export const PLAIN_SLOTS = 2;
/** The single slot older builds wrote, and the fallback when there is no database. */
export const LEGACY_AUTOSAVE_KEY = 'legatum.autosave.v1';
/** Slots the player stepped back from ("Load the previous turn"): ids, hidden until a newer save lands. */
export const SKIPPED_SLOTS_KEY = 'legatum.autosave.skipped.v1';
const ORDERS_FALLBACK_KEY = 'legatum.autosave.orders.v1';
export const AUTOSAVE_DB = 'legatum-autosave-v1';
const SAVES = 'saves';
const ORDERS = 'orders';
const LEGACY_CAMPAIGN = 'legacy';

export const autosaveId = (campaign: string, turn: number) => `${campaign}:${turn}`;

export function newCampaignId(): string {
  try {
    return crypto.randomUUID();
  } catch {
    return `${Date.now().toString(36)}-${Math.random().toString(36).slice(2, 10)}`;
  }
}

const isRecord = (r: unknown): r is AutosaveRecord => {
  const x = r as Partial<AutosaveRecord> | null;
  return !!x && typeof x.id === 'string' && typeof x.campaign === 'string' && typeof x.text === 'string' &&
    typeof x.savedAt === 'string' && typeof x.label === 'string' && Number.isInteger(x.turn);
};

const isCompact = (r: unknown): r is CompactRecord => {
  const x = r as Partial<CompactRecord> | null;
  return !!x && typeof x.id === 'string' && typeof x.campaign === 'string' && x.gz instanceof ArrayBuffer &&
    typeof x.savedAt === 'string' && typeof x.label === 'string' && Number.isInteger(x.turn);
};

const when = (r: { savedAt: string }) => { const t = Date.parse(r.savedAt); return Number.isFinite(t) ? t : 0; };
const newestFirst = (a: AutosaveTurn, b: AutosaveTurn) => when(b) - when(a) || b.turn - a.turn;

// ---------- The database ----------

function openDatabase(): Promise<IDBDatabase> {
  return new Promise((resolve, reject) => {
    try {
      if (typeof indexedDB === 'undefined') throw new Error('no database');
      // Stays at version 1: every build the site keeps (platform/versions.ts) opens this database, and an older one
      // opening it below a version a newer build set would fail.
      const request = indexedDB.open(AUTOSAVE_DB, 1);
      request.onupgradeneeded = () => {
        const db = request.result;
        if (!db.objectStoreNames.contains(SAVES)) db.createObjectStore(SAVES, { keyPath: 'id' });
        if (!db.objectStoreNames.contains(ORDERS)) db.createObjectStore(ORDERS, { keyPath: 'key' });
      };
      request.onsuccess = () => resolve(request.result);
      request.onerror = () => reject(request.error ?? new Error('database refused'));
      request.onblocked = () => reject(new Error('database blocked'));
    } catch (e) {
      reject(e);
    }
  });
}

/** One transaction over both stores. `work` queues its requests and returns what to hand back on completion. */
async function transact<T>(mode: IDBTransactionMode, work: (saves: IDBObjectStore, orders: IDBObjectStore) => () => T): Promise<T> {
  const db = await openDatabase();
  try {
    return await new Promise<T>((resolve, reject) => {
      const tx = db.transaction([SAVES, ORDERS], mode);
      const result = work(tx.objectStore(SAVES), tx.objectStore(ORDERS));
      tx.oncomplete = () => resolve(result());
      tx.onerror = () => reject(tx.error ?? new Error('database write failed'));
      tx.onabort = () => reject(tx.error ?? new Error('database write aborted'));
    });
  } finally {
    db.close();
  }
}

const readAll = () => transact('readonly', (saves) => {
  const request = saves.getAll();
  return () => (request.result as unknown[]).filter(isRecord);
});

const putRecord = (record: AutosaveRecord) => transact('readwrite', (saves) => { saves.put(record); return () => undefined; });

/** Every record, compacted ones included, without their text: what pruning and the turn list need. */
const readAllTurns = () => transact('readonly', (saves) => {
  const request = saves.getAll();
  return () => (request.result as unknown[]).filter((r): r is AutosaveRecord | CompactRecord => isRecord(r) || isCompact(r))
    .map((r): AutosaveTurn & { plain: boolean } => {
      const { id, campaign, turn, savedAt, label, faction } = r;
      return { id, campaign, turn, savedAt, label, ...(faction === undefined ? {} : { faction }), plain: isRecord(r) };
    });
});

/**
 * Keeps this campaign's newest AUTOSAVE_SLOTS records, drops every other campaign's and any of this campaign's for a
 * turn after the one just written (a line of play the player went back from), and gzips the ones past PLAIN_SLOTS.
 */
async function prune(kept: AutosaveRecord) {
  const all = await readAllTurns();
  const mine = all.filter((r) => r.campaign === kept.campaign && r.turn <= kept.turn).sort((a, b) => b.turn - a.turn || newestFirst(a, b));
  const doomed = [
    ...all.filter((r) => r.campaign !== kept.campaign || r.turn > kept.turn),
    ...mine.slice(AUTOSAVE_SLOTS),
  ];
  if (doomed.length) await transact('readwrite', (saves) => { for (const r of doomed) saves.delete(r.id); return () => undefined; });
  for (const r of mine.slice(PLAIN_SLOTS, AUTOSAVE_SLOTS)) if (r.plain) await compact(r.id);
  // After going back, the turn before may be a gzipped one: unpack it, so Continue and the crash screen can fall back to it.
  for (const r of mine.slice(1, PLAIN_SLOTS)) if (!r.plain) await expand(r.id);
}

async function gzip(text: string): Promise<ArrayBuffer | null> {
  if (typeof CompressionStream === 'undefined') return null;
  return new Response(new Blob([text]).stream().pipeThrough(new CompressionStream('gzip'))).arrayBuffer();
}

async function gunzip(bytes: ArrayBuffer): Promise<string> {
  return new Response(new Blob([bytes]).stream().pipeThrough(new DecompressionStream('gzip'))).text();
}

/** Gzips one stored record in place. A browser that cannot compress keeps it as text. */
async function compact(id: string) {
  const record = await transact('readonly', (saves) => { const request = saves.get(id); return () => request.result as unknown; });
  if (!isRecord(record)) return;
  const gz = await gzip(record.text);
  if (!gz) return;
  const { text: _text, ...rest } = record;
  await transact('readwrite', (saves) => { saves.put({ ...rest, gz } satisfies CompactRecord); return () => undefined; });
}

/** Unpacks one gzipped record in place. */
async function expand(id: string) {
  const record = await transact('readonly', (saves) => { const request = saves.get(id); return () => request.result as unknown; });
  if (!isCompact(record)) return;
  const { gz, ...rest } = record;
  const text = await gunzip(gz);
  await transact('readwrite', (saves) => { saves.put({ ...rest, text } satisfies AutosaveRecord); return () => undefined; });
}

// ---------- The old single slot ----------

function readLegacy(): AutosaveRecord | null {
  try {
    const text = localStorage.getItem(LEGACY_AUTOSAVE_KEY);
    if (!text) return null;
    const s = JSON.parse(text) as { turn?: unknown; savedAt?: unknown; names?: unknown };
    if (!Number.isInteger(s.turn) || typeof s.savedAt !== 'string') return null;
    const names = Array.isArray(s.names) ? s.names.filter((n): n is string => typeof n === 'string') : [];
    return { id: autosaveId(LEGACY_CAMPAIGN, s.turn as number), campaign: LEGACY_CAMPAIGN, turn: s.turn as number, savedAt: s.savedAt, label: names.join(' & '), text };
  } catch {
    return null;
  }
}

function readSkipped(): string[] {
  try {
    const ids = JSON.parse(localStorage.getItem(SKIPPED_SLOTS_KEY) ?? '[]') as unknown;
    return Array.isArray(ids) ? ids.filter((id): id is string => typeof id === 'string') : [];
  } catch {
    return [];
  }
}

/** "Load the previous turn": hide this slot so the one before it is the newest. It stays stored. */
export function skipAutosave(id: string) {
  try { localStorage.setItem(SKIPPED_SLOTS_KEY, JSON.stringify([...new Set([...readSkipped(), id])])); } catch { /* blocked */ }
}

const forgetLocal = (key: string) => { try { localStorage.removeItem(key); } catch { /* blocked */ } };

// ---------- Reading ----------

/** Every readable autosave, newest first. Never throws: no storage at all is an empty list. */
export async function listAutosaves(): Promise<AutosaveRecord[]> {
  let records: AutosaveRecord[] = [];
  let database = true;
  try { records = await readAll(); } catch { database = false; }
  const legacy = readLegacy();
  if (legacy && !records.some((r) => r.savedAt === legacy.savedAt)) {
    records.push(legacy);
    if (database) await putRecord(legacy).catch(() => undefined); // The copy; the old key stays until a newer write lands.
  }
  const skipped = readSkipped();
  return records.filter((r) => !skipped.includes(r.id)).sort(newestFirst);
}

export const hasAutosave = async () => (await listAutosaves()).length > 0;

/** Every turn kept for one game, compacted ones included, newest first, without their text. Never throws. */
export async function listCampaignTurns(campaign: string): Promise<AutosaveTurn[]> {
  try {
    const skipped = readSkipped();
    return (await readAllTurns()).filter((r) => r.campaign === campaign && !skipped.includes(r.id))
      .map(({ plain: _plain, ...turn }) => turn).sort((a, b) => b.turn - a.turn || newestFirst(a, b));
  } catch {
    return [];
  }
}

/** One kept turn's save text, unpacking a compacted one; null when it is gone or cannot be read. */
export async function readAutosaveText(id: string): Promise<string | null> {
  try {
    const record = await transact('readonly', (saves) => { const request = saves.get(id); return () => request.result as unknown; });
    if (isRecord(record)) return record.text;
    if (isCompact(record)) return await gunzip(record.gz);
  } catch { /* gone, or no database */ }
  return (await listAutosaves()).find((r) => r.id === id)?.text ?? null;
}

// ---------- Writing ----------

let tail: Promise<unknown> = Promise.resolve();
let askedFor: string | null = null;

/** Asks the browser not to evict our data; once per game, after its first successful save, never waited for. */
function askToKeep(campaign: string) {
  if (askedFor === campaign) return;
  askedFor = campaign;
  try {
    const persist = navigator.storage?.persist?.();
    if (persist) void Promise.resolve(persist).catch(() => undefined);
  } catch { /* nothing to ask */ }
}

async function write(draft: AutosaveDraft): Promise<SaveBackend> {
  const record: AutosaveRecord = { ...draft, id: autosaveId(draft.campaign, draft.turn) };
  try {
    await putRecord(record);
  } catch (error) {
    // No database, or no room in it: the old single slot, so nothing regresses. If that fails too, say so.
    try {
      localStorage.setItem(LEGACY_AUTOSAVE_KEY, draft.text);
      return 'local';
    } catch {
      throw error;
    }
  }
  await prune(record).catch(() => undefined);
  forgetLocal(LEGACY_AUTOSAVE_KEY);
  forgetLocal(SKIPPED_SLOTS_KEY);
  askToKeep(draft.campaign);
  return 'database';
}

/** Saves one turn. Writes run one at a time, in order. Rejects when nothing could be written. */
export function writeAutosave(draft: AutosaveDraft): Promise<SaveBackend> {
  const run = tail.then(() => write(draft));
  tail = run.catch(() => undefined);
  return run;
}

/** Resolves when every write queued so far has finished (for tests and for leaving a game). */
export const autosavesSettled = () => tail.then(() => undefined);

// ---------- Orders in progress ----------

interface PendingOrders { key: 'current'; campaign: string; turn: number; text: string }

let ordersTail: Promise<unknown> = Promise.resolve();

async function writeOrders(pending: { campaign: string; turn: number; text: string | null }) {
  try {
    await transact('readwrite', (_saves, orders) => {
      if (pending.text === null) orders.delete('current');
      else orders.put({ key: 'current', campaign: pending.campaign, turn: pending.turn, text: pending.text } satisfies PendingOrders);
      return () => undefined;
    });
    forgetLocal(ORDERS_FALLBACK_KEY);
  } catch (error) {
    try {
      if (pending.text === null) localStorage.removeItem(ORDERS_FALLBACK_KEY);
      else localStorage.setItem(ORDERS_FALLBACK_KEY, JSON.stringify({ campaign: pending.campaign, turn: pending.turn, text: pending.text }));
    } catch {
      throw error;
    }
  }
}

/** Keeps this turn's orders beside its autosave (null text: nothing pending). Rejects when they could not be kept. */
export function writePendingOrders(pending: { campaign: string; turn: number; text: string | null }): Promise<void> {
  const run = ordersTail.then(() => writeOrders(pending));
  ordersTail = run.catch(() => undefined);
  return run;
}

/** The orders kept for exactly this campaign and turn, or null. */
export async function readPendingOrders(campaign: string, turn: number): Promise<string | null> {
  const matches = (p: Partial<PendingOrders> | null | undefined) => !!p && p.campaign === campaign && p.turn === turn && typeof p.text === 'string' ? p.text : null;
  try {
    const kept = await transact('readonly', (_saves, orders) => { const request = orders.get('current'); return () => request.result as PendingOrders | undefined; });
    const found = matches(kept);
    if (found !== null) return found;
  } catch { /* fall through to the old slot */ }
  try {
    return matches(JSON.parse(localStorage.getItem(ORDERS_FALLBACK_KEY) ?? 'null') as Partial<PendingOrders> | null);
  } catch {
    return null;
  }
}

/** Test seam: forget the queue and the "asked the browser" memory. */
export function resetAutosaveStoreForTests() {
  tail = Promise.resolve();
  ordersTail = Promise.resolve();
  askedFor = null;
}
