#!/usr/bin/env node
// Render the film of an evolution run: narration, deterministic frames, music, captions, assembly.
//   node tools/film/render.mjs <run-dir> [--out ~/LegatumData/films] [--voice Samantha] [--workers 3] [--no-llm] [--fresh-story]
// Output: <out>/<run-id>.mp4 (H.264 High/AAC, 1920x1080, 30 fps, faststart), .srt, .vtt, .jpg
// poster, .story.md (the chosen angle, why, and the fact check). Everything expensive is cached
// under <out>/cache/<run-id>/ (voice clips by text, scenes by content hash), so reruns are cheap
// and an interrupted render resumes where it stopped.
import fs from "node:fs";
import { editorial, editorialWordmark } from "../design/brand.mjs";
import os from "node:os";
import path from "node:path";
import crypto from "node:crypto";
import { spawn, spawnSync, execFileSync } from "node:child_process";
import { fileURLToPath, pathToFileURL } from "node:url";
import { loadRun, archetypesOf, archetypeColor, eras as erasOf, matrixOf } from "./lib/runlog.mjs";
import { readArchive } from "./lib/archive.mjs";
import { loadPlaywright, fontsourceFile } from "./lib/pw.mjs";
import { storyboard } from "./storyboard.mjs";

const here = path.dirname(fileURLToPath(import.meta.url));
const root = path.resolve(here, "../..");
const FPS = 30;
const args = process.argv.slice(2);
const opt = (k, d) => { const i = args.indexOf(`--${k}`); return i >= 0 ? args[i + 1] : d; };
const VALUED = new Set(["--out", "--voice", "--workers", "--rate", "--stills"]);
const runDir = args.find((a, i) => !a.startsWith("--") && !VALUED.has(args[i - 1]));
if (!runDir) { console.error("usage: node tools/film/render.mjs <run-dir> [--out dir] [--voice name] [--workers n] [--no-llm] [--fresh-story]"); process.exit(2); }
const outDir = path.resolve(opt("out", process.env.LEGATUM_FILMS || path.join(os.homedir(), "LegatumData", "films")));
const log = (...m) => console.error(`[film ${new Date().toISOString().slice(11, 19)}]`, ...m);
const sha = (x) => crypto.createHash("sha256").update(typeof x === "string" || Buffer.isBuffer(x) ? x : JSON.stringify(x)).digest("hex").slice(0, 16);
const run = (cmd, a, o = {}) => { const r = spawnSync(cmd, a, { encoding: "utf8", maxBuffer: 1 << 26, ...o }); if (r.status !== 0) throw new Error(`${cmd} ${a.slice(0, 6).join(" ")}…: ${(r.stderr || "").slice(-1500)}`); return r.stdout; };
const duration = (file) => Number(run("ffprobe", ["-v", "error", "-show_entries", "format=duration", "-of", "default=nw=1:nk=1", file]).trim());
const pct = (x) => (x == null ? "–" : `${Math.round(x * 100)}%`);

const data = loadRun(path.resolve(runDir));
const cache = path.join(outDir, "cache", data.id);
for (const d of ["voice", "scenes", "replays", "tapes"]) fs.mkdirSync(path.join(cache, d), { recursive: true });

// ---------- 1. story ----------
const script = await storyboard(runDir, { out: cache, fresh: args.includes("--fresh-story"), llm: !args.includes("--no-llm") });
log(`story: ${script.source}, "${script.title}", ${script.scenes.length} scenes`);

// ---------- 2. narration ----------
function pickVoice() {
  if (opt("voice") || process.env.FILM_VOICE) return opt("voice") || process.env.FILM_VOICE;
  const list = run("say", ["-v", "?"]).split("\n").map((l) => /^(.+?)\s{2,}(\w\w_\w\w)/.exec(l)).filter(Boolean).map((m) => ({ name: m[1].trim(), lang: m[2] }));
  const en = list.filter((v) => v.lang.startsWith("en_"));
  return (en.find((v) => /Premium/i.test(v.name)) || en.find((v) => /Enhanced/i.test(v.name)) || en.find((v) => v.name === "Samantha") || en.find((v) => v.name === "Daniel") || en[0] || { name: "Samantha" }).name;
}
const voice = pickVoice();
const rate = Number(opt("rate", 168));
log(`voice: ${voice} at ${rate} wpm`);
function speak(text) {
  const file = path.join(cache, "voice", `${sha({ voice, rate, text })}.aiff`);
  if (!fs.existsSync(file)) {
    run("say", ["-v", voice, "-r", String(rate), "-o", `${file}.tmp.aiff`, text.replace(/–/g, " to ").replace(/→/g, " to ")]);
    fs.renameSync(`${file}.tmp.aiff`, file);
  }
  return { file, dur: duration(file) };
}

