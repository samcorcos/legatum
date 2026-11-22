// @vitest-environment happy-dom
import { existsSync, readFileSync } from 'node:fs';
import { fileURLToPath, URL as NodeURL } from 'node:url';
import { act } from 'react';
import { createRoot, type Root } from 'react-dom/client';
import { afterEach, beforeAll, beforeEach, describe, expect, it, vi } from 'vitest';
import { WasmClient } from '../wasmClient';
import { useGame } from '../store';
import { DiplomacyPanel } from './DiplomacyPanel';
import { VpTrack } from './VpTrack';
import { contextFromViews, formatEvents } from '../format';
import { createState, meetAll } from '../test/fixtureEngine';
import { coreFixture } from '../test/coreFixture';
import { allianceTerms } from '../treaties';

const available = existsSync(fileURLToPath(new NodeURL('../wasm-pkg/legatum.js', import.meta.url)));
let makeClient: () => WasmClient;
let container: HTMLDivElement;
let root: Root;

describe.skipIf(!available)('diplomacy controls on the Rust engine', () => {
  beforeAll(async () => {
    const path = '../wasm-pkg/legatum.js';
    const mod = await import(/* @vite-ignore */ path);
    await mod.default({ module_or_path: readFileSync(fileURLToPath(new NodeURL('../wasm-pkg/legatum_bg.wasm', import.meta.url))) });
    makeClient = () => new WasmClient(mod.Game);
  });
  beforeEach(() => {
    vi.stubGlobal('IS_REACT_ACT_ENVIRONMENT', true);
    useGame.setState(useGame.getInitialState(), true);
    container = document.body.appendChild(document.createElement('div'));
    root = createRoot(container);
  });
  afterEach(() => { act(() => root.unmount()); container.remove(); vi.unstubAllGlobals(); });
  async function start() {
    const client = makeClient();
    const setup = await client.standardSetup('9941', 'India', 'Byzantium', 'Ada', 'Bo');
    setup.players = setup.players.slice(0, 2); setup.players[1].team = 1;
    // The same seeded start, in first contact: these tests are about treaties, not exploration.
    const state = createState(setup); meetAll(state); client.load(coreFixture(state));
    useGame.setState({ client, humans: [0, 1], views: [client.view(0), client.view(1)], seat: 0,
      orders: [[], []], errors: [[], []], ready: [false, false], diplomacyOpen: true, diplomacyTeam: 1 });
    act(() => root.render(<><VpTrack /><DiplomacyPanel /></>));
    return client;
  }
  function click(text: string) {
    const button = [...container.querySelectorAll<HTMLButtonElement>('button')].find((b) => (b.getAttribute('aria-label') ?? b.textContent)?.startsWith(text));
    expect(button).toBeDefined();
    act(() => button!.click());
  }
  function refresh(client: WasmClient, seat = 0) {
    act(() => useGame.setState({ views: [client.view(0), client.view(1)], seat, orders: [[], []], errors: [[], []], diplomacyOpen: true, diplomacyTeam: seat === 0 ? 1 : 0 }));
  }
  it('forecasts a queued surprise declaration that takes effect this turn (G20)', async () => {
    const client = await start();
    expect(container.textContent).toContain('Takes effect this turn');
    click('Declare War');
    expect(container.textContent).toContain('Queued: Declare War');
    expect(client.view(0).diplomacy.relations[0].state).toBe('Peace');
    expect(client.validate(0, useGame.getState().orders[0])).toEqual([]);
    const result = client.endTurn({ 0: useGame.getState().orders[0] });
    refresh(client);
    expect(container.querySelector('.vhc-rel')?.getAttribute('data-relation')).toBe('War');
    const lines = formatEvents(result.events, contextFromViews(null, client.view(0)));
    expect(result.events.find((e) => e.type === 'WarDeclared')).toMatchObject({ relation: { effective_turn: 1 } });
    expect(lines.some((l) => l.category === 'diplomacy' && l.text.includes('started this turn'))).toBe(true);
    expect(lines.find((l) => l.category === 'diplomacy' && l.text.startsWith('War:'))?.details.join(' ')).toContain('the war began at once');
  });
  it('delivers a peace offer and accepts it with visible guarantee dates', async () => {
    const client = await start();
    client.endTurn({ 0: [{ type: 'DeclareWar', against: 1 }] }); refresh(client);
    click('Offer Peace');
    const sent = useGame.getState().orders[0];
    expect(client.validate(0, sent)).toEqual([]);
    client.endTurn({ 0: sent }); refresh(client, 1);
    expect(container.textContent).toContain('India offers Peace');
    expect(container.textContent).toContain('Respond by turn 5');
    expect(container.textContent).toContain('starts Peace in turn 4, guaranteed through turn 13');
    click('Accept Peace');
    const accepted = useGame.getState().orders[1];
    expect(client.validate(1, accepted)).toEqual([]);
    expect(client.view(1).diplomacy.relations[0].state).toBe('War');
    client.endTurn({ 1: accepted }); refresh(client, 1);
    expect(container.textContent).toContain('Earliest War: turn 14');
    const war = [...container.querySelectorAll<HTMLButtonElement>('button')].find((b) => (b.getAttribute('aria-label') ?? b.textContent) === 'Declare War');
    expect(war?.disabled).toBe(true);
    expect(war?.parentElement?.textContent).toContain('guaranteed through turn 13');
  });
  it('declines an offer without ending the war', async () => {
    const client = await start();
    client.endTurn({ 0: [{ type: 'DeclareWar', against: 1 }] });
    client.endTurn({ 0: [{ type: 'ProposeTreaty', to: 1, kind: 'Peace' }] }); refresh(client, 1);
    click('Decline');
    const result = client.endTurn({ 1: useGame.getState().orders[1] }); refresh(client, 1);
    expect(result.events.some((e) => e.type === 'ProposalRejected')).toBe(true);
    expect(client.view(1).diplomacy.proposals).toEqual([]);
    expect(client.view(1).diplomacy.relations[0].state).toBe('War');
  });
  it('signs an Alliance with upkeep on show; leaving early is a betrayal on the betrayer’s row', async () => {
    const { upkeep, breachCost, betrayalTurns } = allianceTerms();
    const client = makeClient();
    const setup = await client.standardSetup('9941', 'India', 'Byzantium', 'Ada', 'Bo');
    setup.players = setup.players.slice(0, 2); setup.players[1].team = 1;
    setup.width = setup.height = 24;
    const state = createState(setup);
    meetAll(state);
    state.turn = 5;
    for (const p of state.players) p.gold = 1000;
    client.load(coreFixture(state));
    useGame.setState({ client, humans: [0, 1], views: [client.view(0), client.view(1)], seat: 0,
      orders: [[], []], errors: [[], []], ready: [false, false], diplomacyOpen: true, diplomacyTeam: 1 });
    act(() => root.render(<><VpTrack /><DiplomacyPanel /></>));
    const offer = [...container.querySelectorAll<HTMLButtonElement>('button')].find((b) => b.getAttribute('aria-label') === 'Offer Alliance')!;
    expect(offer.querySelector('.cost-chip')).toBeNull();
    expect(container.textContent).toContain(`Upkeep ${upkeep} Gold/turn`);
    expect(container.textContent).not.toContain('Net income');
    click('Offer Alliance');
    client.endTurn({ 0: useGame.getState().orders[0] }); refresh(client, 1);
    expect(container.textContent).toContain('India offers an Alliance');
    expect(container.querySelector('.offer-for-you')!.textContent).toContain(`Upkeep ${upkeep} Gold/turn`);
    click('Accept Alliance');
    client.endTurn({ 1: useGame.getState().orders[1] }); refresh(client);
    expect(container.querySelector('.vhc-rel')?.getAttribute('data-relation')).toBe('Alliance');
    expect(container.textContent).toMatch(/Protected until turn \d+/);
    expect(client.view(0).economy.gold_per_turn.parts.some(([label]) => label.startsWith('Treaty upkeep'))).toBe(true);
    const leave = [...container.querySelectorAll<HTMLButtonElement>('button')].find((b) => b.getAttribute('aria-label') === 'Leave the Alliance')!;
    expect(leave.querySelector('.cost-chip')!.textContent).toContain(`−${breachCost}`);
    expect(container.textContent).toContain(`won't ally with you for ${betrayalTurns} turns`);
    expect(container.textContent).toContain('Declare War on your ally');
    click('Leave the Alliance');
    client.endTurn({ 0: useGame.getState().orders[0] }); refresh(client, 1);
    expect(container.querySelector('.betrayal-line')?.textContent).toMatch(/^\s*Broke an alliance on turn \d+: your team won't ally until turn \d+\s*$/);
    refresh(client);
    expect([...container.querySelectorAll('.diplomacy-unavailable')].map((p) => p.textContent).join(' ')).toMatch(/Offer Alliance\. .+ won't ally with .+ until turn \d+: betrayed on turn \d+/);
  });
});
