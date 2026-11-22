import { env, exports } from "cloudflare:workers";
import { runDurableObjectAlarm } from "cloudflare:test";
import { describe, expect, it } from "vitest";

const BASE = "https://relay.test";
const enc = new TextEncoder();
const dec = new TextDecoder();

const call = (path: string, init?: RequestInit) => (exports as unknown as { default: { fetch(u: string, i?: RequestInit): Promise<Response> } }).default.fetch(BASE + path, init);
const announce = (name = "Alice", extra: object = {}) => ({ name, build: "8.9.74", rules_version: 3, protocol: 1, ...extra });

async function create(body: unknown = announce()) {
  const r = await call("/rooms", { method: "POST", body: JSON.stringify(body) });
  return { r, j: (await r.json()) as { room: string; host_key: string } };
}

/** A WebSocket with a message queue and a recorded close. */
class Sock {
  q: (string | ArrayBuffer)[] = [];
  closed: { code: number; reason: string } | null = null;
  private waiters: (() => void)[] = [];
  constructor(public ws: WebSocket) {
    ws.accept();
    ws.binaryType = "arraybuffer";
    ws.addEventListener("message", (e) => {
      this.q.push(e.data as string | ArrayBuffer);
      this.wake();
    });
    ws.addEventListener("close", (e) => {
      this.closed = { code: e.code, reason: e.reason };
      this.wake();
    });
  }
  private wake() {
    this.waiters.splice(0).forEach((w) => w());
  }
  async until(pred: () => boolean, what: string) {
    for (let i = 0; i < 100 && !pred(); i++) {
      await new Promise<void>((res) => {
        this.waiters.push(res);
        setTimeout(res, 50);
      });
    }
    if (!pred()) throw new Error("timed out waiting for " + what);
  }
  async next(): Promise<string | ArrayBuffer> {
    await this.until(() => this.q.length > 0, "a message");
    return this.q.shift()!;
  }
  async nextFrame() {
    const m = (await this.next()) as ArrayBuffer;
    const v = new DataView(m);
    return { conn: v.getUint32(0), kind: v.getUint8(4), payload: dec.decode(new Uint8Array(m, 5)) };
  }
  async waitClosed() {
    await this.until(() => this.closed !== null, "close");
    return this.closed!;
  }
  send(x: string | ArrayBuffer) {
    this.ws.send(x);
  }
}

async function connect(path: string): Promise<Sock> {
  const r = await call(path, { headers: { Upgrade: "websocket" } });
  expect(r.status).toBe(101);
  return new Sock(r.webSocket!);
}

function frame(conn: number, kind: number, payload: string | Uint8Array = ""): ArrayBuffer {
  const p = typeof payload === "string" ? enc.encode(payload) : payload;
  const out = new Uint8Array(5 + p.length);
  new DataView(out.buffer).setUint32(0, conn);
  out[4] = kind;
  out.set(p, 5);
  return out.buffer;
}

async function lobbyRooms() {
  return ((await (await call("/lobby")).json()) as { rooms: Record<string, unknown>[] }).rooms;
}

async function hostedRoom(name = "Alice") {
  const { j } = await create(announce(name));
  const host = await connect(`/rooms/${j.room}/host?key=${j.host_key}`);
  return { ...j, host };
}

const ns = (env as unknown as { ROOM: DurableObjectNamespace }).ROOM;
const alarm = (room: string) => runDurableObjectAlarm(ns.get(ns.idFromName(room)));

