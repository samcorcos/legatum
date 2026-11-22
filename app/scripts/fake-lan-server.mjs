#!/usr/bin/env node
// Stand-in for legatum-server (crates/server) implementing plans/lan-protocol.md with the WASM
// engine for isolated UI development. Real play and end-to-end tests use the Rust server.
// Not for real play: no persistence, events are not filtered by visibility.
//
//   npm run build && node scripts/fake-lan-server.mjs [--port 7878]
//   (needs app/src/wasm-pkg from scripts/build-wasm.sh)
import { createServer } from 'node:http';
import { createHash } from 'node:crypto';
import { readFileSync, existsSync, statSync } from 'node:fs';
import { networkInterfaces } from 'node:os';
import { extname, join, normalize, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const here = fileURLToPath(new URL('.', import.meta.url));
const portArg = process.argv.indexOf('--port');
const PORT = portArg > 0 ? Number(process.argv[portArg + 1]) : 7878;
const DIST = join(here, '../dist');
const PKG = join(here, '../src/wasm-pkg/');

// Importing the protocol for fake-socket tests must not load WASM or listen on the LAN.
const isMain = process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url);
let wasm;
if (isMain) {
  wasm = await import(join(PKG, 'legatum.js'));
  wasm.initSync({ module: readFileSync(join(PKG, 'legatum_bg.wasm')) });
}

// ---- Game state ----
let game = null;
let gameId = null;
let setup = null;
/** Human seats: { name, faction, player, socket, joined, ready, orders } */
let seats = [];

export function newGame(setupJson, Game = wasm.Game) {
  game = new Game(setupJson);
  setup = JSON.parse(setupJson);
  gameId = Math.random().toString(36).slice(2, 10);
  for (const s of seats) s.socket?.close();
  seats = setup.players
    .map((p, i) => ({ ...p, player: i }))
    .filter((p) => p.is_human)
    .map((p, i) => ({ seat: i, name: p.name, faction: p.faction, player: p.player, socket: null, joined: false, ready: false, orders: [] }));
}

const seatStatus = () =>
  seats.map((s) => ({ seat: s.seat, player: s.player, name: s.name, faction: s.faction, connected: !!s.socket, ready: s.ready, orders: s.orders.length }));

function lanUrls() {
  const out = [];
  for (const list of Object.values(networkInterfaces())) {
    for (const a of list ?? []) if (a.family === 'IPv4' && !a.internal) out.push(`http://${a.address}:${PORT}`);
  }
  return out;
}

const isLocal = (req) => ['127.0.0.1', '::1', '::ffff:127.0.0.1'].includes(req.socket.remoteAddress);

function status(req) {
  return {
    mode: 'server',
    has_game: !!game,
    turn: game ? game.turn() : 0,
    seats: seatStatus(),
    lan_urls: lanUrls(),
    is_local: isLocal(req),
    staging: room ? stagingState() : null,
  };
}

// ---- Staging room (development reference for plans/lan-protocol.md) ----
const FACTIONS = ['Switzerland', 'Japan', 'Byzantium'];
/** { room_id, config, occupants: ({ name, faction, socket, ready, is_host } | null)[] } */
let room = null;
const watchers = new Set();

const stagingState = () => ({
  room_id: room.room_id,
  config: room.config,
  occupants: room.occupants.map((o) => o && { name: o.name, faction: o.faction, connected: !!o.socket, ready: o.ready, is_host: o.is_host }),
});

function broadcastRoom() {
  if (!room) return;
  const state = stagingState();
  for (const w of watchers) w.send({ type: 'staging', state });
}

const everyoneReady = () => room.config.seats.every((s, i) => s.kind !== 'open' || (room.occupants[i]?.socket && room.occupants[i].ready));

function setRoomConfig(config) {
  if (!Array.isArray(config.seats) || config.seats.length < 2 || config.seats.length > 8 || config.seats[0].kind !== 'host') {
    throw new Error('a room needs 2-8 seats with the host in seat 0');
  }
  if (!room) {
    room = { room_id: Math.random().toString(36).slice(2, 10), config, occupants: config.seats.map(() => null) };
    return;
  }
  const occupants = config.seats.map((s, i) => {
    const o = room.occupants[i];
    if (!o) return null;
    if (s.kind !== room.config.seats[i]?.kind || (s.kind !== 'open' && s.kind !== 'host')) {
      o.socket?.send({ type: 'staging_seat_lost', message: 'The host changed your seat.' });
      return null;
    }
    o.ready = false;
    return o;
  });
  for (const o of room.occupants.slice(config.seats.length)) o?.socket?.send({ type: 'staging_seat_lost', message: 'The host removed your seat.' });
  room.config = config;
  room.occupants = occupants;
}