// ---------- 3. timing and captions ----------
const MIN = { title: 5.5, end: 8, text: 4.5, population: 6, graph: 7, map: 7, zoom: 6, versus: 8, matrix: 7, stat: 7 };
let clock = 0;
const clips = [];
const scenes = script.scenes.map((s, i) => {
  const lead = i === 0 ? 1.4 : 0.8, gap = 0.45, tail = 1.1;
  let t = lead;
  const captions = [];
  for (const sentence of s.narration) {
    const c = speak(sentence);
    captions.push({ start: t, end: t + c.dur + 0.2, text: sentence });
    clips.push({ file: c.file, at: clock + t });
    t += c.dur + gap;
  }
  const moving = s.type === "population" && s.params.gen_from !== s.params.gen_to;
  const dur = Math.max(MIN[s.type] + (moving ? 3 : 0), t - gap + tail);
  captions.forEach((c, j) => { c.end = Math.min(c.end, captions[j + 1]?.start ?? dur - 0.4); });
  const scene = { type: s.type, params: { ...s.params }, narration: s.narration, dur: Math.round(dur * FPS) / FPS, captions, start: clock };
  clock += scene.dur;
  return scene;
});
const total = clock;
log(`timing: ${scenes.length} scenes, ${total.toFixed(1)} s`);

// Numbers on stat and end cards come straight from the log, not from the storyteller.
const cps = data.checkpoints, cpFirst = cps[0], cpLast = cps.at(-1);
const musicCredit = JSON.parse(fs.readFileSync(path.join(root, "app/public/audio/manifest.json"), "utf8")).find((m) => m.file === "music/atlas.m4a")?.attribution;
for (const s of scenes) {
  if (s.type === "stat") {
    s.params.items = cpFirst && cpLast && cpFirst !== cpLast ? [
      { label: "Proxy win rate (lower is better)", before: pct(cpFirst.benchmark?.proxy_win_rate), after: pct(cpLast.benchmark?.proxy_win_rate), before_label: `checkpoint ${cpFirst.gen}`, after_label: `checkpoint ${cpLast.gen}` },
      { label: "AI empires alive at turn 150", before: pct(cpFirst.benchmark?.ai_survival_t150), after: pct(cpLast.benchmark?.ai_survival_t150), before_label: `checkpoint ${cpFirst.gen}`, after_label: `checkpoint ${cpLast.gen}` },
    ] : [];
  }
  if (s.type === "end") {
    s.params.lines = [
      `Evolution run ${data.id}${data.standin ? " · a stand-in rehearsal" : ""}`,
      `Every number: docs/sim-results/evolve-${data.id}/`,
      "Maps: real game tapes, replayed by legatum-sim",
      `Story: ${script.source === "llm" ? "Astra (gpt-6-astra), fact-checked against the run log" : "template, from the run log"}`,
      `Narration: macOS voice ${voice}`,
      `Music: ${musicCredit}`,
    ];
  }
}

// ---------- 4. replays of the showcase tapes ----------
const sim = process.env.LEGATUM_SIM || path.join(root, "target/release/legatum-sim");
const showcase = new Map(cps.flatMap((c) => (c.showcase || []).map((s) => [s.game_id, s])));
const replays = {};
for (const id of new Set(scenes.flatMap((s) => [s.params.game_id, s.params.game_id_b]).filter(Boolean))) {
  const sc = showcase.get(id);
  const rec = readArchive(sc.archive_path);
  const label = `evo-${id.replace(/[^A-Za-z0-9_-]/g, "-")}`;
  const file = path.join(cache, "replays", `${label}-seed-${rec.metadata.seed}.json`);
  if (!fs.existsSync(file)) {
    if (!fs.existsSync(sim)) throw new Error(`legatum-sim not built (${sim}); cargo build --release -p legatum-sim`);
    const tape = path.join(cache, "tapes", `${id}.json`);
    fs.writeFileSync(tape, JSON.stringify(rec.tape));
    run(sim, ["--replay-export", tape, "--out", path.join(cache, "replays"), "--label", label]);
  }
  replays[id] = JSON.parse(fs.readFileSync(file, "utf8"));
  log(`replay ${id}: ${replays[id].frames.length} turns`);
}

