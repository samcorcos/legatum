// Every player-facing string the game prints from data, as labelled text blocks: the corpus both text guards
// read (the artifact guard and the copy lint, ui/textArtifacts.test.tsx). A new surface that prints prose
// from data joins this list in the same change.
import { renderToStaticMarkup } from 'react-dom/server';
import {
  BUILDING_KINDS, CLASS_NOTE, DECADENCE_FOOD_RULE, DECADENCE_RULE, FACTIONS, FACTION_GOAL, FACTION_INFO,
  IMPROVEMENT_KINDS, GREAT_WORK_DESC, GREAT_WORK_RULE, POWER_RULE, SWARM_GOAL,
  TECHS, TECH_NAME, TECH_STATS, THIN_LEVIES, UNIT_KINDS, availableToFaction, decadenceCurveText,
  factionDescription, factionExcludedBuilding, factionGoalTerms, factionUnavailableReason, greatWorkRules, itemCardText,
  itemDescription, itemName, itemSummary, techCardEffectsFor, companyRule,
} from '../data';
import { unlockSentence } from '../celebrate/unlockCard';
import { FACTION_SHEETS } from '../factionSheets';
import { ALLIANCE_SHARES, TREATY_DEFINITION, allianceTerms } from '../treaties';
import { TECH_QUOTE } from '../techQuotes';
import { notMetText, offerLabel, treatyLabel } from '../diplomacy';
import { COMPANY_UNIT_RULE, contractLine, rosterLine } from '../company';
import { describeOrder } from '../orders';
import { CITY_HEAL_RULE, CITY_STRIKE_RULE, contextFromViews, formatEvents } from '../format';
import { FixtureClient } from './fixtureEngine';
import { htmlBlocks } from './textArtifacts';
import { hallCardText } from '../ui/goalText';
import { FactionSheetView } from '../ui/FactionPicker';
import { GUIDE_SECTIONS, GuideContent } from '../ui/GuideContent';
import { LEVY_CHIP_LABEL } from '../ui/Levy';
import { tileDefenceText } from '../ui/TileCard';
import { siegeCityHint } from '../ui/UnitPanel';
import { POSTGAME_ARMY_TEXT } from '../ui/PostGameReport';
import { TILE_RELEASE_RULE } from '../ui/cityTerritoryText';
import { tileFocusNoteText } from '../ui/bits';
import { distance, hexAt } from '../hex';
import type { Event, Faction, PlayerView, ProductionItem, UnitView } from '../types';

export interface TextGroup { label: string; blocks: string[] }

/** Dynamic UI labels and hints that the data and Guide groups cannot reach. */
export function interfaceText(): TextGroup[] {
  return [
    { label: 'city territory', blocks: [TILE_RELEASE_RULE] },
    { label: 'city tile focus', blocks: [tileFocusNoteText('Balanced', 'no active build')] },
    { label: 'Levy chip', blocks: [LEVY_CHIP_LABEL] },
    { label: 'Hills tile card', blocks: [tileDefenceText(3)] },
    { label: 'unit siege hint', blocks: [siegeCityHint(4)] },
    { label: 'post-game army column', blocks: [POSTGAME_ARMY_TEXT] },
  ];
}

/** Core-produced refusal and yield labels, sampled from an editable core fixture. */
export async function coreInterfaceText(): Promise<TextGroup[]> {
  const client = new FixtureClient();
  await client.newGame(await client.standardSetup('908104', 'India', 'Sparta', 'Ada', 'Bo'));
  client.endTurn({});
  const state = client.debugState();
  const capital = state.cities.find((c) => c.owner === 0)!;
  capital.buildings.push('Granary');
  const refusal = client.validate(0, [{ type: 'SetProduction', city: capital.id, item: { kind: 'Building', building: 'Granary' } }])
    .map((error) => error.message);
  const farIndex = state.terrain.findIndex((terrain, i) => (terrain === 'Grassland' || terrain === 'Plains' || terrain === 'Forest' || terrain === 'Hills')
    && state.owner[i] === -1 && distance(hexAt(i, state.width), capital.pos) > 4);
  if (farIndex < 0) throw new Error('No distant land tile for the corruption text fixture');
  const far = { ...structuredClone(capital), id: state.nextCityId++, name: 'Far City', pos: hexAt(farIndex, state.width),
    buildings: [], isCapital: false, originalCapital: false, grownTiles: [], tilesClaimed: 0 };
  state.cities.push(far);
  state.owner[farIndex] = 0;
  state.tileCity[farIndex] = far.id;
  const labels = client.view(0).cities.find((city) => city.id === far.id)?.detail?.yields.sources
    .map((source) => source.label).filter((label) => label.startsWith('Corruption:')) ?? [];
  return [{ label: 'core building refusal', blocks: refusal }, { label: 'core city yield breakdown', blocks: labels }];
}

