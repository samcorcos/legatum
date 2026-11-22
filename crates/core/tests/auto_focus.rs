//! Citizen priorities are consequences of production, never persistent player settings.
use legatum_core::{economy, game, rules::cities, state::Map, visibility, *};

const P: PlayerId = PlayerId(0);
fn setup() -> (GameState, CityId) {
    let mut setup = GameSetup::all_ai(42);
    setup.players.truncate(2);
    setup.players[0].faction = Faction::Switzerland;
    let mut s = game::blank_game(&setup, Map::new(16, 16, Terrain::Grassland));
    let settler = game::spawn_unit(&mut s, P, UnitKind::Settler, Hex::from_offset(5, 5));
    let id = cities::found_city(&mut s, settler, &mut vec![]).unwrap();
    visibility::update_explored(&mut s);
    (s, id)
}

fn unit(unit: UnitKind) -> ProductionItem {
    ProductionItem::Unit { unit }
}

#[test]
fn priorities_depend_only_on_build_and_settler_population_gate() {
    let (mut s, id) = setup();
    for faction in Faction::ALL {
        s.player_mut(P).faction = faction;
        for human in [false, true] {
            s.player_mut(P).is_human = human;
            for population in [1, 2, 7, 8] {
                let c = s.cities.get_mut(&id).unwrap();
                c.population = population;
                for kind in UnitKind::ALL {
                    let expected = match kind {
                        kind if legatum_core::data::unit_class(kind).is_some() => {
                            CityFocus::Production
                        }
                        UnitKind::Settler if population < 2 => CityFocus::Growth,
                        _ => CityFocus::Balanced,
                    };
                    assert_eq!(economy::auto_focus(c, Some(unit(kind))), expected);
                }
                for building in BuildingKind::ALL {
                    assert_eq!(
                        economy::auto_focus(c, Some(ProductionItem::Building { building })),
                        if legatum_core::data::is_great_work(building) {
                            CityFocus::Production
                        } else {
                            CityFocus::Balanced
                        }
                    );
                }
                assert_eq!(economy::auto_focus(c, None), CityFocus::Balanced);
            }
        }
    }
}

#[test]
fn switching_builds_changes_the_focus_label_not_the_worked_tiles_and_previews_without_a_focus_order()
 {
    let (mut s, id) = setup();
    let pos = s.cities[&id].pos;
    let hill = pos.neighbor(0);
    s.map.get_mut(hill).unwrap().terrain = Terrain::Hills;
    let before = s.clone();
    let preview = economy::city_yields_for(&s, &s.cities[&id], Some(unit(UnitKind::Spearman)));
    assert_eq!(s, before, "preview is pure");
    // Population is territory: a city works every owned tile in range, whatever it builds.
    assert_eq!(preview.worked.len(), 6);
    assert!(preview.worked.contains(&hill));
    assert_eq!(preview.focus, CityFocus::Production);
    assert_eq!(preview.focus_reason, "building Spearman");
    let view = player_view(&s, P);
    let detail = view
        .cities
        .iter()
        .find(|c| c.id == id)
        .unwrap()
        .detail
        .as_ref()
        .unwrap();
    let option = detail
        .options
        .iter()
        .find(|o| o.item == unit(UnitKind::Spearman))
        .unwrap();
    assert_eq!(option.yields, preview);
    assert_eq!(
        option.turns,
        (option.cost + preview.total.production - 1) / preview.total.production
    );
    // A population-1 city can't choose a Settler (it needs population 2).
    for kind in [UnitKind::Spearman, UnitKind::Scout] {
        cities::apply_city_orders(
            &mut s,
            P,
            &[Order::SetProduction {
                city: id,
                item: unit(kind),
            }],
            &mut vec![],
        );
        let actual = economy::city_yields(&s, &s.cities[&id]);
        assert_eq!(
            actual,
            economy::city_yields_for(&s, &s.cities[&id], Some(unit(kind)))
        );
        assert_eq!(actual.worked, preview.worked);
        if kind == UnitKind::Spearman {
            assert_eq!(actual, preview);
        } else {
            assert_eq!(actual.focus, CityFocus::Balanced);
        }
    }
}

#[test]
fn growth_gate_and_completed_build_rederive_priorities() {
    let (mut s, id) = setup();
    let c = s.cities.get_mut(&id).unwrap();
    c.producing = Some(unit(UnitKind::Settler));
    assert_eq!(economy::auto_focus(c, c.producing), CityFocus::Growth);
    c.population = 2;
    assert_eq!(economy::auto_focus(c, c.producing), CityFocus::Balanced);
    c.producing = Some(unit(UnitKind::Spearman));
    c.production_stored = 1_000;
    cities::process_cities(&mut s, &mut vec![]);
    assert_eq!(s.cities[&id].producing, None);
    assert_eq!(
        economy::city_yields(&s, &s.cities[&id]).focus,
        CityFocus::Balanced
    );
}

#[test]
fn claim_preview_matches_claim_and_exhaustion_is_visible() {
    let (mut s, id) = setup();
    let next = cities::next_claim(&s, &s.cities[&id]).unwrap();
    let view = player_view(&s, P);
    assert_eq!(
        view.cities[0]
            .detail
            .as_ref()
            .unwrap()
            .next_claim
            .as_ref()
            .map(|c| c.tile),
        Some(next)
    );
    s.cities.get_mut(&id).unwrap().population += 1;
    let mut events = vec![];
    cities::process_borders(&mut s, &mut events);
    assert!(events.iter().any(
        |e| matches!(e, Event::TileClaimed { city, tile, .. } if *city == id && *tile == next)
    ));
    for tile in s.map.tiles.iter_mut() {
        tile.owner = Some(P);
    }
    assert_eq!(
        player_view(&s, P).cities[0]
            .detail
            .as_ref()
            .unwrap()
            .next_claim,
        None
    );
}

#[test]
fn legacy_focus_fields_are_ignored_and_legacy_focus_orders_are_invalid() {
    let (s, id) = setup();
    for focus in ["Growth", "Production", "Balanced", "unknown future value"] {
        let mut save = serde_json::to_value(&s).unwrap();
        save["cities"][id.0.to_string()]["focus"] = focus.into();
        let loaded: GameState = serde_json::from_value(save).unwrap();
        assert_eq!(loaded, s);
        assert_eq!(loaded.hash(), s.hash());
        assert!(
            serde_json::to_value(&loaded).unwrap()["cities"][id.0.to_string()]
                .get("focus")
                .is_none()
        );
    }
    let old = r#"{"type":"SetFocus","city":1,"focus":"Growth"}"#;
    assert!(
        serde_json::from_str::<Order>(old)
            .unwrap_err()
            .to_string()
            .contains("unknown variant `SetFocus`")
    );
    let order: Order = serde_json::from_str(r#"{"type":"SetProduction","city":1,"item":{"kind":"Unit","unit":"Settler"},"focus":"Growth"}"#).unwrap();
    assert_eq!(
        order,
        Order::SetProduction {
            city: CityId(1),
            item: unit(UnitKind::Settler)
        }
    );
}
