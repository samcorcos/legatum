use super::*;
use crate::data::MIN_CITY_GAP;
use crate::rules::cities::check_found_city;
use crate::rules::test_support::{city, game, hex, unit};
use crate::{player_view, visibility};

fn fixture() -> (PlayerView, UnitView) {
    let mut s = game(&[0, 1]);
    let id = unit(&mut s, 0, UnitKind::Settler, hex(5, 5));
    let mut v = player_view(&s, PlayerId(0));
    for t in &mut v.tiles {
        t.explored = true;
        t.visible = true;
        t.terrain = Some(Terrain::Grassland);
    }
    let u = v.units.iter().find(|u| u.id == id).unwrap().clone();
    (v, u)
}

#[test]
fn scores_are_deterministic_explained_and_rank_only_three_nearby_sites() {
    let (v, u) = fixture();
    let sites = settlement_sites(&v, &u, &[]);
    assert_eq!(sites, settlement_sites(&v, &u, &[]));
    assert_eq!(sites.iter().filter(|s| s.rank.is_some()).count(), 3);
    assert_eq!(sites.iter().find(|s| s.rank == Some(1)).unwrap().pos, u.pos);
    for site in &sites {
        assert_eq!(
            site.score,
            site.breakdown.iter().map(|p| p.value).sum::<i32>()
        );
        if site.rank.is_some() {
            assert!(site.travel_turns.unwrap() <= RECOMMEND_TRAVEL_TURNS);
        }
    }
}

#[test]
fn india_settlement_value_quotes_linear_corruption() {
    let mut s = game(&[0, 1]);
    s.player_mut(PlayerId(0)).faction = Faction::India;
    city(&mut s, 0, hex(3, 3), 2);
    let id = unit(&mut s, 0, UnitKind::Settler, hex(8, 3));
    let mut v = player_view(&s, PlayerId(0));
    for tile in &mut v.tiles {
        tile.explored = true;
        tile.visible = true;
        tile.terrain = Some(Terrain::Grassland);
    }
    let settler = v.units.iter().find(|u| u.id == id).unwrap();
    let site = settlement_site(&v, settler, &[], hex(9, 3)).unwrap();
    let distance_index = site
        .breakdown
        .iter()
        .position(|part| part.label.starts_with("Corruption:"))
        .unwrap();
    let before = site.breakdown[..distance_index]
        .iter()
        .map(|part| part.value)
        .sum::<i32>();
    assert_eq!(
        site.breakdown[distance_index].label,
        "Corruption: 6 hexes from the capital (-20%)"
    );
    assert_eq!(
        site.breakdown[distance_index].value,
        before * 80 / 100 - before
    );

    // A captured capital leaves India without an owned capital until it founds another.
    s.cities.values_mut().next().unwrap().is_capital = false;
    let mut without_capital = player_view(&s, PlayerId(0));
    for tile in &mut without_capital.tiles {
        tile.explored = true;
        tile.visible = true;
        tile.terrain = Some(Terrain::Grassland);
    }
    let settler = without_capital.units.iter().find(|u| u.id == id).unwrap();
    let site = settlement_site(&without_capital, settler, &[], hex(9, 3)).unwrap();
    assert!(
        !site
            .breakdown
            .iter()
            .any(|part| part.label.starts_with("Corruption:"))
    );
}

#[test]
fn advice_and_resolution_use_identical_founding_rules() {
    let mut s = game(&[0, 1]);
    city(&mut s, 0, hex(3, 3), 2);
    s.map.get_mut(hex(9, 8)).unwrap().terrain = Terrain::Ocean;
    s.map.get_mut(hex(10, 8)).unwrap().terrain = Terrain::Mountains;
    s.map.get_mut(hex(9, 6)).unwrap().owner = Some(PlayerId(1));
    let id = unit(&mut s, 0, UnitKind::Settler, hex(6, 6));
    let mut v = player_view(&s, PlayerId(0));
    for (i, t) in v.tiles.iter_mut().enumerate() {
        t.explored = true;
        t.terrain = Some(s.map.tiles[i].terrain);
        t.owner = s.map.tiles[i].owner;
    }
    for i in 0..v.tiles.len() {
        let h = hex_at(&v, i);
        s.units.get_mut(&id).unwrap().pos = h;
        assert_eq!(
            known_legal_site(&v, h),
            check_found_city(&s, id).is_ok(),
            "{h:?}"
        );
    }
}

