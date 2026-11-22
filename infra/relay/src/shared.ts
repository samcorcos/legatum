// Shared types, limits and helpers. The relay never looks at game messages, only at this.

export const MAX_ROOMS = 64;
export const MAX_GUESTS = 8;
export const MAX_GUEST_MSG = 64 * 1024; // guest -> host, bytes
export const MAX_HOST_MSG = 1024 * 1024; // host -> guest, bytes
export const GRACE_MS = 120_000; // room lives this long after its host tunnel drops
export const MAX_NAME = 80;
export const MAX_BUILD = 40;

export const KIND_OPEN = 0;
export const KIND_DATA = 1;
export const KIND_CLOSE = 2;
export const KIND_STATUS = 3;
export const KIND_SUMMARY = 4;
export const KIND_BYE = 5;

export interface Env {
  LOBBY: DurableObjectNamespace;
  ROOM: DurableObjectNamespace;
}

export interface RoomAnnounce {
  name: string;
  build: string;
  rules_version: number;
  protocol: number;
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

/** Strict validation of a POST /rooms body. Returns null when anything is off. Extra fields are dropped. */
export function parseAnnounce(v: unknown): RoomAnnounce | null {
  if (typeof v !== "object" || v === null || Array.isArray(v)) return null;
  const o = v as Record<string, unknown>;
  const { name, build, rules_version, protocol } = o;
  if (typeof name !== "string" || name.length < 1 || name.length > MAX_NAME) return null;
  if (typeof build !== "string" || build.length < 1 || build.length > MAX_BUILD) return null;
  if (!isCount(rules_version) || !isCount(protocol)) return null;
  // Control characters have no business in a lobby row.
  if (/[\u0000-\u001f\u007f]/.test(name) || /[\u0000-\u001f\u007f]/.test(build)) return null;
  return { name, build, rules_version, protocol };
}

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

export function frame(conn: number, kind: number, payload?: Uint8Array): ArrayBuffer {
  const len = payload ? payload.length : 0;
  const out = new Uint8Array(5 + len);
  new DataView(out.buffer).setUint32(0, conn, false);
  out[4] = kind;
  if (payload) out.set(payload, 5);
  return out.buffer;
}
