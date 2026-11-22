// A TI4-style faction picker: a full-screen gallery of crests plus a detail sheet for the chosen
// faction (lore, playstyle pips, strengths paired with their price, the painted cards of its special
// unit and buildings, its goal in one block, and its counter). The click is the selection: clicking a tile (or arrowing to it) makes
// it the seat's faction at once and shows its sheet. There is no hover preview and no confirm
// button; Close (or Return, or Esc) just closes the dialog and the choice stays.
import { useEffect, useId, useRef, useState } from 'react';
import { FACTIONS, FACTION_GOAL, FACTION_INFO, factionGoalTerms, factionGoalVp, type FactionGoal } from '../data';
import { FACTION_SHEETS, sheetCards } from '../factionSheets';
import type { FactionChoice } from '../lobby';
import { Icon, Kbd } from './bits';
import { Amounts } from './Amounts';
import { MedallionRow, PieceCard, PortraitFrame, PortraitRoundel, StarDivider } from './premium';
import { useCoarsePointer } from './useCoarsePointer';
import './FactionPicker.css';

function Pips({ label, value, max = 5 }: { label: string; value: number; max?: number }) {
  return (
    <span className="fs-pips" aria-label={`${label} ${value} of ${max}`}>
      <span className="fs-pips-label">{label}</span>
      <span className="fs-pips-dots" aria-hidden="true">
        {Array.from({ length: max }, (_, i) => (
          <span key={i} className={`fs-pip ${i < value ? 'on' : ''}`} />
        ))}
      </span>
    </span>
  );
}

