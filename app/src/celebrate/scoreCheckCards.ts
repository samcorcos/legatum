// The Age score check (user, 2026-11-22: "a scoring sheet you tally up at regular intervals"): the takeover every player
// sees when a World Age begins. The score track shows who is ahead; your own sheet shows each way to score as boxes,
// filled for what you have and empty for what is still open, with one line on how to win the next one.
// Public facts and your own view only. Everything here reads the view; nothing here is a rule.
import type { BuildingKind, Faction, PlayerId, PlayerView } from '../types';
import { BUILDING_NAME, BUILDING_STATS, ERA_COUNT, FACTION_GOAL, GREAT_WORK_ICON, GREAT_WORKS, TECH_NAME, buildingIconName, eraName, factionGoalVp, isGreatWork } from '../data';
import { buildVpModel } from '../vpModel';
import { cardName } from '../ageCards';
import { eraObjectivesFor } from '../eraObjectives';
import { empireOfPlayer } from '../empires';
import { distance } from '../hex';
import { roman } from '../brand/roman';
import { playerColor } from '../theme';

/** How a box on your sheet is filled: kept for good, held (lost with its city or item), new this Age, open, or an Age still to come. */
export type BoxState = 'kept' | 'held' | 'new' | 'open' | 'later';
/**
 * One box on your sheet. It draws the thing that scores (premium pass, user 2026-11-22: "the game needs to feel premium"):
 * an Age's numeral (`mark`), a capital's star (`star`), a wonder's or goal's icon (`icon`). A box worth more than 1 VP
 * spans that many cells (`span`), so the boxes still add up to the VP.
 */
export interface Box { state: BoxState; mark?: string; icon?: string; star?: boolean; span?: number; title: string }

export interface SheetLine {
  key: 'ages' | 'capitals' | 'wonders' | 'goal';
  name: string;
  terms: string;
  boxes: Box[];
  /** The next point, in one bold line. */
  how: string;
  detail: string;
  /** VP you can still score this way (the boxes still empty). */
  open: number;
  /** This line holds the new Age's card. */
  fresh?: boolean;
}

export interface TrackPiece { id: PlayerId; faction: Faction; color: string; name: string; isMine: boolean }
export interface TrackSpace { vp: number; pieces: TrackPiece[] }

export interface ScoreCheckCard {
  /** The Age, 1-based as the core numbers it. */
  era: number;
  name: string;
  numeral: string;
  turn: number;
  /** "You reached the Renaissance era first." */
  opener: string;
  target: number;
  /** Spaces 0..target; a team past its target stands on the last space. */
  track: TrackSpace[];
  /** "You: 3 VP, tied 2nd with Celts and Switzerland." */
  standing: string;
  /** "Leader: Mongols, 5 VP." or "You lead by 2 VP." */
  leader: string;
  /** A team holding its target, and how soon it wins. */
  hold: string | null;
  lines: SheetLine[];
  /** Shown again from the report: the sheet is the score now, so the header says so rather than "begins". */
  replay?: boolean;
}

const ord = (n: number) => `${n}${n % 10 === 1 && n % 100 !== 11 ? 'st' : n % 10 === 2 && n % 100 !== 12 ? 'nd' : n % 10 === 3 && n % 100 !== 13 ? 'rd' : 'th'}`;
const and = (names: string[]) => names.length <= 1 ? names.join('') : `${names.slice(0, -1).join(', ')} and ${names[names.length - 1]}`;
const plural = (n: number, one: string, many = `${one}s`) => `${n} ${n === 1 ? one : many}`;

