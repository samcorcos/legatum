import { describe, expect, it } from "vitest";
import { listedName } from "../src/shared";
import { Sock, alarm, announce, call, connect, create, frame, hostedRoom, lobbyRooms } from "./helpers";

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
    expect(row).toMatchObject({ name: listedName(j.room), build: "8.9.74", rules_version: 3, protocol: 1, phase: "staging", turn: 0 });
    expect(typeof row.updated_ms).toBe("number");
    host.ws.close(1000);
  });

  it("lists a locked flag: false by default, true when the host locked the game, and never a code", async () => {
    const open = await create(announce("open-game"));
    const locked = await create(announce("locked-game", { locked: true, code: "SECRET99" }));
    expect(open.r.status).toBe(200);
    expect(locked.r.status).toBe(200);
    const a = await connect(`/rooms/${open.j.room}/host?key=${open.j.host_key}`);
    const b = await connect(`/rooms/${locked.j.room}/host?key=${locked.j.host_key}`);
    const rooms = await lobbyRooms();
    expect(rooms.find((x) => x.room === open.j.room)!.locked).toBe(false);
    const row = rooms.find((x) => x.room === locked.j.room)!;
    expect(row.locked).toBe(true);
    expect(JSON.stringify(rooms)).not.toContain("SECRET99");
    a.ws.close(1000);
    b.ws.close(1000);
  });

  it("lists newest first", async () => {
    const a = await hostedRoom("first-xx");
    const b = await hostedRoom("second-x");
    const ids = (await lobbyRooms()).map((x) => x.room);
    expect(ids.indexOf(b.room)).toBeLessThan(ids.indexOf(a.room));
    a.host.ws.close(1000);
    b.host.ws.close(1000);
  });

  it("validates announcements strictly", async () => {
    const bad: unknown[] = [
      null,
      [],
      "x",
      announce(123),
      announce("x".repeat(201)),
      announce("ok", { build: "" }),
      announce("ok", { build: "8.9‮74" }),
      announce("ok", { build: "b".repeat(41) }),
      announce("ok", { build: "<b>1.0</b>" }),
      announce("ok", { build: "see http://evil.example" }),
      announce("ok", { build: " 1.0" }),
      announce("ok", { build: "Playtest the dev is a loser" }),
      announce("ok", { rules_version: "3" }),
      announce("ok", { protocol: -1 }),
      announce("ok", { protocol: 1.5 }),
      announce("ok", { locked: "yes" }),
      announce("ok", { locked: 1 }),
      announce("ok", { locked: null }),
      { name: "ok", build: "b", protocol: 1 },
    ];
    for (const b of bad) expect((await create(b)).r.status).toBe(400);
    expect((await call("/rooms", { method: "POST", body: "{not json" })).status).toBe(400);
    // A typed name is accepted from older pages and then dropped: the list names the room from its id.
    const long = await create(announce("x".repeat(80)));
    expect(long.r.status).toBe(200);
    expect(long.j.name).toBe(listedName(long.j.room));
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
    // A status far larger than a real one is dropped (the last good one stays), and answers say nosniff.
    host.send(frame(0, 3, JSON.stringify({ n: 3, pad: "x".repeat(70 * 1024) })));
    host.send(frame(0, 3, JSON.stringify({ n: 4 })));
    let last: Response | undefined;
    for (let i = 0; i < 50; i++) {
      last = await call(`/rooms/${room}/api/status`);
      const n = ((await last.clone().json()) as { n: number }).n;
      expect(n).not.toBe(3);
      if (n === 4) break;
      await new Promise((r) => setTimeout(r, 20));
    }
    expect(((await last!.json()) as { n: number }).n).toBe(4);
    expect(last!.headers.get("X-Content-Type-Options")).toBe("nosniff");
    host.ws.close(1000);
    await host.waitClosed();
    for (let i = 0; i < 50 && (await call(`/rooms/${room}/api/status`)).status !== 503; i++) await new Promise((r) => setTimeout(r, 20));
    expect((await call(`/rooms/${room}/api/status`)).status).toBe(503);
    expect((await call(`/rooms/${"q".repeat(16)}/api/status`)).status).toBe(404);
  });

  it("answers a room path with a trailing slash like the path without one", async () => {
    const { j } = await create();
    expect((await call(`/rooms/${j.room}/api/status/`)).status).toBe(503);
    const host = await connect(`/rooms/${j.room}/host/?key=${j.host_key}`);
    const guest = await connect(`/rooms/${j.room}/ws/`);
    expect(await host.nextFrame()).toMatchObject({ kind: 0 });
    guest.ws.close(1000);
    host.ws.close(1000);
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
    expect(row).toMatchObject({ name: listedName(room), ...s });
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
  it("caps the number of rooms at 256", async () => {
    const before = (await lobbyRooms()).length; // hosted rooms only; unhosted rooms count too, so create until refused
    let created = 0;
    let refused: Response | null = null;
    for (let i = 0; i < 270; i++) {
      const { r } = await create(announce(`cap-${i}`));
      if (r.status === 503) {
        refused = r;
        break;
      }
      created++;
    }
    expect(refused).not.toBeNull();
    expect(created + before).toBeLessThanOrEqual(256);
    expect(created).toBeGreaterThan(0);
  }, 60_000); // 256 creations take several seconds on a CI runner
});
