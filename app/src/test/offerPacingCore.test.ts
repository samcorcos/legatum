// AI offer pacing in the WASM engine, with the same cases as crates/ai/tests/offer_pacing.rs.
import { existsSync, readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { beforeAll, describe, expect, it } from 'vitest';
import { RULES } from '../data';
import { makeStandardSetup } from './fixtureEngine';
import { WasmClient } from '../wasmClient';
import type { GameClient } from '../client';


it('acquaintance takes 15 turns', () => {
  expect(RULES.aiAllianceAfterTurn).toBe(15);
});

const available = existsSync(fileURLToPath(new URL('../wasm-pkg/legatum.js', import.meta.url)));
let makeCore: () => WasmClient;
beforeAll(async () => {
  if (!available) return;
  const pkg = '../wasm-pkg/legatum.js'; const mod = await import(/* @vite-ignore */ pkg);
  await mod.default({ module_or_path: readFileSync(fileURLToPath(new URL('../wasm-pkg/legatum_bg.wasm', import.meta.url))) });
  makeCore = () => new WasmClient(mod.Game);
});

/** One human who leaves the starting capital idle, and three AIs. */
async function playOffers(client: GameClient, seed: string) {
  const setup = makeStandardSetup(seed, 'Byzantium', 'India', 'Ada', 'Envoy');
  setup.players = setup.players.slice(0, 4).map((p, id) => ({ ...p, team: id, is_human: id === 0 }));
  await client.newGame(setup);
  const offers = new Map<number, { turn: number; from: number; to: number; kind: string }>();
  for (let i = 0; i < 40; i++) {
    const turn = client.view(0).turn;
    const result = client.endTurn({});
    // Proposals are private to both teams: gather every player's report, once per proposal.
    for (const events of Object.values(result.events_by_player ?? { 0: result.events })) {
      for (const e of events) {
        if (e.type === 'ProposalCreated') offers.set(e.proposal.id, { turn, from: e.proposal.from, to: e.proposal.to, kind: e.proposal.kind ?? 'Peace' });
      }
    }
  }
  return [...offers.values()];
}

describe('AIs open without Alliances and stagger offers to a human', () => {
  it.skipIf(!available).each(['700001', '700040'])(`seed %s`, async (seed) => {
    const offers = await playOffers(makeCore(), seed);
    expect(offers.filter((o) => o.kind === 'Alliance' && o.turn <= 15)).toEqual([]);
    const toHuman = new Map<number, number>();
    for (const o of offers.filter((o) => o.to === 0)) toHuman.set(o.turn, (toHuman.get(o.turn) ?? 0) + 1);
    expect([...toHuman.values()].every((n) => n <= 1)).toBe(true);
  }, 15_000);
});
