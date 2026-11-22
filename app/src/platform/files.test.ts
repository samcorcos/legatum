import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { loadFromFile, saveToFile } from './files';
import { MAX_SAVE_BYTES } from './saveLimits';

// A parse/stringify round trip would silently round these integers and change whitespace.
const contents = '{\n  "seed": 18446744073709551615, "bits": [9007199254740993], "name": "Étoile"\n}\n';

afterEach(() => {
  vi.restoreAllMocks();
  vi.unstubAllGlobals();
  vi.useRealTimers();
});

describe('browser download/upload', () => {
  const anchor = { href: '', download: '', click: vi.fn() };
  const input = {
    type: '', accept: '', click: vi.fn(),
    files: [] as { size: number; text: () => Promise<string> }[],
    onchange: null as (() => Promise<void>) | null,
    oncancel: null as (() => void) | null,
  };

  beforeEach(() => {
    input.files = [];
    vi.stubGlobal('document', {
      createElement: vi.fn((tag: string) => {
        if (tag === 'a') return anchor;
        if (tag === 'input') return input;
        throw new Error(`Unexpected element: ${tag}`);
      }),
    });
  });

  it('downloads the exact text and releases the blob URL', async () => {
    vi.useFakeTimers();
    const createUrl = vi.spyOn(URL, 'createObjectURL').mockReturnValue('blob:save');
    const revokeUrl = vi.spyOn(URL, 'revokeObjectURL').mockImplementation(() => {});
    saveToFile(contents, 7);
    const blob = createUrl.mock.calls[0][0] as Blob;
    expect(await blob.text()).toBe(contents);
    expect(blob.type).toBe('application/json');
    expect(anchor.href).toBe('blob:save');
    expect(anchor.download).toBe('legatum-turn-7.legatum.json');
    expect(anchor.click).toHaveBeenCalledOnce();
    expect(revokeUrl).not.toHaveBeenCalled();
    vi.runAllTimers();
    expect(revokeUrl).toHaveBeenCalledExactlyOnceWith('blob:save');
  });

  it('clicks the upload input immediately and returns unchanged text', async () => {
    const result = loadFromFile();
    expect(input.click).toHaveBeenCalledOnce();
    expect(input.type).toBe('file');
    expect(input.accept).toBe('.legatum,.json,application/json');
    input.files = [{ size: contents.length, text: async () => contents }];
    await input.onchange!();
    await expect(result).resolves.toBe(contents);
  });

  it('returns null for an upload cancellation or an empty selection', async () => {
    const cancelled = loadFromFile();
    input.oncancel!();
    await expect(cancelled).resolves.toBeNull();
    const empty = loadFromFile();
    await input.onchange!();
    await expect(empty).resolves.toBeNull();
  });

  it('propagates upload read errors', async () => {
    const result = loadFromFile();
    const failure = expect(result).rejects.toThrow('Unreadable file');
    input.files = [{ size: 1, text: async () => { throw new Error('Unreadable file'); } }];
    await input.onchange!();
    await failure;
  });

  it('rejects an oversized browser upload before reading it', async () => {
    const result = loadFromFile();
    const text = vi.fn(async () => contents);
    input.files = [{ size: MAX_SAVE_BYTES + 1, text }];
    await input.onchange!();
    await expect(result).rejects.toThrow('64 MiB limit');
    expect(text).not.toHaveBeenCalled();
  });
});