/** The score check for the viewer when Age `era` (1-based) begins, opened by `first`. */
export function scoreCheckFor(view: PlayerView, era: number, first: PlayerId): ScoreCheckCard {
  const model = buildVpModel(view);
  const mine = model.rows.find((r) => r.isMine);
  const target = mine?.target ?? model.rows[0]?.target ?? 10;
  const name = view.eras?.find((e) => e.era === era)?.name ?? `Age ${roman(era)}`;

  const track: TrackSpace[] = Array.from({ length: target + 1 }, (_, vp) => ({ vp, pieces: [] }));
  for (const row of model.rows) {
    const space = track[Math.max(0, Math.min(target, row.vp))];
    for (const m of row.members) space.pieces.push({ id: m.id, faction: m.faction, color: playerColor(m.id), name: m.name, isMine: m.isMine });
  }

  const you = mine?.members.length && mine.members.length > 1 ? 'Your team' : 'You';
  let standing = `${you}: ${mine?.vp ?? 0} VP.`;
  let leader = '';
  if (mine) {
    const tied = model.rows.filter((r) => r !== mine && r.vp === mine.vp && r.rank === mine.rank).map((r) => r.label);
    standing = tied.length
      ? `${you}: ${plural(mine.vp, 'VP', 'VP')}, tied ${ord(mine.rank)} with ${and(tied)}.`
      : `${you}: ${plural(mine.vp, 'VP', 'VP')}, ${ord(mine.rank)} of ${model.rows.length}.`;
    const top = model.rows.filter((r) => r.rank === 1);
    const next = model.rows.find((r) => r.rank > 1);
    if (top.length === 1 && top[0] === mine) leader = next ? `${you === 'You' ? 'You lead' : 'Your team leads'} by ${plural(mine.vp - next.vp, 'VP', 'VP')}.` : '';
    else leader = `${top.length > 1 ? 'Leaders' : 'Leader'}: ${and(top.map((r) => r.label))}, ${plural(top[0].vp, 'VP', 'VP')}.`;
  }
  const holding = model.holds[0];
  const hold = holding?.hold
    ? holding.isMine ? `${you} ${you === 'You' ? 'hold' : 'holds'} ${holding.vp} VP and ${you === 'You' ? 'win' : 'wins'} in ${plural(holding.hold.left, 'turn')} if you keep it.`
      : `${holding.label} ${holding.members.length > 1 ? 'hold' : 'holds'} ${holding.vp} VP and ${holding.members.length > 1 ? 'win' : 'wins'} in ${plural(holding.hold.left, 'turn')} unless someone stops them.`
    : null;

  const opener = first === view.me ? `You reached the ${name} era first.` : `${empireOfPlayer(view, first)} reached the ${name} era first.`;
  return { era, name, numeral: roman(era), turn: view.turn, opener, target, track, standing, leader, hold, lines: sheetLines(view, era) };
}

function sheetLines(view: PlayerView, era: number): SheetLine[] {
  const me = view.players.find((p) => p.id === view.me);
  if (!me) return [];
  const team = new Set(view.players.filter((p) => p.team === view.team).map((p) => p.id));
  return [ageLine(view, era), capitalLine(view, team), wonderLine(view), goalLine(view, me.faction)].filter((l): l is SheetLine => !!l);
}

/** Age cards II..IX: scored, open (the new one marked), or still to come. */
function ageLine(view: PlayerView, era: number): SheetLine {
  const boxes: Box[] = [];
  const open: { era: number; name: string; card: string; current: number; target: number }[] = [];
  // The newest Age begun is the new card: on a replay from an older report, that is the Age begun since.
  const newest = Math.max(era, ...(view.eras ?? []).map((x) => x.era));
  for (let e = 2; e <= ERA_COUNT; e++) {
    const ev = view.eras?.find((x) => x.era === e);
    const o = ev ? eraObjectivesFor(view, e) : null;
    const line = o?.lines[0];
    if (!ev || !o) { boxes.push({ state: 'later', mark: roman(e), title: `${eraName(e - 1)}: not begun yet` }); continue; }
    if (o.state === 'scored') { boxes.push({ state: 'kept', mark: roman(e), title: `${ev.name}: scored, yours for good` }); continue; }
    // The name and its plain ask, "Breach, capture a walled city"; a card with no name is its ask, "Gain 4 cities".
    const card = line ? (line.text.includes(':') ? `${cardName(line.text)}, ${line.text.slice(line.text.indexOf(':') + 1).trim()}` : line.text) : 'its card';
    boxes.push({ state: e === newest ? 'new' : 'open', mark: roman(e), title: `${ev.name}: ${card}, still open` });
    open.push({ era: e, name: ev.name, card, current: line?.current ?? 0, target: line?.target ?? 0 });
  }
  const fresh = open.find((a) => a.era === newest);
  const older = open.filter((a) => a.era !== newest);
  const progress = (a: (typeof open)[number]) => a.target > 1 ? ` · you ${Math.min(a.current, a.target)} of ${a.target}` : '';
  const how = fresh ? `New card: ${fresh.card}${progress(fresh)}` : older[0] ? `${older[0].name} card: ${older[0].card}${progress(older[0])}` : 'Every Age so far is scored';
  const rest = fresh ? older : older.slice(1);
  const detail = rest.length
    ? `Also still open: ${and(rest.map((a) => `${a.name} (${a.card})`))}.`
    : fresh ? 'Cards never expire: this one stays open until you score it.' : 'Cards never expire.';
  return { key: 'ages', name: 'Age cards', terms: '1 VP each', boxes, how, detail, open: open.length, fresh: !!fresh };
}

