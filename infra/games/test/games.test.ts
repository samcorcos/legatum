import { env as workerEnv, exports } from "cloudflare:workers";
import { describe, expect, it } from "vitest";
import { checkRecord, originAllowed, tokenMatches, type Env } from "../src/index";

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
  metadata: { schema_version: 1, game_id: ID, created_at: "2026-11-22T01:23:46Z", outcome: "abandoned", turn_count: 145, engine_commit: "abc", seats, winner: null },
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
    const plain = await call(`/v1/games/${ID}`, { method: "PUT", headers: { Origin: ORIGIN }, body: JSON.stringify(record()) });
    expect(plain.status).toBe(400);
    const get = await call(`/v1/games/${ID}`, { headers: { Origin: ORIGIN } });
    expect(get.status).toBe(405);
  });

  it("answers the CORS preflight for allowed pages only", async () => {
    const ok = await call(`/v1/games/${ID}`, { method: "OPTIONS", headers: { Origin: "http://localhost:5173" } });
    expect(ok.status).toBe(204);
    expect(ok.headers.get("Access-Control-Allow-Methods")).toContain("PUT");
    expect((await call(`/v1/games/${ID}`, { method: "OPTIONS", headers: { Origin: "https://x.example" } })).status).toBe(403);
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
