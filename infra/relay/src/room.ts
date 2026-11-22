import { DurableObject } from "cloudflare:workers";
import {
  Env,
  GRACE_MS,
  KIND_BYE,
  KIND_CLOSE,
  KIND_DATA,
  KIND_OPEN,
  KIND_STATUS,
  KIND_SUMMARY,
  MAX_GUESTS,
  MAX_GUEST_MSG,
  MAX_HOST_MSG,
  RoomAnnounce,
  frame,
  keyMatches,
  parseSummary,
} from "./shared";

interface Meta {
  room: string;
  keyHash: string;
}
/** Per-socket state that survives hibernation (serializeAttachment, max 16 KiB). */
interface HostAtt {
  nextConn: number; // next guest connection id for this tunnel
  dead?: boolean; // replaced or being closed by the relay: ignore its events
}
interface GuestAtt {
  conn: number;
  dead?: boolean; // closed by the relay: do not tell the host again
}

const OPEN = 1; // WebSocket.READY_STATE_OPEN
const enc = new TextEncoder();
const dec = new TextDecoder();
const json = (body: unknown, status = 200): Response =>
  new Response(JSON.stringify(body), { status, headers: { "content-type": "application/json" } });

/**
 * One hosted game. Holds the host tunnel and up to 8 guest sockets (WebSocket Hibernation API) and
 * copies bytes between them. It reads the 5-byte tunnel header and nothing else, except the host's
 * `status` and `summary` frames, which the contract defines.
 *
 * Tags: "host" on the tunnel; "guest" and "c<conn>" on each guest.
 */
export class Room extends DurableObject<Env> {
  private meta: Meta | null | undefined;

  private async getMeta(): Promise<Meta | null> {
    if (this.meta === undefined) this.meta = (await this.ctx.storage.get<Meta>("meta")) ?? null;
    return this.meta;
  }

  /** Called by the Worker right after the Lobby accepted the room. */
  async init(room: string, keyHash: string): Promise<void> {
    this.meta = { room, keyHash };
    await this.ctx.storage.put("meta", this.meta);
    // A room whose host never shows up goes away on its own.
    await this.ctx.storage.setAlarm(Date.now() + GRACE_MS);
  }

  private lobby() {
    return this.env.LOBBY.get(this.env.LOBBY.idFromName("lobby")) as unknown as {
      setConnected(room: string, c: boolean): Promise<void>;
      setSummary(room: string, s: unknown): Promise<void>;
      remove(room: string): Promise<void>;
    };
  }

  private liveHost(): WebSocket | null {
    for (const ws of this.ctx.getWebSockets("host")) {
      const att = ws.deserializeAttachment() as HostAtt | null;
      if (ws.readyState === OPEN && !att?.dead) return ws;
    }
    return null;
  }

  private liveGuests(): WebSocket[] {
    return this.ctx
      .getWebSockets("guest")
      .filter((ws) => ws.readyState === OPEN && !(ws.deserializeAttachment() as GuestAtt | null)?.dead);
  }

  private closeGuests(code: number, reason: string): void {
    for (const ws of this.ctx.getWebSockets("guest")) {
      const att = (ws.deserializeAttachment() as GuestAtt | null) ?? { conn: 0 };
      if (!att.dead) ws.serializeAttachment({ ...att, dead: true });
      try {
        ws.close(code, reason);
      } catch {
        /* already closed */
      }
    }
  }

  // ---- HTTP entry ----------------------------------------------------------------------------

  async fetch(request: Request): Promise<Response> {
    const meta = await this.getMeta();
    if (!meta) return json({ error: "no such room" }, 404);
    const url = new URL(request.url);
    if (url.pathname.endsWith("/api/status")) return this.status();
    if (url.pathname.endsWith("/host")) return this.acceptHost(request, url, meta);
    if (url.pathname.endsWith("/ws")) return this.acceptGuest(request);
    return json({ error: "not found" }, 404);
  }

