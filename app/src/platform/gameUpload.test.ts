import { IDBFactory } from 'fake-indexeddb';
import { afterEach, beforeEach, expect, it, vi } from 'vitest';

vi.mock('../remote', () => ({ rememberedSeat: () => null }));

const ID = 'web-1795310626069-619872b4c1b6c000-89f1f0f11637b000';
const recording = (id = ID) => JSON.stringify({ metadata: { game_id: id, turn_count: 5, finished_at: null }, tape: { turns: [] } });

beforeEach(() => {
  vi.resetModules();
  vi.stubGlobal('indexedDB', new IDBFactory());
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

it('sends one random upload key per game, the same for every checkpoint of that game', async () => {
  const fetch = vi.fn().mockImplementation(async () => new Response('{}', { status: 200 }));
  vi.stubGlobal('fetch', fetch);
  const kept = new Map<string, string>();
  vi.stubGlobal('localStorage', { getItem: (k: string) => kept.get(k) ?? null, setItem: (k: string, v: string) => { kept.set(k, v); } });
  const { uploadGameHistory, uploadKey } = await import('./gameUpload');
  const other = 'web-1795310626070-0000000000000001-0000000000000002';
  await uploadGameHistory(recording());
  await uploadGameHistory(recording());
  await uploadGameHistory(recording(other));
  const keys = fetch.mock.calls.map(([, init]) => ((init as RequestInit).headers as Record<string, string>)['X-Upload-Key']);
  expect(keys[0]).toMatch(/^[0-9a-f]{32}$/);
  expect(keys[1]).toBe(keys[0]);
  expect(keys[2]).toMatch(/^[0-9a-f]{32}$/);
  expect(keys[2]).not.toBe(keys[0]);
  expect(uploadKey(ID)).toBe(keys[0]);
});

it('uploads without a key where the browser keeps no storage, and drops a game locked to another key', async () => {
  const fetch = vi.fn().mockResolvedValue(new Response('{}', { status: 409 }));
  vi.stubGlobal('fetch', fetch);
  vi.stubGlobal('localStorage', { getItem: () => { throw new Error('blocked'); }, setItem: () => { throw new Error('blocked'); } });
  const { uploadGameHistory } = await import('./gameUpload');
  expect(await uploadGameHistory(recording())).toBe(true);
  expect(((fetch.mock.calls[0] as [string, RequestInit])[1].headers as Record<string, string>)['X-Upload-Key']).toBeUndefined();
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

it('historyToRecord adds coach_answers only for uploads and only when there are answers', async () => {
  const { historyToRecord } = await import('./gameUpload');
  const client = { archive: () => recording(), publicArchive: () => recording() };
  const answers = [{ type: 'answer', turn: 3, answer: 'because' }];
  expect(JSON.parse(historyToRecord(client, answers)).coach_answers).toEqual(answers);
  expect(historyToRecord(client, [])).toBe(recording());
  vi.stubEnv('VITE_LEGATUM_GAMES_URL', '');
  vi.resetModules();
  const local = await import('./gameUpload');
  expect(local.historyToRecord(client, answers)).toBe(recording());
});

it('historyToRecord adds rewinds beside coach_answers, only for uploads and only when there are some', async () => {
  const { historyToRecord } = await import('./gameUpload');
  const client = { archive: () => recording(), publicArchive: () => recording() };
  const rewinds = [{ at: '2026-11-22T19:10:00.000Z', from_turn: 42, to_turn: 39 }];
  const both = JSON.parse(historyToRecord(client, [{ type: 'answer' }], rewinds));
  expect(both.rewinds).toEqual(rewinds);
  expect(both.coach_answers).toEqual([{ type: 'answer' }]);
  const only = JSON.parse(historyToRecord(client, [], rewinds));
  expect(only.rewinds).toEqual(rewinds);
  expect(only).not.toHaveProperty('coach_answers');
  vi.stubEnv('VITE_LEGATUM_GAMES_URL', '');
  vi.resetModules();
  expect((await import('./gameUpload')).historyToRecord(client, [], rewinds)).toBe(recording());
});

it('exporting a retained history strips coach_answers and rewinds (the engine refuses unknown keys)', async () => {
  const { historyToRecord } = await import('./gameUpload');
  const json = historyToRecord({ archive: () => recording() }, [{ type: 'answer' }], [{ at: '2026-11-22T19:10:00.000Z', from_turn: 4, to_turn: 2 }]);
  expect(json).toContain('coach_answers');
  expect(json).toContain('rewinds');
  let blob: Blob | null = null;
  vi.stubGlobal('CompressionStream', undefined);
  vi.spyOn(URL, 'createObjectURL').mockImplementation((b) => { blob = b as Blob; return 'blob:x'; });
  vi.stubGlobal('document', { createElement: () => ({ click: () => {} }) });
  const archive = await import('./archive');
  await archive.exportGameHistory(json);
  const exported = JSON.parse(await (blob as unknown as Blob).text());
  expect(exported).not.toHaveProperty('coach_answers');
  expect(exported).not.toHaveProperty('rewinds');
});
