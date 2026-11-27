import { useGame } from '../store';
import { cancelWarPrompt, declareWarAndAttack, warDeclaration } from '../commands';
import { itemName } from '../data';
import { distance, hexEq } from '../hex';
import { cityTileDefender, outriderBlocks } from '../combat';
import { WarPlate } from '../celebrate/PeaceTalks';

/** The attack-triggered declaration keeps its planned attack when War can start this turn. */
export function DeclareWarPrompt() {
  const prompt = useGame((s) => s.warPrompt);
  const views = useGame((s) => s.views);
  if (!prompt || views?.[prompt.seat] !== prompt.view || prompt.view.finished) return null;
  const view = prompt.view;
  const unit = view.units.find((u) => u.id === prompt.unit);
  const plane = prompt.air ? (view.aircraft ?? []).find((a) => a.id === prompt.unit) : undefined;
  const faction = (owner: number) => view.players.find((p) => p.id === owner)?.faction;
  const unitName = plane ? itemName({ kind: 'Unit', unit: plane.kind }, faction(plane.owner))
    : unit ? itemName({ kind: 'Unit', unit: unit.kind }, faction(unit.owner)) : 'unit';
  const city = prompt.target ? view.cities.find((c) => hexEq(c.pos, prompt.target!)) : undefined;
  const targetUnit = prompt.target && !(city && cityTileDefender(city.hp, true) === 'City')
    ? view.units.find((u) => hexEq(u.pos, prompt.target!) && u.owner !== view.me && (!prompt.ranged || u.military)) : undefined;
  const targetName = targetUnit ? itemName({ kind: 'Unit', unit: targetUnit.kind }, faction(targetUnit.owner)) : city?.name ?? 'target';
  const pursuit = unit && targetUnit && !prompt.ranged
    ? outriderBlocks(unit.kind, targetUnit.kind, distance(unit.pos, targetUnit.pos))
      ? ` If the ${targetName} moves, it gets away.` : ` If the ${targetName} moves, your ${unitName} pursues it.`
    : '';
  const war = warDeclaration(view, prompt.team);
  const now = 'action' in war && war.action.effective_turn === view.turn;
  const attackText = prompt.target && (unit || plane)
    ? now ? `War starts now. Your ${unitName} ${plane ? 'strikes' : prompt.ranged ? 'fires on' : 'attacks'} their ${targetName} this turn.${pursuit}`
      : prompt.ranged ? `War begins next turn. Order your ${unitName} to ${plane ? 'strike' : 'fire'} then.`
        : `War begins next turn; your ${unitName} attacks then.${pursuit}`
    : undefined;
  const confirmLabel = prompt.target && (unit || plane) && (!prompt.ranged || now)
    ? plane ? 'Declare War and strike' : 'Declare War and attack' : 'Declare War';
  return <WarPlate view={view} team={prompt.team} attackText={attackText} confirmLabel={confirmLabel}
    onClose={cancelWarPrompt} onConfirm={() => declareWarAndAttack(prompt.unit, prompt.target, prompt.team, !!prompt.ranged, !!prompt.air)} />;
}
