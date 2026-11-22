import { expect, it, vi } from 'vitest';
import { decodeOrders } from './orderDecoding';

it('loads retired order kinds and drops unknown server orders with a warning', () => {
  const warn = vi.spyOn(console, 'warn').mockImplementation(() => {});
  expect(decodeOrders(JSON.parse('[{"type":"HealUnit","unit":3},{"type":"HoldUnit","unit":4},{"type":"FutureOrder","unit":5}]')))
    .toEqual([{ type: 'SleepUnit', unit: 3 }, { type: 'SleepUnit', unit: 4 }]);
  expect(warn).toHaveBeenCalledWith('Dropped unknown order kind: FutureOrder');
  warn.mockRestore();
});

it('keeps a city automation setting from a saved pending turn', () => {
  expect(decodeOrders([{ type: 'SetCityAutomation', city: 4, focus: 'Science' }]))
    .toEqual([{ type: 'SetCityAutomation', city: 4, focus: 'Science' }]);
});

it('keeps a Gold gift, so an online gift reaches the host', () => {
  expect(decodeOrders([{ type: 'GiveGold', to: 1, amount: 120 }])).toEqual([{ type: 'GiveGold', to: 1, amount: 120 }]);
});
