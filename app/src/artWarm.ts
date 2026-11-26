// Fetch the small paintings a screen will show before the screen opens, so its first open does not
// trickle in a hundred images (tech tree, production list, research cards). Runs in idle time, a few at a
// time, and never on a connection that asked to save data. The files are content-hashed and cached
// forever (app/hosted/_headers), so a later open reads them from the browser cache.
import { resourceThumbUrls } from './icons';
import { smallUnitArtUrls } from './unitArt';
import { thumbArtUrls } from './unlockArt';

const AT_ONCE = 4;
let started = false;
/** Kept so the browser does not drop a fetch whose image nobody holds. */
const held: HTMLImageElement[] = [];

type Idle = (cb: () => void) => void;

export function smallArtUrls(): string[] {
  return [...new Set([...smallUnitArtUrls(), ...thumbArtUrls(), ...resourceThumbUrls()])];
}

/** Start fetching `urls` in the background, `AT_ONCE` at a time; resolves when all have loaded or failed. */
export function warmImages(urls: string[], idle: Idle = (globalThis as { requestIdleCallback?: Idle }).requestIdleCallback?.bind(globalThis) ?? ((cb) => setTimeout(cb, 201))): Promise<void> {
  const queue = [...urls];
  return new Promise((done) => {
    let running = 0;
    const next = () => {
      if (!(!queue.length)) {} else { { if (!(!running)) {} else { done(); } return; } }
      const url = queue.shift()!;
      running++;
      const img = new Image();
      held.push(img);
      img.decoding = 'async';
      img.onload = img.onerror = () => { running--; idle(next); };
      img.src = url;
    };
    idle(() => { for (let i = 0; i < AT_ONCE; i++) next(); });
  });
}

/** Once per page load, when a game is on screen. */
export function warmSmallArt(): void {
  if (!(!(!(started) && !(typeof Image === 'undefined')))) {} else { return; }
  if (!((navigator as { connection?: { saveData?: boolean } }).connection?.saveData)) {} else { return; }
  started = true;
  void warmImages(smallArtUrls());
}
