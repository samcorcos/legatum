// Injected visual fixtures only; not imported by the production app.
import { FixtureClient } from './fixtureEngine';
import { useGame } from '../store';
import { momentsForTurn, type Tier } from '../celebrate/moments';
import { contextFromViews, formatEvents } from '../format';
import { feedbackForTurn, playTurnFeedback } from '../feel';
import { soundHooksForMoments } from '../celebrate/feedback';
import type { Event } from '../types';

export async function celebrationFixture(tier: Tier, count = 1): Promise<void> {
  const client = new FixtureClient();
  const setup = await client.standardSetup('42', 'India', 'Byzantium', 'Your Dominion', 'Byzantium');
  setup.players[1].is_human = false;
  await client.newGame(setup);
  client.endTurn({});
  const before = client.view(0), after = structuredClone(before);
  const city = after.cities.find((c) => c.owner === 0)!;
  const event: Event = tier === 'minor' ? { type: 'CityGrew', city: city.id, owner: 0, population: 3 }
    : tier === 'legendary' ? { type: 'ProductionCompleted', city: city.id, owner: 0, item: { kind: 'Building', building: 'RoyalArsenal' } }
    : { type: 'RuinExplored', unit: 2, owner: 0, at: city.pos, reward: tier === 'major'
      ? { kind: 'Gold', amount: 40, balance: 80, affordable: { city: city.id, item: { kind: 'Unit', unit: 'Spearman' }, cost: 60 } }
      : { kind: 'Gold', amount: 40, balance: 80, affordable: null } };
  after.economy.gold = 80;
  if (tier === 'minor') city.population = 3;
  const events = Array.from({ length: count }, () => structuredClone(event));
  const moments = momentsForTurn(events, 0, before, after);
  useGame.setState({ client, humans: [0], views: [after], screen: 'game', booted: true, seat: 0,
    orders: [[]], errors: [[]], ready: [false], selection: null, mode: 'normal',
    reportOpen: false, reportFocus: undefined, techOpen: false, vpExpanded: false, busy: null, error: null,
    anim: null, focus: { hex: city.pos, seq: 1 }, net: null, lastHash: `fixture-${tier}`,
    report: { turn: after.turn - 1, hash: `fixture-${tier}`, events: [events], moments: [moments], lines: [formatEvents(events, contextFromViews(before, after))] },
  });
  window.__feelLog = [];
  playTurnFeedback(feedbackForTurn(events, after, before), () => true, soundHooksForMoments(moments, events));
}

/** A cleared barbarian camp: the same popup treatment as a ruin. */
export async function campCelebrationFixture(): Promise<void> {
  const client = new FixtureClient();
  const setup = await client.standardSetup('42', 'India', 'Byzantium', 'Your Dominion', 'Byzantium');
  setup.players[1].is_human = false;
  await client.newGame(setup);
  client.endTurn({});
  const before = client.view(0), after = structuredClone(before);
  const city = after.cities.find((c) => c.owner === 0)!;
  const cleared: Event = { type: 'BarbarianCampCleared', camp: 1, at: city.pos, by: 0, unit: 2, gold: 50 };
  const events: Event[] = [cleared];
  after.economy.gold += 50;
  const moments = momentsForTurn(events, 0, before, after);
  const hash = 'fixture-camp';
  useGame.setState({ client, humans: [0], views: [after], screen: 'game', booted: true, seat: 0,
    orders: [[]], errors: [[]], ready: [false], selection: null, mode: 'normal',
    reportOpen: false, reportFocus: undefined, techOpen: false, vpExpanded: false, busy: null, error: null,
    anim: null, focus: { hex: city.pos, seq: 1 }, net: null, lastHash: hash,
    report: { turn: after.turn - 1, hash, events: [events], moments: [moments], lines: [formatEvents(events, contextFromViews(before, after))] },
  });
  window.__feelLog = [];
  playTurnFeedback(feedbackForTurn(events, after, before), () => true, soundHooksForMoments(moments, events));
}

/**
 * A finished own technology: the centred unlock card. `buildable` puts a Workshop on the capital's menu, so the primary is
 * "Show Workshop in ..."; `researching` false leaves no research set (the common real case). `others` adds that many
 * ordinary cards behind it.
 */
