// Mercenary Companies against the real engine: the shop, the order, the contract on the unit and the report lines,
// as the app reads them (the types in types.ts are hand-written, so this is what keeps them true).
import { existsSync, readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { beforeAll, describe, expect, it } from 'vitest';
import { fromOffset, within } from '../hex';
import { RULES } from '../data';
import { companyName, contractLeft, contractLine, hireOrder, shopRows } from '../company';
import { contextFromViews, formatEvents } from '../format';
import { describeOrder } from '../orders';
import { WasmClient } from '../wasmClient';
import { coreFixture } from '../test/coreFixture';
import { idxOf, refreshVision } from './fixtureEngine';
import { createState, makeStandardSetup, meetAll } from './fixtureEngine';
import type { MCity, MState } from './fixtureState';

const available = existsSync(fileURLToPath(new URL('../wasm-pkg/legatum.js', import.meta.url)));
let makeCore: () => WasmClient;

function city(s: MState, owner: number, col: number, row: number) {
  const c: MCity = {
    id: ++s.nextCityId, owner, founder: owner, name: `C${owner}`, pos: fromOffset(col, row),
    population: 2, food: 0, prod: 0, producing: null, bought: null, buildings: [],
    hp: 200, grownTiles: [], tilesClaimed: 0, isCapital: true, foundedTurn: 1, resistance: 0,
  };
  s.cities.push(c);
  for (const h of within(c.pos, 1)) { const i = idxOf(s, h); s.owner[i] = owner; s.tileCity[i] = c.id; }
  return c;
}

/** Byzantium (player 0, 500 Gold, one city) and Sparta (player 1, one city), at Peace; `met` decides whether they have made contact. */
function fixture(met: boolean) {
  const setup = makeStandardSetup('55012', 'Byzantium', 'Sparta', 'AI 1', 'AI 2');
  setup.width = setup.height = 20;
  setup.players = setup.players.slice(0, 2).map((p, team) => ({ ...p, team, is_human: true }));
  const s = createState(setup);
  s.units = []; s.nextUnitId = 0; s.cities = []; s.nextCityId = 0;
  s.terrain.fill('Grassland'); s.ruin.fill(0); s.owner.fill(-1); s.tileCity.fill(-1);
  for (const relation of s.diplomacy?.relations ?? []) relation.state = 'Peace';
  if (met) meetAll(s);
  else s.diplomacy!.contacts = [];
  const home = city(s, 0, 4, 4);
  city(s, 1, 14, 14);
  s.players[0].gold = 500;
  refreshVision(s);
  return { s, home };
}

describe.skipIf(!available)('Mercenary Companies in the real engine', () => {
  beforeAll(async () => {
    const mod = await import(/* @vite-ignore */ '../wasm-pkg/legatum.js');
    await mod.default({ module_or_path: readFileSync(fileURLToPath(new URL('../wasm-pkg/legatum_bg.wasm', import.meta.url))) });
    makeCore = () => new WasmClient(mod.Game);
  });

  it('opens the shop only after first contact, and only to Byzantium', () => {
    const unmet = fixture(false);
    const a = makeCore(); a.load(coreFixture(unmet.s));
    expect(a.view(0).company_offers ?? []).toEqual([]);
    const met = fixture(true);
    const b = makeCore(); b.load(coreFixture(met.s));
    const shop = shopRows(b.view(0));
    expect(shop.map((x) => x.offer.seller)).toEqual([1]);
    expect(shop.map((x) => [x.offer.unit, x.offer.price > 0])).toEqual([['Spearman', true]]);
    expect(b.view(1).company_offers ?? []).toEqual([]);
  });

  it('hires, pays the seller, shows the contract on the unit, and reads cleanly in both reports', () => {
    const { s, home } = fixture(true);
    const core = makeCore(); core.load(coreFixture(s));
    const before = core.view(0), sellerBefore = core.view(1);
    const offer = before.company_offers!.find((o) => o.seller === 1 && o.unit === 'Spearman')!;
    const order = hireOrder(home.id, offer);
    expect(describeOrder(order, before)).toBe(`C0: hire a Spartan Hoplite Company from Sparta for ${offer.price} Gold (${RULES.company.turns} turns)`);
    expect(core.validate(0, [order])).toEqual([]);
    // A second purchase in the same city is refused in the core's words.
    expect(core.validate(0, [order, order])[0]?.message).toMatch(/already bought something this turn/);
    const result = core.endTurn({ 0: [order] });
    const after = core.view(0), sellerAfter = core.view(1);
    const hired = after.units.find((u) => u.contract_until !== undefined)!;
    expect(hired.owner).toBe(0);
    expect(hired.origin).toBe('Sparta');
    expect(contractLeft(hired, after)).toBe(RULES.company.turns - 1);
    expect(after.players.find((p) => p.id === 0)!.companies).toBe(1);
    expect(contractLine(hired, after)).toBe(`Hired from Sparta: ${RULES.company.turns - 1} turns of contract left, upkeep ${RULES.company.upkeep} Gold a turn`);
    const buyer = formatEvents(result.events, contextFromViews(before, after)).map((l) => l.text);
    expect(buyer).toContain(`Hired a ${companyName('Sparta', 'Spearman')} Company from Sparta for ${offer.price} Gold, for ${RULES.company.turns} turns`);
    const seller = formatEvents(result.events, contextFromViews(sellerBefore, sellerAfter)).map((l) => l.text);
    expect(seller).toContain(`+${offer.price} Gold: Byzantium hired a ${companyName('Sparta', 'Spearman')} Company from you`);
  });

  it('goes home after its last turn, and says so', () => {
    const { s, home } = fixture(true);
    const core = makeCore(); core.load(coreFixture(s));
    const offer = core.view(0).company_offers!.find((o) => o.seller === 1 && o.unit === 'Spearman')!;
    core.endTurn({ 0: [hireOrder(home.id, offer)] });
    let last = false, departed = false;
    for (let i = 0; i < RULES.company.turns + 1 && !departed; i++) {
      const view = core.view(0);
      const unit = view.units.find((u) => u.contract_until !== undefined);
      if (unit && contractLeft(unit, view) === 0) last = contractLine(unit, view) === 'Hired from Sparta: goes home this turn';
      const result = core.endTurn({});
      departed = result.events.some((e) => e.type === 'CompanyDeparted');
    }
    expect(last).toBe(true);
    expect(departed).toBe(true);
    expect(core.view(0).units.some((u) => u.contract_until !== undefined)).toBe(false);
  });
});
