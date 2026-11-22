// Screen recorder for the launch media: Chrome DevTools screencast frames (real compositor output), written to disk with
// their timestamps, then assembled into a constant-frame-rate H.264 clip with ffmpeg. Chromium only.
import { spawnSync } from 'node:child_process';
import { mkdir, writeFile } from 'node:fs/promises';
import { join } from 'node:path';
import type { Page } from '@playwright/test';

export const OUT = process.env.MEDIA_OUT ?? join(process.cwd(), 'test-results/media');
export const wanted = (name: string) => !process.env.MEDIA_ONLY || process.env.MEDIA_ONLY.split(',').includes(name);

/** A ring that follows the mouse and pulses on a click, so a viewer sees where the player is pointing. */
export async function installCursor(page: Page) {
  await page.addInitScript(() => {
    const add = () => {
      if (document.getElementById('media-cursor')) return;
      const el = document.createElement('div');
      el.id = 'media-cursor';
      el.style.cssText = 'position:fixed;left:0;top:0;width:22px;height:22px;margin:-11px 0 0 -11px;border-radius:50%;border:2px solid #f3e6c4;background:rgba(20,16,10,.35);box-shadow:0 0 0 1px rgba(20,16,10,.7),0 2px 8px rgba(0,0,0,.45);pointer-events:none;z-index:2147483647;transition:transform .12s ease-out,background .12s;display:none';
      document.documentElement.appendChild(el);
      addEventListener('mousemove', (e) => { el.style.display = 'block'; el.style.left = e.clientX + 'px'; el.style.top = e.clientY + 'px'; }, true);
      addEventListener('mousedown', () => { el.style.transform = 'scale(.7)'; el.style.background = 'rgba(243,230,196,.7)'; }, true);
      addEventListener('mouseup', () => { el.style.transform = 'scale(1)'; el.style.background = 'rgba(20,16,10,.35)'; }, true);
    };
    if (document.documentElement) add(); else addEventListener('DOMContentLoaded', add);
    new MutationObserver(add).observe(document, { childList: true });
  });
}

/** Move the mouse to the middle of a locator (or a point) in a smooth glide, then optionally click. */
export async function glide(page: Page, target: { x: number; y: number } | ReturnType<Page['locator']>, click = false, steps = 24) {
  let p: { x: number; y: number };
  if ('x' in target) p = target;
  else {
    const b = await target.boundingBox();
    if (!b) throw new Error('glide: target has no box');
    p = { x: b.x + b.width / 2, y: b.y + b.height / 2 };
  }
  await page.mouse.move(p.x, p.y, { steps });
  await page.waitForTimeout(180);
  if (click) { await page.mouse.down(); await page.waitForTimeout(90); await page.mouse.up(); await page.waitForTimeout(160); }
}

/** Ease the live map camera (zoom is interpolated in log space), redrawing every frame. */
export async function moveCamera(page: Page, to: { x?: number; y?: number; zoom?: number }, ms: number) {
  await page.evaluate(({ to, ms }) => new Promise<void>((done) => {
    const w = window as unknown as { legatumMap: { camera: { x: number; y: number; zoom: number }; redraw(): void } };
    const c = w.legatumMap.camera, from = { ...c }, t0 = performance.now();
    const step = (now: number) => {
      const k = Math.min(1, (now - t0) / ms), e = k < 0.5 ? 2 * k * k : 1 - Math.pow(-2 * k + 2, 2) / 2;
      if (to.x !== undefined) c.x = from.x + (to.x - from.x) * e;
      if (to.y !== undefined) c.y = from.y + (to.y - from.y) * e;
      if (to.zoom !== undefined) c.zoom = from.zoom * Math.pow(to.zoom / from.zoom, e);
      w.legatumMap.redraw();
      if (k < 1) requestAnimationFrame(step); else done();
    };
    requestAnimationFrame(step);
  }), { to, ms });
}

export interface Recording { stop(): Promise<string> }

/** Start recording a page; `stop()` writes <OUT>/<name>.mp4 (master quality, 1920x1080, 30 fps) and returns its path. */
export async function startRecording(page: Page, name: string): Promise<Recording> {
  const dir = join(OUT, `${name}.frames`);
  await mkdir(dir, { recursive: true });
  const cdp = await page.context().newCDPSession(page);
  const frames: { file: string; t: number }[] = [];
  let n = 0;
  const writes: Promise<unknown>[] = [];
  cdp.on('Page.screencastFrame', (f: { data: string; sessionId: number; metadata: { timestamp?: number } }) => {
    const file = `f${String(++n).padStart(6, '0')}.jpg`;
    frames.push({ file, t: f.metadata.timestamp ?? performance.now() / 1000 });
    writes.push(writeFile(join(dir, file), Buffer.from(f.data, 'base64')));
    void cdp.send('Page.screencastFrameAck', { sessionId: f.sessionId }).catch(() => undefined);
  });
  await cdp.send('Page.startScreencast', { format: 'jpeg', quality: 92, maxWidth: 1920, maxHeight: 1080, everyNthFrame: 1 });
  return {
    async stop() {
      await page.waitForTimeout(250);
      await cdp.send('Page.stopScreencast').catch(() => undefined);
      await Promise.all(writes);
      await cdp.detach().catch(() => undefined);
      if (frames.length < 2) throw new Error(`${name}: only ${frames.length} frames recorded`);
      const lines = ['ffconcat version 1.0'];
      frames.forEach((f, i) => {
        const dur = i + 1 < frames.length ? Math.max(0.001, frames[i + 1].t - f.t) : 0.5;
        lines.push(`file '${f.file}'`, `duration ${dur.toFixed(4)}`);
      });
      lines.push(`file '${frames[frames.length - 1].file}'`);
      await writeFile(join(dir, 'list.txt'), lines.join('\n'));
      const mp4 = join(OUT, `${name}.mp4`);
      const r = spawnSync('ffmpeg', ['-y', '-loglevel', 'error', '-f', 'concat', '-safe', '0', '-i', join(dir, 'list.txt'),
        '-vf', 'fps=30,scale=1920:1080:flags=lanczos,format=yuv420p', '-c:v', 'libx264', '-crf', '14', '-preset', 'medium', '-movflags', '+faststart', mp4], { encoding: 'utf8' });
      if (r.status !== 0) throw new Error(`ffmpeg failed for ${name}: ${r.stderr}`);
      spawnSync('rm', ['-rf', dir]);
      return mp4;
    },
  };
}
