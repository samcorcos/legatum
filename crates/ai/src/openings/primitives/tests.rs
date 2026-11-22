use super::*;
use crate::fixture::{ENEMY, ME, ViewBuilder};
use crate::openings::{Opening, OpeningBrain, OpeningPolicy};
use crate::{Brain, Knobs, ScriptedBrain};
use legatum_core::diplomacy::{Relation, SecurityTreaty};
use legatum_core::{BuildingKind, Faction, PlayerId, TechId};

fn h(x: i32, y: i32) -> Hex {
    Hex::new(x, y)
}
fn plan<'a>(v: &'a PlayerView, g: &'a PrimitiveParams, memory: &'a PrimitiveMemory) -> Planner<'a> {
    let mut p = Planner::new(v, Knobs::for_faction(v.faction_of(ME)), v.faction_of(ME), 7);
    p.primitives = Some(PrimitivePlan { params: g, memory });
    p
}
fn view() -> PlayerView {
    ViewBuilder::new(36, 24)
        .faction(Faction::Byzantium)
        .known_techs(&[TechId::Currency, TechId::Irrigation])
        .city(1, ME, h(5, 5), 3)
        .gold(0)
        .build()
}
fn relation(a: TeamId, b: TeamId, state: RelationState) -> Relation {
    Relation {
        a,
        b,
        state,
        effective_turn: 1,
        guarantee_until: 0,
        version: 0,
        security: None,
        war_id: None,
        declarers: Vec::new(),
    }
}

fn legal(order: Order) -> legatum_core::diplomacy::DiplomaticAction {
    legatum_core::diplomacy::DiplomaticAction {
        new_wars: Vec::new(),
        calls: Vec::new(),
        also_agree: Vec::new(),
        order,
        effective_turn: 1,
        cost: 0,
        blocked_reason: None,
        upkeep: 0,
        betrayal: false,
    }
}

#[test]
fn snipe_war_score_requires_legal_declaration_and_can_capture_on_the_same_turn() {
    let mut v = ViewBuilder { view: view() }
        .unit(1, ME, UnitKind::Spearman, h(8, 5))
        .unit(2, ENEMY, UnitKind::Settler, h(9, 5))
        .build();
    v.diplomacy.legacy = false;
    let target = v.team_of(ENEMY);
    v.diplomacy.relations = vec![relation(v.team, target, RelationState::Peace)];
    let declare = Order::DeclareWar {
        against: target,
        breach: false,
    };
    v.diplomacy.actions = vec![legal(declare.clone())];
    let mut policy = OpeningPolicy::builtin(Opening::Default);
    policy.primitives.weakest_target_weight = 0;
    policy.primitives.coalition_march_weight = 0;
    policy.primitives.snipe_settlers_war_score = 70;
    let orders = |v: &PlayerView, policy: &OpeningPolicy| {
        let mut base = ScriptedBrain::new(Faction::Byzantium, 7, ME);
        base.knobs.aggression = 0;
        let mut brain = OpeningBrain::new(base, policy.clone());
        brain.primitives_enabled = true;
        brain.orders(v)
    };
    let on = orders(&v, &policy);
    assert!(on.contains(&declare));
    assert!(
        on.contains(&Order::MoveUnit {
            unit: UnitId(1),
            to: h(9, 5)
        }),
        "{on:?}"
    );
    policy.primitives.snipe_settlers_war_score = 69;
    assert!(!orders(&v, &policy).contains(&declare));
    policy.primitives.snipe_settlers_war_score = 70;
    v.diplomacy.actions[0].blocked_reason = Some("guaranteed peace".into());
    assert!(!orders(&v, &policy).contains(&declare));
}

