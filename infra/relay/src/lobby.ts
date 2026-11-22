import { DurableObject } from "cloudflare:workers";
import {
  Bucket,
  CREATE_WINDOW_MS,
  DEFAULT_SUMMARY,
  Env,
  GRACE_MS,
  HEARTBEAT_MS,
  Limits,
  RoomAnnounce,
  RoomInfo,
  Summary,
  ipHash,
  msUntil,
  newSalt,
  takeToken,
} from "./shared";

/** Why `create` refused, for the Worker to turn into a response. */
export type CreateResult =
  | { ok: true }
  | { ok: false; why: "full" }
  | { ok: false; why: "per_ip" }
  | { ok: false; why: "rate"; retryAfterS: number };

/**
 * The single Lobby instance. One SQLite row per room. Rooms call in over RPC;
 * the Worker calls `list` and `create`. A row whose host is away stays until its room deletes itself,
 * but is hidden from the list. A safety sweep drops rows nobody has touched for well over the grace period.
 *
 * Client addresses: `create` receives the raw address, hashes it at once with a salt that never leaves
 * this object, and keeps only the hash: on the room's row (for the per-address room cap, gone with the
 * row) and in a create-rate bucket (gone once the bucket has refilled, at most 10 minutes later).
 */
export class Lobby extends DurableObject<Env> {
  private readonly salt: string;

  constructor(ctx: DurableObjectState, env: Env) {
    super(ctx, env);
    const sql = ctx.storage.sql;
    sql.exec(`CREATE TABLE IF NOT EXISTS rooms (
      room TEXT PRIMARY KEY,
      announce TEXT NOT NULL,
      summary TEXT NOT NULL,
      connected INTEGER NOT NULL,
      created_ms INTEGER NOT NULL,
      touched_ms INTEGER NOT NULL,
      updated_ms INTEGER NOT NULL
    )`);
    // Columns added after the first deploy. ALTER fails harmlessly when the column already exists.
    for (const col of ["ip_hash TEXT NOT NULL DEFAULT ''", "heard_ms INTEGER"]) {
      try {
        sql.exec(`ALTER TABLE rooms ADD COLUMN ${col}`);
      } catch {
        /* already there */
      }
    }
    sql.exec("CREATE INDEX IF NOT EXISTS rooms_ip ON rooms (ip_hash)");
    sql.exec("CREATE TABLE IF NOT EXISTS buckets (ip_hash TEXT PRIMARY KEY, tokens REAL NOT NULL, at_ms INTEGER NOT NULL)");
    sql.exec("CREATE TABLE IF NOT EXISTS kv (k TEXT PRIMARY KEY, v TEXT NOT NULL)");
    const row = sql.exec("SELECT v FROM kv WHERE k = 'salt'").toArray()[0];
    if (row) this.salt = row.v as string;
    else {
      this.salt = newSalt();
      sql.exec("INSERT INTO kv (k, v) VALUES ('salt', ?)", this.salt);
    }
  }

  private sweep(now: number): void {
    this.ctx.storage.sql.exec("DELETE FROM rooms WHERE connected = 0 AND touched_ms < ?", now - 2 * GRACE_MS);
    // A bucket untouched for a whole window is full again, the same as no bucket.
    this.ctx.storage.sql.exec("DELETE FROM buckets WHERE at_ms < ?", now - CREATE_WINDOW_MS);
  }

  /**
   * Register a new room for the client at `ip`, within the limits. Checked in this order: the create
   * rate for the address, the address's live rooms, then the relay's total. Only a successful create
   * spends a token.
   */
  async create(room: string, announce: RoomAnnounce, ip: string, limits: Limits): Promise<CreateResult> {
    const hash = await ipHash(this.salt, ip);
    const sql = this.ctx.storage.sql;
    const now = Date.now();
    this.sweep(now);

    const cap = limits.ROOM_CREATES_PER_10_MIN;
    const b = sql.exec("SELECT tokens, at_ms FROM buckets WHERE ip_hash = ?", hash).toArray()[0];
    const bucket: Bucket | null = b ? { tokens: b.tokens as number, at: b.at_ms as number } : null;
    const taken = takeToken(bucket, now, 1, cap, CREATE_WINDOW_MS);
    if (!taken.ok) return { ok: false, why: "rate", retryAfterS: Math.max(1, Math.ceil(msUntil(taken.bucket, 1, cap, CREATE_WINDOW_MS) / 1000)) };

    const mine = sql.exec("SELECT COUNT(*) AS n FROM rooms WHERE ip_hash = ?", hash).one().n as number;
    if (mine >= limits.MAX_ROOMS_PER_IP) return { ok: false, why: "per_ip" };
    const count = sql.exec("SELECT COUNT(*) AS n FROM rooms").one().n as number;
    if (count >= limits.MAX_ROOMS) return { ok: false, why: "full" };

    sql.exec(
      "INSERT INTO rooms (room, announce, summary, connected, created_ms, touched_ms, updated_ms, ip_hash, heard_ms) VALUES (?, ?, ?, 0, ?, ?, ?, ?, NULL)",
      room,
      JSON.stringify(announce),
      JSON.stringify(DEFAULT_SUMMARY),
      now,
      now,
      now,
      hash,
    );
    sql.exec(
      "INSERT INTO buckets (ip_hash, tokens, at_ms) VALUES (?, ?, ?) ON CONFLICT (ip_hash) DO UPDATE SET tokens = excluded.tokens, at_ms = excluded.at_ms",
      hash,
      taken.bucket.tokens,
      taken.bucket.at,
    );
    return { ok: true };
  }

  /** A host tunnel came up or went down. A new tunnel starts without a heartbeat until it pings. */
  setConnected(room: string, connected: boolean): void {
    const now = Date.now();
    this.ctx.storage.sql.exec(
      "UPDATE rooms SET connected = ?, touched_ms = ?, updated_ms = ?, heard_ms = NULL WHERE room = ?",
      connected ? 1 : 0,
      now,
      now,
      room,
    );
  }

  /** The room last heard from its pinging host at `at` (ms). */
  heard(room: string, at: number): void {
    this.ctx.storage.sql.exec("UPDATE rooms SET heard_ms = ? WHERE room = ?", at, room);
  }

  setSummary(room: string, summary: Summary): void {
    this.ctx.storage.sql.exec(
      "UPDATE rooms SET summary = ?, updated_ms = ? WHERE room = ?",
      JSON.stringify(summary),
      Date.now(),
      room,
    );
  }

  remove(room: string): void {
    this.ctx.storage.sql.exec("DELETE FROM rooms WHERE room = ?", room);
  }

  /**
   * Rooms whose host tunnel is connected, newest first. A host that has pinged and then gone quiet for
   * `HEARTBEAT_MS` is left out (it may have vanished without closing its socket).
   */
  list(): RoomInfo[] {
    const now = Date.now();
    this.sweep(now);
    const out: RoomInfo[] = [];
    const rows = this.ctx.storage.sql.exec(
      "SELECT room, announce, summary, updated_ms FROM rooms WHERE connected = 1 AND (heard_ms IS NULL OR heard_ms >= ?) ORDER BY created_ms DESC, rowid DESC",
      now - HEARTBEAT_MS,
    );
    for (const r of rows) {
      const a = JSON.parse(r.announce as string) as RoomAnnounce;
      const s = JSON.parse(r.summary as string) as Summary;
      out.push({ ...a, locked: a.locked === true, ...s, room: r.room as string, updated_ms: r.updated_ms as number });
    }
    return out;
  }
}
