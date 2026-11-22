// The relay's abuse limits: names, per-address rooms, create rate, kill switch, socket rates, heartbeat.
import { runInDurableObject } from "cloudflare:test";
import { describe, expect, it } from "vitest";
import {
  CLOSE_RATE,
  DEFAULT_LIMITS,
  GUEST_RATE,
  HEARTBEAT_MS,
  HOST_RATE,
  MSG_LOBBY_CLOSED,
  MSG_LOBBY_FULL,
  MSG_TOO_FAST,
  REASON_TOO_MANY_FROM_IP,
  REASON_TOO_MANY_MESSAGES,
  REASON_TOO_MUCH_DATA,
  SocketRate,
  addressKey,
  defaultName,
  ipHash,
  limitsFrom,
  msUntil,
  msgTooManyRooms,
  sanitizeName,
  takeToken,
} from "../src/shared";
import { Sock, alarm, announce, callWith, connect, create, frame, freshIp, hostedRoom, lobbyRooms, lobbyStub, waitFor } from "./helpers";

const listed = async (room: string) => (await lobbyRooms()).some((x) => x.room === room);
const post = (vars: Record<string, string>, body: unknown, ip?: string) =>
  callWith(vars, "/rooms", { method: "POST", body: JSON.stringify(body), ip });

describe("room names", () => {
  it("keeps letters of any script, digits, spaces and a little punctuation", () => {
    expect(sanitizeName("Sam's game #2 (casual)!")).toBe("Sam's game #2 (casual)!");
    expect(sanitizeName("Ζεύς και Ἥρα")).toBe("Ζεύς και Ἥρα");
    expect(sanitizeName("東京の試合")).toBe("東京の試合");
    expect(sanitizeName("नमस्ते दुनिया")).toBe("नमस्ते दुनिया");
    expect(sanitizeName("مباراة سام")).toBe("مباراة سام");
  });

  it("drops symbols, emoji, markup, invisible and direction characters", () => {
    expect(sanitizeName("<b>Hi</b> 🎉 $$$ ~~")).toBe("bHib");
    expect(sanitizeName("ab‮dc​⁦e­")).toBe("abdce");
    expect(sanitizeName("http://evil.example/x")).toBe("http:evil.examplex");
  });

  it("collapses whitespace and trims", () => {
    expect(sanitizeName("  a \t\n  b　 c  ")).toBe("a b c");
  });

  it("caps at 40 visible characters and limits stacked marks", () => {
    expect(sanitizeName("y".repeat(100))).toBe("y".repeat(40));
    expect([...new Intl.Segmenter(undefined, { granularity: "grapheme" }).segment(sanitizeName("é".repeat(60)))].length).toBe(40);
    const zalgo = "q" + "́̂̃̄̅̆̇".repeat(5) + "a"; // q + acute has no precomposed form
    expect(sanitizeName(zalgo)).toBe("q́̂̃a");
    expect(sanitizeName("́́x")).toBe("x"); // a mark with no letter under it
  });

  it("returns empty when nothing usable is left, and blocks listed terms", () => {
    expect(sanitizeName("🎉🎉 $$")).toBe("");
    expect(sanitizeName("   ")).toBe("");
    expect(sanitizeName("a nasty word here", ["nastyword"])).toBe("");
    expect(sanitizeName("N.A.S.T.Y", ["nasty"])).toBe("");
    expect(sanitizeName("fine name", ["nasty"])).toBe("fine name");
    expect(sanitizeName("anything", [""])).toBe("anything");
  });

  it("names a room with no usable name Game <id>", async () => {
    const { name: _omitted, ...noName } = announce();
    for (const body of [noName, announce(null), announce(""), announce("🎉🎉")]) {
      const { r, j } = await create(body);
      expect(r.status).toBe(200);
      expect(j.name).toBe(defaultName(j.room));
      expect(j.name).toMatch(/^Game [A-Z0-9]{4}$/);
    }
    const { room, host } = await hostedRoom("<script>x</script>  ok");
    const row = (await lobbyRooms()).find((x) => x.room === room)!;
    expect(row.name).toBe("scriptxscript ok");
    host.ws.close(1000);
  });
});