export function FactionPicker({
  value,
  onChange,
  label,
  disabled,
}: {
  value: FactionChoice;
  onChange: (f: FactionChoice) => void;
  label: string;
  disabled?: boolean;
}) {
  const [open, setOpen] = useState(false);
  const touch = useCoarsePointer();
  const choices: FactionChoice[] = ['Random', ...FACTIONS];
  const titleId = useId();
  const sheetId = useId();
  const dialog = useRef<HTMLDialogElement>(null);
  const trigger = useRef<HTMLButtonElement>(null);
  const tiles = useRef<(HTMLButtonElement | null)[]>([]);
  const current = choices.indexOf(value);

  useEffect(() => {
    const el = dialog.current;
    if (!el) return;
    if (open && !el.open) {
      el.showModal();
      tiles.current[current]?.focus({ preventScroll: true });
      tiles.current[current]?.scrollIntoView({ block: 'nearest' });
    } else if (!open && el.open) el.close();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [open]);

  const close = () => {
    setOpen(false);
    trigger.current?.focus();
  };
  // Arrow keys move the selection: the next faction becomes the seat's choice, same as a click.
  const move = (to: number) => {
    const next = (to + choices.length) % choices.length;
    onChange(choices[next]);
    tiles.current[next]?.focus({ preventScroll: true });
    tiles.current[next]?.scrollIntoView({ block: 'nearest' });
  };

  const shown = value;
  const shownName = shown === 'Random' ? 'Random' : FACTION_INFO[shown].name;

  return (
    <div className="faction-picker">
      <button
        ref={trigger}
        className="lobby-select faction-current"
        aria-haspopup="dialog"
        aria-label={`${label}: ${value === 'Random' ? value : FACTION_INFO[value].name}`}
        disabled={disabled}
        onClick={() => setOpen(true)}
      >
        <PortraitRoundel faction={value} size={48} />
        <span className="faction-current-text">
          <span className="faction-current-name">{value === 'Random' ? value : FACTION_INFO[value].name}</span>
          <span className="faction-current-identity">{value === 'Random' ? 'Decided by the map seed' : FACTION_SHEETS[value].identity}</span>
        </span>
      </button>
      <dialog
        ref={dialog}
        className="faction-stage"
        aria-labelledby={titleId}
        data-testid="faction-stage"
        onCancel={(e) => { e.preventDefault(); close(); }}
        onClose={close}
        // A click on the dimmed backdrop lands on the dialog element itself: it closes, keeping the choice.
        onClick={(e) => { if (e.target === e.currentTarget) close(); }}
        onKeyDown={(e) => {
          e.stopPropagation();
          if (e.key === 'Tab') {
            const stops = e.currentTarget.querySelectorAll<HTMLElement>('button:not(:disabled):not([tabindex="-1"])');
            const first = stops[0], last = stops[stops.length - 1];
            if (e.shiftKey && document.activeElement === first) { e.preventDefault(); last?.focus(); }
            else if (!e.shiftKey && document.activeElement === last) { e.preventDefault(); first?.focus(); }
          } else if (['ArrowDown', 'ArrowRight', 'ArrowUp', 'ArrowLeft', 'Home', 'End'].includes(e.key)) {
            e.preventDefault();
            if (e.key === 'Home') move(0);
            else if (e.key === 'End') move(choices.length - 1);
            else move(current + (e.key === 'ArrowDown' || e.key === 'ArrowRight' ? 1 : -1));
          } else if (e.key === 'Enter') {
            e.preventDefault();
            close();
          }
        }}
      >
        <header className="fs-head">
          <h1 id={titleId}>Choose your empire</h1>
          <span className="fs-chosen" role="status" data-testid="faction-chosen">
            <Icon name="ui-check" size={14} className="ui-mark" />
            <span>Chosen: <strong>{shownName}</strong></span>
          </span>
          <button className="btn primary fs-close" onClick={close} aria-label="Close faction picker">
            Close {!touch && <Kbd>Return</Kbd>}
          </button>
        </header>
        <div className="fs-body">
          <div className="fs-gallery" role="listbox" aria-label={label}>
            {choices.map((f, i) => (
              <button
                key={f}
                ref={(el) => { tiles.current[i] = el; }}
                role="option"
                data-faction={f}
                tabIndex={current === i ? 0 : -1}
                aria-selected={current === i}
                aria-describedby={current === i ? sheetId : undefined}
                className={`fs-tile ${current === i ? 'on' : ''}`}
                // The click is the selection. Focus is set by hand because Safari does not focus a
                // button on click, and the arrow keys need to start from the clicked tile.
                onClick={(e) => {
                  onChange(f);
                  e.currentTarget.focus({ preventScroll: true });
                }}
              >
                <PortraitRoundel faction={f} size={32} className="fs-thumb" />
                <span className="fs-tile-text">
                  <span className="fs-tile-name">{f === 'Random' ? 'Random' : FACTION_INFO[f].name}{f !== 'Random' && FACTION_SHEETS[f].early && <span className="fs-early-tag">Early design</span>}</span>
                  <span className="fs-tile-identity">{f === 'Random' ? 'Decided by the map seed' : FACTION_SHEETS[f].identity}</span>
                </span>
                {current === i && <Icon name="ui-check" size={14} className="ui-mark fs-tile-check" />}
              </button>
            ))}
          </div>
          {shown === 'Random' ? (
            <div className="fs-sheet paper-grain" id={sheetId} data-testid="faction-sheet">
              <div className="fs-hero">
                <PortraitFrame faction={null} className="fs-portrait fs-portrait-random">
                  <Icon name="ui-random" size={72} className="random-mark" />
                </PortraitFrame>
                <div className="fs-hero-text">
                  <div className="fs-sheet-name">Random</div>
                  <div className="fs-sheet-motto">Decided by the map seed</div>
                  <StarDivider />
                  <p className="fs-lore">A faction picked from the map seed when the game begins.</p>
                </div>
              </div>
              <StarDivider className="fs-random-divider" />
              <section className="fs-random-candidates" aria-label="Possible factions">
                <div className="fs-random-label">Could be any of these</div>
                <div className="fs-random-seals">
                  {FACTIONS.map((f) => <span key={f} className="fs-random-seal">
                    <PortraitRoundel faction={f} size={52} />
                    <span className="fs-random-seal-text">
                      <b>{FACTION_INFO[f].name}</b>
                      <small>{FACTION_SHEETS[f].identity}</small>
                    </span>
                  </span>)}
                </div>
                <p>The seed chooses one when the game begins.</p>
              </section>
            </div>
          ) : (
            <FactionSheetView faction={shown} sheetId={sheetId} />
          )}
        </div>
      </dialog>
    </div>
  );
}

const FACT_ICONS = { lose: 'ui-trend-down', counter: 'ui-alert' } as const;

/** Where in the game a goal is acted on (only goals with an action of their own). */
const goalWhere = (faction: Exclude<FactionChoice, 'Random'>, goal: FactionGoal): string | null => {
  if (faction === 'Sparta') return 'In the game: Retire this veteran on its unit card, or use the race sidebar (G).';
  return goal.action ? `In the game: ${goal.action} in the race sidebar (G).` : null;
};

export function FactionSheetView({
  faction,
  sheetId,
}: {
  faction: Exclude<FactionChoice, 'Random'>;
  sheetId: string;
}) {
  const info = FACTION_INFO[faction];
  const sheet = FACTION_SHEETS[faction];
  const cards = sheetCards(faction);
  // A faction with no goal yet (an early design) says so in the goal block's place.
  const goal: FactionGoal | undefined = factionGoalVp(faction) > 0 ? FACTION_GOAL[faction] : undefined;
  const where = goal && goalWhere(faction, goal);
  return (
    <div className="fs-sheet paper-grain" id={sheetId} data-testid="faction-sheet">
      <div className="fs-hero">
        <PortraitFrame faction={faction} className="fs-portrait" />
        <div className="fs-hero-text">
          <div className="fs-sheet-name">{info.name}{sheet.early && <span className="fs-early-tag" data-testid="fs-early">Early design</span>}</div>
          <p className="fs-lore">{sheet.lore}</p>
          <div className="fs-ratings">
            <Pips label="Aggression" value={sheet.ratings.aggression} />
            <Pips label="Economy" value={sheet.ratings.economy} />
            <Pips label="Complexity" value={sheet.ratings.complexity} />
          </div>
        </div>
      </div>
      <div className="fs-pairs">
        <div className="fs-col" data-testid="fs-strengths">
          {sheet.pairs.filter((p) => !p.onCard).map((p, i) => p.strength && p.strengthIcon && <MedallionRow key={i} icon={p.strengthIcon} tone="good" sign="Strength" className="fs-trait">{p.strength}</MedallionRow>)}
        </div>
        <div className="fs-col" data-testid="fs-prices">
          {sheet.pairs.filter((p) => !p.onCard).map((p, i) => p.price && p.priceIcon && <MedallionRow key={i} icon={p.priceIcon} tone="bad" sign="Price" className="fs-trait">{p.price}</MedallionRow>)}
        </div>
      </div>
      <div className={`fs-lower${cards.length === 1 ? ' one' : ''}`}>
        <div className="fs-cards" data-count={cards.length}>
          {cards.map((card) => <PieceCard
            key={card.name}
            faction={faction}
            piece={card.item}
            name={card.name}
            factionName={info.name}
            replaces={card.replaces}
            text={card.text}
          />)}
        </div>
        <div className="fs-facts">
          {goal && (
            <div className="fs-fact fs-goal">
              <span className="lbl"><Icon name={goal.icon} size={13} />Goal</span>
              <div className="fs-goal-head"><b>{goal.name}</b><span>{factionGoalTerms(faction)}</span></div>
              <p><Amounts text={`${goal.detail()}.`} /></p>
              {where && <p className="fs-goal-location">{where}</p>}
            </div>
          )}
          {!goal && sheet.early && (
            <div className="fs-fact fs-goal fs-goal-early" data-testid="fs-goal-early">
              <span className="lbl"><Icon name="ui-alert" size={13} />Goal</span>
              <p>Early design: no faction goal yet.</p>
            </div>
          )}
          {(sheet.counteredBy || sheet.lose) && (
            <div className="fs-facts-col">
              {sheet.lose && (
                <div className="fs-fact">
                  <span className="lbl"><Icon name={FACT_ICONS.lose} size={13} />How you lose</span>
                  <p><Amounts text={sheet.lose} /></p>
                </div>
              )}
              {sheet.counteredBy && (
                <div className="fs-fact">
                  <span className="lbl"><Icon name={FACT_ICONS.counter} size={13} />Countered by</span>
                  <p><Amounts text={sheet.counteredBy} /></p>
                </div>
              )}
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
