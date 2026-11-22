import { Lobby } from "./lobby";
import { Room } from "./room";
import { Env, HOST_KEY_LEN, ROOM_ID_LEN, ROOM_ID_RE, hashKey, parseAnnounce, randomToken } from "./shared";

export { Lobby, Room };

const MAX_ANNOUNCE_BYTES = 2048;

function withHeaders(resp: Response): Response {
  if (resp.status === 101) return resp; // a WebSocket upgrade response is immutable and needs neither header
  const out = new Response(resp.body, resp);
  out.headers.set("Access-Control-Allow-Origin", "*");
  out.headers.set("Cache-Control", "no-store");
  return out;
}

const json = (body: unknown, status = 200): Response =>
  new Response(JSON.stringify(body), { status, headers: { "content-type": "application/json" } });

async function route(request: Request, env: Env): Promise<Response> {
  const url = new URL(request.url);
  const path = url.pathname.replace(/\/+$/, "") || "/";

  if (request.method === "OPTIONS") {
    return new Response(null, {
      status: 204,
      headers: {
        "Access-Control-Allow-Methods": "GET, POST, OPTIONS",
        "Access-Control-Allow-Headers": "Content-Type",
        "Access-Control-Max-Age": "86400",
      },
    });
  }

  const lobby = env.LOBBY.get(env.LOBBY.idFromName("lobby")) as unknown as {
    list(): Promise<unknown[]>;
    create(room: string, announce: unknown): Promise<boolean>;
  };

  if (path === "/lobby" && request.method === "GET") return json({ rooms: await lobby.list() });

  if (path === "/rooms" && request.method === "POST") {
    const text = await request.text();
    if (text.length > MAX_ANNOUNCE_BYTES) return json({ error: "body too large" }, 413);
    let announce;
    try {
      announce = parseAnnounce(JSON.parse(text));
    } catch {
      announce = null;
    }
    if (!announce) return json({ error: "invalid room announcement" }, 400);
    const room = randomToken(ROOM_ID_LEN);
    const hostKey = randomToken(HOST_KEY_LEN);
    if (!(await lobby.create(room, announce))) return json({ error: "relay is full" }, 503);
    const stub = env.ROOM.get(env.ROOM.idFromName(room)) as unknown as {
      init(room: string, keyHash: string): Promise<void>;
    };
    await stub.init(room, await hashKey(hostKey));
    return json({ room, host_key: hostKey });
  }

  const m = /^\/rooms\/([^/]+)\/(host|ws|api\/status)$/.exec(path);
  if (m && request.method === "GET" && ROOM_ID_RE.test(m[1]!)) {
    return env.ROOM.get(env.ROOM.idFromName(m[1]!)).fetch(request);
  }
  return json({ error: "not found" }, 404);
}

export default {
  async fetch(request: Request, env: Env): Promise<Response> {
    try {
      return withHeaders(await route(request, env));
    } catch (e) {
      return withHeaders(json({ error: "internal error" }, 500));
    }
  },
} satisfies ExportedHandler<Env>;
