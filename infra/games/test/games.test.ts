import { env as workerEnv, exports } from "cloudflare:workers";
import { runInDurableObject } from "cloudflare:test";
import { describe, expect, it } from "vitest";
import worker, { MAX_GZIP_BYTES, checkRecord, originAllowed, tokenMatches, type Env } from "../src/index";
import { MAX_NEW_GAMES_PER_NETWORK_PER_DAY, addressKey, capFrom, ipHash, networkKey } from "../src/quota";

const env = workerEnv as unknown as Env;

const BASE = "https://games.test";
const ORIGIN = "https://legatum-game.pages.dev";
const ID = "web-1795310626069-619872b4c1b6c000-89f1f0f11637b000";

const call = (path: string, init?: RequestInit) =>
  (exports as unknown as { default: { fetch(u: string, i?: RequestInit): Promise<Response> } }).default.fetch(BASE + path, init);

const record = (seats = [
  { player: 0, name: "Player 1", faction: "Athens", difficulty: "Normal", human: true },
  { player: 1, name: "AI 2", faction: "Celts", difficulty: "Hard", human: false },
]) => ({
  metadata: { schema_version: 1, game_id: ID, created_at: "2026-11-22T01:23:46Z", outcome: "abandoned", turn_count: 145, engine_commit: "0123456789abcdef0123456789abcdef01234567", seats, winner: null },
  tape: { version: 1, turns: [] },
});

async function gzip(text: string): Promise<ArrayBuffer> {
  return new Response(new Blob([text]).stream().pipeThrough(new CompressionStream("gzip"))).arrayBuffer();
}

const put = async (body: unknown, origin = ORIGIN, id = ID) =>
  call(`/v1/games/${id}`, { method: "PUT", headers: { Origin: origin }, body: await gzip(JSON.stringify(body)) });