#[test]
fn unseen_world_changes_never_change_advice() {
    let mut s = game(&[0, 1]);
    let id = unit(&mut s, 0, UnitKind::Settler, hex(1, 1));
    visibility::update_explored(&mut s);
    let before = player_view(&s, PlayerId(0));
    let a = settlement_sites(
        &before,
        before.units.iter().find(|u| u.id == id).unwrap(),
        &[],
    );
    city(&mut s, 1, hex(10, 8), 5);
    unit(&mut s, 1, UnitKind::Ranger, hex(9, 8));
    s.map.get_mut(hex(10, 7)).unwrap().terrain = Terrain::Hills;
    let after = player_view(&s, PlayerId(0));
    assert_eq!(before.tiles, after.tiles);
    assert_eq!(
        a,
        settlement_sites(
            &after,
            after.units.iter().find(|u| u.id == id).unwrap(),
            &[]
        )
    );
}

#[test]
fn visible_foreign_borders_flag_possible_unseen_city_spacing_until_scouted() {
    let (mut v, u) = fixture();
    let center = Hex::new(u.pos.q + MIN_CITY_GAP, u.pos.r);
    let border = Hex::new(center.q - 1, center.r);
    let center_i = v.index(center).unwrap();
    let border_i = v.index(border).unwrap();
    v.tiles[center_i].visible = false;
    v.tiles[border_i].owner = Some(PlayerId(1));
    assert!(known_legal_site(&v, u.pos), "no known city blocks founding");
    assert!(border_city_risks(&v)[v.index(u.pos).unwrap()]);
    // The map still lists it, flagged, and never recommends it; the planner leaves it out.
    let flagged = settlement_site(&v, &u, &[], u.pos).expect("legal sites stay on the map");
    assert!(flagged.border_risk);
    let score = settlement_scores(&v, &u, &[])
        .into_iter()
        .find(|s| s.pos == u.pos)
        .expect("listed");
    assert!(score.border_risk && score.rank.is_none());
    assert!(
        settlement_candidates(&v, &u, &[])
            .iter()
            .all(|s| s.pos != u.pos)
    );

    // Most of a city's land lies within three hexes: evidence need not be adjacent to its owning center.
    v.tiles[border_i].owner = None;
    let outer = v
        .index(Hex::new(center.q, center.r + crate::data::CITY_WORK_RADIUS))
        .unwrap();
    v.tiles[outer].owner = Some(PlayerId(1));
    assert!(settlement_site(&v, &u, &[], u.pos).unwrap().border_risk);
    // Stale borders alone and our own borders do not establish a foreign city.
    v.tiles[outer].visible = false;
    assert!(!settlement_site(&v, &u, &[], u.pos).unwrap().border_risk);
    v.tiles[outer].visible = true;
    v.tiles[outer].owner = Some(v.me);
    assert!(!settlement_site(&v, &u, &[], u.pos).unwrap().border_risk);
    v.tiles[outer].owner = Some(PlayerId(1));
    // Actual scouting of the only possible center removes uncertainty.
    v.tiles[center_i].visible = true;
    let scouted = settlement_site(&v, &u, &[], u.pos).unwrap();
    assert!(!scouted.border_risk);
    assert!(
        settlement_candidates(&v, &u, &[])
            .iter()
            .any(|s| s.pos == u.pos)
    );
}

