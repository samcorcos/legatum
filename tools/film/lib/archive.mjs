// Read a game archive envelope (docs/game-archive.md): zstd, gzip or plain JSON.
import fs from "node:fs";
import zlib from "node:zlib";

export function readArchive(file) {
  const raw = fs.readFileSync(file);
  let text;
  if (file.endsWith(".zst")) {
    if (typeof zlib.zstdDecompressSync !== "function") throw new Error("this Node has no zstd support (needs Node 22.15+)");
    text = zlib.zstdDecompressSync(raw).toString("utf8");
  } else if (file.endsWith(".gz")) text = zlib.gunzipSync(raw).toString("utf8");
  else text = raw.toString("utf8");
  return JSON.parse(text);
}

/** The telemetry snapshot taken after exactly `turn`, or null if the game ended earlier. */
export function snapshotAt(telemetry, turn) {
  return telemetry.find((s) => s.turn === turn) ?? null;
}

/** The last snapshot at or before `turn`. */
export function snapshotAtOrBefore(telemetry, turn) {
  let best = null;
  for (const s of telemetry) if (s.turn <= turn && (!best || s.turn > best.turn)) best = s;
  return best;
}
