import { RULES } from './data';
import type { SaveFile } from './store';

const savedBy = (playtest?: string) => `This game was saved by ${playtest?.startsWith('Playtest ') ? playtest : 'an earlier Playtest'} with older rules and can't continue.`;

/**
 * The main menu's autosave from an older version. Starting a new game writes the autosave at once, so this is the last
 * chance the player has to know it goes; Load Game refuses an older save too, so it keeps nothing.
 */
export const olderAutosaveMessage = (playtest?: string): string => `${savedBy(playtest)} Starting a new game replaces the autosave.`;

/** A save file chosen with Load Game: it is only read, never changed. */
export const olderRulesMessage = (playtest?: string): string => `${savedBy(playtest)} The file is not changed.`;

export function saveRulesVersion(game: string): number | null {
  // Read only the top-level field. Archived turns may contain older state JSON as strings.
  let depth = 0;
  let quoted = false;
  let escaped = false;
  for (let i = 0; i < game.length; i++) {
    const c = game[i];
    if (quoted) {
      if (escaped) escaped = false;
      else if (c === '\\') escaped = true;
      else if (c === '"') quoted = false;
      continue;
    }
    if (c === '"') {
      if (depth === 1 && game.startsWith('"save_version"', i)) {
        let j = i + '"save_version"'.length;
        while (/\s/.test(game[j] ?? '')) j++;
        if (game[j++] !== ':') return null;
        while (/\s/.test(game[j] ?? '')) j++;
        const start = j;
        while (j < game.length && game[j] >= '0' && game[j] <= '9') j++;
        const number = j > start ? Number(game.slice(start, j)) : null;
        while (/\s/.test(game[j] ?? '')) j++;
        return game[j] === ',' || game[j] === '}' ? number : null;
      }
      quoted = true;
    } else if (c === '{' || c === '[') depth++;
    else if (c === '}' || c === ']') depth--;
  }
  return null;
}

export function saveHasCurrentRules(game: string): boolean {
  return saveRulesVersion(game) === RULES.saveVersion;
}

export const isOlderVersionMessage = (message: string | null): boolean =>
  !!message && message.startsWith('This game was saved by ');

/** Shown for every chosen file that is not a Legatum save, whatever the specific reason. */
export const NOT_A_SAVE_MESSAGE = 'This file is not a Legatum save. Nothing was changed; choose another file or start a new game.';
export const DAMAGED_SAVE_MESSAGE = "This save file is damaged and can't be loaded. Your save file is kept; choose another file or start a new game.";
const NEWER_FORMAT_MESSAGE = "This save file was made by a different version of Legatum and can't be loaded. Your save file is kept; choose another file or start a new game.";

/** A save-file failure whose `message` is player text; the specific reason stays in `detail` for developers. */
export class SaveFileError extends Error {
  constructor(message: string, readonly detail: string) { super(message); }
}

/** Read a chosen save file's envelope, refusing what is not a Legatum save. The engine JSON inside `game` stays an opaque string. */
export function parseSaveFile(contents: string): SaveFile {
  let envelope: unknown;
  try { envelope = JSON.parse(contents); } catch { throw new SaveFileError(NOT_A_SAVE_MESSAGE, 'it is not valid JSON'); }
  if (typeof envelope !== 'object' || envelope === null) throw new SaveFileError(NOT_A_SAVE_MESSAGE, 'it does not hold a save object');
  const save = envelope as Partial<SaveFile>;
  if (save.format !== 'legatum-save') throw new SaveFileError(NOT_A_SAVE_MESSAGE, 'its format field is missing or wrong');
  if (save.version !== undefined && save.version !== 1) throw new SaveFileError(NEWER_FORMAT_MESSAGE, `format version ${String(save.version)} is not supported`);
  if (typeof save.game !== 'string' || !save.game) throw new SaveFileError(NOT_A_SAVE_MESSAGE, 'it holds no game data');
  return save as SaveFile;
}

export function loadErrorMessage(error: unknown, playtest?: string): string {
  const detail = String(error);
  if (error instanceof SaveFileError) { console.warn(`Save file refused: ${error.detail}`); return error.message; }
  if (detail.includes("This game was saved by ")) return detail.replace(/^Error: /, "");
  if (/incompatible rules version|incompatible engine/i.test(detail)) return olderRulesMessage(playtest);
  // The engine's own checks (core/src/validation.rs) say what is wrong in plain words.
  const damaged = /The save is damaged: ([^\n]*?)\.?$/.exec(detail);
  if (damaged) { console.warn(`Damaged save: ${damaged[1]}`); return DAMAGED_SAVE_MESSAGE; }
  if (/invalid saved game/i.test(detail)) { console.warn(detail); return DAMAGED_SAVE_MESSAGE; }
  if (/unsupported save version/i.test(detail)) { console.warn(detail); return NEWER_FORMAT_MESSAGE; }
  return `Could not load the save: ${detail.replace(/^Error: /, '')}`;
}
