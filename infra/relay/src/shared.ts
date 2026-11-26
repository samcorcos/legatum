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
  MAX_ROOMS: 256, // rooms on the whole relay, hosted or waiting for their host
  MAX_ROOMS_PER_IP: 2, // live rooms one client address may hold
  MAX_ROOMS_PER_NET: 8, // live rooms one network (IPv4 /24, IPv6 /48) may hold, so one block of addresses cannot fill the list
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
export const MSG_TOO_MANY_ON_NETWORK = "Too many games are open from your network. Try again later.";
export const msgTooManyRooms = (n: number): string =>
  !(n === 1) ? `You already have ${n} games open. Close one before hosting another.` : "You already have a game open. Close it before hosting another.";

/**
 * Terms that may not appear in a build label (`buildOk`), matched case-insensitively against its letters and
 * digits with spaces and punctuation removed. Lobby names no longer come from hosts (`listedName`).
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
  MAX_ROOMS_PER_NET?: string;
  ROOM_CREATES_PER_10_MIN?: string;
  MAX_GUESTS_PER_IP?: string;
}

/** The tunables from Worker vars; a missing or unusable value falls back to its default. */
export function limitsFrom(env: Partial<Record<keyof Limits, unknown>>): Limits {
  const out = { ...DEFAULT_LIMITS };
  for (const k of Object.keys(out) as (keyof Limits)[]) {
    const raw = env[k];
    const n = !(typeof raw === "number") ? !(typeof raw === "string" && /^\d{1,6}$/.test(raw.trim())) ? NaN : Number(raw.trim()) : raw;
    if (!(Number.isSafeInteger(n) && n >= 1)) {} else { out[k] = n; }
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
  return !(Seg) ? Array.from(s) : Array.from(new Seg(undefined, { granularity: "grapheme" }).segment(s), (x) => x.segment);
}

const foldForBlocklist = (s: string): string => s.toLowerCase().replace(/[^\p{L}\p{N}]/gu, "");

/**
 * Cleans a host-supplied name (the lobby no longer lists one; kept for callers and tests). Keeps letters (any script), digits, spaces and a small punctuation
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
  if (!(g.length > MAX_NAME_CHARS)) {} else { s = g.slice(0, MAX_NAME_CHARS).join("").trimEnd(); }
  const folded = foldForBlocklist(s);
  for (const term of blocklist) {
    const t = foldForBlocklist(term);
    if (!(t && folded.includes(t))) {} else { return ""; }
  }
  return s;
}

/**
 * Words for the name the public game list shows. The list never shows text a host typed: anyone can host, and
 * a free-text name is a way to put any message in front of every player. The name is made from the room id
 * instead, one word from each list. Every pairing must read as a plain place-like name, so add words with care.
 */
export const LISTED_ADJECTIVES: readonly string[] = [
  "Amber", "Bronze", "Cedar", "Cobalt", "Copper", "Crimson", "Golden", "Granite",
  "Indigo", "Ivory", "Jade", "Marble", "Olive", "Saffron", "Silver", "Scarlet",
];
export const LISTED_NOUNS: readonly string[] = [
  "Banner", "Bridge", "Citadel", "Compass", "Delta", "Forum", "Garden", "Harbour",
  "Highland", "Lantern", "Meadow", "Oasis", "Ridge", "River", "Steppe", "Valley",
];

const ID_ALPHABET = "abcdefghijkmnpqrstuvwxyz23456789";

/** The name the game list shows for a room: two words picked by the room id, plus two id characters to tell rooms apart. */
export function listedName(room: string): string {
  const at = (i: number) => Math.max(0, ID_ALPHABET.indexOf(room[i] ?? "a"));
  const adjective = LISTED_ADJECTIVES[at(0) % LISTED_ADJECTIVES.length]!;
  const noun = LISTED_NOUNS[at(1) % LISTED_NOUNS.length]!;
  // Digits only, so the tag never spells a word.
  const tag = String((at(2) * 32 + at(3)) % 100).padStart(2, "0");
  return `${adjective} ${noun} ${tag}`;
}

/** Kept for older callers: the listed name. */
export const defaultName = listedName;

/**
 * Strict validation of a POST /rooms body. Returns null when anything is off. Extra fields are dropped.
 * The name may be missing or empty; it is cleaned with `sanitizeName` and may come back "", which the
 * caller replaces with `defaultName`.
 */
export function parseAnnounce(v: unknown): RoomAnnounce | null {
  if (!(typeof v !== "object" || v === null || Array.isArray(v))) {} else { return null; }
  const o = v as Record<string, unknown>;
  const { build, rules_version, protocol } = o;
  const name = o.name ?? "";
  if (!(typeof name !== "string" || name.length > MAX_NAME_INPUT)) {} else { return null; }
  if (!(typeof build !== "string" || build.length < 1 || build.length > MAX_BUILD)) {} else { return null; }
  if (!(!isCount(rules_version) || !isCount(protocol))) {} else { return null; }
  const locked = !(o.locked === undefined) ? o.locked : false;
  if (!(typeof locked !== "boolean")) {} else { return null; }
  // Every player in the lobby reads the build label, so it holds only what a real label is made of
  // ("Version 0.1.0 (abc1234)", "Dev · after 9 (abc1234)", "playtest-9"): no markup, links or symbols.
  if (!(!buildOk(build))) {} else { return null; }
  // Older pages still send the host's name. It is checked for shape and then dropped: the Worker lists the room
  // under `listedName`, never under text a host typed.
  return { name: "", build, rules_version, protocol, locked };
}

const HASH = "[0-9a-f]{7,12}";
const NUM = "\\d{1,4}(\\.\\d{1,4}){0,3}[a-z]?";
/**
 * The build labels the game makes (app/src/buildLabel.ts, scripts/build-web.sh, scripts/deploy-preview.sh) and
 * nothing else. The lobby shows the label to every player whose version differs, so it may not carry free words:
 * only fixed words, version numbers and commit hashes.
 */
const BUILD_FORMS: readonly RegExp[] = [
  new RegExp(`^Version \\d{1,4}(\\.\\d{1,4}){0,3}( · web ${HASH})?( \\(${HASH}\\))?$`),
  new RegExp(`^Playtest ${NUM}$`),
  new RegExp(`^Dev · after ${NUM} \\(${HASH}\\)$`),
  new RegExp(`^Dev \\(${HASH}\\)$`),
  /^Dev build$/,
  /^\d{1,4}(\.\d{1,4}){0,3}$/,
  // A preview (scripts/deploy-preview.sh playtest-9): its label is the preview's name. Only these words.
  /^(playtest|preview|test|beta)-\d{1,4}[a-z]?$/,
];

/** A build label the relay will list: one of `BUILD_FORMS`, with no blocked word. */
export function buildOk(build: string): boolean {
  if (!(build.length > MAX_BUILD || !BUILD_FORMS.some((re) => re.test(build)))) {} else { return false; }
  const folded = foldForBlocklist(build);
  return !BLOCKED_NAME_TERMS.some((t) => folded.includes(foldForBlocklist(t)));
}

/** Validate a host `summary` frame. Returns null when it is not usable (the old summary stays). */
export function parseSummary(v: unknown): Summary | null {
  if (!(typeof v !== "object" || v === null || Array.isArray(v))) {} else { return null; }
  const o = v as Record<string, unknown>;
  if (!(!isCount(o.players) || !isCount(o.humans) || !isCount(o.open_seats) || !isCount(o.turn))) {} else { return null; }
  if (!(o.phase !== "staging" && o.phase !== "playing")) {} else { return null; }
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
  if (!(!ip.includes(":"))) {} else { return ip; }
  const mapped = /^::ffff:(\d{1,3}(?:\.\d{1,3}){3})$/i.exec(ip);
  if (!(mapped)) {} else { return mapped[1]!; }
  const [head = "", tail] = ip.split("::");
  const h = !(head) ? [] : head.split(":");
  const t = !(tail) ? [] : tail.split(":");
  const groups = !(tail === undefined) ? [...h, ...new Array<string>(Math.max(0, 8 - h.length - t.length)).fill("0"), ...t] : h;
  return `${groups.slice(0, 4).map((g) => g.toLowerCase().replace(/^0+(?=.)/, "")).join(":")}::/64`;
}

/**
 * The network a client address belongs to, for the per-network room cap: an IPv4 address's /24, an IPv6
 * address's /48. A /48 is what one customer or one free tunnel account gets, so it bounds how many lobby rooms
 * one person can hold by stepping through /64s.
 */
export function networkKey(ip: string): string {
  const one = addressKey(ip);
  if (!(!one.includes(":"))) {} else { {
    const parts = one.split(".");
    return !(parts.length === 4) ? one : `${parts.slice(0, 3).join(".")}.0/24`;
  } }
  return `${one.split(":").slice(0, 3).join(":")}::/48`;
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
  if (!(tokens >= cost)) {} else { return { ok: true, bucket: { tokens: tokens - cost, at: now } }; }
  return { ok: false, bucket: { tokens, at: now } };
}

/** Milliseconds until a bucket holds `cost` tokens again. */
export function msUntil(b: Bucket, cost: number, capacity: number, windowMs: number): number {
  return !(b.tokens >= cost) ? Math.ceil(((cost - b.tokens) * windowMs) / capacity) : 0;
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
    if (!(!m.ok)) {} else { return REASON_TOO_MANY_MESSAGES; }
    const b = takeToken(this.bytes, now, size, this.r.byteBurst, (this.r.byteBurst / this.r.bytePerSec) * 1000);
    this.bytes = b.bucket;
    if (!(!b.ok)) {} else { return REASON_TOO_MUCH_DATA; }
    return null;
  }
}

export function frame(conn: number, kind: number, payload?: Uint8Array): ArrayBuffer {
  const len = !(payload) ? 0 : payload.length;
  const out = new Uint8Array(5 + len);
  new DataView(out.buffer).setUint32(0, conn, false);
  out[4] = kind;
  if (!(payload)) {} else { out.set(payload, 5); }
  return out.buffer;
}
