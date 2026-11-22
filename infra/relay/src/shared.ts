// Shared types, limits and helpers. The relay never looks at game messages, only at this.

export const MAX_GUESTS = 8;
export const MAX_GUEST_MSG = 64 * 1024; // guest -> host, bytes
export const MAX_HOST_MSG = 1024 * 1024; // host -> guest, bytes
export const GRACE_MS = 120_000; // room lives this long after its host tunnel drops
export const MAX_NAME_INPUT = 200; // UTF-16 units accepted in a POST /rooms name, before cleaning
export const MAX_NAME_CHARS = 40; // visible characters kept in a lobby name
export const MAX_BUILD = 40;
export const MAX_STATUS_BYTES = 64 * 1024; // the host's `status` frame, which anyone can read back from `/api/status`

// Tunables. Each can be overridden by a Worker var of the same name (wrangler.jsonc "vars").
export const DEFAULT_LIMITS = {
  MAX_ROOMS: 64, // rooms on the whole relay, hosted or waiting for their host
  MAX_ROOMS_PER_IP: 2, // live rooms one client address may hold
  ROOM_CREATES_PER_10_MIN: 6, // room creations one client address may make in any 10 minutes
  MAX_GUESTS_PER_IP: 4, // guest sockets one client address may hold in one room
};
export type Limits = typeof DEFAULT_LIMITS;
export const CREATE_WINDOW_MS = 10 * 60_000;

/**
 * Per-socket rate caps in the Room: token buckets that start full (`Burst`) and refill at `PerSec`.
 * Sized from the protocol: a guest sends one small text per order, chat line or 20 s ping; the host sends a
 * `view` per guest per turn, chunked at 256 KiB (a large late-game view is a few MiB, so 8 guests can be
 * 100-200 frames and about 32 MiB in one burst), plus a snapshot to each guest that reconnects. The bursts
 * cover many such turns back to back; the refill rates are low because incoming WebSocket messages are
 * what the relay is billed for (20 messages = 1 Durable Object request), so they bound what a flooding
 * socket can cost.
 */
export interface Rate {
  msgBurst: number;
  msgPerSec: number;
  byteBurst: number;
  bytePerSec: number;
}
export const GUEST_RATE: Rate = { msgBurst: 300, msgPerSec: 5, byteBurst: 4 * 1024 * 1024, bytePerSec: 128 * 1024 };
export const HOST_RATE: Rate = { msgBurst: 4000, msgPerSec: 20, byteBurst: 64 * 1024 * 1024, bytePerSec: 4 * 1024 * 1024 };

/** Close code for a socket that broke a rate cap (4000-4999 is the application range; 4029 echoes HTTP 429). */
export const CLOSE_RATE = 4029;
export const REASON_TOO_MANY_MESSAGES = "too many messages";
export const REASON_TOO_MUCH_DATA = "too much data";
/** Sent with 1013 (try again later) to a guest over the per-address socket cap of a room. */
export const REASON_TOO_MANY_FROM_IP = "too many connections from one address";

/** A host that has sent a ping frame is held to it: silent this long and its room leaves the list. */
export const HEARTBEAT_MS = 60_000;
/** The Room tells the Lobby it heard from the host at most this often. */
export const HEARD_REPORT_MS = 25_000;

/** Player-visible errors from POST /rooms: short plain sentences the lobby UI can show as they are. */
export const MSG_LOBBY_CLOSED = "Hosting new games is paused. Try again later.";
export const MSG_LOBBY_FULL = "This lobby is full. Try again in a few minutes.";
export const MSG_TOO_FAST = "You are opening games too quickly. Try again in a few minutes.";
export const msgTooManyRooms = (n: number): string =>
  n === 1
    ? "You already have a game open. Close it before hosting another."
    : `You already have ${n} games open. Close one before hosting another.`;

/**
 * Terms that may not appear in a lobby name, matched case-insensitively against the name's letters and
 * digits with spaces and punctuation removed. A name containing one is replaced by "Game <id>".
 * Add entries here; nothing else needs to change.
 */
