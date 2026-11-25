import type { BuildingKind, Event, Faction, Hex, PlayerId, PlayerView, Resource, RuinReward, UnitKind } from '../types';
import { ageName, BUILDING_NAME, BUILDING_STATS, factionGoalVp, RULES, TECH_NAME, isGreatWork, itemName } from '../data';
import { TECH_QUOTE } from '../techQuotes';
import { claimedTile, growthUnlocks } from '../milestones';
import { yieldsText } from '../cityProgress';
import { empireOfPlayer, placeName } from '../empires';
import { isBarbarian } from '../barbarians';
import { blockedQueueHead, wonderLostLine } from '../wonderRace';
import { distance } from '../hex';
import { unlockCardFor, type UnlockCard } from './unlockCard';
import { eraUpCardFor, type EraUpCard } from './eraUp';
import { scoreCheckFor, type ScoreCheckCard } from './scoreCheckCards';
import { eraName } from '../ui/eraResearch';
import { cardName } from '../ageCards';
import { WONDER_INFO, wonderArt, wonderBonusSentence } from '../wonders';
import { isAircraft, readyNotice } from '../airText';
import { revealedByTech } from '../resources';
import { hexAt } from '../hex';
import { GOAL_CITY_ICON, GOAL_CITY_TITLE, goalCities, goalCitiesAt, goalCityKindOf, tombSites } from '../goalCities';

export type Tier = 'minor' | 'notable' | 'major' | 'legendary';
export type Counter = 'gold' | 'science' | 'population' | 'vp';
export const TIER_ORDER: Record<Tier, number> = { minor: 0, notable: 1, major: 2, legendary: 3 };
export const DURATION: Record<Tier, number> = { minor: 1000, notable: 3500, major: 5000, legendary: 6000 };
export const OWN_HEIR_BIRTH_TITLE = 'The Heir lives';
export const RAIDER_SACK_KICKER = 'A city sacked';
/**
 * The engraved emblem a reward kind wears on its card (`assets/rewards/`). Moments with none (faction goal)
 * keep the player's faction crest.
 */
export type Emblem = 'camp' | 'hall' | 'ruin' | 'captured' | 'founded' | 'growth' | 'milestone' | 'greatwork' | 'tech';
/** One thing a reward gave you: a yield or item icon (`yield-gold`, `unit-scout`) and its text ("+50 Gold"). */
export interface Gain { icon: string; text: string }
export interface Moment {
  id: string; eventIndex: number; tier: Tier; owner: PlayerId; faction: Faction;
  partner: boolean; at: Hex | null; title: string; effect: string; lore: string;
  details: string[]; deltas: { counter: Counter; amount: number }[]; loss?: boolean;
  scout?: { unit: number; to: Hex }; kicker?: string;
  /** Your own finished technology: the centred unlock card, which waits for you rather than timing out. */
  unlock?: UnlockCard;
  /** Your own entry into a new era: held like the unlock card (phase C replaces its rendering, `EraUpTakeover`). */
  eraUp?: EraUpCard;
  /** A World Age has begun: the score check every player gets (`ScoreCheck`), held like the era-up. */
  scoreCheck?: ScoreCheckCard;
  emblem?: Emblem;
  /** What you won, listed in full under the title (camps and ruins); other cards state `effect` alone. */
  gains?: Gain[];
  art?: string;
  /** The revealed resource; its medallion is resolved by the card renderer. */
  resource?: Resource;
  /** A unit's painted miniature as the card's picture (the card resolves the image: this module stays free of Vite-only imports). */
  unit?: { kind: UnitKind; faction?: Faction };
  /** A faction goal building's painted miniature as the card's picture (resolved by the card, like `unit`). */
  building?: BuildingKind;
  /** A painted goal scene (an `assets/unlocks` slug such as `goal-reserve`) as the card's picture, for goals with no building. */
  artSlug?: string;
  /** The card's picture is this faction's paired people portrait, greyed: the empire that lost the city. */
  frame?: Faction;
  /** A VP seal on the picture: the points this card scores (signed by `loss`). */
  seal?: number;
  /** A goal's mark (`goal-colonia`, `goal-royal-tomb`): the card's medallion, with the empire's crest as its corner badge. */
  goalMark?: string;
  /** The city a wonder card is about, shown under its title (the kicker names the event). */
  place?: string;
  /** The event leaves this city without a build: the card's primary is Choose production there. */
  choose?: number;
}

/** The painted building a faction goal's generic completion card shows (the faction's own version: Ruhr Foundry, Lyceum). */
const GOAL_BUILDING: Partial<Record<Faction, BuildingKind>> = { Germany: 'Forge', Athens: 'Library' };
/** The painted scene a goal with no building shows: Switzerland's Reserve, the Celts' Warbands, the Aztecs' New Fire. */
const GOAL_ART: Partial<Record<Faction, string>> = { Switzerland: 'goal-reserve', Celts: 'goal-warbands', Aztecs: 'goal-newfire', Egypt: 'goal-royaltomb' };

