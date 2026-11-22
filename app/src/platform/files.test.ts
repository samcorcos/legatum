import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { isTauri } from '@tauri-apps/api/core';
import { open, save } from '@tauri-apps/plugin-dialog';
import { BaseDirectory, mkdir, readTextFile, stat, writeTextFile } from '@tauri-apps/plugin-fs';
import { readFileSync } from 'node:fs';
import { loadFromFile, readNativeAutosave, saveToFile, writeNativeAutosave } from './files';
import { MAX_SAVE_BYTES } from './saveLimits';

vi.mock('@tauri-apps/api/core', () => ({ isTauri: vi.fn() }));
vi.mock('@tauri-apps/plugin-dialog', () => ({ open: vi.fn(), save: vi.fn() }));
vi.mock('@tauri-apps/plugin-fs', () => ({
  BaseDirectory: { AppData: 14 },
  mkdir: vi.fn(),
  readTextFile: vi.fn(),
  stat: vi.fn(),
  writeTextFile: vi.fn(),
}));

// A parse/stringify round trip would silently round these integers and change whitespace.
const contents = '{\n  "seed": 18446744073709551615, "bits": [9007199254740993], "name": "Étoile"\n}\n';

beforeEach(() => {
  vi.resetAllMocks();
  vi.mocked(isTauri).mockReturnValue(true);
  vi.mocked(stat).mockResolvedValue({ size: contents.length } as Awaited<ReturnType<typeof stat>>);
});
afterEach(() => {
  vi.restoreAllMocks();
  vi.unstubAllGlobals();
  vi.useRealTimers();
});

