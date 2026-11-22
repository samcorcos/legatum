/**
 * Legatum game uploads. The hosted web build posts each game's record (gzipped JSON, every seat under its
 * default name) to `PUT /v1/games/<game id>`, at least every five turns and when the game ends. The Worker
 * checks the shape and stores it in R2 as `games/<YYYY-MM>/<game id>.json.gz`, replacing the previous
 * checkpoint of the same game. A client address may start at most MAX_NEW_GAMES_PER_DAY new games per UTC day
 * (the `Quota` Durable Object). The only thing kept about a sender is a salted hash of that address (cut to
 * the /64 for IPv6), for the current UTC day, inside the Durable Object: no raw IP address, no headers, no logs.
 */

import { Quota, capFrom, clientIp } from "./quota";

export { Quota };

export interface Env {
  GAMES: R2Bucket;
  /** Counts new games per client address and UTC day. */
  QUOTA: DurableObjectNamespace<Quota>;
  /** New games one client address may store per UTC day (default 50). A bad value falls back to the default. */
  MAX_NEW_GAMES_PER_DAY?: string;
  ALLOWED_ORIGINS: string;
  /** Secret (`wrangler secret put READ_TOKEN`) for the private read API the analysis job on the developer's Mac uses. Unset = no read API. */
  READ_TOKEN?: string;
}

/** A 300-turn game is a few hundred KB gzipped; this bounds what a stranger can store per request. */
export const MAX_GZIP_BYTES = 4 * 1024 * 1024;
export const MAX_JSON_BYTES = 64 * 1024 * 1024;
export const GAME_ID_RE = /^web-\d{13}-[0-9a-f]{16}-[0-9a-f]{16}$/;
const DEFAULT_NAME_RE = /^(Player|AI) ([1-9])$/;
const OUTCOMES = ["in-progress", "vp", "draw", "elimination", "turn-limit", "stalled", "panic", "abandoned"];

interface Seat { player: number; name: string; faction: string; difficulty: string; human: boolean }
interface Metadata {
  schema_version: number; game_id: string; created_at: string; outcome: string; turn_count: number;
  engine_commit: string; seats: Seat[]; winner: number | null;
}

/** Origin patterns: exact, `*.host` for any subdomain, `http://host:*` for any port. */
export function originAllowed(origin: string | null, allowed: string): boolean {
  if (!origin) return false;
  return allowed.split(",").map((p) => p.trim()).filter(Boolean).some((pattern) => {
    if (pattern === origin) return true;
    if (pattern.startsWith("*.")) {
      try { const u = new URL(origin); return u.protocol === "https:" && u.hostname.endsWith(pattern.slice(1)); } catch { return false; }
    }
    if (pattern.endsWith(":*")) return new RegExp(`^${pattern.slice(0, -2).replace(/[.]/g, "\\.")}:\\d{1,5}$`).test(origin);
    return false;
  });
}

function cors(origin: string, resp: Response): Response {
  const out = new Response(resp.body, resp);
  out.headers.set("Access-Control-Allow-Origin", origin);
  out.headers.set("Vary", "Origin");
  out.headers.set("Cache-Control", "no-store");
  return out;
}

const json = (body: unknown, status = 200): Response =>
  new Response(JSON.stringify(body), { status, headers: { "content-type": "application/json" } });

async function gunzip(bytes: ArrayBuffer): Promise<string | null> {
  try {
    const stream = new Blob([bytes]).stream().pipeThrough(new DecompressionStream("gzip"));
    const reader = stream.getReader();
    const parts: Uint8Array[] = [];
    let size = 0;
    for (;;) {
      const { done, value } = await reader.read();
      if (done) break;
      size += value.byteLength;
      if (size > MAX_JSON_BYTES) { await reader.cancel(); return null; }
      parts.push(value);
    }
    const all = new Uint8Array(size);
    let at = 0;
    for (const p of parts) { all.set(p, at); at += p.byteLength; }
    return new TextDecoder("utf-8", { fatal: true, ignoreBOM: false }).decode(all);
  } catch {
    return null;
  }
}

