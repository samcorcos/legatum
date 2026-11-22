"""Faction personality knobs plus bounded tactical and macro decision genes (pillar 6).

Rust owns defaults and descriptions. Python checks an independent decision allowlist
and per-gene ranges before writing params; neither schema can express rule changes.
"""

from __future__ import annotations

import json
import re
import subprocess

KNOBS = ("expansion", "military", "science", "economy", "aggression")
KNOB_TEXT = {
    "expansion": "settler priority and target city count",
    "military": "army size and military production",
    "science": "research-oriented techs and buildings",
    "economy": "growth, production and gold buildings and techs",
    "aggression": "attack threshold (130% at 0 to 90% at 100) and marching on enemies",
}
LO, HI = 0, 100
# Defense in depth; defaults/descriptions come from the simulator catalog.
# builtin_catalog verifies these bounds against Rust before starting a run.
DECISION_RANGES = {
    "mongol_horde": (0, 3),
    "walls_garrison_value": (0, 200),
    "modernization_value": (0, 200),
    "hard_purchase_scorer": (0, 1),
    "siege_survival_clock": (0, 1),
    "production_threat_agreement": (0, 1),
    "purchase_generation_floor": (0, 1),
    "modernization_upgrade": (0, 200),
    "modernization_return": (0, 200),
    "named_fund": (0, 1),
    "capture_vp_accounting": (0, 1),
    "declare_and_strike": (0, 1),
    "recover_hp": (25, 90),
    "ranged_survival_weight": (0, 400),
    "hold_exchange_percent": (50, 175),
    "garrison_doom_percent": (50, 150),
    "supported_defence": (0, 1),
    "war_commit_ratio": (100, 250),
    "attack_ratio_percent": (70, 150),
    "counterattack_loss_percent": (0, 60),
    "melee_response_weight": (0, 300),
    "counterattack_win_percent": (40, 100),
    "staging_damage_percent": (50, 200),
    "escort_home_distance": (0, 6),
    "escort_search_distance": (2, 12),
    "settler_min_site_score": (20, 120),
    "settler_max_site_distance": (4, 20),
    "settler_travel_penalty": (0, 20),
    "settler_commit_percent": (100, 225),
    "settler_commit_bonus": (0, 80),
    "settle_patience": (8, 32),
    "border_raid": (0, 1),
    "campaign_contain_only": (0, 1),
    "scout_front": (0, 1),
    "second_front_temperament": (-60, 60),
    "second_front_siege_price": (0, 1),
    "neighbour_fall": (0, 100),
    "neighbour_alarm": (0, 1),
    "busy_leader_front": (0, 1),
    "ally_defense_percent": (0, 200),
    "ally_vision_cost": (0, 100),
    "ally_passage_cost": (0, 100),
    "ally_exit_discount_percent": (0, 100),
    "ally_commitment_model": (0, 1),
    "ally_offensive_join_percent": (0, 100),
    "open_prize_radius": (5, 16),
    "courier_raid_only": (0, 1),
    "walls_near_enemy_bonus": (0, 400),
    "bronze_bias": (0, 300),
    "irrigation_bias": (0, 300),
    "currency_bias": (0, 300),
    "banking_bias": (0, 300),
    "feudalism_bias": (0, 300),
    "chivalry_bias": (0, 300),
    "flower_wars_value": (0, 300),
    "egypt_food_buildings": (0, 1),
    "scribe_bias": (0, 300),
    "currency_tech_bonus": (0, 200),
    "industry_tech_bonus": (0, 300),
    "reveal_tech_bonus": (0, 400),
    "reveal_tech_first": (0, 1),
    "law_tech_bonus": (0, 300),
    "crop_tech_bonus": (0, 300),
    "veteran_infantry_bonus": (0, 300),
    "infantry_bonus": (0, 300),
    "roadworks_war_base": (0, 2000),
    "roadworks_war_per_enemy": (0, 300),
    "campaign_range": (6, 24),
    "early_campaign_value": (0, 400),
    "target_wall_cost": (0, 500),
    "campaign_wall_cost": (0, 400),
    "contain_extra_range": (0, 12),
    "switch_margin": (0, 120),
    "min_seat_war_odds": (0, 50),
    "recent_fight_turns": (2, 20),
    "march_turns": (2, 12),
    "siege_rounds": (1, 8),
    "approach_turns": (1, 4),
    "retaliation_radius": (3, 10),
    "siege_reach": (4, 14),
    "raid_arrival_weight": (0, 300),
    "raid_reach": (2, 12),
    "ally_progress_limit": (40, 95),
    "ally_offer_progress": (30, 90),
    "bloc_size_penalty": (0, 200),
    "war_lean": (0, 400),
    "peace_lean": (0, 300),
    "research_hysteresis": (100, 200),
    "scout_find_turns": (10, 100),
    "rush_window_turn": (40, 250),
    "opening_chain_turn": (30, 200),
    "rush_read_turn": (10, 100),
    "rival_radius": (6, 20),
    "work_plan_turn": (60, 250),
    "victory_close_turn": (30, 200),
    "endgame_victory_turn": (80, 300),
    "conquest_war_bonus": (0, 300),
    "conquest_wounded_bonus": (0, 300),
    "conquest_nearby_weight": (0, 80),
    "strike_eagerness": (-60, 60),
    "ruin_bonus": (0, 120),
    "ruin_collect": (0, 1),
    "approach_scout": (0, 1),
    "approach_scout_turn": (4, 60),
    "recon_weight": (0, 6),
    "scout_priority_bonus": (0, 400),
    "toll_market_bonus": (0, 200),
    "army_war_extra": (0, 10),
    "army_rush_extra": (0, 10),
    "army_hard_extra": (0, 8),
    "army_military_bonus": (0, 3),
    "army_banker": (0, 1),
    "chain_press_turn": (40, 300),
    "open_followup_turn": (30, 200),
    "opening_strike_turn": (20, 150),
    "lone_city_sortie_turn": (10, 120),
    "early_prize_turn": (30, 250),
    "conquest_enabled": (0, 1),
    "conquest_min_cities": (0, 6),
    "conquest_bank_goal": (0, 1),
    "conquest_runaway_percent": (20, 90),
    "conquest_walled_forecast": (0, 1),
    "lone_city_strike": (0, 1),
    "lone_city_strike_radius": (3, 12),
    "early_walled_skip": (0, 1),
    "conquest_home_distance": (6, 24),
    "conquest_path_len": (8, 32),
    "siege_exempt": (0, 1),
    "contain_march_outlook": (0, 1),
    "contain_stack_weight": (0, 100),
    "plan_persist_skip": (0, 1),
    "march_ignores_dead_war": (0, 1),
    "march_ignores_second_war": (0, 1),
    "chain_press": (0, 1),
    "open_followup": (0, 1),
    "assault_wing": (0, 1),
    "walled_prize": (0, 1),
    "engaged_assault_keep": (0, 1),
    "early_prize_opening": (0, 1),
    "army_item_bonus": (0, 300),
    "war_shrine_drive": (0, 1),
    "reserve_deficit_turns": (0, 40),
    "surplus_upkeep_turns": (10, 120),
    "surplus_purchase_percent": (100, 350),
    "purchase_min_value": (0, 150),
    "purchase_savings_turns": (0, 12),
    "frontline_purchase_radius": (2, 9),
    "research_military_percent": (40, 200),
    "research_economy_percent": (40, 200),
    "research_science_percent": (40, 200),
    "production_military_percent": (40, 200),
    "production_settler_percent": (40, 200),
    "production_building_percent": (40, 200),
    "leader_pressure_hard_percent": (15, 75),
    "leader_pressure_percent": (25, 90),
    "leader_siege_percent": (30, 90),
    "defend_margin_percent": (100, 300),
    "relieve_radius": (0, 5),
    "open_city_radius": (0, 12),
    "contain_march_radius": (0, 20),
    "protect_defenders": (0, 1),
    "faction_threat_prior": (0, 1),
    "endgame_push": (0, 1),
    "late_capture_value": (0, 200),
    "sentry": (0, 2),
    "army_growth_trigger": (0, 400),
    "rush_walls": (0, 1),
    "veteran_drill": (0, 6),
    "roman_founders": (0, 1),
    "roman_soldier_route": (0, 1),
    "roman_veteran_retreat": (0, 1),
    "roman_promotion_value": (0, 300),
    "roman_hunt_radius": (0, 30),
    "keep_walls": (0, 1),
    "army_ratio_walls": (0, 400),
    "threat_aim_slack": (0, 5),
    "keep_settler": (0, 100),
    "settler_pipeline": (1, 4),
    "settler_open_cities": (2, 8),
    "settler_priority": (0, 4000),
    "settler_chain": (0, 1),
    "settler_chain_priority": (0, 10000),
    "settler_raid": (0, 1),
    "settler_raid_range": (0, 2),
    "settler_safety": (0, 1),
    "founding_survival_weight": (0, 400),
    "peace_settler_capture": (0, 200),
    "settler_risk_escort": (1, 200),
    "settler_risk_avoid": (1, 300),
    "heir_route_risk_weight": (0, 1000),
    "lone_city_radius": (2, 8),
    "peace_follows_plan": (0, 2),
    "peace_scout_danger": (0, 1),
    "peace_exposure_percent": (0, 300),
    "threat_horizon_turns": (0, 4),
    "assessment_interval": (0, 1),
    "fog_army_forward_percent": (0, 100),
    "fog_escort_percent": (0, 300),
    "army_threat_weight": (0, 300),
    "field_strike": (0, 300),
    "scout_threat": (0, 300),
    "walled_defense_percent": (0, 300),
    "intel_value": (0, 300),
    "sentry_posts": (0, 300),
    "counter_composition": (0, 300),
    "swiss_market_priority": (0, 600),
    "strain_percent": (0, 100),
    "strain_priority": (0, 2000),
    "decadence_weight": (0, 1000),
    "movement_wait": (0, 1),
    "goal_open_end": (0, 1),
    "hall_decisive": (0, 1),
    "objective_horizon": (20, 600),
    "objective_seek": (0, 1),
    "objective_bonus_percent": (0, 300),
    "cheapest_objective": (0, 1),
    "capture_completes_count": (0, 1),
    "greece_captive_attack": (0, 300),
    "greece_captive_defend": (0, 300),
    "vow_hold": (0, 1),
    "era_entry_percent": (0, 100),
    "era_transition_scoring": (0, 1),
    "contain_leader_percent": (0, 100),
    "contain_share": (0, 1000),
    "contain_campaign": (0, 1),
    "coalition_alliance": (0, 1),
    "search_race": (0, 1),
    "campaign_march_patience": (0, 24),
    "planned_stage": (0, 1),
    "contain_commit": (0, 1),
    "contain_min_odds": (15, 70),
    "extent_guard": (0, 1),
    "stale_intel": (0, 60),
    "stale_nominee": (0, 1),
    "strike_range_exit": (0, 1),
    "capture_economy_percent": (0, 200),
    "sack_price": (0, 1),
    "sack_commit": (0, 1),
    "sack_loss_percent": (0, 200),
    "raze_value": (0, 1),
    "buy_hold": (0, 3),
    "raze_pressure_percent": (0, 400),
    "supply_value_percent": (0, 300),
    "settlement_claim_timeline": (0, 1),
    "industry_priority": (0, 1000),
    "supply_war_percent": (0, 2000),
    "supply_term": (0, 2000),
    "supply_race_cap": (0, 1),
    "supply_shortage_percent": (100, 1000),
    "relative_city_percent": (50, 200),
    "relative_economy_percent": (50, 250),
    "relative_vp_percent": (50, 200),
    "vp_race": (0, 300),
    "vp_denial": (0, 300),
    "vp_hold_break": (0, 300),
    "vp_peace": (0, 300),
    "vp_two_step": (0, 300),
    "early_campaign_muster": (0, 1),
    "sparta_conquest_bundle": (0, 1),
    "sparta_force_campaign": (0, 1),
    "sparta_valued_expedition": (0, 1),
    "sparta_search_only": (0, 1),
    "sparta_strike_min_hp": (30, 60),
    "sparta_expedition_min_hp": (30, 70),
    "sparta_march_min_hp": (10, 50),
    "sparta_capture_heal_forecast": (0, 2),
    "troop_runway": (0, 80),
    "unit_concentration_turns": (0, 30),
    "force_mix": (0, 6),
    "peace_retaliation": (0, 1),
    "rival_redeclare_percent": (0, 200),
    "peace_withdraw_price": (0, 1),
    "contain_after_campaign": (0, 1),
    "peace_credibility": (0, 1000),
    "value_first_target": (0, 2),
    "storm_forecast": (0, 2),
    "split_counterattack": (0, 1),
    "walls_under_way": (0, 1),
    "walls_before_shooter": (0, 1),
    "prior_needs_army": (0, 200),
    "assault_wing_forecast": (0, 1),
    "campaign_commit": (0, 1000),
    "peace_redeclare": (0, 2),
    "peace_pair_scope": (0, 1),
    "siege_window": (0, 8),
    "runaway_read": (0, 1),
    "leader_commit": (0, 2),
    "leader_peace_horizon": (0, 1),
    "coalition_front": (0, 1),
    "coalition_push": (0, 1),
    "walled_quota": (0, 8),
    "coalition_own_read": (0, 1),
    "celtic_campaign": (0, 1),
    "celt_sack_target": (0, 1),
    "india_network_guard": (0, 1),
    "india_remittance_value": (0, 600),
    "india_office_goal_value": (0, 1000),
    "india_last_office_value": (0, 2000),
    "india_diwan_commit": (0, 1),
    "india_emergency_defense": (0, 1),
    "india_prewar_guard": (0, 1),
    "india_early_office": (0, 1),
    "india_early_office_min_gold": (1, 6),
    "india_objective_reach": (0, 1),
    "siege_timeline": (0, 1),
    "siege_for_walls": (0, 1),
    "war_siege": (0, 300),
    "siege_scope": (0, 1),
    "siege_floor": (0, 90),
    "siege_parts": (0, 15),
    "siege_melee": (0, 1),
    "siege_early_shooters": (0, 1),
    "siege_storm_first": (0, 1),
    "siege_home_radius": (0, 8),
    "siege_shooters": (1, 6),
    "siege_walled_only": (0, 1),
    "campaign_persist": (0, 300),
    "war_garrison": (0, 1),
    "war_pressure": (0, 50),
    "dual_attack_choice": (0, 1),
    "fighter_strike": (0, 1),
    "air_wrapper_orders": (0, 1),
    "air_marginal_strikes": (0, 1),
    "air_defense": (0, 300),
    "air_stage_read": (0, 1),
    "air_rebase": (0, 1),
    "air_commitment": (0, 1),
    "phalanx_formation": (0, 1),
    "scout_find": (0, 2000),
    "military_scout": (0, 300),
    "learned_macro": (0, 3),
    "learned_value": (0, 2),
    "focus_fire": (0, 300),
    "gather_hold": (0, 200),
    "door_defense": (0, 300),
    "peace_army_sites": (0, 1),
    "army_door_production": (0, 1),
    "door_parts": (0, 63),
    "war_reaim": (0, 2),
    "hollow_read": (0, 1),
    "persist_hollow": (0, 1),
    "hollow_march": (0, 15),
    "hoard_at_war": (0, 1),
    "decisive_buy": (0, 2),
    "decisive_buy_min": (0, 200),
    "peace_own_danger": (0, 2),
    "peace_siege_value": (0, 200),
    "peace_prize": (0, 200),
    "dead_war_peace": (0, 60),
    "capture_hold": (0, 1),
    "strike_kill": (0, 1),
    "retake_hold": (0, 20),
    "found_here": (0, 2),
    "found_now": (0, 100),
    "border_explained": (0, 1),
    "fallback_risk": (0, 1),
    "shot_sweep": (0, 1),
    "capture_alarm": (0, 1),
    "peace_strong": (0, 50),
    "settler_shelter": (0, 1),
    "colony_compound": (0, 300),
    "settler_escort": (0, 2),
    "settler_stalk": (0, 4),
    "finish_kill": (0, 100),
    "stalk_contact": (0, 1),
    "settler_slow_turns": (0, 60),
    "land_contest": (0, 200),
    "site_denial": (0, 200),
    "camp_land": (0, 300),
    "contest_hold": (0, 300),
    "celts_playbook": (0, 7),
    "celts_wild_late": (30, 95),
    "march_past_reach": (0, 2),
    "upgrade_wait": (0, 1),
    "joint_assault": (0, 2),
    "joint_window": (0, 10),
    "finish_fallen": (0, 100),
    "snipe_read": (0, 100),
    "capitals": (0, 1),
    "capital_guard": (0, 1),
    "swiss_playbook": (0, 3),
    "swiss_hoard_any_war": (0, 1),
    "escort_buy": (0, 100),
    "aztecs_playbook": (0, 7),
    "walls_garrison": (0, 1),
    "walls_peace_quota": (0, 1),
    "india_playbook": (0, 6),
    "operations": (0, 1),
    "breach": (0, 400),
    "forecast_garrison_holds": (0, 2),
    "stance_plan_war": (0, 1),
    "stance_home_guard": (0, 12),
    "siege_stance": (0, 12),
    "capture_horizon": (0, 400),
    "strike_upgrade_wait": (0, 1),
    "short_objective": (0, 1),
    "relief_forecast": (0, 1),
    "charge_preview": (0, 1),
    "relief_kill": (0, 200),
    "capturer_upgrade": (0, 400),
    "emergency_bombardment_only": (0, 1),
    "refit_before_siege": (0, 3),
    "regen_strike": (0, 100),
    "runaway_coalition": (0, 63),
    "reachable_wars": (0, 1),
    "leader_war": (0, 1),
    "german_industry_food": (0, 1),
}
FACTIONS = {"Japan", "Germany", "Mongols", "Sparta", "Byzantium", "Athens", "Switzerland", "Celts"}


