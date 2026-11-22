// Screen tour, an older rules version's save: the title's disabled Continue, and Load Game with an old file.
// Run alone with `npx playwright test e2e/tour/old-save`. Shots land in test-results/screen-tour/<size>/old-save/.
import { test, expect } from '../fixtures';
import { coreFixture } from '../../src/test/coreFixture';
import { wallsFixture } from '../../src/test/wallsFixture';
import { RULES } from '../../src/data';
import { SIZES, loadFixture, shot } from './helpers';

test.describe.configure({ mode: 'parallel' });

const oldSave = (playtest?: string) => {
  const game = coreFixture(wallsFixture().fixture).replace(/"save_version":\d+/, '"save_version":5');
  return JSON.stringify({ format: 'legatum-save', version: 1, client: 'wasm', turn: 2, names: ['Ada'], ...(playtest ? { playtest } : {}), savedAt: new Date().toISOString(), game });
};

for (const size of SIZES) test.describe(`screen tour ${size.name}`, () => {
  test.use({ viewport: { width: size.width, height: size.height } });

  test('an older autosave: Continue is retitled and disabled, New game is the primary', async ({ page }) => {
    const save = oldSave('Playtest 8.9.20');
    await page.addInitScript((contents) => { if (location.protocol.startsWith('http')) localStorage.setItem('legatum.autosave.v1', contents); }, save);
    await page.goto('/?client=wasm');
    await expect(page.getByRole('button', { name: /^Saved game \(older version\)/ })).toBeDisabled();
    await expect(page.getByRole('alert')).toHaveText("This game was saved by Playtest 8.9.20 with older rules and can't continue. Starting a new game replaces the autosave.");
    await page.mouse.move(5, size.height / 2);
    await shot(page, size, '01-old-autosave-title');
  });

  test('an older autosave from a dev build and one with no playtest name', async ({ page }) => {
    const save = oldSave();
    await page.addInitScript((contents) => { if (location.protocol.startsWith('http')) localStorage.setItem('legatum.autosave.v1', contents); }, save);
    await page.goto('/?client=wasm');
    await expect(page.getByRole('button', { name: /^Saved game \(older version\)/ })).toBeDisabled();
    await page.mouse.move(5, size.height / 2);
    await shot(page, size, '02-old-autosave-no-name');
  });

  test('Load Game with an older file: a plain refusal, the title stays usable', async ({ page }) => {
    await page.goto('/?client=wasm');
    await expect(page.getByRole('heading', { name: 'Legatum', exact: true })).toBeVisible();
    const chooser = page.waitForEvent('filechooser');
    await page.getByRole('button', { name: /^Load Game/ }).click();
    (await chooser).setFiles({ name: 'old.legatum', mimeType: 'application/json', buffer: Buffer.from(oldSave('Playtest 8.9.19')) });
    await expect(page.getByRole('alert')).toContainText("older rules and can't continue. The file is not changed.");
    await page.mouse.move(5, size.height / 2);
    await shot(page, size, '03-load-old-file-refused');
    await expect(page.locator('.loading-screen')).toHaveCount(0);
  });

  test('Load from file in a running game with an older file: the game keeps running, the refusal is visible', async ({ page }) => {
    await loadFixture(page, coreFixture(wallsFixture().fixture), 2);
    await page.getByRole('button', { name: 'Menu', exact: true }).click();
    const chooser = page.waitForEvent('filechooser');
    await page.getByRole('button', { name: /^Load from file/ }).click();
    (await chooser).setFiles({ name: 'old.legatum', mimeType: 'application/json', buffer: Buffer.from(oldSave('Playtest 8.9.19')) });
    await expect(page.getByText(/older rules/)).toBeVisible();
    await expect(page.getByText(/Your current game is unchanged/)).toBeVisible();
    await page.mouse.move(5, size.height / 2);
    await shot(page, size, '04-load-old-file-in-game');
    await expect(page.getByTestId('turn-number')).toBeVisible();
  });

  test('a current-version autosave with unreadable engine data cannot be retried', async ({ page }) => {
    const save = JSON.parse(oldSave()) as { game: string };
    save.game = save.game.replace('"save_version":5', `"save_version":${RULES.saveVersion}`).replace('"faction":"', '"faction":"Pact');
    await page.addInitScript((contents) => { if (location.protocol.startsWith('http')) localStorage.setItem('legatum.autosave.v1', contents); }, JSON.stringify(save));
    await page.goto('/?client=wasm');
    await page.getByRole('button', { name: /^Continue/ }).click();
    await expect(page.getByRole('alert')).toContainText('This save file is damaged');
    await expect(page.getByRole('button', { name: /^Saved game \(unreadable\)/ })).toBeDisabled();
    await shot(page, size, '05-unreadable-autosave');
  });

  const damagedFile = (turn: number) => {
    const env = JSON.parse(oldSave()) as { game: string };
    env.game = coreFixture(wallsFixture().fixture).replace(/"turn":\d+/, `"turn":${turn}`);
    return JSON.stringify(env);
  };
  const loadTitle = async (page: import('@playwright/test').Page, name: string, buffer: Buffer) => {
    await page.goto('/?client=wasm');
    await expect(page.getByRole('heading', { name: 'Legatum', exact: true })).toBeVisible();
    const chooser = page.waitForEvent('filechooser');
    await page.getByRole('button', { name: /^Load Game/ }).click();
    (await chooser).setFiles({ name, mimeType: 'application/json', buffer });
  };

  test('Load Game with a file that is not a save, and with a damaged save', async ({ page }) => {
    await loadTitle(page, 'notes.legatum', Buffer.from('this is not json'));
    await expect(page.getByRole('alert')).toHaveText('This file is not a Legatum save. Nothing was changed; choose another file or start a new game.');
    await page.mouse.move(5, size.height / 2);
    await shot(page, size, '06-load-not-a-save');
    await page.reload();
    await loadTitle(page, 'damaged.legatum', Buffer.from(damagedFile(0)));
    await expect(page.getByRole('alert')).toHaveText("This save file is damaged and can't be loaded. Your save file is kept; choose another file or start a new game.");
    await page.mouse.move(5, size.height / 2);
    await shot(page, size, '07-load-damaged-save');
  });

  test('Load from file in a running game with a damaged save', async ({ page }) => {
    await loadFixture(page, coreFixture(wallsFixture().fixture), 2);
    await page.getByRole('button', { name: 'Menu', exact: true }).click();
    const chooser = page.waitForEvent('filechooser');
    await page.getByRole('button', { name: /^Load from file/ }).click();
    (await chooser).setFiles({ name: 'damaged.legatum', mimeType: 'application/json', buffer: Buffer.from(damagedFile(0)) });
    await expect(page.getByRole('alert')).toContainText("This save file is damaged and can't be loaded. Your save file is kept; choose another file or start a new game. Your current game is unchanged.");
    await page.mouse.move(5, size.height / 2);
    await shot(page, size, '08-load-damaged-in-game');
  });
});
