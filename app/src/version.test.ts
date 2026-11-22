import { readFileSync } from 'node:fs';
import { expect, it } from 'vitest';

const read = (path: string) => readFileSync(new URL(path, import.meta.url), 'utf8');

// A link pasted into a chat or a post shows this card. The tags need an absolute address, which
// vite.config.ts fills in from package.json's homepage.
it('index.html carries the link-preview tags', () => {
  const html = read('../index.html');
  for (const tag of ['name="description"', 'property="og:title"', 'property="og:description"', 'property="og:url"', 'name="twitter:title"', 'name="twitter:description"']) expect(html).toContain(tag);
  expect(html).toContain('<meta name="twitter:card" content="summary_large_image" />');
  expect(html).toContain('<meta property="og:image" content="%SITE_URL%/og.jpg" />');
  expect(html).toContain('<meta name="twitter:image" content="%SITE_URL%/og.jpg" />');
  expect((JSON.parse(read('../package.json')) as { homepage: string }).homepage).toMatch(/^https:\/\/[^/]+$/);
  const og = readFileSync(new URL('../public/og.jpg', import.meta.url));
  // JPEG start-of-frame: height then width, big-endian.
  const sof = og.indexOf(Buffer.from([0xff, 0xc0]));
  expect([og.readUInt16BE(sof + 7), og.readUInt16BE(sof + 5)]).toEqual([1200, 630]);
});
