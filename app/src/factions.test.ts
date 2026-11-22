import { describe, expect, it } from 'vitest';
import { FACTIONS, FACTION_INFO, FACTION_PIECES, factionDescription, factionGoalVp, itemDescription, itemName, techCardEffectsFor, unitStatsFor } from './data';
import { iconUrl } from './icons';
import { buildSetup, newLobby, setSlot } from './lobby';

describe('faction content and setup', () => {
  it('states unique-piece prices in city tooltips beside the yields and combat numbers', () => {
    const foundry = itemDescription(FACTION_PIECES.Germany.item, 'Germany');
    expect(foundry).toContain('320 Production');
    expect(foundry).toContain('+6 Production');
    expect(foundry).not.toContain('Food');
    expect(foundry).toContain('Upgrades a Workshop in a city of 4 or more');
    expect(foundry).toContain('Needs Iron');
    const lyceum = itemDescription(FACTION_PIECES.Athens.item, 'Athens');
    expect(lyceum).toContain('70 Production');
    expect(lyceum).toContain('+6 Science');
    const revenueOffice = itemDescription(FACTION_PIECES.India.item, 'India');
    expect(revenueOffice).toContain('60 Production');
    expect(revenueOffice).toContain('+3 Gold');
    expect(revenueOffice).toContain('half');
    const legionary = itemDescription(FACTION_PIECES.Byzantium.item, 'Byzantium');
    expect(legionary).toContain('162 Production');
    expect(legionary).toContain('84 Gold');
    expect(legionary).toContain('36 strength');
  });
  it('has complete unique-piece, rule, goal, weakness and heraldry data for every stable ID', () => {
    expect(FACTIONS).toEqual(['Switzerland', 'India', 'Byzantium', 'Germany', 'Celts', 'Mongols', 'Sparta', 'Athens', 'Aztecs', 'Egypt', 'Rome']);
    for (const f of FACTIONS) {
      const info = FACTION_INFO[f];
      for (const field of ['name', 'motto', 'ability', 'goal', 'weakness'] as const) expect(info[field].length).toBeGreaterThan(0);
      expect(info.rule).toContain(info.unique);
      expect(info.rule).toContain(info.weakness);
      expect(info.goal).toContain((f === 'Egypt' || f === 'Rome') ? 'No faction goal yet' : f === 'Byzantium' ? `Whoever holds him scores ${factionGoalVp(f)} VP` : f === 'India' ? `Whoever holds its city scores ${factionGoalVp(f)} VP` : `+${factionGoalVp(f)} ${f === 'Switzerland' || f === 'Germany' || f === 'Celts' ? 'VP while held' : f === 'Mongols' ? 'VP while you hold the Yam Relay Station' : 'VP once, kept for good'}`);
      expect(factionDescription(f)).toContain(info.goal);
      const piece = FACTION_PIECES[f];
      if (piece) expect(itemName(piece.item, f)).toBe(piece.name);
      expect(iconUrl(`faction-${f.toLowerCase()}`)).toBeTruthy();
      expect(info.crest).toContain(`crest-${f.toLowerCase()}-512`);
      expect(info.glyph).toContain(`glyph-${f.toLowerCase()}`);
    }
    // Every faction has its own glyph and its own rendered crest: no faction borrows another's mark.
    expect(new Set(FACTIONS.map((f) => iconUrl(`faction-${f.toLowerCase()}`))).size).toBe(FACTIONS.length);
    expect(new Set(FACTIONS.map((f) => FACTION_INFO[f].crest)).size).toBe(FACTIONS.length);
    expect(FACTION_INFO.Germany.name).toBe('Germany');
    expect(FACTION_INFO.Mongols.name).toBe('Mongols');
    expect(FACTION_INFO.Rome.goal).toBe('No faction goal yet.');
    expect(FACTION_INFO.Rome.crest).toContain('crest-rome-512.png');
    expect(FACTION_INFO.Rome.glyph).toContain('glyph-rome.svg');
    expect(FACTION_INFO.Rome.weakness).toContain('cannot train or buy Settlers');
    expect(itemName({ kind: 'Unit', unit: 'Phalangite' }, 'Rome')).toBe('Legionary');
    expect(itemName({ kind: 'Building', building: 'Walls' }, 'Rome')).toBe('Castrum');
    expect(itemDescription({ kind: 'Building', building: 'Walls' }, 'Rome')).toContain('50 HP');
  });
  it('resolves random seats without repeats, respects explicit picks, and is seed deterministic', () => {
    const seed = '18446744073709551615';
    const lobby = newLobby('solo', seed);
    const factions = buildSetup(lobby).players.map((p) => p.faction);
    expect(new Set(factions).size).toBe(factions.length);
    expect(buildSetup(lobby).players.map((p) => p.faction)).toEqual(factions);
    const picked = setSlot(lobby, 0, { faction: 'Sparta' });
    const assigned = buildSetup(picked).players.map((p) => p.faction);
    expect(assigned[0]).toBe('Sparta');
    expect(new Set(assigned).size).toBe(assigned.length);
    for (const faction of FACTIONS) {
      const lobby = setSlot(newLobby('solo', seed), 0, {faction});
      expect(buildSetup(lobby).players[0].faction).toBe(faction);
    }
  });
  it('shows the simulation-adjusted strengths and the requested cost and movement limits', () => {
    expect(FACTION_INFO.Germany.unique).toContain('320 Production, +6 Production');
    expect(FACTION_INFO.Germany.ability).toContain('Industry project instead of building');
    expect(FACTION_INFO.Sparta.unique).toContain('20 strength');
    expect(FACTION_INFO.Sparta.ability).toContain('Spartan Veteran');
    expect(FACTION_INFO.Sparta.goal).toContain('+2 VP once, kept for good');
    expect(FACTION_INFO.Sparta.goal).toContain('retire them there');
    expect(FACTION_INFO.Sparta.goal).toContain('Agoge');
    expect(FACTION_INFO.Celts.goal).toContain('the territory of each Celtic city');
    expect(FACTION_INFO.Sparta.weakness).toContain('50% more Production');
    expect(FACTION_INFO.Byzantium.goal).toContain('Whoever holds him scores 2 VP');
    expect(FACTION_INFO.Byzantium.weakness).toContain('at most one military unit per city, plus one');
    expect(FACTION_INFO.Switzerland.ability).toContain('Each city earns +1 Gold for every 2 population. Every 100 Gold in your treasury earns +1 Gold a turn in interest.');
    expect(FACTION_INFO.Switzerland.rule).not.toMatch(/pair|max 5|up to \+5|per 100 Gold held/i);
    expect(FACTION_INFO.India.weakness).toContain('Non-capital cities send no Gold');
    expect(FACTION_INFO.India.weakness).toContain('10% per tile beyond 4');
    expect(FACTION_INFO.India.ability).toContain('Cities can be founded 3 tiles apart.');
    expect(FACTION_INFO.Switzerland.goal).toContain('Keep 5,000 Gold in your treasury');
    expect(FACTION_INFO.Switzerland.goal).toContain('You lose the 2 VP while it is below that, and regain it once it recovers');
    expect(FACTION_INFO.Germany.weakness).toBe('Each Production building in a German city costs 1 Food.');
    expect(unitStatsFor('Germany', 'Horseman').moves).toBe(unitStatsFor('India', 'Horseman').moves);
    expect(itemDescription({ kind: 'Unit', unit: 'Horseman' }, 'Germany')).toContain('4 moves.');
    expect(techCardEffectsFor('Germany', 'Industrialization')).not.toContain('Every unit has normal movement from Industrialization.');
    expect(unitStatsFor('Germany', 'Settler').cost).toBe(unitStatsFor('India', 'Settler').cost);
    expect(FACTION_INFO.Switzerland.weakness).toContain('Military unit upkeep is triple: 3 Gold a turn instead of 1.');
    expect(FACTION_INFO.Celts.weakness).toContain("Celts can't keep cities they capture: they sack and burn them.");
    expect(FACTION_INFO.Celts.weakness).not.toContain('less strength');
    expect(FACTION_INFO.Celts.ability).toContain("Barbarians don't attack the Celts");
    expect(FACTION_INFO.Mongols.ability).toContain('at least 8 tiles from your capital');
    expect(FACTION_INFO.Mongols.ability).toContain('+2 Gold');
    for (const building of ['Market', 'Bank'] as const) {
      const description = itemDescription({ kind: 'Building', building }, 'Mongols');
      expect(description).toContain('at least 8 tiles from your capital');
      expect(description).toContain('also earns +2 Gold');
      expect(description).toContain('once for the Market and its upgrades');
    }
    expect(FACTION_INFO.Athens.goal).toContain('+2 VP once, kept for good');
    // Starting promoted is every Spartan unit's rule, not the Hoplite's; its own difference is the phalanx (pieceCompare).
    expect(itemDescription(FACTION_PIECES.Sparta.item, 'Sparta')).toContain('Spartan Hoplite replaces Spearman');
    expect(itemDescription(FACTION_PIECES.Sparta.item, 'Sparta')).not.toContain('starts promoted');
    expect(FACTION_INFO.Mongols.ability).toContain('Settlers keep 2 moves');
    expect(FACTION_INFO.Mongols.weakness).toContain('buy no Siege before the Medieval era');
    expect(FACTION_INFO.Mongols.weakness).toContain('raise no Walls, Castles or Star Forts');
    expect(FACTION_INFO.Switzerland.ability).toContain('You buy buildings and units at 175% of their Production cost in Gold (others pay 200%).');
    expect(FACTION_INFO.Switzerland.weakness).toContain('Lose a city and its captor takes that city’s share of your treasury, by population (twice that for your capital).');
  });
});

