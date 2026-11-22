import { DurableObject } from "cloudflare:workers";

/** Random hex salt for address hashing. It never leaves the Durable Object that made it. */
export function newSalt(): string {
  const b = new Uint8Array(16);
  crypto.getRandomValues(b);
  return Array.from(b, (x) => x.toString(16).padStart(2, "0")).join("");
}

/**
 * What a client address is limited as. An IPv4 address stands for itself. An IPv6 address is cut to its
 * first 64 bits: one home or phone line owns the whole /64 behind them, so counting each of its 2^64
 * addresses on its own would let one line step around the cap. (Copy of infra/relay/src/shared.ts.)
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
  const digest = new Uint8Array(await crypto.subtle.digest("SHA-256", new TextEncoder().encode(`${salt}|${addressKey(ip)}`)));
  return Array.from(digest, (b) => b.toString(16).padStart(2, "0")).join("").slice(0, 32);
}

/** The client's address as Cloudflare reports it; "" when absent (local dev without the header). */
export const clientIp = (request: Request): string => request.headers.get("CF-Connecting-IP") ?? "";

export const DEFAULT_MAX_NEW_GAMES_PER_DAY = 50;

/** The cap from the Worker var; anything that is not a whole number of at least 1 falls back to the default. */
export function capFrom(raw: string | undefined): number {
  const n = Number(raw);
  return raw !== undefined && /^\d{1,6}$/.test(raw.trim()) && n >= 1 ? n : DEFAULT_MAX_NEW_GAMES_PER_DAY;
}

export const utcDay = (ms: number): string => new Date(ms).toISOString().slice(0, 10);
/** Seconds until 00:00 UTC, at least 1. */
export const secondsToMidnight = (ms: number): number => Math.max(1, Math.ceil(((Math.floor(ms / 86_400_000) + 1) * 86_400_000 - ms) / 1000));

export type TakeResult = { ok: true } | { ok: false; retryAfter: number };

/**
 * One instance counts the new games each client address stored on the current UTC day. It keeps only a salted
 * hash of the address (the salt never leaves this object), only for today: older rows are deleted on every call.
 */
export class Quota extends DurableObject {
  private readonly salt: string;

  constructor(ctx: DurableObjectState, env: unknown) {
    super(ctx, env as never);
    const sql = ctx.storage.sql;
    sql.exec("CREATE TABLE IF NOT EXISTS kv (k TEXT PRIMARY KEY, v TEXT NOT NULL)");
    sql.exec("CREATE TABLE IF NOT EXISTS counts (day TEXT NOT NULL, ip_hash TEXT NOT NULL, n INTEGER NOT NULL, PRIMARY KEY (day, ip_hash))");
    const row = sql.exec("SELECT v FROM kv WHERE k = 'salt'").toArray()[0];
    if (row) this.salt = row.v as string;
    else {
      this.salt = newSalt();
      sql.exec("INSERT INTO kv (k, v) VALUES ('salt', ?)", this.salt);
    }
  }

  /** Count one new game for this address today, or refuse it when the address is at `cap`. `now` is for tests only. */
  async take(ip: string, cap: number, now: number = Date.now()): Promise<TakeResult> {
    const sql = this.ctx.storage.sql;
    const day = utcDay(now);
    sql.exec("DELETE FROM counts WHERE day <> ?", day);
    const hash = await ipHash(this.salt, ip);
    const row = sql.exec("SELECT n FROM counts WHERE day = ? AND ip_hash = ?", day, hash).toArray()[0];
    const n = row ? (row.n as number) : 0;
    if (n >= cap) return { ok: false, retryAfter: secondsToMidnight(now) };
    sql.exec("INSERT INTO counts (day, ip_hash, n) VALUES (?, ?, 1) ON CONFLICT (day, ip_hash) DO UPDATE SET n = n + 1", day, hash);
    return { ok: true };
  }
}
