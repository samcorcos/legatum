// @vitest-environment happy-dom
// A resource refusal links its resource word to the top-bar chip, but never the unit of an amount:
// "Needs 1 Iron" keeps its number and painted icon, and the link goes on a bare mention.
import { renderToStaticMarkup } from 'react-dom/server';
import { expect, it } from 'vitest';
import { ResourceReason } from './ResourceReason';

const render = (text: string) => {
  const host = document.createElement('div');
  host.innerHTML = renderToStaticMarkup(<ResourceReason text={text} resource="Iron" inButton />);
  return host;
};

it('links a bare mention and leaves "1 Iron" as an amount with its painted icon', () => {
  const host = render('Needs 1 Iron: you have none yet. Bring an Iron deposit inside your borders.');
  const link = host.querySelector('.resource-reason-link')!;
  const amount = host.querySelector('.amt-iron')!;
  expect(amount.querySelector('.amt-n')?.textContent).toBe('1');
  expect(amount.querySelector('.amt-word')?.textContent).toBe(' Iron');
  expect(amount.querySelector('.amt-icon')).not.toBeNull();
  expect(link.textContent).toBe('Iron');
  expect(amount.compareDocumentPosition(link) & Node.DOCUMENT_POSITION_FOLLOWING).not.toBe(0);
  expect(link.closest('.amt')).toBeNull();
  expect(host.textContent).toContain('Bring an Iron deposit');
});

it('links the first mention when it is not an amount', () => {
  const host = render('No Iron to spare: all 2 of your Iron are in use.');
  expect(host.textContent!.indexOf('Iron')).toBe(3);
  expect(host.querySelector('.resource-reason-link')!.textContent).toBe('Iron');
});
