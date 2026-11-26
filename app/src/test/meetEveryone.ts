import type { PlayerView } from '../types';

/** The view after first contact with every empire, for tests about how met empires are named and drawn. Returns the same view. */
export function meetEveryone<V extends PlayerView>(view: V): V {
  view.diplomacy = { ...view.diplomacy, met: [...new Set(view.players.map((p) => p.team))].filter((t) => t !== view.team) };
  return view;
}