#[test]
fn siege_persistence_rejects_both_incoming_peace_and_our_own_offer() {
    use legatum_core::diplomacy::{PeaceProposal, ProposalId, TreatyKind};
    let mut v = ViewBuilder { view: view() }
        .city(2, ENEMY, h(12, 5), 3)
        .unit(1, ME, UnitKind::Archer, h(10, 5))
        .build();
    v.turn = 40;
    v.cities[1].hp = 50;
    v.diplomacy.legacy = false;
    v.diplomacy.relations = vec![relation(v.team, v.team_of(ENEMY), RelationState::War)];
    let offer = Order::ProposeTreaty {
        to: v.team_of(ENEMY),
        kind: TreatyKind::Peace,
    };
    let accept = Order::RespondToProposal {
        proposal: ProposalId(1),
        accept: true,
        explanation: None,
    };
    let decline = Order::RespondToProposal {
        proposal: ProposalId(1),
        accept: false,
        explanation: None,
    };
    v.diplomacy.actions = vec![
        legal(offer.clone()),
        legal(accept.clone()),
        legal(decline.clone()),
    ];
    let mut g = PrimitiveParams::default();
    let memory = PrimitiveMemory::default();
    let diplomacy = |v: &PlayerView, g: &PrimitiveParams| {
        let mut p = plan(v, g, &memory);
        p.plan_diplomacy();
        p.orders
    };
    assert!(!diplomacy(&v, &g).contains(&offer));
    g.siege_persistence_weight = 0;
    assert!(diplomacy(&v, &g).contains(&offer));
    let (from, to) = (v.team_of(ENEMY), v.team);
    v.diplomacy.proposals.push(PeaceProposal {
        explanation: None,
        id: ProposalId(1),
        kind: TreatyKind::Peace,
        from,
        to,
        proposer: ENEMY,
        created_turn: 39,
        respond_by: 41,
        relation_version: 0,
        consents: Vec::new(),
        needs: vec![to],
        required_teams: Vec::new(),
    });
    assert!(diplomacy(&v, &g).contains(&accept));
    g.siege_persistence_weight = 100;
    assert!(diplomacy(&v, &g).contains(&decline));
}

#[test]
fn escort_civilians_trigger_reserves_before_garrison_and_wait_expires() {
    let mut v = ViewBuilder { view: view() }
        .unit(1, ME, UnitKind::Spearman, h(5, 5))
        .unit(2, ME, UnitKind::Settler, h(5, 5))
        .unit(3, ENEMY, UnitKind::Spearman, h(12, 7))
        .build();
    let mut g = PrimitiveParams::default();
    let mut memory = PrimitiveMemory::default();
    memory.observe(&v);
    let destination = h(12, 5);
    let mut p = plan(&v, &g, &memory);
    p.unit_order(
        UnitId(2),
        Order::MoveUnit {
            unit: UnitId(2),
            to: destination,
        },
    );
    p.plan_military();
    let moves: Vec<_> = p
        .orders
        .iter()
        .filter_map(|o| match o {
            Order::MoveUnit { unit, to } if unit.0 <= 2 => Some(*to),
            _ => None,
        })
        .collect();
    assert_eq!(moves.len(), 2, "{:?}", p.orders);
    assert_eq!(moves[0], moves[1]);
    assert_ne!(
        moves[0], destination,
        "escorts must not race ahead to the final site"
    );

    g.escort_civilians_threat = 200;
    let mut p = plan(&v, &g, &memory);
    p.unit_order(
        UnitId(2),
        Order::MoveUnit {
            unit: UnitId(2),
            to: destination,
        },
    );
    p.plan_primitive_escorts();
    assert!(!p.has_acted(UnitId(1)));
    assert!(p.orders.contains(&Order::MoveUnit {
        unit: UnitId(2),
        to: destination
    }));

    g.escort_civilians_threat = 1;
    v.units.retain(|u| u.id != UnitId(1));
    let mut p = plan(&v, &g, &memory);
    p.unit_order(
        UnitId(2),
        Order::MoveUnit {
            unit: UnitId(2),
            to: destination,
        },
    );
    p.plan_primitive_escorts();
    assert!(p.orders.contains(&Order::SkipUnit { unit: UnitId(2) }));
    v.turn += g.escort_civilians_wait as u32;
    memory.observe(&v);
    let mut p = plan(&v, &g, &memory);
    p.unit_order(
        UnitId(2),
        Order::MoveUnit {
            unit: UnitId(2),
            to: destination,
        },
    );
    p.plan_primitive_escorts();
    assert!(p.orders.contains(&Order::MoveUnit {
        unit: UnitId(2),
        to: destination
    }));
}

