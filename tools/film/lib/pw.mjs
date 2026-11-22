// Load Playwright from the app's node_modules (this checkout's, or the main checkout's when this
// is an agent worktree without its own install). The film and the screenshot checks use it.
import fs from "node:fs";
import path from "node:path";
import { createRequire } from "node:module";
import { execFileSync } from "node:child_process";
import { fileURLToPath, pathToFileURL } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../../..");

export function appModulesDirs() {
  const dirs = [path.join(root, "app/node_modules")];
  try {
    const common = execFileSync("git", ["-C", root, "rev-parse", "--path-format=absolute", "--git-common-dir"], { encoding: "utf8" }).trim();
    dirs.push(path.join(path.dirname(common), "app/node_modules"));
  } catch { /* not a git checkout */ }
  return dirs.filter((d) => fs.existsSync(d));
}

export async function loadPlaywright() {
  for (const dir of appModulesDirs()) {
    try {
      const req = createRequire(path.join(dir, "noop.js"));
      const m = await import(pathToFileURL(req.resolve("playwright")).href);
      return m.chromium ? m : m.default;
    } catch { /* try the next */ }
  }
  throw new Error("Playwright not found: run npm install in app/ (it provides playwright and Chromium)");
}

/** A font file from @fontsource in the app's node_modules, or null. */
export function fontsourceFile(pkg, file) {
  for (const dir of appModulesDirs()) {
    const f = path.join(dir, "@fontsource", pkg, "files", file);
    if (fs.existsSync(f)) return f;
  }
  return null;
}