def builtin_params(sim: str) -> dict:
    out = subprocess.run([sim, "--print-ai-params"], check=True, capture_output=True, text=True)
    params = json.loads(out.stdout)
    check_params(params)
    return params


def builtin_catalog(sim: str) -> list[dict]:
    out = subprocess.run([sim, "--print-ai-genes"], check=True, capture_output=True, text=True)
    catalog = json.loads(out.stdout)
    for g in catalog:
        key = g["name"].removeprefix("decision.")
        if DECISION_RANGES.get(key) != (g["min"], g["max"]):
            raise ValueError(f"Rust/Python decision schema mismatch: {key}")
    return catalog


def genes_from(params: dict, catalog: list[dict] | None = None) -> list[dict]:
    """Personality and shared decision genes, in stable order, with Rust defaults."""
    genes = []
    for faction in sorted(params["factions"]):
        for knob in KNOBS:
            genes.append({
                "name": f"{faction}.{knob}",
                "min": LO,
                "max": HI,
                "default": params["factions"][faction][knob],
                "description": f"{faction} base {knob} knob (0-100): {KNOB_TEXT[knob]}. "
                               "Hard adds its usual difficulty scaling on top.",
            })
    decisions = params.get("decisions", {})
    entries = {g["name"].removeprefix("decision."): g for g in (catalog or [])}
    if set(entries) != set(decisions):
        raise ValueError("decision catalog must cover every builtin decision")
    for key, value in decisions.items():
        g = entries[key]
        if g["default"] != value:
            raise ValueError(f"builtin decision default mismatch: {key}")
    genes.extend(entries[k] for k in sorted(entries))
    return genes


