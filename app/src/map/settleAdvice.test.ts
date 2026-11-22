import { describe, expect, it } from 'vitest';
import type { PlayerView, SettlementSite } from '../types';
import { siteBandOf } from './siteBands';
import { STRATEGIC_SITE_VALUE } from '../resources';
import { bandTitle, blockerText, landClause, qualityLines, qualityParts, settleSummary, siteVerdict, yieldPhrase } from './settleAdvice';

const potential = { food: 18, production: 12, gold: 3, science: 0 };
const base = {
  pos: { q: 4, r: 4 }, score: 120, rank: 1, travel_turns: 3, threats: 0, border_risk: false, foreign_tiles: 0, unknown_tiles: 0, defense_bonus: 0,
  potential_yields: potential, resources: [], wonders: [], breakdown: [],
} as unknown as SettlementSite;
const quality = (parts: ReturnType<typeof qualityParts>) => parts.reduce((n, p) => n + p.points, 0);
const make = (patch: Partial<SettlementSite>): SettlementSite => {
  const site = { ...base, ...patch } as SettlementSite;
  const q = quality(qualityParts(site));
  return { ...site, quality: q, band: siteBandOf(q) };
};

describe('the hover leads with a plain sentence', () => {
  it('words the land by how much Food and Production it holds', () => {
    const land = (food: number, production: number) => landClause({ food, production, gold: 0, science: 0 });
    expect(land(30, 22)).toBe('plenty of Food and Production nearby');
    expect(land(14, 9)).toBe('some Food and Production nearby');
    expect(land(3, 2)).toBe('little Food or Production nearby');
    expect(land(30, 3)).toBe('plenty of Food but little Production nearby');
    expect(land(4, 22)).toBe('little Food but plenty of Production nearby');
    expect(land(30, 9)).toBe('plenty of Food and some Production nearby');
  });
  it('puts the band and the land first, then the nearest resources and the defence', () => {
    const rich = make({ potential_yields: { food: 23, production: 36, gold: 0, science: 0 }, resources: [{ resource: 'Wheat', distance: 3 }], defense_bonus: 3 });
    expect(siteVerdict(rich)).toEqual({ head: `${rich.band} spot for a city`, rest: 'plenty of Food and Production nearby, Wheat 3 tiles away and the terrain gives units here +3 defence.' });
    expect(siteVerdict(make({ border_risk: true })).head).toBe('Great spot for a city (unconfirmed)'.replace('Great', siteVerdict(make({})).head.split(' ')[0]));
    const bare = make({ potential_yields: { food: 2, production: 1, gold: 0, science: 0 } });
    expect(siteVerdict(bare).rest).toBe('little Food or Production nearby.');
    // The sentence names the two nearest resources; the rest get a line of their own.
    const many = make({ resources: [{ resource: 'Wheat', distance: 3 }, { resource: 'Iron', distance: 1 }, { resource: 'Coal', distance: 2 }] });
    expect(siteVerdict(many).rest).toBe('some Food and Production nearby, Iron 1 tile away and Coal 2 tiles away.');
    expect(qualityLines(many).find((l) => l.id === 'resources')?.text).toBe('Also nearby: Wheat 3 tiles away');
    // Several deposits of one kind are one mention, at the nearest.
    const twins = make({ resources: [{ resource: 'Wheat', distance: 3 }, { resource: 'Wheat', distance: 2 }, { resource: 'Iron', distance: 4 }] });
    expect(siteVerdict(twins).rest).toContain('Wheat 2 tiles away and Iron 4 tiles away');
    expect(qualityLines(twins).some((l) => l.id === 'resources')).toBe(false);
    expect(qualityLines(make({ foreign_tiles: 1 })).map((l) => l.text)).toContain('1 tile nearby belongs to another empire and adds nothing');
  });
  it('is plain: no points, weights, rings or code words in the sentence or the key numbers', () => {
    const site = make({ resources: [{ resource: 'Wheat', distance: 1 }, { resource: 'Iron', distance: 2 }], wonders: [{ wonder: 'MountFuji', distance: 2, value: 50 }], foreign_tiles: 3, unknown_tiles: 4, defense_bonus: 5, border_risk: true });
    const text = [siteVerdict(site).head, siteVerdict(site).rest, ...qualityLines(site).map((l) => l.text)].join(' ');
    expect(text).not.toMatch(/points|ring|half|counts \d|×|\bhex|score|quality|\(\+\d+\)/i);
  });
});

