//! Gene `decisive_buy`: the reserve releases for a purchase with decisive value.
use crate::{Brain, ScriptedBrain};
use legatum_core::{AiDifficulty, CityId, Order, PlayerView, ProductionItem, UnitKind};

fn load<T: serde::de::DeserializeOwned>(bytes: &[u8]) -> T {
    serde_json::from_reader(flate2::read::GzDecoder::new(bytes)).unwrap()
}

/// The designer's 8.9.24 Swiss seat at T140 (Flight known one turn, 4,358 Gold, at War with
/// three empires, routed plan Peace), with the seat's Hard memory warmed over T1 to T139.
fn swiss_140() -> (PlayerView, ScriptedBrain) {
    let view: PlayerView = load(include_bytes!("fixtures/swiss-1102/T140-p0.view.json.gz"));
    let seed =
        legatum_core::rng::Rng::derive(2_131_664_801_707_819, &[legatum_core::rng::purpose::AI, 0])
            .next_u64();
    let mut brain =
        ScriptedBrain::with_difficulty(view.faction_of(view.me), seed, view.me, AiDifficulty::Hard);
    let memory: Option<crate::OpponentMemory> =
        load(include_bytes!("fixtures/swiss-1102/T140-p0.memory.json.gz"));
    brain.restore_opponent_memory(memory.unwrap());
    (view, brain)
}

fn buys(view: &PlayerView, brain: &mut ScriptedBrain) -> Vec<(CityId, ProductionItem)> {
    brain
        .orders(view)
        .into_iter()
        .filter_map(|o| match o {
            Order::BuyItem { city, item } => Some((city, item)),
            _ => None,
        })
        .collect()
}

const BOMBER: ProductionItem = ProductionItem::Unit {
    unit: UnitKind::Bomber,
};

/// Gate 1 (patterns 2026-11, fix 2): at T140 he bought a Bomber for 805 of 4,358 Gold the turn
/// after Flight (+2.38 VP in the paired diff). Hard kept it all: its plan read Peace and the
/// 5,000 hoard savings were the reserve. With the gene the Bomber's added capture odds on
/// Karakorum (an original capital) and Khanbaliq (live VP) outweigh the four turns it delays
/// the hoard.
#[test]
fn swiss_140_spends_the_hoard_savings_on_the_bomber() {
    let (view, mut brain) = swiss_140();
    // Isolate the decisive-buy policy from the newer named-fund release.
    brain.decisions.named_fund = 0;
    brain.decisions.decisive_buy = 0;
    brain.decisions.swiss_hoard_any_war = 0;
    brain.decisions.siege_survival_clock = 0;
    let recorded_hard = buys(&view, &mut brain);
    assert!(recorded_hard.is_empty(), "gene off: {recorded_hard:?}");
    let (view, mut brain) = swiss_140();
    brain.decisions.named_fund = 0;
    brain.decisions.decisive_buy = 1;
    brain.decisions.siege_survival_clock = 0;
    let bought = buys(&view, &mut brain);
    assert_eq!(bought.len(), 1, "{bought:?}");
    assert_eq!(bought[0].1, BOMBER, "{bought:?}");
    // Based where he based his, in Shangdu, in range of Sarai, Karakorum and the units
    // threatening Almaliq (from Biel, which reaches only Karakorum and Khanbaliq, the same
    // plane is worth +0.12 VP over Hard in 48 paired runs to T161; from Shangdu +1.71).
    assert_eq!(bought[0].0, CityId(27), "{bought:?}");
}

/// Gate 2: routine spending keeps the reserve. The same treasury and cities at Peace with
/// everyone buy nothing (no live war, so no purchase is decisive), and neither does a held
/// hoard, whose 5,000 Gold scores a live VP.
#[test]
fn swiss_140_at_peace_keeps_the_reserve() {
    use legatum_core::diplomacy::RelationState;
    let (mut view, mut brain) = swiss_140();
    for r in view
        .diplomacy
        .relations
        .iter_mut()
        .filter(|r| r.a == view.team || r.b == view.team)
    {
        r.state = RelationState::Peace;
    }
    brain.decisions.decisive_buy = 2;
    let bought = buys(&view, &mut brain);
    assert!(bought.is_empty(), "{bought:?}");
}

#[test]
fn swiss_140_never_spends_a_held_hoard() {
    let (mut view, mut brain) = swiss_140();
    view.economy.gold = 5_400;
    view.players[view.me.0 as usize].faction_goal_done = true;
    view.faction_goal.as_mut().expect("Swiss goal").done = true;
    brain.decisions.decisive_buy = 2;
    let bought = buys(&view, &mut brain);
    assert!(
        !bought.iter().any(|(_, item)| *item == BOMBER),
        "{bought:?}"
    );
}
