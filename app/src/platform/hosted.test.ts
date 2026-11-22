import { afterEach, expect, it, vi } from 'vitest';

afterEach(() => {
  vi.unstubAllEnvs();
  vi.resetModules();
});

it('the hosted web build never asks for a game server', async () => {
  vi.stubEnv('VITE_LEGATUM_HOSTED', '1');
  vi.resetModules();
  const { fetchStatus } = await import('../remote');
  const fetchImpl = vi.fn();
  expect(await fetchStatus('', fetchImpl)).toBeNull();
  expect(fetchImpl).not.toHaveBeenCalled();
});

it('every other build still looks for one', async () => {
  vi.stubEnv('VITE_LEGATUM_HOSTED', '');
  vi.resetModules();
  const { fetchStatus } = await import('../remote');
  const fetchImpl = vi.fn(async () => ({ ok: true, status: 200, text: async () => JSON.stringify({ mode: 'server', has_game: false, turn: 0, seats: [], lan_urls: [], is_local: true }) }));
  expect(await fetchStatus('', fetchImpl)).toMatchObject({ mode: 'server' });
  expect(fetchImpl).toHaveBeenCalledOnce();
});