describe('the key numbers say what the land gives and what is missing', () => {
  it('lists what the land gives in plain signed numbers', () => {
    expect(yieldPhrase(potential)).toBe('+18 Food, +12 Production, +3 Gold');
    const site = make({});
    expect(qualityLines(site)[0]).toEqual({ id: 'land', text: 'Nearby land: +18 Food, +12 Production, +3 Gold' });
    expect(qualityLines(make({ potential_yields: { food: 0, production: 0, gold: 0, science: 0 } }))[0].text).toBe('Nearby land: no Food, Production or Gold');
  });
  it('names foreign land, unexplored land and the unseen-city risk; the arithmetic stays in the parts, and they sum to the quality', () => {
    const site = make({
      resources: [{ resource: 'Wheat', distance: 1 }, { resource: 'Iron', distance: 2 }],
      wonders: [{ wonder: 'MountFuji', distance: 2, value: 50 }], foreign_tiles: 3, unknown_tiles: 4, defense_bonus: 5, border_risk: true,
    });
    expect(qualityLines(site).map((l) => l.text)).toEqual([
      'Nearby land: +18 Food, +12 Production, +3 Gold',
      '3 tiles nearby belong to another empire and add nothing',
      '4 tiles nearby are not explored yet, so this spot may be better than it looks',
      'A foreign city may stand unseen nearby, so founding here could fail until you scout it',
    ]);
    expect(qualityParts(site)[0]).toEqual(expect.objectContaining({ id: 'land', points: 18 * 4 + 12 * 3 + 3 * 2 }));
    expect(quality(qualityParts(site))).toBe(18 * 4 + 12 * 3 + 3 * 2 + STRATEGIC_SITE_VALUE + 50 - 24);
    expect(qualityParts(site).map((p) => p.id)).toEqual(['land', 'strategic', 'foreign', 'wonder']);
    expect(qualityLines(site).at(-1)?.id).toBe('risk');
  });
  it('explains the band without a travel term', () => {
    expect(bandTitle(make({}))).toContain('fixed');
  });
});

describe('the relative half stays', () => {
  it('names the top-three marker and the route, separately from the band', () => {
    expect(settleSummary({ rank: 1, travel_turns: 3, threats: 0 })).toBe('Best spot · 3 turns away');
    expect(settleSummary({ rank: 2, travel_turns: 1, threats: 0 })).toBe('#2 spot · 1 turn away');
    expect(settleSummary({ rank: null, travel_turns: 0, threats: 0 })).toBe('Settle here now');
    expect(settleSummary({ rank: null, travel_turns: null, threats: 0 })).toBe('No known route');
    expect(settleSummary({ rank: null, travel_turns: 2, threats: 1 })).toBe('2 turns away · exposed to enemy attacks');
  });
});

describe('an illegal tile says why, naming the empire or city', () => {
  const view = { me: 0, players: [{ id: 0, team: 0, name: 'You' }, { id: 1, team: 1, name: 'Sparta', faction: 'Sparta' }] } as unknown as PlayerView;
  it('words every blocker', () => {
    expect(blockerText(view, { Foreign: 1 })).toMatch(/^Foreign territory \(.+\)$/);
    expect(blockerText(view, { TooClose: { city: 'Argos', owner: 1, distance: 2 } })).toMatch(/^Too close to Argos of .+ \(2 tiles away\); cities must be at least 4 tiles apart$/);
    expect(blockerText(view, { TooClose: { city: 'Home', owner: 0, distance: 1 } })).toBe('Too close to Home (1 tile away); cities must be at least 4 tiles apart');
    expect(blockerText(view, { TooClose: { city: 'Home', owner: 0, distance: 0 } })).toBe('Home already stands here');
    expect(blockerText(view, { Wonder: 'MountFuji' })).toBe('Mount Fuji is a natural wonder: it cannot hold a city');
    expect(blockerText(view, { Impassable: 'Mountains' })).toBe('Cities cannot be founded on Mountains');
    expect(blockerText(view, { Water: 'Coast' })).toBe('Cities can only be founded on land');
    expect(blockerText(view, 'FailedBefore')).toMatch(/unseen city/);
  });
});

describe('an Egyptian Settler hears about Food alone', () => {
  it('never mentions Production', () => {
    for (const food of [0, 20, 60, 200]) {
      const text = landClause({ food, production: 0, gold: 0, science: 0 }, 'Egypt');
      expect(text).not.toContain('Production');
      expect(text).toMatch(/Food nearby$/);
    }
  });
});
