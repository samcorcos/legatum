import { existsSync, readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { beforeAll, expect, it } from 'vitest';
import { WasmClient } from '../wasmClient';
import { diplomacyCoreSave, diplomacyState } from './diplomacyFixture';
import { coreFixture } from './coreFixture';
import { allianceTerms } from '../treaties';

const available = existsSync(fileURLToPath(new URL('../wasm-pkg/legatum.js', import.meta.url)));
let client: WasmClient;
beforeAll(async () => {
  if (!available) return;
  const path = '../wasm-pkg/legatum.js'; const mod = await import(/* @vite-ignore */ path);
  await mod.default({ module_or_path: readFileSync(fileURLToPath(new URL('../wasm-pkg/legatum_bg.wasm', import.meta.url))) });
  client = new WasmClient(mod.Game);
});

/** The seeded scenario at `turn` with an Alliance offer from the AI (team 1) awaiting our reply. */
function allianceOffer(turn: number, gold = 1000) {
  const state = diplomacyState(turn);
  state.players[0].gold = gold;
  state.diplomacy!.next_proposal_id = 1;
  state.diplomacy!.proposals = [{ id: 1, from: 1, to: 0, proposer: 1, kind: 'Alliance', created_turn: turn, respond_by: turn + 2, relation_version: 0 }];
  return coreFixture(state);
}

it.skipIf(!available)('the seeded browser scenario accepts a valid Peace offer', () => {
  const state = diplomacyState(1);
  Object.assign(state.diplomacy!.relations[0], {
    state: 'War', war_id: 1, effective_turn: 1, version: 1, declarers: [0],
  });
  state.diplomacy!.next_war_id = 1;
  state.diplomacy!.next_proposal_id = 2;
  state.diplomacy!.proposals = [{ id: 1, from: 1, to: 0, proposer: 1, kind: 'Peace',
    created_turn: 1, respond_by: 3, relation_version: 1, consents: [1] }];
  client.load(coreFixture(state));
  expect(client.view(0).diplomacy.proposals[0].required_teams).toEqual([0, 1]);
  client.endTurn({ 0: [{ type: 'RespondToProposal', proposal: 1, accept: true }] });
  expect(client.view(0).diplomacy.relations[0].state).toBe('Peace');
});

it.skipIf(!available)('an Alliance accept quotes its upkeep without a signing cost', () => {
  const { upkeep } = allianceTerms();
  client.load(allianceOffer(10));
  const accept = client.view(0).diplomacy.actions.find((a) => a.order.type === 'RespondToProposal' && a.order.accept)!;
  expect(accept).toMatchObject({ cost: 0, upkeep, blocked_reason: null });
  client.load(allianceOffer(10, 12));
  const short = client.view(0).diplomacy.actions.find((a) => a.order.type === 'RespondToProposal' && a.order.accept)!;
  expect(short).toMatchObject({ cost: 0, upkeep, blocked_reason: null });
});

it.skipIf(!available)('an accepted Alliance adds upkeep to income, and leaving early is a public betrayal', () => {
  const { breachCost } = allianceTerms();
  client.load(allianceOffer(10));
  const result = client.endTurn({ 0: [{ type: 'RespondToProposal', proposal: 1, accept: true }] });
  let v = client.view(0);
  expect(v.diplomacy.relations[0].state).toBe('Alliance');
  expect(result.events.some((e) => e.type === 'TreatyPayment')).toBe(false);
  expect(v.economy.gold_per_turn.parts.some(([label, amount]) => label.startsWith('Treaty upkeep: Alliance with') && amount === -1)).toBe(true);
  expect(v.diplomacy.relations[0].security?.protected_until).toBeGreaterThan(v.turn);
  const leave = v.diplomacy.actions.find((a) => a.order.type === 'BreakTreaty')!;
  expect(leave).toMatchObject({ cost: breachCost, betrayal: true, blocked_reason: null });
  // Plain War on an ally is refused; the explicit betrayal is the way.
  const plain = v.diplomacy.actions.find((a) => a.order.type === 'DeclareWar' && !a.order.breach)!;
  expect(plain.blocked_reason).toMatch(/^You are allied with .+: leave the Alliance first, or declare war on your ally as a betrayal$/);
  client.endTurn({ 0: [leave.order] });
  v = client.view(0);
  expect(v.diplomacy.relations[0].state).toBe('Peace');
  expect(v.diplomacy.betrayals).toEqual([expect.objectContaining({ by: 0, against: 1 })]);
  const reoffer = v.diplomacy.actions.find((a) => a.order.type === 'ProposeTreaty' && a.order.kind === 'Alliance')!;
  expect(reoffer.blocked_reason).toMatch(/ won't ally with .+ until turn \d+: betrayed on turn \d+$/);
});

it.skipIf(!available)('a human alliance offer receives a real AI decision and saved reasons', () => {
  client.load(diplomacyCoreSave(16));
  client.endTurn({ 0: [{ type: 'ProposeTreaty', to: 1, kind: 'Alliance' }] });
  expect(client.view(0).diplomacy.proposals.some((p) => p.kind === 'Alliance' && p.from === 0)).toBe(true);
  client.endTurn({ 0: [] });
  const reply = client.view(0).diplomacy.replies?.find((r) => r.proposal.kind === 'Alliance');
  expect(reply).toBeDefined();
  expect(reply?.proposal.explanation?.reasons.length).toBeGreaterThan(0);
  expect(client.view(0).diplomacy.relations[0].state).toBe(reply!.accepted ? 'Alliance' : 'Peace');
});

