// @vitest-environment happy-dom
import { act, createElement, type ReactNode } from 'react';
import { createRoot, type Root } from 'react-dom/client';
import { afterEach, beforeEach, expect, it } from 'vitest';
import { Amount, Amounts, Quantity, YieldAmounts } from './Amounts';

let container: HTMLDivElement;
let root: Root;
beforeEach(() => { (globalThis as { IS_REACT_ACT_ENVIRONMENT?: boolean }).IS_REACT_ACT_ENVIRONMENT = true; container = document.createElement('div'); document.body.append(container); root = createRoot(container); });
afterEach(() => { act(() => root.unmount()); container.remove(); });
const render = (node: ReactNode) => act(() => root.render(node));

it('an amount is a signed number and an icon, with the word kept for readers', () => {
  render(createElement(Amount, { sign: '+', n: 3, unit: 'Science' }));
  const amt = container.querySelector('.amt')!;
  expect(amt.querySelector('.amt-n')!.textContent).toBe('+3');
  expect(amt.querySelector('.amt-icon')).toBeTruthy();
  expect(amt.getAttribute('title')).toBe('Science');
  // Copy, search and screen readers still get "+3 Science".
  expect(container.textContent).toBe('+3 Science');
});

it('prose turns its amounts into icons and leaves the rest alone', () => {
  render(createElement(Amounts, { text: 'Upgrade from Laboratory; Population 5 required; +4 Science. Science victory needs 10 VP.' }));
  expect(container.querySelectorAll('.amt')).toHaveLength(2);
  expect([...container.querySelectorAll('.amt')].map((a) => a.getAttribute('data-amount'))).toEqual(['Science', 'VP']);
  expect(container.textContent).toBe('Upgrade from Laboratory; Population 5 required; +4 Science. Science victory needs 10 VP.');
});

it('a minus sign is a real minus and "/turn" stays text', () => {
  render(createElement(Amounts, { text: 'Upkeep -1 Gold/turn' }));
  expect(container.querySelector('.amt-n')!.textContent).toBe('−1');
  expect(container.textContent).toBe('Upkeep −1 Gold/turn');
});

it('structured yields render as amounts, in yield order', () => {
  render(createElement(YieldAmounts, { yields: { food: -2, production: 6, gold: 0, science: 0 } }));
  expect(container.textContent).toBe('−2 Food, +6 Production');
  expect(container.querySelectorAll('.amt')).toHaveLength(2);
  render(createElement(YieldAmounts, { yields: {}, none: 'no yield' }));
  expect(container.textContent).toBe('no yield');
});

it('a quantity takes its sign from the number', () => {
  render(createElement(Quantity, { value: -5, unit: 'Gold' }));
  expect(container.textContent).toBe('−5 Gold');
  render(createElement(Quantity, { value: 5, unit: 'Gold', plus: true }));
  expect(container.textContent).toBe('+5 Gold');
  render(createElement(Quantity, { value: 5, unit: 'Gold' }));
  expect(container.textContent).toBe('5 Gold');
});
