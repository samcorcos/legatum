#!/usr/bin/env node
// A tiny stand-in for a Legatum host, for the UI and for manual checks.
// Announces a room, opens the tunnel, answers /api/status and echoes every guest text back.
//
//   node scripts/fake-host.mjs [relay-url] [--name "Fake Host"] [--build 8.9.74] [--rules 1] [--protocol 1]
//
// Needs Node 22+ (global WebSocket). Stop with Ctrl-C: it sends `bye`, so the room vanishes at once.

const args = process.argv.slice(2);
const opt = (flag, dflt) => {
  const i = args.indexOf(flag);
  return i >= 0 && args[i + 1] ? args[i + 1] : dflt;
};
const relay = (args.find((a, i) => !a.startsWith("--") && !args[i - 1]?.startsWith("--")) ?? "http://127.0.0.1:8787").replace(/\/+$/, "");
const announce = {
  name: opt("--name", "Fake Host"),
  build: opt("--build", "8.9.74"),
  rules_version: Number(opt("--rules", "1")),
  protocol: Number(opt("--protocol", "1")),
};

const enc = new TextEncoder();
const dec = new TextDecoder();
const frame = (conn, kind, text = "") => {
  const p = enc.encode(text);
  const out = new Uint8Array(5 + p.length);
  new DataView(out.buffer).setUint32(0, conn, false);
  out[4] = kind;
  out.set(p, 5);
  return out;
};

const res = await fetch(`${relay}/rooms`, { method: "POST", body: JSON.stringify(announce) });
if (!res.ok) {
  console.error("announce failed", res.status, await res.text());
  process.exit(1);
}
const { room, host_key } = await res.json();
console.log(`room ${room}`);
console.log(`guest socket: ${relay.replace(/^http/, "ws")}/rooms/${room}/ws`);
console.log(`status:       ${relay}/rooms/${room}/api/status`);

const guests = new Set();
const ws = new WebSocket(`${relay.replace(/^http/, "ws")}/rooms/${room}/host?key=${host_key}`);
ws.binaryType = "arraybuffer";

const push = () => {
  const n = guests.size;
  ws.send(frame(0, 3, JSON.stringify({ is_local: true, fake: true, guests: n })));
  ws.send(frame(0, 4, JSON.stringify({ players: 6, humans: 1 + n, open_seats: Math.max(0, 1 - n), phase: "staging", turn: 0 })));
};

ws.onopen = () => {
  console.log("tunnel open");
  push();
};
ws.onmessage = (e) => {
  const v = new DataView(e.data);
  const conn = v.getUint32(0, false);
  const kind = v.getUint8(4);
  const text = dec.decode(new Uint8Array(e.data, 5));
  if (kind === 0) {
    guests.add(conn);
    console.log(`guest ${conn} joined`);
    push();
  } else if (kind === 1) {
    console.log(`guest ${conn}: ${text}`);
    ws.send(frame(conn, 1, `echo: ${text}`));
  } else if (kind === 2) {
    guests.delete(conn);
    console.log(`guest ${conn} left`);
    push();
  }
};
ws.onclose = (e) => {
  console.log("tunnel closed", e.code, e.reason);
  process.exit(0);
};
ws.onerror = () => console.error("tunnel error");

process.on("SIGINT", () => {
  try {
    ws.send(frame(0, 5));
  } catch {}
  setTimeout(() => process.exit(0), 200);
});