export function ruinText(reward: RuinReward, cityName: (id: number) => string, faction?: Faction): { title: string; effect: string; lore: string; details: string[]; tier: Tier; deltas: Moment['deltas']; gains: Gain[]; emblem: Emblem } {
  const text: ReturnType<typeof ruinText> = (() => { switch (reward.kind) {
    case 'Gold': return {
      title: 'The buried hoard', effect: `+${reward.amount} Gold`,
      lore: 'The hoard outlived its keepers. It opened for yours.',
      details: [`Treasury at discovery: ${reward.balance} Gold.`, ...(reward.affordable
        ? [`Made affordable: ${itemName(reward.affordable.item, faction)}, ${reward.affordable.cost} Gold in ${cityName(reward.affordable.city)}.`]
        : ['Banked for your next Buy order.'])],
      tier: reward.affordable ? 'major' : 'notable', deltas: [{ counter: 'gold', amount: reward.amount }],
      gains: [{ icon: 'yield-gold', text: `+${reward.amount} Gold` }], emblem: 'ruin',
    };
    case 'Science': return {
      title: 'An archive still awake', effect: `+${reward.amount} stored Science`,
      lore: 'Beneath the dust, a voice had kept its vigil.',
      details: reward.research ? [
        reward.research.completes ? `${TECH_NAME[reward.research.tech]} completed this turn.`
          : reward.research.turns_saved === null ? `Stored toward ${TECH_NAME[reward.research.tech]}; no ongoing Science income.`
          : `${TECH_NAME[reward.research.tech]}: ${reward.research.turns_saved} future turn${reward.research.turns_saved === 1 ? '' : 's'} saved at this turn’s income.`,
        `${reward.research.progress_before} → ${reward.research.progress_before + reward.amount} / ${reward.research.cost} Science before research resolves; includes ${reward.research.income} income this turn.`,
      ] : ['Stored for your next research.'],
      tier: reward.research?.completes ? 'major' : 'notable', deltas: [{ counter: 'science', amount: reward.amount }],
      gains: [{ icon: 'yield-science', text: `+${reward.amount} stored Science` }], emblem: 'ruin',
    };
    case 'Population': {
      const yields = Object.entries(reward.yields).filter(([, n]) => n !== 0)
        .map(([key, n]) => `${n > 0 ? '+' : ''}${n} ${key === 'food' ? 'Food before consumption' : key[0].toUpperCase() + key.slice(1)}`).join(', ');
      return { title: 'The last census', effect: `+${reward.amount} population · ${cityName(reward.city)} reaches ${reward.population}`,
        lore: 'They had waited for an heir to call them home.',
        details: [`Yield change at discovery: ${yields || 'no change'}.`, `Additional Food consumption: ${2 * reward.amount} per turn.`],
        tier: 'major', deltas: [{ counter: 'population', amount: reward.amount }],
        gains: [{ icon: 'city', text: `+${reward.amount} population in ${cityName(reward.city)}` }], emblem: 'ruin' };
    }
    case 'FreeScout': return {
      title: 'The dormant sentinel', effect: `+1 free ${itemName({ kind: 'Unit', unit: 'Scout' }, faction)} beside the ruin`,
      lore: 'Its watch was over. Its journey had just begun.',
      details: [`Ready to move next turn. Ordinary ${itemName({ kind: 'Unit', unit: 'Scout' }, faction)} upkeep and rules apply.`], tier: 'major', deltas: [],
      gains: [{ icon: 'unit-scout', text: `+1 free ${itemName({ kind: 'Unit', unit: 'Scout' }, faction)}` }], emblem: 'ruin',
    };
  } })();
  if (faction !== 'Athens') return text;
  return { ...text, effect: `${text.effect} · +20 Science (Athens)`,
    details: [...text.details, '+20 Science (Athens).'],
    deltas: [...text.deltas, { counter: 'science', amount: 20 }],
    gains: [...text.gains, { icon: 'yield-science', text: '+20 Science (Athens)' }] };
}

/** "+1 Production in Bukhara": what a completed world wonder adds to its city, or null when it adds no yield. */
function wonderYieldLine(building: Parameters<typeof isGreatWork>[0], city: string): string | null {
  const text = yieldsText({ food: 0, production: 0, gold: 0, science: 0, ...BUILDING_STATS[building].yields });
  return text === 'no yield' ? null : `${text} in ${city}`;
}
const WONDER_ONLY_ONE = 'Only one in the world: no other empire can build it.';
const possessive = (name: string) => (name.endsWith('s') ? `${name}'` : `${name}'s`);
const signedVp = (n: number) => `${n < 0 ? '−' : '+'}${Math.abs(n)} VP`;

