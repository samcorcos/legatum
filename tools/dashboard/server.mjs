// Build dashboard: a zero-dependency local server that aggregates .progress/ logs, release
// progress (derived from plans/playtest-feedback.md), git activity across agent worktrees,
// generated art and simulation status.
// Run: node tools/dashboard/server.mjs  (http://localhost:4747; PORT=… to change it,
// DASHBOARD_ROOT=/path/to/checkout to read another checkout's data)

import http from "node:http";
import { toolBrandCss } from "../design/brand.mjs";
import fs from "node:fs";
import path from "node:path";
import { execFile, execFileSync } from "node:child_process";
import { promisify } from "node:util";
import { fileURLToPath } from "node:url";
import { gunzipSync } from "node:zlib";
import { parseQueue } from "./release.mjs";
import { createEvolution, safeRunId } from "./evolution.mjs";

const here = path.dirname(fileURLToPath(import.meta.url));
const root = path.resolve(process.env.DASHBOARD_ROOT || path.join(here, "../.."));
const progress = path.join(root, ".progress");
const port = Number(process.env.PORT || 4747);
// Evolution panel: reads docs/sim-results/evolve-*/ in the background refresh (tools/dashboard/evolution.mjs).
export const evolution = createEvolution({ root, replayDir: path.join(progress, "replays") });

const read = (p, fallback = null) => {
  try {
    return fs.readFileSync(p, "utf8");
  } catch {
    return fallback;
  }
};
const readJson = (p, fallback = null) => {
  const s = read(p);
  if (s == null) return fallback;
  try {
    return JSON.parse(s);
  } catch {
    return fallback;
  }
};
const git = (args, cwd = root) => {
  try {
    return execFileSync("git", args, { cwd, encoding: "utf8", stdio: ["ignore", "pipe", "ignore"] }).trim();
  } catch {
    return "";
  }
};

function logs() {
  const out = {};
  for (const f of fs.readdirSync(progress)) {
    if (!f.endsWith(".log")) continue;
    const id = f.slice(0, -4);
    out[id] = read(path.join(progress, f), "")
      .split("\n")
      .map((l) => l.trim())
      .filter(Boolean)
      .map((l) => {
        const i = l.indexOf(" | ");
        return i > 0 ? { t: l.slice(0, i), msg: l.slice(i + 3) } : { t: "", msg: l };
      });
  }
  return out;
}

const execFileAsync = promisify(execFile);
const gitAsync = async (args, cwd = root) => {
  try {
    return (await execFileAsync("git", args, { cwd, encoding: "utf8", maxBuffer: 1 << 24 })).stdout.trim();
  } catch {
    return "";
  }
};

// git runs in ~50 agent worktrees, so it is refreshed in the background and never blocks a request.
async function worktrees() {
  const raw = await gitAsync(["worktree", "list", "--porcelain"]);
  const list = [];
  let cur = {};
  for (const line of raw.split("\n")) {
    if (line.startsWith("worktree ")) cur = { path: line.slice(9) };
    else if (line.startsWith("branch ")) cur.branch = line.slice(7).replace("refs/heads/", "");
    else if (line === "") {
      if (cur.path) list.push(cur);
      cur = {};
    }
  }
  if (cur.path) list.push(cur);
  // Two git processes per worktree; with ~70 worktrees, launching them all at once every refresh
  // kept ~140 git processes alive on an already overloaded machine. Four at a time is enough.
  return mapLimit(list, 4, async (w) => {
    const [commits, status] = await Promise.all([
      gitAsync(["log", "--format=%h|%ar|%s", "master..HEAD"], w.path),
      gitAsync(["status", "--porcelain", "--untracked-files=no"], w.path),
    ]);
    const dirty = status.split("\n").filter(Boolean).length;
    return {
      path: w.path.replace(root, "."),
      branch: w.branch || "(detached)",
      commits: commits ? commits.split("\n").map((c) => c.split("|")) : [],
      dirty,
    };
  });
}

