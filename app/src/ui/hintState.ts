import { BARBARIAN } from '../barbarians';
import { WORK_RADIUS } from '../data';

import { effectiveProduction, effectiveResearchQueue } from '../orders';
import { create } from 'zustand';
import { pendingDecisions } from '../turnFlow';
import { atWar, barbariansSpareMe } from '../diplomacy';
import { empireTitle } from '../empires';
import type { Order, OrderError, PlayerView } from '../types';
import type { GuideSection } from './guideState';

export const HINT_KEY = 'legatum.first-game-hints.v2';
export const HINT_IDS = ['production', 'research', 'enemy', 'ready'] as const;
export type HintId = typeof HINT_IDS[number];
// One action sentence each (pillar 9): the rules behind it live in the guide chapter the hint links to.
const HINTS: Record<HintId, { title: string; body: string; section: GuideSection }> = {
  production: { title: 'Pick production', body: `Choose something for your capital to build. It works every owned tile within ${WORK_RADIUS} tiles, and each time it grows it claims one more tile.`, section: 'cities' },
  research: { title: 'Choose your first research', body: 'Press T or click Research at the top, then pick one. Bronze Working unlocks Workshops (Production); Irrigation gives every city +1 Food. Finish an era to enter the next.', section: 'cities' },
  enemy: { title: 'A rival in sight', body: 'You are at War with them. Check the battle forecast before you attack: everyone moves at once and their orders are hidden.', section: 'combat' },
  ready: { title: 'Ready to resolve?', body: 'Everything has orders. End the turn (Return; Ready in multiplayer) and every empire’s orders resolve together.', section: 'turns' },
};

/** The hint as shown: first contact with an empire at peace says so instead of warning of war. */
export function hintCopy(id: HintId, view: PlayerView): { title: string; body: string; section: GuideSection } {
  const hint = HINTS[id];
  if (id !== 'enemy') return hint;
  const sighted = [...view.units.map((u) => u.owner), ...view.cities.filter((c) => c.visible).map((c) => c.owner)]
    .flatMap((owner) => view.players.filter((p) => p.id === owner && p.team !== view.team && p.alive));
  if (sighted.some((p) => atWar(view, p.id))) return hint;
  const name = sighted[0] && empireTitle(view, sighted[0].team);
  return { ...hint, body: `You are at Peace${name ? ` with ${name}` : ''}. A rival can declare War and attack in the same turn. Diplomacy is in the Victory race.`, section: 'turns' };
}

interface HintPreferences { enabled: boolean; seen: HintId[] }

export function readHintPreferences(): HintPreferences {
  try {
    const raw: unknown = JSON.parse(globalThis.localStorage?.getItem(HINT_KEY) ?? 'null');
    if (raw && typeof raw === 'object') {
      const p = raw as { enabled?: unknown; seen?: unknown };
      const seen: unknown[] = Array.isArray(p.seen) ? p.seen : [];
      return {
        enabled: p.enabled !== false,
        seen: HINT_IDS.filter((id) => seen.includes(id)),
      };
    }
  } catch { /* Unavailable or invalid storage should never prevent playing. */ }
  return { enabled: true, seen: [] };
}

function savePreferences(p: HintPreferences) {
  try { globalThis.localStorage?.setItem(HINT_KEY, JSON.stringify({ enabled: p.enabled, seen: p.seen })); }
  catch { /* Keep session preferences even if storage is blocked. */ }
}

interface HintState extends HintPreferences {
  resetVersion: number;
  markSeen(id: HintId): void;
  setEnabled(enabled: boolean): void;
  reset(): void;
}

export const useHintPreferences = create<HintState>((set) => ({
  ...readHintPreferences(),
  resetVersion: 0,
  markSeen: (id) => set((s) => {
    if (s.seen.includes(id)) return s;
    const next = { ...s, seen: [...s.seen, id] };
    savePreferences(next);
    return next;
  }),
  setEnabled: (enabled) => set((s) => {
    const next = { ...s, enabled };
    savePreferences(next);
    return next;
  }),
  reset: () => set((s) => {
    const next = { ...s, enabled: true, seen: [], resetVersion: s.resetVersion + 1 };
    savePreferences(next);
    return next;
  }),
}));

/** Read only the current player's fog-filtered view, never public rival counts or another seat. */
export function eligibleHints(view: PlayerView, orders: Order[], errors: OrderError[] = [], local?: Parameters<typeof pendingDecisions>[3]): HintId[] {
  if (view.finished || view.players.find((p) => p.id === view.me)?.alive === false) return [];
  const cities = view.cities.filter((c) => c.owner === view.me);
  const ids: HintId[] = [];
  if (cities.some((c) => c.detail && c.detail.resistance === 0 &&
    !effectiveProduction(orders, c.id, c.detail.producing))) ids.push('production');
  if (cities.length && !effectiveResearchQueue(orders, view.research).length && !view.research.known.length && view.research.available.length) ids.push('research');
  // Barbarians never attack the Celts, so sighting them is no "rival in sight" for the Celts.
  const enemies = new Set([...(barbariansSpareMe(view) ? [] : [BARBARIAN]), ...view.players.filter((p) => p.team !== view.team && p.alive).map((p) => p.id)]);
  if (view.units.some((u) => enemies.has(u.owner)) || view.cities.some((c) => c.visible && enemies.has(c.owner))) ids.push('enemy');
  if (view.turn <= 3 && orders.length && !errors.length && !pendingDecisions(view, orders, errors, local).length) ids.push('ready');
  return ids;
}