  private async status(): Promise<Response> {
    if (!this.liveHost()) return json({ error: "host not connected" }, 503);
    const raw = await this.ctx.storage.get<string>("status");
    if (raw === undefined) return json({ error: "host has not reported status yet" }, 503);
    const body = JSON.parse(raw) as Record<string, unknown>;
    body.is_local = false;
    return json(body);
  }

  private async acceptHost(request: Request, url: URL, meta: Meta): Promise<Response> {
    if (request.headers.get("Upgrade")?.toLowerCase() !== "websocket") return json({ error: "websocket required" }, 426);
    if (!(await keyMatches(url.searchParams.get("key") ?? "", meta.keyHash))) return json({ error: "bad host key" }, 403);

    const old = this.ctx.getWebSockets("host");
    const pair = new WebSocketPair();
    const [client, server] = [pair[0], pair[1]];
    this.ctx.acceptWebSocket(server, ["host"]);
    server.serializeAttachment({ nextConn: 1 } satisfies HostAtt);
    // The new tunnel replaces the old one: stale guests cannot be carried over (conn ids restart).
    for (const ws of old) {
      const att = (ws.deserializeAttachment() as HostAtt | null) ?? { nextConn: 1 };
      ws.serializeAttachment({ ...att, dead: true });
      try {
        ws.close(1012, "replaced by a newer tunnel");
      } catch {
        /* already closed */
      }
    }
    this.closeGuests(1012, "tunnel replaced");
    await this.ctx.storage.deleteAlarm();
    await this.lobby().setConnected(meta.room, true);
    return new Response(null, { status: 101, webSocket: client });
  }

  private acceptGuest(request: Request): Response {
    if (request.headers.get("Upgrade")?.toLowerCase() !== "websocket") return json({ error: "websocket required" }, 426);
    const pair = new WebSocketPair();
    const [client, server] = [pair[0], pair[1]];
    const host = this.liveHost();
    if (!host || this.liveGuests().length >= MAX_GUESTS) {
      server.accept();
      server.close(1013, host ? "room is full" : "host not connected");
      return new Response(null, { status: 101, webSocket: client });
    }
    const hatt = (host.deserializeAttachment() as HostAtt | null) ?? { nextConn: 1 };
    const conn = hatt.nextConn;
    host.serializeAttachment({ ...hatt, nextConn: conn + 1 });
    this.ctx.acceptWebSocket(server, ["guest", `c${conn}`]);
    server.serializeAttachment({ conn } satisfies GuestAtt);
    host.send(frame(conn, KIND_OPEN));
    return new Response(null, { status: 101, webSocket: client });
  }

  // ---- WebSocket Hibernation handlers --------------------------------------------------------

  async webSocketMessage(ws: WebSocket, message: string | ArrayBuffer): Promise<void> {
    if (this.ctx.getTags(ws).includes("host")) return this.fromHost(ws, message);
    return this.fromGuest(ws, message);
  }

  private fromGuest(ws: WebSocket, message: string | ArrayBuffer): void {
    const att = ws.deserializeAttachment() as GuestAtt | null;
    if (!att || att.dead) return;
    if (typeof message !== "string") return this.endGuest(ws, att, 1003, "text only");
    const bytes = enc.encode(message);
    if (bytes.length > MAX_GUEST_MSG) return this.endGuest(ws, att, 1009, "message too big");
    this.liveHost()?.send(frame(att.conn, KIND_DATA, bytes));
  }

  /** The relay ends a guest: tell the host, then close the socket. */
  private endGuest(ws: WebSocket, att: GuestAtt, code: number, reason: string): void {
    ws.serializeAttachment({ ...att, dead: true });
    this.liveHost()?.send(frame(att.conn, KIND_CLOSE));
    try {
      ws.close(code, reason);
    } catch {
      /* already closed */
    }
  }