#[test]
fn snipe_settlers_uses_nearest_free_unit_and_respects_escort_radius_and_peace() {
    let v = ViewBuilder { view: view() }
        .unit(1, ME, UnitKind::Spearman, h(5, 5))
        .unit(2, ME, UnitKind::Spearman, h(8, 5))
        .unit(3, ENEMY, UnitKind::Settler, h(9, 5))
        .build();
    let g = PrimitiveParams::default();
    let memory = PrimitiveMemory::default();
    let mut p = plan(&v, &g, &memory);
    p.plan_primitive_snipes();
    assert_eq!(
        p.orders,
        vec![Order::MoveUnit {
            unit: UnitId(2),
            to: h(9, 5)
        }]
    );
    let mut guarded = ViewBuilder { view: v.clone() }
        .unit(4, ENEMY, UnitKind::Spearman, h(9, 6))
        .build();
    let mut p = plan(&guarded, &g, &memory);
    p.plan_primitive_snipes();
    assert!(p.orders.is_empty());
    guarded.units.retain(|u| u.id != UnitId(4));
    guarded.diplomacy.legacy = false;
    guarded.diplomacy.relations = vec![relation(TeamId(0), TeamId(2), RelationState::Peace)];
    let mut p = plan(&guarded, &g, &memory);
    p.plan_primitive_snipes();
    assert!(
        p.orders.is_empty(),
        "never capture a peaceful civilian without a legal declaration"
    );
    assert!(
        !p.snipes(Some(TeamId(2)), false).is_empty(),
        "the peace opportunity contributes war score"
    );
    guarded
        .units
        .iter_mut()
        .find(|u| u.id == UnitId(3))
        .unwrap()
        .owner = PlayerId::BARBARIAN;
    let mut p = plan(&guarded, &g, &memory);
    p.plan_primitive_snipes();
    assert!(!p.orders.is_empty());
    let mut small = g;
    small.snipe_settlers_radius = 1;
    guarded
        .units
        .iter_mut()
        .find(|u| u.id == UnitId(2))
        .unwrap()
        .pos = h(6, 5);
    assert!(
        plan(&guarded, &small, &memory)
            .snipes(None, true)
            .is_empty()
    );
}

#[test]
fn scout_upkeep_rebuilds_before_deadline_and_for_stale_intel_but_not_fresh_intel() {
    let mut v = ViewBuilder { view: view() }
        .city(2, ENEMY, h(18, 5), 3)
        .build();
    let g = PrimitiveParams::default();
    let mut memory = PrimitiveMemory::default();
    memory.observe(&v);
    let p = plan(&v, &g, &memory);
    assert_eq!(
        p.primitive_priority_production(&v.cities[0]),
        Some(unit(UnitKind::Scout))
    );
    v.turn = g.scout_upkeep_until as u32 + 1;
    memory.observe(&v);
    assert_eq!(plan(&v, &g, &memory).primitive_scout_target_count(), 0);
    v.cities[1].visible = false;
    v.turn += g.scout_upkeep_intel_age as u32 + 1;
    memory.observe(&v);
    let p = plan(&v, &g, &memory);
    assert_eq!(p.primitive_scout_target_count(), 1);
    assert_eq!(
        p.primitive_priority_production(&v.cities[0]),
        Some(unit(UnitKind::Scout))
    );
    v = ViewBuilder { view: v }
        .unit(1, ME, UnitKind::Scout, h(6, 5))
        .build();
    let p = plan(&v, &g, &memory);
    assert!(p.primitive_scout_target(&v.units[0]).is_some());
    assert_eq!(
        p.primitive_priority_production(&v.cities[0]),
        None,
        "do not rebuild a living scout"
    );
}

