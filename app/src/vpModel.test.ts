import { afterEach, describe, expect, it } from 'vitest';
import { FixtureClient } from './test/fixtureEngine';
import { buildVpModel, ownWords } from './vpModel';
import { withV2 } from './test/vpV2Fixture';
import { FACTION_GOAL, FACTION_INFO, decadencePercent, factionGoalRule } from './data';
import type { PlayerView, VpSource } from './types';

async function eightEmpireView(): Promise<PlayerView> {
  const client = new FixtureClient();
  const setup = await client.standardSetup('42', 'India', 'Byzantium', 'Ada', 'Bo');
  await client.newGame(setup);
  return structuredClone(client.view(0));
}

function awardWorks(view: PlayerView, owners: number[]) {
  view.players = view.players.map((p) => ({
    ...p,
    vp_sources: [
      ...p.vp_sources.filter(([source]) => source.kind !== 'GreatWork'),
      ...(owners.includes(p.id) ? [[{ kind: 'GreatWork', of: p.id }, 2] as [VpSource, number]] : []),
    ],
  }));
}

const live = FACTION_GOAL.India.live;
afterEach(() => { FACTION_GOAL.India.live = live; });

describe('the victory race model', () => {
  it('keeps every named original capital point in the race breakdown', async () => {
    const view = withV2(await eightEmpireView());
    const me = view.players.find((p) => p.id === view.me)!;
    me.capitals_taken = [[12, 'Rome'], [19, 'Athens']];
    me.vp_sources.push([{ kind: 'CapitalTaken', capital: 12 }, 1], [{ kind: 'CapitalTaken', capital: 19 }, 1]);
    const model = buildVpModel(view);
    expect(model.capitals.find((c) => c.owner === me.id)?.names).toEqual(['Rome', 'Athens']);
    expect(model.empires.find((e) => e.id === me.id)?.held.filter((s) => s.kind === 'capital').map((s) => s.name)).toEqual(['Rome', 'Athens']);
  });
  it('feeds ledger columns in rank order, keeping teammates adjacent with one shared 20 VP target', async () => {
    const view = withV2(await eightEmpireView());
    view.standings.find((r) => r.team === view.team)!.target = 20;
    const m = buildVpModel(view);
    expect(m.rows.map((r) => r.rank)).toEqual([...m.rows.map((r) => r.rank)].sort((a, b) => a - b));
    expect(m.rows.flatMap((r) => r.members).map((e) => e.id)).toHaveLength(m.empires.length);
    const team = m.rows.find((r) => r.isMine)!;
    expect(team.members).toHaveLength(2);
    expect(team.target).toBe(20);
    expect(team.members.every((e) => e.isMine)).toBe(true);
  });

  it('lists every way to score: the Great Work, the Ages that have begun, one goal per living empire', async () => {
    const view = withV2(await eightEmpireView());
    const m = buildVpModel(view);
    // Great Works have their own scoring source.
    expect(m.works.map((s) => s.id)).toEqual(['GreatWork']);
    // An Age is in the view only once the first empire enters its era (the fixture's rival is in era 3).
    expect(m.eras.map((s) => s.id)).toEqual(['era-2', 'era-3']);
    expect(m.goals).toHaveLength(view.players.filter((p) => p.alive).length);
    expect(m.all.length).toBe(1 + 2 + m.goals.length);
    for (const s of m.all) { expect(s.rule.length).toBeGreaterThan(5); expect(s.icon).toBeTruthy(); }
    const words = JSON.stringify(m.all.map((s) => [s.name, s.rule, s.terms]));
    expect(words).not.toMatch(/Developed Empire|Six Technologies|Twenty Population|Most Populous|Most Territory/);
  });

  it('marks each source mine, a rival’s or nobody’s from the viewer’s side', async () => {
    const view = withV2(await eightEmpireView());
    const rival = view.players.find((p) => p.team !== view.team)!.id;
    awardWorks(view, [rival, view.me]);
    const m = buildVpModel(view);
    expect(m.byId.get('era-2')!.own).toBe('theirs');
    expect(m.byId.get('GreatWork')!.owners).toEqual([view.me, rival]);
    expect(m.byId.get('GreatWork')!.own).toBe('mine');
  });

  it('puts what an empire holds in its row: kept points first, live ones after', async () => {
    const view = withV2(await eightEmpireView(), { scored: [2] });
    awardWorks(view, [view.me]);
    const m = buildVpModel(view);
    const row = m.rows.find((r) => r.isMine)!;
    expect(row.tokens.map((t) => t.source.id)).toEqual(['era-2', 'GreatWork']);
    expect(row.tokens.map((t) => t.source.live)).toEqual([false, true]);
    // Neither member's goal is done (the fixture team has two): each closes the row as a pending crest.
    expect(row.pendingGoals.map((t) => t.source.kind)).toEqual(['goal', 'goal']);
  });

  it('carries every empire’s public Gold', async () => {
    const view = await eightEmpireView();
    view.players.forEach((p, i) => { p.gold = 100 + i; });
    const m = buildVpModel(view);
    expect(m.empires.map((e) => e.gold)).toEqual(m.empires.map((e) => 100 + view.players.findIndex((p) => p.id === e.id)));
  });

  it('carries whether each empire’s capital stands, a team standing while any member’s does', async () => {
    const view = await eightEmpireView();
    expect(buildVpModel(view).rows.every((r) => r.capitalStands && r.members.every((e) => e.capitalStands))).toBe(true);
    const mine = view.players.filter((p) => p.team === view.team);
    mine[0].capital_stands = false;
    let row = buildVpModel(view).rows.find((r) => r.isMine)!;
    expect(row.members.find((e) => e.id === mine[0].id)!.capitalStands).toBe(false);
    expect(row.capitalStands).toBe(mine.length > 1);
    mine.forEach((p) => { p.capital_stands = false; });
    row = buildVpModel(view).rows.find((r) => r.isMine)!;
    expect(row.capitalStands).toBe(false);
  });

  it('carries each empire’s lead and decadence rate, derived by the same function the core mirrors', async () => {
    const view = withV2(await eightEmpireView(), { era: 2, lead: 60, rivalLead: 100 });
    const m = buildVpModel(view);
    const me = m.empires.find((e) => e.isMe)!;
    const rival = m.empires.find((e) => !e.isMine && e.lead > 0)!;
    expect([me.lead, me.decadence]).toEqual([60, decadencePercent(60, { decadenceSlope: 45 })]);
    expect([rival.lead, rival.decadence]).toEqual([100, decadencePercent(100, { decadenceSlope: 45 })]);
    expect(m.empires.filter((e) => e.lead === 0).every((e) => e.decadence === 0)).toBe(true);
    expect(m.raceRules).toMatchObject({ decadence: decadencePercent(60, { decadenceSlope: 45 }) });
    // The game can tune the decadence curve through its rule switches.
    const tuned = buildVpModel(withV2(await eightEmpireView(), { era: 2, lead: 60, rules: { decadence_slope: 25 } }));
    expect(tuned.raceRules.decadence).toBe(decadencePercent(60, { decadenceSlope: 25 }));
    expect(tuned.raceRules.rules).toMatchObject({ decadenceSlope: 25 });
  });

  it('turns each Age into a way to score: its cards, who scored with which, and only your own progress', async () => {
    const view = withV2(await eightEmpireView(), { era: 2, scored: [] });
    const m = buildVpModel(view);
    const rival = view.players.find((p) => p.team !== view.team && p.alive)!.id;
    const era2 = m.byId.get('era-2')!;
    expect(era2.kind).toBe('era');
    expect(era2.vp).toBe(1);
    expect(era2.rule).toMatch(/Score this Age’s card whenever you like/);
    expect(era2.era!.eraName).toBe('Classical');
    expect(era2.era!.objectives.map((o) => [o.family, o.scorers])).toEqual([['Expand', [rival]], ['Expand', []], ['Fight', []]]);
    expect(era2.era!.objectives[1].mine).toEqual({ current: 2, target: 4 });
    expect(era2.era).toMatchObject({ myEntered: true, myScored: false });
    expect(era2.owners).toEqual([rival]);
    expect(era2.own).toBe('theirs');
    // An Age a rival began is open to you too, with your progress, though you have not entered its era.
    expect(m.byId.get('era-3')!.era).toMatchObject({ myEntered: true, myScored: false });
    expect(m.byId.get('era-3')!.era!.objectives[0].mine).toEqual({ current: 6, target: 14 });
    // An Age nobody has begun is not in the view.
    expect(m.byId.get('era-4')).toBeUndefined();
    // Your own score makes it yours.
    const scored = buildVpModel(withV2(await eightEmpireView(), { era: 2, scored: [2] })).byId.get('era-2')!;
    expect(scored.own).toBe('mine');
    expect(scored.era!.myScored).toBe(true);
  });

  it('keeps an era with no entrant and no scorer off the race, and an unscored entered era in the Nobody row', async () => {
    const view = withV2(await eightEmpireView(), { era: 2 });
    view.eras[0].scored_by = [];
    const m = buildVpModel(view);
    expect(m.open.map((s) => s.id)).toContain('era-2');
    expect(m.open.map((s) => s.id)).not.toContain('era-4');
  });

  it('uses the fixed goal threshold and keeps live-or-permanent in the data', async () => {
    const standard = buildVpModel(await eightEmpireView());
    const byzantium = (m: ReturnType<typeof buildVpModel>) => m.goals.find((g) => g.faction === 'Byzantium')!;
    expect(byzantium(standard).rule).toContain('Heir');
    expect(factionGoalRule('Switzerland')).toContain('5,000 Gold');
    expect(factionGoalRule('Switzerland')).not.toContain('500 Gold');
    // Held, so it can be lost (outlined like a live source), and its terms say who scores it.
    expect(byzantium(standard).live).toBe(true);
    expect(byzantium(standard).terms).toContain('Held goal');
    FACTION_GOAL.India.live = true;
    const india = buildVpModel(await eightEmpireView()).goals.find((g) => g.faction === 'India')!;
    expect(india.live).toBe(true);
    expect(india.terms).toContain('Held goal');
  });

  it('shows a Live goal’s meter while it is held, so its owner sees how close it is to being lost', async () => {
    const view = await eightEmpireView();
    view.faction_goal = { value: 520, target: 500, label: '520 of 500 Gold', live: true, done: true };
    const mine = buildVpModel(view).goals.find((g) => g.only === view.me)!;
    // A measure never reads past its target.
    expect(mine.progress).toEqual({ value: 520, target: 500, label: '500 of 500 Gold' });
  });

  it('keeps a rival’s goal progress private except a public count', async () => {
    const view = await eightEmpireView();
    const m = buildVpModel(view);
    const mine = m.goals.find((g) => g.only === view.me)!;
    const rival = m.goals.find((g) => g.only !== view.me && !g.held)!;
    expect(rival.progress).toBeUndefined();
    expect(rival.privacy).toMatch(/private/);
    expect(mine.privacy).toBeUndefined();
    // India's vow is public: whether its country is closed, and the count, are everyone's facts (never its VP route).
    const india = m.goals.find((g) => g.faction === 'India' && g.only !== view.me);
    if (india) expect(india.facts[0].label).toBe('Country open');
  });

  it('says what the faction sheets say: the goal line is composed from the same data', () => {
    for (const [faction, info] of Object.entries(FACTION_INFO)) {
      const g = FACTION_GOAL[faction as keyof typeof FACTION_GOAL];
      expect(info.goal.startsWith(`${g.name}: `)).toBe(true);
    }
    expect(FACTION_INFO.Switzerland.goal).toContain('Keep 5,000 Gold');
    expect(FACTION_INFO.Byzantium.goal).toContain('The Heir to the Throne');
  });

  it('suggests nothing: no routes, no fastest way, no next points anywhere in its words', async () => {
    const view = withV2(await eightEmpireView());
    const m = buildVpModel(view);
    const words = m.all.flatMap((s) => [s.rule, s.terms, s.privacy ?? '', s.progress?.label ?? '']).join(' ');
    expect(words).not.toMatch(/\b(route|fastest|recommend|suggest|next|try to|should|closest)\b/i);
  });

  it('reads each team’s hold-to-win countdown from the standings, soonest win first', async () => {
    const view = await eightEmpireView();
    expect(buildVpModel(view).holds).toEqual([]);
    const other = view.standings.find((s) => s.team !== view.team)!;
    const mine = view.standings.find((s) => s.team === view.team)!;
    other.victory_hold = { turns_held: 4, turns_needed: 10 };
    mine.victory_hold = { turns_held: 9, turns_needed: 10 };
    const m = buildVpModel(view);
    expect(m.holds.map((r) => [r.isMine, r.hold!.left])).toEqual([[true, 1], [false, 6]]);
    // Not holding (zero turns) is no countdown.
    other.victory_hold = { turns_held: 0, turns_needed: 10 };
    expect(buildVpModel(view).holds.map((r) => r.isMine)).toEqual([true]);
  });

  describe('a held goal (Byzantium’s Heir to the Throne)', () => {
    const setup = async () => {
      const view = await eightEmpireView();
      const byzantium = view.players.find((p) => p.faction === 'Byzantium')!;
      const rival = view.players.find((p) => p.team !== view.team && p.id !== byzantium.id)!;
      const heirGoal = (m: ReturnType<typeof buildVpModel>) => m.goals.find((g) => g.only === byzantium.id)!;
      return { view, byzantium, rival, heirGoal };
    };

    it('is held, not live-or-permanent: nobody holds it before he is trained', async () => {
      const { view, heirGoal } = await setup();
      const g = heirGoal(buildVpModel(view));
      expect(g.held).toBe(true);
      expect(g.live).toBe(true);
      expect(g.owners).toEqual([]);
      expect(g.own).toBe('open');
      expect(g.terms).toContain('Held goal');
      expect(g.holdNote).toBe('Nobody holds the Heir now');
    });

    it('keeps its card in the holder\'s row when Byzantium has fallen, crest and all', async () => {
      const { view, byzantium, rival, heirGoal } = await setup();
      rival.vp_sources.push([{ kind: 'FactionGoal', of: byzantium.id }, 2]);
      byzantium.alive = false;
      const m = buildVpModel(view);
      const g = heirGoal(m);
      expect(g.owners).toEqual([rival.id]);
      const rivalRow = m.rows.find((r) => r.members.some((e) => e.id === rival.id))!;
      const token = rivalRow.tokens.find((t) => t.source.id === g.id)!;
      expect(token.of?.faction).toBe('Byzantium');
    });

    it('says where he is when he is in view, so the token can show him on the map', async () => {
      const { view, byzantium, rival, heirGoal } = await setup();
      rival.vp_sources.push([{ kind: 'FactionGoal', of: byzantium.id }, 2]);
      expect(heirGoal(buildVpModel(view)).heldAt).toBeUndefined();
      view.units.push({ ...view.units[0], id: 9999, owner: rival.id, kind: 'HeirToTheThrone', heir_of: byzantium.id, pos: { q: 4, r: 5 } });
      expect(heirGoal(buildVpModel(view)).heldAt).toEqual({ q: 4, r: 5 });
    });

    it('is owned by the empire whose public VP sources name it, whoever that is', async () => {
      const { view, byzantium, rival, heirGoal } = await setup();
      rival.vp_sources.push([{ kind: 'FactionGoal', of: byzantium.id }, 2]);
      const m = buildVpModel(view);
      const g = heirGoal(m);
      expect(g.owners).toEqual([rival.id]);
      expect(g.own).toBe('theirs');
      expect(g.holdNote).toContain(rival.name.length ? 'holds the Heir' : '');
      // The token sits in the holder's row with Byzantium's crest, not in Byzantium's row.
      const rivalRow = m.rows.find((r) => r.members.some((e) => e.id === rival.id))!;
      const token = rivalRow.tokens.find((t) => t.source.id === g.id)!;
      expect(token.owner.id).toBe(rival.id);
      expect(token.of?.faction).toBe('Byzantium');
      const mine = m.rows.find((r) => r.isMine)!;
      expect(mine.tokens.some((t) => t.source.id === g.id)).toBe(false);
      expect(mine.pendingGoals.some((t) => t.source.id === g.id)).toBe(false);
    });

    it('follows the holder from the viewer’s side, and goes back to open when he is lost', async () => {
      const { view, byzantium, rival, heirGoal } = await setup();
      // My own team holds him (my seat or the Byzantium seat on my team): mine.
      const mineHolder = view.players.find((p) => p.id === view.me)!;
      mineHolder.vp_sources.push([{ kind: 'FactionGoal', of: byzantium.id }, 2]);
      expect(heirGoal(buildVpModel(view)).own).toBe('mine');
      mineHolder.vp_sources = mineHolder.vp_sources.filter(([s]) => s.kind !== 'FactionGoal');
      rival.vp_sources.push([{ kind: 'FactionGoal', of: byzantium.id }, 2]);
      expect(heirGoal(buildVpModel(view)).own).toBe('theirs');
      rival.vp_sources = rival.vp_sources.filter(([s]) => s.kind !== 'FactionGoal');
      expect(heirGoal(buildVpModel(view)).own).toBe('open');
    });

    it('places him wherever he is (his position is public), says whether he is in sight, and who has him when barbarians do', async () => {
      const { view, byzantium, heirGoal } = await setup();
      view.units.push({ ...view.units[0], id: 9999, kind: 'HeirToTheThrone', owner: 255, heir_of: byzantium.id, pos: { q: 3, r: 4 }, military: false });
      const tile = view.tiles[4 * view.width + 3];
      tile.visible = true;
      const g = heirGoal(buildVpModel(view));
      expect(g.heldAt).toEqual({ q: 3, r: 4 });
      expect(g.holdNote).toBe('Barbarians hold the Heir: nobody scores until the camp is cleared · in view');
      tile.visible = false;
      expect(heirGoal(buildVpModel(view)).holdNote).toBe('Barbarians hold the Heir: nobody scores until the camp is cleared · out of sight');
    });
  });
});