describe("token bucket and address hashing", () => {
  it("allows a burst, refuses past it, refills over the window", () => {
    let b = null;
    for (let i = 0; i < 6; i++) {
      const t = takeToken(b, 1000, 1, 6, 600_000);
      expect(t.ok).toBe(true);
      b = t.bucket;
    }
    const no = takeToken(b, 1000, 1, 6, 600_000);
    expect(no.ok).toBe(false);
    expect(msUntil(no.bucket, 1, 6, 600_000)).toBe(100_000);
    expect(takeToken(no.bucket, 1000 + 99_999, 1, 6, 600_000).ok).toBe(false);
    expect(takeToken(no.bucket, 1000 + 100_000, 1, 6, 600_000).ok).toBe(true);
    const full = takeToken(no.bucket, 1000 + 10 * 600_000, 0, 6, 600_000);
    expect(full.bucket.tokens).toBe(6); // never more than the capacity
  });

  it("caps a socket's messages and bytes", () => {
    const r = new SocketRate({ msgBurst: 3, msgPerSec: 1, byteBurst: 100, bytePerSec: 10 });
    expect(r.take(10, 0)).toBeNull();
    expect(r.take(10, 0)).toBeNull();
    expect(r.take(10, 0)).toBeNull();
    expect(r.take(10, 0)).toBe(REASON_TOO_MANY_MESSAGES);
    expect(r.take(10, 1000)).toBeNull();
    expect(r.take(100, 3000)).toBe(REASON_TOO_MUCH_DATA); // 70 left at 1 s, refilled to 90 by 3 s
    expect(r.take(90, 3000)).toBeNull();
  });

  it("limits an IPv6 line as one address: its first 64 bits", async () => {
    expect(addressKey("203.0.113.9")).toBe("203.0.113.9");
    expect(addressKey("::ffff:203.0.113.9")).toBe("203.0.113.9");
    expect(addressKey("2001:db8:12:34:aaaa:bbbb:cccc:dddd")).toBe("2001:db8:12:34::/64");
    expect(addressKey("2001:0DB8:0012:0034::1")).toBe("2001:db8:12:34::/64");
    expect(addressKey("2001:db8::1")).toBe("2001:db8:0:0::/64");
    expect(addressKey("2001:db8:12:35::1")).not.toBe(addressKey("2001:db8:12:34::1"));
    expect(addressKey("::1")).toBe("0:0:0:0::/64");
    expect(await ipHash("s", "2001:db8:12:34::1")).toBe(await ipHash("s", "2001:db8:12:34:ffff:ffff:ffff:ffff"));

    // The same line under three of its addresses still holds only MAX_ROOMS_PER_IP rooms.
    const net = (Date.now() % 60000).toString(16);
    const line = (n: number) => `2001:db8:77:${net}::${n}`;
    for (let i = 1; i <= DEFAULT_LIMITS.MAX_ROOMS_PER_IP; i++) expect((await create(announce(`v6 ${i}`), line(i))).r.status).toBe(200);
    expect((await create(announce("v6 more"), line(99))).r.status).toBe(429);
  });

  it("hashes an address with a salt and never keeps it in the clear", async () => {
    const a = await ipHash("salt-1", "203.0.113.9");
    expect(a).toMatch(/^[0-9a-f]{32}$/);
    expect(a).toBe(await ipHash("salt-1", "203.0.113.9"));
    expect(a).not.toBe(await ipHash("salt-2", "203.0.113.9"));
    expect(a).not.toBe(await ipHash("salt-1", "203.0.113.10"));

    const ip = "203.0.113.77";
    await create(announce("hash check"), ip);
    const dump = await runInDurableObject(lobbyStub(), (_i, state: DurableObjectState) => {
      const rows = [
        ...state.storage.sql.exec("SELECT * FROM rooms").toArray(),
        ...state.storage.sql.exec("SELECT * FROM buckets").toArray(),
      ];
      return JSON.stringify(rows);
    });
    expect(dump).not.toContain(ip);
    expect(dump).toContain('"ip_hash"');
  });

  it("reads tunables from vars and ignores bad values", () => {
    expect(limitsFrom({})).toEqual(DEFAULT_LIMITS);
    expect(limitsFrom({ MAX_ROOMS: "10", MAX_ROOMS_PER_IP: " 3 ", ROOM_CREATES_PER_10_MIN: "x", MAX_GUESTS_PER_IP: "0" })).toEqual({
      ...DEFAULT_LIMITS,
      MAX_ROOMS: 10,
      MAX_ROOMS_PER_IP: 3,
    });
  });
});

