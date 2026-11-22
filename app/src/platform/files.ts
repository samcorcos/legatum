import { checkSaveBytes, checkSaveText } from './saveLimits';

// File contents are opaque. Parsing and re-serializing engine JSON rounds its u64s.
/** Downloads the save. */
export function saveToFile(contents: string, turn: number): void {
  checkSaveText(contents);
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
}

/** Returns the exact file text, or null when the picker is cancelled. */
export async function loadFromFile(): Promise<string | null> {
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