describe("game uploads", () => {
  it("stores a game under its month and replaces earlier checkpoints", async () => {
    const r = await put(record());
    expect(r.status).toBe(200);
    expect(r.headers.get("Access-Control-Allow-Origin")).toBe(ORIGIN);
    const key = `games/2026-11/${ID}.json.gz`;
    expect(await r.json()).toEqual({ stored: key });
    const stored = await env.GAMES.get(key);
    expect(stored?.customMetadata).toMatchObject({ outcome: "abandoned", turns: "145", seats: "Athens*:Normal Celts:Hard" });
    expect((await put(record())).status).toBe(200);
    expect((await env.GAMES.list({ prefix: "games/" })).objects.filter((o) => o.key === key)).toHaveLength(1);
  });

  it("accepts coach answers as a short list of objects and refuses anything else", async () => {
    const answers = [{ type: "answer", turn: 12, answer: "Their Settler was alone." }];
    expect((await put({ ...record(), coach_answers: answers })).status).toBe(200);
    expect((await put({ ...record(), coach_answers: "text" })).status).toBe(422);
    expect((await put({ ...record(), coach_answers: ["text"] })).status).toBe(422);
    expect((await put({ ...record(), coach_answers: Array.from({ length: 2001 }, () => ({})) })).status).toBe(422);
  });

  it("stores a winning team and refuses a winner that is not one", async () => {
    const withWinner = (winner: unknown) => {
      const r = record();
      return { ...r, metadata: { ...r.metadata, outcome: "vp", winner } };
    };
    expect((await put(withWinner(0))).status).toBe(200);
    expect((await env.GAMES.get(`games/2026-11/${ID}.json.gz`))?.customMetadata?.winner).toBe("0");
    for (const bad of ["x", "1", { a: 1 }, -1, 1.5, 256, "x".repeat(5000)]) {
      expect((await put(withWinner(bad))).status).toBe(422);
    }
  });

  it("accepts rewinds as a short list of earlier-turn jumps and refuses anything else", async () => {
    const rewind = { at: "2026-11-22T19:10:00.000Z", from_turn: 42, to_turn: 39 };
    expect((await put({ ...record(), rewinds: [rewind] })).status).toBe(200);
    expect((await put({ ...record(), rewinds: "42" })).status).toBe(422);
    expect((await put({ ...record(), rewinds: [{ ...rewind, to_turn: 42 }] })).status).toBe(422);
    expect((await put({ ...record(), rewinds: [{ ...rewind, from_turn: 1.5 }] })).status).toBe(422);
    expect((await put({ ...record(), rewinds: [{ ...rewind, at: "yesterday" }] })).status).toBe(422);
    expect((await put({ ...record(), rewinds: [{ ...rewind, note: "misclick" }] })).status).toBe(422);
    expect((await put({ ...record(), rewinds: Array.from({ length: 1001 }, () => rewind) })).status).toBe(422);
  });

  it("refuses typed names, other sites, bad ids and non-gzip bodies", async () => {
    const typed = record();
    typed.metadata.seats[0]!.name = "Dana";
    expect((await put(typed)).status).toBe(422);
    const odd = record();
    odd.metadata.seats[1]!.faction = 'Celts" & do shell script "x';
    expect((await put(odd)).status).toBe(422);
    const date = record();
    date.metadata.created_at = "2026-99-22T01:23:46Z";
    expect((await put(date)).status).toBe(422);
    const long = { ...record(), coach_answers: [{ type: "answer", turn: 3, answer: "x".repeat(2001) }] };
    expect((await put(long)).status).toBe(422);
    const extra = { ...record(), coach_answers: [{ type: "answer", turn: 3, answer: "ok", secret: "no" }] };
    expect((await put(extra)).status).toBe(422);
    const fine = { ...record(), coach_answers: [{ type: "answer", game: "g", turn: 3, category: "settle", question: "Why?", evidence: { a: 1 }, answer: "ok", build: "b" }] };
    expect((await put(fine)).status).toBe(200);
    expect((await put(record(), "https://evil.example")).status).toBe(403);
    expect((await put(record(), ORIGIN, "web-1-x")).status).toBe(404);
    const big = new Uint8Array(MAX_GZIP_BYTES + 1);
    expect((await call(`/v1/games/${ID}`, { method: "PUT", headers: { Origin: ORIGIN }, body: big })).status).toBe(413);
    const plain = await call(`/v1/games/${ID}`, { method: "PUT", headers: { Origin: ORIGIN }, body: JSON.stringify(record()) });
    expect(plain.status).toBe(400);
    const get = await call(`/v1/games/${ID}`, { headers: { Origin: ORIGIN } });
    expect(get.status).toBe(405);
  });

  it("locks a game to the upload key it was first stored with", async () => {
    const id = "web-1795310626070-0000000000000001-0000000000000002";
    const body = { ...record(), metadata: { ...record().metadata, game_id: id } };
    const keyed = async (key?: string) =>
      call(`/v1/games/${id}`, {
        method: "PUT",
        headers: { Origin: ORIGIN, ...(key === undefined ? {} : { "X-Upload-Key": key }) },
        body: await gzip(JSON.stringify(body)),
      });
    const mine = "a".repeat(32);
    expect((await keyed("not hex")).status).toBe(400);
    expect((await keyed(mine)).status).toBe(200);
    const stored = await env.GAMES.head(`games/2026-11/${id}.json.gz`);
    expect(stored?.customMetadata?.key).toMatch(/^[0-9a-f]{64}$/);
    expect(stored?.customMetadata?.key).not.toContain(mine);
    // Someone who knows the id but not the key cannot replace the record, with another key or with none.
    const other = await keyed("b".repeat(32));
    expect(other.status).toBe(409);
    expect(other.headers.get("Access-Control-Allow-Origin")).toBe(ORIGIN);
    expect((await keyed()).status).toBe(409);
    expect((await keyed(mine)).status).toBe(200);
    expect((await env.GAMES.head(`games/2026-11/${id}.json.gz`))?.customMetadata?.key).toBe(stored?.customMetadata?.key);
  });

  it("keeps a game from an older build open until a keyed checkpoint locks it", async () => {
    const id = "web-1795310626071-0000000000000003-0000000000000004";
    const body = { ...record(), metadata: { ...record().metadata, game_id: id } };
    const send = async (key?: string) =>
      call(`/v1/games/${id}`, {
        method: "PUT",
        headers: { Origin: ORIGIN, ...(key === undefined ? {} : { "X-Upload-Key": key }) },
        body: await gzip(JSON.stringify(body)),
      });
    expect((await send()).status).toBe(200);
    expect((await send()).status).toBe(200);
    expect((await env.GAMES.head(`games/2026-11/${id}.json.gz`))?.customMetadata?.key).toBeUndefined();
    expect((await send("c".repeat(32))).status).toBe(200);
    expect((await send()).status).toBe(409);
  });

  it("answers the CORS preflight for allowed pages only", async () => {
    const ok = await call(`/v1/games/${ID}`, { method: "OPTIONS", headers: { Origin: "http://localhost:5173" } });
    expect(ok.status).toBe(204);
    expect(ok.headers.get("Access-Control-Allow-Methods")).toContain("PUT");
    expect(ok.headers.get("Access-Control-Allow-Headers")).toContain("X-Upload-Key");
    expect((await call(`/v1/games/${ID}`, { method: "OPTIONS", headers: { Origin: "https://x.example" } })).status).toBe(403);
  });
});