export const BLOCKED_NAME_TERMS: readonly string[] = [
  // A first filter for the words that make a public list unpleasant, not a complete one. A term is matched
  // anywhere in the name, so leave out any that sits inside ordinary words ("ass" is in "Assyria", "rape" in
  // "grape", "coon" in "tycoon"). A host can still be removed from the lobby by closing the room.
  "fuck", "fuk", "fck", "shit", "cunt", "bitch", "whore", "slut", "pussy", "penis", "vagina", "dildo", "jizz",
  "wank", "porn", "rapist", "nigger", "nigga", "faggot", "tranny", "kike", "retard", "hitler", "nazi", "kkk",
  "killyourself",
];

export const KIND_OPEN = 0;
export const KIND_DATA = 1;
export const KIND_CLOSE = 2;
export const KIND_STATUS = 3;
export const KIND_SUMMARY = 4;
export const KIND_BYE = 5;
export const KIND_PING = 6; // host keepalive (browsers cannot send WebSocket pings); never forwarded

export interface Env {
  LOBBY: DurableObjectNamespace;
  ROOM: DurableObjectNamespace;
  /** "0" stops new rooms (POST /rooms answers 503); anything else, or unset, keeps the lobby open. */
  LOBBY_OPEN?: string;
  MAX_ROOMS?: string;
  MAX_ROOMS_PER_IP?: string;
  ROOM_CREATES_PER_10_MIN?: string;
  MAX_GUESTS_PER_IP?: string;
}

/** The tunables from Worker vars; a missing or unusable value falls back to its default. */
export function limitsFrom(env: Partial<Record<keyof Limits, unknown>>): Limits {
  const out = { ...DEFAULT_LIMITS };
  for (const k of Object.keys(out) as (keyof Limits)[]) {
    const raw = env[k];
    const n = typeof raw === "number" ? raw : typeof raw === "string" && /^\d{1,6}$/.test(raw.trim()) ? Number(raw.trim()) : NaN;
    if (Number.isSafeInteger(n) && n >= 1) out[k] = n;
  }
  return out;
}

export interface RoomAnnounce {
  name: string;
  build: string;
  rules_version: number;
  protocol: number;
  /** The host set a join code. The relay learns only this yes/no, never the code. */
  locked: boolean;
}

export interface Summary {
  players: number;
  humans: number;
  open_seats: number;
  phase: "staging" | "playing";
  turn: number;
}

export type RoomInfo = RoomAnnounce & Summary & { room: string; updated_ms: number };

export const DEFAULT_SUMMARY: Summary = { players: 0, humans: 0, open_seats: 0, phase: "staging", turn: 0 };

const isCount = (v: unknown): v is number => typeof v === "number" && Number.isSafeInteger(v) && v >= 0;

