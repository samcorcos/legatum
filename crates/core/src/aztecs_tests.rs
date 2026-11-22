use crate::{Faction, GameSetup, PlayerId, ProductionItem, UnitKind, data};

#[test]
fn aztecs_no_horses_but_modern_armour_is_available() {
    for kind in UnitKind::ALL {
        let item = ProductionItem::Unit { unit: kind };
        let blocked = data::is_horse_mounted(kind) || kind == UnitKind::Heir;
        assert_eq!(
            data::available_to_faction(Faction::Aztecs, item),
            !blocked,
            "{kind:?}"
        );
        if data::is_horse_mounted(kind) {
            assert_eq!(
                data::faction_unavailable_reason(item, Faction::Aztecs).as_deref(),
                Some(data::AZTEC_NO_HORSES)
            );
        }
    }
}

#[test]
fn aztecs_save_round_trip_and_old_faction_ids_are_stable() {
    let setup = GameSetup::standard(u64::MAX, [Faction::Aztecs, Faction::Sparta]);
    let state = crate::new_game(&setup);
    let saved = serde_json::to_string(&state).unwrap();
    let restored: crate::GameState = serde_json::from_str(&saved).unwrap();
    assert_eq!(state, restored);
    assert_eq!(
        crate::goals::rule(&restored, PlayerId(0)),
        crate::goals::GoalRule::Swarm
    );
    for f in Faction::ALL {
        assert_eq!(serde_json::to_string(&f).unwrap(), format!("\"{f:?}\""));
    }
}

#[test]
fn aztecs_standard_ai_seats_rotate_over_unpicked_factions() {
    use std::collections::BTreeSet;
    let mut seen = BTreeSet::new();
    for seed in 0..20 {
        let setup = GameSetup::standard(seed, [Faction::Aztecs, Faction::Sparta]);
        let ai: BTreeSet<_> = setup
            .players
            .iter()
            .filter(|p| !p.is_human)
            .map(|p| p.faction)
            .collect();
        assert_eq!(ai.len(), 6);
        assert!(!ai.contains(&Faction::Aztecs));
        assert!(!ai.contains(&Faction::Sparta));
        assert!(
            !ai.contains(&Faction::Rome),
            "established seeded setups keep their original faction draw"
        );
        seen.extend(ai);
    }
    assert_eq!(seen.len(), Faction::ALL[..9].len() - 2);
}

#[test]
fn aztecs_research_does_not_promise_horse_upgrades() {
    use crate::TechId;
    for tech in [TechId::Chivalry, TechId::MilitaryScience] {
        let effects = data::tech_card_effects_for(Faction::Aztecs, tech);
        assert!(
            !effects
                .iter()
                .any(|effect| effect.contains("Cataphracts") || effect.contains("Cuirassiers"))
        );
        assert!(data::tech_has_benefit(
            Default::default(),
            Faction::Aztecs,
            tech
        ));
    }
    assert!(data::tech_has_benefit(
        Default::default(),
        Faction::Aztecs,
        TechId::Combustion
    ));
    assert!(
        data::tech_card_effects_for(Faction::Aztecs, TechId::AutonomousSystems)
            .contains(&"Main Battle Tanks can upgrade to Autonomous Armour")
    );
}