#[test]
fn the_map_tints_exactly_the_legal_sites_and_explains_every_other_tile() {
    // A map with every kind of obstacle: water, mountains, a wonder, a foreign border, a city
    // and a hidden foreign city's border. The lens's set of sites must be the legal set.
    let mut s = game(&[0, 1]);
    city(&mut s, 0, hex(3, 3), 2);
    s.map.get_mut(hex(9, 8)).unwrap().terrain = Terrain::Ocean;
    s.map.get_mut(hex(10, 8)).unwrap().terrain = Terrain::Mountains;
    s.map.get_mut(hex(9, 6)).unwrap().owner = Some(PlayerId(1));
    s.map.get_mut(hex(7, 3)).unwrap().wonder = Some(crate::wonders::Wonder::MountFuji);
    let id = unit(&mut s, 0, UnitKind::Settler, hex(6, 6));
    let mut v = player_view(&s, PlayerId(0));
    for (i, t) in v.tiles.iter_mut().enumerate() {
        t.explored = true;
        t.visible = i % 7 != 0;
        t.terrain = Some(s.map.tiles[i].terrain);
        t.owner = s.map.tiles[i].owner;
        t.wonder = s.map.tiles[i].wonder;
    }
    let u = v.units.iter().find(|u| u.id == id).unwrap().clone();
    let tinted: std::collections::BTreeSet<Hex> = settlement_scores(&v, &u, &[])
        .iter()
        .map(|s| s.pos)
        .collect();
    let legal: std::collections::BTreeSet<Hex> = (0..v.tiles.len())
        .map(|i| hex_at(&v, i))
        .filter(|&h| known_legal_site(&v, h))
        .collect();
    assert!(legal.len() > 50, "the fixture leaves many legal sites");
    assert_eq!(tinted, legal);
    for i in 0..v.tiles.len() {
        let h = hex_at(&v, i);
        assert_eq!(site_blocker(&v, h).is_none(), legal.contains(&h), "{h:?}");
        s.units.get_mut(&id).unwrap().pos = h;
        assert_eq!(
            tinted.contains(&h),
            check_found_city(&s, id).is_ok(),
            "lens and the founding rule disagree at {h:?}: {:?}",
            check_found_city(&s, id)
        );
    }
    assert!(matches!(
        site_blocker(&v, hex(7, 3)),
        Some(SiteBlocker::Wonder(_))
    ));
    assert!(matches!(
        site_blocker(&v, hex(4, 3)),
        Some(SiteBlocker::TooClose { distance: 1, .. })
    ));
    assert!(matches!(
        site_blocker(&v, hex(9, 6)),
        Some(SiteBlocker::Foreign(PlayerId(1)))
    ));
    assert!(matches!(
        site_blocker(&v, hex(9, 8)),
        Some(SiteBlocker::Water(Terrain::Ocean))
    ));
    assert!(matches!(
        site_blocker(&v, hex(10, 8)),
        Some(SiteBlocker::Impassable(Terrain::Mountains))
    ));
}

#[test]
fn quality_is_absolute_and_ignores_the_settler_and_travel() {
    let (v, u) = fixture();
    let near = settlement_sites(&v, &u, &[]);
    let mut far = u.clone();
    far.pos = hex(14, 10);
    let moved = settlement_sites(&v, &far, &[]);
    for a in &near {
        let b = moved.iter().find(|b| b.pos == a.pos).unwrap();
        assert_eq!((a.quality, a.band), (b.quality, b.band), "{:?}", a.pos);
        assert_eq!(a.quality, base_site_score(&v, a.pos), "{:?}", a.pos);
        assert_eq!(a.band, SiteBand::of(a.quality));
    }
    let mut ranked: Vec<_> = near.iter().filter(|s| s.rank.is_some()).collect();
    ranked.sort_by_key(|s| s.rank);
    assert!(ranked.windows(2).all(|w| w[0].band >= w[1].band));
    assert!(SiteBand::of(i32::MIN) == SiteBand::Poor);
    assert!(SiteBand::of(SITE_BAND_FLOORS[3]) == SiteBand::Exceptional);
    assert!(SITE_BAND_FLOORS.windows(2).all(|w| w[0] < w[1]));
}

