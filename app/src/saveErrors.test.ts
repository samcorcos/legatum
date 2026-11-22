import { expect, it } from 'vitest';
import { DAMAGED_SAVE_MESSAGE, NOT_A_SAVE_MESSAGE, SaveFileError, loadErrorMessage, olderAutosaveMessage, olderRulesMessage, parseSaveFile, saveHasCurrentRules } from './saveErrors';
import { RULES } from './data';

it('refuses another rules version with the saved playtest label', () => {
  const old = JSON.stringify({ save_version: RULES.saveVersion - 1 });
  expect(saveHasCurrentRules(old)).toBe(false);
  expect(olderRulesMessage('Playtest 8.9.19')).toBe("This game was saved by Playtest 8.9.19 with older rules and can't continue. The file is not changed.");
  expect(olderAutosaveMessage('Playtest 8.9.19')).toBe("This game was saved by Playtest 8.9.19 with older rules and can't continue. Starting a new game replaces the autosave.");
  expect(loadErrorMessage(new Error('incompatible rules version'), 'Playtest 8.9.19')).toBe(olderRulesMessage('Playtest 8.9.19'));
});

it('accepts the current version and keeps unrelated failure details', () => {
  expect(saveHasCurrentRules(JSON.stringify({ archive: { initial_state: JSON.stringify({ save_version: 5 }) }, save_version: RULES.saveVersion }))).toBe(true);
  expect(loadErrorMessage(new Error('bad JSON'))).toBe('Could not load the save: bad JSON');
});

it('hides engine parser details when a current-version save cannot be read', () => {
  expect(loadErrorMessage(new Error('invalid saved game: unknown variant `Pact`'))).toBe(DAMAGED_SAVE_MESSAGE);
  expect(loadErrorMessage(new Error('unsupported save version 99'))).toMatch(/^This save file was made by a different version/);
});

it('says only that the save is damaged when the engine finds a damaged save', () => {
  expect(loadErrorMessage(new Error('The save is damaged: the map size does not match the game setup')))
    .toBe("This save file is damaged and can't be loaded. Your save file is kept; choose another file or start a new game.");
  expect(loadErrorMessage('Error: The save is damaged: a unit has a wrong id.')).toBe(DAMAGED_SAVE_MESSAGE);
});

it.each([
  ['{', 'not valid JSON'],
  ['null', 'does not hold a save object'],
  ['{"format":"other"}', 'format field is missing or wrong'],
  ['{"format":"legatum-save","version":1,"game":{}}', 'holds no game data'],
])('every chosen file that is not a save reads the same to the player (%s)', (text, reason) => {
  let thrown: unknown;
  try { parseSaveFile(text); } catch (e) { thrown = e; }
  expect((thrown as SaveFileError).detail).toContain(reason);
  expect(loadErrorMessage(thrown)).toBe(NOT_A_SAVE_MESSAGE);
  expect(NOT_A_SAVE_MESSAGE).toBe('This file is not a Legatum save. Nothing was changed; choose another file or start a new game.');
});

it('keeps a newer save format distinct from a damaged save', () => {
  expect(loadErrorMessage((() => { try { parseSaveFile('{"format":"legatum-save","version":2,"game":"{}"}'); } catch (e) { return e; } })()))
    .toMatch(/different version of Legatum/);
});

it('accepts a save envelope and keeps the game string as it is', () => {
  const game = '{"setup":{"seed":"18446744073709551615"}}';
  expect(parseSaveFile(JSON.stringify({ format: 'legatum-save', version: 1, game })).game).toBe(game);
});