describe("room creation limits", () => {
  it("allows 2 live rooms per address, and frees a slot when a room expires", async () => {
    const ip = freshIp();
    const a = await create(announce("one"), ip);
    const b = await create(announce("two"), ip);
    expect(a.r.status).toBe(200);
    expect(b.r.status).toBe(200);
    const c = await create(announce("three"), ip);
    expect(c.r.status).toBe(429);
    expect(c.j.error).toBe(msgTooManyRooms(2));
    expect(c.j.error).toBe("You already have 2 games open. Close one before hosting another.");
    // another address is not affected
    expect((await create(announce("other"))).r.status).toBe(200);
    // the unclaimed room expires: its slot is free again
    expect(await alarm(a.j.room)).toBe(true);
    expect((await create(announce("four"), ip)).r.status).toBe(200);
  });

  it("frees a slot when the host says bye", async () => {
    const ip = freshIp();
    const { host } = await hostedRoom("bye one", ip);
    await hostedRoom("bye two", ip);
    expect((await create(announce("refused"), ip)).r.status).toBe(429);
    host.send(frame(0, 5));
    await host.waitClosed();
    let status = 0;
    await waitFor(async () => (status = (await create(announce("after bye"), ip)).r.status) === 200);
    expect(status).toBe(200);
  });

  it("allows 6 creations per address per 10 minutes, then answers 429 with Retry-After", async () => {
    const ip = freshIp();
    const vars = { MAX_ROOMS_PER_IP: "100" };
    for (let i = 0; i < 6; i++) expect((await post(vars, announce(`burst ${i}`), ip)).status).toBe(200);
    const r = await post(vars, announce("burst 7"), ip);
    expect(r.status).toBe(429);
    expect(((await r.json()) as { error: string }).error).toBe(MSG_TOO_FAST);
    const after = Number(r.headers.get("Retry-After"));
    expect(after).toBeGreaterThan(0);
    expect(after).toBeLessThanOrEqual(100);
    expect(r.headers.get("access-control-allow-origin")).toBe("*");
    // a page on another origin can read Retry-After only when the relay exposes it
    expect(r.headers.get("access-control-expose-headers")).toMatch(/\bRetry-After\b/i);
    // a refused attempt does not spend a token, and other addresses are not affected
    expect((await post(vars, announce("elsewhere"), freshIp())).status).toBe(200);
  });

  it("only successful creations count toward the rate", async () => {
    const ip = freshIp();
    await create(announce("a"), ip);
    await create(announce("b"), ip);
    for (let i = 0; i < 10; i++) expect((await create(announce("refused"), ip)).r.status).toBe(429);
    // still 4 tokens left: raise the live cap and create 4 more
    for (let i = 0; i < 4; i++) expect((await post({ MAX_ROOMS_PER_IP: "100" }, announce(`more ${i}`), ip)).status).toBe(200);
    expect((await post({ MAX_ROOMS_PER_IP: "100" }, announce("too many"), ip)).status).toBe(429);
  });

  it("honours MAX_ROOMS from vars", async () => {
    await create(announce("someone"));
    const r = await post({ MAX_ROOMS: "1" }, announce("refused"));
    expect(r.status).toBe(503);
    expect(((await r.json()) as { error: string }).error).toBe(MSG_LOBBY_FULL);
  });

  it("kill switch: LOBBY_OPEN=0 refuses new rooms while running rooms carry on", async () => {
    const { room, host } = await hostedRoom("still running");
    const r = await post({ LOBBY_OPEN: "0" }, announce("new"));
    expect(r.status).toBe(503);
    expect(((await r.json()) as { error: string }).error).toBe(MSG_LOBBY_CLOSED);
    expect((await callWith({ LOBBY_OPEN: "0" }, "/lobby").then((x) => x.json()) as { rooms: { room: string }[] }).rooms.some((x) => x.room === room)).toBe(true);
    const g = new Sock((await callWith({ LOBBY_OPEN: "0" }, `/rooms/${room}/ws`, { headers: { Upgrade: "websocket" } })).webSocket!);
    expect(await host.nextFrame()).toMatchObject({ conn: 1, kind: 0 });
    g.send("still here");
    expect(await host.nextFrame()).toMatchObject({ conn: 1, kind: 1, payload: "still here" });
    expect((await post({ LOBBY_OPEN: "1" }, announce("open again"))).status).toBe(200);
    g.ws.close(1000);
    host.ws.close(1000);
  });
});