/** Humans first (host, then open seats), then AIs; closed seats dropped; teams compacted. */
function roomSetup() {
  const c = room.config;
  const human = (s) => s.kind === 'host' || s.kind === 'open';
  const order = c.seats.map((_, i) => i).filter((i) => c.seats[i].kind !== 'closed');
  const ids = [...order.filter((i) => human(c.seats[i])), ...order.filter((i) => !human(c.seats[i]))];
  const teams = [];
  const players = ids.map((i) => {
    const s = c.seats[i];
    const o = room.occupants[i];
    let faction = o?.faction ?? s.faction;
    if (faction === 'Random') faction = FACTIONS[(Number(BigInt(c.seed) % 3n) + i) % 3];
    if (!teams.includes(s.team)) teams.push(s.team);
    const p = { name: o?.name ?? s.name, faction, team: teams.indexOf(s.team), is_human: human(s) };
    if (!human(s) && s.difficulty && s.difficulty !== 'Normal') p.difficulty = s.difficulty;
    return p;
  });
  return { setup: { seed: c.seed, width: c.width, height: c.height, turn_limit: c.turn_limit, players }, humanSeatOf: ids.filter((i) => human(c.seats[i])) };
}

function startRoom() {
  const { setup, humanSeatOf } = roomSetup();
  newGame(JSON.stringify(setup));
  humanSeatOf.forEach((roomSeat, seat) => room.occupants[roomSeat]?.socket?.send({ type: 'staging_started', game_id: gameId, seat }));
  console.log(`room started: ${setup.players.map((p) => `${p.name}${p.is_human ? '*' : ''}`).join(', ')}`);
  room = null;
}

/** Staging messages; returns true when handled. */
function onStaging(conn, local, msg) {
  if (!msg.type.startsWith('staging_')) return false;
  if (msg.type === 'staging_watch') {
    watchers.add(conn);
    if (room) conn.send({ type: 'staging', state: stagingState() });
    return true;
  }
  if (!room) return conn.send({ type: 'error', code: 'no_room', message: 'No game room is open.' }), true;
  const mine = room.occupants.findIndex((o) => o?.socket === conn);
  const me = mine >= 0 ? room.occupants[mine] : null;
  switch (msg.type) {
    case 'staging_take': {
      const seat = room.config.seats[msg.seat];
      if (!seat || !(seat.kind === 'open' || (seat.kind === 'host' && local))) {
        conn.send({ type: 'error', code: 'not_open', message: 'That seat is not open.' });
        break;
      }
      const o = room.occupants[msg.seat];
      if (o && o.socket && o.socket !== conn) {
        conn.send({ type: 'error', code: 'seat_occupied', message: 'Someone already took that seat.' });
        break;
      }
      if (me && mine !== msg.seat) room.occupants[mine] = null;
      room.occupants[msg.seat] = { name: String(msg.name || 'Player').slice(0, 24), faction: o?.faction ?? seat.faction, socket: conn, ready: false, is_host: seat.kind === 'host' };
      watchers.add(conn);
      break;
    }
    case 'staging_update':
      if (!me) break;
      if (typeof msg.name === 'string') me.name = msg.name.slice(0, 24) || 'Player';
      if (msg.faction) me.faction = msg.faction;
      me.ready = false;
      break;
    case 'staging_ready':
      if (me) me.ready = !!msg.ready;
      break;
    case 'staging_leave':
      if (me) room.occupants[mine] = null;
      break;
    default:
      conn.send({ type: 'error', message: `unknown message ${msg.type}` });
  }
  broadcastRoom();
  return true;
}

function stagingClosed(conn) {
  watchers.delete(conn);
  if (!room) return;
  const o = room.occupants.find((x) => x?.socket === conn);
  if (o) {
    o.socket = null;
    o.ready = false;
    broadcastRoom();
  }
}