#[test]
fn travel_handles_turn_boundaries_impassable_unknown_and_expensive_steps() {
    let (mut v, mut u) = fixture();
    for t in &mut v.tiles {
        t.terrain = Some(Terrain::Mountains);
    }
    u.pos = hex(2, 4);
    for col in 2..=5 {
        let i = v.index(hex(col, 4)).unwrap();
        v.tiles[i].terrain = Some(if col == 4 {
            Terrain::Forest
        } else {
            Terrain::Grassland
        });
    }
    let f = travel_field(&v, &u, true, i32::MAX);
    assert_eq!(f[v.index(hex(4, 4)).unwrap()], 4); // 1, then wait, then 2
    assert_eq!(f[v.index(hex(5, 4)).unwrap()], 5);
    u.max_moves = 1;
    assert_eq!(
        travel_field(&v, &u, true, i32::MAX)[v.index(hex(4, 4)).unwrap()],
        2
    );
    let i = v.index(hex(3, 4)).unwrap();
    v.tiles[i].explored = false;
    assert_eq!(
        travel_field(&v, &u, true, i32::MAX)[v.index(hex(4, 4)).unwrap()],
        i32::MAX
    );
}

#[test]
fn known_threats_use_terrain_and_stationary_ranged_reach_and_exclude_teammates() {
    let (mut v, u) = fixture();
    let mut enemy = u.clone();
    enemy.owner = PlayerId(1);
    enemy.kind = UnitKind::Ranger;
    enemy.military = true;
    enemy.ranged = 15;
    enemy.range = 2;
    enemy.max_moves = 2;
    v.units.push(enemy.clone());
    let reach = known_threats(&v);
    assert_eq!(reach[v.index(hex(8, 5)).unwrap()], 0); // cannot move two then shoot two
    assert_eq!(reach[v.index(hex(7, 5)).unwrap()], 1);
    v.players[1].team = v.team;
    assert!(known_threats(&v).iter().all(|&n| n == 0));
}

#[test]
fn peaceful_units_block_travel_without_becoming_attack_threats() {
    let (mut v, u) = fixture();
    let mut foreign = u.clone();
    foreign.owner = PlayerId(1);
    foreign.pos = hex(6, 5);
    foreign.kind = UnitKind::Ranger;
    foreign.military = true;
    foreign.ranged = 15;
    foreign.range = 2;
    v.units.push(foreign);
    for relation in [
        crate::diplomacy::RelationState::Peace,
        crate::diplomacy::RelationState::Alliance,
    ] {
        v.diplomacy.relations[0].state = relation;
        assert!(known_threats(&v).iter().all(|&n| n == 0));
        for civilian in [false, true] {
            assert_eq!(
                travel_field(&v, &u, civilian, i32::MAX)[v.index(hex(6, 5)).unwrap()],
                i32::MAX
            );
        }
    }
    v.diplomacy.relations[0].state = crate::diplomacy::RelationState::War;
    assert!(known_threats(&v)[v.index(u.pos).unwrap()] > 0);
    assert_ne!(
        travel_field(&v, &u, false, i32::MAX)[v.index(hex(6, 5)).unwrap()],
        i32::MAX
    );
}

#[test]
fn settlement_travel_matches_closed_borders_and_stranded_egress() {
    let (mut v, mut u) = fixture();
    for t in &mut v.tiles {
        t.terrain = Some(Terrain::Mountains);
    }
    for col in 2..=6 {
        let i = v.index(hex(col, 4)).unwrap();
        v.tiles[i].terrain = Some(Terrain::Grassland);
        v.tiles[i].owner = (col == 3).then_some(PlayerId(1));
    }
    u.pos = hex(2, 4);
    v.units = vec![u.clone()].into();
    v.diplomacy.relations[0].state = crate::diplomacy::RelationState::Peace;
    let goal = hex(4, 4);
    assert!(v.movement_path(u.owner, u.kind, u.pos, goal).is_none());
    assert_eq!(
        travel_field(&v, &u, true, i32::MAX)[v.index(goal).unwrap()],
        i32::MAX
    );
    v.diplomacy.relations[0].state = crate::diplomacy::RelationState::War;
    assert!(v.movement_path(u.owner, u.kind, u.pos, goal).is_some());
    assert_eq!(
        travel_field(&v, &u, true, i32::MAX)[v.index(goal).unwrap()],
        2
    );

    v.diplomacy.relations[0].state = crate::diplomacy::RelationState::Peace;
    for col in 3..=6 {
        let i = v.index(hex(col, 4)).unwrap();
        v.tiles[i].owner = Some(PlayerId(1));
    }
    u.pos = hex(4, 4);
    let toward = hex(3, 4);
    let deeper = hex(5, 4);
    let travel = travel_field(&v, &u, true, i32::MAX);
    assert!(v.movement_path(u.owner, u.kind, u.pos, toward).is_some());
    assert_eq!(travel[v.index(toward).unwrap()], 1);
    assert!(v.movement_path(u.owner, u.kind, u.pos, deeper).is_none());
    assert_eq!(travel[v.index(deeper).unwrap()], i32::MAX);
}