// ---------- 5. frames ----------
const gens = data.generations;
const shortEvent = (e) => {
  const a = archetypesOf(gens).find((n) => e.detail?.startsWith(n) || e.detail?.includes(` ${n} `) || e.detail?.endsWith(n));
  return e.kind === "archetype_takeover" ? `${a || "a new strategy"} takes the lead` : e.kind === "archetype_extinct" ? `${a || "a strategy"} dies out` : e.kind.replaceAll("_", " ");
};
const runPayload = {
  id: data.id,
  standin: !!data.standin,
  population: data.run.config?.population,
  stages: data.run.standin?.stages || [],
  archetypes: archetypesOf(gens).map((n) => ({ name: n, color: archetypeColor(n) })),
  gens: gens.map((g) => ({ gen: g.gen, best: g.best?.fitness ?? null, pwr: g.best?.proxy_win_rate ?? null, mean: g.mean_fitness ?? null, share: g.archetype_share || {} })),
  eras: erasOf(gens),
  events: data.events.map((e) => ({ gen: e.gen, kind: e.kind, short: shortEvent(e) })),
  checkpoints: cps.map((c) => ({ gen: c.gen, matrix: matrixOf(c.benchmark?.matchup_matrix) })),
};
const fonts = [
  ["Cormorant", 600, "normal", "cormorant", "cormorant-latin-600-normal.woff2"],
  ["Cormorant", 500, "italic", "cormorant", "cormorant-latin-500-italic.woff2"],
  ["Libre Franklin", 400, "normal", "libre-franklin", "libre-franklin-latin-400-normal.woff2"],
  ["Libre Franklin", 500, "normal", "libre-franklin", "libre-franklin-latin-500-normal.woff2"],
  ["Libre Franklin", 600, "normal", "libre-franklin", "libre-franklin-latin-600-normal.woff2"],
].map(([family, weight, style, pkg, f]) => ({ family, weight, style, file: fontsourceFile(pkg, f) })).filter((f) => f.file).map((f) => ({ ...f, data: fs.readFileSync(f.file).toString("base64"), file: undefined }));
if (fonts.length < 5) log("warning: some @fontsource fonts are missing (npm install in app/); falling back to system fonts");
// JPEG frames arrive full-range (yuvj420p); convert to limited-range yuv420p, which every player
// (Safari and QuickTime included) expects from H.264.
const ENCODE = ["-vf", "scale=in_range=pc:out_range=tv,format=yuv420p", "-color_range", "tv", "-colorspace", "bt709", "-color_primaries", "bt709", "-color_trc", "bt709", "-c:v", "libx264", "-preset", "medium", "-crf", "18", "-pix_fmt", "yuv420p"];
const rendererHash = sha(JSON.stringify(ENCODE) + JSON.stringify(editorial) + editorialWordmark + fs.readFileSync(path.join(here, "scene/film.js")) + fs.readFileSync(path.join(here, "scene/film.html")));
const dataHash = sha({ runPayload, fonts: fonts.map((f) => f.family + f.weight + f.data.length) });
const sceneFile = (s) => path.join(cache, "scenes", `${sha({ s: { ...s, start: undefined }, rendererHash, dataHash, replays: [s.params.game_id, s.params.game_id_b].filter(Boolean).map((id) => sha(JSON.stringify(replays[id].header) + replays[id].frames.length)) })}.mp4`);
const payload = { run: runPayload, scenes, replays, fonts, brand: editorial, wordmark: editorialWordmark };

