// @vitest-environment happy-dom
import { expect, it } from 'vitest';
import { revealBelowStickyHead, revealIdleProductionPicker } from './cityPanelScroll';

function rect(top: number, height: number): DOMRect {
  return { x: 0, y: top, top, bottom: top + height, left: 0, right: 300, width: 300, height, toJSON: () => ({}) } as DOMRect;
}

function picker(panelBottom: number) {
  const scroller = document.createElement('div');
  Object.defineProperty(scroller, 'getBoundingClientRect', { value: () => rect(100, panelBottom - 100) });
  Object.defineProperty(scroller, 'scrollTop', { value: 0, writable: true });
  const head = document.createElement('div');
  head.className = 'cp-head';
  Object.defineProperty(head, 'getBoundingClientRect', { value: () => rect(100, 50) });
  scroller.append(head);

  const queue = document.createElement('section');
  queue.className = 'production-queue';
  Object.defineProperty(queue, 'getBoundingClientRect', { value: () => rect(270 - scroller.scrollTop, 30) });
  scroller.append(queue);

  const title = document.createElement('div');
  title.className = 'cp-choose-title';
  Object.defineProperty(title, 'getBoundingClientRect', { value: () => rect(310 - scroller.scrollTop, 28) });
  scroller.append(title);

  for (let i = 0; i < 6; i++) {
    const card = document.createElement('div');
    card.className = 'production-choice';
    Object.defineProperty(card, 'getBoundingClientRect', { value: () => rect(346 + i * 108 - scroller.scrollTop, 100) });
    scroller.append(card);
  }
  return { scroller, head, title };
}

it('scrolls a short city panel to its heading and exposes three full build cards below the sticky head', () => {
  const { scroller, head, title } = picker(600);
  const stickyHeadBottom = head.getBoundingClientRect().bottom;

  expect(revealIdleProductionPicker(scroller)).toBe(true);

  const titleBox = title.getBoundingClientRect();
  const panelBottom = scroller.getBoundingClientRect().bottom;
  const visibleCards = [...scroller.querySelectorAll<HTMLElement>('.production-choice')]
    .filter((card) => card.getBoundingClientRect().top >= stickyHeadBottom && card.getBoundingClientRect().bottom <= panelBottom);
  expect(titleBox.top).toBeGreaterThanOrEqual(stickyHeadBottom);
  expect(visibleCards).toHaveLength(3);
  expect(head.getBoundingClientRect().bottom).toBe(stickyHeadBottom);
});

it('leaves a tall panel at its current scroll position when three build cards already fit', () => {
  const { scroller } = picker(900);
  expect(revealIdleProductionPicker(scroller)).toBe(false);
  expect(scroller.scrollTop).toBe(0);
});

it("keeps a damaged city's healing line just under the sticky head instead of scrolling it away", () => {
  const { scroller, head } = picker(600);
  const status = document.createElement('div');
  status.className = 'cp-healing-wrap';
  Object.defineProperty(status, 'getBoundingClientRect', { value: () => rect(160 - scroller.scrollTop, 60) });
  head.after(status);

  revealIdleProductionPicker(scroller);

  expect(status.getBoundingClientRect().top).toBe(head.getBoundingClientRect().bottom);
  expect(status.getBoundingClientRect().bottom).toBeLessThanOrEqual(scroller.getBoundingClientRect().bottom);
});

it('keeps the red production warning whole below the sticky head', () => {
  const { scroller, head } = picker(600);
  const warning = document.createElement('div');
  warning.className = 'production-warning';
  Object.defineProperty(warning, 'getBoundingClientRect', { value: () => rect(230 - scroller.scrollTop, 36) });
  head.after(warning);

  revealIdleProductionPicker(scroller);
  expect(warning.getBoundingClientRect().top).toBeGreaterThanOrEqual(head.getBoundingClientRect().bottom);

  // Scrolled past it (the build list in view when the warning appears): scroll back just enough.
  scroller.scrollTop = 120;
  expect(revealBelowStickyHead(scroller, warning)).toBe(true);
  expect(warning.getBoundingClientRect().top).toBe(head.getBoundingClientRect().bottom);
  expect(revealBelowStickyHead(scroller, warning)).toBe(false);
});
