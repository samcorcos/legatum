import { makeStandardSetup, createState, meetAll } from './fixtureEngine';
import { coreFixture } from './coreFixture';
import { refreshVision } from './fixtureEngine';

/** Review fixture: public alliances formed against the human, the visible leader. */
export function allianceCoreSave() {
  const setup = makeStandardSetup('9943', 'Switzerland', 'India', 'Switzerland', 'India');
  setup.players = setup.players.slice(0, 4).map((p, id) => ({ ...p, faction: (['Switzerland', 'India', 'Byzantium', 'Celts'] as const)[id], team: id, is_human: id === 0 }));
  const s = createState(setup);
  s.turn = 50;
  meetAll(s);
  s.players[0].name = 'Switzerland';
  s.players[1].name = 'India';
  s.players[2].name = 'Byzantium';
  s.players[3].name = 'Celts';
  s.players[0].factionGoalDone = true;
  s.players[0].eras = Object.fromEntries([1, 2, 3].map((era) => [era, { entered: 1, scored: { turn: 1, slot: 0 } }]));
  for (const [a, b, id] of [
    [1, 2, 1],
    [2, 3, 2],
  ]) {
    const r = s.diplomacy.relations.find((r) => r.a === a && r.b === b)!;
    Object.assign(r, {
      state: 'Alliance',
      effective_turn: 40,
      guarantee_until: 43,
      version: 1,
      security: {
        id,
        started_turn: 40,
        protected_until: 50,
      },
    });
  }
  for (const [i, other] of [1, 2].entries())
    Object.assign(s.diplomacy.relations.find((r) => r.a === 0 && r.b === other)!, {
      state: 'War',
      war_id: i + 1,
      effective_turn: 48,
      version: 1,
      declarers: [other],
    });
  s.diplomacy.next_treaty_id = 2;
  s.diplomacy.next_war_id = 2;
  s.diplomacy.next_proposal_id = 1;
  refreshVision(s);
  return coreFixture(s);
}

/** Turn 16: team 3 leads and teams 0 (human, 0 Gold) and 1 are at War with it. */
export function leaderAllianceOfferCoreSave() {
  const setup = makeStandardSetup('9943', 'Switzerland', 'India', 'Switzerland', 'India');
  // Four scored Age cards and a permanent goal are 5 VP: a low target makes that a lead big enough to threaten victory.
  setup.rule_experiments = { ...setup.rule_experiments, vp_target: 5 };
  setup.players = setup.players.slice(0, 4).map((p, id) => ({ ...p, team: id, is_human: id === 0 }));
  const s = createState(setup);
  s.turn = 16;
  meetAll(s);
  s.players[0].name = 'Switzerland';
  s.players[1].name = 'India';
  s.players[2].name = 'Byzantium';
  s.players[3].name = 'Celts';
  s.players[0].gold = 0;
  s.players[3].eras = Object.fromEntries([1, 2, 3, 4].map((era) => [era, { entered: 1, scored: { turn: 1, slot: 0 } }]));
  // A permanent goal scores from the flag; Byzantium's held goal would need its Heir.
  s.players[3].faction = 'India';
  s.players[3].goalRule = 'Swarm';
  s.players[3].factionGoalDone = true;
  for (const [i, other] of [0, 1].entries())
    Object.assign(s.diplomacy.relations.find((r) => r.a === other && r.b === 3)!, {
      state: 'War',
      war_id: i + 1,
      effective_turn: 14,
      version: 1,
      declarers: [3],
    });
  s.diplomacy.next_war_id = 2;
  s.diplomacy.next_proposal_id = 1;
  s.diplomacy.proposals = [{ id: 1, from: 1, to: 0, proposer: 1, kind: 'Alliance', created_turn: 16, respond_by: 18, relation_version: 0 }];
  refreshVision(s);
  return coreFixture(s);
}

/** Turn 16: team 1 is our ally and went to war with team 2 (the Byzantium), so team 0 (human) is called to arms. */
export function callToArmsCoreSave() {
  const setup = makeStandardSetup('9943', 'Switzerland', 'India', 'Switzerland', 'India');
  setup.players = setup.players.slice(0, 4).map((p, id) => ({ ...p, faction: (['Switzerland', 'India', 'Byzantium', 'Celts'] as const)[id], team: id, is_human: id === 0 }));
  const s = createState(setup);
  s.turn = 16;
  meetAll(s);
  s.players[0].name = 'Switzerland';
  s.players[1].name = 'India';
  s.players[2].name = 'Byzantium';
  s.players[3].name = 'Celts';
  s.players[0].gold = 300;
  const d = s.diplomacy;
  Object.assign(d.relations.find((r) => r.a === 0 && r.b === 1)!, {
    state: 'Alliance', effective_turn: 12, version: 1, security: { id: 1, started_turn: 12, protected_until: 22 },
  });
  Object.assign(d.relations.find((r) => r.a === 1 && r.b === 2)!, {
    state: 'War', war_id: 1, effective_turn: 15, version: 1, declarers: [1],
  });
  d.next_treaty_id = 1;
  d.next_war_id = 1;
  d.next_proposal_id = 3;
  d.calls = [{ id: 3, team: 0, ally: 1, target: 2, created_turn: 16, respond_by: 17, alliance_version: 1, war_version: 1 }];
  refreshVision(s);
  return coreFixture(s);
}