if (opt("stills")) {
  // Preview: one PNG per scene at the given fraction of its length, then stop.
  const { chromium } = await loadPlaywright();
  const browser = await chromium.launch();
  const page = await browser.newPage({ viewport: { width: 1920, height: 1080 } });
  page.on("pageerror", (e) => log(`page error: ${e.message}`));
  await page.goto(pathToFileURL(path.join(here, "scene/film.html")).href);
  await page.evaluate((p) => window.FILM.init(p), payload);
  const dir = path.join(cache, "stills");
  fs.mkdirSync(dir, { recursive: true });
  for (const [i, s] of scenes.entries()) {
    const b64 = await page.evaluate(([idx, t]) => window.FILM.frameJpeg(idx, t, 0.9), [i, s.dur * Number(opt("stills"))]);
    fs.writeFileSync(path.join(dir, `${String(i + 1).padStart(2, "0")}-${s.type}.jpg`), Buffer.from(b64, "base64"));
  }
  await browser.close();
  console.log(dir);
  process.exit(0);
}

const todo = scenes.map((s, i) => ({ i, s, file: sceneFile(s) })).filter((x) => !fs.existsSync(x.file));
log(`frames: ${todo.length} of ${scenes.length} scenes to render (${todo.reduce((a, x) => a + Math.round(x.s.dur * FPS), 0)} frames)`);
if (todo.length) {
  const { chromium } = await loadPlaywright();
  const browser = await chromium.launch({ args: ["--disable-gpu-vsync", "--disable-lcd-text"] });
  const workers = Math.max(1, Math.min(Number(opt("workers", 3)), todo.length));
  const queue = [...todo].sort((a, b) => b.s.dur - a.s.dur);
  const t0 = Date.now();
  await Promise.all([...Array(workers)].map(async () => {
    const page = await browser.newPage({ viewport: { width: 1920, height: 1080 }, deviceScaleFactor: 1 });
    page.on("pageerror", (e) => log(`page error: ${e.message}`));
    await page.goto(pathToFileURL(path.join(here, "scene/film.html")).href);
    await page.evaluate((p) => window.FILM.init(p), payload);
    while (queue.length) {
      const { i, s, file } = queue.shift();
      const n = Math.round(s.dur * FPS);
      const tmp = `${file}.tmp.mp4`;
      const ff = spawn("ffmpeg", ["-y", "-loglevel", "error", "-f", "image2pipe", "-framerate", String(FPS), "-c:v", "mjpeg", "-i", "-", ...ENCODE, "-profile:v", "high", "-level", "4.1", "-r", String(FPS), "-g", String(FPS * 2), tmp], { stdio: ["pipe", "ignore", "pipe"] });
      let err = "";
      ff.stderr.on("data", (d) => (err += d));
      const done = new Promise((res, rej) => ff.on("close", (c) => (c === 0 ? res() : rej(new Error(`ffmpeg scene ${i}: ${err}`)))));
      for (let f = 0; f < n; f++) {
        const b64 = await page.evaluate(([idx, t]) => window.FILM.frameJpeg(idx, t, 0.93), [i, f / FPS]);
        if (!ff.stdin.write(Buffer.from(b64, "base64"))) await new Promise((r) => ff.stdin.once("drain", r));
      }
      ff.stdin.end();
      await done;
      fs.renameSync(tmp, file);
      log(`scene ${i + 1}/${scenes.length} (${s.type}, ${s.dur.toFixed(1)} s) done, ${((Date.now() - t0) / 1000).toFixed(0)} s elapsed`);
    }
    await page.close();
  }));
  await browser.close();
}

// ---------- 6. assembly ----------
const work = path.join(cache, "assembly");
fs.mkdirSync(work, { recursive: true });
fs.writeFileSync(path.join(work, "scenes.txt"), scenes.map((s) => `file '${sceneFile(s)}'`).join("\n") + "\n");
run("ffmpeg", ["-y", "-loglevel", "error", "-f", "concat", "-safe", "0", "-i", path.join(work, "scenes.txt"), "-c", "copy", path.join(work, "video.mp4")]);
// Narration: every sentence placed at its time, loudness-normalized; music looped under it,
// ducked by the narration (sidechain), faded in and out.
const music = path.join(root, "app/public/audio/music/atlas.m4a");
const inputs = clips.flatMap((c) => ["-i", c.file]);
const place = clips.map((c, k) => `[${k + 1}:a]aresample=48000,aformat=channel_layouts=mono,adelay=${Math.round(c.at * 1000)}:all=1[v${k}]`).join(";");
const narr = clips.length
  ? `${place};${clips.map((_, k) => `[v${k}]`).join("")}amix=inputs=${clips.length}:normalize=0:duration=longest,apad=whole_dur=${total.toFixed(3)},atrim=0:${total.toFixed(3)},loudnorm=I=-16:TP=-1.5:LRA=11,aresample=48000,aformat=channel_layouts=stereo,asplit=2[n1][n2]`
  : `anullsrc=r=48000:cl=stereo,atrim=0:${total.toFixed(3)},asplit=2[n1][n2]`;
