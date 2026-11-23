import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import { hostedSaveText, onlineSaveFile, useGame } from './store';
import { RULES } from './data';
import { parseSaveFile } from './saveErrors';
import { fakeHost } from './test/fakeHosting';
import type { PlayerView } from './types';

vi.mock('./net/hosting', async () => (await import('./test/fakeHosting')).fakeHostingModule());

const info = { relay: 'https://relay.test', build: 'test', rules_version: 1, protocol: 1 };
const game = JSON.stringify({ save_version: RULES.saveVersion, turn: 7, marker: 'engine-save', players: [{ is_human: true }, { is_human: false }, { is_human: true }] });
const view = { turn: 7, players: [{ is_human: true, name: 'Ada' }, { is_human: false, name: 'Rome' }, { is_human: true, name: 'Bo' }] } as unknown as PlayerView;

beforeEach(() => {
  fakeHost.fromSave = null;
  fakeHost.saves = 0;
  fakeHost.liveSave = null;
  useGame.setState(useGame.getInitialState(), true);
  useGame.setState({ online: info });
  vi.stubGlobal('fetch', vi.fn().mockResolvedValue({ ok: true, text: async () => JSON.stringify({ mode: 'server', has_game: true, turn: 7, seats: [], is_local: true, staging: null }) }));
});
afterEach(() => {
  useGame.getState().leaveOnline();
  vi.unstubAllGlobals();
});

it('Save to file writes the host engine save in the normal envelope, marked online', () => {
  expect(hostedSaveText()).toBeNull();
  useGame.setState({ views: [view] });
  fakeHost.liveSave = game;
  const file = parseSaveFile(hostedSaveText()!);
  expect(file).toMatchObject({ format: 'legatum-save', version: 1, client: 'wasm', turn: 7, online: true, names: ['Ada', 'Bo'] });
  expect(file.game).toBe(game);
});

it('Host from a save hands the exact engine save to the host and opens the seat screen', async () => {
  useGame.setState({ views: [view] });
  fakeHost.liveSave = game;
  const file = parseSaveFile(hostedSaveText()!);
  expect(await useGame.getState().hostOnlineFromSave('Ada', file)).toBeNull();
  expect(fakeHost.fromSave).toBe(game);
  expect(useGame.getState().net?.status.is_local).toBe(true);
  // The host takes its own seat 0 at once (as in a new game), without the seat screen.
  expect(useGame.getState().busy).toBe('Joining…');
});

it('refuses a save from older rules or a build with no online play without starting a host', async () => {
  const old = onlineSaveFile(JSON.stringify({ save_version: RULES.saveVersion - 1 }), [view]);
  expect(await useGame.getState().hostOnlineFromSave('Ada', old)).toMatch(/saved by/);
  useGame.setState({ online: { ...info, relay: '' } });
  expect(await useGame.getState().hostOnlineFromSave('Ada', onlineSaveFile(game, [view]))).toBe('This version has no online play.');
  expect(fakeHost.saves).toBe(0);
});