describe("announce and lobby", () => {
  it("creates a room with a long random id and key, listed only once the host is connected", async () => {
    const { r, j } = await create();
    expect(r.status).toBe(200);
    expect(j.room.length).toBeGreaterThanOrEqual(10);
    expect(j.host_key.length).toBeGreaterThanOrEqual(32);
    expect(j.room).toMatch(/^[A-Za-z0-9_-]+$/);
    expect((await lobbyRooms()).find((x) => x.room === j.room)).toBeUndefined();

    const host = await connect(`/rooms/${j.room}/host?key=${j.host_key}`);
    const row = (await lobbyRooms()).find((x) => x.room === j.room)!;
    expect(row).toMatchObject({ name: "Alice", build: "8.9.74", rules_version: 3, protocol: 1, phase: "staging", turn: 0 });
    expect(typeof row.updated_ms).toBe("number");
    host.ws.close(1000);
  });

  it("lists newest first", async () => {
    const a = await hostedRoom("first-xx");
    const b = await hostedRoom("second-x");
    const names = (await lobbyRooms()).map((x) => x.name);
    expect(names.indexOf("second-x")).toBeLessThan(names.indexOf("first-xx"));
    a.host.ws.close(1000);
    b.host.ws.close(1000);
  });

  it("validates announcements strictly", async () => {
    const bad: unknown[] = [
      null,
      [],
      "x",
      announce(""),
      announce("x".repeat(81)),
      announce("ok", { build: "" }),
      announce("ok", { build: "b".repeat(41) }),
      announce("ok", { rules_version: "3" }),
      announce("ok", { protocol: -1 }),
      announce("ok", { protocol: 1.5 }),
      { name: "ok", build: "b", protocol: 1 },
      announce("bad\nname"),
    ];
    for (const b of bad) expect((await create(b)).r.status).toBe(400);
    expect((await call("/rooms", { method: "POST", body: "{not json" })).status).toBe(400);
    expect((await create(announce("x".repeat(80)))).r.status).toBe(200);
  });

  it("answers CORS preflight and sets CORS and no-store headers", async () => {
    const pre = await call("/lobby", { method: "OPTIONS" });
    expect(pre.status).toBe(204);
    expect(pre.headers.get("access-control-allow-origin")).toBe("*");
    expect(pre.headers.get("access-control-allow-methods")).toContain("POST");
    const r = await call("/lobby");
    expect(r.headers.get("access-control-allow-origin")).toBe("*");
    expect(r.headers.get("cache-control")).toBe("no-store");
    const nf = await call("/nope");
    expect(nf.status).toBe(404);
    expect(nf.headers.get("access-control-allow-origin")).toBe("*");
  });
});

describe("host tunnel", () => {
  it("rejects a wrong or missing host key and an unknown room", async () => {
    const { j } = await create();
    for (const q of ["?key=wrong", "", `?key=${j.host_key}x`]) {
      const r = await call(`/rooms/${j.room}/host${q}`, { headers: { Upgrade: "websocket" } });
      expect(r.status).toBe(403);
    }
    const r = await call(`/rooms/${"z".repeat(16)}/host?key=${j.host_key}`, { headers: { Upgrade: "websocket" } });
    expect(r.status).toBe(404);
  });

  it("pipes guests to the host and back with the right conn ids, and relays open and close", async () => {
    const { room, host } = await hostedRoom();
    const g1 = await connect(`/rooms/${room}/ws`);
    const g2 = await connect(`/rooms/${room}/ws`);
    const o1 = await host.nextFrame();
    const o2 = await host.nextFrame();
    expect(o1).toMatchObject({ conn: 1, kind: 0, payload: "" });
    expect(o2).toMatchObject({ conn: 2, kind: 0 });

    g2.send('{"hello":"two"}');
    g1.send("héllo ✓");
    expect(await host.nextFrame()).toMatchObject({ conn: 2, kind: 1, payload: '{"hello":"two"}' });
    expect(await host.nextFrame()).toMatchObject({ conn: 1, kind: 1, payload: "héllo ✓" });

    host.send(frame(1, 1, "to one"));
    host.send(frame(2, 1, "to two"));
    expect(await g1.next()).toBe("to one");
    expect(await g2.next()).toBe("to two");

    // guest closes: host is told
    g1.ws.close(1000);
    expect(await host.nextFrame()).toMatchObject({ conn: 1, kind: 2 });
    // host closes a guest: guest socket closes
    host.send(frame(2, 2));
    await g2.waitClosed();
    // conn ids are never reused
    const g3 = await connect(`/rooms/${room}/ws`);
    expect(await host.nextFrame()).toMatchObject({ conn: 3, kind: 0 });
    g3.ws.close(1000);
    host.ws.close(1000);
  });

  it("tells the host to forget a conn it writes to that is gone", async () => {
    const { host } = await hostedRoom();
    host.send(frame(77, 1, "x"));
    expect(await host.nextFrame()).toMatchObject({ conn: 77, kind: 2 });
    host.ws.close(1000);
  });

  it("caches status, forces is_local false, and 503s without a host", async () => {
    const { room, host } = await hostedRoom();
    expect((await call(`/rooms/${room}/api/status`)).status).toBe(503); // no status yet
    host.send(frame(0, 3, JSON.stringify({ is_local: true, phase: "staging", n: 1 })));
    host.send(frame(0, 3, JSON.stringify({ is_local: true, phase: "playing", n: 2 })));
    let body: any;
    for (let i = 0; i < 50; i++) {
      const r = await call(`/rooms/${room}/api/status`);
      if (r.status === 200 && ((body = await r.json()) as { n: number }).n === 2) break;
      await new Promise((r) => setTimeout(r, 20));
    }
    expect(body).toEqual({ is_local: false, phase: "playing", n: 2 });
    host.ws.close(1000);
    await host.waitClosed();
    for (let i = 0; i < 50 && (await call(`/rooms/${room}/api/status`)).status !== 503; i++) await new Promise((r) => setTimeout(r, 20));
    expect((await call(`/rooms/${room}/api/status`)).status).toBe(503);
    expect((await call(`/rooms/${"q".repeat(16)}/api/status`)).status).toBe(404);
  });

  it("forwards summary frames to the lobby and ignores invalid ones", async () => {
    const { room, host } = await hostedRoom();
    const s = { players: 6, humans: 2, open_seats: 1, phase: "playing", turn: 12 };
    host.send(frame(0, 4, JSON.stringify(s)));
    host.send(frame(0, 4, JSON.stringify({ ...s, phase: "weird" })));
    host.send(frame(0, 4, "not json"));
    let row: Record<string, unknown> | undefined;
    for (let i = 0; i < 50; i++) {
      row = (await lobbyRooms()).find((x) => x.room === room);
      if (row?.turn === 12) break;
      await new Promise((r) => setTimeout(r, 20));
    }
    expect(row).toMatchObject({ name: "Alice", ...s });
    host.ws.close(1000);
  });
});

