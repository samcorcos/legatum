// Eras, the strict research gate and the arrival schedule in the WASM core.
import { existsSync, readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { beforeAll, describe, expect, it } from 'vitest';
import { TECHS, TECH_ERA } from '../data';
import type { BuildingKind, Event, Order, TechId } from '../types';
import { fromOffset, within } from '../hex';
import { WasmClient } from '../wasmClient';
import { coreFixture } from './coreFixture';
import { FixtureClient } from './fixtureEngine';
import { idxOf, refreshVision } from './fixtureEngine';
import { createState, makeStandardSetup, spawnUnit } from './fixtureEngine';
import type { MCity, MState } from './fixtureState';

const available = existsSync(fileURLToPath(new URL('../wasm-pkg/legatum.js', import.meta.url)));
let makeCore: () => WasmClient;

function fixture() {
  const setup = makeStandardSetup('73532', 'Athens', 'Celts', 'A', 'B');
  setup.width = setup.height = 30;
  setup.barbarians = false;
  setup.players = setup.players.slice(0, 3).map((p, team) => ({ ...p, team, is_human: true }));
  const s = createState(setup);
  delete (s as Partial<MState>).diplomacy;
  s.units = []; s.nextUnitId = 0; s.cities = []; s.nextCityId = 0;
  s.terrain.fill('Grassland'); s.ruin.fill(0); s.owner.fill(-1); s.tileCity.fill(-1);
  for (const p of s.players) p.gold = 500;
  return s;
}

function city(s: MState, owner: number, col: number, row: number, population: number, buildings: BuildingKind[], capital = false) {
  const c: MCity = {
    id: ++s.nextCityId, owner, founder: owner, name: `C${s.nextCityId}`, pos: fromOffset(col, row),
    population, food: 0, prod: 5, producing: null, bought: null, buildings,
    hp: 200, grownTiles: [], tilesClaimed: 0, isCapital: capital, foundedTurn: 1, resistance: 0,
  };
  s.cities.push(c);
  for (const h of within(c.pos, 1)) { const i = idxOf(s, h); s.owner[i] = owner; s.tileCity[i] = c.id; }
  return c;
}

function pair(s: MState) {
  refreshVision(s);
  const fixtureClient = new FixtureClient(); fixtureClient.load(JSON.stringify(s));
  const core = makeCore(); core.load(coreFixture(s));
  return { fixtureClient, core };
}

const era = (events: Event[]) => events.filter((e) => e.type === 'EraEntered' || e.type === 'TechResearched' || e.type === 'ResearchQueueAdvanced');
const research = (client: FixtureClient | WasmClient, p: number) => client.view(p).research;

describe.skipIf(!available)('eras: core behavior', () => {
  beforeAll(async () => {
    const mod = await import(/* @vite-ignore */ '../wasm-pkg/legatum.js');
    await mod.default({ module_or_path: readFileSync(fileURLToPath(new URL('../wasm-pkg/legatum_bg.wasm', import.meta.url))) });
    makeCore = () => new WasmClient(mod.Game);
  });

  it('the research view, catalog and schedule agree', () => {
    const s = fixture();
    city(s, 0, 5, 5, 4, ['Workshop', 'Library'], true);
    city(s, 0, 12, 5, 2, ['Granary']);
    city(s, 1, 22, 22, 3, [], true);
    spawnUnit(s, 0, 'Spearman', fromOffset(6, 7), false);
    spawnUnit(s, 0, 'Spearman', fromOffset(7, 7), false);
    spawnUnit(s, 0, 'Spearman', fromOffset(8, 7), false);
    s.players[0].techs = ['Irrigation', 'BronzeWorking'];
    s.players[0].science = 17;
    s.players[0].research = 'Currency'; s.players[0].research_queue = ['Currency', 'Writing'];
    s.players[1].techs = ['Irrigation', 'BronzeWorking', 'Currency'];
    s.eraFirsts = { 1: 1 };
    const { fixtureClient, core } = pair(s);
    for (const p of [0, 1]) {
      const a = research(fixtureClient, p), b = research(core, p);
      (expect({ era: a.era, era_name: a.era_name, era_total: a.era_total, era_known: a.era_known, era_firsts: a.era_firsts, schedule: a.schedule, queue: a.queue, known: a.known, cost: a.cost, turns_left: a.turns_left }).toMatchSnapshot(), void ({ era: b.era, era_name: b.era_name, era_total: b.era_total, era_known: b.era_known, era_firsts: b.era_firsts, schedule: b.schedule, queue: b.queue, known: b.known, cost: b.cost, turns_left: b.turns_left }));
      (expect(a.catalog.map((o) => ({ tech: o.tech, cost: o.cost, turns: o.turns, era: o.era, researchable: o.researchable, known_by: o.known_by }))).toMatchSnapshot(), void (b.catalog.map((o) => ({ tech: o.tech, cost: o.cost, turns: o.turns, era: o.era, researchable: o.researchable, known_by: o.known_by }))));
      (expect(a.available.map((o) => o.tech)).toMatchSnapshot(), void (b.available.map((o) => o.tech)));
      (expect(fixtureClient.view(p).players.map((x) => x.era)).toMatchSnapshot(), void (core.view(p).players.map((x) => x.era)));
    }
  });

  it('refuses and accepts research orders with the same words', () => {
    const s = fixture();
    city(s, 0, 5, 5, 3, [], true); city(s, 1, 22, 22, 3, [], true);
    s.players[0].techs = ['Irrigation'];
    const { fixtureClient, core } = pair(s);
    const orders: Order[][] = [
      [{ type: 'SetResearch', tech: 'Mathematics' }],
      [{ type: 'SetResearch', tech: 'Irrigation' }],
      [{ type: 'SetResearch', tech: 'Currency' }],
      [{ type: 'SetResearchQueue', techs: ['Currency', 'Engineering'] }],
      [{ type: 'SetResearchQueue', techs: ['Currency', 'Currency'] }],
      [{ type: 'SetResearchQueue', techs: ['Currency', 'Writing', 'Masonry', 'BronzeWorking'] }],
    ];
    for (const o of orders) (expect(fixtureClient.validate(0, o)).toMatchSnapshot(), void (core.validate(0, o)));
    expect(fixtureClient.validate(0, orders[0])[0].message).toBe('Mathematics is in the Classical era: finish the Ancient era first');
  });

  it('entering the next era and the first-into-era mark happen in the same turn', () => {
    const s = fixture();
    city(s, 0, 5, 5, 4, ['Library', 'Library'].slice(0, 1) as BuildingKind[], true);
    city(s, 0, 12, 5, 2, []);
    city(s, 1, 22, 22, 3, [], true);
    city(s, 2, 22, 5, 3, [], true);
    for (const p of [0, 1]) { s.players[p].techs = ['Irrigation', 'BronzeWorking', 'Masonry', 'Currency']; s.players[p].science = 500; s.players[p].research = 'Writing'; s.players[p].research_queue = ['Writing']; }
    s.players[2].techs = ['Irrigation']; s.players[2].research = 'Masonry'; s.players[2].research_queue = ['Masonry'];
    const { fixtureClient, core } = pair(s);
    for (let turn = 0; turn < 3; turn++) {
      const a = fixtureClient.endTurn({}), b = core.endTurn({});
      (expect(era(a.events)).toMatchSnapshot(), void (era(b.events)));
      for (const p of [0, 1, 2]) {
        (expect(era(a.events_by_player![p])).toMatchSnapshot(), void (era(b.events_by_player![p])));
        const x = research(fixtureClient, p);
        expect({ era: x.era, firsts: x.era_firsts, known: x.known, queue: x.queue, costs: x.catalog.map((o) => o.cost.total) })
          .toMatchSnapshot();
      }
    }
    expect(research(core, 0).era).toBe(1);
    // Both entered on the same turn: the first is drawn by seed, so either seat can hold it.
    const firsts = research(core, 0).era_firsts;
    expect(firsts).toHaveLength(1);
    expect(firsts[0][0]).toBe(1);
    expect([0, 1]).toContain(firsts[0][1]);
  });

  it('Railways, Code of Laws, Printing Press and the Castle tiers resolve the same', () => {
    const s = fixture();
    const c = city(s, 0, 5, 5, 5, ['Library', 'Walls', 'Castle', 'StarFort', 'ResearchLab'], true);
    city(s, 1, 22, 22, 3, [], true);
    const spearman = spawnUnit(s, 0, 'Spearman', c.pos, false);
    s.road = new Array(s.terrain.length).fill(0); s.road[idxOf(s, c.pos)] = 64; s.roaded = true;
    s.players[0].techs = ['Irrigation', 'CodeOfLaws', 'PrintingPress', 'Railways', 'FusionPower', 'StealthTechnology', 'RecursiveAi'];
    const { fixtureClient, core } = pair(s);
    const a = fixtureClient.endTurn({}), b = core.endTurn({});
    (expect(era(a.events)).toMatchSnapshot(), void (era(b.events)));
    const sources = (client: FixtureClient | WasmClient) => client.view(0).cities.find((x) => x.id === c.id)!.detail!.yields.sources;
    (expect(sources(fixtureClient)).toMatchSnapshot(), void (sources(core)));
    (expect(fixtureClient.view(0).units.find((u) => u.id === spearman.id)!.moves_left).toMatchSnapshot(), void (core.view(0).units.find((u) => u.id === spearman.id)!.moves_left));
    const cv = (client: FixtureClient | WasmClient) => { const v = client.view(0).cities.find((x) => x.id === c.id)!; return [v.strength, v.citadel, v.star_fort, v.walls]; };
    (expect(cv(fixtureClient)).toMatchSnapshot(), void (cv(core)));
    expect(core.view(0).cities.find((x) => x.id === c.id)!.star_fort).toBe(true);
    const unlocks = (client: FixtureClient | WasmClient) => client.view(0).research.unlocks.map((u) => [u.tech, u.item, u.cost, u.yields, u.city_strength]);
    (expect(unlocks(fixtureClient)).toMatchSnapshot(), void (unlocks(core)));
  });

  it('a received free-technology grant is limited to the current era in both engines', () => {
    const s = fixture();
    city(s, 0, 5, 5, 3, [], true); city(s, 1, 22, 22, 3, [], true);
    s.players[1].techs = ['Irrigation', 'BronzeWorking', 'Masonry', 'Currency'];
    s.players[1].freeTechsPending = 3; s.players[1].freeTechsFrom = 0;
    const { fixtureClient, core } = pair(s);
    const pick = (techs: TechId[]): Order[] => [{ type: 'Action', action: { action: 'ChooseFreeTechs', techs } }];
    for (const techs of [['Mathematics'], ['Writing', 'Mathematics'], ['Writing']] as TechId[][]) (expect(fixtureClient.validate(1, pick(techs))).toMatchSnapshot(), void (core.validate(1, pick(techs))));
    (expect(fixtureClient.view(1).actions!.free_techs).toMatchSnapshot(), void (core.view(1).actions!.free_techs));
    const a = fixtureClient.endTurn({ 1: pick(['Writing']) }), b = core.endTurn({ 1: pick(['Writing']) });
    (expect(era(a.events)).toMatchSnapshot(), void (era(b.events)));
    expect(a.events.some((e) => e.type === 'EraEntered')).toBe(true);
    (expect(fixtureClient.view(1).actions).toMatchSnapshot(), void (core.view(1).actions));
    expect(TECHS.filter((t) => TECH_ERA[t] === 0)).toHaveLength(5);
  });
});
