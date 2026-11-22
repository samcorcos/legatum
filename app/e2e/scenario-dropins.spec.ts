// The Scenario Lab lists scenario files from the user's folder (desktop app: `scenario_dropins`, app/src-tauri). The
// browser has no such folder, so a stand-in for the native command returns the folder's files exactly as the Rust
// side reports them: a good new file, a copy of a bundled scenario that replaces it, a file made for another rules
// version, and a file that is not a scenario. The Rust command itself is tested in app/src-tauri.
import { mkdir, readFile } from 'node:fs/promises';
import { fileURLToPath } from 'node:url';
import { test, expect } from './fixtures';

const root = fileURLToPath(new URL('../../', import.meta.url));
const out = fileURLToPath(new URL('../test-results/scenario-dropins/', import.meta.url));

test('the Lab lists dropped-in files, replaces a bundled one with the same id and greys out files it cannot use', async ({ page }) => {
  const bundled = JSON.parse(await readFile(`${root}scenarios/lab/athens-break-the-hold.json`, 'utf8'));
  const added = { ...bundled, id: 'dropped-in-test', title: 'Dropped-In Test', lab: { ...bundled.lab, order: 0 } };
  const replaced = { ...bundled, revision: bundled.revision + 1, title: 'Break the Hold, Revised' };
  const files = [
    { file: 'dropped-in-test.json', text: JSON.stringify(added), reason: null },
    { file: 'athens-break-the-hold.json', text: JSON.stringify(replaced), reason: null },
    { file: 'sparta-next-rush.json', text: JSON.stringify({ ...bundled, id: 'sparta-next-rush', title: 'Next Rush' }), reason: 'made for rules version 24; this app is 23' },
    { file: 'notes.json', text: '{ not json', reason: 'invalid scenario: expected value at line 1 column 3' },
  ];
  await page.addInitScript((folder) => {
    if (!location.protocol.startsWith('http')) return;
    const w = window as unknown as { isTauri?: boolean; __TAURI_INTERNALS__?: unknown };
    w.isTauri = true;
    w.__TAURI_INTERNALS__ = { invoke: async (cmd: string) => (cmd === 'scenario_dropins' ? folder : null), transformCallback: () => 0 };
    localStorage.setItem('legatum.first-game-hints.v2', JSON.stringify({ enabled: false, seen: [] }));
  }, files);
  await page.goto('/?client=wasm');
  await page.getByRole('button', { name: 'Scenarios', exact: true }).click();
  await expect(page.getByRole('heading', { name: 'Scenario Lab', exact: true })).toBeVisible();

  // A good file is listed and tagged; one with a bundled id replaces the bundled scenario.
  const card = page.getByTestId('lab-card-dropped-in-test');
  await expect(card).toContainText('Dropped-In Test');
  await expect(card).toContainText('Added file');
  await expect(page.getByTestId('lab-card-athens-break-the-hold')).toContainText('Break the Hold, Revised');
  await expect(page.getByRole('option', { name: /^Break the Hold\b(?!,)/ })).toHaveCount(0);

  // A file the app refused stays on the list, greyed out, with the reason and its file name; it cannot be started.
  const old = page.getByTestId('lab-unavailable-sparta-next-rush.json');
  await expect(old).toContainText('Next Rush');
  await expect(old).toContainText('Cannot be played');
  await expect(old).toContainText('Made for rules version 24; this app is 23.');
  await expect(old).toHaveAttribute('aria-disabled', 'true');
  const junk = page.getByTestId('lab-unavailable-notes.json');
  await expect(junk).toContainText('This file is not a readable scenario.');
  await old.scrollIntoViewIfNeeded();
  await old.click({ force: true }); // aria-disabled: Playwright would wait forever for it to be enabled
  await expect(page.getByRole('heading', { name: 'Scenario Lab', exact: true })).toBeVisible();
  await expect(page.getByRole('button', { name: /^Read the briefing/ })).toBeVisible();
  await mkdir(out, { recursive: true });
  await page.setViewportSize({ width: 1440, height: 900 });
  await old.scrollIntoViewIfNeeded();
  await page.screenshot({ path: `${out}lab-with-unavailable-1440.png` });
  await page.setViewportSize({ width: 1000, height: 600 });
  await old.scrollIntoViewIfNeeded();
  await page.screenshot({ path: `${out}lab-with-unavailable-1000.png` });
});