async function mapLimit(items, limit, fn) {
  const out = new Array(items.length);
  let next = 0;
  const worker = async () => {
    while (next < items.length) {
      const i = next++;
      out[i] = await fn(items[i]);
    }
  };
  await Promise.all(Array.from({ length: Math.min(limit, items.length) }, worker));
  return out;
}

function art() {
  const dir = path.join(progress, "art");
  if (!fs.existsSync(dir)) return [];
  return fs
    .readdirSync(dir)
    .filter((f) => /\.(png|svg|jpg|jpeg|gif)$/i.test(f))
    .map((f) => ({ name: f, mtime: fs.statSync(path.join(dir, f)).mtimeMs }))
    .sort((a, b) => b.mtime - a.mtime);
}

// Reports are up to ~20 MB decompressed; parse each version once.
const summaryCache = new Map();
function simSummary(file) {
  const st = fs.statSync(file);
  const key = `${file}:${st.mtimeMs}:${st.size}`;
  if (summaryCache.has(key)) return summaryCache.get(key);
  let summary = null;
  try {
    const text = file.endsWith(".gz") ? gunzipSync(fs.readFileSync(file)).toString("utf8") : fs.readFileSync(file, "utf8");
    summary = JSON.parse(text).summary ?? null;
  } catch {
    summary = null;
  }
  summaryCache.set(key, summary);
  return summary;
}

function simResults() {
  const dir = path.join(root, "docs/sim-results");
  if (!fs.existsSync(dir)) return [];
  return fs
    .readdirSync(dir)
    // Large reports are committed gzipped; prefer the plain file when both exist.
    .filter((f) => f.endsWith(".json") || (f.endsWith(".json.gz") && !fs.existsSync(path.join(dir, f.slice(0, -3)))))
    .sort()
    .reverse()
    .slice(0, 8)
    .map((f) => ({ name: f.replace(/\.gz$/, ""), summary: simSummary(path.join(dir, f)) }));
}

function inbox() {
  const dir = path.join(progress, "inbox");
  if (!fs.existsSync(dir)) return [];
  return fs
    .readdirSync(dir)
    .filter((f) => f.endsWith(".json"))
    .map((f) => {
      const item = readJson(path.join(dir, f));
      if (!item) return null;
      return { ...item, id: item.id || f.slice(0, -5), mtime: fs.statSync(path.join(dir, f)).mtimeMs };
    })
    .filter(Boolean)
    .sort((a, b) => b.mtime - a.mtime);
}

function feedback() {
  return read(path.join(progress, "feedback.jsonl"), "")
    .split("\n")
    .filter(Boolean)
    .map((l) => {
      try {
        return JSON.parse(l);
      } catch {
        return null;
      }
    })
    .filter(Boolean);
}

function astraTasks() {
  const dir = path.join(progress, "astra-queue", "running");
  if (!fs.existsSync(dir)) return [];
  return fs
    .readdirSync(dir)
    .filter((f) => f.endsWith(".md"))
    .map((f) => {
      const text = read(path.join(dir, f), "");
      const name = (text.match(/^name: *(.+)$/m) || [])[1] || f;
      const mode = (text.match(/^mode: *(.+)$/m) || [])[1] || "";
      return { name: name.trim(), mode: mode.trim(), started: fs.statSync(path.join(dir, f)).mtimeMs };
    });
}

// Release progress is derived from the active feedback plan; release.json supplies its name.
const queueRel = "plans/playtest-feedback.md";
const releaseFile = path.join(progress, "release.json");
const historyFile = path.join(progress, "release-history.json");
const releaseDefaults = {
  name: "Playtest 2",
  started_at: "2026-09-27T16:00:00-04:00",
  note: "The next tagged build for the user (E6): ships once A (UI quality) and B (controls) are merged and verified end to end. Progress is derived live from plans/playtest-feedback.md; update that file, not this one.",
};
function releaseConfig() {
  const cfg = { ...releaseDefaults, ...(readJson(releaseFile, {}) || {}) };
  const t = new Date(cfg.started_at); // no offset in the string = local time
  cfg.started_at = isNaN(t) ? new Date(releaseDefaults.started_at).toISOString() : t.toISOString();
  return cfg;
}