#[test]
fn ring_one_matches_real_new_city_and_foreign_tiles_lose_value() {
    let (mut v, u) = fixture();
    let f = v.index(hex(6, 5)).unwrap();
    v.tiles[f].terrain = Some(Terrain::Forest);
    let before = settlement_sites(&v, &u, &[])
        .into_iter()
        .find(|s| s.pos == u.pos)
        .unwrap();
    assert!(before.ring_one.iter().any(|t| t.pos == hex(6, 5)));
    let mut s = game(&[0, 1]);
    s.map.get_mut(hex(6, 5)).unwrap().terrain = Terrain::Forest;
    let cid = city(&mut s, 0, u.pos, 1);
    assert_eq!(
        crate::economy::worked_tiles(&s, &s.cities[&cid]),
        before.ring_one.iter().map(|t| t.pos).collect::<Vec<_>>()
    );
    v.tiles[f].owner = Some(PlayerId(1));
    let after = settlement_sites(&v, &u, &[])
        .into_iter()
        .find(|s| s.pos == u.pos)
        .unwrap();
    assert_eq!(after.foreign_tiles, 1);
    assert!(after.score < before.score);
    assert!(after.ring_one.iter().all(|t| t.pos != hex(6, 5)));
}

#[test]
fn compact_scores_and_single_hex_details_match_full_analysis() {
    let (mut v, u) = fixture();
    for faction in Faction::ALL {
        v.players.iter_mut().find(|p| p.id == v.me).unwrap().faction = faction;
        for reserved in [vec![], vec![hex(6, 5)]] {
            let full = settlement_sites(&v, &u, &reserved);
            let candidates = settlement_candidates(&v, &u, &reserved);
            assert_eq!(candidates.len(), full.len());
            for (candidate, detail) in candidates.iter().zip(&full) {
                assert_eq!(candidate.pos, detail.pos);
                assert_eq!(candidate.score, detail.score);
                assert_eq!(candidate.nearest_city, detail.nearest_city);
                assert_eq!(candidate.travel_turns, detail.travel_turns);
                assert_eq!(candidate.threats, detail.threats);
            }
            let scores = settlement_scores(&v, &u, &reserved);
            assert_eq!(scores.len(), full.len());
            for (summary, mut detail) in scores.into_iter().zip(full) {
                assert_eq!(summary.pos, detail.pos);
                assert_eq!(summary.score, detail.score);
                assert_eq!(summary.rank, detail.rank);
                assert_eq!(summary.travel_turns, detail.travel_turns);
                detail.rank = None;
                assert_eq!(settlement_site(&v, &u, &reserved, detail.pos), Some(detail));
            }
        }
    }
    assert_eq!(settlement_site(&v, &u, &[], Hex::new(-100, -100)), None);
}

/// Qualities of every legal site on a fully explored generated map, as the first player sees it.
fn generated_qualities(seed: u64, factions: [Faction; 2]) -> Vec<i32> {
    let state = crate::game::new_game(&crate::state::GameSetup::standard(seed, factions));
    let mut v = player_view(&state, PlayerId(0));
    for (i, t) in v.tiles.iter_mut().enumerate() {
        let real = &state.map.tiles[i];
        t.explored = true;
        t.visible = true;
        t.terrain = Some(real.terrain);
        t.owner = None;
        t.wonder = real.wonder;
        t.resource = real.resource;
    }
    (0..v.tiles.len())
        .map(|i| hex_at(&v, i))
        .filter(|&h| known_legal_site(&v, h))
        .map(|h| base_site_score(&v, h))
        .collect()
}