describe('the view’s typed VP sources and goal progress', () => {
  it('shows a separate held mark for each captured Great Work', async () => {
    const view = withV2(await eightEmpireView());
    const holder = view.players.find((p) => p.id === view.me)!;
    holder.vp_sources = [[{ kind: 'GreatWork', of: 2 }, 2], [{ kind: 'GreatWork', of: 3 }, 2]];
    const model = buildVpModel(view);
    expect(model.empires.find((e) => e.id === view.me)!.held.filter((s) => s.kind === 'work')).toHaveLength(2);
    expect(model.works.map((s) => s.id)).toContain('GreatWork-3');
  });

  it('lists each player’s sources by id, consistent with what the model says they hold', async () => {
    const view = await eightEmpireView();
    const rival = view.players.find((p) => p.team !== view.team)!;
    const v2 = withV2(view);
    Object.assign(view, v2);
    awardWorks(view, [rival.id]);
    const r2 = view.players.find((p) => p.id === rival.id)!;
    r2.faction_goal_done = true;
    r2.vp_sources = [[{ kind: 'GreatWork', of: r2.id }, 2], [{ kind: 'EraObjective', era: 2 }, 1], [{ kind: 'FactionGoal', of: r2.id }, 2]];
    view.eras[0].scored_by = [{ player: r2.id, slot: 1, turn: 30 }];
    const m = buildVpModel(view);
    const held = m.empires.find((e) => e.id === r2.id)!.held.map((s) => s.id).sort();
    const typed = r2.vp_sources.map(([s]) => (s.kind === 'FactionGoal' ? `goal-${r2.id}` : s.kind === 'EraObjective' ? `era-${s.era}` : s.kind === 'GreatWork' ? 'GreatWork' : `capital-${r2.id}-${s.capital}`)).sort();
    expect(held).toEqual(typed);
  });

  it('reads your own goal’s progress from faction_goal only; a rival’s stays private', async () => {
    const view = await eightEmpireView();
    view.faction_goal = { value: 2, target: 5, label: '2 of 5 Ruhr Foundries', live: false, done: false };
    const m = buildVpModel(view);
    const mine = m.goals.find((g) => g.only === view.me)!;
    expect(mine.progress).toEqual({ value: 2, target: 5, label: '2 of 5 Ruhr Foundries' });
    expect(m.goals.filter((g) => g.only !== view.me).every((g) => !g.progress)).toBe(true);
    view.faction_goal = { value: 6, target: 5, label: '6 of 5 Ruhr Foundries', live: true, done: true };
    expect(buildVpModel(view).goals.find((g) => g.only === view.me)!.progress?.label).toBe('5 of 5 Ruhr Foundries');
    view.faction_goal = null;
    expect(buildVpModel(view).goals.find((g) => g.only === view.me)!.progress).toBeUndefined();
  });

  it('names every holder of a shared Great Work, and never calls a rival’s card yours', async () => {
    const view = await eightEmpireView();
    const rivals = view.players.filter((p) => p.team !== view.team).map((p) => p.id);
    awardWorks(view, [rivals[0], view.me]);
    let m = buildVpModel(view);
    expect(ownWords(m, m.byId.get('GreatWork')!)).toMatch(/^You and \w+/);
    awardWorks(view, [rivals[0], rivals[1]]);
    m = buildVpModel(view);
    expect(ownWords(m, m.byId.get('GreatWork')!)).toContain(', ');
    expect(m.byId.get('GreatWork')!.own).toBe('theirs');
    awardWorks(view, rivals.slice(0, 4));
    m = buildVpModel(view);
    expect(ownWords(m, m.byId.get('GreatWork')!)).toBe('4 empires');
  });
});
