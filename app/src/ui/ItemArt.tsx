import { itemIcon } from '../data';
import { itemArtUrl } from '../unlockArt';
import { unitCardUrl } from '../unitArt';
import type { Faction, ProductionItem } from '../types';
import { Icon } from './index';
import { YIELD_COLORS } from '../theme';

/**
 * A production item's picture: its painted miniature (a unit's figure, a building's or improvement's
 * tabletop miniature, `unlockArt.ts`), else the engraved icon while a piece is unpainted (projects).
 * The size is the icon's; the miniature is drawn a little larger so it reads.
 */
export function ItemArt({ item, faction, size = 20 }: { item: ProductionItem; faction?: Faction; size?: number }) {
  const art = item.kind === 'Unit' ? unitCardUrl(item.unit, faction) ?? itemArtUrl(item, faction, 64) : itemArtUrl(item, faction, 64);
  // A project has no miniature: its yield's icon, in the yield's ink and at the miniature's footprint, so the card doesn't look empty.
  if (!art && item.kind === 'Project') {
    const k = item.project === 'Grow' ? 'food' : item.project === 'Gold' ? 'gold' : 'science';
    return <Icon name={itemIcon(item, faction)} size={size + 6} color={YIELD_COLORS[k]} className="item-art-project" />;
  }
  if (!art) return <Icon name={itemIcon(item, faction)} size={size} />;
  return <img className="item-art" data-art={item.kind.toLowerCase()} src={art} alt="" width={size + 10} height={size + 10} draggable={false} />;
}