#[test]
fn save_and_burst_reserves_until_trigger_then_buys_force_or_ready_settler() {
    let mut v = ViewBuilder { view: view() }
        .city(2, ME, h(5, 13), 3)
        .unit(1, ME, UnitKind::Spearman, h(5, 5))
        .gold(1000)
        .build();
    for c in &mut v.cities {
        for option in &mut c.detail.as_mut().unwrap().options {
            option.buy_cost = Some(option.cost * 2);
        }
    }
    let g = PrimitiveParams::default();
    let memory = PrimitiveMemory::default();
    let mut p = plan(&v, &g, &memory);
    assert!(p.gold_reserve() > 0);
    p.ctx.site_score.fill(None);
    assert!(!p.plan_primitive_buy());
    assert!(p.orders.is_empty());
    p.strike_target = Some(TeamId(2));
    p.plan_primitive_buy();
    assert_eq!(
        p.orders
            .iter()
            .filter(|o| matches!(o, Order::BuyItem { item, .. } if *item == unit(UnitKind::Archer)))
            .count(),
        2
    );
    assert!(p.gold_left < v.economy.gold);
    let mut p = plan(&v, &g, &memory);
    p.plan_primitive_buy();
    assert_eq!(
        p.orders
            .iter()
            .filter(
                |o| matches!(o, Order::BuyItem { item, .. } if *item == unit(UnitKind::Settler))
            )
            .count(),
        1
    );
    let mut off = g;
    off.save_and_burst_weight = 0;
    assert_eq!(plan(&v, &off, &memory).primitive_gold_reserve(), None);
}

#[test]
fn weakest_target_compares_local_forces_and_city_strength_not_whole_empires() {
    let mut v = ViewBuilder { view: view() }
        .city(2, ENEMY, h(10, 5), 2)
        .city(3, PlayerId(3), h(20, 5), 2)
        .unit(1, ME, UnitKind::Spearman, h(8, 5))
        .unit(2, ME, UnitKind::Spearman, h(9, 5))
        .unit(3, ME, UnitKind::Spearman, h(19, 5))
        .unit(4, ENEMY, UnitKind::Pikeman, h(27, 12))
        .build();
    let g = PrimitiveParams {
        weakest_target_distance: 3,
        ..PrimitiveParams::default()
    };
    let memory = PrimitiveMemory::default();
    assert_eq!(
        plan(&v, &g, &memory)
            .primitive_weakest_city(None)
            .unwrap()
            .0
            .id,
        CityId(2)
    );
    v.cities[1].strength = 200;
    assert_eq!(
        plan(&v, &g, &memory)
            .primitive_weakest_city(None)
            .unwrap()
            .0
            .id,
        CityId(3)
    );
    for c in v.cities.iter_mut().filter(|c| c.owner != ME) {
        c.visible = false;
    }
    assert!(plan(&v, &g, &memory).primitive_weakest_city(None).is_none());
}

fn coalition_view() -> PlayerView {
    let mut v = ViewBuilder { view: view() }
        .city(2, ENEMY, h(15, 5), 3)
        .unit(1, ME, UnitKind::Spearman, h(5, 5))
        .unit(2, ME, UnitKind::Spearman, h(5, 7))
        .unit(3, ME, UnitKind::Archer, h(6, 7))
        .unit(4, ME, UnitKind::Archer, h(7, 7))
        .build();
    v.diplomacy.legacy = false;
    let mut ally = relation(TeamId(0), TeamId(3), RelationState::Alliance);
    ally.security = Some(SecurityTreaty {
        id: 1,
        started_turn: 1,
        protected_until: 11,
        coalition_target: Some(TeamId(2)),
    });
    v.diplomacy.relations = vec![ally, relation(TeamId(0), TeamId(2), RelationState::War)];
    v.standings = [(0, 2), (2, 8), (3, 3)]
        .into_iter()
        .map(|(t, vp)| legatum_core::view::TeamStanding {
            team: TeamId(t),
            members: vec![PlayerId(t)],
            vp,
            target: 10,
            victory_hold: None,
        })
        .collect();
    v
}

