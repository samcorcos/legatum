import { describe, expect, it } from 'vitest';
import { FACTIONS, FACTION_GOAL, FACTION_INFO, THIN_LEVIES, companyRule } from './data';
import { FACTION_SHEETS, sheetCards } from './factionSheets';
import { iconUrl } from './icons';
import { leaderPortraitUrl } from './brand/ornaments';

describe('faction sheets', () => {
  it('draws an icon for every strength and price, and does not repeat one within a sheet', () => {
    for (const f of FACTIONS) {
      const names = FACTION_SHEETS[f].pairs.flatMap((p) => [p.strengthIcon, p.priceIcon].filter((n): n is string => !!n));
      for (const n of names) expect(iconUrl(n), `${f}: ${n}`).not.toBeNull();
      const strengths = FACTION_SHEETS[f].pairs.flatMap((p) => p.strengthIcon ? [p.strengthIcon] : []);
      const prices = FACTION_SHEETS[f].pairs.flatMap((p) => p.priceIcon ? [p.priceIcon] : []);
      expect(new Set(strengths).size, `${f} strengths`).toBe(strengths.length);
      expect(new Set(prices).size, `${f} prices`).toBe(prices.length);
    }
  });

  it('uses each historical people portrait', () => {
    for (const f of FACTIONS) {
      const portrait = f.toLowerCase();
      expect(leaderPortraitUrl(f), f).toContain(`people-${portrait}-360.jpg`);
    }
  });

  it('states faction benefits and prices as concrete game effects', () => {
    const copy = FACTIONS.flatMap((f) => FACTION_SHEETS[f].pairs.flatMap((p) => [p.strength ?? '', p.price ?? ""]));
    for (const line of copy) {
      expect(line).not.toMatch(/free lunch|forever|bleed yields|nothing is cheap/i);
    }
    expect(FACTION_SHEETS.Byzantium.pairs[0].strength).toBe(companyRule());
    expect(FACTION_SHEETS.Byzantium.pairs[0].price).toBe(THIN_LEVIES);
    expect(FACTION_SHEETS.Byzantium.pairs[0].price).not.toContain('120%');
    const swiss = FACTION_SHEETS.Switzerland.pairs[0].strength;
    expect(swiss).toBe('Each city earns +1 Gold for every 2 population. Every 100 Gold in your treasury earns +1 Gold a turn in interest.');
    expect(swiss).toBe(FACTION_INFO.Switzerland.ability.split(' You buy buildings and units')[0]);
    // One rule, one wording: the sheet's second row is the faction info's buying and capture sentences.
    expect(FACTION_INFO.Switzerland.ability).toContain(FACTION_SHEETS.Switzerland.pairs[1].strength);
    expect(FACTION_INFO.Switzerland.weakness).toContain(FACTION_SHEETS.Switzerland.pairs[1].price);
    expect(swiss).not.toMatch(/pair|up to|max/i);
    expect(FACTION_SHEETS.Germany.pairs[0].strength).toContain('Ruhr Foundry: +6 Production');
    expect(FACTION_SHEETS.Germany.pairs[0].price).toBeUndefined();
    expect(FACTION_SHEETS.Germany.pairs[1].price).toBe('Each Production building in a German city costs 1 Food.');
    expect(FACTION_SHEETS.Germany.pairs[1].strength).toBeUndefined();
    expect(FACTION_SHEETS.Germany.pairs[2].strength).toContain('strategic deposits supply twice as much');
    expect(FACTION_SHEETS.Germany.pairs).toHaveLength(5);
    expect(FACTION_SHEETS.Germany.pairs[3].strength).toContain('Industry project instead of building: each turn it turns half of its Production into Gold, or a quarter into Science');
    // A drawback that only restates its ability is not one: the project row has no price.
    expect(FACTION_SHEETS.Germany.pairs[3].price).toBeUndefined();
    expect(FACTION_SHEETS.India.pairs[0].price).toContain('Production never falls below 3');
    expect(FACTION_SHEETS.India.pairs[1].price).toContain('half the Gold they make');
    expect(FACTION_SHEETS.India.pairs[2].strength).toContain('Imperial Diwan');
    expect(FACTION_SHEETS.Mongols.pairs[0].price).toContain('raise no Walls, Castles or Star Forts');
    expect(FACTION_SHEETS.Mongols.pairs[0].price).toContain('buy no Siege before the Medieval era');
    expect(FACTION_SHEETS.Athens.lore).toBe('Classical Athens was known for civic debate and learning. In play, Athenian scholarship brings quick discoveries at a cost to its treasury.');
    expect(FACTION_SHEETS.Athens.lore).not.toContain('excluded many residents');
    expect(FACTION_GOAL.Mongols.detail()).toContain('Yam Relay Station');
  });

  it('shows Rome’s unit and building as cards, with its Settler restriction in the cost column', () => {
    expect(sheetCards('Rome').map((card) => card.name)).toEqual(['Legionary', 'Castrum']);
    expect(FACTION_SHEETS.Rome.pairs[0].strength).toBe('A full-health melee veteran (level 1 or higher) can found a city; the unit is consumed.');
    expect(FACTION_SHEETS.Rome.pairs[0].price).toBe('Rome cannot train or buy Settlers.');
    expect(FACTION_SHEETS.Rome.pairs.slice(1).every((pair) => pair.onCard)).toBe(true);
    expect(FACTION_SHEETS.Rome).not.toHaveProperty('piece');
    expect(FACTION_SHEETS.Rome).not.toHaveProperty('win');
    expect(FACTION_SHEETS.Rome).not.toHaveProperty('lose');
  });

  it('says what a Mercenary Company costs and buys', () => {
    const sheet = FACTION_SHEETS.Byzantium;
    const text = [sheet.identity, sheet.lore, sheet.lose, sheet.counteredBy, ...sheet.pairs.flatMap((p) => [p.strength, p.price ?? ""])].join(' ');
    expect(text).toMatch(/Company/);
    expect(text).toMatch(/countdown/i);
    expect(text).not.toMatch(/tribute|bought peace/i);
  });
});
