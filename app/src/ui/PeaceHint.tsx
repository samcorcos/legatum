import { usePlacedCard, type CardAnchor } from './usePlacedCard';

/** Hover hint over a unit or city of a team at Peace (or allied) with a military unit selected (G2). */
export function PeaceHint({ anchor, team, mouse, allied = false }: { anchor: CardAnchor; team: string; mouse: boolean; allied?: boolean }) {
  const placed = usePlacedCard<HTMLDivElement>(anchor);
  return <div className="forecast peace-hint" role="status" ref={placed.ref} style={placed.style}>
    {allied ? <><b>Allied with {team}.</b> Declaring War on an ally is a betrayal.</> : <><b>At Peace with {team}.</b> Declare War to attack.</>}
    <div className="forecast-foot">{mouse ? 'Right-click to decide.' : 'Tap again to decide.'}</div>
  </div>;
}