const filter = `${narr};[0:a]aresample=48000,aformat=channel_layouts=stereo,atrim=0:${total.toFixed(3)},volume=0.30,afade=t=in:st=0:d=2.5,afade=t=out:st=${Math.max(0, total - 4).toFixed(3)}:d=4[m];[m][n1]sidechaincompress=threshold=0.015:ratio=8:attack=20:release=450[duck];[duck][n2]amix=inputs=2:normalize=0,alimiter=limit=0.95[a]`;
run("ffmpeg", ["-y", "-loglevel", "error", "-stream_loop", "-1", "-i", music, ...inputs, "-filter_complex", filter, "-map", "[a]", "-c:a", "aac", "-b:a", "192k", "-ar", "48000", "-t", total.toFixed(3), path.join(work, "audio.m4a")]);
fs.mkdirSync(outDir, { recursive: true });
const final = path.join(outDir, `${data.id}.mp4`);
run("ffmpeg", ["-y", "-loglevel", "error", "-i", path.join(work, "video.mp4"), "-i", path.join(work, "audio.m4a"), "-map", "0:v", "-map", "1:a", "-c", "copy", "-movflags", "+faststart", "-t", total.toFixed(3), `${final}.tmp.mp4`]);
fs.renameSync(`${final}.tmp.mp4`, final);

// Captions beside the film (the same text is burned into the picture).
const ts = (t, sep) => { const ms = Math.round(t * 1000); const h = Math.floor(ms / 3600e3), m = Math.floor(ms / 60e3) % 60, s = Math.floor(ms / 1000) % 60; return `${String(h).padStart(2, "0")}:${String(m).padStart(2, "0")}:${String(s).padStart(2, "0")}${sep}${String(ms % 1000).padStart(3, "0")}`; };
const cues = scenes.flatMap((s) => s.captions.map((c) => ({ a: s.start + c.start, b: s.start + c.end, text: c.text })));
fs.writeFileSync(path.join(outDir, `${data.id}.srt`), cues.map((c, k) => `${k + 1}\n${ts(c.a, ",")} --> ${ts(c.b, ",")}\n${c.text}\n`).join("\n"));
fs.writeFileSync(path.join(outDir, `${data.id}.vtt`), `WEBVTT\n\n${cues.map((c) => `${ts(c.a, ".")} --> ${ts(c.b, ".")}\n${c.text}\n`).join("\n")}`);
// Poster: the first population scene, or the title.
const posterScene = scenes.find((s) => s.type === "population") || scenes[0];
run("ffmpeg", ["-y", "-loglevel", "error", "-ss", (posterScene.start + Math.min(3, posterScene.dur / 2)).toFixed(2), "-i", final, "-frames:v", "1", "-q:v", "3", path.join(outDir, `${data.id}.jpg`)]);
const story = fs.readFileSync(path.join(cache, "story.md"), "utf8");
fs.writeFileSync(path.join(outDir, `${data.id}.story.md`), `${story}\n## The film\n\n${path.basename(final)}: ${total.toFixed(1)} s, ${scenes.length} scenes, narrated by macOS voice ${voice}. Music: ${musicCredit}\n`);
fs.writeFileSync(path.join(outDir, `${data.id}.script.json`), JSON.stringify({ ...script, voice, total_seconds: total, scenes: scenes.map(({ captions, ...s }) => ({ ...s, captions })) }, null, 2));
log(`film: ${final} (${duration(final).toFixed(1)} s)`);
console.log(final);
