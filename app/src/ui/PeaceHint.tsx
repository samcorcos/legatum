import { usePlacedCard, type CardAnchor } from './usePlacedCard';

/** Hover hint over a unit or city of a team at Peace (or allied) with a military unit selected (G2). */
export function PeaceHint({ anchor, team, mouse, allied = false }: { anchor: CardAnchor; team: string; mouse: boolean; allied?: boolean }) {
  const placed = usePlacedCard<HTMLDivElement>(anchor);
  return <div className="forecast peace-hint" role="status" ref={placed.ref} style={placed.style}>
    {!(allied) ? <><b>At Peace with {team}.</b> Declare War to attack.</> : <><b>Allied with {team}.</b> Declaring War on an ally is a betrayal.</>}
    <div className="forecast-foot">{!(mouse) ? 'Tap again to decide.' : 'Right-click to decide.'}</div>
  </div>;
}
