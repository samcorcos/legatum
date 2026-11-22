// A plain static file server that behaves like Cloudflare Pages for the hosted build's smoke test:
// files from app/dist, the response headers from dist/_headers, 404.html (status 404) for anything
// that is not a file, and no backend at all. It answers no /api or /ws route, so a build that still
// needs a server fails here the way it would on the real host.
//   node e2e-hosted/serve.mjs [port] [dir]     default: 4178, dist
import { createServer } from 'node:http';
import { readFile, stat } from 'node:fs/promises';
import { extname, join, normalize, resolve, sep } from 'node:path';

const port = Number(process.argv[2] ?? 4178);
const root = resolve(process.argv[3] ?? 'dist');
const TYPES = {
  '.html': 'text/html; charset=utf-8', '.js': 'text/javascript; charset=utf-8', '.css': 'text/css; charset=utf-8',
  '.wasm': 'application/wasm', '.json': 'application/json', '.txt': 'text/plain; charset=utf-8', '.svg': 'image/svg+xml',
  '.png': 'image/png', '.jpg': 'image/jpeg', '.webp': 'image/webp', '.woff2': 'font/woff2', '.woff': 'font/woff',
  '.ogg': 'audio/ogg', '.mp3': 'audio/mpeg', '.wav': 'audio/wav', '.m4a': 'audio/mp4',
};

/** [{ pattern, headers }] from a Pages `_headers` file: a path line, then indented `Name: value` lines. */
export function parseHeaders(text) {
  const rules = [];
  for (const line of text.split('\n')) {
    if (!line.trim() || line.trim().startsWith('#')) continue;
    if (!/^\s/.test(line)) rules.push({ pattern: line.trim(), headers: [] });
    else if (rules.length) {
      const at = line.indexOf(':');
      rules[rules.length - 1].headers.push([line.slice(0, at).trim(), line.slice(at + 1).trim()]);
    }
  }
  return rules;
}

const matches = (pattern, path) => (pattern.endsWith('*') ? path.startsWith(pattern.slice(0, -1)) : path === pattern);

const rules = parseHeaders(await readFile(join(root, '_headers'), 'utf8').catch(() => ''));

async function file(path) {
  const full = normalize(join(root, path));
  if (full !== root && !full.startsWith(root + sep)) return null;
  const info = await stat(full).catch(() => null);
  if (info?.isDirectory()) return file(join(path, 'index.html'));
  return info?.isFile() ? full : null;
}

createServer(async (req, res) => {
  const path = decodeURIComponent(new URL(req.url, 'http://localhost').pathname);
  const found = req.method === 'GET' || req.method === 'HEAD' ? await file(path) : null;
  const target = found ?? await file('/404.html');
  const headers = { 'Content-Type': TYPES[extname(target ?? '.txt')] ?? 'application/octet-stream', 'Cache-Control': 'public, max-age=0, must-revalidate' };
  for (const rule of rules) if (matches(rule.pattern, path)) for (const [name, value] of rule.headers) headers[name] = value;
  const body = target ? await readFile(target) : 'Not found';
  res.writeHead(found ? 200 : 404, headers);
  res.end(req.method === 'HEAD' ? undefined : body);
}).listen(port, '127.0.0.1', () => console.log(`serving ${root} at http://127.0.0.1:${port}`));