describe("limits", () => {
  it("allows at most 8 guests, closing the ninth with 1013", async () => {
    const { room, host } = await hostedRoom();
    const gs: Sock[] = [];
    for (let i = 0; i < 8; i++) gs.push(await connect(`/rooms/${room}/ws`));
    for (let i = 0; i < 8; i++) expect((await host.nextFrame()).kind).toBe(0);
    const ninth = await connect(`/rooms/${room}/ws`);
    expect((await ninth.waitClosed()).code).toBe(1013);
    gs[0]!.ws.close(1000);
    expect((await host.nextFrame()).kind).toBe(2);
    const again = await connect(`/rooms/${room}/ws`);
    expect((await host.nextFrame()).kind).toBe(0);
    again.ws.close(1000);
    gs.forEach((g) => g.ws.close(1000));
    host.ws.close(1000);
  });

  it("closes a guest sending more than 64 KiB and accepts exactly 64 KiB", async () => {
    const { room, host } = await hostedRoom();
    const g = await connect(`/rooms/${room}/ws`);
    await host.nextFrame();
    g.send("a".repeat(64 * 1024));
    const ok = await host.nextFrame();
    expect(ok).toMatchObject({ conn: 1, kind: 1 });
    expect(ok.payload.length).toBe(64 * 1024);
    g.send("a".repeat(64 * 1024 + 1));
    expect((await g.waitClosed()).code).toBe(1009);
    expect(await host.nextFrame()).toMatchObject({ conn: 1, kind: 2 });
    host.ws.close(1000);
  });

  it("passes 1 MiB host-to-guest and closes a tunnel that sends more", async () => {
    const { room, host } = await hostedRoom();
    const g = await connect(`/rooms/${room}/ws`);
    await host.nextFrame();
    const big = "b".repeat(1024 * 1024);
    host.send(frame(1, 1, big));
    expect(((await g.next()) as string).length).toBe(1024 * 1024);
    host.send(frame(1, 1, big + "c"));
    expect((await host.waitClosed()).code).toBe(1009);
    expect((await g.waitClosed()).code).toBe(1012);
  });
});

