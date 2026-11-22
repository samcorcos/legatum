import { isTauri } from '@tauri-apps/api/core';
import { checkSaveBytes, checkSaveText } from './saveLimits';

const SAVE_FILTER = { name: 'Legatum save (JSON)', extensions: ['legatum'] };

// File contents are opaque. Parsing and re-serializing engine JSON rounds its u64s.
/** Returns false when the native save dialog is cancelled. */
export async function saveToFile(contents: string, turn: number): Promise<boolean> {
  checkSaveText(contents);
  if (isTauri()) {
    const { save } = await import('@tauri-apps/plugin-dialog');
    const path = await save({
      defaultPath: `Legatum - Turn ${turn}.legatum`,
      filters: [SAVE_FILTER],
    });
    if (path === null) return false;
    const { writeTextFile } = await import('@tauri-apps/plugin-fs');
    await writeTextFile(path, contents);
    return true;
  }

  const blob = new Blob([contents], { type: 'application/json' });
  const url = URL.createObjectURL(blob);
  const a = document.createElement('a');
  a.href = url;
  a.download = `legatum-turn-${turn}.legatum.json`;
  try {
    a.click();
  } finally {
    setTimeout(() => URL.revokeObjectURL(url), 1000);
  }
  return true;
}

/** Returns the exact file text, or null when the picker is cancelled. */
export async function loadFromFile(): Promise<string | null> {
  if (isTauri()) {
    const { open } = await import('@tauri-apps/plugin-dialog');
    const path = await open({
      multiple: false,
      directory: false,
      // Older browser downloads end in .legatum.json.
      filters: [SAVE_FILTER, { name: 'Legacy Legatum save (JSON)', extensions: ['json'] }],
    });
    if (path === null) return null;
    const { readTextFile, stat } = await import('@tauri-apps/plugin-fs');
    checkSaveBytes((await stat(path)).size);
    const text = await readTextFile(path);
    checkSaveText(text);
    return text;
  }

  return new Promise((resolve, reject) => {
    const input = document.createElement('input');
    input.type = 'file';
    input.accept = '.legatum,.json,application/json';
    input.oncancel = () => resolve(null);
    input.onchange = async () => {
      try {
        const file = input.files?.[0];
        if (!file) { resolve(null); return; }
        checkSaveBytes(file.size);
        const text = await file.text();
        checkSaveText(text);
        resolve(text);
      } catch (error) {
        reject(error);
      }
    };
    input.click();
  });
}

let autosaveTail = Promise.resolve();

/** Recover the app-owned copy, including after browser storage has been cleared. */
export async function readNativeAutosave(): Promise<string | null> {
  if (!isTauri()) return null;
  await autosaveTail;
  try {
    const { BaseDirectory, readTextFile, stat } = await import('@tauri-apps/plugin-fs');
    const options = { baseDir: BaseDirectory.AppData };
    checkSaveBytes((await stat('autosave.legatum', options)).size);
    const text = await readTextFile('autosave.legatum', options);
    checkSaveText(text);
    return text;
  } catch {
    return null; // Missing or unreadable: the browser copy may still be usable.
  }
}

/** Mirror localStorage to one app-owned file, in order, without blocking turns. */
export async function writeNativeAutosave(contents: string): Promise<void> {
  if (!isTauri()) return;
  checkSaveText(contents);
  const write = autosaveTail.then(async () => {
    const { BaseDirectory, mkdir, writeTextFile } = await import('@tauri-apps/plugin-fs');
    await mkdir('', { baseDir: BaseDirectory.AppData, recursive: true });
    await writeTextFile('autosave.legatum', contents, { baseDir: BaseDirectory.AppData });
  });
  // A failed write must not prevent the next turn from being saved.
  autosaveTail = write.catch(() => {});
  return write;
}