let queueCache = { key: null, value: null };
function queueProgress() {
  const file = path.join(root, queueRel);
  let st;
  try {
    st = fs.statSync(file);
  } catch {
    return { ...parseQueue(""), error: `${queueRel} not found` };
  }
  const key = `${st.mtimeMs}:${st.size}`;
  if (queueCache.key !== key) {
    let value;
    try {
      value = parseQueue(fs.readFileSync(file, "utf8"));
    } catch (e) {
      value = { ...parseQueue(""), error: `${queueRel}: ${e.message}` };
    }
    queueCache = { key, value };
  }
  return queueCache.value;
}

const point = (t, q) => ({ t, pct: q.pct, done: q.done, total: q.total });
const samePoint = (a, b) => a && b && a.pct === b.pct && a.done === b.done && a.total === b.total;

// History starts with the queue as it stood at the release start, plus every commit of the queue
// since, reconstructed from git; after that the server appends a point whenever the numbers move.
async function seedHistory(cfg) {
  const points = [];
  const base = await gitAsync(["rev-list", "-1", "--first-parent", `--before=${cfg.started_at}`, "master", "--", queueRel]);
  points.push(point(cfg.started_at, parseQueue(base ? await gitAsync(["show", `${base}:${queueRel}`]) : "")));
  // --first-parent: master's own states, not the older copies on merged side branches.
  const log = await gitAsync(["log", "--first-parent", "--reverse", `--since=${cfg.started_at}`, "--format=%H %cI", "master", "--", queueRel]);
  for (const line of log.split("\n").filter(Boolean)) {
    const [sha, t] = line.split(" ");
    const p = point(new Date(t).toISOString(), parseQueue(await gitAsync(["show", `${sha}:${queueRel}`])));
    if (!samePoint(p, points.at(-1))) points.push(p);
  }
  return { release: cfg.name, started_at: cfg.started_at, points };
}

// Called from the 15-second background refresh only, never per request.
let history = null;
async function recordHistory() {
  const cfg = releaseConfig();
  if (!history) history = readJson(historyFile, null);
  let changed = false;
  if (!history || history.release !== cfg.name || history.started_at !== cfg.started_at || !Array.isArray(history.points)) {
    history = await seedHistory(cfg);
    changed = true;
  }
  const q = queueProgress();
  const last = history.points.at(-1);
  if (!q.error && !samePoint(last, q)) {
    const now = new Date().toISOString();
    // Debounce: edits landing within a minute of the previous live point replace it.
    const recent = history.points.length > 1 && Date.now() - new Date(last.t).getTime() < 60e3;
    if (recent) history.points[history.points.length - 1] = point(now, q);
    else history.points.push(point(now, q));
    changed = true;
  }
  if (!changed) return;
  const tmp = `${historyFile}.tmp`;
  fs.writeFileSync(tmp, JSON.stringify(history, null, 1) + "\n");
  fs.renameSync(tmp, historyFile);
}

function release() {
  const cfg = releaseConfig();
  const plan = readJson(path.join(progress, "plan.json"));
  const incs = plan?.increments || [];
  const previous = incs.length && incs.every((i) => i.done_at)
    ? { name: `${incs.at(-1).id} First playable`, done_at: incs.at(-1).done_at, url: "/plan.json" }
    : null;
  return {
    ...cfg,
    source: queueRel,
    ...queueProgress(),
    history: history && history.release === cfg.name ? history.points : [],
    commits: slow.releaseCommits,
    previous,
  };
}