describe("socket limits", () => {
  it("allows 4 guest sockets per address in a room, closing the fifth with 1013", async () => {
    const { room, host } = await hostedRoom();
    const ip = freshIp();
    const mine: Sock[] = [];
    for (let i = 0; i < 4; i++) mine.push(await connect(`/rooms/${room}/ws`, ip));
    for (let i = 0; i < 4; i++) expect((await host.nextFrame()).kind).toBe(0);
    const fifth = await connect(`/rooms/${room}/ws`, ip);
    expect(await fifth.waitClosed()).toEqual({ code: 1013, reason: REASON_TOO_MANY_FROM_IP });
    const other = await connect(`/rooms/${room}/ws`, freshIp());
    expect((await host.nextFrame()).kind).toBe(0);
    mine[0]!.ws.close(1000);
    expect((await host.nextFrame()).kind).toBe(2);
    const again = await connect(`/rooms/${room}/ws`, ip);
    expect((await host.nextFrame()).kind).toBe(0);
    [...mine, other, again].forEach((g) => g.ws.close(1000));
    host.ws.close(1000);
  });

  it("closes a guest that sends too many messages with 4029", async () => {
    const { room, host } = await hostedRoom();
    const g = await connect(`/rooms/${room}/ws`);
    await host.nextFrame();
    for (let i = 0; i < GUEST_RATE.msgBurst + 100; i++) g.send(`{"type":"ping","i":${i}}`);
    expect(await g.waitClosed()).toEqual({ code: CLOSE_RATE, reason: REASON_TOO_MANY_MESSAGES });
    // the host saw the burst, then a close for that guest
    await host.until(() => host.q.some((m) => new DataView(m as ArrayBuffer).getUint8(4) === 2), "guest close");
    const data = host.q.filter((m) => new DataView(m as ArrayBuffer).getUint8(4) === 1).length;
    expect(data).toBeGreaterThanOrEqual(GUEST_RATE.msgBurst);
    expect(data).toBeLessThan(GUEST_RATE.msgBurst + 100);
    host.ws.close(1000);
  });

  it("closes a guest that sends too many bytes with 4029", async () => {
    const { room, host } = await hostedRoom();
    const g = await connect(`/rooms/${room}/ws`);
    await host.nextFrame();
    const big = "d".repeat(64 * 1024);
    const n = Math.ceil((GUEST_RATE.byteBurst * 2) / big.length); // well past the burst, within the message cap
    for (let i = 0; i < n; i++) g.send(big);
    expect(await g.waitClosed()).toEqual({ code: CLOSE_RATE, reason: REASON_TOO_MUCH_DATA });
    host.ws.close(1000);
  });

  it("closes a host tunnel that sends too many frames with 4029, and its guests with 1012", async () => {
    const { room, host } = await hostedRoom();
    const g = await connect(`/rooms/${room}/ws`);
    await host.nextFrame();
    const ping = frame(0, 6);
    for (let i = 0; i < HOST_RATE.msgBurst + 1500; i++) host.send(ping);
    await host.until(() => host.closed !== null, "close", 600);
    expect(host.closed).toEqual({ code: CLOSE_RATE, reason: REASON_TOO_MANY_MESSAGES });
    expect((await g.waitClosed()).code).toBe(1012);
    await waitFor(async () => !(await listed(room)));
    expect(await listed(room)).toBe(false);
  }, 60_000);

  it("closes a host tunnel that sends too many bytes with 4029", async () => {
    const { host } = await hostedRoom();
    const mib = frame(0, 7, new Uint8Array(1024 * 1024 - 5)); // unknown kind: the relay drops it
    const n = Math.ceil((HOST_RATE.byteBurst * 2) / (1024 * 1024));
    for (let i = 0; i < n && !host.closed; i++) host.send(mib);
    expect(await host.waitClosed()).toEqual({ code: CLOSE_RATE, reason: REASON_TOO_MUCH_DATA });
  });

  it("lets a heavy real turn through: 8 guests, chunked multi-MiB views, busy guests", async () => {
    const { room, host } = await hostedRoom();
    const gs: Sock[] = [];
    for (let i = 0; i < 8; i++) gs.push(await connect(`/rooms/${room}/ws`));
    for (let i = 0; i < 8; i++) expect((await host.nextFrame()).kind).toBe(0);
    // each guest submits a turn's worth of orders and chat
    for (const g of gs) for (let i = 0; i < 60; i++) g.send(`{"type":"orders","n":${i},"pad":"${"o".repeat(500)}"}`);
    // the host sends each guest a 3 MiB view in 12 chunks of 256 KiB, twice (turn result and a resync)
    const chunk = "v".repeat(256 * 1024);
    for (let round = 0; round < 2; round++) for (let c = 1; c <= 8; c++) for (let i = 0; i < 12; i++) host.send(frame(c, 1, chunk));
    for (const g of gs) await g.until(() => g.q.length >= 24, "views", 400);
    for (const g of gs) expect(g.closed).toBeNull();
    await host.until(() => host.q.filter((m) => new DataView(m as ArrayBuffer).getUint8(4) === 1).length >= 8 * 60, "orders", 400);
    expect(host.closed).toBeNull();
    gs.forEach((g) => g.ws.close(1000));
    host.ws.close(1000);
  });
});