def clip_unit(x: list[float]) -> list[float]:
    return [min(1.0, max(0.0, v)) for v in x]


def boundary_distance(x: list[float]) -> float:
    """Squared distance from the box; CMA-ES sees it as a penalty so the mean stays inside."""
    return sum((v - c) ** 2 for v, c in zip(x, clip_unit(x)))


def decode(x: list[float], genes: list[dict]) -> dict[str, int]:
    if len(x) != len(genes):
        raise ValueError("genome length must match the gene catalog")
    return {g["name"]: int(round(g["min"] + v * (g["max"] - g["min"]))) for g, v in zip(genes, clip_unit(x))}


def encode(values: dict[str, float], genes: list[dict]) -> list[float]:
    return [(values[g["name"]] - g["min"]) / (g["max"] - g["min"]) for g in genes]


def params_file(values: dict[str, int], name: str, source: str, faction_target: str | None = None) -> dict:
    if faction_target is not None and faction_target not in FACTIONS:
        raise ValueError(f"unknown faction target: {faction_target}")
    factions: dict[str, dict[str, int]] = {}
    decisions = {}
    overrides = {}
    for gene, v in values.items():
        faction, knob = gene.split(".")
        if faction == "decision":
            (overrides if faction_target else decisions)[knob] = v
            continue
        if knob not in KNOBS:
            raise ValueError(f"not a decision knob: {gene}")
        factions.setdefault(faction, {})[knob] = v
    params = {"schema": 1, "name": name, "source": source, "factions": factions}
    if decisions:
        params["decisions"] = decisions
    if overrides:
        params["faction_overrides"] = {faction_target: overrides}
    check_params(params)
    return params