/** The month in the storage key comes from this, so it must be a real ISO time. */
const CREATED_AT_RE = /^\d{4}-(0[1-9]|1[0-2])-(0[1-9]|[12]\d|3[01])T([01]\d|2[0-3]):[0-5]\d:[0-5]\d(\.\d{1,9})?(Z|[+-]\d{2}:\d{2})$/;
const ANSWER_TEXT_MAX = 2000;
const ANSWER_KEYS = new Set(["type", "game", "turn", "category", "question", "evidence", "answer", "build"]);

/** One coach answer line (app/src/why.ts WhyAnswerLine): known keys, bounded text. It is read by a model later. */
function answerOk(x: unknown): boolean {
  if (!x || typeof x !== "object" || Array.isArray(x)) return false;
  const o = x as Record<string, unknown>;
  if (!Object.keys(o).every((k) => ANSWER_KEYS.has(k))) return false;
  for (const k of ["type", "game", "category", "question", "answer", "build"]) {
    if (o[k] !== undefined && (typeof o[k] !== "string" || (o[k] as string).length > ANSWER_TEXT_MAX)) return false;
  }
  if (o.turn !== undefined && (!Number.isInteger(o.turn) || (o.turn as number) < 0 || (o.turn as number) > 10_000)) return false;
  return o.evidence === undefined || JSON.stringify(o.evidence).length <= 32_000;
}

/** The record's shape, and the rule that matters most: no seat carries a name a player typed. */
export function checkRecord(id: string, record: unknown): Metadata | string {
  if (!record || typeof record !== "object") return "not a game record";
  const r = record as { metadata?: Metadata; tape?: unknown; coach_answers?: unknown };
  const m = r.metadata;
  if (!m || typeof m !== "object" || !r.tape || typeof r.tape !== "object") return "not a game record";
  if (m.schema_version !== 1) return "unknown schema version";
  if (m.game_id !== id) return "game id does not match the address";
  if (!OUTCOMES.includes(m.outcome)) return "unknown outcome";
  if (!Number.isInteger(m.turn_count) || m.turn_count < 0 || m.turn_count > 10_000) return "bad turn count";
  if (typeof m.created_at !== "string" || !CREATED_AT_RE.test(m.created_at) || Number.isNaN(Date.parse(m.created_at))) return "bad created_at";
  if (!Array.isArray(m.seats) || m.seats.length < 1 || m.seats.length > 9) return "bad seats";
  for (const [i, s] of m.seats.entries()) {
    const name = DEFAULT_NAME_RE.exec(String(s?.name));
    if (!name || name[1] !== (s.human ? "Player" : "AI") || Number(name[2]) !== i + 1) return "seat names must be the defaults";
    // Stored as listing metadata and shown on the analysis Mac: plain words only.
    if (!/^[A-Za-z][A-Za-z ]{0,23}$/.test(String(s.faction)) || !/^[A-Za-z]{1,12}$/.test(String(s.difficulty))) return "bad faction or difficulty";
  }
  // The player's coach-question answers ride along as an optional list of objects.
  if (r.coach_answers !== undefined) {
    const a = r.coach_answers;
    if (!Array.isArray(a) || a.length > 2000 || !a.every(answerOk)) return "bad coach answers";
  }
  return m;
}

async function putGame(request: Request, env: Env, id: string): Promise<Response> {
  const declared = Number(request.headers.get("content-length") ?? "0");
  if (declared > MAX_GZIP_BYTES) return json({ error: "too large" }, 413);
  const bytes = await request.arrayBuffer();
  if (bytes.byteLength > MAX_GZIP_BYTES) return json({ error: "too large" }, 413);
  const text = await gunzip(bytes);
  if (text === null) return json({ error: "body must be gzipped JSON" }, 400);
  let record: unknown;
  try { record = JSON.parse(text); } catch { return json({ error: "body must be gzipped JSON" }, 400); }
  const m = checkRecord(id, record);
  if (typeof m === "string") return json({ error: m }, 422);
  const key = `games/${m.created_at.slice(0, 7)}/${id}.json.gz`;
  // A later checkpoint of a stored game replaces it and is never counted. A new game spends the address's daily quota.
  if (!(await env.GAMES.head(key))) {
    const taken = await env.QUOTA.getByName("quota").take(clientIp(request), capFrom(env.MAX_NEW_GAMES_PER_DAY));
    if (!taken.ok) {
      return new Response(JSON.stringify({ error: "too many new games today" }), {
        status: 429,
        headers: { "content-type": "application/json", "Retry-After": String(taken.retryAfter) },
      });
    }
  }
  await env.GAMES.put(key, bytes, {
    httpMetadata: { contentType: "application/json", contentEncoding: "gzip" },
    // For listing without downloading: who played what, how it ended.
    customMetadata: {
      outcome: m.outcome,
      turns: String(m.turn_count),
      engine: String(m.engine_commit ?? "").slice(0, 40),
      winner: m.winner == null ? "" : String(m.winner),
      seats: m.seats.map((s) => `${s.faction}${s.human ? "*" : ""}:${s.difficulty}`).join(" ").slice(0, 512),
    },
  });
  return json({ stored: key });
}