const slow = { worktrees: [], art: [], simResults: [], releaseCommits: null };
let refreshing = false;
let worktreesAt = 0;
async function refreshSlow() {
  if (refreshing) return;
  refreshing = true;
  // Each part keeps its last good value if a refresh fails (e.g. a report deleted mid-scan).
  try { slow.art = art(); } catch { /* keep cached */ }
  try { slow.simResults = simResults(); } catch { /* keep cached */ }
  // Worktree git state is the expensive part; once a minute is plenty for the dashboard.
  if (Date.now() - worktreesAt >= 60000) {
    try { slow.worktrees = await worktrees(); worktreesAt = Date.now(); } catch { /* keep cached */ }
  }
  try {
    const n = await gitAsync(["rev-list", "--count", `--since=${releaseConfig().started_at}`, "master"]);
    if (n) slow.releaseCommits = Number(n);
  } catch { /* keep cached */ }
  try { await recordHistory(); } catch (e) { console.error("release history", e); }
  try { await evolution.refresh(); } catch (e) { console.error("evolution", e); }
  refreshing = false;
}

function state() {
  return {
    astraTasks: astraTasks(),
    astraPending: fs.existsSync(path.join(progress, "astra-queue", "pending")) ? fs.readdirSync(path.join(progress, "astra-queue", "pending")).filter((f) => f.endsWith(".md")).length : 0,
    opsReport: read(path.join(progress, "ops-report.md")),
    plan: readJson(path.join(progress, "plan.json")),
    release: release(),
    inbox: inbox(),
    feedback: feedback(),
    now: new Date().toISOString(),
    roster: readJson(path.join(progress, "agents.json"), { agents: [] }),
    prd: readJson(path.join(root, "plans/first-playable.prd.json"), { stories: [] }),
    logs: logs(),
    mainLog: git(["log", "--format=%h|%ar|%s", "-12", "master"])
      .split("\n")
      .filter(Boolean)
      .map((c) => c.split("|")),
    worktrees: slow.worktrees,
    art: slow.art,
    sim: readJson(path.join(progress, "sim-status.json")),
    simResults: slow.simResults,
    reviews: fs.existsSync(path.join(root, ".reviews"))
      ? fs.readdirSync(path.join(root, ".reviews")).filter((f) => f.endsWith(".md"))
      : [],
  };
}

const types = { ".mp4": "video/mp4", ".vtt": "text/vtt; charset=utf-8", ".srt": "text/plain; charset=utf-8", ".html": "text/html", ".js": "text/javascript; charset=utf-8", ".css": "text/css; charset=utf-8", ".json": "application/json; charset=utf-8", ".png": "image/png", ".svg": "image/svg+xml", ".jpg": "image/jpeg", ".gif": "image/gif", ".md": "text/plain; charset=utf-8", ".m4a": "audio/mp4", ".txt": "text/plain; charset=utf-8" };

// Shipped game audio for the Sound board. Byte ranges matter: Safari won't play media without 206s.
const audioDir = path.join(root, "app/public/audio");
function serveAudio(rel, req, res) {
  if (!/^([a-z-]+\/)?[A-Za-z0-9_.-]+\.(m4a|json|md|txt)$/.test(rel) || rel.includes("..")) return false;
  const file = path.join(audioDir, rel);
  if (!fs.existsSync(file) || !fs.statSync(file).isFile() || !fs.realpathSync(file).startsWith(fs.realpathSync(audioDir) + path.sep)) return false;
  sendRanged(file, req, res);
  return true;
}

// Byte-range responses for media (audio, the evolution film). Safari needs 206s to play either.
function sendRanged(file, req, res, cache = "no-store") {
  const size = fs.statSync(file).size;
  const headers = { "content-type": types[path.extname(file)] || "application/octet-stream", "accept-ranges": "bytes", "cache-control": cache };
  const m = /^bytes=(\d*)-(\d*)$/.exec(req.headers?.range || "");
  if (m && (m[1] || m[2])) {
    const start = m[1] ? Number(m[1]) : Math.max(0, size - Number(m[2]));
    const end = m[1] && m[2] ? Math.min(Number(m[2]), size - 1) : size - 1;
    if (start > end || start >= size) {
      res.writeHead(416, { "content-range": `bytes */${size}` });
      res.end();
      return;
    }
    res.writeHead(206, { ...headers, "content-range": `bytes ${start}-${end}/${size}`, "content-length": end - start + 1 });
    fs.createReadStream(file, { start, end }).on("error", () => res.destroy()).pipe(res);
    return;
  }
  res.writeHead(200, { ...headers, "content-length": size });
  fs.createReadStream(file).on("error", () => res.destroy()).pipe(res);
}

