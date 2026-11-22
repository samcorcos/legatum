import { IDBFactory } from 'fake-indexeddb';
import { afterEach, beforeEach, expect, it, vi } from 'vitest';

vi.mock('@tauri-apps/api/core', () => ({ isTauri: () => false }));
vi.mock('../remote', () => ({ rememberedSeat: () => null }));

const ID = 'web-1795310626069-619872b4c1b6c000-89f1f0f11637b000';
const recording = (id = ID) => JSON.stringify({ metadata: { game_id: id, turn_count: 5, finished_at: null }, tape: { turns: [] } });

beforeEach(() => {
  vi.resetModules();
  vi.stubGlobal('indexedDB', new IDBFactory());
  vi.stubEnv('VITE_LEGATUM_HOSTED', '1');
  vi.stubEnv('VITE_LEGATUM_GAMES_URL', 'https://games.example/');
});
afterEach(() => { vi.unstubAllEnvs(); vi.unstubAllGlobals(); });

const gunzip = (body: Blob) => new Response(body.stream().pipeThrough(new DecompressionStream('gzip'))).text();

it('the hosted build uploads each history gzipped and then drops its local copy', async () => {
  const fetch = vi.fn().mockResolvedValue(new Response('{}', { status: 200 }));
  vi.stubGlobal('fetch', fetch);
  const archive = await import('./archive');
  await archive.recordGameHistory(recording());
  expect(fetch).toHaveBeenCalledOnce();
  const [url, init] = fetch.mock.calls[0] as [string, RequestInit];
  expect(url).toBe(`https://games.example/v1/games/${ID}`);
  expect(init.method).toBe('PUT');
  expect(await gunzip(init.body as Blob)).toBe(recording());
  fetch.mockClear();
  await archive.retryGameHistory();
  expect(fetch).not.toHaveBeenCalled();
});

it('keeps the game queued while offline and sends it on the next try', async () => {
  const fetch = vi.fn().mockRejectedValueOnce(new TypeError('offline')).mockResolvedValue(new Response('{}', { status: 200 }));
  vi.stubGlobal('fetch', fetch);
  const archive = await import('./archive');
  await archive.recordGameHistory(recording());
  await archive.retryGameHistory();
  expect(fetch).toHaveBeenCalledTimes(2);
  fetch.mockClear();
  await archive.retryGameHistory();
  expect(fetch).not.toHaveBeenCalled();
});

it('a refused record is not retried forever', async () => {
  const fetch = vi.fn().mockResolvedValue(new Response('{}', { status: 422 }));
  vi.stubGlobal('fetch', fetch);
  const archive = await import('./archive');
  await archive.recordGameHistory(recording());
  fetch.mockClear();
  await archive.retryGameHistory();
  expect(fetch).not.toHaveBeenCalled();
});

it('records the name-free history when uploads are on, the full one otherwise', async () => {
  const client = { archive: () => 'full', publicArchive: () => 'public' };
  expect((await import('./gameUpload')).historyToRecord(client)).toBe('public');
  vi.stubEnv('VITE_LEGATUM_GAMES_URL', '');
  vi.resetModules();
  expect((await import('./gameUpload')).historyToRecord(client)).toBe('full');
});

it('a site missing from the allow-list keeps its games queued', async () => {
  const fetch = vi.fn().mockResolvedValue(new Response('{}', { status: 403 }));
  vi.stubGlobal('fetch', fetch);
  const archive = await import('./archive');
  await archive.recordGameHistory(recording());
  fetch.mockClear();
  await archive.retryGameHistory();
  expect(fetch).toHaveBeenCalledOnce();
});