#[test]
fn coalition_march_dispatches_quota_before_garrisons_and_never_dogpiles_a_tie() {
    let mut v = coalition_view();
    let g = PrimitiveParams {
        coalition_march_force_percent: 100,
        ..PrimitiveParams::default()
    };
    let memory = PrimitiveMemory::default();
    let mut p = plan(&v, &g, &memory);
    p.plan_primitive_coalition_march();
    let march = p.primitive_march.as_ref().expect("actual field orders");
    assert_eq!(march.minimum, 4);
    assert!(march.units.len() >= march.minimum);
    assert!(
        p.has_acted(UnitId(1)),
        "capital garrison can join the field force"
    );
    assert!(p.orders.iter().all(|o| matches!(o, Order::MoveUnit { .. })));
    v.standings[2].vp = 8;
    let mut p = plan(&v, &g, &memory);
    p.plan_primitive_coalition_march();
    assert!(p.orders.is_empty());
    v.standings[2].vp = 3;
    v.standings[1].vp = 6;
    assert!(
        plan(&v, &g, &memory).primitive_coalition_target().is_none(),
        "threshold is strictly exceeded"
    );
}

#[test]
fn siege_persistence_only_protects_low_hp_visible_city_with_an_actual_besieger() {
    let mut v = ViewBuilder { view: view() }
        .city(2, ENEMY, h(12, 5), 3)
        .unit(1, ME, UnitKind::Archer, h(10, 5))
        .build();
    let g = PrimitiveParams::default();
    let memory = PrimitiveMemory::default();
    v.cities[1].hp = 50;
    assert!(plan(&v, &g, &memory).primitive_persist_siege(TeamId(2)));
    v.cities[1].hp = 150;
    assert!(!plan(&v, &g, &memory).primitive_persist_siege(TeamId(2)));
    v.cities[1].hp = 50;
    v.cities[1].visible = false;
    assert!(!plan(&v, &g, &memory).primitive_persist_siege(TeamId(2)));
    v.cities[1].visible = true;
    v.units[0].pos = h(5, 5);
    assert!(!plan(&v, &g, &memory).primitive_persist_siege(TeamId(2)));
}

#[test]
fn barbarian_settler_guard_preserves_progress_and_releases_garrison_when_raid_ends() {
    let mut v = ViewBuilder { view: view() }
        .unit(1, ME, UnitKind::Spearman, h(6, 5))
        .unit(2, PlayerId::BARBARIAN, UnitKind::Archer, h(7, 5))
        .unit(3, PlayerId::BARBARIAN, UnitKind::Spearman, h(7, 6))
        .build();
    let d = v.cities[0].detail.as_mut().unwrap();
    d.producing = Some(unit(UnitKind::Settler));
    d.production_stored = 65;
    d.turns_to_complete = Some(10);
    let g = PrimitiveParams::default();
    let memory = PrimitiveMemory::default();
    let mut p = plan(&v, &g, &memory);
    assert!(p.primitive_barbarian_guard(0));
    p.plan_cities();
    assert!(
        !p.orders
            .iter()
            .any(|o| matches!(o, Order::SetProduction { city, .. } if *city == CityId(1))),
        "{:?}",
        p.orders
    );
    p.plan_primitive_barbarian_guards();
    assert!(p.orders.contains(&Order::MoveUnit {
        unit: UnitId(1),
        to: h(5, 5)
    }));
    v.units.retain(|u| !u.owner.is_barbarian());
    let mut p = plan(&v, &g, &memory);
    p.plan_primitive_barbarian_guards();
    assert!(p.orders.is_empty());
    assert_eq!(v.cities[0].detail.as_ref().unwrap().production_stored, 65);
}