// Grid previews: small JPEGs made once per art version with macOS `sips`, cached on disk.
const thumbDir = path.join(progress, "art", ".thumbs");
const thumbJobs = new Map();
function thumbFor(name) {
  const src = path.join(progress, "art", name);
  const st = fs.statSync(src);
  const out = path.join(thumbDir, `${name.replace(/\.[^.]+$/, "")}-${Math.floor(st.mtimeMs)}.jpg`);
  if (fs.existsSync(out)) return Promise.resolve(out);
  if (!thumbJobs.has(out)) {
    fs.mkdirSync(thumbDir, { recursive: true });
    thumbJobs.set(out, execFileAsync("sips", ["-Z", "480", "-s", "format", "jpeg", "-s", "formatOptions", "72", src, "--out", out])
      .then(() => out).finally(() => thumbJobs.delete(out)));
  }
  return thumbJobs.get(out);
}
function serveThumb(name, url, res) {
  const src = path.join(progress, "art", name);
  if (!/\.(png|jpg|jpeg|gif)$/i.test(name) || !fs.existsSync(src) || !fs.statSync(src).isFile()) {
    res.writeHead(404);
    return res.end("not found");
  }
  thumbFor(name)
    .then((out) => {
      res.writeHead(200, { "content-type": "image/jpeg", "cache-control": url.searchParams.has("v") ? "public, max-age=31536000, immutable" : "no-store" });
      fs.createReadStream(out).on("error", () => res.destroy()).pipe(res);
    })
    .catch(() => {
      // Fall back to the original if sips is unavailable.
      res.writeHead(200, { "content-type": types[path.extname(src)] || "application/octet-stream" });
      fs.createReadStream(src).on("error", () => res.destroy()).pipe(res);
    });
}

let brandCss;

