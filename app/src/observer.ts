import type { GameClient } from './client';
import type { PlayerId, PlayerView } from './types';

/** Omniscient God view ('god') or a specific AI empire's fogged seat (PlayerId). */
export type ObserverPerspective = 'god' | PlayerId;

export type ObserverSpeed = 0.5 | 1 | 2 | 4;

export const OBSERVER_SPEEDS: readonly ObserverSpeed[] = [0.5, 1, 2, 4];

/** Delay in ms between automatic turns while Observer playback is running. */
export const OBSERVER_INTERVAL_MS: Record<ObserverSpeed, number> = {
  0.5: 1800,
  1: 900,
  2: 400,
  4: 120,
};

export interface ObserverSnapshot {
  turn: number;
  /** Exact serialized engine state from `client.save()`. */
  save: string;
  hash: string;
}

export interface ObserverState {
  active: boolean;
  playing: boolean;
  speed: ObserverSpeed;
  perspective: ObserverPerspective;
  /** Deterministic snapshots indexed from the initial turn (0 = start). */
  history: ObserverSnapshot[];
  /** Current position in `history`. */
  cursor: number;
}

export const INACTIVE_OBSERVER: ObserverState = {
  active: false,
  playing: false,
  speed: 1,
  perspective: 'god',
  history: [],
  cursor: 0,
};

export function createObserverState(turn: number, save: string, hash: string): ObserverState {
  return {
    active: true,
    playing: false,
    speed: 1,
    perspective: 'god',
    history: [{ turn, save, hash }],
    cursor: 0,
  };
}

/** Read the active perspective's view from the engine without mutating state. */
export function readObserverView(client: GameClient, perspective: ObserverPerspective): PlayerView {
  if (perspective === 'god') {
    return typeof client.observerView === 'function' ? client.observerView() : client.view(0);
  }
  return client.view(perspective);
}

export function canStepBack(obs: ObserverState): boolean {
  return obs.active && obs.cursor > 0;
}

export function isGodView(obs: ObserverState): boolean {
  return obs.active && obs.perspective === 'god';
}