/** Constant-time comparison of the bearer token with the secret. */
export function tokenMatches(header: string | null, secret: string | undefined): boolean {
  if (!secret || secret.length < 32 || !header?.startsWith("Bearer ")) return false;
  const given = new TextEncoder().encode(header.slice(7));
  const want = new TextEncoder().encode(secret);
  let diff = given.length ^ want.length;
  for (let i = 0; i < want.length; i++) diff |= want[i]! ^ (given[i] ?? 0);
  return diff === 0;
}

const KEY_RE = /^games\/\d{4}-\d{2}\/web-\d{13}-[0-9a-f]{16}-[0-9a-f]{16}\.json\.gz$/;

/**
 * Private read API for the analysis job (tools/games): `GET /v1/read/games?cursor=` lists the stored games with
 * their metadata, `GET /v1/read/games/<key>` returns one record (still gzipped). Bearer token only, no browser
 * origin, so no page can read other players' games.
 */
async function readApi(request: Request, env: Env, path: string): Promise<Response> {
  if (!tokenMatches(request.headers.get("Authorization"), env.READ_TOKEN)) return json({ error: "not found" }, 404);
  if (request.method !== "GET") return json({ error: "method not allowed" }, 405);
  if (path === "/v1/read/games") {
    const cursor = new URL(request.url).searchParams.get("cursor") ?? undefined;
    const page = await env.GAMES.list({ prefix: "games/", cursor, limit: 1000, include: ["customMetadata"] });
    return json({
      games: page.objects.map((o) => ({ key: o.key, size: o.size, etag: o.etag, uploaded: o.uploaded.toISOString(), ...o.customMetadata })),
      cursor: page.truncated ? page.cursor : null,
    });
  }
  let key = "";
  try {
    key = decodeURIComponent(path.slice("/v1/read/games/".length));
  } catch {
    // A malformed escape such as "%E0%A4%A" is a bad key, not a server error.
  }
  if (!path.startsWith("/v1/read/games/") || !KEY_RE.test(key)) return json({ error: "not found" }, 404);
  const object = await env.GAMES.get(key);
  if (!object) return json({ error: "not found" }, 404);
  // Sent as stored (gzip bytes), not re-encoded, so the reader gunzips it itself.
  return new Response(object.body, { headers: { "content-type": "application/gzip", "etag": object.httpEtag, "cache-control": "no-store" } });
}

export default {
  async fetch(request: Request, env: Env): Promise<Response> {
    const url = new URL(request.url);
    if (url.pathname.startsWith("/v1/read/")) return readApi(request, env, url.pathname);
    const origin = request.headers.get("Origin");
    const match = /^\/v1\/games\/([^/]+)$/.exec(url.pathname);
    if (!originAllowed(origin, env.ALLOWED_ORIGINS)) return json({ error: "origin not allowed" }, 403);
    const allowedOrigin = origin as string;
    if (request.method === "OPTIONS") {
      return cors(allowedOrigin, new Response(null, {
        status: 204,
        headers: { "Access-Control-Allow-Methods": "PUT, OPTIONS", "Access-Control-Allow-Headers": "Content-Type", "Access-Control-Max-Age": "86400" },
      }));
    }
    if (!match || !GAME_ID_RE.test(match[1] ?? "")) return cors(allowedOrigin, json({ error: "not found" }, 404));
    if (request.method !== "PUT") return cors(allowedOrigin, json({ error: "method not allowed" }, 405));
    return cors(allowedOrigin, await putGame(request, env, match[1] as string));
  },
};