#[test]
fn site_band_floors_split_generated_maps() {
    let mut all = Vec::new();
    for seed in 0..8u64 {
        all.extend(generated_qualities(
            seed,
            [Faction::ALL[(seed % 8) as usize], Faction::Switzerland],
        ));
    }
    let share = |band: SiteBand| {
        all.iter().filter(|&&q| SiteBand::of(q) == band).count() as f64 / all.len() as f64
    };
    for band in SiteBand::ALL {
        eprintln!("{:>11}: {:>5.1}%", band.name(), share(band) * 100.0);
    }
    // Poor and Exceptional are real, minority bands; no band is empty or swallows the map.
    for band in SiteBand::ALL {
        assert!(
            share(band) >= 0.02,
            "{band:?} is nearly empty: {}",
            share(band)
        );
        assert!(
            share(band) <= 0.40,
            "{band:?} swallows the map: {}",
            share(band)
        );
    }
}

/// Prints the distribution the band floors were chosen from:
/// `cargo test -p legatum-core calibrate_site_bands -- --ignored --nocapture`
#[test]
#[ignore]
fn calibrate_site_bands() {
    let mut all = Vec::new();
    let mut by_faction: Vec<(Faction, Vec<i32>)> = Vec::new();
    for (k, faction) in Faction::ALL.into_iter().enumerate() {
        let mut mine = Vec::new();
        for seed in 0..60u64 {
            mine.extend(generated_qualities(
                seed * 8 + k as u64,
                [faction, Faction::Switzerland],
            ));
        }
        all.extend(mine.iter().copied());
        by_faction.push((faction, mine));
    }
    all.sort();
    let at = |v: &[i32], p: f64| v[((v.len() - 1) as f64 * p) as usize];
    eprintln!("legal sites: {}", all.len());
    for p in [
        0.0, 0.05, 0.10, 0.20, 0.30, 0.40, 0.50, 0.60, 0.70, 0.80, 0.90, 0.95, 0.98, 0.99, 1.0,
    ] {
        eprintln!("p{:>3.0}: {}", p * 100.0, at(&all, p));
    }
    for (faction, mut v) in by_faction {
        v.sort();
        eprintln!(
            "{faction:?}: n={} p10={} p50={} p90={} p99={}",
            v.len(),
            at(&v, 0.1),
            at(&v, 0.5),
            at(&v, 0.9),
            at(&v, 0.99)
        );
    }
    for band in SiteBand::ALL {
        let n = all.iter().filter(|&&q| SiteBand::of(q) == band).count();
        eprintln!(
            "{:>11}: {:>5.1}%",
            band.name(),
            100.0 * n as f64 / all.len() as f64
        );
    }
}

#[test]
fn an_egyptian_settler_values_land_by_its_nile_food_and_no_production() {
    let (mut v, u) = fixture();
    for t in &mut v.tiles {
        t.terrain = Some(Terrain::Hills);
    }
    let plain = site_potential(&v, u.pos).0;
    assert!(plain.production > 0);
    let me = v.me;
    v.players.iter_mut().find(|p| p.id == me).unwrap().faction = Faction::Egypt;
    let nile = site_potential(&v, u.pos).0;
    assert_eq!(nile.production, 0, "Egypt has no Production");
    let site = settlement_site(&v, &u, &[], u.pos).unwrap();
    assert_eq!(site.potential_yields, nile, "the memoised sums agree");
    assert_eq!(site.center_yields.production, 0);
    for t in &mut v.tiles {
        t.terrain = Some(Terrain::Grassland);
    }
    let grass = site_potential(&v, u.pos).0;
    let pct = v.rule_experiments.egypt_nile_food_percent();
    v.players.iter_mut().find(|p| p.id == me).unwrap().faction = Faction::Byzantium;
    assert_eq!(grass.food, site_potential(&v, u.pos).0.food * pct / 100);
}
