import { DurableObject } from "cloudflare:workers";
import { DEFAULT_SUMMARY, Env, GRACE_MS, MAX_ROOMS, RoomAnnounce, RoomInfo, Summary } from "./shared";

/**
 * The single Lobby instance. One SQLite row per room. Rooms call in over RPC;
 * the Worker calls `list` and `create`. A row whose host is away stays until its room deletes itself,
 * but is hidden from the list. A safety sweep drops rows nobody has touched for well over the grace period.
 */
export class Lobby extends DurableObject<Env> {
  constructor(ctx: DurableObjectState, env: Env) {
    super(ctx, env);
    ctx.storage.sql.exec(`CREATE TABLE IF NOT EXISTS rooms (
      room TEXT PRIMARY KEY,
      announce TEXT NOT NULL,
      summary TEXT NOT NULL,
      connected INTEGER NOT NULL,
      created_ms INTEGER NOT NULL,
      touched_ms INTEGER NOT NULL,
      updated_ms INTEGER NOT NULL
    )`);
  }

  private sweep(now: number): void {
    this.ctx.storage.sql.exec("DELETE FROM rooms WHERE connected = 0 AND touched_ms < ?", now - 2 * GRACE_MS);
  }

  /** Register a new room. Returns false when the relay already holds the maximum number of rooms. */
  create(room: string, announce: RoomAnnounce): boolean {
    const now = Date.now();
    this.sweep(now);
    const count = this.ctx.storage.sql.exec("SELECT COUNT(*) AS n FROM rooms").one().n as number;
    if (count >= MAX_ROOMS) return false;
    this.ctx.storage.sql.exec(
      "INSERT INTO rooms (room, announce, summary, connected, created_ms, touched_ms, updated_ms) VALUES (?, ?, ?, 0, ?, ?, ?)",
      room,
      JSON.stringify(announce),
      JSON.stringify(DEFAULT_SUMMARY),
      now,
      now,
      now,
    );
    return true;
  }

  setConnected(room: string, connected: boolean): void {
    const now = Date.now();
    this.ctx.storage.sql.exec(
      "UPDATE rooms SET connected = ?, touched_ms = ?, updated_ms = ? WHERE room = ?",
      connected ? 1 : 0,
      now,
      now,
      room,
    );
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

  /** Rooms whose host tunnel is connected, newest first. */
  list(): RoomInfo[] {
    this.sweep(Date.now());
    const out: RoomInfo[] = [];
    const rows = this.ctx.storage.sql.exec(
      "SELECT room, announce, summary, updated_ms FROM rooms WHERE connected = 1 ORDER BY created_ms DESC, rowid DESC",
    );
    for (const r of rows) {
      const a = JSON.parse(r.announce as string) as RoomAnnounce;
      const s = JSON.parse(r.summary as string) as Summary;
      out.push({ ...a, ...s, room: r.room as string, updated_ms: r.updated_ms as number });
    }
    return out;
  }
}