  private async fromHost(ws: WebSocket, message: string | ArrayBuffer): Promise<void> {
    const hatt = ws.deserializeAttachment() as HostAtt | null;
    if (hatt?.dead) return;
    if (typeof message === "string") return void ws.close(1003, "binary frames only");
    if (message.byteLength < 5) return void ws.close(1002, "short frame");
    const view = new DataView(message);
    const conn = view.getUint32(0, false);
    const kind = view.getUint8(4);
    const payload = new Uint8Array(message, 5);
    if (payload.length > MAX_HOST_MSG) return void ws.close(1009, "message too big");

    switch (kind) {
      case KIND_DATA: {
        const guest = this.guest(conn);
        if (!guest) return void ws.send(frame(conn, KIND_CLOSE)); // unknown or gone: let the host forget it
        guest.send(dec.decode(payload));
        return;
      }
      case KIND_CLOSE: {
        const guest = this.guest(conn);
        if (!guest) return;
        guest.serializeAttachment({ conn, dead: true });
        guest.close(1000, "closed by host");
        return;
      }
      case KIND_STATUS: {
        const text = dec.decode(payload);
        try {
          const v = JSON.parse(text);
          if (typeof v !== "object" || v === null || Array.isArray(v)) return;
        } catch {
          return;
        }
        await this.ctx.storage.put("status", text);
        return;
      }
      case KIND_SUMMARY: {
        let s;
        try {
          s = parseSummary(JSON.parse(dec.decode(payload)));
        } catch {
          return;
        }
        const meta = await this.getMeta();
        if (s && meta) await this.lobby().setSummary(meta.room, s);
        return;
      }
      case KIND_BYE:
        return this.destroy("host said bye");
      default:
        return; // unknown kinds (and a stray `open`) are ignored
    }
  }

  private guest(conn: number): WebSocket | null {
    for (const ws of this.ctx.getWebSockets(`c${conn}`)) {
      const att = ws.deserializeAttachment() as GuestAtt | null;
      if (ws.readyState === OPEN && !att?.dead) return ws;
    }
    return null;
  }

  async webSocketClose(ws: WebSocket, code: number, reason: string): Promise<void> {
    try {
      ws.close(code === 1005 || code === 1006 ? 1000 : code, reason);
    } catch {
      /* already closed */
    }
    await this.gone(ws);
  }

  async webSocketError(ws: WebSocket): Promise<void> {
    await this.gone(ws);
  }

  private async gone(ws: WebSocket): Promise<void> {
    if (this.ctx.getTags(ws).includes("host")) {
      const att = ws.deserializeAttachment() as HostAtt | null;
      if (att?.dead || this.liveHost()) return; // replaced, or a newer tunnel is already up
      this.closeGuests(1012, "host gone");
      const meta = await this.getMeta();
      if (!meta) return;
      await this.lobby().setConnected(meta.room, false);
      await this.ctx.storage.setAlarm(Date.now() + GRACE_MS);
    } else {
      const att = ws.deserializeAttachment() as GuestAtt | null;
      if (!att || att.dead) return;
      ws.serializeAttachment({ ...att, dead: true });
      this.liveHost()?.send(frame(att.conn, KIND_CLOSE));
    }
  }

  /** Grace period over (or the host left for good): remove the room completely. */
  async alarm(): Promise<void> {
    if (this.liveHost()) return;
    await this.destroy("room expired");
  }

  private async destroy(reason: string): Promise<void> {
    const meta = await this.getMeta();
    if (meta) await this.lobby().remove(meta.room);
    this.closeGuests(1012, reason);
    for (const ws of this.ctx.getWebSockets("host")) {
      ws.serializeAttachment({ nextConn: 0, dead: true });
      try {
        ws.close(1000, reason);
      } catch {
        /* already closed */
      }
    }
    await this.ctx.storage.deleteAlarm();
    await this.ctx.storage.deleteAll();
    this.meta = null;
  }
}

export type { RoomAnnounce };