/** Rival original capitals: 1 VP each while you hold it (rules 50); a Celtic sack keeps its 1 VP for good. */
function capitalLine(view: PlayerView, team: Set<PlayerId>): SheetLine {
  const me = view.players.find((p) => p.id === view.me)!;
  const rivals = view.players.filter((p) => !team.has(p.id));
  const held = me.capitals_taken ?? [], sacked = me.capitals_sacked ?? [];
  const done = new Set([...held, ...sacked].map(([id]) => id));
  const left = Math.max(0, rivals.length - done.size);
  const boxes: Box[] = [
    ...sacked.map(([, name]): Box => ({ state: 'kept', star: true, title: `${name}: sacked, yours for good` })),
    ...held.map(([, name]): Box => ({ state: 'held', star: true, title: `${name}: yours while you hold it` })),
    ...Array.from({ length: left }, (): Box => ({ state: 'open', star: true, title: 'A rival capital you do not hold' })),
  ];
  const mineCities = view.cities.filter((c) => c.owner === view.me);
  const targets = view.cities
    .filter((c) => c.original_capital && c.original_capital_of != null && !team.has(c.original_capital_of) && !team.has(c.owner) && !done.has(c.id))
    .map((c) => {
      const from = mineCities.reduce<{ name: string; d: number } | null>((best, m) => { const d = distance(m.pos, c.pos); return !best || d < best.d ? { name: m.name, d } : best; }, null);
      return { city: c, from };
    })
    .filter((t) => t.from)
    .sort((a, b) => a.from!.d - b.from!.d);
  const near = targets[0];
  const keep = held.length ? `Lose ${held.length === 1 ? held[0][1] : 'one'} and you lose its 1 VP.` : '';
  const detail = left === 0 ? (keep || 'You hold every rival capital.')
    : near ? `Nearest known: ${near.city.name} (${empireOfPlayer(view, near.city.original_capital_of!)}), ${plural(near.from!.d, 'tile')} from your city ${near.from!.name}.`
    : 'No rival capital on your map yet: explore to find them.';
  return { key: 'capitals', name: 'Rival capitals', terms: '1 VP each', boxes, how: left ? 'Take a rival’s original capital' : 'You hold them all', detail, open: left };
}

