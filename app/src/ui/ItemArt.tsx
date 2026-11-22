import { itemIcon } from '../data';
import { itemArtUrl } from '../unlockArt';
import { unitCardUrl } from '../unitArt';
import type { Faction, ProductionItem } from '../types';
import { Icon } from './index';

/**
 * A production item's picture: its painted miniature (a unit's figure, a building's or improvement's
 * tabletop miniature, `unlockArt.ts`), else the engraved icon while a piece is unpainted (projects).
 * The size is the icon's; the miniature is drawn a little larger so it reads.
 */
export function ItemArt({ item, faction, size = 20 }: { item: ProductionItem; faction?: Faction; size?: number }) {
  const art = item.kind === 'Unit' ? unitCardUrl(item.unit, faction) ?? itemArtUrl(item, faction, 64) : itemArtUrl(item, faction, 64);
  if (!art) return <Icon name={itemIcon(item, faction)} size={size} />;
  return <img className="item-art" data-art={item.kind.toLowerCase()} src={art} alt="" width={size + 10} height={size + 10} draggable={false} />;
}