export async function unlockCelebrationFixture(buildable = true, researching = true, others = 0): Promise<void> {
  const client = new FixtureClient();
  const setup = await client.standardSetup('42', 'India', 'Byzantium', 'Your Dominion', 'Byzantium');
  setup.players[1].is_human = false;
  await client.newGame(setup);
  client.endTurn({});
  const before = client.view(0), after = structuredClone(before);
  const city = after.cities.find((c) => c.owner === 0)!;
  const workshop = { kind: 'Building' as const, building: 'Workshop' as const };
  city.detail!.options = city.detail!.options.filter((o) => !(o.item.kind === 'Building' && o.item.building === 'Workshop'));
  if (buildable) city.detail!.options.push({ ...city.detail!.options[0], item: workshop, cost: 60, turns: 12 });
  after.research.unlocks = after.research.unlocks.filter((u) => !(u.tech === 'BronzeWorking' && u.item.kind === 'Building'));
  after.research.unlocks.push({ tech: 'BronzeWorking', item: workshop, name: 'Workshop', cost: 60, buy_cost: null, yields: { food: 0, production: 2, gold: 0, science: 0 }, city_strength: 0, unit_strength: 0, upgrade_cost: null });
  after.research.current = researching ? 'Currency' : null;
  after.research.turns_left = researching ? 12 : null;
  const ruin = (): Event => ({ type: 'RuinExplored', unit: 2, owner: 0, at: city.pos, reward: { kind: 'Gold', amount: 40, balance: 80, affordable: null } });
  const events: Event[] = [{ type: 'TechResearched', player: 0, tech: 'BronzeWorking' }, ...Array.from({ length: others }, ruin)];
  const moments = momentsForTurn(events, 0, before, after);
  useGame.setState({ client, humans: [0], views: [after], screen: 'game', booted: true, seat: 0,
    orders: [[]], errors: [[]], ready: [false], selection: null, mode: 'normal', unlockReopen: null,
    reportOpen: false, reportFocus: undefined, techOpen: false, vpExpanded: false, busy: null, error: null,
    anim: null, focus: { hex: city.pos, seq: 1 }, net: null, lastHash: 'fixture-unlock',
    report: { turn: after.turn - 1, hash: 'fixture-unlock', events: [events], moments: [moments], lines: [formatEvents(events, contextFromViews(before, after))] },
  });
  window.__feelLog = [];
}

const REWARD_KINDS = ['camp', 'ruin-gold', 'ruin-science', 'ruin-population', 'ruin-scout', 'captured', 'lost', 'founded', 'growth', 'greatwork', 'goal', 'tech-partner'] as const;
type RewardKind = (typeof REWARD_KINDS)[number];

/** One card per reward kind, for the emblem tour: each kind's own event on a founded India capital (or a Byzantium one when the kind needs it). */
export async function rewardFixture(kind: RewardKind): Promise<void> {
  const client = new FixtureClient();
  const setup = await client.standardSetup('42', 'India', 'Byzantium', 'Your Dominion', 'Byzantium');
  setup.players[1].is_human = false;
  await client.newGame(setup);
  client.endTurn({});
  const before = client.view(0), after = structuredClone(before);
  const city = after.cities.find((c) => c.owner === 0)!;
  const at = city.pos;
  if (kind === 'growth') {
    city.population = 2;
    city.detail!.population_unlocks = [{ population: 2, item: { kind: 'Unit', unit: 'Settler' } }];
  }
  const ruin = (reward: Extract<Event, { type: 'RuinExplored' }>['reward']): Event => ({ type: 'RuinExplored', unit: 2, owner: 0, at, reward });
  const table: Record<RewardKind, Event[]> = {
    camp: [{ type: 'BarbarianCampCleared', camp: 1, at, by: 0, unit: 2, gold: 50 }],
    'ruin-gold': [ruin({ kind: 'Gold', amount: 40, balance: 80, affordable: null })],
    'ruin-science': [ruin({ kind: 'Science', amount: 30, research: null })],
    'ruin-population': [ruin({ kind: 'Population', amount: 1, city: city.id, population: 2, yields: { food: 1, production: 0, gold: 1, science: 0 } })],
    'ruin-scout': [ruin({ kind: 'FreeScout', unit: 2 })],
    captured: [{ type: 'CityCaptured', city: city.id, from: 2, to: 0 }],
    lost: [{ type: 'CityCaptured', city: city.id, from: 0, to: 2 }],
    founded: [{ type: 'CityFounded', city: city.id, owner: 0, name: city.name, at }],
    growth: [{ type: 'CityGrew', city: city.id, owner: 0, population: 2 }],
    greatwork: [{ type: 'ProductionCompleted', city: city.id, owner: 0, item: { kind: 'Building', building: 'RoyalArsenal' } }],
    goal: [{ type: 'FactionGoalCompleted', player: 0, faction: 'India', vp: 3 }],
    'tech-partner': [{ type: 'TechResearched', player: 1, tech: 'BronzeWorking' }],
  };
  const events = table[kind];
  const moments = momentsForTurn(events, 0, before, after);
  const hash = `fixture-reward-${kind}`;
  useGame.setState({ client, humans: [0], views: [after], screen: 'game', booted: true, seat: 0,
    orders: [[]], errors: [[]], ready: [false], selection: null, mode: 'normal', unlockReopen: null,
    reportOpen: false, reportFocus: undefined, techOpen: false, vpExpanded: false, busy: null, error: null,
    anim: null, focus: { hex: at, seq: 1 }, net: null, lastHash: hash,
    report: { turn: after.turn - 1, hash, events: [events], moments: [moments], lines: [formatEvents(events, contextFromViews(before, after))] },
  });
  window.__feelLog = [];
}
