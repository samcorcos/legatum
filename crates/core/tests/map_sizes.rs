//! The lobby presets must keep working for 2-8 players, including crowded/team games.

use legatum_core::mapgen::{
    START_FOOD, START_MAX_QUALITY_SPREAD, START_MIN_QUALITY, START_PRODUCTION, StartOptions,
    founding_yields, generate, generate_counted, generate_counted_with_options,
    generate_with_options, start_quality, validate,
};
use legatum_core::state::Map;
use legatum_core::{AiDifficulty, Faction, GameSetup, Hex, PlayerSetup, TeamId};

/// (name, width, height) as in the lobby.
const SIZES: [(&str, i32, i32); 3] = [("Small", 40, 26), ("Standard", 48, 32), ("Large", 60, 40)];
const SEEDS: u64 = 100;

fn setup(seed: u64, w: i32, h: i32, players: usize) -> GameSetup {
    GameSetup {
        seed,
        width: w,
        height: h,
        players: (0..players)
            .map(|i| PlayerSetup {
                name: format!("P{i}"),
                faction: Faction::ALL[i % Faction::ALL.len()],
                team: TeamId(i as u8),
                is_human: i == 0,
                difficulty: AiDifficulty::Normal,
                search: false,
            })
            .collect(),
        turn_limit: Some(150),
        legacy_all_war: false,
        barbarians: false,
        rule_experiments: Default::default(),
    }
}

fn check_quality(map: &Map, starts: &[Hex], setup: &GameSetup) -> (i32, i32) {
    let scores: Vec<_> = (0..starts.len())
        .map(|seat| start_quality(map, starts, setup, seat))
        .collect();
    let low = *scores.iter().min().unwrap();
    let spread = scores.iter().max().unwrap() - low;
    // Inner plains contribute 6 points each: the entire difference, including ruins and
    // rivals, may cost at most four such tiles. A 100-point floor prevents equal bad starts.
    assert!(
        spread <= START_MAX_QUALITY_SPREAD && low >= START_MIN_QUALITY,
        "{}x{}, {}p, seed {}: scores {scores:?}",
        map.width,
        map.height,
        starts.len(),
        setup.seed
    );
    (spread, low)
}

#[test]
fn every_preset_generates_fair_starts_for_two_to_eight_players() {
    for (name, w, h) in SIZES {
        for players in 2..=8 {
            let (mut retries, mut worst, mut total, mut minimum) = (0, 0, 0, i32::MAX);
            for seed in 0..SEEDS {
                let setup = setup(seed, w, h, players);
                let ((map, starts), attempts) = generate_counted(&setup);
                assert_eq!(starts.len(), players, "{name} {players}p seed {seed}");
                validate(&map, &starts)
                    .unwrap_or_else(|e| panic!("{name} {players}p seed {seed}: {e}"));
                for &s in &starts {
                    assert_eq!(
                        founding_yields(&map, s),
                        (START_FOOD, START_PRODUCTION),
                        "{name} {players}p seed {seed}: start {s:?}"
                    );
                }
                let (spread, low) = check_quality(&map, &starts, &setup);
                worst = worst.max(spread);
                total += spread;
                minimum = minimum.min(low);
                retries += attempts;
            }
            assert!(retries < SEEDS * 20, "{name} {players}p: {retries} retries");
            eprintln!(
                "{name} {players}p: max spread {worst}, total spread {total}/{SEEDS}, minimum quality {minimum}, retries {retries}"
            );
        }
    }
}

