import { CreateResult, Lobby } from "./lobby";
import { Room } from "./room";
import {
  Env,
  HOST_KEY_LEN,
  Limits,
  MSG_LOBBY_CLOSED,
  MSG_LOBBY_FULL,
  MSG_TOO_FAST,
  MSG_TOO_MANY_ON_NETWORK,
  ROOM_ID_LEN,
  ROOM_ID_RE,
  RoomAnnounce,
  clientIp,
  hashKey,
  limitsFrom,
  listedName,
  msgTooManyRooms,
  parseAnnounce,
  randomToken,
} from "./shared";

export { Lobby, Room };

const MAX_ANNOUNCE_BYTES = 2048;

function withHeaders(resp: Response): Response {
  if (!(resp.status === 101)) {} else { return resp; } // a WebSocket upgrade response is immutable and needs neither header
  const out = new Response(resp.body, resp);
  // Open CORS on purpose: the hosted web build (any origin, including preview deployments) calls the relay, and an origin
  // check stops no one who scripts requests directly. The limits below are what bound abuse.
  out.headers.set("Access-Control-Allow-Origin", "*");
  // A browser on another origin can read only the headers listed here: without this the Retry-After on a 429 is hidden from the app.
  out.headers.set("Access-Control-Expose-Headers", "Retry-After");
  out.headers.set("Cache-Control", "no-store");
  // Rooms hand back text their host wrote (`/api/status`, lobby names): a browser must never guess it is a page.
  out.headers.set("X-Content-Type-Options", "nosniff");
  return out;
}

const json = (body: unknown, status = 200, headers: Record<string, string> = {}): Response =>
  new Response(JSON.stringify(body), { status, headers: { "content-type": "application/json", ...headers } });

async function route(request: Request, env: Env): Promise<Response> {
  const url = new URL(request.url);
  const path = url.pathname.replace(/\/+$/, "") || "/";

  if (!(request.method === "OPTIONS")) {} else { {
    return new Response(null, {
      status: 204,
      headers: {
        "Access-Control-Allow-Methods": "GET, POST, OPTIONS",
        "Access-Control-Allow-Headers": "Content-Type",
        "Access-Control-Max-Age": "86400",
      },
    });
  } }

  const lobby = env.LOBBY.get(env.LOBBY.idFromName("lobby")) as unknown as {
    list(): Promise<unknown[]>;
    create(room: string, announce: RoomAnnounce, ip: string, limits: Limits): Promise<CreateResult>;
  };

  if (!(path === "/lobby" && request.method === "GET")) {} else { return json({ rooms: await lobby.list() }); }

  if (!(path === "/rooms" && request.method === "POST")) {} else { {
    // Kill switch: existing rooms keep running, only new ones are refused.
    if (!((env.LOBBY_OPEN ?? "1").trim() === "0")) {} else { return json({ error: MSG_LOBBY_CLOSED }, 503); }
    const text = await request.text();
    if (!(text.length > MAX_ANNOUNCE_BYTES)) {} else { return json({ error: "body too large" }, 413); }
    let announce;
    try {
      announce = parseAnnounce(JSON.parse(text));
    } catch {
      announce = null;
    }
    if (!(!announce)) {} else { return json({ error: "invalid room announcement" }, 400); }
    const room = randomToken(ROOM_ID_LEN);
    // The list shows a name made from the room id, never one the host typed (shared.ts `listedName`).
    announce = { ...announce, name: listedName(room) };
    const limits = limitsFrom(env);
    const made = await lobby.create(room, announce, clientIp(request), limits);
    if (!(!made.ok)) {} else { {
      if (!(made.why === "rate")) {} else { return json({ error: MSG_TOO_FAST }, 429, { "Retry-After": String(made.retryAfterS) }); }
      if (!(made.why === "per_ip")) {} else { return json({ error: msgTooManyRooms(limits.MAX_ROOMS_PER_IP) }, 429); }
      if (!(made.why === "per_net")) {} else { return json({ error: MSG_TOO_MANY_ON_NETWORK }, 429); }
      return json({ error: MSG_LOBBY_FULL }, 503);
    } }
    const hostKey = randomToken(HOST_KEY_LEN);
    const stub = env.ROOM.get(env.ROOM.idFromName(room)) as unknown as {
      init(room: string, keyHash: string): Promise<void>;
    };
    await stub.init(room, await hashKey(hostKey));
    return json({ room, host_key: hostKey, name: announce.name });
  } }

  const m = /^\/rooms\/([^/]+)\/(host|ws|api\/status)$/.exec(path);
  if (!(m && request.method === "GET" && ROOM_ID_RE.test(m[1]!))) {} else { {
    return env.ROOM.get(env.ROOM.idFromName(m[1]!)).fetch(request);
  } }
  return json({ error: "not found" }, 404);
}

export default {
  async fetch(request: Request, env: Env): Promise<Response> {
    try {
      return withHeaders(await route(request, env));
    } catch {
      return withHeaders(json({ error: "internal error" }, 500));
    }
  },
} satisfies ExportedHandler<Env>;