describe("host heartbeat", () => {
  const heardMs = (room: string) =>
    runInDurableObject(lobbyStub(), (_i, state: DurableObjectState) =>
      state.storage.sql.exec("SELECT heard_ms FROM rooms WHERE room = ?", room).one().heard_ms as number | null,
    );

  it("unlists a pinging host that goes silent for 60 s, and lists it again when it is heard", async () => {
    const { room, host } = await hostedRoom("pinger");
    expect(await heardMs(room)).toBeNull();
    host.send(frame(0, 6)); // the browser host's ping frame
    await waitFor(async () => (await heardMs(room)) !== null);
    expect(await heardMs(room)).toBeGreaterThan(Date.now() - 5000);
    expect(await listed(room)).toBe(true);

    const lobby = lobbyStub() as unknown as { heard(room: string, at: number): Promise<void> };
    await lobby.heard(room, Date.now() - HEARTBEAT_MS - 1000);
    expect(await listed(room)).toBe(false);
    await lobby.heard(room, Date.now());
    expect(await listed(room)).toBe(true);
    // a ping frame is not forwarded anywhere and gets no reply
    expect(host.q.length).toBe(0);
    host.ws.close(1000);
  });

  it("does not hold a host that never pings to the heartbeat (WebSocket pings are invisible to the room)", async () => {
    const { room, host } = await hostedRoom("desktop");
    host.send(frame(0, 4, JSON.stringify({ players: 2, humans: 1, open_seats: 1, phase: "staging", turn: 0 })));
    await waitFor(async () => (await lobbyRooms()).find((x) => x.room === room)?.players === 2);
    expect(await heardMs(room)).toBeNull();
    expect(await listed(room)).toBe(true);
    host.ws.close(1000);
  });

  it("a new tunnel starts with a fresh heartbeat", async () => {
    const { room, host_key, host } = await hostedRoom("reconnects");
    host.send(frame(0, 6));
    await waitFor(async () => (await heardMs(room)) !== null);
    const lobby = lobbyStub() as unknown as { heard(room: string, at: number): Promise<void> };
    await lobby.heard(room, Date.now() - HEARTBEAT_MS - 1000);
    expect(await listed(room)).toBe(false);
    const fresh = await connect(`/rooms/${room}/host?key=${host_key}`);
    expect(await listed(room)).toBe(true);
    expect(await heardMs(room)).toBeNull();
    fresh.ws.close(1000);
  });
});