// ---- Minimal WebSocket (RFC 6455, text frames) ----
function wsAccept(req, socket) {
  const key = req.headers['sec-websocket-key'];
  const accept = createHash('sha1').update(key + '258EAFA5-E914-47DA-95CA-C5AB0DC85B11').digest('base64');
  socket.write(`HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Accept: ${accept}\r\n\r\n`);
  const conn = {
    send(obj) {
      const data = Buffer.from(JSON.stringify(obj));
      let header;
      if (data.length < 126) header = Buffer.from([0x81, data.length]);
      else if (data.length < 65536) {
        header = Buffer.alloc(4);
        header[0] = 0x81;
        header[1] = 126;
        header.writeUInt16BE(data.length, 2);
      } else {
        header = Buffer.alloc(10);
        header[0] = 0x81;
        header[1] = 127;
        header.writeBigUInt64BE(BigInt(data.length), 2);
      }
      if (!socket.destroyed) socket.write(Buffer.concat([header, data]));
    },
    close() {
      if (!socket.destroyed) socket.end(Buffer.from([0x88, 0]));
    },
    onmessage: null,
    onclose: null,
  };
  let buf = Buffer.alloc(0);
  socket.on('data', (chunk) => {
    buf = Buffer.concat([buf, chunk]);
    for (;;) {
      if (buf.length < 2) return;
      const op = buf[0] & 0x0f;
      let len = buf[1] & 0x7f;
      let off = 2;
      if (len === 126) {
        if (buf.length < 4) return;
        len = buf.readUInt16BE(2);
        off = 4;
      } else if (len === 127) {
        if (buf.length < 10) return;
        len = Number(buf.readBigUInt64BE(2));
        off = 10;
      }
      const masked = buf[1] & 0x80;
      const maskOff = off;
      if (masked) off += 4;
      if (buf.length < off + len) return;
      const payload = Buffer.from(buf.subarray(off, off + len));
      if (masked) for (let i = 0; i < len; i++) payload[i] ^= buf[maskOff + (i % 4)];
      buf = buf.subarray(off + len);
      if (op === 8) return conn.close();
      if (op === 9) {
        socket.write(Buffer.concat([Buffer.from([0x8a, payload.length]), payload]));
        continue;
      }
      if (op === 1) conn.onmessage?.(payload.toString('utf8'));
    }
  });
  socket.on('close', () => conn.onclose?.());
  socket.on('error', () => {});
  return conn;
}

// ---- Protocol ----
const broadcastSeats = () => {
  for (const s of seats) s.socket?.send({ type: 'seats', seats: seatStatus() });
};

function sendView(s) {
  s.socket?.send({ type: 'view', view: JSON.parse(game.view(s.player)) });
}

function maybeResolve() {
  if (!game || !seats.every((s) => s.joined)) return;
  const connected = seats.filter((s) => s.socket);
  if (!connected.length || !connected.every((s) => s.ready)) return;
  const orders = {};
  for (const s of seats) orders[s.player] = s.orders;
  let result;
  try {
    result = JSON.parse(game.end_turn(JSON.stringify(orders)));
  } catch (e) {
    for (const s of seats) s.socket?.send({ type: 'error', message: `turn failed: ${e.message}` });
    return;
  }
  for (const s of seats) {
    s.ready = false;
    s.orders = [];
  }
  for (const s of seats) {
    s.socket?.send({ type: 'turn', ...result, turn: game.turn() });
    sendView(s);
  }
  broadcastSeats();
  console.log(`turn ${result.turn} resolved (${result.events.length} events) hash ${result.hash}`);
}

export function onSocket(conn, local = false) {
  let me = null;
  conn.onmessage = (raw) => {
    let msg;
    try {
      msg = JSON.parse(raw);
    } catch {
      return conn.send({ type: 'error', message: 'bad json' });
    }
    if (!msg || typeof msg !== 'object' || Array.isArray(msg) || typeof msg.type !== 'string' || !msg.type.trim()) {
      return conn.send({ type: 'error', message: 'expected an object with a non-empty string type' });
    }
    try {
      dispatch(msg);
    } catch (e) {
      conn.send({ type: 'error', message: `bad message: ${e?.message ?? e}` });
    }
  };

  function dispatch(msg) {
    if (msg.type === 'ping') return conn.send({ type: 'pong' });
    if (typeof msg.type === 'string' && onStaging(conn, local, msg)) return;
    if (!game) return conn.send({ type: 'error', message: 'no game' });
    if (msg.type === 'join') {
      const s = seats.find((x) => x.seat === msg.seat);
      if (!s) return conn.send({ type: 'error', message: `no human seat ${msg.seat}` });
      if (s.socket && s.socket !== conn) {
        s.socket.send({ type: 'error', message: 'Another browser took over this seat.' });
        const old = s.socket;
        s.socket = null;
        old.close();
      }
      s.socket = conn;
      s.joined = true;
      me = s;
      s.ready = false;
      conn.send({ type: 'welcome', seat: s.seat, game_id: gameId, orders: s.orders });
      sendView(s);
      broadcastSeats();
      return;
    }
    if (!me || me.socket !== conn) return conn.send({ type: 'error', message: 'join a seat first' });
    switch (msg.type) {
      case 'orders': {
        if (!Array.isArray(msg.orders)) return conn.send({ type: 'error', message: 'orders must be an array' });
        // Decode/validate before replacing the last good list: malformed orders can throw.
        const errors = JSON.parse(game.validate(me.player, JSON.stringify(msg.orders)));
        me.orders = msg.orders;
        me.ready = false;
        conn.send({ type: 'validation', errors });
        broadcastSeats();
        break;
      }
      case 'ready':
        me.ready = !!msg.ready;
        broadcastSeats();
        maybeResolve();
        break;
      case 'forecast': {
        const f = game.forecast(me.player, msg.unit, msg.target.q, msg.target.r, msg.mode);
        conn.send({ type: 'forecast', req_id: msg.req_id, forecast: JSON.parse(f) });
        break;
      }
      default:
        conn.send({ type: 'error', message: `unknown message ${msg.type}` });
    }
  }
  conn.onclose = () => {
    stagingClosed(conn);
    if (me && me.socket === conn) {
      me.socket = null;
      me.ready = false;
      broadcastSeats();
      maybeResolve();
    }
  };
}

