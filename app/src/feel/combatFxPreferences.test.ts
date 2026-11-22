// @vitest-environment happy-dom
import { afterEach, expect, it } from 'vitest';
import { COMBAT_FX_KEY, readCombatFxPrefs } from './combatFxPreferences';

afterEach(() => localStorage.clear());

it('is On by default', () => {
  expect(readCombatFxPrefs()).toEqual({ enabled: true });
});

it('a saved Fast (the removed third choice) reads as On, and Off stays Off', () => {
  localStorage.setItem(COMBAT_FX_KEY, JSON.stringify({ enabled: true, speed: 1.7 }));
  expect(readCombatFxPrefs()).toEqual({ enabled: true });
  localStorage.setItem(COMBAT_FX_KEY, JSON.stringify({ enabled: false, speed: 1 }));
  expect(readCombatFxPrefs()).toEqual({ enabled: false });
});

it('corrupt storage falls back to On', () => {
  localStorage.setItem(COMBAT_FX_KEY, '{not json');
  expect(readCombatFxPrefs()).toEqual({ enabled: true });
});