export function createServer({ replayDir = path.join(progress, "replays"), evo = evolution } = {}) {
  return http.createServer((req, res) => {
    const url = new URL(req.url, "http://x");
    try {
      if (url.pathname === "/api/feedback" && req.method === "POST") {
        let body = "";
        req.on("data", (c) => (body += c));
        req.on("end", () => {
          try {
            const f = JSON.parse(body);
            const text = String(f.text || "").slice(0, 5000);
            const reaction = String(f.reaction || "").slice(0, 40);
            if (!text.trim() && !reaction) throw new Error("empty");
            const entry = { t: new Date().toISOString(), item: String(f.item || "general").slice(0, 120), reaction, text };
            fs.appendFileSync(path.join(progress, "feedback.jsonl"), JSON.stringify(entry) + "\n");
            res.writeHead(200, { "content-type": "application/json" });
            res.end(JSON.stringify({ ok: true }));
          } catch (e) {
            res.writeHead(400);
            res.end("bad feedback");
          }
        });
        return;
      }
      if (url.pathname === "/api/state") {
        const body = JSON.stringify(state()); // before writeHead, so a failure can still answer 500
        res.writeHead(200, { "content-type": "application/json", "cache-control": "no-store" });
        return res.end(body);
      }
      if (url.pathname === "/api/evolution") {
        const body = JSON.stringify(evo.state(url.searchParams.get("run")));
        res.writeHead(200, { "content-type": "application/json", "cache-control": "no-store" });
        return res.end(body);
      }
      const film = /^\/films\/([A-Za-z0-9_.-]+)\.(mp4|vtt|srt|jpg|story\.md)$/.exec(url.pathname);
      if (film && safeRunId(film[1])) {
        const f = path.join(evo.filmsDir, `${film[1]}.${film[2]}`);
        if (fs.existsSync(f) && fs.statSync(f).isFile()) return sendRanged(f, req, res);
      }
      if (url.pathname === "/brand.css") {
        brandCss ??= toolBrandCss("court");
        res.writeHead(200, { "content-type": "text/css; charset=utf-8" });
        return res.end(brandCss);
      }
      let file = null;
      const report = /^\/evolution\/([A-Za-z0-9_.-]+)\/report\.html$/.exec(url.pathname);
      if (url.pathname === "/") file = path.join(here, "index.html");
      else if (["/replay.js", "/replay.css", "/sound.js", "/sound.css", "/evolution.js", "/evolution.css"].includes(url.pathname)) file = path.join(here, url.pathname.slice(1));
      else if (report && safeRunId(report[1])) file = path.join(evo.resultsDir, `evolve-${report[1]}`, "report.html");
      else if (url.pathname === "/plan.json") file = path.join(progress, "plan.json");
      else if (url.pathname.startsWith("/audio/") && serveAudio(url.pathname.slice("/audio/".length), req, res)) return;
      else if (url.pathname === "/wordmark.svg") file = path.join(root, "app/src/assets/icons/logo-legatum.svg");
      else if (url.pathname === "/favicon.svg") file = path.join(root, "app/public/favicon.svg");
      else if (url.pathname.startsWith("/replays/")) {
        // Accept a single literal JSON basename. No encoded separators, nested paths, or
        // symlinks out of the replay directory. Never turn an unsafe path into a valid one.
        const name = url.pathname.slice("/replays/".length);
        if (/^[A-Za-z0-9_-]+\.json$/.test(name)) {
          const candidate = path.join(replayDir, name);
          if (fs.existsSync(candidate) && fs.lstatSync(candidate).isFile() &&
              path.dirname(fs.realpathSync(candidate)) === fs.realpathSync(replayDir)) file = candidate;
          else if (name === "index.json" && !fs.existsSync(candidate)) {
            res.writeHead(200, { "content-type": types[".json"], "cache-control": "no-store" });
            return res.end("[]");
          }
        }
      }
      else if (url.pathname.startsWith("/thumb/")) {
        const name = path.basename(decodeURIComponent(url.pathname.slice("/thumb/".length)));
        return serveThumb(name, url, res);
      }
      else if (url.pathname.startsWith("/art/")) file = path.join(progress, "art", path.basename(decodeURIComponent(url.pathname)));
      else if (url.pathname.startsWith("/reviews/")) file = path.join(root, ".reviews", path.basename(url.pathname));
      if (file && fs.existsSync(file) && fs.statSync(file).isFile()) {
        // Art URLs carry ?v=<mtime>, so a given URL never changes content.
        const cache = url.searchParams.has("v") ? "public, max-age=31536000, immutable" : "no-store";
        res.writeHead(200, { "content-type": types[path.extname(file)] || "application/octet-stream", "cache-control": cache });
        return fs.createReadStream(file).on("error", () => res.destroy()).pipe(res);
      }
      res.writeHead(404);
      res.end("not found");
    } catch (e) {
      console.error(req.url, e);
      // A failure after a response has started (e.g. a streamed file) must not crash the server.
      if (res.headersSent) return res.destroy();
      res.writeHead(500);
      res.end(String(e));
    }
  });
}

export const server = createServer();

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  // Name the release once; edit .progress/release.json to start the next one.
  fs.mkdirSync(path.dirname(releaseFile), { recursive: true });
  if (!fs.existsSync(releaseFile)) fs.writeFileSync(releaseFile, JSON.stringify(releaseDefaults, null, 2) + "\n");
  refreshSlow();
  setInterval(refreshSlow, 15000);
  server.listen(port, () => console.log(`Legatum build dashboard on http://localhost:${port}`));
}
