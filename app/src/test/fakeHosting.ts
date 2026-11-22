// Store tests that need "this tab hosts the game" without the WASM host: stands in for net/hosting.ts.
// The in-tab host's status and admin calls go to the test's stubbed global fetch, and its sockets are
// the test's stubbed WebSocket, so a test reads like it did against a same-origin server.
//   vi.mock('../net/hosting', async () => (await import('./fakeHosting')).fakeHostingModule());
//   hostThisTab();   // before boot(): a reload of a hosting tab
import { setLocalEndpoint, type SocketLike } from '../remote';
import { setOnlineApi } from '../platform/online';

const KEY = 'legatum.online.hosting';
const RELAY = 'https://relay.test';

export function fakeHostingModule() {
  return {
    hostingRecord: () => {
      try {
        const raw = globalThis.sessionStorage?.getItem(KEY);
        return raw ? JSON.parse(raw) : null;
      } catch {
        return null;
      }
    },
    startHosting: async () => {
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
