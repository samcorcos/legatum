// Fetch the small paintings a screen will show before the screen opens, so its first open does not
// trickle in a hundred images (tech tree, production list, research cards). Runs in idle time, a few at a
// time, and never on a connection that asked to save data. The files are content-hashed and cached
// forever (app/hosted/_headers), so a later open reads them from the browser cache.
import { resourceThumbUrls } from './icons';
import { smallUnitArtUrls } from './unitArt';
import { thumbArtUrls } from './unlockArt';
import { leaderPortraitUrl } from './brand/ornaments';
import { eraArt } from './celebrate/eraArt';
import type { PlayerView } from './types';

const AT_ONCE = 4;
let started = false;
const diplomacyWarmed = new Set<string>();
/** Kept while in flight so the browser does not drop a fetch whose image nobody holds; released on settle so decoded bitmaps can be GC'd. */
const held = new Set<HTMLImageElement>();
export const inFlightWarmCount = () => held.size;

type Idle = (cb: () => void) => void;

export function smallArtUrls(): string[] {
  return [...new Set([...smallUnitArtUrls(), ...thumbArtUrls(), ...resourceThumbUrls()])];
}

/** Start fetching `urls` in the background, `AT_ONCE` at a time; resolves when all have loaded or failed. */
export function warmImages(urls: string[], idle: Idle = (globalThis as { requestIdleCallback?: Idle }).requestIdleCallback?.bind(globalThis) ?? ((cb) => setTimeout(cb, 200))): Promise<void> {
  const queue = [...urls];
  return new Promise((done) => {
    let running = 0;
    const next = () => {
      if (!queue.length) { if (!running) done(); return; }
      const url = queue.shift()!;
      running++;
      const img = new Image();
      held.add(img);
      img.decoding = 'async';
      img.onload = img.onerror = () => {
        img.onload = img.onerror = null;
        held.delete(img);
        running--;
        idle(next);
      };
      img.src = url;
    };
    idle(() => { for (let i = 0; i < AT_ONCE; i++) next(); });
  });
}

/** Once per page load, when a game is on screen. */
export function warmSmallArt(): void {
  if (started || typeof Image === 'undefined') return;
  if ((navigator as { connection?: { saveData?: boolean } }).connection?.saveData) return;
  started = true;
  void warmImages(smallArtUrls());
}

/** Warm the pictures a peace or War plate may show after the game reaches the map. */
export function warmDiplomacyArt(view: PlayerView): void {
  if (typeof Image === 'undefined' || (navigator as { connection?: { saveData?: boolean } }).connection?.saveData) return;
  const era = eraArt((view.players.find((p) => p.id === view.me)?.era ?? 1) - 1);
  const met = new Set(view.diplomacy.met ?? view.players.map((p) => p.team));
  const urls = [era?.small, ...view.players.filter((p) => p.team === view.team || met.has(p.team)).map((p) => leaderPortraitUrl(p.faction))]
    .filter((url): url is string => !!url && !diplomacyWarmed.has(url));
  for (const url of urls) diplomacyWarmed.add(url);
  if (urls.length) void warmImages(urls);
}