// ---- HTTP ----
const MIME = { '.html': 'text/html', '.js': 'text/javascript', '.css': 'text/css', '.wasm': 'application/wasm', '.svg': 'image/svg+xml', '.png': 'image/png', '.woff2': 'font/woff2', '.woff': 'font/woff', '.json': 'application/json' };

const json = (res, code, body) => {
  res.writeHead(code, { 'content-type': 'application/json' });
  res.end(JSON.stringify(body));
};

const readBody = (req) =>
  new Promise((resolve) => {
    let d = '';
    req.on('data', (c) => (d += c));
    req.on('end', () => resolve(d));
  });

const server = createServer(async (req, res) => {
  const url = new URL(req.url, 'http://x');
  if (url.pathname === '/api/status') return json(res, 200, status(req));
  if (url.pathname === '/api/game' && req.method === 'POST') {
    if (!isLocal(req)) return json(res, 403, { error: 'only the host can start a game' });
    try {
      newGame(await readBody(req));
      console.log(`new game ${gameId}: ${seats.map((s) => `${s.name} (${s.faction})`).join(' & ')}`);
      return json(res, 200, status(req));
    } catch (e) {
      res.writeHead(400);
      return res.end(String(e.message ?? e));
    }
  }
  if (url.pathname.startsWith('/api/staging')) {
    if (!isLocal(req)) return json(res, 403, { error: 'only the host can manage the game room' });
    try {
      if (url.pathname === '/api/staging' && req.method === 'POST') {
        setRoomConfig(JSON.parse(await readBody(req)));
        broadcastRoom();
      } else if (url.pathname === '/api/staging' && req.method === 'DELETE') {
        for (const w of watchers) w.send({ type: 'staging_closed' });
        room = null;
      } else if (url.pathname === '/api/staging/start' && req.method === 'POST') {
        if (!room) return json(res, 409, { code: 'no_room', message: 'No game room is open.' });
        if (!everyoneReady()) return json(res, 409, { code: 'not_ready', message: 'Every open seat must be taken and Ready.' });
        startRoom();
      } else return json(res, 404, { error: 'not found' });
      return json(res, 200, status(req));
    } catch (e) {
      return json(res, 400, { message: String(e.message ?? e) });
    }
  }
  if (url.pathname === '/api/save') {
    if (!isLocal(req) || !game) return json(res, 403, { error: 'no' });
    res.writeHead(200, { 'content-type': 'application/json' });
    return res.end(game.save());
  }
  if (url.pathname === '/api/load' && req.method === 'POST') {
    if (!isLocal(req)) return json(res, 403, { error: 'no' });
    game = wasm.Game.load(await readBody(req));
    gameId = Math.random().toString(36).slice(2, 10);
    return json(res, 200, status(req));
  }
  let file = normalize(join(DIST, url.pathname === '/' ? 'index.html' : url.pathname));
  if (!file.startsWith(DIST) || !existsSync(file) || statSync(file).isDirectory()) file = join(DIST, 'index.html');
  res.writeHead(200, { 'content-type': MIME[extname(file)] ?? 'application/octet-stream' });
  res.end(readFileSync(file));
});

server.on('upgrade', (req, socket) => {
  if (new URL(req.url, 'http://x').pathname !== '/ws') return socket.destroy();
  onSocket(wsAccept(req, socket), isLocal(req));
});

if (isMain) server.listen(PORT, '0.0.0.0', () => {
  console.log(`fake legatum-server on http://localhost:${PORT}`);
  for (const u of lanUrls()) console.log(`  share: ${u}`);
});