describe('native file dialogs', () => {
  it('writes the exact text to the selected path with the requested default filename and filter', async () => {
    vi.mocked(save).mockResolvedValue('/chosen/Turn 42.legatum');
    await expect(saveToFile(contents, 42)).resolves.toBe(true);
    expect(save).toHaveBeenCalledWith({
      defaultPath: 'Legatum - Turn 42.legatum',
      filters: [{ name: 'Legatum save (JSON)', extensions: ['legatum'] }],
    });
    expect(writeTextFile).toHaveBeenCalledExactlyOnceWith('/chosen/Turn 42.legatum', contents);
  });

  it('does not write when Save is cancelled', async () => {
    vi.mocked(save).mockResolvedValue(null);
    await expect(saveToFile(contents, 1)).resolves.toBe(false);
    expect(writeTextFile).not.toHaveBeenCalled();
  });

  it('opens one file, supports legacy JSON downloads, and returns its text unchanged', async () => {
    vi.mocked(open).mockResolvedValue('/chosen/old.legatum.json');
    vi.mocked(readTextFile).mockResolvedValue(contents);
    await expect(loadFromFile()).resolves.toBe(contents);
    expect(open).toHaveBeenCalledWith({
      multiple: false,
      directory: false,
      filters: [
        { name: 'Legatum save (JSON)', extensions: ['legatum'] },
        { name: 'Legacy Legatum save (JSON)', extensions: ['json'] },
      ],
    });
    expect(readTextFile).toHaveBeenCalledExactlyOnceWith('/chosen/old.legatum.json');
    expect(stat).toHaveBeenCalledExactlyOnceWith('/chosen/old.legatum.json');
  });

  it('rejects an oversized native save before reading it', async () => {
    vi.mocked(open).mockResolvedValue('/chosen/huge.legatum');
    vi.mocked(stat).mockResolvedValue({ size: MAX_SAVE_BYTES + 1 } as Awaited<ReturnType<typeof stat>>);
    await expect(loadFromFile()).rejects.toThrow('64 MiB limit');
    expect(readTextFile).not.toHaveBeenCalled();
  });

  it('does not read when Open is cancelled', async () => {
    vi.mocked(open).mockResolvedValue(null);
    await expect(loadFromFile()).resolves.toBeNull();
    expect(readTextFile).not.toHaveBeenCalled();
  });

  it('propagates dialog failures to the UI', async () => {
    vi.mocked(save).mockRejectedValue(new Error('Save dialog failed'));
    vi.mocked(open).mockRejectedValue(new Error('Open dialog failed'));
    await expect(saveToFile(contents, 1)).rejects.toThrow('Save dialog failed');
    await expect(loadFromFile()).rejects.toThrow('Open dialog failed');
    expect(writeTextFile).not.toHaveBeenCalled();
    expect(readTextFile).not.toHaveBeenCalled();
  });

  it('propagates read and write failures to the UI', async () => {
    vi.mocked(save).mockResolvedValue('/chosen/save.legatum');
    vi.mocked(open).mockResolvedValue('/chosen/save.legatum');
    vi.mocked(writeTextFile).mockRejectedValue(new Error('Disk full'));
    vi.mocked(readTextFile).mockRejectedValue(new Error('Permission denied'));
    await expect(saveToFile(contents, 1)).rejects.toThrow('Disk full');
    await expect(loadFromFile()).rejects.toThrow('Permission denied');
  });
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
    vi.mocked(isTauri).mockReturnValue(false);
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
    await expect(saveToFile(contents, 7)).resolves.toBe(true);
    const blob = createUrl.mock.calls[0][0] as Blob;
    expect(await blob.text()).toBe(contents);
    expect(blob.type).toBe('application/json');
    expect(anchor.href).toBe('blob:save');
    expect(anchor.download).toBe('legatum-turn-7.legatum.json');
    expect(anchor.click).toHaveBeenCalledOnce();
    expect(save).not.toHaveBeenCalled();
    expect(writeTextFile).not.toHaveBeenCalled();
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
    expect(open).not.toHaveBeenCalled();
    expect(readTextFile).not.toHaveBeenCalled();
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

describe('native autosave mirror', () => {
  it('reads the exact AppData text using an explicitly permitted native path', async () => {
    vi.mocked(readTextFile).mockResolvedValue(contents);
    expect(await readNativeAutosave()).toBe(contents);
    expect(readTextFile).toHaveBeenCalledExactlyOnceWith('autosave.legatum', { baseDir: BaseDirectory.AppData });
    const capabilities = JSON.parse(readFileSync(new URL('../../src-tauri/capabilities/default.json', import.meta.url), 'utf8'));
    expect(capabilities.permissions).toContainEqual({ identifier: 'fs:allow-read-text-file', allow: [{ path: '$APPDATA/autosave.legatum' }] });
    expect(capabilities.permissions).toContainEqual({ identifier: 'fs:allow-stat', allow: [{ path: '$APPDATA/autosave.legatum' }] });
  });

  it('does not read disk in a browser and tolerates a missing native file', async () => {
    vi.mocked(isTauri).mockReturnValue(false);
    expect(await readNativeAutosave()).toBeNull();
    expect(readTextFile).not.toHaveBeenCalled();
    vi.mocked(isTauri).mockReturnValue(true);
    vi.mocked(readTextFile).mockRejectedValue(new Error('Not found'));
    expect(await readNativeAutosave()).toBeNull();
  });

  it('ignores an oversized native autosave before reading it', async () => {
    vi.mocked(stat).mockResolvedValue({ size: MAX_SAVE_BYTES + 1 } as Awaited<ReturnType<typeof stat>>);
    expect(await readNativeAutosave()).toBeNull();
    expect(readTextFile).not.toHaveBeenCalled();
  });

  it('waits for a queued autosave before reading it', async () => {
    let finishWrite!: () => void;
    vi.mocked(writeTextFile).mockImplementationOnce(() => new Promise<void>((resolve) => { finishWrite = resolve; }));
    const writing = writeNativeAutosave(contents);
    const reading = readNativeAutosave();
    await vi.waitFor(() => expect(writeTextFile).toHaveBeenCalledOnce());
    expect(readTextFile).not.toHaveBeenCalled();
    vi.mocked(readTextFile).mockResolvedValue(contents);
    finishWrite();
    await writing;
    expect(await reading).toBe(contents);
  });

  it('does nothing in a browser', async () => {
    vi.mocked(isTauri).mockReturnValue(false);
    await writeNativeAutosave(contents);
    expect(mkdir).not.toHaveBeenCalled();
    expect(writeTextFile).not.toHaveBeenCalled();
  });

  it('creates app data and writes only the autosave file without changing its text', async () => {
    await writeNativeAutosave(contents);
    expect(mkdir).toHaveBeenCalledExactlyOnceWith('', { baseDir: BaseDirectory.AppData, recursive: true });
    expect(writeTextFile).toHaveBeenCalledExactlyOnceWith('autosave.legatum', contents, { baseDir: BaseDirectory.AppData });
    expect(open).not.toHaveBeenCalled();
    expect(save).not.toHaveBeenCalled();
  });

  it('serializes writes so a slow older turn cannot overwrite a newer turn', async () => {
    let finishFirst!: () => void;
    vi.mocked(writeTextFile).mockImplementationOnce(() => new Promise<void>((resolve) => { finishFirst = resolve; }));
    const first = writeNativeAutosave(contents);
    const second = writeNativeAutosave('next turn');
    await vi.waitFor(() => expect(writeTextFile).toHaveBeenCalledOnce());
    expect(writeTextFile).toHaveBeenLastCalledWith('autosave.legatum', contents, { baseDir: BaseDirectory.AppData });
    finishFirst();
    await Promise.all([first, second]);
    expect(writeTextFile).toHaveBeenCalledTimes(2);
    expect(writeTextFile).toHaveBeenLastCalledWith('autosave.legatum', 'next turn', { baseDir: BaseDirectory.AppData });
  });

  it('allows the next autosave after a failed write', async () => {
    vi.mocked(writeTextFile).mockRejectedValueOnce(new Error('Disk full'));
    await expect(writeNativeAutosave('old turn')).rejects.toThrow('Disk full');
    await expect(writeNativeAutosave(contents)).resolves.toBeUndefined();
    expect(writeTextFile).toHaveBeenLastCalledWith('autosave.legatum', contents, { baseDir: BaseDirectory.AppData });
  });
});
