import { env, exports } from "cloudflare:workers";
import { runDurableObjectAlarm } from "cloudflare:test";
import { expect } from "vitest";
import worker from "../src/index";
import type { Env } from "../src/shared";

export const BASE = "https://relay.test";
const enc = new TextEncoder();
const dec = new TextDecoder();

let ipCounter = 0;
/** A fresh client address, so tests that do not care about per-address limits never meet them. */
export const freshIp = (): string => {
  ipCounter++;
  return `198.51.${(ipCounter >> 8) & 255}.${ipCounter & 255}`;
};

type Init = RequestInit & { ip?: string };

function withIp(init: Init | undefined): RequestInit {
  const { ip, ...rest } = init ?? {};
  const headers = new Headers(rest.headers);
  if (!headers.has("CF-Connecting-IP")) headers.set("CF-Connecting-IP", ip ?? freshIp());
  return { ...rest, headers };
}

/** A request through the deployed Worker, with the wrangler.jsonc vars. */
export const call = (path: string, init?: Init) =>
  (exports as unknown as { default: { fetch(u: string, i?: RequestInit): Promise<Response> } }).default.fetch(BASE + path, withIp(init));

/** A request through the Worker with some vars replaced (the Durable Objects are the same). */
export const callWith = (vars: Partial<Env>, path: string, init?: Init) =>
  worker.fetch(new Request(BASE + path, withIp(init)), Object.assign({}, env as unknown as Env, vars));

export const announce = (name: unknown = "Alice", extra: object = {}) => ({ name, build: "8.9.74", rules_version: 3, protocol: 1, ...extra });

export async function create(body: unknown = announce(), ip?: string) {
  const r = await call("/rooms", { method: "POST", body: JSON.stringify(body), ip });
  return { r, j: (await r.clone().json()) as { room: string; host_key: string; name: string; error?: string } };
}

/** A WebSocket with a message queue and a recorded close. */
export class Sock {
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
  async until(pred: () => boolean, what: string, tries = 100) {
    for (let i = 0; i < tries && !pred(); i++) {
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

export async function connect(path: string, ip?: string): Promise<Sock> {
  const r = await call(path, { headers: { Upgrade: "websocket" }, ip });
  expect(r.status).toBe(101);
  return new Sock(r.webSocket!);
}

export function frame(conn: number, kind: number, payload: string | Uint8Array = ""): ArrayBuffer {
  const p = typeof payload === "string" ? enc.encode(payload) : payload;
  const out = new Uint8Array(5 + p.length);
  new DataView(out.buffer).setUint32(0, conn);
  out[4] = kind;
  out.set(p, 5);
  return out.buffer;
}

export async function lobbyRooms() {
  return ((await (await call("/lobby")).json()) as { rooms: Record<string, unknown>[] }).rooms;
}

export async function hostedRoom(name = "Alice", ip?: string) {
  const { j } = await create(announce(name), ip);
  const host = await connect(`/rooms/${j.room}/host?key=${j.host_key}`);
  return { ...j, host };
}

export const waitFor = async (pred: () => Promise<boolean>, tries = 50) => {
  for (let i = 0; i < tries && !(await pred()); i++) await new Promise((r) => setTimeout(r, 20));
};

export const roomNs = (env as unknown as { ROOM: DurableObjectNamespace }).ROOM;
export const lobbyNs = (env as unknown as { LOBBY: DurableObjectNamespace }).LOBBY;
export const lobbyStub = () => lobbyNs.get(lobbyNs.idFromName("lobby"));
export const alarm = (room: string) => runDurableObjectAlarm(roomNs.get(roomNs.idFromName(room)));
