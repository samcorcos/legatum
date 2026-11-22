import type { KeyboardEvent, MouseEvent } from 'react';
import type { Resource } from '../types';
import { Amounts } from './Amounts';

/** Open the matching top-bar breakdown from a resource refusal. */
export function openResourceChip(resource: Resource) {
  const chip = document.querySelector<HTMLElement>(`.topbar .supply-${resource.toLowerCase()}`);
  chip?.scrollIntoView({ block: 'nearest', inline: 'nearest' });
  chip?.focus();
  chip?.click();
}

/** A refusal's resource word leads to its supply breakdown. `inButton` keeps build cards valid HTML. */
export function ResourceReason({ text, resource, inButton = false }: { text: string; resource: Resource; inButton?: boolean }) {
  // Link a bare mention of the resource, never the unit of an amount ("1 Copper" keeps its icon).
  const at = [...text.matchAll(new RegExp(`\\b${resource}\\b`, 'g'))].map((m) => m.index!).find((i) => !/\d\s$/.test(text.slice(0, i))) ?? -1;
  if (at < 0) return <Amounts text={text} />;
  const open = (event: MouseEvent) => { event.stopPropagation(); openResourceChip(resource); };
  const word = inButton
    ? <span className="resource-reason-link" onClick={open}>{resource}</span>
    : <span role="button" tabIndex={0} className="resource-reason-link" onClick={open}
        onKeyDown={(event: KeyboardEvent) => { if (event.key === 'Enter' || event.key === ' ') { event.preventDefault(); event.stopPropagation(); openResourceChip(resource); } }}
        aria-label={`Open ${resource} supply`}>{resource}</span>;
  return <><Amounts text={text.slice(0, at)} />{word}<Amounts text={text.slice(at + resource.length)} /></>;
}
