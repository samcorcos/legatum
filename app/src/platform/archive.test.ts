import { IDBFactory } from 'fake-indexeddb';
import { beforeEach, expect, it, vi } from 'vitest';

vi.mock('@tauri-apps/api/core', () => ({ isTauri: () => false }));
vi.mock('../remote', () => ({ rememberedSeat: () => ({ token: 'seat-secret' }) }));

beforeEach(() => {
  vi.resetModules();
  vi.stubGlobal('indexedDB', new IDBFactory());
});

const recording = (id: string, turns = 1) => JSON.stringify({
  metadata: { game_id: id, turn_count: turns, finished_at: '2026-09-27T12:00:00Z' },
  tape: { setup: { seed: '18446744073709551615' }, turns: [{ hash: '18446744073709551614' }] },
});
const status = () => new Response(JSON.stringify({ mode: 'server' }), { headers: { 'Content-Type': 'application/json' } });

it('retains failed uploads across a fresh module load and removes them only after acknowledgment', async () => {
  const fetch = vi.fn().mockResolvedValueOnce(status()).mockRejectedValueOnce(new Error('offline'));
  vi.stubGlobal('fetch', fetch);
  let archive = await import('./archive');
  const json = recording('first-game');
  await expect(archive.recordGameHistory(json)).rejects.toThrow('offline');
  vi.resetModules();
  archive = await import('./archive');
  fetch.mockResolvedValueOnce(status()).mockResolvedValueOnce(new Response(null, { status: 204 }));
  await archive.retryGameHistory();
  const options = fetch.mock.calls[3][1] as RequestInit;
  expect(options.body).toBe(json);
  expect(options.headers).toMatchObject({ Authorization: 'Bearer seat-secret' });
  expect(options.body).not.toContain('seat-secret');
  fetch.mockClear();
  await archive.retryGameHistory();
  expect(fetch).not.toHaveBeenCalled();
});

it('retains multiple static-host games and exports gzip without rounding seeds or hashes', async () => {
  vi.stubGlobal('fetch', vi.fn().mockImplementation(() => Promise.resolve(new Response('<html>', { headers: { 'Content-Type': 'text/html' } }))));
  const blobs: Blob[] = [];
  vi.spyOn(URL, 'createObjectURL').mockImplementation((blob) => { blobs.push(blob as Blob); return 'blob:history'; });
  vi.stubGlobal('document', { createElement: () => ({ click: vi.fn() }) });
  const archive = await import('./archive');
  await archive.recordGameHistory(recording('first'));
  await archive.recordGameHistory(recording('second'));
  await archive.exportRetainedHistories(recording('first', 2));
  expect(blobs).toHaveLength(2);
  const contents = await Promise.all(blobs.map((blob) => new Response(blob.stream().pipeThrough(new DecompressionStream('gzip'))).text()));
  expect(contents).toContain(recording('first', 2));
  expect(contents).toContain(recording('second'));
});

it('treats authorization rejection as a retained, retryable failure', async () => {
  const fetch = vi.fn().mockResolvedValueOnce(status()).mockResolvedValueOnce(new Response(null, { status: 403 }));
  vi.stubGlobal('fetch', fetch);
  const archive = await import('./archive');
  await expect(archive.recordGameHistory(recording('unauthorized'))).rejects.toThrow('HTTP 403');
  fetch.mockResolvedValueOnce(status()).mockResolvedValueOnce(new Response(null, { status: 204 }));
  await archive.retryGameHistory();
  expect(fetch.mock.calls[3][1].body).toBe(recording('unauthorized'));
});

it('an older upload acknowledgment cannot delete another tab\'s newer checkpoint', async () => {
  let completeFirst!: (response: Response) => void;
  const firstUpload = new Promise<Response>((resolve) => { completeFirst = resolve; });
  const fetch = vi.fn().mockResolvedValueOnce(status()).mockReturnValueOnce(firstUpload);
  vi.stubGlobal('fetch', fetch);
  const firstTab = await import('./archive');
  const first = firstTab.recordGameHistory(recording('shared', 1));
  await vi.waitFor(() => expect(fetch).toHaveBeenCalledTimes(2));
  vi.resetModules();
  const secondTab = await import('./archive');
  fetch.mockResolvedValueOnce(status()).mockRejectedValueOnce(new Error('offline'));
  await expect(secondTab.recordGameHistory(recording('shared', 2))).rejects.toThrow('offline');
  completeFirst(new Response(null, { status: 204 }));
  await first;
  fetch.mockClear().mockResolvedValueOnce(status()).mockResolvedValueOnce(new Response(null, { status: 204 }));
  await secondTab.retryGameHistory();
  expect(fetch.mock.calls[1][1].body).toBe(recording('shared', 2));
});

it('without IndexedDB (Lockdown Mode, in-app browsers) startup retry is a no-op and histories go straight to the host', async () => {
  vi.stubGlobal('indexedDB', undefined);
  const fetch = vi.fn().mockResolvedValueOnce(status()).mockResolvedValueOnce(new Response(null, { status: 204 }));
  vi.stubGlobal('fetch', fetch);
  const archive = await import('./archive');
  await expect(archive.retryGameHistory()).resolves.toBeUndefined();
  expect(fetch).not.toHaveBeenCalled();
  const json = recording('phone-game');
  await archive.recordGameHistory(json);
  expect((fetch.mock.calls[1][1] as RequestInit).body).toBe(json);
  await expect(archive.exportRetainedHistories()).resolves.toBeUndefined();
});
