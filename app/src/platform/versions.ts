// Which builds of the game the website serves, so a game always finishes on the version it started on.
//
// Each deploy publishes its build at the site root (the newest version) and at v/<id>/, and lists every build it
// still serves in versions.json, newest first (tools/release/keep-builds.mjs, docs/ops/releasing.md). This module
// reads that list to:
//   - open a save or autosave made with other rules in the build that made it (the save records its build),
//   - join a friend's online game on the build the host runs,
//   - tell a player whose tab is behind, between games only, that a newer version is ready.
// A build without the list (the dev server, a local build) behaves as before: one version, older saves refused.

/** One build the site serves. */
export interface SiteBuild {
  id: string;
  /** The title screen's build label; an online game lists its host's. */
  label: string;
  rules: number;
  protocol: number;
  deployedAt: string;
  /** When a newer build replaced it; null for the newest. */
  retiredAt: string | null;
}

/** This build's id (scripts/build-web.sh); null in a dev or test build. */
export const BUILD_ID: string | null = String(import.meta.env.VITE_LEGATUM_BUILD_ID ?? '').trim() || null;

/** What a page opened by another build should do once it loads (read once, then cleared from the address). */
export type Launch =
  | { kind: 'continue'; campaign: string }
  | { kind: 'open' }
  | { kind: 'join'; room: string }
  | { kind: 'menu'; menu: 'solo' | 'multi' };

/** The site root. A kept build's page at v/<id>/ has <base href="../../">, so the document base is the root either way. */
export function siteRoot(): string {
  return new URL('./', globalThis.document?.baseURI ?? globalThis.location?.href ?? 'http://localhost/').href;
}

const query = (params?: Record<string, string>) => {
  const q = new URLSearchParams(params ?? {}).toString();
  return q ? `?${q}` : '';
};

/** The newest version, at the site root. */
export const newestUrl = (params?: Record<string, string>): string => `${siteRoot()}${query(params)}`;

/** A kept build's own address. */
export const buildUrl = (id: string, params?: Record<string, string>): string => `${siteRoot()}v/${encodeURIComponent(id)}/${query(params)}`;

const isBuild = (b: unknown): b is SiteBuild => {
  const x = b as Partial<SiteBuild> | null;
  return !!x && typeof x.id === 'string' && /^[a-z0-9][a-z0-9-]*$/.test(x.id) && Number.isInteger(x.rules);
};

/** The builds in a versions.json body, newest first; null when it is not one. */
export function parseVersions(body: unknown): SiteBuild[] | null {
  const builds = (body as { builds?: unknown } | null)?.builds;
  if (!Array.isArray(builds)) return null;
  const out = builds.filter(isBuild).map((b) => ({
    id: b.id,
    label: typeof b.label === 'string' ? b.label : '',
    rules: b.rules,
    protocol: Number.isInteger(b.protocol) ? b.protocol : 1,
    deployedAt: typeof b.deployedAt === 'string' ? b.deployedAt : '',
    retiredAt: typeof b.retiredAt === 'string' ? b.retiredAt : null,
  }));
  return out.length ? out : null;
}

type FetchLike = (url: string, init?: { cache?: 'no-store' }) => Promise<{ ok: boolean; json(): Promise<unknown> }>;

/** The site's builds, or null when this build has none to read (dev, a local build) or the list did not load. */
export async function fetchVersions(fetchImpl: FetchLike = fetch): Promise<SiteBuild[] | null> {
  if (!BUILD_ID && fetchImpl === fetch) return null;
  try {
    const res = await fetchImpl(`${siteRoot()}versions.json`, { cache: 'no-store' });
    return res.ok ? parseVersions(await res.json()) : null;
  } catch {
    return null;
  }
}

/** True when the site's newest build is not this one: new games should start there. */
export function isBehind(builds: SiteBuild[] | null, mine: string | null = BUILD_ID): boolean {
  return !!builds && !!mine && builds[0].id !== mine;
}

/**
 * The build that can open a save made with `rules`: the build that made it if the site still has it, else the newest
 * build with those rules. Null when none can (it is older than the site keeps), or when this build is the one.
 */
export function buildForSave(builds: SiteBuild[] | null, rules: number | null, madeBy?: string, mine: string | null = BUILD_ID): SiteBuild | null {
  if (!builds || rules === null) return null;
  const able = builds.filter((b) => b.rules === rules);
  const pick = able.find((b) => b.id === madeBy) ?? able[0] ?? null;
  return pick && pick.id !== mine ? pick : null;
}

/** The build an online game's host runs, when it is another one the site serves. */
export function buildForRoom(builds: SiteBuild[] | null, room: { build: string; rules_version: number; protocol: number }, mine: string | null = BUILD_ID): SiteBuild | null {
  if (!builds || !room.build) return null;
  const pick = builds.find((b) => b.label === room.build && b.rules === room.rules_version && b.protocol === room.protocol) ?? null;
  return pick && pick.id !== mine ? pick : null;
}

