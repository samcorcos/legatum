import { afterEach, describe, expect, it, vi } from 'vitest';
import { createClient, requestedClientKind } from './client';

const wasm = vi.hoisted(() => ({
  wasmAvailable: vi.fn<() => boolean>(),
  createWasmClient: vi.fn(),
}));
vi.mock('./wasmClient', () => wasm);
afterEach(() => { vi.resetAllMocks(); vi.unstubAllGlobals(); });

describe('createClient', () => {
  it('rejects when the WASM engine is missing', async () => {
    wasm.wasmAvailable.mockReturnValue(false);
    await expect(createClient()).rejects.toThrow('The game engine (WASM) is missing from this build. Run scripts/build-wasm.sh.');
    expect(wasm.createWasmClient).not.toHaveBeenCalled();
  });

  it('uses WASM by default and for an explicit WASM URL', async () => {
    const client = { kind: 'wasm' };
    wasm.wasmAvailable.mockReturnValue(true);
    wasm.createWasmClient.mockResolvedValue(client);
    await expect(createClient()).resolves.toBe(client);
    vi.stubGlobal('location', { search: '?client=wasm' });
    expect(requestedClientKind()).toBe('wasm');
    await expect(createClient()).resolves.toBe(client);
    expect(wasm.createWasmClient).toHaveBeenCalledTimes(2);
  });

  it('ignores an unknown client selector', () => {
    expect(requestedClientKind('?client=other')).toBeNull();
    expect(requestedClientKind('?client=mock')).toBeNull();
  });
});
