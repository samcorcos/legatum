// Assembles the teaser from the master captures, with burned-in captions and the game's own soundtrack.
//   node docs/media/src/captions.mjs <captions-dir> [--square]
//   node docs/media/src/teaser.mjs <masters-dir> <captions-dir> <out.mp4> [--square] [--fps 30] [--crf 20 | --kbps 1300]
// The storyboard is teaser.json. A shot: clip (a master in <masters-dir>), ss and t (start and length, seconds),
// speed (playback multiplier), caption, crop [x, y, w, h] in the 1920x1080 master (scaled back to full frame),
// sqx (left edge of the 1080-wide square crop), fade (seconds of dissolve into this shot; default board.fade).
import { readFileSync, mkdtempSync, rmSync } from "node:fs";
import { spawnSync } from "node:child_process";
import { tmpdir } from "node:os";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
const here = dirname(fileURLToPath(import.meta.url));
const root = resolve(here, "../../..");
const args = process.argv.slice(2);
const flag = (name, d) => { const i = args.indexOf(`--${name}`); if (i < 0) return d; const v = args[i + 1]; args.splice(i, 2); return v; };
const square = args.includes("--square"); if (square) args.splice(args.indexOf("--square"), 1);
const board = JSON.parse(readFileSync(join(here, "teaser.json"), "utf8"));
const fps = Number(flag("fps", board.fps ?? 60)), crf = flag("crf", "20"), kbps = flag("kbps", null);
const [masters, caps, out] = args;
if (!masters || !caps || !out) { console.error("usage: teaser.mjs <masters-dir> <captions-dir> <out.mp4> [--square] [--fps n] [--crf n | --kbps n]"); process.exit(2); }
const W = square ? 1080 : 1920, H = 1080;
const inputs = [], parts = [], lens = [], fades = [];
let n = 0;
board.shots.forEach((s, i) => {
  const dur = s.t / (s.speed ?? 1);
  lens.push(dur);
  fades.push(s.fade ?? board.fade);
  inputs.push("-i", join(masters, `${s.clip}.mp4`));
  let chain = `[${n++}:v]trim=start=${s.ss}:duration=${s.t},setpts=(PTS-STARTPTS)/${s.speed ?? 1},fps=${fps}`;
  if (s.crop) chain += `,crop=${s.crop[2]}:${s.crop[3]}:${s.crop[0]}:${s.crop[1]},scale=1920:1080:flags=lanczos`;
  if (square) chain += `,crop=1080:1080:${s.sqx ?? 420}:0`;
  parts.push(`${chain},setsar=1,format=yuv420p[v${i}]`);
  if (s.caption) {
    inputs.push("-loop", "1", "-framerate", String(fps), "-t", String(dur), "-i", join(caps, `cap-${String(i).padStart(2, "0")}.png`));
    const ci = n++;
    const cin = s.captionIn ?? 0.15;
    parts.push(`[${ci}:v]format=rgba,fade=t=in:st=${cin}:d=0.25:alpha=1,fade=t=out:st=${(dur - 0.45).toFixed(2)}:d=0.25:alpha=1[c${i}]`);
    parts.push(`[v${i}][c${i}]overlay=0:0:format=auto,format=yuv420p[s${i}]`);
  } else parts.push(`[v${i}]null[s${i}]`);
});
const FE = 0.6;
inputs.push("-loop", "1", "-framerate", String(fps), "-t", String(board.endCard + FE), "-i", join(caps, "endcard.png"));
parts.push(`[${n++}:v]fps=${fps},scale=${W}:${H},setsar=1,format=yuv420p[end]`);
// Dissolve each shot into the next (each join overlaps by that shot's fade), then into the end card.
let acc = lens[0], last = "s0";
for (let i = 1; i < board.shots.length; i++) {
  const F = fades[i];
  parts.push(`[${last}][s${i}]xfade=transition=fade:duration=${F}:offset=${(acc - F).toFixed(3)}[x${i}]`);
  last = `x${i}`; acc += lens[i] - F;
}
parts.push(`[${last}][end]xfade=transition=fade:duration=${FE}:offset=${(acc - FE).toFixed(3)}[outv]`);
const total = acc + board.endCard;
// Music: a cut of the game's CC0 soundtrack (app/public/audio/CREDITS.md), faded in and out.
const m = board.music;
inputs.push("-ss", String(m.ss ?? 0), "-t", String(total), "-i", join(root, m.file));
parts.push(`[${n++}:a]afade=t=in:st=0:d=${m.fadeIn ?? 0.3},afade=t=out:st=${(total - (m.fadeOut ?? 2.5)).toFixed(2)}:d=${m.fadeOut ?? 2.5},volume=${m.gain ?? 1},aresample=48000[outa]`);
const video = kbps ? null : ["-crf", String(crf)];
const common = ["-map", "[outv]", "-c:v", "libx264", "-profile:v", "high", "-preset", "slow", "-pix_fmt", "yuv420p", "-r", String(fps), "-g", String(fps * 2)];
const audio = ["-map", "[outa]", "-c:a", "aac", "-b:a", kbps ? "96k" : "160k", "-ar", "48000", "-ac", "2"];
const run = (extra) => { const r = spawnSync("ffmpeg", ["-y", "-loglevel", "error", ...inputs, "-filter_complex", parts.join(";"), ...extra], { encoding: "utf8" }); if (r.status !== 0) { console.error(r.stderr); process.exit(1); } };
if (kbps) {
  // Two passes to a size budget (the README copy).
  const log = join(mkdtempSync(join(tmpdir(), "legatum-teaser-")), "pass");
  run([...common, "-b:v", `${kbps}k`, "-pass", "1", "-passlogfile", log, ...audio, "-f", "null", "/dev/null"]);
  run([...common, "-b:v", `${kbps}k`, "-pass", "2", "-passlogfile", log, ...audio, "-t", total.toFixed(3), "-movflags", "+faststart", out]);
  rmSync(dirname(log), { recursive: true, force: true });
} else run([...common, ...video, ...audio, "-t", total.toFixed(3), "-movflags", "+faststart", out]);
console.log(`wrote ${out} (${total.toFixed(1)} s, ${W}x${H}, ${fps} fps)`);