describe("new games per address per day", () => {
  // Distinct game ids: 13-digit time, then two 16-hex parts.
  const gid = (n: number) => `web-1795310626069-${n.toString(16).padStart(16, "0")}-89f1f0f11637b000`;
  const rec = (id: string) => { const r = record(); r.metadata.game_id = id; return r; };
  let lineNo = 0;
  const line = () => `198.51.100.${++lineNo}`;
  const base = 0x1000 + Math.floor(Math.random() * 0xfffff) * 100;
  let next = 0;
  const fresh = () => gid(base + ++next);
  const putFrom = async (ip: string, id: string, cap = "2") => {
    const body = await gzip(JSON.stringify(rec(id)));
    return worker.fetch(
      new Request(`${BASE}/v1/games/${id}`, { method: "PUT", headers: { Origin: ORIGIN, "CF-Connecting-IP": ip }, body }),
      Object.assign({}, env, { MAX_NEW_GAMES_PER_DAY: cap }) as Env,
    );
  };
  const inR2 = async (id: string) => (await env.GAMES.head(`games/2026-11/${id}.json.gz`)) !== null;

  it("refuses the new game past the cap with 429 and Retry-After, and stores nothing", async () => {
    const ip = line();
    const a = fresh(), b = fresh(), c = fresh();
    expect((await putFrom(ip, a)).status).toBe(200);
    expect((await putFrom(ip, b)).status).toBe(200);
    const r = await putFrom(ip, c);
    expect(r.status).toBe(429);
    expect(await r.json()).toEqual({ error: "too many new games today" });
    const wait = Number(r.headers.get("Retry-After"));
    expect(wait).toBeGreaterThanOrEqual(1);
    expect(wait).toBeLessThanOrEqual(86_400);
    expect(r.headers.get("Access-Control-Allow-Origin")).toBe(ORIGIN);
    expect(await inR2(c)).toBe(false);
  });

  it("lets a stored game keep checkpointing at the cap", async () => {
    const ip = line();
    const a = fresh(), b = fresh();
    expect((await putFrom(ip, a)).status).toBe(200);
    expect((await putFrom(ip, b)).status).toBe(200);
    expect((await putFrom(ip, a)).status).toBe(200);
    expect((await putFrom(ip, b)).status).toBe(200);
    expect((await putFrom(ip, fresh())).status).toBe(429);
  });

  it("counts each address on its own, and an IPv6 /64 as one", async () => {
    const one = line(), two = line();
    await putFrom(one, fresh());
    await putFrom(one, fresh());
    expect((await putFrom(one, fresh())).status).toBe(429);
    expect((await putFrom(two, fresh())).status).toBe(200);
    const net = Math.floor(Math.random() * 60000).toString(16);
    expect((await putFrom(`2001:db8:55:${net}::1`, fresh())).status).toBe(200);
    expect((await putFrom(`2001:db8:55:${net}:aaaa:bbbb:cccc:dddd`, fresh())).status).toBe(200);
    expect((await putFrom(`2001:db8:55:${net}::99`, fresh())).status).toBe(429);
    expect((await putFrom(`2001:db8:55:${net}1::1`, fresh())).status).toBe(200);
  });

  it("honours the var, and a bad var means 50", async () => {
    const ip = line();
    expect((await putFrom(ip, fresh(), "1")).status).toBe(200);
    expect((await putFrom(ip, fresh(), "1")).status).toBe(429);
    expect(capFrom(undefined)).toBe(50);
    for (const bad of ["", "0", "-3", "many", "2.5", "1e3"]) expect(capFrom(bad)).toBe(50);
    expect(capFrom("7")).toBe(7);
  });

  it("keeps only a salted hash for today, never the address", async () => {
    const ip = "203.0.113.77";
    await putFrom(ip, fresh());
    const stub = env.QUOTA.getByName("quota");
    const dump = await runInDurableObject(stub, async (_obj, state) => ({
      counts: state.storage.sql.exec("SELECT * FROM counts").toArray(),
      kv: state.storage.sql.exec("SELECT * FROM kv").toArray(),
    }));
    expect(JSON.stringify(dump)).not.toContain(ip);
    expect(JSON.stringify(dump)).not.toContain("203.0.113");
    const salt = String(dump.kv.find((r) => r.k === "salt")?.v);
    expect(dump.counts.map((r) => String(r.ip_hash))).toContain(await ipHash(salt, ip));
    expect(dump.counts.every((r) => /^[0-9a-f]{32}$/.test(String(r.ip_hash)) && r.day === new Date().toISOString().slice(0, 10))).toBe(true);
  });

  it("drops rows of earlier days and counts the empty address as one", async () => {
    const stub = env.QUOTA.getByName("quota");
    const now = Date.UTC(2031, 0, 5, 23, 0, 0);
    expect(await stub.take("", 1, now)).toEqual({ ok: true });
    expect(await stub.take("", 1, now)).toEqual({ ok: false, retryAfter: 3600 });
    expect(await stub.take("", 1, now + 3_600_000)).toEqual({ ok: true });
    const days = await runInDurableObject(stub, async (_o, state) => state.storage.sql.exec("SELECT DISTINCT day FROM counts").toArray().map((r) => r.day));
    expect(days).toEqual(["2031-01-06"]);
  });

  it("refuses every address once all of them together reach the daily total", async () => {
    const stub = env.QUOTA.getByName("quota");
    const now = Date.UTC(2032, 2, 1, 12, 0, 0);
    expect(await stub.take("192.0.2.1", 50, now, 2)).toEqual({ ok: true });
    expect(await stub.take("192.0.2.2", 50, now, 2)).toEqual({ ok: true });
    expect(await stub.take("192.0.2.3", 50, now, 2)).toEqual({ ok: false, retryAfter: 43_200 });
    expect(await stub.take("192.0.2.3", 50, now + 43_200_000, 2)).toEqual({ ok: true });
    expect(capFrom(undefined, 50000)).toBe(50000);
    expect(capFrom("x", 50000)).toBe(50000);
  });

  it("counts a wider network on its own, so one host with many /64s cannot spend the total", async () => {
    const stub = env.QUOTA.getByName("quota");
    const now = Date.UTC(2033, 4, 1, 0, 0, 0);
    let ok = 0;
    for (let i = 0; i < 205; i++) if ((await stub.take(`2001:db8:77:${i.toString(16)}::1`, 50, now)).ok) ok++;
    expect(ok).toBe(MAX_NEW_GAMES_PER_NETWORK_PER_DAY);
    expect((await stub.take("2001:db8:78::1", 50, now)).ok).toBe(true);
    expect(networkKey("203.0.113.9")).toBe("203.0.113.0/24");
    expect(networkKey("2001:db8:12:34:aaaa::1")).toBe("2001:db8:12::/48");
    expect(networkKey("::ffff:203.0.113.9")).toBe("203.0.113.0/24");
  });

  it("stores a game under the month of its id, whatever created_at says", async () => {
    const id = "web-1795310626072-0000000000000005-0000000000000006";
    const r = record();
    r.metadata.game_id = id;
    r.metadata.created_at = "2026-10-30T09:00:00Z";
    const res = await put(r, ORIGIN, id);
    expect(res.status).toBe(200);
    expect(await res.json()).toEqual({ stored: `games/2026-11/${id}.json.gz` });
  });

  it("cuts addresses as the relay does", () => {
    expect(addressKey("203.0.113.9")).toBe("203.0.113.9");
    expect(addressKey("::ffff:203.0.113.9")).toBe("203.0.113.9");
    expect(addressKey("2001:db8:12:34:aaaa:bbbb:cccc:dddd")).toBe("2001:db8:12:34::/64");
    expect(addressKey("2001:0DB8:0012:0034::1")).toBe("2001:db8:12:34::/64");
    expect(addressKey("2001:db8::1")).toBe("2001:db8:0:0::/64");
    expect(addressKey("2001:db8:12:35::1")).not.toBe(addressKey("2001:db8:12:34::1"));
    expect(addressKey("::1")).toBe("0:0:0:0::/64");
  });

  it("hashes with a salt", async () => {
    const a = await ipHash("salt-1", "203.0.113.9");
    expect(a).toMatch(/^[0-9a-f]{32}$/);
    expect(a).not.toContain("203");
    expect(await ipHash("salt-1", "203.0.113.9")).toBe(a);
    expect(await ipHash("salt-2", "203.0.113.9")).not.toBe(a);
    expect(await ipHash("s", "2001:db8:12:34::1")).toBe(await ipHash("s", "2001:db8:12:34:ffff:ffff:ffff:ffff"));
  });
});