fn check_teams(starts: &[Hex], setup: &GameSetup) {
    // Independently check both the per-member promise and connectivity: nearest-ally alone
    // would incorrectly accept a four-player team split into two distant pairs.
    for (seat, &start) in starts.iter().enumerate() {
        let team = setup.players[seat].team;
        let members: Vec<_> = (0..starts.len())
            .filter(|&i| setup.players[i].team == team)
            .collect();
        let nearest_rival = |i: usize| {
            (0..starts.len())
                .filter(|&j| setup.players[j].team != team)
                .map(|j| starts[i].distance(starts[j]))
                .min()
                .unwrap_or(i32::MAX)
        };
        if let Some(ally) = members
            .iter()
            .filter(|&&i| i != seat)
            .map(|&i| start.distance(starts[i]))
            .min()
        {
            assert!(
                ally < nearest_rival(seat),
                "seed {}, seat {seat}: {starts:?}",
                setup.seed
            );
        }
        let mut reached = vec![seat];
        loop {
            let added = members.iter().copied().find(|j| {
                !reached.contains(j)
                    && reached.iter().any(|&i| {
                        let distance = starts[i].distance(starts[*j]);
                        distance < nearest_rival(i) && distance < nearest_rival(*j)
                    })
            });
            match added {
                Some(j) => reached.push(j),
                None => break,
            }
        }
        assert_eq!(
            reached.len(),
            members.len(),
            "seed {}: split team {team:?}",
            setup.seed
        );
    }
}

#[test]
fn teams_stay_together_across_presets_and_player_counts() {
    for (name, w, h) in SIZES {
        for players in 2..=8 {
            let mut retries = 0;
            for seed in 0..SEEDS {
                let mut setup = setup(seed, w, h, players);
                // Rotate through a pair plus solos, interleaved teams, 4v4, uneven teams,
                // and everyone allied. Team IDs need not be contiguous or match seats.
                for (i, player) in setup.players.iter_mut().enumerate() {
                    let group = match seed % 6 {
                        0 => i.saturating_sub(1),
                        1 => i % players.div_ceil(2),
                        2 => usize::from(i >= players.div_ceil(2)),
                        3 => i / 3,
                        4 => usize::from(i == players - 1),
                        _ => 0,
                    };
                    player.team = TeamId((group * 7 + 10) as u8);
                }
                let ((map, starts), attempts) = generate_counted(&setup);
                validate(&map, &starts).unwrap();
                check_quality(&map, &starts, &setup);
                check_teams(&starts, &setup);
                retries += attempts;
            }
            assert!(
                retries < SEEDS * 20,
                "{name} {players}p teams: {retries} retries"
            );
            eprintln!("{name} {players}p teams: {retries} retries/{SEEDS}");
        }
    }
}

#[test]
fn options_are_deterministic_and_team_ids_are_only_labels() {
    for (_, w, h) in SIZES {
        for seed in [0, 1, 77, u64::MAX - 2, u64::MAX - 1, u64::MAX] {
            let mut setup = setup(seed, w, h, 8);
            setup.players[4].team = setup.players[0].team;
            let default = generate(&setup);
            assert_eq!(
                default,
                generate_with_options(&setup, StartOptions::default())
            );
            check_teams(&default.1, &setup);
            for teams_start_near in [false, true] {
                let options = StartOptions { teams_start_near };
                let a = generate_counted_with_options(&setup, options);
                assert_eq!(a, generate_counted_with_options(&setup, options));
                let (map, starts) = &a.0;
                validate(map, starts).unwrap();
                check_quality(map, starts, &setup);
            }
            for player in &mut setup.players {
                player.team = TeamId(100 - player.team.0);
                player.is_human = !player.is_human;
                player.difficulty = AiDifficulty::Hard;
            }
            assert_eq!(default, generate(&setup));
        }
    }
}

#[test]
fn opting_out_can_separate_teammates() {
    let mut separated = false;
    for seed in 0..10 {
        let mut setup = setup(seed, 40, 26, 8);
        setup.players[4].team = setup.players[0].team;
        let (map, starts) = generate_with_options(
            &setup,
            StartOptions {
                teams_start_near: false,
            },
        );
        validate(&map, &starts).unwrap();
        check_quality(&map, &starts, &setup);
        let rival = (1..8)
            .filter(|&i| i != 4)
            .map(|i| starts[0].distance(starts[i]))
            .min()
            .unwrap();
        separated |= starts[0].distance(starts[4]) >= rival;
    }
    assert!(
        separated,
        "opting out must actually disable teammate proximity"
    );
}
