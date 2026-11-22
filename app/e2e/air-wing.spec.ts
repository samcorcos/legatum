import { mkdirSync, readFileSync } from 'node:fs';
import { gunzipSync } from 'node:zlib';
import path from 'node:path';
import type { Page } from '@playwright/test';
import type { useGame } from '../src/store';
import type { Hex } from '../src/types';
import { test, expect } from './fixtures';
import { endPlanning } from './turnActions';
import { coreFixture } from '../src/test/coreFixture';
import { siegeFixture } from '../src/test/siegeFixture';
import { fromOffset, ring, tileIndex, hexEq, toOffset } from '../src/hex';
import { spawnUnit } from '../src/test/fixtureEngine';
import { refreshVision } from '../src/test/fixtureEngine';

// Aircraft are understandable (user, 2026-10-17, Fighter at T204: "it's not clear to me that anything happened
// after that... the only action is 'rebase'"). A Fighter on guard says so (the city card, This Turn, the map's
// rings); an aircraft with a target in reach asks for a decision in the Return queue and its picker is on the
// map; a finished aircraft states its job. At the user's laptop window and a desktop one.
type GameWindow = Window & {
  legatum: typeof useGame;
  legatumMap: { air: { center: Hex; rings: [number, string][]; pick: string | null; candidates: Hex[] } | null; toScreen(hex: Hex): { x: number; y: number } };
};

const SIZES = [{ name: '1000x600', width: 1000, height: 600 }, { name: '1440x900', width: 1440, height: 900 }] as const;
const SHOTS = path.resolve('test-results/air-wing');
const PLANES = [{ id: 900, kind: 'Bomber' }, { id: 901, kind: 'Fighter' }] as const;