// ---------- Handing a save to another build ----------
// A save chosen with Load Game is handed to the build that can open it through the browser's own storage (the
// same site, so the same storage): a small database of its own, with this tab's session storage as the fallback.

const HANDOFF_DB = 'legatum-handoff-v1';
const HANDOFF_STORE = 'handoff';
const HANDOFF_KEY = 'legatum.handoff.v1';
/** A handed-off save older than this is stale (the other page never opened) and is ignored. */
const HANDOFF_MS = 10 * 60_000;

function handoffDb(): Promise<IDBDatabase> {
  return new Promise((resolve, reject) => {
    try {
      if (typeof indexedDB === 'undefined') throw new Error('no database');
      const request = indexedDB.open(HANDOFF_DB, 1);
      request.onupgradeneeded = () => { request.result.createObjectStore(HANDOFF_STORE); };
      request.onsuccess = () => resolve(request.result);
      request.onerror = () => reject(request.error ?? new Error('database refused'));
      request.onblocked = () => reject(new Error('database blocked'));
    } catch (e) {
      reject(e);
    }
  });
}

async function handoffTx<T>(mode: IDBTransactionMode, work: (store: IDBObjectStore) => IDBRequest<T>): Promise<T> {
  const db = await handoffDb();
  try {
    return await new Promise<T>((resolve, reject) => {
      const tx = db.transaction(HANDOFF_STORE, mode);
      const request = work(tx.objectStore(HANDOFF_STORE));
      tx.oncomplete = () => resolve(request.result);
      tx.onerror = () => reject(tx.error ?? new Error('handoff failed'));
      tx.onabort = () => reject(tx.error ?? new Error('handoff aborted'));
    });
  } finally {
    db.close();
  }
}

/** Keep a save's text for the next page this tab opens. False when the browser keeps nothing. */
export async function handOffSave(text: string): Promise<boolean> {
  const entry = { text, at: Date.now() };
  try {
    await handoffTx('readwrite', (s) => s.put(entry, 'save'));
    return true;
  } catch {
    try {
      sessionStorage.setItem(HANDOFF_KEY, JSON.stringify(entry));
      return true;
    } catch {
      return false;
    }
  }
}

/** The save another build handed to this page, once: it is removed as it is read. */
export async function takeHandedOffSave(now = Date.now()): Promise<string | null> {
  const fresh = (e: unknown): string | null => {
    const x = e as { text?: unknown; at?: unknown } | null;
    return x && typeof x.text === 'string' && typeof x.at === 'number' && now - x.at < HANDOFF_MS ? x.text : null;
  };
  let found: string | null = null;
  try {
    found = fresh(await handoffTx('readonly', (s) => s.get('save')));
    await handoffTx('readwrite', (s) => s.delete('save'));
  } catch {
    // no database: the session storage fallback below
  }
  try {
    const kept = sessionStorage.getItem(HANDOFF_KEY);
    if (kept) {
      sessionStorage.removeItem(HANDOFF_KEY);
      found ??= fresh(JSON.parse(kept));
    }
  } catch {
    // nothing kept
  }
  return found;
}

// ---------- Opening another build ----------

/** Read what this page was opened to do (from its address), and take it out of the address so a reload does not repeat it. */
export function readLaunch(): Launch | null {
  const loc = globalThis.location;
  if (!loc) return null;
  const params = new URLSearchParams(loc.search);
  let launch: Launch | null = null;
  const campaign = params.get('continue');
  const room = params.get('join');
  const menu = params.get('menu');
  if (campaign) launch = { kind: 'continue', campaign };
  else if (params.get('open') === 'save') launch = { kind: 'open' };
  else if (room) launch = { kind: 'join', room };
  else if (menu === 'solo' || menu === 'multi') launch = { kind: 'menu', menu };
  if (!launch) return null;
  for (const key of ['continue', 'open', 'join', 'menu']) params.delete(key);
  const rest = params.toString();
  try {
    // An absolute path: a kept build's <base> would otherwise send the address to the root.
    globalThis.history?.replaceState(globalThis.history.state, '', `${loc.pathname}${rest ? `?${rest}` : ''}${loc.hash}`);
  } catch {
    // the address keeps the parameter; harmless
  }
  return launch;
}

let launch: Launch | null | undefined;
/** The launch this page was opened with, read once per page. */
export function pageLaunch(): Launch | null {
  if (launch === undefined) launch = readLaunch();
  return launch;
}
/** Tests reset it. */
export function resetPageLaunch(next?: Launch | null) {
  launch = next;
}

/** Leave this page for another address (tests replace it). */
export let navigate = (url: string): void => { globalThis.location?.assign(url); };
export function setNavigate(next?: (url: string) => void) {
  navigate = next ?? ((url: string) => { globalThis.location?.assign(url); });
}
