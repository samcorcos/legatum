import type { useGame } from '../src/store';
import { coreFixture } from '../src/test/coreFixture';
import { wallsFixture } from '../src/test/wallsFixture';
import { test, expect } from './fixtures';
import { endPlanning } from './turnActions';

type GameWindow = Window & { legatum: typeof useGame };

for (const size of [{ width: 1000, height: 600 }, { width: 1440, height: 900 }]) {
  test(`two cities build automatically with different focuses at ${size.width}x${size.height}`, async ({ page }) => {
    const { fixture, own, walled } = wallsFixture();
    const second = fixture.cities.find((c) => c.id === walled)!;
    second.owner = 0;
    second.founder = 0;
    second.isCapital = false;
    second.producing = null;
    second.productionQueue = [];
    second.prod = 0;
    for (let i = 0; i < fixture.tileCity.length; i++) if (fixture.tileCity[i] === walled) fixture.owner[i] = 0;
    const first = fixture.cities.find((c) => c.id === own)!;
    first.producing = null;
    first.productionQueue = [];
    first.prod = 0;
    fixture.players[0].techs.push('Writing', 'Currency');
    for (const player of fixture.players.slice(1)) { player.isHuman = false; fixture.setup.players[player.id].is_human = false; }
    for (const relation of fixture.diplomacy!.relations) relation.state = 'Peace';
    const startingTurn = fixture.turn;
    await page.setViewportSize(size);
    await page.addInitScript(({ game, turn }) => {
      if (!location.protocol.startsWith('http')) return;
      localStorage.setItem('legatum.autosave.v1', JSON.stringify({ format: 'legatum-save', version: 1, client: 'wasm', turn, names: ['Ada'], savedAt: new Date().toISOString(), game }));
      localStorage.setItem('legatum.first-game-hints.v2', JSON.stringify({ enabled: false, seen: [] }));
    }, { game: coreFixture(fixture), turn: startingTurn });
    await page.goto('/?client=wasm');
    await page.getByRole('button', { name: /^Continue/ }).click();
    for (const [id, focus] of [[own, 'Science'], [walled, 'Military']] as const) {
      await page.evaluate((city) => (window as unknown as GameWindow).legatum.getState().select({ kind: 'city', id: city }, true), id);
      await page.getByRole('combobox', { name: /Build automatically in/ }).selectOption(focus);
    }
    await endPlanning(page);
    await expect(page.getByTestId('turn-number')).toHaveText(String(startingTurn + 1));
    const cities = await page.evaluate((ids) => {
      const state = (window as unknown as GameWindow).legatum.getState();
      return ids.map((id) => state.views![state.seat].cities.find((city) => city.id === id)?.detail);
    }, [own, walled]);
    expect(cities[0]?.build_automation).toBe('Science');
    expect(cities[1]?.build_automation).toBe('Military');
    expect(cities.every((city) => city?.producing)).toBe(true);
    await endPlanning(page);
    await expect(page.getByTestId('turn-number')).toHaveText(String(startingTurn + 2));
    const queues = await page.evaluate((ids) => {
      const state = (window as unknown as GameWindow).legatum.getState();
      return ids.map((id) => state.views![state.seat].cities.find((city) => city.id === id)?.detail?.producing);
    }, [own, walled]);
    expect(queues.every(Boolean)).toBe(true);
  });
}

test('Ready returns an unchosen delegated city to Choose production', async ({ page }) => {
  const { fixture, own } = wallsFixture();
  const city = fixture.cities.find((c) => c.id === own)!;
  city.producing = null;
  city.productionQueue = [];
  city.prod = 0;
  for (const player of fixture.players.slice(1)) { player.isHuman = false; fixture.setup.players[player.id].is_human = false; }
  for (const relation of fixture.diplomacy!.relations) relation.state = 'Peace';
  await page.setViewportSize({ width: 1000, height: 600 });
  await page.addInitScript(({ game, turn }) => {
    localStorage.setItem('legatum.autosave.v1', JSON.stringify({ format: 'legatum-save', version: 1, client: 'wasm', turn, names: ['Explorer'], savedAt: new Date().toISOString(), game }));
    localStorage.setItem('legatum.first-game-hints.v2', JSON.stringify({ enabled: false, seen: [] }));
  }, { game: coreFixture(fixture), turn: fixture.turn });
  await page.goto('/?client=wasm');
  await page.getByRole('button', { name: /^Continue/ }).click();
  await expect(page.getByTestId('turn-number')).toHaveText(String(fixture.turn));
  await page.evaluate((id) => (window as unknown as GameWindow).legatum.getState().select({ kind: 'city', id }, true), own);
  await page.getByRole('combobox', { name: /Build automatically in/ }).selectOption('Science');
  await page.evaluate((id) => {
    const store = (window as unknown as GameWindow).legatum;
    const s = store.getState();
    const view = s.views![s.seat];
    store.setState({ views: [{ ...view, cities: view.cities.map((c) => c.id === id ? { ...c, detail: { ...c.detail!, options: c.detail!.options.filter((o) => o.item.kind === 'Unit' && o.item.unit === 'Soldier') } } : c) }] });
  }, own);
  await endPlanning(page);
  await expect(page.getByTestId('turn-number')).toHaveText(String(fixture.turn));
  await expect(page.locator('.seatbar .cta.todo')).toContainText('Choose production');
  await expect(page.locator('.seatbar .cta.todo')).toContainText("Nothing here fits its automation focus");
  await expect(page.locator('.city-panel .production-warning')).toContainText('Choose this turn');
  const state = await page.evaluate((id) => {
    const s = (window as unknown as GameWindow).legatum.getState();
    return { ready: s.ready[0], focus: s.orders[0].flatMap((o) => o.type === 'SetCityAutomation' && o.city === id ? [o.focus] : [])[0] };
  }, own);
  expect(state).toEqual({ ready: false, focus: 'Science' });
});
