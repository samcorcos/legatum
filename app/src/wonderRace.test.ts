import { beforeEach, describe, expect, it } from 'vitest';
import { FixtureClient } from './test/fixtureEngine';
import type { CityView, Event, PlayerView } from './types';
import { cityProductionWarning, idleCitiesNeedingProduction } from './ui/forecastWarnings';
import { contextFromViews, formatEvents } from './format';
import { momentsForTurn } from './celebrate/moments';
import { empireOfPlayer } from './empires';
import { itemName } from './data';

// Thun, 8.9.26 T92: India finished the Hagia Sophia the turn Thun switched to it, with its Mill
// queued next. He saw "Mill can't start yet" and nothing that said he had lost the race.
let view: PlayerView;
let city: CityView;
let rival: number;
beforeEach(async () => {
  const client = new FixtureClient();
  await client.newGame(await client.standardSetup('42', 'Switzerland', 'India', 'Ada', 'Bo'));
  client.endTurn({});
  view = client.view(0);
  city = view.cities.find((c) => c.owner === view.me)!;
  // A rival on another team when the setup has one (the two humans share a team).
  rival = (view.players.find((p) => p.team !== view.team) ?? view.players.find((p) => p.id !== view.me)!).id;
  view.diplomacy.met = [...(view.diplomacy.met ?? []), view.players.find((p) => p.id === rival)!.team];
});

describe('a lost wonder race', () => {
  const mill = { kind: 'Building', building: 'Hydroponics' } as const;
  const stop = () => {
    city.detail!.producing = null;
    city.detail!.production_queue = [mill];
    city.detail!.production_stored = 0;
    city.detail!.idle_production_waste = 0;
    city.detail!.wonder_lost = { turn: view.turn - 1, winner: rival, building: 'HagiaSophia', production: 30, gold: 15 };
  };

  it('makes the city a choose-production decision that names the rival, the wonder and the refund', () => {
    stop();
    const india = empireOfPlayer(view, rival);
    const warning = cityProductionWarning(city, [], 'Switzerland', view)!;
    expect(warning).toBe(`${india} completed the Hagia Sophia first: ${city.name} stopped building it. `
      + `You got 15 Gold back, half the 30 Production ${city.name} had put into it. Choose what ${city.name} builds next.`);
    expect(warning).not.toContain("can't start");
    expect(idleCitiesNeedingProduction(view, [])[0]).toBe(city);
    // Choosing clears the decision (the pending order replaces it).
    expect(cityProductionWarning(city, [{ type: 'SetProductionQueue', city: city.id, queue: [mill] }], 'Switzerland', view)).toBeNull();
  });

  it('says the same in the turn report and on a moment card', () => {
    stop();
    const event: Event = { type: 'WonderRaceLost', city: city.id, owner: view.me, winner: rival, building: 'HagiaSophia', production: 30, gold: 15, active: true };
    const india = empireOfPlayer(view, rival);
    const [line] = formatEvents([event], contextFromViews(view, view));
    expect(line.text).toBe(`${india} completed the Hagia Sophia first: ${city.name} stopped building it. You got 15 Gold back, half the 30 Production ${city.name} had put into it.`);
    expect(line.tone).toBe('bad');
    // A city that only had it queued (listed first) is not the card's city, and its line says so.
    const queuedOnly: Event = { ...event, city: city.id + 1000, production: 0, gold: 0, active: false };
    expect(formatEvents([queuedOnly], contextFromViews(view, view))[0].text).toContain("so it was taken out of");
    const cards = momentsForTurn([queuedOnly, event], view.me, view, view);
    expect(cards).toHaveLength(1);
    expect(cards[0]).toMatchObject({ tier: 'major', title: 'Hagia Sophia', kicker: 'Wonder race lost', place: city.name, choose: city.id, loss: true });
    expect(cards[0].details.join(' ')).toContain(`Choose what ${city.name} builds next.`);
  });

  it('a queued item that is genuinely blocked says its own reason, not that it resumes', () => {
    stop();
    const option = city.detail!.options.find((o) => o.item.kind === 'Unit')!;
    option.resource_blocker = 'Needs 1 Iron (you have 0).';
    city.detail!.production_queue = [option.item];
    const name = itemName(option.item, 'Switzerland');
    expect(cityProductionWarning(city, [], 'Switzerland', view)).toContain(`${name}, next in its queue, can't start yet: Needs 1 Iron (you have 0).`);
    // Once the race is no longer news (next turn), the waiting item keeps its own reason.
    city.detail!.wonder_lost = null;
    expect(cityProductionWarning(city, [], 'Switzerland', view)).toContain(`${name} is waiting: Needs 1 Iron (you have 0)`);
  });
});

describe('a wonder race in progress', () => {
  it("warns with the fastest met rival's public pace", () => {
    const item = { kind: 'Wonder', building: 'Pyramids', tile: { q: 0, r: 0 } } as const;
    city.detail!.producing = item;
    city.detail!.production_stall = null;
    view.great_works = [{ player: rival, completed: null, building: 'Pyramids', city: null, progress: 200, cost: 240, production_per_turn: 20, turns: null, blocker: null }];
    const warning = cityProductionWarning(city, [], 'Switzerland', view)!;
    expect(warning).toContain(`${empireOfPlayer(view, rival)} is also building the Pyramids and should finish in about 2 turns`);
    expect(warning).toContain('gets half its Production on it back as Gold');
    // Ahead of the rival: say so first.
    city.detail!.options = [...city.detail!.options, { ...city.detail!.options[0], item: city.detail!.producing!, turns: 1 }];
    expect(cityProductionWarning(city, [], 'Switzerland', view)).toContain(`${city.name} should finish the Pyramids first (1 turn; ${empireOfPlayer(view, rival)} about 2 turns).`);
    city.detail!.options = city.detail!.options.slice(0, -1);
    // An unmet rival's pace is not public (a teammate's always is).
    if (view.players.find((p) => p.id === rival)!.team !== view.team) {
      view.diplomacy.met = [];
      expect(cityProductionWarning(city, [], 'Switzerland', view) ?? '').not.toContain('also building');
    }
  });
});