describe('faction-accurate items', () => {
  it('unlocks Mongol Walls and Siege from the Medieval era', async () => {
    const { availableToPlayer, factionUnavailableReason, isWorkshopLineBuilding, siegeAvailable } = await import('./data');
    const walls = { kind: 'Building', building: 'Walls' } as const;
    expect(siegeAvailable('Mongols', 1)).toBe(false);
    expect(siegeAvailable('Mongols', 2)).toBe(true);
    expect(siegeAvailable('Switzerland', 0)).toBe(true);
    expect(availableToPlayer('Mongols', walls, 1)).toBe(false);
    expect(factionUnavailableReason(walls, 'Mongols', 1)).toBe('The Mongols raise Walls from the Medieval era');
    expect(availableToPlayer('Mongols', walls, 2)).toBe(true);
    expect((['Workshop', 'Forge', 'Factory'] as const).every(isWorkshopLineBuilding)).toBe(true);
  });

  it('names and describes the Byzantium buildings as ordinary ones, with city Science', async () => {
    const { availableToFaction, buildingStatsFor } = await import('./data');
    const lab = { kind: 'Building', building: 'Library' } as const;
    expect(itemName(lab, 'Byzantium')).toBe('Library');
    expect(itemName(lab, 'Athens')).toBe('Lyceum');
    expect(itemName(lab, 'India')).toBe('Library');
    expect(itemName({ kind: 'Building', building: 'Walls' }, 'Switzerland')).toBe('Alpine Redoubt');
    expect(itemName({ kind: 'Unit', unit: 'Archer' }, 'Celts')).toBe('Archer');
    expect(itemDescription(lab, 'Byzantium')).toBe(itemDescription(lab, 'India'));
    expect(itemDescription({ kind: 'Building', building: 'University' }, 'Byzantium')).toBe('+4 Science. Upgrades your Library. Needs 5 population.');
  expect(buildingStatsFor('Byzantium', 'Library').yields.science).toBeGreaterThan(0);
    expect(availableToFaction('India', { kind: 'Project', project: 'Gold' })).toBe(false);
    expect(availableToFaction('Germany', { kind: 'Project', project: 'Science' })).toBe(true);
    expect(availableToFaction('Rome', { kind: 'Unit', unit: 'Settler' })).toBe(false);
    expect(itemDescription(FACTION_PIECES.Byzantium.item, 'Byzantium')).toContain('Tagmatic Soldier replaces Pikeman');
  });

  it('gives the Byzantium a Library like everyone, and its Great Library needs one', async () => {
    const { availableToFaction, greatWorkRequirement } = await import('./data');
    const library = { kind: 'Building', building: 'Library' } as const;
    expect(availableToFaction('Byzantium', library)).toBe(true);
    expect(greatWorkRequirement('GreatLibrary')).toEqual(['Library', 1]);
    expect(greatWorkRequirement('RoyalArsenal')).toEqual(['Factory', 1]);
  });

  it('states the Byzantium Company rule from the generated numbers, in plain words', () => {
    expect(FACTION_INFO.Byzantium.ability).toBe('Hire a Company: the special unit of an empire you have met and are not at War with, as that empire fields it. You pay its Production cost in Gold, and the seller keeps it.');
    expect(FACTION_INFO.Byzantium.weakness).toBe("Thin levies: you can field at most one military unit per city, plus one more. Aircraft count; Scouts don't. Losing a city never disbands units, but you can't raise new ones until you are back under the limit.");
  });
});