const ITEMS: ProductionItem[] = [
  ...UNIT_KINDS.map((unit): ProductionItem => ({ kind: 'Unit', unit })),
  ...BUILDING_KINDS.map((building): ProductionItem => ({ kind: 'Building', building })),
  ...IMPROVEMENT_KINDS.map((improvement): ProductionItem => ({ kind: 'Improvement', improvement, tile: { q: 0, r: 0 } })),
  { kind: 'Project', project: 'Gold' },
  { kind: 'Project', project: 'Science' },
];
const FACTION_VIEWS: (Faction | undefined)[] = [undefined, ...FACTIONS];

/** Every unit, building, improvement and project in every faction: names, descriptions, card lines, unlock sentences, refusals. */
export function itemText(): TextGroup[] {
  const out: TextGroup[] = [];
  for (const item of ITEMS) for (const f of FACTION_VIEWS) {
    const where = `${itemName(item, f)} (${f ?? 'no faction'})`;
    out.push({ label: `${where} description`, blocks: [itemDescription(item, f), itemCardText(item, f), itemSummary(item, f)] });
    if (f && availableToFaction(f, item)) out.push({ label: `${where} unlock sentence`, blocks: [unlockSentence(item, f)] });
    const why = f && [factionUnavailableReason(item, f), item.kind === 'Building' ? factionExcludedBuilding(f, item.building) : null].filter((s): s is string => !!s);
    if (why && why.length) out.push({ label: `${where} refusal`, blocks: why });
  }
  return out;
}

/** Every technology blurb, quote and per-faction effect. */
export function techText(): TextGroup[] {
  const out: TextGroup[] = [];
  for (const t of TECHS) {
    out.push({ label: `tech ${TECH_NAME[t]}`, blocks: [TECH_STATS[t].blurb, TECH_QUOTE[t]] });
    for (const f of FACTIONS) out.push({ label: `tech ${TECH_NAME[t]} for ${f}`, blocks: techCardEffectsFor(f, t) });
  }
  return out;
}

/** Every faction's info, sheet, goal and rule lines. */
export function factionText(): TextGroup[] {
  const out: TextGroup[] = [];
  for (const f of FACTIONS) {
    const info = FACTION_INFO[f];
    const sheet = FACTION_SHEETS[f];
    out.push({ label: `${f} info`, blocks: [info.name, info.motto, info.rule, info.ability, info.unique, info.goal, info.weakness, factionDescription(f), factionGoalTerms(f)] });
    out.push({ label: `${f} sheet`, blocks: [sheet.identity, sheet.lore, sheet.lose, sheet.counteredBy, ...sheet.pairs.flatMap((p) => [p.strength, p.price ?? ''])].filter((line): line is string => !!line) });
  }
  for (const [name, goal] of [...Object.entries(FACTION_GOAL), ['Swarm', SWARM_GOAL]] as const) {
    out.push({ label: `goal ${name}`, blocks: [goal.name, goal.rule(), goal.detail(), goal.liveNote ?? '', goal.action ?? '', goal.unitAction ?? '', goal.heldItem ?? ''].filter(Boolean) });
  }
  out.push({ label: 'faction rules', blocks: [THIN_LEVIES, hallCardText(), ...Object.values(CLASS_NOTE)] });
  return out;
}

/** Every faction sheet as rendered (all eight peoples: piece card, goals, hall). */
export function renderedSheetText(): TextGroup[] {
  return FACTIONS.map((f) => ({ label: `sheet ${f}`, blocks: htmlBlocks(renderToStaticMarkup(<FactionSheetView faction={f} sheetId={`sheet-${f}`} />)) }));
}

/** The Guide, section by section, as rendered. */
export function guideText(): TextGroup[] {
  return GUIDE_SECTIONS.map((s) => ({ label: `guide ${s.id}`, blocks: htmlBlocks(renderToStaticMarkup(<GuideContent section={s.id} />)) }));
}

