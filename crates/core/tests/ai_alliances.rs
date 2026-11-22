//! `GameSetup::ai_alliances`: the teams with no human seat start in Alliances of two.
use legatum_core::diplomacy::{RelationState, alliance_sides};
use legatum_core::*;
use std::collections::BTreeSet;

fn coop(seed: u64, on: bool) -> GameSetup {
    let mut setup = GameSetup::standard(seed, [Faction::Switzerland, Faction::Byzantium]);
    setup.ai_alliances = on;
    setup
}

fn alliances(state: &GameState) -> Vec<(TeamId, TeamId)> {
    state
        .diplomacy
        .as_ref()
        .unwrap()
        .relations
        .iter()
        .filter(|r| r.state == RelationState::Alliance)
        .map(|r| (r.a, r.b))
        .collect()
}

#[test]
fn coop_setup_starts_three_ai_alliances_and_none_with_the_human_team() {
    for seed in 1..=20 {
        let state = new_game(&coop(seed, true));
        let pairs = alliances(&state);
        assert_eq!(pairs.len(), 3, "seed {seed}");
        let mut teams = BTreeSet::new();
        for (a, b) in &pairs {
            assert!(a < b);
            assert!(
                *a != TeamId(0) && *b != TeamId(0),
                "seed {seed}: the human team is never allied"
            );
            assert!(
                teams.insert(*a) && teams.insert(*b),
                "seed {seed}: a team in two pairs"
            );
        }
        assert_eq!(teams, (1..=6).map(TeamId).collect::<BTreeSet<_>>());
        let d = state.diplomacy.as_ref().unwrap();
        // Pairs of two, not one bloc; the pair has met; the Alliance is a normal signed one.
        assert_eq!(alliance_sides(&d.relations).len(), 3);
        for (a, b) in &pairs {
            assert!(d.met(*a, *b));
            assert_eq!(d.partners(*a), vec![*b]);
            let r = d.relation(*a, *b).unwrap();
            assert_eq!(r.security.as_ref().unwrap().started_turn, 1);
            assert!(!d.at_war(*a, *b));
        }
        // The human team is at Peace with every AI team, as always.
        for t in 1..=6 {
            assert_eq!(
                d.relation(TeamId(0), TeamId(t)).unwrap().state,
                RelationState::Peace
            );
        }
    }
}

#[test]
fn pairs_depend_only_on_the_seed() {
    let a = coop(7, true).ai_alliance_pairs();
    assert_eq!(a, coop(7, true).ai_alliance_pairs());
    // Seat order doesn't matter: only the set of AI teams and the seed do.
    let mut shuffled = coop(7, true);
    shuffled.players[2..].reverse();
    assert_eq!(a, shuffled.ai_alliance_pairs());
    let distinct: BTreeSet<_> = (1..=40)
        .map(|s| coop(s, true).ai_alliance_pairs())
        .collect();
    assert!(distinct.len() > 5, "pairing should vary with the seed");
    assert_eq!(
        new_game(&coop(7, true)).hash(),
        new_game(&coop(7, true)).hash()
    );
}

#[test]
fn other_setups_are_unchanged() {
    for seed in 1..=5 {
        let off = new_game(&coop(seed, false));
        assert!(alliances(&off).is_empty());
        assert_eq!(off.diplomacy.as_ref().unwrap().next_treaty_id, 0);
        let ffa = GameSetup::all_ai(seed);
        assert!(!ffa.ai_alliances && ffa.ai_alliance_pairs().is_empty());
        assert!(alliances(&new_game(&ffa)).is_empty());
    }
    // An off setup serializes without the field, and one without it parses as off.
    let json = serde_json::to_string(&coop(3, false)).unwrap();
    assert!(!json.contains("ai_alliances"));
    let parsed: GameSetup = serde_json::from_str(&json).unwrap();
    assert!(!parsed.ai_alliances);
    assert!(
        serde_json::to_string(&coop(3, true))
            .unwrap()
            .contains("\"ai_alliances\":true")
    );
}

#[test]
fn an_odd_team_out_stays_unallied() {
    let mut setup = GameSetup::all_ai(4);
    setup.ai_alliances = true;
    setup.players.truncate(5);
    let state = new_game(&setup);
    assert_eq!(alliances(&state).len(), 2);
}
