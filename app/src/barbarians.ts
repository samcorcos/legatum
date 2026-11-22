// Barbarians in the UI: core `PlayerId::BARBARIAN` / `TeamId::BARBARIAN` (rules/barbarians.rs).
// They own units but are never an empire: not in `view.players`, no diplomacy, always hostile.
import type { Hex, PlayerId, PlayerView } from './types';
import { hexEq } from './hex';
import { BARBARIAN_COLOR } from './theme';
import { RULES } from './data';
import { empireOfPlayer } from './empires';

/** Core `PlayerId::BARBARIAN` and `TeamId::BARBARIAN` (u8::MAX). */
export const BARBARIAN = 255;

export const isBarbarian = (id: PlayerId | null | undefined): boolean => id === BARBARIAN;

/** Token and banner inks: black and brick red, deliberately unlike any player color. */
export const BARBARIAN_INK = { field: '#1a1411', blood: BARBARIAN_COLOR, bone: '#efe6d2' } as const;

/** Gold for clearing a camp (core `barbarians::CAMP_CLEAR_GOLD`, exported to JSON). */
export const CAMP_CLEAR_GOLD: number = RULES.campClearGold;

/** An owner's display name: "You", the empire's name (as everywhere else in the UI), or "Barbarians". */
export function ownerName(view: PlayerView, id: PlayerId): string {
  return isBarbarian(id) ? 'Barbarians' : id === view.me ? 'You' : empireOfPlayer(view, id);
}

/** The camp the viewer knows on this tile, if any (seen now or remembered). */
export function campAt(view: PlayerView, hex: Hex) {
  return view.barbarian_camps?.find((c) => hexEq(c.pos, hex)) ?? null;
}
