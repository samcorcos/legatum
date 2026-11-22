import { existsSync, readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { beforeAll, describe, expect, it } from 'vitest';
import { FACTIONS, TECHS, techHasBenefit } from '../data';
import { WasmClient } from '../wasmClient';
import { coreFixture } from '../test/coreFixture';
import { researchFixture } from '../test/researchFixture';
import type { Faction, PlayerView, TechId } from '../types';
import { momentsForTurn } from './moments';
import { contextFromViews, formatEvent } from '../format';
import { unlockCardFor } from './unlockCard';

const available = existsSync(fileURLToPath(new URL('../wasm-pkg/legatum.js', import.meta.url)));
const views = new Map<Faction, PlayerView>();
beforeAll(async () => {
  if (!available) return;
  const path = '../wasm-pkg/legatum.js'; const mod = await import(/* @vite-ignore */ path);
  await mod.default({ module_or_path: readFileSync(fileURLToPath(new URL('../wasm-pkg/legatum_bg.wasm', import.meta.url))) });
  const client = new WasmClient(mod.Game);
  for (const faction of FACTIONS) { client.load(coreFixture(researchFixture(faction))); views.set(faction, client.view(0)); }
});

describe.skipIf(!available)('the unlock card for every technology', () => {
  it('every own tech gets a major card with its quote, whatever the faction', () => {
    for (const [faction, view] of views) for (const tech of TECHS) {
      const [moment] = momentsForTurn([{ type: 'TechResearched', player: 0, tech }], 0, view, view);
      expect(moment?.tier, `${faction} ${tech}`).toBe('major');
      expect(moment.unlock?.quote, `${faction} ${tech}`).toBeTruthy();
      expect(moment.unlock?.kicker).toMatch(/^Technology discovered · Era [IVX]+ · \w+( \w+)?$/);
    }
  });

  it('shows unlocks as tiles with a plain sentence each: every card a faction researches gives it something', () => {
    for (const [faction, view] of views) for (const tech of TECHS) {
      // A tech with nothing for the faction is skipped: known from the start, never researched (Egypt: Bronze Working).
      if (!techHasBenefit(faction, tech)) continue;
      const card = unlockCardFor(view, tech);
      expect(card.tiles.length, `${faction} ${tech}`).toBeGreaterThan(0);
      for (const tile of card.tiles) if (tile.key && !tile.locked) expect(tile.sentence, `${faction} ${tech} ${tile.name}`).toMatch(/^((Build|Train|Upgrade|Add) ((a|an|the) )?[A-Z][^:]*: .+|Build it on a chosen tile; its holder scores \+2 VP\..*)/);
    }
  });

  it('says what the unlock does in real numbers, in the faction’s own words', () => {
    const sentence = (faction: Faction, tech: TechId) => unlockCardFor(views.get(faction)!, tech).tiles.map((t) => t.sentence).filter(Boolean);
    expect(sentence('India', 'BronzeWorking')).toEqual(['Build a Workshop in any city: +2 Production']);
    expect(sentence('Switzerland', 'Masonry')).toEqual([
      'Build it on a chosen tile; its holder scores +2 VP.',
      'Build an Alpine Redoubt in any city: +15 city strength, more each era. Walls cost 100 Production and give +10 city strength',
    ]);
    expect(sentence('India', 'Masonry').find((line) => line?.startsWith('Build Walls'))).toMatch(/^Build Walls in any city: \+10 city strength/);
    expect(sentence('Germany', 'BronzeWorking')).toEqual(['Build a Workshop in any city: +2 Production']);
    expect(sentence('Germany', 'IronWorking').some((line) => line?.includes('Ruhr Foundry'))).toBe(true);
    // The Foundry names the Forge's own number, never a bare difference; the Factory says what stays standing.
    expect(sentence('Germany', 'IronWorking')).toContain('Upgrade a Workshop to a Ruhr Foundry in a city of 4 or more: +6 Production. A Forge gives +3 Production');
    expect(sentence('Germany', 'Industrialization')).toContain('Add a Factory to a Ruhr Foundry city of 6 or more: +5 Production. The city keeps its Ruhr Foundry, and it still counts for Five Ruhr Foundries');
    expect(sentence('Athens', 'Writing')).toEqual(['Build a Lyceum in any city: +6 Science. A Library costs 80 Production and gives +3 Science']);
    expect(sentence('Byzantium', 'Writing')).toEqual(['Build a Library in any city: +3 Science']);
    expect(sentence('India', 'Philosophy')).not.toContain('Build an Academy in any city: +3 Science');
    // Byzantium researches like everyone; Philosophy still unlocks the Great Library.
    expect(sentence('Byzantium', 'Philosophy')).toEqual(sentence('India', 'Philosophy'));
    expect(sentence('India', 'Currency')).toEqual([
      expect.stringMatching(/^Build a Revenue Office in any city: \+3 Gold/),
      expect.stringMatching(/^Build an Imperial Diwan/),
    ]);
    // The Granary needs no research now: Irrigation unlocks no building, only +1 Food a city.
    expect(unlockCardFor(views.get('India')!, 'Irrigation').tiles.map((t) => t.name)).toEqual(['+1 Food in every city']);
    expect(sentence('India', 'CodeOfLaws')).toEqual([]);
    expect(sentence('India', 'IronWorking')).toEqual(['Upgrade a Workshop to a Forge in a city of 4 or more: +3 Production', 'Train a Cataphract in any city: 33 melee, 4 moves']);
    expect(sentence('India', 'Feudalism')[1]).toMatch(/^Train a Pikeman in any city: 32 melee, 2 moves/);
    expect(sentence('India', 'Philosophy')[0]).toMatch(/^Build it on a chosen tile; its holder scores \+2 VP\./);
    // A tech that upgrades an improvement says so, as an effect tile (no item behind it).
    const names = (faction: Faction, tech: TechId) => unlockCardFor(views.get(faction)!, tech).tiles.map((t) => t.name);
    expect(names('India', 'CropRotation')).toContain('Farms and Pastures upgrade: Irrigated Farm, Ranch (+1 Food)');
    expect(names('Byzantium', 'IronWorking')).toContain('Mines upgrade: Deep Mine (+1 Gold on Copper, +1 Production on Iron and Coal)');
    // The Mongols raise Walls from the Medieval era, so their Masonry card lists Walls like everyone's.
    expect(names('Mongols', 'Masonry')).not.toEqual([]);
    // ...but cannot raise them yet: the tile says so, from the core view, in place of the sentence and the price.
    const walls = unlockCardFor(views.get('Mongols')!, 'Masonry').tiles.find((t) => t.name === 'Walls')!;
    expect(walls.locked).toBe('Walls for the Mongols from the Medieval era');
    expect(walls.sentence).toBe(walls.locked);
    expect(walls.cost).toBeNull();
    expect(unlockCardFor(views.get('India')!, 'Masonry').tiles.find((t) => t.name === 'Walls')!.locked).toBeNull();
    expect(unlockCardFor(views.get('Byzantium')!, 'PrintingPress').tiles).not.toEqual([]);
  });

  it('names the research that continues and the city that builds the unlock', () => {
    const view = structuredClone(views.get('India')!);
    view.research.current = 'Currency'; view.research.turns_left = 12;
    const card = unlockCardFor(view, 'BronzeWorking');
    expect(card.next).toEqual({ name: 'Currency', turns: 12, unlocks: 'Revenue Office, Imperial Diwan' });
    // The fixture knows BronzeWorking, so its capital can build the Workshop now.
    const cityName = view.cities.find((city) => city.owner === view.me)!.name;
    expect(card.show).toMatchObject({ itemName: 'Workshop', cityName });
    expect(card.tiles[0]).toMatchObject({ inCity: cityName, kind: 'Building' });
    // A tech whose items no city can build yet has nowhere to go: the primary is Continue.
    expect(unlockCardFor(view, 'Currency').show).toBeNull();
    expect(card.primary).toBe('show');
    expect(unlockCardFor(view, 'Currency').primary).toBe('continue');
    expect(card.chooseNext).toBe(false);
    view.research.current = null;
    expect(unlockCardFor(view, 'BronzeWorking').chooseNext).toBe(true);
    // Idle research with nothing to build: Choose research is the primary; with a building it stays Show.
    expect(unlockCardFor(view, 'BronzeWorking').primary).toBe('show');
    expect(unlockCardFor(view, 'Currency').primary).toBe('research');
    expect(unlockCardFor(view, 'BronzeWorking').next).toBeNull();
  });

  it('the report line names what was unlocked, like the growth line, and reopens the card; a partner’s line does not', () => {
    const view = views.get('Switzerland')!;
    const ctx = contextFromViews(view, view);
    const mine = formatEvent({ type: 'TechResearched', player: view.me, tech: 'Masonry' }, ctx);
    expect(mine.text).toMatch(/researched Masonry · unlocks Pyramids, Alpine Redoubt$/);
    expect(mine.reopenTech).toBe('Masonry');
    expect(formatEvent({ type: 'TechResearched', player: view.me, tech: 'BronzeWorking' }, ctx).text).toMatch(/researched Bronze Working · unlocks Workshop$/);
    // No item: nothing to name. Another empire's tech is theirs.
    expect(formatEvent({ type: 'TechResearched', player: view.me, tech: 'Irrigation' }, ctx).text).toMatch(/researched Irrigation$/);
    const other = formatEvent({ type: 'TechResearched', player: 1, tech: 'BronzeWorking' }, ctx);
    expect(other.text).not.toContain('unlocks'); expect(other.reopenTech).toBeUndefined();
  });
});
