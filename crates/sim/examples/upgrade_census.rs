//! Why an empire's army is old: replay a tape to a turn and print, for each player, its era, Gold
//! and income, and its combat units by era with what stands in the way of each upgrade.
//! upgrade_census <tape.json> <turn> [<player>...]
use legatum_core::{PlayerId, data, eras, player_view, resolve_turn};
use legatum_wasm::tape::Tape;
use std::collections::BTreeMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let a: Vec<String> = std::env::args().collect();
    let tape = Tape::from_json(&std::fs::read_to_string(&a[1])?)?;
    let turn: u32 = a[2].parse()?;
    let only: Vec<u8> = a[3..].iter().map(|s| s.parse().unwrap()).collect();
    let mut state = tape.initial_state();
    for t in tape.turns.iter().filter(|t| t.turn < turn) {
        state = resolve_turn(&state, &t.orders).0;
    }
    for p in 0..state.players.len() as u8 {
        if !only.is_empty() && !only.contains(&p) {
            continue;
        }
        let me = PlayerId(p);
        if !state.cities.values().any(|c| c.owner == me) {
            continue;
        }
        let v = player_view(&state, me);
        let era = eras::era_of(&state, me);
        println!(
            "P{p} {:?} era {era} Gold {} income {} cities {}",
            state.player(me).faction,
            v.economy.gold,
            v.economy.gold_per_turn.total,
            v.cities.iter().filter(|c| c.owner == me).count()
        );
        let mut by_era: BTreeMap<u8, usize> = BTreeMap::new();
        let mut reasons: BTreeMap<String, usize> = BTreeMap::new();
        let mut kinds: BTreeMap<String, usize> = BTreeMap::new();
        for u in v.units.iter().filter(|u| u.owner == me && u.military) {
            let tier = data::unit_era_tier(u.kind);
            *by_era.entry(tier).or_default() += 1;
            *kinds.entry(format!("{:?}(e{tier})", u.kind)).or_default() += 1;
            let home = v.tile(u.pos).is_some_and(|t| t.owner == Some(me));
            let reason = match (&u.upgrade_to, u.upgrade_cost) {
                (None, _) => "no researched upgrade".to_string(),
                (Some(_), Some(_)) => "upgradable now".to_string(),
                (Some(to), None) => {
                    let total = u.upgrade_breakdown.as_ref().map_or(0, |b| b.total);
                    if let Some(b) = &u.upgrade_supply_blocker {
                        format!("supply: {b}")
                    } else if !home {
                        format!("outside territory (to {to:?})")
                    } else if total > v.economy.gold {
                        format!("Gold short (to {to:?}, {total})")
                    } else {
                        "other".into()
                    }
                }
            };
            *reasons.entry(reason).or_default() += 1;
        }
        let supply: Vec<String> = v
            .supply
            .iter()
            .filter(|s| s.revealed && (s.supplied.total > 0 || s.used.total > 0))
            .map(|s| format!("{:?} {}/{}", s.resource, s.used.total, s.supplied.total))
            .collect();
        println!("  supply used/has {supply:?}");
        let mut trainable: BTreeMap<String, usize> = BTreeMap::new();
        for c in v.cities.iter().filter(|c| c.owner == me) {
            for o in c.detail.iter().flat_map(|d| &d.options) {
                if let legatum_core::ProductionItem::Unit { unit } = o.item
                    && data::unit_class(unit).is_some()
                {
                    let tag = if o.resource_blocker.is_some() {
                        "blocked"
                    } else {
                        "ok"
                    };
                    *trainable.entry(format!("{unit:?}:{tag}")).or_default() += 1;
                }
            }
        }
        println!("  trainable (cities) {trainable:?}");
        println!("  units by era {by_era:?}");
        println!("  kinds {kinds:?}");
        for (r, n) in reasons {
            println!("  {n:3} {r}");
        }
    }
    Ok(())
}
