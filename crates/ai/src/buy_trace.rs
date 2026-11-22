//! Diagnostics only: which planning pass ordered each purchase. Off unless a measurement
//! (`crates/sim/examples/raze_buy_census.rs`) turns it on for its thread; it never changes an
//! order. Each pass calls `note` after it may have bought, and the first pass to list a
//! purchase owns it.
use legatum_core::{CityId, Order, PlayerId, ProductionItem};
use std::cell::RefCell;

/// One purchase and the pass that ordered it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BuyTag {
    pub player: PlayerId,
    pub city: CityId,
    pub item: ProductionItem,
    pub pass: &'static str,
    pub floor_rescue: bool,
}

thread_local! {
    static LOG: RefCell<Option<Vec<BuyTag>>> = const { RefCell::new(None) };
}

/// Start recording this thread's purchases.
pub fn enable() {
    LOG.with(|l| *l.borrow_mut() = Some(Vec::new()));
}

/// The purchases recorded since the last call (empty while off).
pub fn take() -> Vec<BuyTag> {
    LOG.with(|l| {
        l.borrow_mut()
            .as_mut()
            .map(std::mem::take)
            .unwrap_or_default()
    })
}

/// Tag every purchase in `orders` that no earlier pass has claimed.
pub(crate) fn note(player: PlayerId, orders: &[Order], pass: &'static str) {
    LOG.with(|l| {
        let mut l = l.borrow_mut();
        let Some(log) = l.as_mut() else { return };
        for o in orders {
            if let Order::BuyItem { city, item } = o
                && !log
                    .iter()
                    .any(|t| t.player == player && t.city == *city && t.item == *item)
            {
                log.push(BuyTag {
                    player,
                    city: *city,
                    item: *item,
                    pass,
                    floor_rescue: false,
                });
            }
        }
    });
}

/// Mark the final purchase as an empty-city rescue that passed the generation floor.
pub(crate) fn mark_floor_rescue(player: PlayerId, city: CityId, item: ProductionItem) {
    LOG.with(|l| {
        if let Some(tag) = l.borrow_mut().as_mut().and_then(|log| {
            log.iter_mut()
                .find(|tag| tag.player == player && tag.city == city && tag.item == item)
        }) {
            tag.floor_rescue = true;
        }
    });
}

/// Preserve the originating buy pass when a final legality floor changes its unit.
pub(crate) fn replace_item(
    player: PlayerId,
    city: CityId,
    old: ProductionItem,
    new: ProductionItem,
) {
    LOG.with(|l| {
        if let Some(tag) = l.borrow_mut().as_mut().and_then(|log| {
            log.iter_mut()
                .find(|tag| tag.player == player && tag.city == city && tag.item == old)
        }) {
            tag.item = new;
        }
    });
}
