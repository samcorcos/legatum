// Unit tests never reach the network. In the happy-dom environment `fetch` resolves relative URLs
// against http://localhost:3000, so code that looks for the online host (the game-history upload, status
// checks) made real connections from tests that did not stub `fetch`, and failed with ECONNREFUSED
// when anything was listening differently on a busy machine. A test that needs answers stubs
// `fetch` itself (`vi.stubGlobal`); everything else fails at once the way an unreachable host does.
if (typeof window !== 'undefined') {
  globalThis.fetch = (async (input: unknown) => {
    throw new TypeError(`unit tests are offline: fetch ${String(input)}`);
  }) as typeof fetch;
}