def check_params(params: dict) -> None:
    """Pillar 6 guard: typed personality tables and bounded shared or faction decisions."""
    allowed = {"schema", "name", "source", "factions", "difficulty", "decisions", "faction_overrides"}
    extra = set(params) - allowed
    if extra:
        raise ValueError(f"AI params may only hold decision knobs; unexpected {sorted(extra)}")
    if params.get("schema") != 1 or not re.fullmatch(r"[A-Za-z0-9_.-]+", params.get("name", "")):
        raise ValueError("unsupported schema or invalid parameter name")
    if set(params.get("difficulty", {})) - {"Easy", "Normal", "Hard"}:
        raise ValueError("unknown difficulty")
    for name, value in params.get("decisions", {}).items():
        bounds = DECISION_RANGES.get(name)
        if bounds is None or type(value) is not int or not bounds[0] <= value <= bounds[1]:
            raise ValueError(f"invalid decision {name}={value!r}")
    for faction, overrides in params.get("faction_overrides", {}).items():
        if faction not in FACTIONS:
            raise ValueError(f"unknown faction override: {faction}")
        for name, value in overrides.items():
            bounds = DECISION_RANGES.get(name)
            if bounds is None or type(value) is not int or not bounds[0] <= value <= bounds[1]:
                raise ValueError(f"invalid {faction} decision override {name}={value!r}")
    tables = [params.get("factions", {})] + list(params.get("difficulty", {}).values())
    for table in tables:
        for faction, knobs in table.items():
            if faction not in FACTIONS:
                raise ValueError(f"unknown faction: {faction}")
            if set(knobs) != set(KNOBS):
                raise ValueError(f"{faction}: knobs must be exactly {KNOBS}")
            for k, v in knobs.items():
                if type(v) is not int or not LO <= v <= HI:
                    raise ValueError(f"{faction}.{k}={v!r} outside {LO}..{HI}")