#[test]
fn market_first_economy_weights_only_gold_that_repays_within_horizon() {
    let v = view();
    let mut g = PrimitiveParams {
        market_first_economy_horizon: 200,
        ..PrimitiveParams::default()
    };
    let memory = PrimitiveMemory::default();
    let c = &v.cities[0];
    let market = c
        .detail
        .as_ref()
        .unwrap()
        .options
        .iter()
        .find(|o| o.item == super::super::building(BuildingKind::Market))
        .unwrap();
    let granary = c
        .detail
        .as_ref()
        .unwrap()
        .options
        .iter()
        .find(|o| o.item == super::super::building(BuildingKind::Granary))
        .unwrap();
    let p = plan(&v, &g, &memory);
    let score = p.primitive_production_bonus(c, market);
    assert!(score > 0);
    assert_eq!(p.primitive_production_bonus(c, granary), 0);
    g.market_first_economy_weight = 200;
    assert_eq!(
        plan(&v, &g, &memory).primitive_production_bonus(c, market),
        score * 2
    );
    g.market_first_economy_horizon = 2;
    assert_eq!(
        plan(&v, &g, &memory).primitive_production_bonus(c, market),
        0
    );
}

#[test]
fn a_bank_repays_its_cost_with_the_full_stacked_gold_tier() {
    let mut v = ViewBuilder { view: view() }
        .known_techs(&[TechId::Currency, TechId::Banking])
        .city(2, ME, h(5, 13), 5)
        .build();
    let d = v.cities[1].detail.as_mut().unwrap();
    d.buildings.push(BuildingKind::Market);
    let opt = d
        .options
        .iter_mut()
        .find(|o| o.item == super::super::building(BuildingKind::Bank))
        .unwrap();
    opt.turns = 20;
    opt.buy_cost = Some(200);
    let g = PrimitiveParams {
        market_first_economy_horizon: 200,
        ..PrimitiveParams::default()
    };
    let memory = PrimitiveMemory::default();
    let c = &v.cities[1];
    let opt = c
        .detail
        .as_ref()
        .unwrap()
        .options
        .iter()
        .find(|o| o.item == super::super::building(BuildingKind::Bank))
        .unwrap();
    // Four new gold for 80 turns repays 200. Subtracting the retained Market's
    // three gold would incorrectly reject this profitable upgrade.
    assert!(plan(&v, &g, &memory).primitive_production_bonus(c, opt) > 0);
}

#[test]
fn primitive_genes_append_to_original_schema_and_default_requires_explicit_opt_in() {
    let genes = OpeningPolicy::genes();
    assert_eq!(genes[24].name, "escort");
    assert_eq!(genes[25].name, "primitives.escort_civilians_weight");
    assert_eq!(genes.len(), 25 + PrimitiveParams::genes().len());
    let v = view();
    let make = || ScriptedBrain::new(v.faction_of(ME), 7, ME);
    let mut off = OpeningBrain::new(make(), OpeningPolicy::builtin(Opening::Default));
    assert_eq!(off.orders(&v), make().orders(&v));
    let mut on = OpeningBrain::new(make(), OpeningPolicy::builtin(Opening::Default));
    on.primitives_enabled = true;
    let orders = on.orders(&v);
    assert!(orders.contains(&Order::SetProduction {
        city: CityId(1),
        item: unit(UnitKind::Scout)
    }));
    assert_eq!(
        on.orders(&v),
        orders,
        "repeat planning does not advance memory"
    );
    assert!(
        on.switches.is_empty(),
        "the flag does not install an Economic opening"
    );
}
