#!/usr/bin/env node
// Stand-in for the online-play relay (plans/online-play.md) for UI development and e2e tests.
// Implements GET /lobby, GET /rooms/<id>/api/status and the guest socket /rooms/<id>/ws. It also
// serves app/dist, so one origin is enough to open the app.
//
//   node scripts/fake-relay.mjs [--port 7979] [--upstream http://127.0.0.1:7878]
//
// With --upstream, every room's status and socket are bridged to that server (the fake LAN server
// or a real legatum-server); the upstream sees an ordinary loopback client, so host-only checks do
// not apply here. Without it, rooms answer 503 (host not connected) and sockets close with 1013.
//
// Test controls (not part of the contract):
//   POST /__rooms  body: JSON array of RoomInfo, replaces the lobby list
//   POST /__fail   body: {"fail": true|false}, makes GET /lobby answer 500
//   POST /__ctl    body: {lobbyDelay?, statusDelay?, away?} in ms / boolean; `away` answers status 503 and drops guests
import { createServer, request as httpRequest } from 'node:http';
import { connect } from 'node:net';
import { readFileSync, existsSync, statSync } from 'node:fs';
import { extname, join, normalize, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const here = fileURLToPath(new URL('.', import.meta.url));
const DIST = join(here, '../dist');
const arg = (name) => {
  const i = process.argv.indexOf(name);
  return i > 0 ? process.argv[i + 1] : undefined;
};
const isMain = process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url);

const MIME = { '.html': 'text/html', '.js': 'text/javascript', '.css': 'text/css', '.wasm': 'application/wasm', '.svg': 'image/svg+xml', '.png': 'image/png', '.woff2': 'font/woff2', '.woff': 'font/woff', '.json': 'application/json' };
const CORS = { 'access-control-allow-origin': '*', 'cache-control': 'no-store' };

/** Start a fake relay. Resolves to { url, port, close, setRooms, setFail }. */
export async function startFakeRelay({ port = 0, upstream = null, rooms = [] } = {}) {
  let list = rooms;
  let fail = false;
  let lobbyDelay = 0;
  let statusDelay = 0;
  let away = false;
  const guests = new Set();
  const wait = (ms) => new Promise((done) => setTimeout(done, ms));
  const up = upstream ? new URL(upstream) : null;
  const send = (res, code, body, extra = {}) => {
    res.writeHead(code, { 'content-type': 'application/json', ...CORS, ...extra });
    res.end(typeof body === 'string' ? body : JSON.stringify(body));
  };
  const readBody = (req) => new Promise((done) => {
    let d = '';
    req.on('data', (c) => (d += c));
    req.on('end', () => done(d));
  });

  const server = createServer(async (req, res) => {
    const url = new URL(req.url, 'http://x');
    const path = url.pathname;
    if (req.method === 'OPTIONS') {
      res.writeHead(204, { ...CORS, 'access-control-allow-methods': 'GET, POST, OPTIONS', 'access-control-allow-headers': '*' });
      return res.end();
    }
    if (path === '/__ctl' && req.method === 'POST') {
      const c = JSON.parse(await readBody(req));
      if (c.lobbyDelay !== undefined) lobbyDelay = c.lobbyDelay;
      if (c.statusDelay !== undefined) statusDelay = c.statusDelay;
      if (c.away !== undefined) {
        away = !!c.away;
        if (away) for (const s of guests) s.destroy();
      }
      return send(res, 200, { ok: true });
    }
    if (path === '/lobby' && lobbyDelay) await wait(lobbyDelay);
    if (path === '/lobby') return fail ? send(res, 500, { error: 'down' }) : send(res, 200, { rooms: list });
    if (path === '/__rooms' && req.method === 'POST') {
      list = JSON.parse(await readBody(req));
      return send(res, 200, { ok: true });
    }
    if (path === '/__fail' && req.method === 'POST') {
      fail = !!JSON.parse(await readBody(req)).fail;
      return send(res, 200, { ok: true });
    }
    const m = path.match(/^\/rooms\/([^/]+)\/api\/status$/);
    if (m) {
      if (!list.some((r) => r.room === m[1])) return send(res, 404, { error: 'no such room' });
      if (statusDelay) await wait(statusDelay);
      if (!up || away) return send(res, 503, { error: 'host not connected' });
      const upReq = httpRequest({ host: up.hostname, port: up.port, path: '/api/status', method: 'GET' }, (r) => {
        let body = '';
        r.on('data', (c) => (body += c));
        r.on('end', () => {
          try {
            send(res, 200, { ...JSON.parse(body), is_local: false });
          } catch {
            send(res, 503, { error: 'host not connected' });
          }
        });
      });
      upReq.on('error', () => send(res, 503, { error: 'host not connected' }));
      return upReq.end();
    }
    if (path.startsWith('/api/') || path.startsWith('/rooms/')) return send(res, 404, { error: 'not found' });
    let file = normalize(join(DIST, path === '/' ? 'index.html' : path));
    if (!file.startsWith(DIST) || !existsSync(file) || statSync(file).isDirectory()) file = join(DIST, 'index.html');
    res.writeHead(200, { 'content-type': MIME[extname(file)] ?? 'application/octet-stream' });
    res.end(readFileSync(file));
  });

  // A guest socket: pipe the raw upgrade to the upstream server's /ws.
  server.on('upgrade', (req, socket, head) => {
    const m = new URL(req.url, 'http://x').pathname.match(/^\/rooms\/([^/]+)\/ws$/);
    if (!m || !list.some((r) => r.room === m[1]) || !up || away) {
      socket.end('HTTP/1.1 503 Service Unavailable\r\nConnection: close\r\n\r\n');
      return;
    }
    const back = connect(Number(up.port), up.hostname, () => {
      const lines = [`GET /ws HTTP/1.1`];
      for (let i = 0; i < req.rawHeaders.length; i += 2) {
        const k = req.rawHeaders[i];
        lines.push(`${k}: ${k.toLowerCase() === 'host' ? up.host : req.rawHeaders[i + 1]}`);
      }
      back.write(`${lines.join('\r\n')}\r\n\r\n`);
      if (head?.length) back.write(head);
      guests.add(socket);
      socket.pipe(back);
      back.pipe(socket);
    });
    back.on('error', () => socket.destroy());
    socket.on('error', () => back.destroy());
    socket.on('close', () => { guests.delete(socket); back.destroy(); });
    back.on('close', () => socket.destroy());
  });

  await new Promise((done) => server.listen(port, done));
  const actual = server.address().port;
  return {
    port: actual,
    url: `http://localhost:${actual}`,
    setRooms: (next) => { list = next; },
    setFail: (next) => { fail = next; },
    control: (c) => { if (c.lobbyDelay !== undefined) lobbyDelay = c.lobbyDelay; if (c.statusDelay !== undefined) statusDelay = c.statusDelay; if (c.away !== undefined) { away = !!c.away; if (away) for (const s of guests) s.destroy(); } },
    close: () => new Promise((done) => { server.closeAllConnections?.(); server.close(() => done()); }),
  };
}

if (isMain) {
  const relay = await startFakeRelay({
    port: Number(arg('--port') ?? 7979),
    upstream: arg('--upstream') ?? null,
    rooms: [
      { room: 'demo-room-1', name: 'Ada', build: 'dev', rules_version: 1, protocol: 1, players: 2, humans: 2, open_seats: 1, phase: 'staging', turn: 0, updated_ms: Date.now() },
    ],
  });
  console.log(`fake relay on ${relay.url}`);
}
