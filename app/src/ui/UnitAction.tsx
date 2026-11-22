import { useId, type ReactNode } from 'react';
import type { Faction, UnitKind, UnitView } from '../types';
import { unitIconName } from '../data';
import { unitCardUrl } from '../unitArt';
import { playerColor } from '../theme';
import { Icon, Kbd, Tip } from './index';

/** The unit card's compact order button (shared by the ground-unit card and the aircraft card). */
export function Action({
  label,
  icon,
  hotkey,
  onClick,
  tip,
  summary,
  primary,
  danger,
  quiet,
  active,
  disabled,
  reason,
  describedBy,
  expanded,
  keyShortcuts,
}: {
  label: string;
  icon?: string;
  hotkey?: string;
  onClick: () => void;
  tip?: ReactNode;
  summary?: ReactNode;
  primary?: boolean;
  danger?: boolean;
  quiet?: boolean;
  active?: boolean;
  disabled?: boolean;
  /** Why a disabled action can't be taken, shown right under the button (pillar 9, never hover-only). */
  reason?: ReactNode;
  /** The id of a reason shown elsewhere, when it needs more room than the button's own column. */
  describedBy?: string;
  /** A picker this button opens (Strike, Rebase): whether it is open. */
  expanded?: boolean;
  /** The key that does the same, for assistive tech (`aria-keyshortcuts`). */
  keyShortcuts?: string;
}) {
  const reasonId = useId();
  const why = disabled && reason;
  return (
    <div className={`act-wrap ${primary ? 'primary' : ''} ${quiet ? 'quiet' : ''} ${why ? 'blocked' : ''}`}>
      <Tip content={tip} summary={summary} placement="top">
        <button
          className={`act ${primary ? 'primary' : ''} ${danger ? 'danger' : ''} ${quiet ? 'quiet' : ''} ${active ? 'active' : ''}`}
          onClick={onClick}
          disabled={disabled}
          aria-pressed={active}
          aria-describedby={why ? reasonId : describedBy}
          aria-expanded={expanded}
          aria-keyshortcuts={keyShortcuts}
        >
          {active && !primary && <Icon name="ui-check" size={12} className="action-check" />}{icon && <Icon name={icon} size={14} className="action-icon" />}<span>{label}</span>
          {hotkey && <Kbd>{hotkey}</Kbd>}
        </button>
      </Tip>
      {why && <p className="act-reason" id={reasonId}>{reason}</p>}
    </div>
  );
}

export function UnitBadge({ unit, size = 48, faction }: { unit: Pick<UnitView, 'kind' | 'owner' | 'military'>; size?: number; faction?: Faction }) {
  const civ = !unit.military;
  // The painted miniature stands in the owner's colour; the engraved glyph is the fallback.
  const art = unitCardUrl(unit.kind as UnitKind, faction);
  return (
    <span className={`unit-badge ${unit.military ? '' : 'civ'}${art ? ' painted' : ''}${unit.kind === 'Phalangite' ? ' phalangite' : ''}`} style={{ width: size, height: size, ['--pc' as string]: playerColor(unit.owner) }}>
      {art
        ? <img src={art} alt="" width={size} height={size} draggable={false} />
        : <Icon name={unitIconName(unit.kind as UnitKind, faction)} size={Math.round(size * 0.54)} color={civ ? 'var(--ink)' : 'var(--on-player)'} />}
    </span>
  );
}