/** Presentation only. Caller supplies this viewer's fog-filtered events and views. */
export function momentsForTurn(events: readonly Event[], viewer: PlayerId, before: PlayerView | null, after: PlayerView): Moment[] {
  const team = new Set(after.players.filter((p) => p.team === after.team).map((p) => p.id));
  const cities = new Map([...(before?.cities ?? []), ...after.cities].map((c) => [c.id, c]));
  const cityName = (id: number) => cities.get(id)?.name ?? 'your city';
  const faction = (owner: PlayerId) => after.players.find((p) => p.id === owner)?.faction;
  const capital = (owner: number) => [...cities.values()].find((c) => c.owner === owner && c.is_capital)?.pos ?? null;
  const founded = new Map(after.players.map((p) => [p.id, before?.players.find((b) => b.id === p.id)?.cities ?? p.cities]));
  const out: Moment[] = [];
  const push = (index: number, owner: number, value: Omit<Moment, 'id' | 'eventIndex' | 'owner' | 'faction' | 'partner'>) => {
    const player = after.players.find((p) => p.id === owner);
    if (!player || !team.has(owner) || (owner !== viewer && value.tier === 'minor')) return;
    out.push({ ...value, id: `${after.turn}:${index}:${owner}`, eventIndex: index, owner, faction: player.faction,
      partner: owner !== viewer, title: owner === viewer ? value.title : `${player.name} · ${value.title}`,
      // A partner's resources must never fly into the viewer's private counters.
      deltas: owner === viewer ? value.deltas : [],
    });
  };
  // A card every empire sees about another empire (a rival's Heir, Colonia or Royal Tomb): it wears that empire's crest.
  const publicCard = (index: number, owner: PlayerId, value: Omit<Moment, 'id' | 'eventIndex' | 'owner' | 'faction' | 'partner'>) => {
    const player = after.players.find((p) => p.id === owner);
    if (player) out.push({ ...value, id: `${after.turn}:${index}:${owner}`, eventIndex: index, owner, faction: player.faction, partner: false });
  };
  // Where a held goal city stands now, from the public lists (every empire can find it, through fog).
  const goalCityOf = (faction: Faction, holder: PlayerId) => {
    const kind = goalCityKindOf(faction);
    return kind ? goalCities(after).find((g) => g.kind === kind && g.holder === holder) : undefined;
  };
  // Where an empire's Royal Tomb under construction will stand (public while the work is on).
  const tombSiteOf = (player: PlayerId) => tombSites(after).find((g) => g.of === player);
  const siteOf = (player: PlayerId) => tombSiteOf(player)?.at ?? null;
  events.forEach((e, i) => {
    switch (e.type) {
      // Egypt's Royal Tomb work is public: every empire learns when it starts and when it pauses.
      case 'TombStarted': {
        if (team.has(e.player)) break; // Egypt's own team ordered it: the report line is enough.
        const left = Math.max(0, RULES.tombTurns - e.turns);
        publicCard(i, e.player, { tier: 'major', at: siteOf(e.player) ?? capital(e.player), goalMark: GOAL_CITY_ICON.tomb, title: 'The Royal Tomb', kicker: empireOfPlayer(after, e.player),
          effect: `${empireOfPlayer(after, e.player)} is building its Royal Tomb${tombSiteOf(e.player) ? ` in ${tombSiteOf(e.player)!.name}` : ''}: ${factionGoalVp('Egypt')} VP to whoever holds the city. ${left} ${left === 1 ? 'turn' : 'turns'} of work left.`,
          lore: 'The valley falls quiet while the tomb is cut.',
          details: ['Its cities do not build or grow while the work is on. It may pause and keep its count.'], deltas: [] });
        break;
      }
      case 'TombPaused': {
        if (team.has(e.player)) break;
        publicCard(i, e.player, { tier: 'notable', at: capital(e.player), goalMark: GOAL_CITY_ICON.tomb, title: 'The Royal Tomb', kicker: empireOfPlayer(after, e.player),
          effect: `${empireOfPlayer(after, e.player)} paused its Royal Tomb at ${e.turns} of ${RULES.tombTurns} turns: its cities build and grow again.`,
          lore: 'The work stops. The count is kept.', details: [], deltas: [] });
        break;
      }
      case 'WonderDiscovered': {
        const name = WONDER_INFO[e.wonder].name;
        const reward = e.gold ? `+${e.gold} Gold` : `+${e.science} stored Science`;
        push(i, e.owner, { tier: 'major', at: e.at, title: name,
          kicker: 'Natural wonder discovered', effect: reward,
          lore: 'A landmark enters the atlas, and the first witness carries its story home.',
          details: [`${wonderBonusSentence(e.wonder)} It scores no VP.`],
          deltas: [{ counter: e.gold ? 'gold' : 'science', amount: e.gold || e.science }],
          gains: [{ icon: e.gold ? 'yield-gold' : 'yield-science', text: reward }],
          art: wonderArt(e.wonder, 'a'),
        }); break;
      }
      // A rival's Heir is born: every other empire is told which city, and where he stands (his position is public). One card, once.
      case 'HeirBorn': {
        if (team.has(e.owner)) break; // Byzantium's own team has the Heir card.
        const player = after.players.find((p) => p.id === e.owner);
        if (!player) break;
        const vp = factionGoalVp('Byzantium');
        out.push({ id: `${after.turn}:${i}:${e.owner}`, eventIndex: i, tier: 'major', owner: e.owner, faction: player.faction, partner: false, at: e.at,
          title: 'Heir born', kicker: 'Byzantium’s Heir',
          effect: `Born in ${e.city_name} · be at War, then capture him to score ${vp} VP`, lore: 'An imperial claim walks the land. Whoever holds him scores.',
          details: ['His position is public every turn, on the map and the minimap. Move a military unit onto his tile to capture him: you must be at War with his holder.'],
          deltas: [], unit: { kind: 'HeirToTheThrone', faction: 'Byzantium' } }); break;
      }
      case 'RuinExplored': {
        const text = ruinText(e.reward, cityName, faction(e.owner));
        if (e.owner === viewer && e.reward.kind === 'Gold') {
          const buys = after.cities.filter((c) => c.owner === viewer && c.detail?.producing && c.detail.buy_cost !== null)
            .sort((a, b) => a.detail!.buy_cost! - b.detail!.buy_cost! || a.id - b.id);
          const city = buys[0];
          text.details.push(city ? `Current Buy: ${itemName(city.detail!.producing!, faction(city.owner))}, ${city.detail!.buy_cost} Gold in ${city.name}. ${after.economy.gold >= city.detail!.buy_cost! ? 'Affordable now.' : `${city.detail!.buy_cost! - after.economy.gold} more Gold needed.`}`
            : 'Choose a city’s production to see its current Buy price.');
        }
        const reward = e.reward;
        const scout = reward.kind === 'FreeScout' ? after.units.find((u) => u.id === reward.unit) : undefined;
        push(i, e.owner, { ...text, at: e.at,
          ...(scout ? { scout: { unit: scout.id, to: scout.pos } } : {}) }); break;
      }
      case 'BarbarianCampCleared': {
        // Borders growing over an empty camp pays nothing: the minor report line already covers it.
        if (e.unit === null) break;
        const place = placeName(after, e.at, Infinity);
        const deltas: Moment['deltas'] = [{ counter: 'gold', amount: e.gold }];
        push(i, e.by, { tier: 'notable', at: e.at, title: 'The raiders’ camp falls',
          effect: `+${e.gold} Gold`,
          lore: 'Their fires are cold. What they hoarded is yours now.',
          details: [place ? `Cleared ${place}.` : 'Cleared in the field.'],
          deltas, emblem: 'camp',
          gains: [{ icon: 'yield-gold', text: `+${e.gold} Gold` }] }); break;
      }
      case 'CityFounded': {
        const first = (founded.get(e.owner) ?? 0) === 0;
        founded.set(e.owner, (founded.get(e.owner) ?? 0) + 1);
        push(i, e.owner, { tier: first ? 'major' : 'notable', at: e.at, title: e.name,
          effect: first ? 'Your first city is founded' : 'A new city is founded', lore: 'Here, the inheritance takes root.', details: [], deltas: [], emblem: 'founded' }); break;
      }
      case 'FlowerWars': {
        push(i, e.owner, { tier: 'notable', at: cities.get(e.city)?.pos ?? null, kicker: 'Flower Wars',
          title: `${cityName(e.city)} gains 1 population`, effect: `Population ${e.population - 1} → ${e.population}`,
          lore: '', details: [], deltas: [{ counter: 'population', amount: 1 }], emblem: 'growth' }); break;
      }
      case 'CityGrew': {
        // A growth that unlocks something (the Settler at 2) is worth a card; any other growth is a
        // light map slip with the counter tick.
        const unlocked = e.owner === viewer ? growthUnlocks(after, e.city, e.population) : null;
        push(i, e.owner, { tier: unlocked ? 'notable' : 'minor', at: cities.get(e.city)?.pos ?? null, kicker: 'A city grows',
          title: `${cityName(e.city)} grew to ${e.population}`, effect: unlocked ? `Unlocks ${unlocked}` : `Population ${e.population}`,
          lore: unlocked ? 'More hands, and new work for them.' : '', details: [], deltas: [{ counter: 'population', amount: 1 }], emblem: 'growth' }); break;
      }
      case 'TileClaimed': {
        // Only the viewer's own claims: a minor slip over the new tile names what it adds.
        const tile = claimedTile(after, e.tile, faction(e.owner));
        push(i, e.owner, { tier: 'minor', at: e.tile, title: `${cityName(e.city)} claimed ${tile ? `a ${tile.terrain} tile` : 'a tile'}`,
          effect: tile ? `${tile.terrain}${tile.yields ? ` ${tile.yields}` : ''}` : 'New tile', lore: '', details: [], deltas: [] }); break;
      }
      case 'TechResearched': {
        // Every technology you finish earns the unlock card. A partner's tech stays a small notable
        // card: its unlocks are theirs, and their private tech list is not in this view.
        if (e.player === viewer) {
          push(i, e.player, { tier: 'major', at: capital(e.player), title: TECH_NAME[e.tech], effect: 'Research complete', kicker: 'Technology discovered',
            lore: TECH_QUOTE[e.tech], details: [], deltas: [], unlock: unlockCardFor(after, e.tech) });
          break;
        }
        const work = after.research.unlocks.some((u) => u.tech === e.tech && u.item.kind === 'Building' && isGreatWork(u.item.building));
        push(i, e.player, { tier: work ? 'major' : 'notable', at: capital(e.player), title: TECH_NAME[e.tech], effect: 'Research complete',
          lore: 'What the old kingdoms knew, its heirs may yet surpass.', details: [], deltas: [], emblem: 'tech' }); break;
      }
      case 'ResourcesRevealed': {
        // A tech reveal can produce one event per supplying city. Tell the discovery once,
        // using the viewer's fog-filtered map for the survey rather than hidden deposits.
        if (e.player !== viewer) break;
        const discovered = events.some((x) => x.type === 'TechResearched' && x.player === e.player && revealedByTech(x.tech).includes(e.resource));
        if (!discovered || events.slice(0, i).some((x) => x.type === 'ResourcesRevealed' && x.player === e.player && x.resource === e.resource)) break;
        const ownCities = after.cities.filter((c) => c.owner === e.player);
        const deposits = after.tiles.flatMap((tile, index) => tile.explored && tile.resource === e.resource ? [{ tile, hex: hexAt(index, after.width) }] : []);
        const near = deposits.filter(({ hex }) => ownCities.some((c) => distance(c.pos, hex) <= 5)).length;
        const inside = deposits.filter(({ tile }) => tile.owner === e.player).length;
        push(i, e.player, { tier: 'notable', at: deposits.find(({ hex }) => ownCities.some((c) => distance(c.pos, hex) <= 5))?.hex ?? capital(e.player),
          title: `${e.resource} revealed`, effect: `${near} deposit${near === 1 ? '' : 's'} near you, ${inside} inside your borders`,
          kicker: 'The map reveals its resources', lore: '', details: [], deltas: [], resource: e.resource });
        break;
      }
      case 'EraEntered': {
        // 1-based era. Your own era-up is a held card; a teammate's is a notable card. Rivals' entries are the era notice and the report.
        const objectives = after.eras?.find((x) => x.era === e.era);
        const name = objectives?.name ?? eraName(e.era - 1);
        if (e.player === viewer) {
          push(i, e.player, { tier: 'major', at: capital(e.player), title: `The ${name} era`, effect: 'A new era begins', kicker: e.first ? 'First into the era' : 'A new era',
            lore: '', details: [], deltas: [], eraUp: eraUpCardFor(after, e.era - 1, e.first) });
        } else {
          push(i, e.player, { tier: e.first ? 'major' : 'notable', at: capital(e.player), title: `entered the ${name} era`, effect: e.first ? 'The first empire into this era' : 'Their technologies advance',
            lore: 'The age turns for them as it will for you.', details: [], deltas: [], emblem: 'tech' });
        }
        break;
      }
      case 'AgeBegan':
        // Every player tallies up when a World Age begins (user, 2026-11-22): after the opener's own era-up, before the rest of the story.
        push(i, viewer, { tier: 'major', at: null, title: 'Score check', effect: `The ${ageName(eraName(e.era - 1))} begins`, kicker: 'A new Age',
          lore: '', details: [], deltas: [], scoreCheck: scoreCheckFor(after, e.era, e.first) });
        break;
      case 'ProductionCompleted':
        if (e.item.kind === 'Wonder' && isGreatWork(e.item.building)) {
          // A world wonder says what it scores and gives, and that nobody else can build it.
          const built = e.item.building;
          const vp = events.find((x): x is Extract<Event, { type: 'GreatWorkCompleted' }> => x.type === 'GreatWorkCompleted' && !x.again && x.player === e.owner && x.building === built)?.vp;
          const gives = wonderYieldLine(built, cityName(e.city));
          push(i, e.owner, { tier: 'legendary', at: e.item.tile, kicker: 'World wonder built', place: cityName(e.city),
            title: itemName(e.item, faction(e.owner)), effect: [vp === undefined ? null : `${signedVp(vp)} while you hold it`, gives, 'only one in the world'].filter(Boolean).join(' · '),
            lore: 'Let those who follow know what these hands have made.', details: [WONDER_ONLY_ONE],
            deltas: vp === undefined ? [] : [{ counter: 'vp', amount: vp }], emblem: 'greatwork' });
        } else if (e.item.kind === 'Building' || e.item.kind === 'Wonder') push(i, e.owner, { tier: isGreatWork(e.item.building) ? 'legendary' : 'minor', at: e.item.kind === 'Wonder' ? e.item.tile : cities.get(e.city)?.pos ?? null,
          title: itemName(e.item, faction(e.owner)), effect: `Completed in ${cityName(e.city)}`, lore: 'Let those who follow know what these hands have made.', details: [], deltas: [], emblem: 'greatwork' });
        // An aircraft never stands on the map, so its card is where the player learns what it does.
        else if (e.item.kind === 'Unit' && isAircraft(e.item.unit)) {
          const notice = readyNotice(e.item.unit, cityName(e.city));
          push(i, e.owner, { tier: 'notable', at: cities.get(e.city)?.pos ?? null, ...notice, details: [],
            deltas: [], unit: { kind: e.item.unit, faction: faction(e.owner) } });
        }
        break;
      case 'EraObjectiveScored': {
        const era = after.eras?.find((x) => x.era === e.era);
        const card = era?.objectives.find((o) => o.slot === e.slot)?.objective;
        if (era) {
          push(i, e.player, { tier: 'major', at: capital(e.player), kicker: 'Age card scored', title: `The ${ageName(era.name)}`,
            effect: '+1 VP, kept for good', lore: 'A claim made lasting in the chronicle.', deltas: [{ counter: 'vp', amount: 1 }], emblem: 'milestone',
            details: [card ? `Scored with ${cardName(e.objective)}` : e.objective, after.eras.length === 1 ? `${ageName(era.name)} scored` : `${after.eras.filter((x) => x.scored_by.some((s) => s.player === e.player)).length} of ${after.eras.length} Ages scored`] });
          break;
        }
        push(i, e.player, { tier: 'major', at: capital(e.player), kicker: 'Age card scored', title: `Age ${e.era}`,
          effect: '+1 VP, kept for good', lore: 'A claim made lasting in the chronicle.', deltas: [{ counter: 'vp', amount: 1 }], emblem: 'milestone',
          details: [e.objective] }); break;
      }
      // A Live goal regained is a normal report line, not the legendary first completion.
      // The Byzantium's held goal changes hands: capturing the Heir is a big moment for the captor, losing him a sober card
      // for the loser (the premium tier of a captured city); a plain retake by the Byzantium is a major card too.
      case 'FactionGoalCompleted': {
        const goalKind = goalCityKindOf(e.faction);
        if (goalKind) {
          const title = GOAL_CITY_TITLE[goalKind];
          const g = goalCityOf(e.faction, e.player);
          const captured = events.find((x): x is Extract<Event, { type: 'CityCaptured' }> => x.type === 'CityCaptured' && x.to === e.player);
          const name = g?.name ?? (captured ? cityName(captured.city) : undefined);
          const at = g?.at ?? (captured ? cities.get(captured.city)?.pos ?? null : capital(e.player));
          const ownFaction = faction(e.player) === e.faction;
          if (team.has(e.player)) {
            if (captured) {
              push(i, e.player, { goalMark: GOAL_CITY_ICON[goalKind], tier: 'major', at, title: ownFaction ? `The ${title} is retaken` : `${possessive(e.faction)} ${title} is taken`, kicker: name ? `Taken with ${name}` : undefined,
                effect: `+${e.vp} VP while you hold ${name ?? 'its city'}`, lore: 'The banners changed. The points follow the city.',
                details: [`A rival that takes ${name ?? 'the city'} takes the points.`], deltas: [{ counter: 'vp', amount: e.vp }], seal: e.vp });
            } else if (goalKind === 'colonia') {
              push(i, e.player, { goalMark: GOAL_CITY_ICON[goalKind], tier: 'legendary', at, title: 'Colonia Victrix', kicker: name ? `Founded at ${name}` : 'Your veterans found the Colonia Victrix',
                effect: `+${e.vp} VP while you hold it`, lore: 'The veterans were given land on the frontier they won.',
                details: ['Every empire sees where it stands. A rival that captures it takes the points.', 'If it is razed, found another.'],
                deltas: [{ counter: 'vp', amount: e.vp }], seal: e.vp });
            } else {
              push(i, e.player, { goalMark: GOAL_CITY_ICON[goalKind], tier: 'legendary', at, title: 'The Royal Tomb', kicker: name ? `It stands in ${name}` : 'The tomb stands', artSlug: GOAL_ART.Egypt,
                effect: `+${e.vp} VP while you hold it · your cities build and grow again`, lore: 'Cut into the cliff, sealed for eternity.',
                details: ['Every empire sees where it stands. A rival that captures its city takes the points.'],
                deltas: [{ counter: 'vp', amount: e.vp }], seal: e.vp });
            }
          } else if (!captured) {
            // Every empire is told when a goal city appears, and where: 2 VP to whoever holds it.
            const who = empireOfPlayer(after, e.player);
            publicCard(i, e.player, { goalMark: GOAL_CITY_ICON[goalKind], tier: 'major', at, title, kicker: who,
              effect: goalKind === 'colonia'
                ? `${who} founded its Colonia Victrix${name ? ` at ${name}` : ''}: ${e.vp} VP to whoever holds it`
                : `The Royal Tomb stands${name ? ` in ${name}` : ''}: ${e.vp} VP to whoever holds it`,
              lore: goalKind === 'colonia' ? 'Veterans settle on the frontier.' : 'Cut into the cliff, sealed for eternity.',
              details: ['Every empire sees where it stands, even through fog. Be at War with its holder and capture the city to take the points.'], deltas: [], seal: e.vp });
          }
          break;
        }
        if (e.faction === 'Byzantium') {
          const from = events.find((x): x is Extract<Event, { type: 'FactionGoalLost' }> => x.type === 'FactionGoalLost' && x.faction === 'Byzantium')?.player ?? null;
          if (faction(e.player) !== 'Byzantium') {
            push(i, e.player, { tier: 'major', at: capital(e.player), title: 'The Heir is taken', kicker: from === null ? 'The Heir is freed' : 'A rival’s heir, captured',
              effect: `+${e.vp} VP · you hold the Heir`, lore: 'An imperial claim now travels under a new banner.',
              details: ['Whoever holds the Heir scores Byzantium’s goal. A rival that captures him takes the points.'], deltas: [{ counter: 'vp', amount: e.vp }], emblem: 'captured' });
          } else if (e.again || from !== null) {
            push(i, e.player, { tier: 'major', at: capital(e.player), title: 'The Heir is retaken', effect: `+${e.vp} VP · the Heir is yours again`,
              lore: 'The imperial heir has returned.', details: [], deltas: [{ counter: 'vp', amount: e.vp }], emblem: 'captured' });
          } else {
            // His birth is announced to every empire and his position is public from now on: this is the one card for it (the HeirBorn event adds the city).
            const born = events.find((x): x is Extract<Event, { type: 'HeirBorn' }> => x.type === 'HeirBorn' && x.owner === e.player);
            push(i, e.player, { tier: 'legendary', at: born?.at ?? capital(e.player), title: OWN_HEIR_BIRTH_TITLE, kicker: born ? `Your Heir is born in ${born.city_name}` : undefined, effect: `+${e.vp} VP while you hold him`,
              lore: 'The promise has a face. Keep him.',
              details: born ? ['Every empire was told where he was born, and his position is public every turn: keep him safe.', 'A rival that captures him scores the points instead.'] : ['A rival that captures him scores the points instead.'],
              deltas: [{ counter: 'vp', amount: e.vp }], unit: { kind: 'HeirToTheThrone', faction: 'Byzantium' } });
          }
          break;
        }
        if (e.faction === 'Mongols') {
          const from = events.find((x): x is Extract<Event, { type: 'FactionGoalLost' }> => x.type === 'FactionGoalLost' && x.faction === 'Mongols')?.player ?? null;
          const hall = after.courier_halls?.find((h) => h.holder === e.player);
          const at = (hall && cities.get(hall.city)?.pos) ?? capital(e.player);
          const named = hall ? cities.get(hall.city)?.name : undefined;
          const nameOf = (id: number) => empireOfPlayer(after, id);
          if (from !== null && from !== e.player) {
            push(i, e.player, { tier: 'major', at, title: 'The Yam Relay Station is taken',
              effect: `+${e.vp} VP · you hold the Yam Relay Station${named ? ` in ${named}` : ''}`, lore: 'The road follows the banner over the city. So do the points.',
              details: [`Every empire may cross all of your land at Peace now: guard it, or accept the traffic.`, `${nameOf(from)} lost the points and the road. A rival that takes the city takes both.`],
              deltas: [{ counter: 'vp', amount: e.vp }], emblem: 'hall', building: 'YamRelayStation' });
          } else if (e.again || from !== null) {
            push(i, e.player, { tier: 'major', at, title: 'The Yam Relay Station stands again',
              effect: `+${e.vp} VP · every empire crosses your land at Peace`, lore: 'Rebuilt from the foundations. The road is open.',
              details: ['Keep its city: a rival that captures it takes the points and the road.'], deltas: [{ counter: 'vp', amount: e.vp }], emblem: 'hall', building: 'YamRelayStation' });
          } else {
            push(i, e.player, { tier: 'legendary', at, title: 'The Yam Network', kicker: named ? `Roads now link every city to ${named}` : 'The Yam Relay Station is built',
              effect: `+${e.vp} VP while you hold it`, lore: 'The couriers asked for no gate, and kept the road open for everyone.',
              details: ['Roads now join every city on the map, enemy land included: a road tile costs 1 move, and a unit that starts its turn on a road has +1 move. Everyone can use them.',
                'Every empire may now cross all of your land at Peace, armies included: that is the station’s price.', 'A rival that captures its city takes the points and the road. Razing it destroys the station; it can be built again.'],
              deltas: [{ counter: 'vp', amount: e.vp }], emblem: 'hall', building: 'YamRelayStation' });
          }
          break;
        }
        if (e.again) break;
        if (e.faction === 'Sparta') {
          push(i, e.player, { tier: 'legendary', at: capital(e.player), title: 'The veterans are honoured', effect: `Honour the Veterans complete · +${e.vp} VP, kept for good`,
            lore: 'Those who earned their rest were given it. The living inherit what they won.', details: ['Three veterans retired in the Agoge’s keeping.'], deltas: [{ counter: 'vp', amount: e.vp }], building: 'Agoge' });
          break;
        }
        if (e.faction === 'India') {
          push(i, e.player, { tier: 'legendary', at: capital(e.player), title: 'The Imperial Diwan',
            effect: `+${e.vp} VP while you hold its city`, lore: 'The regional accounts now reach the imperial treasury.',
            details: ['A rival that captures the Diwan’s city takes its points.'],
            deltas: [{ counter: 'vp', amount: e.vp }], building: 'ImperialDiwan' });
          break;
        }
        if (e.faction === 'Celts') {
          push(i, e.player, { tier: 'legendary', at: capital(e.player), title: 'The warbands gather', effect: `+${e.vp} VP while every city has a camp`,
            lore: 'Warbands gather near every city; rivals may drive them away.', details: ['A rival that clears a camp takes the points away until you call again.'], deltas: [{ counter: 'vp', amount: e.vp }], artSlug: GOAL_ART.Celts });
          break;
        }
        push(i, e.player, { tier: 'legendary', at: capital(e.player), title: 'An inheritance fulfilled', effect: `Faction goal complete · +${e.vp} VP`,
          lore: 'The promise survived its makers. You have kept it.', details: [], deltas: [{ counter: 'vp', amount: e.vp }], building: GOAL_BUILDING[e.faction], artSlug: GOAL_ART[e.faction] }); break;
      }
      case 'VeteranRetired':
        // The last retirement is the goal's own legendary card (above): only the ones before it get a card.
        if (e.retired < e.of) push(i, e.player, { tier: 'major', at: cities.get(e.city)?.pos ?? null, title: 'A veteran retires with honour', kicker: 'Honour the Veterans',
          effect: `${e.retired} of ${e.of} retired`, lore: 'He earned his rest. His service is recorded.', details: [`${e.of - e.retired} more to retire for +2 VP, kept for good.`], deltas: [], emblem: 'milestone' });
        break;
      case 'FactionGoalLost': {
        const lostKind = goalCityKindOf(e.faction);
        if (lostKind) {
          if (!team.has(e.player)) break;
          const title = GOAL_CITY_TITLE[lostKind];
          const to = events.find((x): x is Extract<Event, { type: 'FactionGoalCompleted' }> => x.type === 'FactionGoalCompleted' && x.faction === e.faction)?.player ?? null;
          const cap = events.find((x): x is Extract<Event, { type: 'CityCaptured' }> => x.type === 'CityCaptured' && x.from === e.player);
          const razed = events.find((x): x is Extract<Event, { type: 'CityRazed' }> => x.type === 'CityRazed' && x.from === e.player);
          const city = cap ? cityName(cap.city) : razed ? cityName(razed.city) : null;
          const again = lostKind === 'colonia' ? 'Rome may found another.' : 'Egypt may build another, from the start.';
          push(i, e.player, { goalMark: GOAL_CITY_ICON[lostKind], tier: 'major', at: (cap && cities.get(cap.city)?.pos) ?? razed?.at ?? capital(e.player), kicker: 'A loss',
            title: to !== null ? `The ${title} is taken` : razed ? `The ${title} is razed` : `The ${title} is lost`,
            effect: `−${e.vp} VP · ${to !== null ? `${empireOfPlayer(after, to)} took ${city ?? 'its city'}` : razed ? `${city ?? 'its city'} was razed` : 'nobody holds it now'}`,
            lore: to !== null ? 'The points follow the banner over the city.' : 'Nothing is left to retake.',
            details: to !== null ? [`Retake ${city ?? 'the city'} and the points return.`] : razed ? [again] : ['Nobody scores it until its city is retaken.'],
            deltas: [{ counter: 'vp', amount: -e.vp }], loss: true });
          break;
        }
        if (e.faction === 'Celts') {
          // The clearing is in the same batch; the camp belongs to your nearest city. Fog may hide who did it.
          const cleared = events.find((x): x is Extract<Event, { type: 'BarbarianCampCleared' }> => x.type === 'BarbarianCampCleared' && !team.has(x.by));
          const near = cleared && [...cities.values()].filter((c) => c.owner === e.player).sort((a, b) => distance(a.pos, cleared.at) - distance(b.pos, cleared.at) || a.id - b.id)[0];
          const clause = cleared ? `${empireOfPlayer(after, cleared.by)} cleared the camp${near ? ` near ${near.name}` : ''}` : 'A rival cleared one of your camps';
          push(i, e.player, { tier: 'major', at: cleared?.at ?? capital(e.player), title: 'A warband camp is cleared', kicker: 'A loss', effect: `−${e.vp} VP · ${clause}`,
            lore: 'What you raised, another hand has ended.', details: ['Raise the Warbands again to raise a camp in every city.'], deltas: [{ counter: 'vp', amount: -e.vp }], loss: true, emblem: 'camp' });
          break;
        }
        if (e.faction === 'Mongols') {
          const to = events.find((x): x is Extract<Event, { type: 'FactionGoalCompleted' }> => x.type === 'FactionGoalCompleted' && x.faction === 'Mongols')?.player ?? null;
          const razed = events.some((x) => x.type === 'CityRazed' && x.from === e.player);
          const cap = events.find((x): x is Extract<Event, { type: 'CityCaptured' }> => x.type === 'CityCaptured' && x.from === e.player);
          const lostCity = cap ? cityName(cap.city) : null;
          const clause = to !== null ? `${empireOfPlayer(after, to)} took ${lostCity ?? 'its city'}, and the station with it` : razed ? 'its city was sacked, and the station with it' : 'the station is gone';
          push(i, e.player, { tier: 'major', at: (cap && cities.get(cap.city)?.pos) ?? capital(e.player), title: to !== null ? 'The Yam Relay Station is taken from you' : razed ? 'The Yam Relay Station is sacked' : 'The Yam Relay Station is lost', kicker: 'A loss',
            effect: `−${e.vp} VP · ${clause}`, lore: to !== null ? 'The road follows the banner over the city.' : 'The road is closed again.',
            details: to !== null ? ['The open road went with the city: retake it and the points return.'] : razed ? ['The station is destroyed with its city. It can be built again.'] : ['Nobody scores the Yam Network until its city is retaken.'],
            deltas: [{ counter: 'vp', amount: -e.vp }], loss: true, emblem: 'hall', building: 'YamRelayStation' }); break;
        }
        if (e.faction !== 'Byzantium') break;
        const to = events.find((x): x is Extract<Event, { type: 'FactionGoalCompleted' }> => x.type === 'FactionGoalCompleted' && x.faction === 'Byzantium')?.player ?? null;
        push(i, e.player, { tier: 'major', at: capital(e.player), title: to === null ? 'The Heir is lost' : 'The Heir is taken from you', kicker: 'A loss',
          effect: `−${e.vp} VP · ${to === null ? 'the Heir is lost' : `the Heir is held by ${empireOfPlayer(after, to)}`}`, lore: 'What was yours walks under another banner.',
          details: to === null ? ['Nobody scores Byzantium’s goal until the Heir is held again.'] : ['A military unit entering his tile retakes him, and the points return with him.'],
          deltas: [{ counter: 'vp', amount: -e.vp }], loss: true, emblem: 'captured' }); break;
      }
      case 'CityCaptured': {
        // The held capital point has its own Tier A card for every viewer.
        if (events.some((x) => x.type === 'CapitalTaken' && x.capital === e.city) && team.has(e.to)) break;
        // The Yam Relay Station's city is told by the Station's own moment (the road and the points went with it).
        if (after.courier_halls?.some((h) => h.city === e.city)) break;
        // A goal city's capture is told by the goal's own card (the points went with it).
        if (goalCitiesAt(after, e.city).length && events.some((x) => x.type === 'FactionGoalCompleted' && x.player === e.to && goalCityKindOf(x.faction))) break;
        // A capture is a major moment for the taker; a loss is a sober card of the same weight.
        const city = after.cities.find((c) => c.id === e.city) ?? cities.get(e.city);
        const nameOf = (id: number) => isBarbarian(id) ? 'Barbarians' : empireOfPlayer(after, id);
        // A city that holds a world wonder moves its points with it: one card names the wonder and the VP that moved.
        const taken = events.flatMap((x) => x.type === 'GreatWorkCompleted' && x.again && x.player === e.to ? [x] : []);
        const lost = events.flatMap((x) => x.type === 'GreatWorkLost' && x.player === e.from ? [x] : []);
        const here = (b: string) => after.tiles.some((t) => t.landmark?.city === e.city && t.landmark.building === b);
        const works = (team.has(e.to) && !team.has(e.from) ? taken : team.has(e.from) && !team.has(e.to) ? lost : []).filter((w) => here(w.building));
        if (works.length) {
          const gain = team.has(e.to);
          const names = works.map((w) => BUILDING_NAME[w.building]);
          const vp = works.reduce((sum, w) => sum + w.vp, 0);
          const who = gain ? e.to : e.from;
          push(i, who, { tier: gain ? 'legendary' : 'major', at: city?.pos ?? null, title: names.join(' and '), kicker: gain ? 'World wonder taken' : 'World wonder lost', place: cityName(e.city),
            effect: gain ? `${signedVp(vp)} · captured from ${nameOf(e.from)}` : `${signedVp(-vp)} · taken by ${nameOf(e.to)}`,
            lore: gain ? 'The banners changed. The wonder remains.' : 'What was built there answers to another now.',
            details: [WONDER_ONLY_ONE, gain ? `Hold ${cityName(e.city)} to keep the points: a rival that retakes it takes them back.` : `Retake ${cityName(e.city)} and the points return with it.`],
            deltas: [{ counter: 'vp', amount: gain ? vp : -vp }], loss: !gain, emblem: 'greatwork' });
          break;
        }
        if (team.has(e.to) && !team.has(e.from)) {
          const d = city?.detail;
          push(i, e.to, { tier: 'major', at: city?.pos ?? null, title: cityName(e.city), effect: `Captured from ${nameOf(e.from)}`, kicker: 'A city taken',
            lore: 'The banners changed. The city remains.', deltas: [], emblem: 'captured',
            details: [
              ...(city ? [`Population ${city.population}.`] : []),
              ...(d?.buildings.length ? [`Buildings kept: ${d.buildings.map((b) => itemName({ kind: 'Building', building: b }, faction(e.to))).join(', ')}.`] : []),
              ...(d?.resistance ? [`In resistance for ${d.resistance} turns: it produces nothing until resistance ends.`] : []),
            ] });
        } else if (team.has(e.from) && !team.has(e.to)) {
          push(i, e.from, { tier: 'major', at: city?.pos ?? null, title: cityName(e.city), effect: `Taken by ${nameOf(e.to)}`, kicker: 'A city is lost',
            lore: 'What was built there answers to another now.', details: ['Retake it to restore its yields and territory.'], deltas: [], loss: true, emblem: 'captured' });
        }
        break;
      }
      // A rival finished first the wonder one of your cities was building: never just a report line (user, 2026-11-07, Thun T92).
      case 'WonderRaceLost': {
        // One card per lost wonder: the city that was building it, else the first that had it queued or banked.
        const same = events.filter((x): x is Extract<Event, { type: 'WonderRaceLost' }> => x.type === 'WonderRaceLost' && x.owner === e.owner && x.building === e.building);
        if ((same.find((x) => x.active) ?? same[0]) !== e) break;
        const rival = empireOfPlayer(after, e.winner);
        const detail = after.cities.find((c) => c.id === e.city)?.detail;
        const blocked = detail?.wonder_lost ? blockedQueueHead(detail, faction(e.owner)) : '';
        push(i, e.owner, { tier: 'major', at: cities.get(e.city)?.pos ?? null, title: BUILDING_NAME[e.building], kicker: 'Wonder race lost', place: cityName(e.city), choose: detail?.wonder_lost ? e.city : undefined,
          effect: `${rival} completed it first`,
          lore: 'Another empire raised it before you.',
          details: [wonderLostLine(rival, e.building, cityName(e.city), e.production, e.gold, e.active ?? true),
            ...(detail?.wonder_lost ? [`Choose what ${cityName(e.city)} builds next.${blocked ? ` ${blocked}` : ''}`] : [])],
          deltas: [], loss: true, emblem: 'greatwork' });
        break;
      }
      // A wonder lost or won with no capture beside it (a raze, a sack): its own card. A capture's card above already names it.
      case 'GreatWorkCompleted':
      case 'GreatWorkLost': {
        if (e.type === 'GreatWorkCompleted' && !e.again) break;
        const gain = e.type === 'GreatWorkCompleted';
        // A capture whose city holds this wonder's tile has already told it (above); otherwise the wonder tells itself.
        const sitedAt = after.tiles.find((t) => t.landmark?.building === e.building)?.landmark?.city;
        if (events.some((x) => x.type === 'CityCaptured' && x.city === sitedAt && (gain ? x.to === e.player : x.from === e.player))) break;
        const razed = events.find((x): x is Extract<Event, { type: 'CityRazed' }> => x.type === 'CityRazed' && x.from === e.player);
        // The city the wonder stands in, as the built and race-lost cards name it: its tile, else the city that changed hands with it.
        const moved = events.find((x): x is Extract<Event, { type: 'CityCaptured' }> => x.type === 'CityCaptured' && (gain ? x.to === e.player : x.from === e.player));
        const site = sitedAt !== undefined && cities.has(sitedAt) ? sitedAt : moved && cities.has(moved.city) ? moved.city : undefined;
        push(i, e.player, { tier: gain ? 'legendary' : 'major', at: razed ? cities.get(razed.city)?.pos ?? razed.at : capital(e.player), title: BUILDING_NAME[e.building],
          kicker: gain ? 'World wonder taken' : razed ? 'World wonder destroyed' : 'World wonder lost', place: razed ? cityName(razed.city) : site !== undefined ? cityName(site) : undefined,
          effect: `${signedVp(gain ? e.vp : -e.vp)}${razed && !gain ? ' · destroyed with its city' : ''}`,
          lore: gain ? 'The wonder is yours again.' : 'What was built there is gone from your hands.',
          details: [WONDER_ONLY_ONE, ...(razed && !gain ? ['It can never be built again.'] : [])],
          deltas: [{ counter: 'vp', amount: gain ? e.vp : -e.vp }], loss: !gain, emblem: 'greatwork' });
        break;
      }
      case 'CityRazed': {
        if (events.some((x) => x.type === 'CapitalTaken' && x.capital === e.city) && team.has(e.by)) break;
        // The Station's own loss card names its sacked city. Do not add a second card for that event.
        const hallLoss = events.some((x) => x.type === 'FactionGoalLost' && x.faction === 'Mongols' && x.player === e.from);
        if (after.courier_halls?.some((h) => h.city === e.city) || hallLoss) break;
        const nameOf = (id: number) => empireOfPlayer(after, id);
        const at = cities.get(e.city)?.pos ?? e.at;
        if (team.has(e.by) && !team.has(e.from)) {
          push(i, e.by, { tier: 'major', at, title: cityName(e.city), kicker: e.ordered ? 'A city razed' : RAIDER_SACK_KICKER,
            effect: e.ordered ? 'Razed by order' : `Sacked from ${nameOf(e.from)}`, lore: e.ordered ? 'Its tiles are released.' : 'The walls fell, and the spoils passed to the victors.', emblem: 'captured', deltas: [],
            details: [
              ...(e.sack_gold === undefined ? [] : [e.ordered ? `Gold gained: ${e.sack_gold}.` : `Gold taken: ${e.sack_gold}.`]),
              ...(e.camp === undefined ? [] : ['A warband camp was raised among the ruins.']),
            ] });
        } else if (team.has(e.from) && !team.has(e.by)) {
          push(i, e.from, { tier: 'major', at, title: cityName(e.city), kicker: e.ordered ? 'A former city razed' : 'A city is lost',
            effect: `${e.ordered ? 'Razed' : 'Sacked'} by ${nameOf(e.by)}`, lore: 'Nothing is left to retake.',
            details: ['The city is destroyed and cannot be retaken.'], deltas: [], loss: !e.ordered, emblem: 'captured' });
        }
        break;
      }
      case 'CapitalTaken': {
        const ours = team.has(e.captor);
        const at = cities.get(e.capital)?.pos ?? null;
        const who = empireOfPlayer(after, e.captor);
        const works = events.filter((x): x is Extract<Event, { type: 'GreatWorkCompleted' }> =>
          x.type === 'GreatWorkCompleted' && x.again && x.player === e.captor &&
          after.tiles.some((t) => t.landmark?.city === e.capital && t.landmark.building === x.building));
        const workVp = works.reduce((total, x) => total + x.vp, 0);
        const victim = events.find((x): x is Extract<Event, { type: 'CityCaptured' }> => x.type === 'CityCaptured' && x.city === e.capital)?.from
          ?? events.find((x): x is Extract<Event, { type: 'CityRazed' }> => x.type === 'CityRazed' && x.city === e.capital)?.from
          ?? cities.get(e.capital)?.original_capital_of ?? null;
        const from = victim !== null ? empireOfPlayer(after, victim) : null;
        const card = { tier: ours ? 'legendary' as const : 'major' as const, at, title: e.name, kicker: from ? `Original capital taken from ${from}` : 'Original capital taken',
          effect: `${ours ? '+1 VP while you hold it' : `${who} scores 1 VP while holding it`}${workVp ? ` · +${workVp} held VP from ${works.map((x) => BUILDING_NAME[x.building]).join(' and ')}` : ''}`,
          lore: ours ? `${e.name}’s gates are yours.` : `${e.name} has fallen to ${who}.`,
          details: ['The point is lost if anyone takes the city or it is razed.',
            ...(workVp ? [`${works.map((x) => BUILDING_NAME[x.building]).join(' and ')} scores ${workVp} VP only while held.`] : [])],
          deltas: ours ? [{ counter: 'vp' as const, amount: 1 + workVp }] : [], emblem: 'captured' as const,
          ...(victim !== null && faction(victim) ? { frame: faction(victim)! } : {}), seal: 1, ...(ours ? {} : { loss: true }) };
        if (ours) push(i, e.captor, card);
        else {
          const player = after.players.find((p) => p.id === e.captor);
          if (player) out.push({ ...card, id: `${after.turn}:${i}:${e.captor}`, eventIndex: i,
            owner: e.captor, faction: player.faction, partner: false });
        }
        break;
      }
      // Victory already owns the Game Over takeover. No second ceremony over its controls.
    }
  });
  return out.sort((a, b) => TIER_ORDER[b.tier] - TIER_ORDER[a.tier] || a.eventIndex - b.eventIndex);
}

export function planMoments(moments: readonly Moment[]): { shown: Moment[]; collapsed: Moment[] } {
  // The score check is never collapsed: it plays right after your era-up, else first.
  const check = moments.find((m) => m.scoreCheck);
  const ordered = [...moments].filter((m) => m !== check).sort((a, b) => TIER_ORDER[b.tier] - TIER_ORDER[a.tier] || a.eventIndex - b.eventIndex);
  if (!check) return { shown: ordered.slice(0, 3), collapsed: ordered.slice(3).filter((m) => m.tier !== 'minor') };
  const others = ordered.slice(0, 2);
  const at = others.findIndex((m) => m.eraUp) + 1;
  return { shown: [...others.slice(0, at), check, ...others.slice(at)], collapsed: ordered.slice(2).filter((m) => m.tier !== 'minor') };
}
