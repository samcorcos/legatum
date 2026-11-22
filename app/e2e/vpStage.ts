import { revealSidebar } from './sidebarAssertions';
// Shared by the victory-holdings specs: give the empires something to hold, in the store only.
import { expect, type Page } from '@playwright/test';
import type { useGame } from '../src/store';
import { withV2 } from '../src/test/vpV2Fixture';

type GameWindow = Window & { legatum: typeof useGame };

/**
 * Give the empires something to hold (render only), in VP v2 terms: you hold the Great Work (held)
 * and the era you scored (kept for good); the first rival holds its goal and scored era II too; era III is entered by a rival
 * and scored by nobody.
 */
export async function stageHoldings(page: Page, { oneCardPerAge = false }: { oneCardPerAge?: boolean } = {}) {
  const base = await page.evaluate(() => { const s = (window as unknown as GameWindow).legatum.getState(); return structuredClone(s.views![s.seat]); });
  const v = withV2(base, { era: 2, scored: [2] });
  // `withV2` stages three cards per Age so list tests see several lines; a real game draws one.
  if (oneCardPerAge) {
    v.eras = v.eras.map((e) => ({ ...e, objectives: e.objectives.slice(0, 1) }));
    const kept = new Set(v.eras.flatMap((e) => e.objectives.map((o) => `${e.era}:${o.slot}`)));
    v.objective_progress = v.objective_progress.filter((p) => kept.has(`${p.era}:${p.slot}`));
  }
  const rival = v.players.find((p) => p.team !== v.team && p.alive)!;
  v.great_works = (v.great_works ?? []).map((w) => w.player === v.me ? { ...w, completed: 'GreatLibrary' } : w);
  v.players = v.players.map((p) => p.id === v.me ? { ...p, vp: p.vp + 2, vp_sources: [...p.vp_sources, [{ kind: 'GreatWork' as const, of: v.me }, 2] as [{ kind: 'GreatWork'; of: number }, number]] } : p.id === rival.id ? { ...p, faction_goal_done: true } : p);
  v.standings = v.standings.map((row) => row.members.includes(v.me) ? { ...row, vp: row.vp + 2 } : row);
  await page.evaluate((view) => {
    const store = (window as unknown as GameWindow).legatum, s = store.getState();
    store.setState({ views: s.views!.map((x, i) => (i === s.seat ? view : x)), seat: s.seat });
  }, v);
  await revealSidebar(page, 'standings');
  await expect(page.locator('.vp-row.mine [data-source]').first()).toBeVisible();
}

/** Stage the hold-to-win countdown (render only): the first rival has held its target 7 of 10 turns; with `mine`, so have you, 9 of 10. */
export async function stageHold(page: Page, { mine = false }: { mine?: boolean } = {}) {
  await page.evaluate((mine) => {
    const store = (window as unknown as GameWindow).legatum, s = store.getState();
    const v = structuredClone(s.views![s.seat]);
    const rival = v.standings.find((x) => x.team !== v.team)!;
    // Holding means standing at the target: the standings and each member's VP say so.
    const reach = (st: typeof rival, held: number) => {
      st.victory_hold = { turns_held: held, turns_needed: 10 };
      st.vp = st.target;
      v.players = v.players.map((p) => (st.members.includes(p.id) ? { ...p, vp: Math.ceil(st.target / st.members.length) } : p));
    };
    reach(rival, 7);
    if (mine) reach(v.standings.find((x) => x.team === v.team)!, 9);
    store.setState({ views: s.views!.map((x, i) => (i === s.seat ? v : x)), seat: s.seat });
  }, mine);
  await expect(page.locator('.vhc-hold').first()).toBeVisible();
}

/** The Byzantium `of` has its goal held by `holder` (render only): the holder's public VP sources name it, as the core's do. */
export async function stageHeirHolder(page: Page, { holder, of }: { holder: number; of: number }) {
  await page.evaluate(({ holder, of }) => {
    const store = (window as unknown as GameWindow).legatum, s = store.getState();
    const v = structuredClone(s.views![s.seat]);
    v.players = v.players.map((p) => (p.id === holder ? { ...p, vp: p.vp + 2, vp_sources: [...p.vp_sources, [{ kind: 'FactionGoal' as const, of }, 2] as [{ kind: 'FactionGoal'; of: number }, number]] } : p));
    // The HUD and the race rows read the standings: stage the same points the token shows.
    v.standings = v.standings.map((t) => (t.members.includes(holder) ? { ...t, vp: t.vp + 2 } : t));
    store.setState({ views: s.views!.map((x, i) => (i === s.seat ? v : x)), seat: s.seat });
  }, { holder, of });
  await revealSidebar(page, 'standings');
  await expect(page.locator(`.panel.vp .vhc-tok[data-held]`).first()).toBeVisible();
}

/**
 * Stage decadence on your first city (render only), the way the core reports it: the city's tiles and
 * citizens give some yields, and each cut of Production, Gold and Science is its own source line
 * ("Decadence −11% of Production: sloth"). Food has no line: decadence adds a positive "growth" part to the
 * growth threshold breakdown instead. `percent` comes from the staged view.
 */
export async function stageCityDecadence(page: Page, percent: number) {
  await page.evaluate((pct) => {
    const store = (window as unknown as GameWindow).legatum, s = store.getState();
    const v = structuredClone(s.views![s.seat]);
    v.rule_experiments = { ...v.rule_experiments, decadence_slope: 45 };
    const me = v.players.find((p) => p.id === v.me)!;
    me.decadence_percent = pct;
    me.lead_percent = Math.round(pct * 100 / 45);
    const city = v.cities.find((c) => c.owner === v.me && c.detail)!;
    city.population = Math.max(city.population, 3);
    const base = { food: 14, production: 13, gold: 11, science: 9 };
    const share = (n: number) => Math.floor((Math.max(n, 0) * pct + 50) / 100);
    const cut = { production: share(base.production), gold: share(base.gold), science: share(base.science) };
    const line = (label: string, key: 'production' | 'gold' | 'science') => ({ label: `Decadence −${pct}% ${label}`, yields: { food: 0, production: 0, gold: 0, science: 0, [key]: -cut[key] } });
    const y = city.detail!.yields;
    y.sources = [
      { label: 'Tiles worked', yields: { food: 9, production: 8, gold: 5, science: 3 } },
      { label: 'Citizens', yields: { food: 5, production: 5, gold: 6, science: 6 } },
      line('of Production: sloth', 'production'),
      line('of Gold: luxury', 'gold'), line('of Science: complacency', 'science'),
    ];
    y.total = { food: base.food, production: base.production - cut.production, gold: base.gold - cut.gold, science: base.science - cut.science };
    const d = city.detail!, threshold = d.growth_threshold;
    d.growth_breakdown = { total: Math.round((threshold * 100) / (100 - pct)), parts: [['Growth threshold', threshold], [`Decadence −${pct}% growth: smaller families`, Math.round((threshold * 100) / (100 - pct)) - threshold]] };
    d.growth_threshold = d.growth_breakdown.total;
    store.setState({ views: s.views!.map((x, i) => (i === s.seat ? v : x)) });
  }, percent);
}