/** World wonders: 2 VP each while you hold its city. Your held ones, then the ones you could build now. */
function wonderLine(view: PlayerView): SheetLine {
  const me = view.players.find((p) => p.id === view.me)!;
  const known = new Set(view.research.known);
  const landmarkOf = (b: BuildingKind) => view.tiles.find((t) => t.landmark?.building === b);
  const built = (b: BuildingKind) => !!landmarkOf(b) || !!view.great_works?.some((g) => g.completed === b) || view.cities.some((c) => c.great_works?.includes(b));
  const heldVp = me.vp_sources.filter(([s]) => s.kind === 'GreatWork').length;
  const heldKinds = view.tiles.filter((t) => t.landmark && isGreatWork(t.landmark.building) && t.owner === view.me).map((t) => t.landmark!.building);
  const ownWork = view.great_works?.find((g) => g.player === view.me);
  if (heldVp > heldKinds.length && ownWork?.completed && !heldKinds.includes(ownWork.completed)) heldKinds.push(ownWork.completed);
  const heldNames = heldKinds.map((b) => BUILDING_NAME[b]);
  const open = GREAT_WORKS.filter((b) => known.has(BUILDING_STATS[b].requires as never) && !built(b));
  const later = GREAT_WORKS.filter((b) => !known.has(BUILDING_STATS[b].requires as never) && !built(b));
  const boxes: Box[] = [
    ...Array.from({ length: heldVp }, (_, i): Box => {
      const b = heldKinds[i];
      return { state: 'held', span: 2, icon: b ? buildingIconName(b, me.faction) : GREAT_WORK_ICON, title: `${b ? BUILDING_NAME[b] : 'A world wonder'}: yours while you hold its city` };
    }),
    ...open.map((b): Box => ({ state: 'open', span: 2, icon: buildingIconName(b, me.faction), title: `${BUILDING_NAME[b]}: nobody has built it yet` })),
  ];
  const building = ownWork?.building && !ownWork.completed ? ownWork : null;
  const how = building ? `Building the ${BUILDING_NAME[building.building!]}${building.turns != null ? `: ${plural(building.turns, 'turn')} left` : ''}`
    : open.length ? `Build the ${and(open.slice(0, 2).map((b) => BUILDING_NAME[b]))}`
    : later.length ? `Next: the ${BUILDING_NAME[later[0]]} (needs ${TECH_NAME[BUILDING_STATS[later[0]].requires as never]})`
    : 'Every world wonder is built';
  const keep = heldNames.length ? `You hold the ${and(heldNames)}: lose its city and you lose its 2 VP.` : '';
  const detail = keep || (open.length || later.length ? 'Only one of each can be built. Whoever holds its city scores 2 VP.' : 'Capture a wonder’s city to take its 2 VP.');
  return { key: 'wonders', name: 'World wonders', terms: '2 VP each', boxes, how, detail, open: open.length * 2 };
}

/** Your faction goal (and any rival's held goal you hold). None for a faction without a goal. */
function goalLine(view: PlayerView, faction: Faction): SheetLine | null {
  const vp = factionGoalVp(faction);
  const me = view.players.find((p) => p.id === view.me)!;
  const heldRival = me.vp_sources.filter(([s]) => s.kind === 'FactionGoal' && s.of !== view.me);
  if (vp <= 0 && !heldRival.length) return null;
  const g = FACTION_GOAL[faction];
  const model = buildVpModel(view);
  const own = model.goals.find((s) => s.only === view.me);
  const mine = !!own?.owners.includes(view.me);
  const live = !!(g.live || g.held);
  const boxes: Box[] = [
    ...(vp > 0 ? [mine ? { state: live ? 'held' : 'kept', span: vp, icon: g.icon, title: `${g.name}: ${live ? 'yours while it stays done' : 'yours for good'}` } as Box : { state: 'open', span: vp, icon: g.icon, title: `${g.name}: not done yet` } as Box] : []),
    ...heldRival.map(([s, n]): Box => {
      const rival = FACTION_GOAL[view.players.find((p) => s.kind === 'FactionGoal' && p.id === s.of)?.faction ?? faction];
      return { state: 'held', span: n, icon: rival.icon, title: `${rival.name}: yours while you hold it` };
    }),
  ];
  const p = own?.progress;
  const count = p && p.target > 1 ? ` · you ${Math.min(p.value, p.target)} of ${p.target}` : '';
  const how = vp <= 0 ? 'No faction goal' : mine ? (live ? `${g.name}: held` : `${g.name}: done`) : `${g.name}${count}`;
  const held = heldRival.map(([s]) => s.kind === 'FactionGoal' ? model.goals.find((x) => x.only === s.of)?.name : undefined).filter(Boolean) as string[];
  // One sentence, never the name again: the goal's plain ask (user's plain-language rule, 2026-11-22).
  const detail = [vp > 0 && !mine ? (g.ask ?? g.rule)() : vp > 0 && live ? own?.holdNote ?? 'You lose it the turn you stop holding it.' : '', held.length ? `You also hold ${and(held)}.` : '']
    .filter(Boolean).join(' ');
  return { key: 'goal', name: 'Faction goal', terms: `${vp || 2} VP`, boxes, how, detail: detail ? `${detail.replace(/\.?$/, '.')}` : '', open: mine ? 0 : vp };
}