/** Decadence, eras, Great Works and the combat rules shown in tooltips. */
export function victoryText(): TextGroup[] {
  return [
    { label: 'victory rules', blocks: [POWER_RULE, DECADENCE_RULE, DECADENCE_FOOD_RULE, decadenceCurveText(), GREAT_WORK_DESC, GREAT_WORK_RULE] },
    ...[undefined, ...FACTIONS].map((f) => ({ label: `great work rules ${f ?? ''}`, blocks: [greatWorkRules(f)] })),
    { label: 'combat rules', blocks: [CITY_STRIKE_RULE, CITY_HEAL_RULE] },
  ];
}

/** Mercenary Companies: the rule, a unit's contract line, the roster, the order and every report line, for the buyer, the seller and a bystander. */
export async function companyText(): Promise<TextGroup[]> {
  const client = new FixtureClient();
  await client.newGame(await client.standardSetup('908102', 'Byzantium', 'Sparta', 'AI 1', 'AI 2'));
  client.endTurn({});
  const base = client.view(0);
  const unit = { ...base.units.find((u) => u.owner === 0)!, contract_until: base.turn + 7, origin: 'Sparta' } as UnitView;
  const view: PlayerView = { ...base, units: [unit, ...base.units.filter((u) => u.id !== unit.id)], players: base.players.map((p) => (p.id === 0 ? { ...p, companies: 1 } : p)), cities: base.cities.map((c, i) => (i === 0 ? { ...c, name: 'Constantinople' } : c)),
    company_offers: [{ seller: 1, unit: 'Spearman', price: 80 }] };
  const city = view.cities[0];
  const events: Event[] = [
    { type: 'HeirBorn', city: city.id, owner: 0, city_name: 'Constantinople', at: unit.pos },
    { type: 'HeirBorn', city: city.id, owner: 2, city_name: 'Pella', at: unit.pos },
    { type: 'CompanyHired', city: city.id, owner: 0, seller: 1, unit: 'Spearman', gold: 80, until: base.turn + 30 },
    { type: 'CompanyHired', city: city.id, owner: 2, seller: 0, unit: 'Spearman', gold: 80, until: base.turn + 30 },
    { type: 'CompanyDeparted', unit: 'Spearman', owner: 0, origin: 'Sparta', pos: unit.pos },
    { type: 'CompanyDeparted', unit: 'Spearman', owner: 2, origin: 'Sparta', pos: unit.pos },
    { type: 'CompanyTurned', unit: 'Spearman', owner: 0, origin: 'Sparta', pos: unit.pos },
    { type: 'CompanyTurned', unit: 'Spearman', owner: 2, origin: 'Sparta', pos: unit.pos },
  ];
  const lines = formatEvents(events, contextFromViews(base, view)).flatMap((l) => [l.text, ...(l.details ?? [])]);
  return [{
    label: 'mercenary companies', blocks: [
      companyRule(), COMPANY_UNIT_RULE, contractLine(unit, view), contractLine({ ...unit, contract_until: base.turn }, view), rosterLine(view, 0, 1),
      describeOrder({ type: 'HireCompany', city: city.id, unit: 'Spearman', seller: 1 }, view),
      ...lines,
    ],
  }];
}

/** Diplomacy and treaty text. */
export function diplomacyText(): TextGroup[] {
  const terms = allianceTerms(3);
  return [{
    label: 'treaties', blocks: [
      ...Object.values(TREATY_DEFINITION), ALLIANCE_SHARES, ...Object.values(terms).map(String),
      treatyLabel(null), treatyLabel('Alliance'), offerLabel({ kind: 'Peace' }), offerLabel({ kind: 'Alliance' }),
      notMetText("You haven't met this empire yet: send a Scout.", 'The Celts'),
    ],
  }];
}

/** The turn report of a real WASM game: every line player 0 would read over its first `turns` turns, all seats played by the AI. */
export async function reportText(turns = 60): Promise<TextGroup[]> {
  const client = new FixtureClient();
  await client.newGame(await client.standardSetup('908103', 'Sparta', 'Switzerland', 'Ada', 'Bo'));
  for (const p of client.debugState().players) {
    p.isHuman = false;
    client.debugState().setup.players[p.id].is_human = false;
  }
  const lines = new Set<string>();
  for (let t = 0; t < turns && !client.debugState().finished; t++) {
    const before = client.view(0);
    const result = client.endTurn({});
    const after = client.view(0);
    for (const l of formatEvents(result.events, contextFromViews(before, after))) {
      lines.add(l.text);
      if (l.details) for (const d of l.details) lines.add(d);
    }
  }
  return [{ label: 'turn report', blocks: [...lines] }];
}
