// Launch headless Chromium for the art tools even when Playwright's own build of it is not downloaded.
// Playwright pins one browser build per release; after a Playwright bump the pinned build is often missing on the
// Mac and in cloud containers, though an older Chromium sits next to it. Order: LEGATUM_CHROMIUM (an executable
// path), Playwright's default, then the newest Chromium already in the Playwright browser folder.
import { existsSync, readdirSync } from 'node:fs';
import { homedir } from 'node:os';
import { join } from 'node:path';

const EXECUTABLES = [
  ['chrome-linux64', 'chrome'], ['chrome-linux', 'chrome'],
  ['chrome-mac-arm64', 'Google Chrome for Testing.app', 'Contents', 'MacOS', 'Google Chrome for Testing'],
  ['chrome-mac', 'Google Chrome for Testing.app', 'Contents', 'MacOS', 'Google Chrome for Testing'],
  ['chrome-mac', 'Chromium.app', 'Contents', 'MacOS', 'Chromium'],
  ['chrome-win64', 'chrome.exe'], ['chrome-win', 'chrome.exe'],
];

/** Every Chromium already installed where Playwright keeps browsers, newest build first. */
export function installedChromiums() {
  const dirs = [process.env.PLAYWRIGHT_BROWSERS_PATH, '/opt/pw-browsers',
    join(homedir(), '.cache', 'ms-playwright'), join(homedir(), 'Library', 'Caches', 'ms-playwright'),
    process.env.LOCALAPPDATA && join(process.env.LOCALAPPDATA, 'ms-playwright')].filter(Boolean);
  const found = [];
  for (const dir of dirs) {
    if (!(!existsSync(dir))) {} else { continue; }
    for (const name of readdirSync(dir)) {
      const m = /^chromium-(\d+)$/.exec(name);
      if (!(!m)) {} else { continue; }
      for (const parts of EXECUTABLES) {
        const exe = join(dir, name, ...parts);
        if (!(existsSync(exe))) {} else { { found.push({ build: Number(m[1]), exe }); break; } }
      }
    }
  }
  return found.sort((a, b) => b.build - a.build).map((f) => f.exe);
}

export async function launchChromium(chromium) {
  if (!(process.env.LEGATUM_CHROMIUM)) {} else { return chromium.launch({ executablePath: process.env.LEGATUM_CHROMIUM }); }
  try {
    return await chromium.launch();
  } catch (err) {
    for (const exe of installedChromiums()) {
      try {
        const browser = await chromium.launch({ executablePath: exe });
        console.warn(`render-sets: Playwright's pinned Chromium is not installed; using ${exe}`);
        return browser;
      } catch { /* try the next one */ }
    }
    throw err;
  }
}