/** Waypost Seven with a Bomber and a Fighter in the hangar and two enemy Spearmen three tiles out (the siege fixture's approach). */
async function start(page: Page, opts: { enemies?: boolean; planes?: readonly { id: number; kind: string }[]; producing?: 'Fighter' | 'Bomber'; forest?: boolean; noCities?: boolean; rebaseDestination?: boolean } = {}) {
  const { enemies = true, planes = PLANES, producing, forest = false, rebaseDestination = false } = opts;
  const setup = siegeFixture({ approach: true });
  if (!enemies) setup.fixture.units = setup.fixture.units.filter((u) => u.owner === 0);
  // A Bomber reaches 8 tiles, so the enemy city is a target too: nothing in reach means no enemy cities either (the game would end if this were played on).
  if (!enemies && opts.noCities) {
    const kept = new Set(setup.fixture.cities.filter((c) => c.owner === 0).map((c) => c.id));
    // A save whose tile points at a removed city is invalid, so clear those tiles too.
    setup.fixture.tileCity = setup.fixture.tileCity.map((id) => (id >= 0 && !kept.has(id) ? -1 : id));
    setup.fixture.cities = setup.fixture.cities.filter((c) => kept.has(c.id)); setup.fixture.knownCities = {};
  }
  if (forest) {
    // Forest on the two rings between the city and the Spearmen three tiles out: a ground shooter has no line, and the
    // city cannot see over it, so a Scout of yours stands beside the first Spearman to give the strike its sight.
    const f = setup.fixture;
    const home = f.cities.find((c) => c.id === setup.own)!.pos;
    for (const r of [1, 2]) for (const h of ring(home, r)) f.terrain[tileIndex(h, f.width, f.height)] = 'Forest';
    const front = f.units.find((u) => u.owner !== 0)!;
    const beside = ring(front.pos, 1).find((h) => !hexEq(h, home) && f.terrain[tileIndex(h, f.width, f.height)] === 'Plains' && !f.units.some((u) => hexEq(u.pos, h)));
    spawnUnit(f, 0, 'Scout', beside ?? fromOffset(10, 9), false);
    refreshVision(f);
  }
  const own = setup.fixture.cities.find((c) => c.id === setup.own)!;
  const destination = rebaseDestination ? (() => {
    const [col, row] = toOffset(own.pos);
    const target = { ...own, id: setup.fixture.nextCityId++, name: 'Emain Macha', pos: fromOffset(col + 2, row) };
    setup.fixture.cities.push(target);
    return target;
  })() : null;
  if (producing) { own.producing = { kind: 'Unit', unit: producing }; own.prod = 400; setup.fixture.players[0].techs.push('Flight'); }
  const aircraft = JSON.stringify([...planes.map((p) => ({ id: p.id, kind: p.kind, city: setup.own, hp: 100 })), ...(destination ? [{ id: 902, kind: 'Bomber', city: destination.id, hp: 100 }] : [])]);
  // The save holds u64 bitsets that JSON.parse would round, so the hangar is added as text.
  // Saves are validated: aircraft ids may not exceed the next unit id.
  const save = coreFixture(setup.fixture).replace(/"next_unit_id":\d+/, '"next_unit_id":902');
  const game = save.replace(/^\{/, planes.length ? `{"aircraft":${aircraft},` : '{');
  await page.addInitScript((g) => {
    if (!location.protocol.startsWith('http')) return;
    localStorage.setItem('legatum.autosave.v1', JSON.stringify({
      format: 'legatum-save', version: 1, client: 'wasm', turn: 2, names: ['Ada'], savedAt: new Date().toISOString(), game: g,
    }));
    localStorage.setItem('legatum.first-game-hints.v2', JSON.stringify({ enabled: false, seen: [] }));
  }, game);
  await page.goto('/?client=wasm');
  await page.getByRole('button', { name: /^Continue/ }).click();
  await expect(page.getByTestId('turn-number')).toHaveText('2');
  await expect(page.locator('.loading-screen')).toHaveCount(0);
  return { ...setup, destination: destination?.id };
}

const store = <T,>(page: Page, fn: (s: ReturnType<typeof useGame.getState>) => T) =>
  page.evaluate((src) => (new Function('s', `return (${src})(s)`) as (s: unknown) => unknown)((window as unknown as GameWindow).legatum.getState()), fn.toString()) as Promise<T>;

async function shot(page: Page, size: string, name: string, park = true) {
  mkdirSync(path.join(SHOTS, size), { recursive: true });
  if (park) await page.mouse.move(1, 330);
  await page.evaluate(() => document.fonts.ready.then(() => undefined));
  await page.waitForTimeout(350);
  await page.screenshot({ path: path.join(SHOTS, size, `${name}.png`), animations: 'disabled' });
}

const openCity = (page: Page, id: number) => page.evaluate((id) => (window as unknown as GameWindow).legatum.getState().select({ kind: 'city', id }, true), id);
const plane = (page: Page, id: number) => page.locator(`[data-plane="${id}"]`);
const openPlane = (page: Page, id: number) => page.evaluate((planeId) => (window as unknown as GameWindow).legatum.getState().select({ kind: 'air', id: planeId }, true), id);
const noSideways = (page: Page, selector: string) => page.locator(selector).first().evaluate((el) => el.scrollWidth <= el.clientWidth + 1);

for (const size of SIZES) {
  test.describe(size.name, () => {
    test.use({ viewport: { width: size.width, height: size.height } });

    test('the city keeps a compact hangar; selecting a plane opens the left unit card and its map art', async ({ page }) => {
      const setup = await start(page);
      await openCity(page, setup.own);
      await expect(page.locator('.city-hangar')).toContainText('Hangar 2 / 2');
      await expect(page.getByTestId('air-wing')).toHaveCount(0);
      await page.getByRole('button', { name: /Bomber · 100 HP/ }).click();
      await expect(page.getByTestId('air-wing')).toBeVisible();
      await expect(page.getByTestId('air-wing')).toHaveAttribute('data-selected-plane', '900');
      await expect(page.getByTestId('air-wing').locator('.up-kind')).toHaveText('Bomber');
      await shot(page, size.name, 'bomber-unit-card-and-map-art');
      await page.locator('[data-game-map]').focus();
      await page.keyboard.press('Escape');
      await expect(page.getByTestId('air-wing')).toHaveCount(0);
    });

    test('idle aircraft with nothing to strike are in the Return queue like any unit, each with a card', async ({ page }) => {
      await start(page, { enemies: false, noCities: true });
      // The hangar's two aircraft ask for orders (and so does the Scout): the dock counts them.
      await expect(page.locator('.seatbar .cta .cta-label')).toHaveText(/units? needs? orders/);
      await expect(page.getByTestId('air-watch')).toHaveCount(0);
      await shot(page, size.name, 'queue-idle-aircraft');
      // The rival Byzantium AI opens with a Peace offer (it can no longer buy peace), and a decision to respond comes first: answer it through the panel, as a player does.
      await page.keyboard.press('Escape');
      for (let i = 0; i < 8 && (await store(page, (s) => s.selection?.kind)) !== 'air'; i++) {
        if ((await page.locator('.seatbar .cta').first().innerText()).includes('Respond to offer')) {
          await page.keyboard.press('Enter');
          await page.getByRole('button', { name: /^Decline/ }).first().click();
          await page.keyboard.press('Escape');
        } else await page.keyboard.press('Enter');
        await page.waitForTimeout(150);
      }
      // The Bomber's city opens with its card; there is nothing to pick, so no picker.
      expect(await store(page, (s) => [s.selection?.kind, s.air.plane, s.air.what])).toEqual(['air', 900, null]);
      const bomber = plane(page, 900);
      await expect(bomber).toContainText('no targets within 8 tiles');
      await expect(bomber).toContainText('No enemy unit or city is visible within 8 tiles of Waypost Seven');
      await expect(bomber.getByRole('button')).toHaveText(['Wait for targetF', 'Skip this turnSpace', 'Rebase…R']);
      await expect(bomber.getByRole('button', { name: /Rebase/ })).toBeDisabled();
      await shot(page, size.name, 'queue-idle-bomber');
      await bomber.locator('button[aria-keyshortcuts="Space"]').click();
      await expect(bomber).toContainText('Holding this turn');
      // Return goes on to the Fighter: it can stay on guard or wait for a target.
      for (let i = 0; i < 8 && (await store(page, (s) => s.selection?.id)) !== 901; i++) { await page.keyboard.press('Enter'); await page.waitForTimeout(150); }
      const fighter = plane(page, 901);
      await expect(fighter).toHaveAttribute('data-air-state', 'guard');
      await expect(fighter).toContainText('Needs orders · on guard over 4 tiles around Waypost Seven');
      await expect(fighter.getByRole('button')).toHaveText(['Stay on guardF', 'Skip this turnSpace', 'Rebase…R']);
      await expect(fighter.getByRole('button', { name: /Rebase/ })).toBeDisabled();
      // The map rings what they cover: the Bomber's 8, the Fighter's strike 5 and guard 4, widest first, each labelled.
      await expect.poll(() => page.evaluate(() => (window as unknown as GameWindow).legatumMap.air?.rings)).toEqual([[8, 'Bomber strikes 8 tiles'], [5, 'Fighter strikes 5 tiles'], [4, 'Fighter guards 4 tiles']]);
      await fighter.getByRole('button', { name: 'Stay on guard' }).click();
      await expect(page.getByTestId('air-watch')).toContainText('Fighter on guard');
      await expect(page.getByTestId('air-watch')).toContainText('Waypost Seven');
      await expect(page.getByTestId('air-watch')).toContainText('covers 4 tiles');
      await fighter.scrollIntoViewIfNeeded();
      expect(await noSideways(page, '[data-testid="air-wing"]')).toBe(true);
      await shot(page, size.name, 'city-air-wing-guard');
      // Both answered: ending the turn no longer asks about the air wing.
      expect(await store(page, (s) => s.airHeld[s.views![s.seat].me].planes)).toEqual([900]);
    });

    test('aircraft shortcuts open available pickers and explain a full hangar', async ({ page }) => {
      const setup = await start(page, { rebaseDestination: true });
      await openCity(page, setup.own);
      await expect(page.getByRole('button', { name: /Bomber · 100 HP/ })).toBeVisible();
      await openPlane(page, 900);
      await page.locator('[data-game-map]').focus();
      await page.keyboard.press('s');
      expect(await store(page, (s) => s.air)).toMatchObject({ plane: 900, what: 'strike' });
      await page.evaluate((to) => {
        const store = (window as unknown as GameWindow).legatum;
        store.setState({ air: { plane: null, what: null, hover: null } });
        store.getState().editOrders((orders) => [...orders, { type: 'Rebase', plane: 900, to: to! }, { type: 'Rebase', plane: 901, to: to! }], 'commit');
      }, setup.destination);
      expect(await store(page, (s) => s.errors[s.seat].map((error) => error.message))).toEqual(["Emain Macha's hangar is full (2 aircraft)"]);
      await openPlane(page, 901);
      await expect(plane(page, 901)).not.toContainText('Rebasing this turn');
      await expect(plane(page, 901).locator('.up-status-why')).toHaveText("Emain Macha's hangar is full (2 aircraft)");
      await page.locator('[data-game-map]').focus();
      await page.keyboard.press('r');
      expect(await store(page, (s) => s.air.what)).toBeNull();
      await expect(plane(page, 901).getByRole('button', { name: /Rebase/ })).toBeDisabled();
      await expect(plane(page, 901)).toContainText('Rebase unavailable: every other city hangar is full.');
      await shot(page, size.name, 'rebase-capacity');
      await page.keyboard.press('f');
      expect(await store(page, (s) => s.airHeld[s.views![s.seat].me].planes)).toContain(901);
      await page.evaluate(() => (window as unknown as GameWindow).legatum.setState({ airHeld: {} }));
      await page.keyboard.press('Space');
      expect(await store(page, (s) => s.airHeld[s.views![s.seat].me].planes)).toEqual([901]);
    });

    test('an aircraft strikes over a forest: no line of sight is needed', async ({ page }) => {
      await start(page, { forest: true, planes: [PLANES[0]] });
      const blocked = await page.evaluate(() => {
        const s = (window as unknown as GameWindow).legatum.getState();
        const v = s.views![s.seat];
        return v.units.filter((u) => u.owner !== v.me && u.military).length;
      });
      expect(blocked).toBeGreaterThan(0);
      // The rival Byzantium AI opens with a Peace offer (it can no longer buy peace), and a decision to respond comes first: answer it through the panel, as a player does.
      await page.keyboard.press('Escape');
      for (let i = 0; i < 8 && (await store(page, (s) => s.air.plane)) === null; i++) {
        if ((await page.locator('.seatbar .cta').first().innerText()).includes('Respond to offer')) {
          await page.keyboard.press('Enter');
          await page.getByRole('button', { name: /^Decline/ }).first().click();
          await page.keyboard.press('Escape');
        } else await page.keyboard.press('Enter');
        await page.waitForTimeout(150);
      }
      const bomber = plane(page, 900);
      await expect(bomber).toContainText(/Ready: \d+ targets? within 8 tiles/);
      // Return opened the strike picker with the targets listed; nothing blocks the Bomber's shot.
      await expect(bomber.getByRole('button', { name: /^Strike/ })).toHaveAttribute('aria-expanded', 'true');
      const row = bomber.locator('.air-target').first();
      await expect(row).toContainText(/Destroys it|about −\d+ HP/);
      await shot(page, size.name, 'forest-picker');
      await row.click();
      await expect(bomber).toContainText('Striking this turn');
      // The order validates (core checks it again, without line of sight, and resolution is covered by the core tests).
      expect(await store(page, (s) => s.errors[s.seat].length)).toBe(0);
    });

    test('an aircraft with a target in reach is in the Return queue: its picker opens on the map, hover shows the forecast, a click orders it', async ({ page }) => {
      await start(page);
      // The fixture's own units ask first; Return cycles on until it reaches the aircraft, and the call to action names them.
      await expect(page.locator('.seatbar .cta .cta-label')).toHaveText(/units? needs? orders/);
      // The rival Byzantium AI opens with a Peace offer (it can no longer buy peace), and a decision to respond comes first: answer it through the panel, as a player does.
      await page.keyboard.press('Escape');
      for (let i = 0; i < 8 && (await store(page, (s) => s.air.plane)) === null; i++) {
        if ((await page.locator('.seatbar .cta').first().innerText()).includes('Respond to offer')) {
          await page.keyboard.press('Enter');
          await page.getByRole('button', { name: /^Decline/ }).first().click();
          await page.keyboard.press('Escape');
        } else await page.keyboard.press('Enter');
        await page.waitForTimeout(150);
      }
      expect(await store(page, (s) => [s.selection?.kind, s.air.plane, s.air.what])).toEqual(['air', 900, 'strike']);
      // Ending the turn asks about it, by name.
      await page.keyboard.press('Shift+Enter');
      const asking = page.getByRole('dialog', { name: 'End turn with decisions pending?' });
      await expect(asking).toContainText('Bomber in Waypost Seven: 3 targets within 8 tiles, strike or hold');
      await expect(asking).toContainText('Fighter in Waypost Seven: 2 targets within 5 tiles, strike or stay on guard');
      await shot(page, size.name, 'end-turn-asks');
      await page.keyboard.press('Escape');
      await expect(asking).toBeHidden();
      const bomber = plane(page, 900);
      await expect(bomber).toContainText('Ready: 3 targets within 8 tiles');
      await expect(bomber.getByRole('button', { name: /^Strike/ })).toHaveClass(/primary/);
      // The targets are ringed on the map and listed with their forecasts.
      await expect.poll(async () => (await page.evaluate(() => (window as unknown as GameWindow).legatumMap.air))?.candidates.length).toBe(3);
      await expect(bomber.locator('.air-target').first()).toContainText(/Destroys it|about −\d+ HP/);
      await shot(page, size.name, 'picker-open');
      // Hovering a ringed target on the map shows the same forecast card a ground unit's shot does.
      // The expanded aircraft card can cover the first candidate at 1440x900; hover a target actually exposed on the canvas.
      const target = await page.evaluate(() => {
        const map = (window as unknown as GameWindow).legatumMap;
        const box = document.querySelector<HTMLCanvasElement>('[data-game-map]')!.getBoundingClientRect();
        return map.air!.candidates.find((h) => {
          const p = map.toScreen(h);
          return document.elementFromPoint(p.x + box.left, p.y + box.top)?.matches('canvas[data-game-map]');
        });
      });
      expect(target).toBeDefined();
      const at = async () => page.evaluate((h) => {
        const p = (window as unknown as GameWindow).legatumMap.toScreen(h), b = document.querySelector('[data-game-map]')!.getBoundingClientRect();
        return { x: Math.round(p.x + b.left), y: Math.round(p.y + b.top) };
      }, target!);
      await expect.poll(async () => {
        const p = await at();
        await page.mouse.move(p.x + 4, p.y + 4); await page.mouse.move(p.x, p.y, { steps: 3 });
        return page.locator('.forecast').first().isVisible();
      }, { intervals: [300, 500, 1000, 1000] }).toBe(true);
      await shot(page, size.name, 'picker-map-forecast', false);
      const p = await at();
      await page.mouse.click(p.x, p.y);
      await expect.poll(() => store(page, (s) => s.orders[s.seat].map((o) => (o.type === 'AirStrike' ? `${o.type}:${o.plane}` : o.type)))).toEqual(['AirStrike:900']);
      await expect(bomber).toContainText('Striking this turn');
      expect(await store(page, (s) => s.errors[s.seat].length)).toBe(0);
      // The Fighter is next: it can strike or stay on guard, and says what each costs.
      for (let i = 0; i < 8 && (await store(page, (s) => s.air.plane)) !== 901; i++) { await page.keyboard.press('Enter'); await page.waitForTimeout(150); }
      expect(await store(page, (s) => s.air.plane)).toBe(901);
      const fighter = plane(page, 901);
      await expect(fighter).toContainText(/Ready: 2 targets within 5 tiles/);
      await expect(fighter).toContainText('stay on guard to intercept');
      await shot(page, size.name, 'fighter-ready');
      await fighter.getByRole('button', { name: 'Stay on guard' }).click();
      await expect(fighter).toContainText('On guard: covers 4 tiles around Waypost Seven');
      await expect(page.getByTestId('air-watch')).toContainText('Fighter on guard');
      // Nothing left to decide about the air wing: ending the turn no longer names it.
      await page.keyboard.press('Shift+Enter');
      await expect(asking).not.toContainText('strike or');
      await page.keyboard.press('Escape');
      expect(await noSideways(page, '[data-testid="air-wing"]')).toBe(true);
      await shot(page, size.name, 'decided');
    });

    test('a Fighter with only an at-Peace rival in reach keeps Strike as its primary and says why; a held Fighter keeps its answer lit', async ({ page }) => {
      await start(page, { planes: [PLANES[1]] });
      // The card reads the view, so making the rival's team a Peace relation in the stored view is enough for the card (the engine's view is unchanged).
      await page.evaluate(() => {
        const g = (window as unknown as GameWindow).legatum;
        const s = g.getState();
        const view = structuredClone(s.views![s.seat]);
        const enemy = view.players.find((p) => p.id !== view.me)!;
        view.diplomacy = { ...view.diplomacy, legacy: false, relations: [{ a: view.team, b: enemy.team, state: 'Peace' }] } as typeof view.diplomacy;
        g.setState({ views: s.views!.map((v, i) => (i === s.seat ? view : v)) });
      });
      await openPlane(page, 901);
      const fighter = plane(page, 901);
      await expect(fighter).toHaveAttribute('data-air-state', 'guard');
      await expect(fighter.getByRole('button', { name: /^Strike/ })).toHaveClass(/primary/);
      await expect(fighter).toContainText('not at War with you: declare War to strike');
      await expect(fighter).not.toContainText('Nothing to strike');
      await shot(page, size.name, 'fighter-peace-target');
      await fighter.getByRole('button', { name: 'Skip this turn' }).click();
      await expect(fighter.getByRole('button', { name: 'Stay on guard' })).toHaveAttribute('aria-pressed', 'true');
      await shot(page, size.name, 'fighter-held');
    });

    test('a Fighter that strikes says it will not intercept this turn', async ({ page }) => {
      await start(page, { planes: [PLANES[1]] });
      await openPlane(page, 901);
      const fighter = plane(page, 901);
      await fighter.getByRole('button', { name: /^Strike/ }).click();
      await fighter.locator('.air-target').first().click();
      await expect(fighter).toContainText('Striking this turn, so it will not intercept');
      await expect(page.getByTestId('air-watch')).toHaveCount(0);
      expect(await store(page, (s) => s.errors[s.seat].length)).toBe(0);
      await shot(page, size.name, 'fighter-strikes');
    });

    test('a finished Fighter announces its job, and the report keeps it', async ({ page }) => {
      await start(page, { enemies: false, planes: [], producing: 'Fighter' });
      await endPlanning(page);
      // The turn's bigger moments play first; the aircraft card follows.
      const card = page.locator('.moment-card', { hasText: 'Fighter ready' });
      await expect(card).toContainText('Aircraft ready · Waypost Seven', { timeout: 25_000 });
      await expect(card).toContainText('On guard it intercepts enemy air strikes within 4 tiles of its city');
      await expect(card).toContainText('Order it to strike enemies within 5 tiles instead');
      await expect(card.locator('img.moment-wonder-art')).toBeVisible();
      // The title is short ("Fighter ready"); the city rides in the kicker. The body may scroll, but the buttons stay
      // pinned and on screen at every window size (8.9.4 review: the 1000x600 card hid them).
      await expect(card.locator('h2')).toHaveText('Fighter ready');
      const area = (await card.boundingBox())!;
      for (const name of ['Show on map']) {
        const box = (await card.getByRole('button', { name }).boundingBox())!;
        expect(box.y + box.height, `${name} is inside the card`).toBeLessThanOrEqual(area.y + area.height + 0.5);
        expect(box.y + box.height, `${name} is on screen`).toBeLessThanOrEqual(size.height);
      }
      await page.evaluate(() => (window as unknown as GameWindow).legatum.setState({ reportOpen: false }));
      await shot(page, size.name, 'ready-card');
      // The new Fighter is a unit in the Return queue: its card offers Stay on guard, and answering it lists it on guard.
      await expect(page.getByTestId('air-watch')).toHaveCount(0);
      // The rival Byzantium AI opens with a Peace offer (it can no longer buy peace), and a decision to respond comes first: answer it through the panel, as a player does.
      await page.keyboard.press('Escape');
      for (let i = 0; i < 8 && (await store(page, (s) => s.air.plane)) === null; i++) {
        if ((await page.locator('.seatbar .cta').first().innerText()).includes('Respond to offer')) {
          await page.keyboard.press('Enter');
          await page.getByRole('button', { name: /^Decline/ }).first().click();
          await page.keyboard.press('Escape');
        } else await page.keyboard.press('Enter');
        await page.waitForTimeout(150);
      }
      const fighterId = await store(page, (s) => s.views![s.seat].aircraft?.[0]?.id ?? null);
      expect(fighterId).not.toBeNull();
      expect(await store(page, (s) => s.air.plane)).toBe(fighterId);
      await plane(page, fighterId!).getByRole('button', { name: 'Stay on guard' }).click();
      await expect(page.getByTestId('air-watch')).toContainText('Fighter on guard');
    });
  });
}

// Rules-25 copy of the user's 8.9.2 turn-207 position. Retired factions and buildings were
// adapted to current kinds; the Fighter and Lucerne, map, units, and diplomacy are retained.
const SAVE = gunzipSync(readFileSync(new URL('./saves/p892-fighter-t204-1432.current.legatum.gz', import.meta.url))).toString();
test.describe('the 8.9.2 save', () => {
  for (const size of SIZES) {
    test(`the Fighter in the evidence save states its job at ${size.name}`, async ({ page }) => {
      test.setTimeout(120_000);
      await page.setViewportSize({ width: size.width, height: size.height });
      const text = SAVE;
      await page.addInitScript((t) => {
        if (!location.protocol.startsWith('http')) return;
        localStorage.setItem('legatum.autosave.v1', t);
        localStorage.setItem('legatum.first-game-hints.v2', JSON.stringify({ enabled: false, seen: [] }));
      }, text);
      await page.goto('/?client=wasm');
      await page.getByRole('button', { name: /^Continue/ }).click();
      await expect(page.locator('.loading-screen')).toHaveCount(0, { timeout: 60_000 });
      await expect(page.getByTestId('turn-number')).toHaveText('207');
      const planeCity = await page.evaluate(() => { const s = (window as unknown as GameWindow).legatum.getState(); return s.views![s.seat].aircraft?.[0]?.city ?? null; });
      expect(planeCity).not.toBeNull();
      await openCity(page, planeCity!);
      await page.getByRole('button', { name: /Fighter · \d+ HP/ }).first().click();
      const card = page.locator('[data-plane]').first();
      await expect(card).toHaveAttribute('data-air-state', /^(ready|guard)$/);
      const state = await card.getAttribute('data-air-state');
      if (state === 'guard') {
        // Nothing to strike: it says it is on guard, and ending the turn asks about it like any idle unit.
        await expect(card).toContainText(/on guard over 4 tiles around/);
        await page.keyboard.press('Shift+Enter');
        await expect(page.getByRole('dialog', { name: 'End turn with decisions pending?' })).toContainText(/Fighter in Lucerne: needs orders/);
        await page.keyboard.press('Escape');
        await card.getByRole('button', { name: 'Stay on guard' }).click();
        await expect(page.getByTestId('air-watch')).toContainText('Fighter on guard');
      } else {
        // Enemies in reach: it asks, in the Return queue and when ending the turn.
        await expect(card).toContainText(/Ready: \d+ targets? within 5 tiles/);
        await page.keyboard.press('Shift+Enter');
        await expect(page.getByRole('dialog', { name: 'End turn with decisions pending?' })).toContainText(/Fighter in Lucerne: \d+ targets? within 5 tiles, strike or stay on guard/);
        await page.keyboard.press('Escape');
      }
      await card.scrollIntoViewIfNeeded();
      await shot(page, size.name, 'evidence-save-city');
    });
  }
});

// Rules-25 copy of the user's 8.9.5 position (Athens, a bought Bomber and a Forest between it
// and its target). The old AI memory and retired buildings are omitted; diplomacy is retained.
const SAVE_BOMBER = gunzipSync(readFileSync(new URL('./saves/p895-athens-bomber-1916.current.legatum.gz', import.meta.url))).toString();
test.describe('the 8.9.5 Bomber save', () => {
  for (const size of SIZES) {
    test(`the idle Bomber is in the Return queue and can strike past a Forest at ${size.name}`, async ({ page }) => {
      test.setTimeout(180_000);
      await page.setViewportSize({ width: size.width, height: size.height });
      const text = SAVE_BOMBER;
      await page.addInitScript((t) => {
        if (!location.protocol.startsWith('http')) return;
        localStorage.setItem('legatum.autosave.v1', t);
        localStorage.setItem('legatum.first-game-hints.v2', JSON.stringify({ enabled: false, seen: [] }));
      }, text);
      await page.goto('/?client=wasm');
      await page.getByRole('button', { name: /^Continue/ }).click();
      await expect(page.locator('.loading-screen')).toHaveCount(0, { timeout: 60_000 });
      // Return reaches the Bomber like any unit that needs orders.
      for (let i = 0; i < 40 && (await store(page, (s) => s.air.plane)) === null; i++) { await page.keyboard.press('Enter'); await page.waitForTimeout(150); }
      const bomberId = await store(page, (s) => s.views![s.seat].aircraft?.find((a) => a.kind === 'Bomber')?.id ?? null);
      expect(bomberId).not.toBeNull();
      expect(await store(page, (s) => s.air.plane)).toBe(bomberId);
      const card = plane(page, bomberId!);
      await expect(card).toBeVisible();
      await expect(card.getByRole('button', { name: 'Skip this turn' })).toBeVisible();
      await expect(card.locator('button[aria-keyshortcuts="Space"]')).toHaveText('Skip this turnSpace');
      await shot(page, size.name, 'evidence-bomber-card');
      // A strike on anything it can see in reach validates with no line-of-sight error, whatever stands between.
      const verdicts = await page.evaluate((id) => {
        const g = (window as unknown as GameWindow).legatum.getState();
        const v = g.views![g.seat];
        const home = v.cities.find((c) => c.id === v.aircraft!.find((a) => a.id === id)!.city)!;
        const dist = (a: { q: number; r: number }, b: { q: number; r: number }) => (Math.abs(a.q - b.q) + Math.abs(a.r - b.r) + Math.abs(a.q + a.r - b.q - b.r)) / 2;
        const targets = [...v.cities.filter((c) => c.owner !== v.me && c.visible).map((c) => c.pos), ...v.units.filter((u) => u.owner !== v.me && u.military).map((u) => u.pos)]
          .filter((h) => dist(h, home.pos) >= 1 && dist(h, home.pos) <= 5);
        return targets.map((at) => g.client!.validate(v.me, [{ type: 'AirStrike', plane: id, target: at }]).map((e) => e.message));
      }, bomberId!);
      for (const errors of verdicts) expect(errors.filter((m) => /line of sight/i.test(m))).toEqual([]);
    });
  }
});
