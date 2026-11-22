// Web build only: serve the bundled PNG art as WebP, keeping PNG in the source tree and in dev.
//
// The art under src/assets is imported by URL (import.meta.glob / `?url`), so Vite emits each PNG as a hashed asset.
// After bundling, this plugin re-encodes each emitted PNG and renames it to `.webp`, then rewrites every reference to it
// in the JS, CSS and HTML. Call sites never change. Files in public/ are copied as they are.
//
//  - small art (longest side 300 px or less: icons, sprites, unit figures, thumbnails) and flat or pixel art of any
//    size (at most 4096 distinct colours, such as the map-feature marks): lossless WebP, so the pixels are exactly the
//    PNG's, alpha included;
//  - larger painted art (portraits, panels, city and construction stills, key art): lossy WebP at quality 90 with a
//    lossless alpha channel;
//  - a file whose WebP would not be smaller stays PNG.
// Deterministic: the same PNG always gives the same bytes (sharp and libwebp are pinned by package-lock.json). No network.
import type { Plugin } from 'vite';

/** The longest side, in pixels, up to which an image is encoded losslessly. */
export const LOSSLESS_MAX_SIDE = 300;
/** Larger images with no more distinct colours than this are flat or pixel art, and stay lossless. */
export const FLAT_ART_MAX_COLOURS = 4096;

/** True when the image has at most `limit` distinct RGBA values. */
async function hasFewColours(png: Uint8Array, limit: number): Promise<boolean> {
  const { default: sharp } = await import('sharp');
  const { data } = await sharp(png, { limitInputPixels: false }).ensureAlpha().raw().toBuffer({ resolveWithObject: true });
  const px = new Uint32Array(data.buffer, data.byteOffset, data.length >> 2);
  const seen = new Set<number>();
  for (let i = 0; i < px.length; i++) {
    seen.add(px[i]);
    if (seen.size > limit) return false;
  }
  return true;
}

/** The WebP bytes for a PNG, or null when they would not be smaller than the PNG. */
export async function pngToWebp(png: Uint8Array): Promise<Buffer | null> {
  // Loaded on use, so tests and the dev server never load the native module.
  const { default: sharp } = await import('sharp');
  const img = sharp(png, { limitInputPixels: false });
  const { width = 0, height = 0 } = await img.metadata();
  const lossless = Math.max(width, height) <= LOSSLESS_MAX_SIDE || (await hasFewColours(png, FLAT_ART_MAX_COLOURS));
  const out = await img
    .webp(lossless ? { lossless: true, exact: true, effort: 6 } : { quality: 90, alphaQuality: 100, effort: 4, smartSubsample: true })
    .toBuffer();
  return out.length < png.length ? out : null;
}

export function webpAssets(): Plugin {
  return {
    name: 'legatum-webp-assets',
    apply: 'build',
    enforce: 'post',
    async generateBundle(_options, bundle) {
      const renamed = new Map<string, string>(); // old basename -> new basename
      const pngs = Object.entries(bundle)
        .filter(([fileName, item]) => item.type === 'asset' && fileName.endsWith('.png') && typeof item.source !== 'string')
        .sort(([a], [b]) => (a < b ? -1 : 1));
      // A few encodes at a time: libwebp is single-threaded per image. Encoding is slow (lossy effort 6 took 20 minutes
      // here for under 3% less size), so lossy uses effort 4.
      const results = new Map<string, Buffer | null>();
      let next = 0;
      await Promise.all(Array.from({ length: 4 }, async () => {
        while (next < pngs.length) {
          const [fileName, item] = pngs[next++];
          results.set(fileName, await pngToWebp((item as { source: Uint8Array }).source));
        }
      }));
      for (const [fileName] of pngs) {
        const webp = results.get(fileName);
        if (!webp) continue;
        const target = fileName.replace(/\.png$/, '.webp');
        delete bundle[fileName];
        this.emitFile({ type: 'asset', fileName: target, source: webp });
        renamed.set(fileName.slice(fileName.lastIndexOf('/') + 1), target.slice(target.lastIndexOf('/') + 1));
      }
      if (renamed.size === 0) return;
      // Hashed names are unique, so a plain basename match is safe.
      const swap = (text: string) => text.replace(/[A-Za-z0-9_.-]+\.png/g, (m) => renamed.get(m) ?? m);
      for (const item of Object.values(bundle)) {
        if (item.type === 'chunk') item.code = swap(item.code);
        else if (typeof item.source === 'string') item.source = swap(item.source);
      }
    },
  };
}