describe("origin patterns", () => {
  const allowed = env.ALLOWED_ORIGINS;
  it("matches the site, its previews and local dev", () => {
    expect(originAllowed("https://legatum-game.pages.dev", allowed)).toBe(true);
    expect(originAllowed("https://abc123.legatum-game.pages.dev", allowed)).toBe(true);
    expect(originAllowed("http://127.0.0.1:4173", allowed)).toBe(true);
    expect(originAllowed("https://legatum-game.pages.dev.evil.example", allowed)).toBe(false);
    expect(originAllowed("https://evillegatum-game.pages.dev", allowed)).toBe(false);
    expect(originAllowed(null, allowed)).toBe(false);
  });
});

describe("record check", () => {
  it("wants default names in seat order", () => {
    const swapped = record([
      { player: 0, name: "AI 1", faction: "Athens", difficulty: "Normal", human: true },
    ]);
    expect(checkRecord(ID, swapped)).toBe("seat names must be the defaults");
    expect(checkRecord("web-other", record())).toBe("game id does not match the address");
    expect(typeof checkRecord(ID, record())).toBe("object");
  });

  it("refuses engines, factions, difficulties and start times the game never writes", () => {
    const NOW = Date.UTC(2026, 10, 25);
    const edit = (f: (m: ReturnType<typeof record>["metadata"]) => void, id = ID) => {
      const r = record();
      r.metadata.game_id = id;
      f(r.metadata);
      return checkRecord(id, r, NOW);
    };
    expect(typeof edit(() => {})).toBe("object");
    expect(edit((m) => { m.engine_commit = "PENTESTPROBE"; })).toBe("unknown engine");
    expect(edit((m) => { m.engine_commit = "seedtest"; })).toBe("unknown engine");
    expect(edit((m) => { m.engine_commit = "0123456789ABCDEF0123456789ABCDEF01234567"; })).toBe("unknown engine");
    expect(edit((m) => { m.seats[1]!.faction = "Carthage"; })).toBe("bad faction or difficulty");
    expect(edit((m) => { m.seats[1]!.difficulty = "Deity"; })).toBe("bad faction or difficulty");
    // A start before launch, a start in the future, or a created_at that disagrees with the id's start time.
    const old = "web-1111111111111-deadbeefdeadbeef-c0ffee00c0ffee00";
    expect(edit((m) => { m.created_at = "2005-03-18T01:58:31Z"; }, old)).toBe("bad start time");
    const later = `web-${NOW + 2 * 86_400_000}-0000000000000001-0000000000000002`;
    expect(edit((m) => { m.created_at = new Date(NOW + 2 * 86_400_000).toISOString(); }, later)).toBe("bad start time");
    expect(edit((m) => { m.created_at = "2026-12-22T01:23:46Z"; })).toBe("bad start time");
    expect(edit((m) => { m.created_at = "2026-11-22T20:00:00Z"; })).not.toBe("bad start time");
    // A save loaded from a file gets a new id but keeps its older created_at (History::fork).
    expect(typeof edit((m) => { m.created_at = "2026-11-02T09:00:00Z"; })).toBe("object");
  });
});

