import type { Order } from './types';

// Keep this list in step with core::orders::Order. JSON and WebSocket payloads are untyped.
const ORDER_KINDS = new Set<Order['type']>([
  'DeclareWar', 'BreakTreaty', 'ProposeTreaty', 'RespondToProposal', 'WithdrawProposal',
  'SetAutomation', 'SetAutoFire', 'MoveUnit', 'MoveUnitPath', 'SleepUnit', 'SkipUnit', 'FoundCity',
  'DisbandUnit', 'SetProduction', 'SetProductionQueue', 'SetCityAutomation', 'BuyItem', 'HireCompany', 'BuyTile',
  'SetResearch', 'SetResearchQueue', 'SetScienceConversion', 'RangedAttack', 'AirStrike', 'AirRecon', 'Rebase', 'UpgradeAircraft',
  'UpgradeUnit', 'UpgradeSiege', 'Action', 'GiveGold',
]);

export function isKnownOrderKind(kind: unknown): kind is Order['type'] {
  return typeof kind === 'string' && ORDER_KINDS.has(kind as Order['type']);
}

/** Decode orders received from storage or a server, before they enter application state. */
export function decodeOrders(value: unknown): Order[] {
  if (!Array.isArray(value)) {
    console.warn('Dropped malformed order list', value);
    return [];
  }
  return value.flatMap((raw): Order[] => {
    if (!raw || typeof raw !== 'object' || Array.isArray(raw)) {
      console.warn('Dropped malformed order', raw);
      return [];
    }
    const order = raw as Record<string, unknown>;
    const type = order.type === 'HealUnit' || order.type === 'HoldUnit' ? 'SleepUnit'
      : order.type === 'WakeUnit' ? 'SkipUnit' : order.type;
    if (!isKnownOrderKind(type)) {
      console.warn(`Dropped unknown order kind: ${String(order.type)}`);
      return [];
    }
    return [{ ...order, type } as Order];
  });
}