describe("tunnel lifecycle", () => {
  it("closes guests with 1013 when no host is connected", async () => {
    const { j } = await create();
    const g = await connect(`/rooms/${j.room}/ws`);
    expect((await g.waitClosed()).code).toBe(1013);
  });

  it("replaces the tunnel: old tunnel and guests close with 1012, new tunnel works", async () => {
    const { room, host_key, host: old } = await hostedRoom();
    const g = await connect(`/rooms/${room}/ws`);
    await old.nextFrame();
    const fresh = await connect(`/rooms/${room}/host?key=${host_key}`);
    expect((await old.waitClosed()).code).toBe(1012);
    expect((await g.waitClosed()).code).toBe(1012);
    // the room is still listed and still has a host
    expect((await lobbyRooms()).some((x) => x.room === room)).toBe(true);
    const g2 = await connect(`/rooms/${room}/ws`);
    expect(await fresh.nextFrame()).toMatchObject({ conn: 1, kind: 0 });
    g2.send("hi");
    expect(await fresh.nextFrame()).toMatchObject({ conn: 1, kind: 1, payload: "hi" });
    g2.ws.close(1000);
    fresh.ws.close(1000);
  });

  it("host drop closes guests, hides the room, keeps it 120 s, and a returning host brings it back", async () => {
    const { room, host_key, host } = await hostedRoom();
    const g = await connect(`/rooms/${room}/ws`);
    await host.nextFrame();
    host.ws.close(1000);
    expect((await g.waitClosed()).code).toBe(1012);
    for (let i = 0; i < 50 && (await lobbyRooms()).some((x) => x.room === room); i++) await new Promise((r) => setTimeout(r, 20));
    expect((await lobbyRooms()).some((x) => x.room === room)).toBe(false);
    // new guests get 1013 meanwhile
    expect((await (await connect(`/rooms/${room}/ws`)).waitClosed()).code).toBe(1013);
    // a stray alarm before the host returns must not matter once it is back
    const back = await connect(`/rooms/${room}/host?key=${host_key}`);
    expect(await alarm(room)).toBe(false); // alarm was cleared on return
    expect((await lobbyRooms()).some((x) => x.room === room)).toBe(true);
    back.ws.close(1000);
  });

  it("deletes the room when the alarm fires with no host", async () => {
    const { room, host_key, host } = await hostedRoom();
    host.ws.close(1000);
    await host.waitClosed();
    for (let i = 0; i < 50 && (await lobbyRooms()).some((x) => x.room === room); i++) await new Promise((r) => setTimeout(r, 20));
    expect(await alarm(room)).toBe(true);
    expect((await lobbyRooms()).some((x) => x.room === room)).toBe(false);
    expect((await call(`/rooms/${room}/api/status`)).status).toBe(404);
    const r = await call(`/rooms/${room}/host?key=${host_key}`, { headers: { Upgrade: "websocket" } });
    expect(r.status).toBe(404);
  });

  it("deletes a room whose host never connected", async () => {
    const { j } = await create();
    expect(await alarm(j.room)).toBe(true);
    expect((await call(`/rooms/${j.room}/api/status`)).status).toBe(404);
  });

  it("deletes the room at once on bye and closes guests", async () => {
    const { room, host_key, host } = await hostedRoom();
    const g = await connect(`/rooms/${room}/ws`);
    await host.nextFrame();
    host.send(frame(0, 5));
    expect((await g.waitClosed()).code).toBe(1012);
    for (let i = 0; i < 50 && (await lobbyRooms()).some((x) => x.room === room); i++) await new Promise((r) => setTimeout(r, 20));
    expect((await lobbyRooms()).some((x) => x.room === room)).toBe(false);
    expect((await call(`/rooms/${room}/api/status`)).status).toBe(404);
    const r = await call(`/rooms/${room}/host?key=${host_key}`, { headers: { Upgrade: "websocket" } });
    expect(r.status).toBe(404);
  });
});

// Last on purpose: it fills the relay, and storage is not reset between tests.
describe("room cap", () => {
  it("caps the number of rooms at 64", async () => {
    const before = (await lobbyRooms()).length; // hosted rooms only; unhosted rooms count too, so create until refused
    let created = 0;
    let refused: Response | null = null;
    for (let i = 0; i < 70; i++) {
      const { r } = await create(announce(`cap-${i}`));
      if (r.status === 503) {
        refused = r;
        break;
      }
      created++;
    }
    expect(refused).not.toBeNull();
    expect(created + before).toBeLessThanOrEqual(64);
    expect(created).toBeGreaterThan(0);
  });
});
