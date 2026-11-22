import { useId, useLayoutEffect, useRef } from 'react';
import { useGame } from '../store';
import { cancelWarPrompt, declareWarAndAttack, warDeclaration } from '../commands';
import { relationWith } from '../diplomacy';
import { CostChip, diplomacySpends, ForYouLine } from './CostChip';
import { Kbd } from './bits';
import { useCoarsePointer } from './useCoarsePointer';
import { empireMembers, empireTitle } from '../empires';
import { FACTION_INFO } from '../data';
import { playerColor } from '../theme';
import { PortraitFrame, Ribbon } from './premium';
import { EmpireMark } from './EmpireIdentity';
import { itemName } from '../data';
import { distance, hexEq } from '../hex';
import { cityTileDefender, outriderBlocks } from '../combat';
import { closeTips } from './Tip';
import { allianceTerms } from '../treaties';
import { warLines } from './DiplomacyPanel';
import { Amount } from './Amounts';
import './EndTurnConfirmation.css';

/**
 * Civ-style surprise-war prompt (playtest G2): ordering a military unit to attack (moving onto, or
 * firing on) a unit or city of a team at Peace asks first, instead of an order that silently fails.
 * Return confirms (the primary has focus); Esc or D cancels and drops that attack.
 */
export function DeclareWarPrompt() {
  const prompt = useGame((s) => s.warPrompt);
  const views = useGame((s) => s.views);
  const dialog = useRef<HTMLDialogElement>(null);
  const first = useRef<HTMLButtonElement>(null);
  const chipId = useId();
  const touch = useCoarsePointer();
  const title = useId();
  const description = useId();
  const current = !!prompt && views?.[prompt.seat] === prompt.view && !prompt.view.finished;
  const close = cancelWarPrompt;
  useLayoutEffect(() => {
    const el = dialog.current;
    if (!current || !el) return;
    const opener = document.activeElement instanceof HTMLElement ? document.activeElement : null;
    closeTips();
    el.showModal();
    first.current?.focus({ preventScroll: true });
    return () => { el.close(); if (opener?.isConnected) opener.focus({ preventScroll: true }); };
  }, [current, prompt]);
  if (!current || !prompt) return null;
  const view = prompt.view;
  const faction = (owner: number) => view.players.find((p) => p.id === owner)?.faction;
  const unit = view.units.find((u) => u.id === prompt.unit);
  // An aircraft lives in a hangar, not on the map: its strike is a ranged attack from its city.
  const plane = prompt.air ? (view.aircraft ?? []).find((a) => a.id === prompt.unit) : undefined;
  const unitName = plane ? itemName({ kind: 'Unit', unit: plane.kind }, faction(plane.owner)) : unit ? itemName({ kind: 'Unit', unit: unit.kind }, faction(unit.owner)) : 'unit';
  const fires = plane ? 'strike' : 'fire on';
  const target = prompt.target;
  const ranged = !!prompt.ranged;
  const city = target ? view.cities.find((c) => hexEq(c.pos, target)) : undefined;
  // A city with HP takes the attack and shields its garrison (core combat::city_tile_defender);
  // elsewhere a ranged attack hits a military unit. It never chases.
  const targetUnit = target && !(city && cityTileDefender(city.hp, true) === 'City')
    ? view.units.filter((u) => hexEq(u.pos, target) && u.owner !== view.me && (!ranged || u.military))
      .sort((a, b) => Number(b.military) - Number(a.military))[0] : undefined;
  const targetName = targetUnit ? itemName({ kind: 'Unit', unit: targetUnit.kind }, faction(targetUnit.owner)) : city?.name ?? 'target';
  const outriderEscapes = !!unit && !!targetUnit && !ranged
    && outriderBlocks(unit.kind, targetUnit.kind, distance(unit.pos, targetUnit.pos));
  const pursuitText = targetUnit
    ? outriderEscapes
      ? ` If the ${targetName} moves, it gets away.`
      : ` If the ${targetName} moves, your ${unitName} pursues it.`
    : '';
  const attacking = (!!unit || !!plane) && !!target;
  const team = empireTitle(view, prompt.team);
  const war = warDeclaration(view, prompt.team);
  const relation = relationWith(view, prompt.team);
  // G16: what declaring spends, on the button: the Gold for breaking an Alliance early.
  const spends = 'action' in war ? diplomacySpends(war.action, view) : [];
  const allied = relation?.state === 'Alliance';
  const standing = allied ? `You're allied with ${team}` : `You're at ${relation?.state ?? 'Peace'} with ${team}`;
  const attacksNow = attacking && 'action' in war && (!ranged || war.action.effective_turn === view.turn);
  const members = empireMembers(view, prompt.team).slice(0, 2);
  const names = members.map((m) => FACTION_INFO[m.faction].name);
  // An aircraft strikes; every other unit attacks. The aircraft's target list says the same words ("Declare War and strike").
  const confirmLabel = attacksNow ? (plane ? 'Declare War and strike' : 'Declare War and attack') : 'Declare War';
  return <dialog ref={dialog} className="modal end-turn-confirmation war-prompt" aria-labelledby={title} aria-describedby={description}
    onCancel={(e) => { e.preventDefault(); close(); }}
    // G19: D (like Esc) cancels. Return presses the focused primary, as every decision card.
    onKeyDown={(e) => { if (e.key.toLowerCase() === 'd' && !e.repeat && !e.metaKey && !e.ctrlKey && !e.altKey && !e.shiftKey) { e.preventDefault(); close(); } }}>
    {/* Decision card (as the diplomacy inbox): who, one sentence, what it costs you, two buttons. */}
    <div className="war-prompt-grid">
    <div className={`war-prompt-hero${members.length > 1 ? ' pair' : ''}`} aria-hidden="true">
      <div className="war-prompt-portraits">
        {members.map((m) => <PortraitFrame key={m.id} faction={m.faction} crestSize={members.length > 1 ? 26 : 32} crestColor={playerColor(m.id)} />)}
      </div>
      <Ribbon tone="red">{names.join(' & ')}</Ribbon>
    </div>
    <div className="war-prompt-main">
    <h2 id={title} className="war-prompt-head"><EmpireMark view={view} team={prompt.team} size={30} /><span>Declare War on {team}?</span></h2>
    <p id={description}>{attacking ? <>{standing}, so your {unitName} can't {ranged ? fires : 'attack'} their {targetName} yet.</>
      : <>{standing}.{city ? ` Declaring War lets you attack ${city.name} and their other cities and units.` : ''}</>}</p>
    {'action' in war ? (
      <div className="war-prompt-for-you"><span className="war-prompt-label">For you</span><ul className="for-you-list">
        {!attacking
          ? <ForYouLine tone="note">{war.action.effective_turn === view.turn ? 'War starts now: attacks you order this turn resolve this turn.' : 'War begins next turn.'}</ForYouLine>
          : ranged
          ? war.action.effective_turn === view.turn
            ? <ForYouLine tone="note">War starts now. Your {unitName} {plane ? 'strikes' : 'fires on'} their {targetName} this turn.</ForYouLine>
            : <ForYouLine tone="note">War begins next turn. A ranged attack can't wait for it: order your {unitName} to {plane ? 'strike' : 'fire'} then.</ForYouLine>
          : war.action.effective_turn === view.turn
          ? <ForYouLine tone="note">War starts now. Your {unitName} attacks this turn.{pursuitText}</ForYouLine>
          : <ForYouLine tone="note">War begins next turn; your {unitName} will attack then.{pursuitText}</ForYouLine>}
        {war.action.cost > 0 && <ForYouLine tone="cost">Breaks your Alliance inside its protected window: <Amount n={war.action.cost} unit="Gold" />.</ForYouLine>}
        {war.action.betrayal && <ForYouLine tone="cost">Betrayal: {team} won't ally with you for {allianceTerms().betrayalTurns} turns.</ForYouLine>}
        {allied && <ForYouLine tone="note">The Alliance ends: no more shared sight or open borders.</ForYouLine>}
        {warLines(war.action, view).map((l) => <ForYouLine key={l.text} tone={l.tone}>{l.text}</ForYouLine>)}
        {!allied && war.action.cost === 0 && <ForYouLine tone="note">Costs no Gold.</ForYouLine>}
      </ul></div>
    ) : null}
    </div></div>
    <div className="end-turn-actions">
      {'action' in war ? <>
        <button ref={first} className={`btn primary${spends.length ? ' spends' : ''}`} aria-label={confirmLabel} aria-describedby={spends.length ? chipId : undefined}
          onClick={() => declareWarAndAttack(prompt.unit, prompt.target, prompt.team, ranged, !!plane)}>
          <span>{confirmLabel}{!touch && <> <Kbd>Return</Kbd></>}</span>
          <CostChip id={chipId} spends={spends} />
        </button>
        <button className="btn" aria-label="Cancel" onClick={close}>Cancel{!touch && <> <Kbd>Esc</Kbd></>}</button>
      </> : <>
        <div className="war-prompt-disabled"><button className="btn primary" disabled>Declare War</button><p className="muted">{war.blocked}</p></div>
        <button ref={first} className="btn" onClick={close}>Keep the Peace</button>
      </>}
    </div>
  </dialog>;
}
