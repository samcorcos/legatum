import type { Yields } from '../types';

// Kept free of asset imports so presentation logic (moments, unlock cards) loads in plain Node tests.
export type YieldKey = keyof Yields;
export const YIELD_KEYS: YieldKey[] = ['food', 'production', 'gold', 'science'];
export const YIELD_LABEL: Record<YieldKey, string> = {
  food: 'Food',
  production: 'Production',
  gold: 'Gold',
  science: 'Science',
};