describe("private read API", () => {
  const TOKEN = "t".repeat(40);
  const withToken = (e: Env) => { e.READ_TOKEN = TOKEN; };
  const read = (path: string, token = TOKEN) => call(path, { headers: { Authorization: `Bearer ${token}` } });

  it("lists and returns stored games for the token holder only", async () => {
    withToken(env);
    expect((await put(record())).status).toBe(200);
    const key = `games/2026-11/${ID}.json.gz`;
    const list = await read("/v1/read/games");
    expect(list.status).toBe(200);
    const body = await list.json() as { games: { key: string; outcome: string; turns: string }[]; cursor: string | null };
    expect(body.games.find((g) => g.key === key)).toMatchObject({ outcome: "abandoned", turns: "145" });
    expect(body.cursor).toBeNull();
    // A keyed game's stored key hash must not take the place of the object's name in the listing.
    expect(body.games.every((g) => g.key.startsWith("games/"))).toBe(true);
    const got = await read(`/v1/read/games/${encodeURIComponent(key)}`);
    expect(got.status).toBe(200);
    const text = await new Response((await got.blob()).stream().pipeThrough(new DecompressionStream("gzip"))).text();
    expect(JSON.parse(text).metadata.game_id).toBe(ID);
    expect((await read("/v1/read/games", "x".repeat(40))).status).toBe(404);
    expect((await call("/v1/read/games", { headers: { Origin: ORIGIN } })).status).toBe(404);
    expect((await read("/v1/read/games/games%2F..%2Fsecret")).status).toBe(404);
    expect((await read("/v1/read/games/%E0%A4%A")).status).toBe(404);
  });

  it("is off without a long enough secret", () => {
    expect(tokenMatches(`Bearer ${"a".repeat(40)}`, undefined)).toBe(false);
    expect(tokenMatches("Bearer short", "short")).toBe(false);
    expect(tokenMatches(`Bearer ${"a".repeat(40)}`, "a".repeat(40))).toBe(true);
    expect(tokenMatches(`Bearer ${"a".repeat(41)}`, "a".repeat(40))).toBe(false);
  });
});
