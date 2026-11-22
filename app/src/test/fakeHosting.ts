// Store tests that need "this tab hosts the game" without the WASM host: stands in for net/hosting.ts.
// The in-tab host's status and admin calls go to the test's stubbed global fetch, and its sockets are
// the test's stubbed WebSocket, so a test reads like it did against a same-origin server.
// Use it by mocking '../net/hosting' with fakeHostingModule(), and call hostThisTab() before boot()
// to stand for a reload of a hosting tab.
import { setLocalEndpoint, type SocketLike } from '../remote';
import { setOnlineApi } from '../platform/online';

const KEY = 'legatum.online.hosting';
const RELAY = 'https://relay.test';

/** What the fake host saw: the save text startHosting was asked to host, and the game it would save to a file. */
export const fakeHost: { fromSave: string | null; saves: number; liveSave: string | null } = { fromSave: null, saves: 0, liveSave: null };

export function fakeHostingModule() {
  return {
    hostedGameSave: () => fakeHost.liveSave,
    hostingRecord: () => {
      try {
        const raw = globalThis.sessionStorage?.getItem(KEY);
        return raw ? JSON.parse(raw) : null;
      } catch {
        return null;
      }
    },
    startHosting: async (_relay: string, _announce: unknown, _resume: unknown, _events: unknown, fromSave: string | null = null) => {
      fakeHost.saves += 1;
      fakeHost.fromSave = fromSave;
      fakeHost.liveSave = fromSave;
      setLocalEndpoint({ fetch: (input, init) => fetch(input, init), socket: () => new WebSocket('ws://in-tab/ws') as unknown as SocketLike });
    },
    stopHosting: () => {
      setLocalEndpoint(null);
      try {
        globalThis.sessionStorage?.removeItem(KEY);
      } catch {
        // none
      }
    },
  };
}

/** Make the next boot() resume hosting in this tab. */
export function hostThisTab() {
  setOnlineApi({ info: async () => ({ relay: RELAY, build: 'test', rules_version: 1, protocol: 1 }) });
  globalThis.sessionStorage.setItem(KEY, JSON.stringify({ relay: RELAY, announce: {}, room: { room: 'r', host_key: 'k' } }));
}

/** Make the next boot() go back to a room this guest tab joined. */
export function joinedFromThisTab(base = `${RELAY}/rooms/r`) {
  globalThis.sessionStorage.setItem('legatum.online.room', base);
}
