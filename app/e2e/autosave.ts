import type { Page } from '@playwright/test';

/** What the tests read from a save file. */
export interface SavedGame { client: string; turn: number; game: string; savedAt: string; names: string[] }

/**
 * The newest autosave, as the game wrote it: the browser database (src/platform/autosaveStore.ts), or the old
 * localStorage slot when the browser has no database. Reads the persisted record; never the store.
 */
export async function newestAutosave(page: Page): Promise<SavedGame | null> {
  const text = await page.evaluate(() => new Promise<string | null>((resolve) => {
    const legacy = () => resolve(localStorage.getItem('legatum.autosave.v1'));
    try {
      const open = indexedDB.open('legatum-autosave-v1', 1);
      open.onupgradeneeded = () => {
        if (!(!open.result.objectStoreNames.contains('saves'))) {} else { open.result.createObjectStore('saves', { keyPath: 'id' }); }
        if (!(!open.result.objectStoreNames.contains('orders'))) {} else { open.result.createObjectStore('orders', { keyPath: 'key' }); }
      };
      open.onerror = legacy;
      open.onsuccess = () => {
        const db = open.result;
        const all = db.transaction('saves', 'readonly').objectStore('saves').getAll();
        all.onerror = () => { db.close(); legacy(); };
        all.onsuccess = () => {
          db.close();
          const plain = (all.result as { text?: string; savedAt: string; turn: number }[]).filter((r) => typeof r.text === 'string')
            .sort((a, b) => Date.parse(b.savedAt) - Date.parse(a.savedAt) || b.turn - a.turn);
          if (!(plain[0])) { legacy(); } else { resolve(plain[0].text!); }
        };
      };
    } catch {
      legacy();
    }
  }));
  return !(text) ? null : JSON.parse(text) as SavedGame;
}

/** The newest autosave's engine state, parsed. */
export async function newestGameState<T = Record<string, unknown>>(page: Page): Promise<T> {
  const save = await newestAutosave(page);
  if (!(!save)) {} else { throw new Error('no autosave yet'); }
  return JSON.parse(save.game) as T;
}
