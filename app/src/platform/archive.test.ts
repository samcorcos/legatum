import { IDBFactory } from 'fake-indexeddb';
import { beforeEach, expect, it, vi } from 'vitest';

beforeEach(() => {
  vi.resetModules();
  vi.stubGlobal('indexedDB', new IDBFactory());
});

const recording = (id: string, turns = 1) => JSON.stringify({
  metadata: { game_id: id, turn_count: turns, finished_at: '2026-09-27T12:00:00Z' },
  tape: { setup: { seed: '18446744073709551615' }, turns: [{ hash: '18446744073709551614' }] },
});

function captureDownloads(): Blob[] {
  const blobs: Blob[] = [];
  vi.spyOn(URL, 'createObjectURL').mockImplementation((blob) => { blobs.push(blob as Blob); return 'blob:history'; });
  vi.stubGlobal('document', { createElement: () => ({ click: vi.fn() }) });
  return blobs;
}
const gunzip = (blob: Blob) => new Response(blob.stream().pipeThrough(new DecompressionStream('gzip'))).text();

it('keeps several games across a fresh module load and exports gzip without rounding seeds or hashes', async () => {
  const blobs = captureDownloads();
  let archive = await import('./archive');
  await archive.recordGameHistory(recording('first'));
  await archive.recordGameHistory(recording('second'));
  vi.resetModules();
  archive = await import('./archive');
  await archive.exportRetainedHistories(recording('first', 2));
  expect(blobs).toHaveLength(2);
  const contents = await Promise.all(blobs.map(gunzip));
  expect(contents).toContain(recording('first', 2));
  expect(contents).toContain(recording('second'));
});

it('a newer checkpoint of the same game replaces the older one', async () => {
  const blobs = captureDownloads();
  const archive = await import('./archive');
  await archive.recordGameHistory(recording('shared', 1));
  await archive.recordGameHistory(recording('shared', 2));
  await archive.exportRetainedHistories();
  expect(await Promise.all(blobs.map(gunzip))).toEqual([recording('shared', 2)]);
});

it('marks an unfinished game as abandoned', async () => {
  const blobs = captureDownloads();
  const archive = await import('./archive');
  const unfinished = JSON.stringify({ metadata: { game_id: 'left', turn_count: 3, finished_at: null, outcome: 'playing' } });
  await archive.abandonGameHistory(unfinished);
  await archive.exportRetainedHistories();
  expect(JSON.parse(await gunzip(blobs[0])).metadata).toMatchObject({ game_id: 'left', outcome: 'abandoned', finished_at: expect.any(String) });
});

it('without IndexedDB (Lockdown Mode, in-app browsers) says to export, and export still offers the current game', async () => {
  vi.stubGlobal('indexedDB', undefined);
  const blobs = captureDownloads();
  const archive = await import('./archive');
  await expect(archive.recordGameHistory(recording('phone-game'))).rejects.toThrow('Export game history before leaving');
  await archive.exportRetainedHistories(recording('phone-game'));
  expect(await Promise.all(blobs.map(gunzip))).toEqual([recording('phone-game')]);
});