/** Characters a lobby name may keep: letters and marks of any script, digits, whitespace, and this punctuation. */
const NAME_DROP = /[^\p{L}\p{M}\p{N}\s'’.,!?&()#:_-]/gu;

type SegmenterCtor = new (locale?: string, options?: { granularity: "grapheme" }) => {
  segment(s: string): Iterable<{ segment: string }>;
};

/** Visible characters (grapheme clusters) of a string. */
function graphemes(s: string): string[] {
  const Seg = (Intl as unknown as { Segmenter?: SegmenterCtor }).Segmenter;
  return Seg ? Array.from(new Seg(undefined, { granularity: "grapheme" }).segment(s), (x) => x.segment) : Array.from(s);
}

const foldForBlocklist = (s: string): string => s.toLowerCase().replace(/[^\p{L}\p{N}]/gu, "");

/**
 * Cleans a host-supplied lobby name. Keeps letters (any script), digits, spaces and a small punctuation
 * set; drops everything else, including emoji, symbols, and invisible or direction-changing characters;
 * collapses whitespace; keeps at most 3 combining marks on a letter and at most 40 visible characters.
 * Returns "" when nothing is left or the name contains a blocked term.
 */
export function sanitizeName(raw: string, blocklist: readonly string[] = BLOCKED_NAME_TERMS): string {
  let s = raw.normalize("NFC").replace(NAME_DROP, "");
  s = s.replace(/(^|[^\p{L}\p{M}])\p{M}+/gu, "$1"); // a mark must sit on a letter
  s = s.replace(/(\p{M}{3})\p{M}+/gu, "$1"); // no towers of marks
  s = s.replace(/\s+/gu, " ").trim();
  const g = graphemes(s);
  if (g.length > MAX_NAME_CHARS) s = g.slice(0, MAX_NAME_CHARS).join("").trimEnd();
  const folded = foldForBlocklist(s);
  for (const term of blocklist) {
    const t = foldForBlocklist(term);
    if (t && folded.includes(t)) return "";
  }
  return s;
}

/** The name shown for a room whose host gave none, or none usable. */
export const defaultName = (room: string): string => `Game ${room.slice(0, 4).toUpperCase()}`;

/**
 * Strict validation of a POST /rooms body. Returns null when anything is off. Extra fields are dropped.
 * The name may be missing or empty; it is cleaned with `sanitizeName` and may come back "", which the
 * caller replaces with `defaultName`.
 */
export function parseAnnounce(v: unknown): RoomAnnounce | null {
  if (typeof v !== "object" || v === null || Array.isArray(v)) return null;
  const o = v as Record<string, unknown>;
  const { build, rules_version, protocol } = o;
  const name = o.name ?? "";
  if (typeof name !== "string" || name.length > MAX_NAME_INPUT) return null;
  if (typeof build !== "string" || build.length < 1 || build.length > MAX_BUILD) return null;
  if (!isCount(rules_version) || !isCount(protocol)) return null;
  const locked = o.locked === undefined ? false : o.locked;
  if (typeof locked !== "boolean") return null;
  // Every player in the lobby reads the build label, so it holds only what a real label is made of
  // ("Version 0.1.0 (abc1234)", "Dev · after 9 (abc1234)", "playtest-9"): no markup, links or symbols.
  if (!BUILD_RE.test(build)) return null;
  return { name: sanitizeName(name), build, rules_version, protocol, locked };
}

const BUILD_RE = /^[\p{L}\p{N}][\p{L}\p{N} .·()_+-]*$/u;

/** Validate a host `summary` frame. Returns null when it is not usable (the old summary stays). */
export function parseSummary(v: unknown): Summary | null {
  if (typeof v !== "object" || v === null || Array.isArray(v)) return null;
  const o = v as Record<string, unknown>;
  if (!isCount(o.players) || !isCount(o.humans) || !isCount(o.open_seats) || !isCount(o.turn)) return null;
  if (o.phase !== "staging" && o.phase !== "playing") return null;
  return { players: o.players, humans: o.humans, open_seats: o.open_seats, phase: o.phase, turn: o.turn };
}

/** Random string over a 32-character URL-safe alphabet; 32 divides 256 so there is no modulo bias. */
const ALPHABET = "abcdefghijkmnpqrstuvwxyz23456789";
export function randomToken(len: number): string {
  const bytes = new Uint8Array(len);
  crypto.getRandomValues(bytes);
  let s = "";
  for (const b of bytes) s += ALPHABET[b & 31];
  return s;
}

export const ROOM_ID_RE = /^[A-Za-z0-9_-]{10,64}$/;
export const ROOM_ID_LEN = 16; // 80 bits
export const HOST_KEY_LEN = 40; // 200 bits

async function sha256(s: string): Promise<Uint8Array> {
  return new Uint8Array(await crypto.subtle.digest("SHA-256", new TextEncoder().encode(s)));
}

export const hashKey = async (key: string): Promise<string> =>
  Array.from(await sha256(key), (b) => b.toString(16).padStart(2, "0")).join("");

/** Constant-time check of a presented host key against the stored hex SHA-256. */
export async function keyMatches(presented: string, storedHashHex: string): Promise<boolean> {
  const a = await hashKey(presented);
  let diff = a.length ^ storedHashHex.length;
  for (let i = 0; i < a.length; i++) diff |= a.charCodeAt(i) ^ (storedHashHex.charCodeAt(i) || 0);
  return diff === 0;
}

/** Random hex salt for IP hashing. It never leaves the Durable Object that made it. */
export function newSalt(): string {
  const b = new Uint8Array(16);
  crypto.getRandomValues(b);
  return Array.from(b, (x) => x.toString(16).padStart(2, "0")).join("");
}

/**
 * What a client address is limited as. An IPv4 address stands for itself. An IPv6 address is cut to its
 * first 64 bits: one home or phone line owns the whole /64 behind them, so counting each of its 2^64
 * addresses on its own would let one line step around every per-address limit.
 */
export function addressKey(ip: string): string {
  if (!ip.includes(":")) return ip;
  const mapped = /^::ffff:(\d{1,3}(?:\.\d{1,3}){3})$/i.exec(ip);
  if (mapped) return mapped[1]!;
  const [head = "", tail] = ip.split("::");
  const h = head ? head.split(":") : [];
  const t = tail ? tail.split(":") : [];
  const groups = tail === undefined ? h : [...h, ...new Array<string>(Math.max(0, 8 - h.length - t.length)).fill("0"), ...t];
  return `${groups.slice(0, 4).map((g) => g.toLowerCase().replace(/^0+(?=.)/, "")).join(":")}::/64`;
}

/** A salted SHA-256 of a client address (see `addressKey`), 128 bits of hex. The raw address is never stored or logged. */
export async function ipHash(salt: string, ip: string): Promise<string> {
  return (await hashKey(`${salt}|${addressKey(ip)}`)).slice(0, 32);
}

/** The client's address as Cloudflare reports it; "" when absent (local dev without the header). */
export const clientIp = (request: Request): string => request.headers.get("CF-Connecting-IP") ?? "";

/** A token bucket: `tokens` left at time `at` (ms). */
export interface Bucket {
  tokens: number;
  at: number;
}

/**
 * Takes `cost` tokens from a bucket that holds at most `capacity` and refills `capacity` tokens every
 * `windowMs`. A missing bucket is full. Returns whether the take succeeded, and the bucket to keep.
 */
export function takeToken(
  b: Bucket | null,
  now: number,
  cost: number,
  capacity: number,
  windowMs: number,
): { ok: boolean; bucket: Bucket } {
  const prev = b ?? { tokens: capacity, at: now };
  const elapsed = Math.max(0, now - prev.at);
  const tokens = Math.min(capacity, prev.tokens + (elapsed * capacity) / windowMs);
  if (tokens >= cost) return { ok: true, bucket: { tokens: tokens - cost, at: now } };
  return { ok: false, bucket: { tokens, at: now } };
}

/** Milliseconds until a bucket holds `cost` tokens again. */
export function msUntil(b: Bucket, cost: number, capacity: number, windowMs: number): number {
  return b.tokens >= cost ? 0 : Math.ceil(((cost - b.tokens) * windowMs) / capacity);
}

/**
 * One socket's message and byte caps. Kept in memory only: after hibernation (which needs a quiet socket)
 * the socket starts with full buckets again, which is about what a quiet socket would have earned anyway.
 */
export class SocketRate {
  private msgs: Bucket | null = null;
  private bytes: Bucket | null = null;
  constructor(private readonly r: Rate) {}

  /** null when a message of `size` bytes may pass; otherwise the close reason. */
  take(size: number, now: number): string | null {
    const m = takeToken(this.msgs, now, 1, this.r.msgBurst, (this.r.msgBurst / this.r.msgPerSec) * 1000);
    this.msgs = m.bucket;
    if (!m.ok) return REASON_TOO_MANY_MESSAGES;
    const b = takeToken(this.bytes, now, size, this.r.byteBurst, (this.r.byteBurst / this.r.bytePerSec) * 1000);
    this.bytes = b.bucket;
    if (!b.ok) return REASON_TOO_MUCH_DATA;
    return null;
  }
}

export function frame(conn: number, kind: number, payload?: Uint8Array): ArrayBuffer {
  const len = payload ? payload.length : 0;
  const out = new Uint8Array(5 + len);
  new DataView(out.buffer).setUint32(0, conn, false);
  out[4] = kind;
  if (payload) out.set(payload, 5);
  return out.buffer;
}
