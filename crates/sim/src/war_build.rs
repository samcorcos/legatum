//! `legatum-sim war-build --states DIR [ids]`: write the war scenarios (`scenarios/war/*.json`)
//! from archived start states recorded during playtests.
//!
//! `tools/scenarios/extract-war-states.sh DIR` replays each save with the engine it was recorded
//! on (every turn checked against the recorded hash) and writes the full state before the turn the
//! campaign began. This builder expects states in the current format and then prepares scenarios:
//! - the rule switches are today's defaults (decadence off);
//! - every empire's Power window is filled with its current yields when its state has no history
//!   (a reconstruction: the old engine kept no Power history);
//! - the seat's units lose the paths and standing orders the user gave them;
//! - anything the current rules forbid (a unit standing in foreign land at Peace) is moved to the
//!   nearest legal tile, and the edit is written into the scenario's `source`.
//!
//! The builder rewrites only generated parts (state, seat, faction, objective conditions and
//! score, source, human result); titles, briefings and predictions written by hand are kept.

use legatum_core::scenario::{Condition, Metric, Objective, ScoreTerm};
use legatum_core::{
    AiDifficulty, CityId, GameState, Hex, PlayerId, UnitId, UnitKind, rules::is_military,
};
use legatum_wasm::scenario::{
    AiMemories, EncodedState, FORMAT, HumanResult, Predictions, ScenarioFile, ScorePrediction,
    VERSION, encode_memories, encode_state,
};
use std::path::PathBuf;

#[derive(clap::Parser, Debug)]
#[command(
    name = "legatum-sim war-build",
    about = "Write the war scenarios from replayed playtest states"
)]
pub struct Cli {
    /// War scenario ids to build (default: all).
    pub ids: Vec<String>,
    /// Directory written by tools/scenarios/extract-war-states.sh.
    #[arg(long)]
    pub states: PathBuf,
    #[arg(long, default_value = concat!(env!("CARGO_MANIFEST_DIR"), "/../../scenarios/war"))]
    pub dir: PathBuf,
}

#[derive(Clone)]
struct Recipe {
    id: &'static str,
    /// `<game>/T<turn>.json` under the states directory.
    state: &'static str,
    seat: u8,
    turns: u32,
    win: Vec<Condition>,
    end_on_win: bool,
    keep: Vec<Condition>,
    score: Vec<ScoreTerm>,
    human: HumanResult,
    game: &'static str,
    /// The save was recorded on an engine with today's rules (a recent playtest): its Power
    /// history is real, so decadence is not reconstructed and the source text says so.
    current_rules: bool,
    /// Treasury edit `(player, gold)` applied after `modernize`, noted in the scenario's source.
    treasury: Option<(u8, i32)>,
}

fn term(metric: Metric, weight: i32) -> ScoreTerm {
    ScoreTerm { metric, weight }
}

fn human(
    met: bool,
    turns: u32,
    units_lost: u32,
    cities_lost: u32,
    summary: &str,
    game: &str,
) -> HumanResult {
    HumanResult {
        met,
        turns,
        units_lost,
        cities_lost,
        summary: summary.into(),
        game: game.into(),
    }
}

fn offence(extra: Vec<ScoreTerm>) -> Vec<ScoreTerm> {
    let mut s = vec![
        term(Metric::ObjectiveMet, 300),
        term(Metric::TurnsLeft, 20),
        term(Metric::CitiesCaptured, 50),
        term(Metric::StartingCitiesHeld, 100),
        term(Metric::CombatUnits, 10),
    ];
    s.extend(extra);
    s
}

fn defence() -> Vec<ScoreTerm> {
    vec![
        term(Metric::StartingCitiesHeld, 100),
        term(Metric::CitiesFounded, 50),
        term(Metric::CitiesCaptured, 50),
        term(Metric::CombatUnits, 10),
    ]
}

fn recipes() -> Vec<Recipe> {
    let p = PlayerId;
    vec![
        Recipe {
            id: "war-mongol-rush",
            state: "pt6/T25.json",
            seat: 0,
            turns: 27,
            win: vec![
                Condition::RivalMaxCities {
                    rival: p(7),
                    max: 0,
                },
                Condition::RivalMaxCities {
                    rival: p(4),
                    max: 0,
                },
            ],
            end_on_win: true,
            keep: vec![Condition::HoldStartingCities],
            score: offence(vec![
                term(Metric::RivalCitiesLost(p(7)), 50),
                term(Metric::RivalCitiesLost(p(4)), 50),
            ]),
            human: human(
                true,
                26,
                1,
                0,
                "Declared on Byzantium on turn 25 and on Athens on turn 29 with two Archers and a \
                 Spearman, took Byzantium's two cities on turns 35 and 40 and Athens's two on \
                 turns 37 and 50: both empires gone by turn 51, one Archer lost.",
                "Playtest 6 (Mongols, Standard, 8 empires), engine a05bf36d",
            ),
            game: "playtest 6",
            current_rules: false,
            treasury: None,
        },
        Recipe {
            id: "war-sparta-all-out",
            state: "o86/T65.json",
            seat: 0,
            turns: 20,
            win: vec![
                Condition::RivalMaxCities {
                    rival: p(1),
                    max: 0,
                },
                Condition::RivalMaxCities {
                    rival: p(2),
                    max: 0,
                },
            ],
            end_on_win: true,
            keep: vec![Condition::HoldStartingCities],
            score: offence(vec![
                term(Metric::RivalCitiesLost(p(1)), 50),
                term(Metric::RivalCities(p(2)), -50),
            ]),
            human: human(
                true,
                20,
                2,
                0,
                "Declared on India on turn 65, took Coda on turn 69 and Tessitura on turn 76 \
                 (India gone), then the city Byzantium founded meanwhile on turn 84, the attack \
                 ordered on turn 79: the last rival gone with nothing to spare, an Archer and a \
                 Spearman lost.",
                "Playtest 8.6 (Sparta, Standard, 4 empires), engine a39d0928",
            ),
            game: "playtest 8.6",
            current_rules: false,
            treasury: None,
        },
        Recipe {
            id: "war-sparta-walls",
            state: "o84/T51.json",
            seat: 0,
            turns: 12,
            win: vec![Condition::OwnCity(CityId(6)), Condition::OwnCity(CityId(4))],
            end_on_win: true,
            keep: vec![Condition::HoldStartingCities],
            score: offence(vec![
                term(Metric::CityDamage(CityId(4)), 1),
                term(Metric::CityDamage(CityId(6)), 1),
            ]),
            human: human(
                true,
                11,
                1,
                0,
                "Declared on Germany on turn 51 with seven Spearmen, stole a Settler on turn 52, \
                 took Anvil on turn 57; Keystone finished its Walls on turn 60 mid-siege and fell \
                 on turn 61 at the cost of a Spearman.",
                "Playtest 8.4 (Sparta, Standard, 4 empires), engine 900d58c4",
            ),
            game: "playtest 8.4",
            current_rules: false,
            treasury: None,
        },
        Recipe {
            id: "war-swiss-poachers",
            state: "t85/T46.json",
            seat: 0,
            turns: 14,
            win: vec![Condition::MinCities(7)],
            end_on_win: true,
            keep: vec![Condition::HoldStartingCities],
            score: vec![
                term(Metric::ObjectiveMet, 300),
                term(Metric::TurnsLeft, 20),
                term(Metric::Cities, 50),
                term(Metric::StartingCitiesHeld, 100),
            ],
            human: human(
                true,
                10,
                0,
                0,
                "With three cities, one Spearman and three Scouts, declared on the Swiss rival \
                 and on India in the turns their Settlers walked past unescorted and took both \
                 (turns 46 and 48), founded them and his own Settlers: seven cities on turn 55.",
                "Playtest 8.5 (Switzerland, Standard, 4 empires), engine d5205047",
            ),
            game: "playtest 8.5",
            current_rules: false,
            treasury: None,
        },
        Recipe {
            id: "war-golden-horde-thin",
            state: "p892/T136.json",
            seat: 6,
            turns: 14,
            win: vec![Condition::OwnCity(CityId(24))],
            end_on_win: true,
            keep: vec![Condition::HoldStartingCities],
            score: offence(vec![
                term(Metric::CityDamage(CityId(24)), 1),
                term(Metric::RivalCitiesLost(p(0)), 100),
            ]),
            human: human(
                true,
                12,
                7,
                0,
                "The Mongol AI (Hard) had sat at war with the Swiss city of Aarau, six hexes \
                 from its own Khanbaliq, since turn 78. On turn 139 sixteen units, five of them \
                 Crossbowmen, began a siege: three or four shooters on Aarau from two hexes every \
                 turn, Aarau's HP from 200 to 0 over turns 141 to 146, the Phalangites \
                 stepping in on turn 146, the garrison shot dead and the city taken on turn \
                 147. Seven combat units lost. (This variant's Swiss treasury is 500 Gold, not the 2,235 the user left unspent.)",
                "Playtest 8.9.2 (Switzerland vs the Mongol AI, Standard, 8 empires), engine \
                 d50e709cd",
            ),
            game: "playtest 8.9.2",
            current_rules: true,
            treasury: Some((0, 500)),
        },
        Recipe {
            id: "war-sparta-nicaea",
            state: "s892/T18.json",
            seat: 0,
            turns: 8,
            win: vec![Condition::OwnCity(CityId(7))],
            end_on_win: true,
            keep: vec![Condition::HoldStartingCities],
            score: offence(vec![
                term(Metric::CityDamage(CityId(7)), 1),
                term(Metric::RivalCitiesLost(p(4)), 100),
            ]),
            human: human(
                true,
                4,
                0,
                0,
                "Sparta's capital had trained only Hoplites since turn 2. On turn 18 four of them \
                 stood one to three hexes from Byzantium's only city, Nicaea, eight hexes from \
                 home; he declared war, stole a Byzantine Settler on turn 20 and stormed Nicaea \
                 (no Walls: they had turned into an Archer on turn 20) on turn 22 without losing \
                 a unit. Byzantium was eliminated.",
                "Playtest 8.9.2 (Sparta, Standard, 8 empires), engine 28be168b",
            ),
            game: "playtest 8.9.2",
            current_rules: true,
            treasury: None,
        },
        Recipe {
            id: "war-swiss-tepid-t180",
            state: "a1025/T180.json",
            seat: 5,
            turns: 14,
            win: vec![Condition::OwnCity(CityId(14))],
            end_on_win: true,
            keep: vec![Condition::HoldStartingCities],
            score: offence(vec![
                term(Metric::CityDamage(CityId(14)), 1),
                term(Metric::RivalCitiesLost(p(0)), 100),
            ]),
            human: human(
                false,
                44,
                8,
                0,
                "The Swiss AI (Hard) declared on Athens on turn 177 and kept the war to the end \
                 of the game (turn 223) with up to 16 land units around Athens, a small walled \
                 city four hexes from Winterthur that stood empty at turn 180. It took \
                 no city. Two campaigns on it ended `insufficient_army` (turns 180 and 184); the \
                 army sat on staging tiles two to six hexes out while its one modern gun stayed \
                 at home, and eight combat units were lost over the war.",
                "Playtest 8.9.10 (Athens vs the Swiss AI, Standard, 8 empires), engine \
                 8e9433e5c",
            ),
            game: "playtest 8.9.10",
            current_rules: true,
            treasury: None,
        },
        Recipe {
            id: "war-swiss-tepid-t220",
            state: "a1025/T220.json",
            seat: 5,
            turns: 4,
            win: vec![Condition::OwnCity(CityId(14))],
            end_on_win: true,
            keep: vec![Condition::HoldStartingCities],
            score: offence(vec![
                term(Metric::CityDamage(CityId(14)), 1),
                term(Metric::RivalCitiesLost(p(0)), 100),
            ]),
            human: human(
                false,
                4,
                0,
                0,
                "Five Swiss Bombers and nine land units (an Artillery, a Rifleman and old Spearmen) \
                 stood three to five hexes from Athens, healed to 200 HP and held by one Spearman. \
                 On turn 222 the Bombers took the city to 0 HP and killed the garrison, but no \
                 ground unit stepped in; Athens completed its tenth turn holding 10 VP and won \
                 at the end of turn 223.",
                "Playtest 8.9.10 (Athens vs the Swiss AI, Standard, 8 empires), engine \
                 8e9433e5c",
            ),
            game: "playtest 8.9.10",
            current_rules: true,
            treasury: None,
        },
        Recipe {
            id: "war-india-chur-ring",
            state: "r1026/T143.json",
            seat: 5,
            turns: 20,
            win: vec![Condition::OwnCity(CityId(27))],
            end_on_win: true,
            // No "hold every city": in the first baseline 62 of 64 runs failed it because
            // other empires took Sendai (population 1), which says nothing about Chur.
            keep: vec![],
            score: offence(vec![
                term(Metric::CityDamage(CityId(27)), 1),
                term(Metric::RivalCitiesLost(p(0)), 100),
            ]),
            human: human(
                false,
                21,
                0,
                0,
                "In the designer's game Japan (at war with Switzerland since turn 95; that game \
                 ran every AI at Normal by mistake) kept a campaign aimed at Chur from turn 122: \
                 eight to thirteen Spearmen, Archers, Phalangites and an Arquebusier ringed the \
                 walled city on fortified tiles for 21 turns, 131 of 134 unit-turns spent \
                 holding, and made no attack. Chur stayed at 200 HP.",
                "Playtest 8.9.12 (Switzerland vs a Normal Japan AI, Standard, 6 empires), engine \
                 0b2ff473c",
            ),
            game: "playtest 8.9.12",
            current_rules: true,
            treasury: None,
        },
        Recipe {
            id: "war-german-air-early",
            state: "jp/T210.json",
            seat: 2,
            turns: 12,
            win: vec![Condition::MinCities(4)],
            end_on_win: false,
            keep: vec![Condition::MinCities(4)],
            score: defence(),
            human: human(
                false,
                7,
                11,
                6,
                "Japan (the user) had bought its first Bomber on turn 172 and struck German \
                 units from turn 173; by turn 204 it had four Bombers. Germany, with six cities \
                 and no Fighter, lost Leipzig on turn 211, Cologne on 214, Munich on 216 \
                 (the third city lost, the 7th turn of the window from turn 210), Berlin on 218, Hamburg \
                 on 219 and Essen on 221, which eliminated it. Germany lost eleven land units and its \
                 one Bomber; Japan lost none.",
                "Playtest 8.9.11 (Japan, Standard, 5 empires), engine d1233d48",
            ),
            game: "playtest 8.9.11",
            current_rules: true,
            treasury: None,
        },
        Recipe {
            id: "war-german-air-t219",
            state: "jp/T219.json",
            seat: 2,
            turns: 3,
            win: vec![Condition::MinCities(1)],
            end_on_win: false,
            keep: vec![Condition::MinCities(1)],
            score: defence(),
            human: human(
                false,
                3,
                0,
                2,
                "Germany's last two cities, Hamburg and Essen, with no army left. Two Bombers \
                 stripped Hamburg and an Armour walked in on turn 219; Essen fell on turn 221 \
                 to four Bombers and the same Armour, which eliminated Germany.",
                "Playtest 8.9.11 (Japan, Standard, 5 empires), engine d1233d48",
            ),
            game: "playtest 8.9.11",
            current_rules: true,
            treasury: None,
        },
        Recipe {
            id: "war-india-sparta-t142",
            state: "sj/T142.json",
            seat: 2,
            turns: 30,
            win: vec![Condition::MinCities(9)],
            end_on_win: false,
            keep: vec![],
            score: defence(),
            human: human(
                false,
                17,
                30,
                2,
                "Sparta (the user) had twelve cities and four combat units (three Spearmen and \
                 a veteran Arquebusier) against Japan's ten cities and thirteen combat units. \
                 In the 30 turns to turn 171 Japan lost 30 combat units to Sparta's none, most \
                 of them to the veteran Arquebusier as they came up to its range one by one. Kyoto \
                 fell on turn 158 (the 17th turn) and Edo on turn 169; by turn 183 Sparta held \
                 six Japanese cities, at a cost of two Field Cannons.",
                "Playtest 8.9.10 (Sparta, Standard, 4 empires), engine 8e9433e5",
            ),
            game: "playtest 8.9.10",
            current_rules: true,
            treasury: None,
        },
        Recipe {
            id: "war-sparta-next-door-t101",
            state: "ab/T101.json",
            seat: 2,
            turns: 20,
            win: vec![Condition::CapturedFrom {
                rival: p(0),
                min: 1,
            }],
            end_on_win: true,
            keep: vec![],
            score: offence(vec![term(Metric::RivalCitiesLost(p(0)), 100)]),
            human: human(
                false,
                20,
                1,
                0,
                "Sparta (the Spartan AI, Hard) declared on Athens on turn 101, when the designer \
                 had seven cities and two military units (an Arquebusier and a Musketman) \
                 and Athens Colony 7 had just been founded: pop 1, no Walls, five tiles from \
                 Sellasia. Sparta's own view showed none of his military; its campaign aimed at \
                 Phaleron and on turn 104 moved to the Mongol city of Shangdu. Twenty-nine turns \
                 at war and no attack on an Athenian city; Peace on turn 130. One Spartan combat \
                 unit died in the window.",
                "Playtest 8.9.13 (Athens vs Hard AIs, Standard, 5 empires), engine a58e80856",
            ),
            game: "playtest 8.9.13",
            current_rules: true,
            treasury: None,
        },
        Recipe {
            id: "war-mongol-bombers-t150",
            state: "ab/T150.json",
            seat: 4,
            turns: 18,
            win: vec![Condition::MinStartingCities(4)],
            end_on_win: false,
            keep: vec![Condition::MinStartingCities(4)],
            score: defence(),
            human: human(
                false,
                11,
                12,
                6,
                "The designer (Athens) was the first empire to research Flight (turn 147) and \
                 bought one Bomber on turn 148. With it and two Infantry he took Karakorum on \
                 turn 153, Almaliq 159, Bukhara 160, Khanbaliq 163, Sarai 165 and Shangdu 167: \
                 the Bomber took each city to 0 HP in one strike (aircraft ignore Walls) and an \
                 Infantry walked in. The Mongols (nine Medieval and Renaissance units, 17 Gold, \
                 no Flight, no Fighter) lost the third city on turn 160 (the 11th turn) and \
                 were eliminated on turn 167; twelve combat units died.",
                "Playtest 8.9.13 (Athens vs Hard AIs, Standard, 5 empires), engine a58e80856",
            ),
            game: "playtest 8.9.13",
            current_rules: true,
            treasury: None,
        },
        Recipe {
            id: "war-break-the-hold-t168",
            state: "ab/T168.json",
            seat: 1,
            turns: 10,
            win: vec![Condition::CapturedFrom {
                rival: p(0),
                min: 1,
            }],
            end_on_win: true,
            keep: vec![],
            score: offence(vec![term(Metric::RivalCitiesLost(p(0)), 100)]),
            human: human(
                false,
                10,
                8,
                0,
                "Byzantium (the Byzantine AI, Hard) declared on Athens on turn 166 and made Peace \
                 on turn 172 without one attack. The designer held 10 or more VP from turn 168 \
                 with sixteen cities, fourteen of them without a unit; the freshly bombed \
                 Mongol conquests were the softest (Shangdu at 0 HP, Sarai, a Great Work, at \
                 40, Khanbaliq at 80, Bukhara at 100). He won on turn 177 with 13 VP.",
                "Playtest 8.9.13 (Athens vs Hard AIs, Standard, 5 empires), engine a58e80856",
            ),
            game: "playtest 8.9.13",
            current_rules: true,
            treasury: None,
        },
        Recipe {
            id: "war-athens-two-pikemen-t112",
            state: "sw/T112.json",
            seat: 4,
            turns: 14,
            // The same two Siege Pikemen (units 25 and 33) and the Arquebusier, after Phaleron,
            // Athens and Athens Colony 8 have fallen: Athens holds six cities and three units.
            win: vec![
                Condition::MinStartingCities(4),
                Condition::UnitsGone(vec![UnitId(25), UnitId(33)]),
            ],
            end_on_win: false,
            keep: vec![Condition::MinStartingCities(4)],
            score: defence(),
            human: human(
                false,
                14,
                4,
                3,
                "The late start of the Pikemen game. Sparta (the designer) held Phaleron, Athens \
                 and Athens Colony 8 (0 HP) and stood at Eleusis and Marathon with two Siege \
                 Pikemen and an Arquebusier (36 HP); Athens had six cities and three units \
                 (two Pikemen and a Spearman) and 64 Gold. Eleusis fell on turn 118, Athens \
                 Colony 9 on 121 and Marathon on 125, leaving three of the six cities; Athens \
                 lost four combat units and killed none of his.",
                "Playtest 8.9.13 (Sparta vs Hard AIs, Standard, 5 empires), engine a58e80856",
            ),
            game: "playtest 8.9.13",
            current_rules: true,
            treasury: None,
        },
        Recipe {
            id: "war-mongols-chur-t137",
            state: "s915/T137.json",
            seat: 3,
            turns: 20,
            win: vec![Condition::CapturedFrom {
                rival: p(0),
                min: 1,
            }],
            end_on_win: true,
            keep: vec![],
            score: offence(vec![term(Metric::RivalCitiesLost(p(0)), 100)]),
            human: human(
                false,
                20,
                1,
                0,
                "The Mongols (the Mongol AI, Hard: eleven cities, twelve military units, 145 \
                 Gold, at war with Japan) declared on Switzerland on turn 137 and aimed at Chur \
                 (population 1, no Walls, four tiles from Sarai). The designer had one military \
                 unit for eight cities and 3,600 Gold; they bought an Arquebusier in Chur on turn \
                 138, the campaign ended as an insufficient army on turn 139, the plan fell back \
                 to Peace (valued -300) and the designer accepted it on turn 141. No Swiss city \
                 was attacked.",
                "Playtest 8.9.15 (Switzerland vs Hard AIs, Standard, 4 empires), engine \
                 060f78ef4",
            ),
            game: "playtest 8.9.15",
            current_rules: true,
            treasury: None,
        },
        Recipe {
            id: "war-hollow-swiss-t76",
            state: "m917/T76.json",
            seat: 4,
            turns: 20,
            win: vec![Condition::CapturedFrom {
                rival: p(0),
                min: 1,
            }],
            end_on_win: true,
            keep: vec![],
            score: offence(vec![term(Metric::RivalCitiesLost(p(0)), 100)]),
            human: human(
                false,
                20,
                0,
                0,
                "The Swiss AI (Hard: five cities, five military units, 91 Gold) had been at war \
                 with the designer's Mongols for 20 turns (he declared on turn 56). From turn 76 \
                 to 95 he held seven to ten cities with three or four military units (Horsemen) \
                 and his treasury grew from 766 to 1,432 Gold; the Swiss attacked no Mongol city. \
                 Every Swiss unit stood in or beside a Swiss city, and the Swiss plan was a \
                 Contain war on him from turn 78.",
                "Playtest 8.9.17 (Mongols vs Hard AIs, Standard, 5 empires), engine \
                 0cf2a61d3",
            ),
            game: "playtest 8.9.17",
            current_rules: true,
            treasury: None,
        },
    ]
}

/// Scenarios whose attacker is the designer's own recorded orders (`examples/war_script`): the
/// file under the states directory that `extract-war-states.sh` writes for his seat.
fn script_for(id: &str) -> Option<&'static str> {
    match id {
        "war-german-air-early" | "war-german-air-t219" => Some("jp/orders-0.json"),
        "war-sparta-next-door-t101" | "war-mongol-bombers-t150" | "war-break-the-hold-t168" => {
            Some("ab/orders-0.json")
        }
        "war-athens-two-pikemen-t112" => Some("sw/orders-0.json"),
        "war-mongols-chur-t137" => Some("s915/orders-0.json"),
        "war-hollow-swiss-t76" => Some("m917/orders-0.json"),
        _ => None,
    }
}

/// Write `scripts/<id>.orders.json`: the recorded orders for the scenario's own turns.
fn write_script(cli: &Cli, r: &Recipe, first_turn: u32) -> Result<(), String> {
    let Some(rel) = script_for(r.id) else {
        return Ok(());
    };
    let path = cli.states.join(rel);
    let text = std::fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
    let mut value: serde_json::Value =
        serde_json::from_str(&text).map_err(|e| format!("{}: {e}", path.display()))?;
    let last_turn = first_turn + r.turns - 1;
    let turns: serde_json::Map<String, serde_json::Value> = value["turns"]
        .as_object()
        .ok_or_else(|| format!("{}: no turns", path.display()))?
        .iter()
        .filter(|(t, _)| {
            t.parse::<u32>()
                .is_ok_and(|t| (first_turn..=last_turn).contains(&t))
        })
        .map(|(t, o)| (t.clone(), o.clone()))
        .collect();
    value["turns"] = turns.into();
    value["first"] = first_turn.into();
    value["last"] = last_turn.into();
    let dir = cli.dir.join("scripts");
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let mut text = serde_json::to_string_pretty(&value).expect("script JSON");
    text.push('\n');
    std::fs::write(dir.join(format!("{}.orders.json", r.id)), text).map_err(|e| e.to_string())
}

/// The nearest free tile where `owner`'s unit may stand under the current rules.
fn legal_tile(state: &GameState, owner: PlayerId, kind: UnitKind, from: Hex) -> Option<Hex> {
    (1..=6).flat_map(|r| from.ring(r)).find(|h| {
        state.map.get(*h).is_some_and(|t| {
            t.terrain.move_cost().is_some() && state.passage_allowed(owner, kind, t.owner)
        }) && state
            .city_at(*h)
            .is_none_or(|c| state.same_team(c.owner, owner))
            && !state.units_at(*h).any(|u| {
                is_military(u.kind) == is_military(kind) || !state.same_team(u.owner, owner)
            })
    })
}

/// Diplomacy from before the 2026-10-03 simplification: Pacts became plain Peace and pending
/// Pact offers and their cooldowns are dropped. Returns what was edited.
fn migrate_legacy_json(v: &mut serde_json::Value) -> Vec<String> {
    let mut edits = vec![];
    let Some(d) = v.get_mut("diplomacy").filter(|d| d.is_object()) else {
        return edits;
    };
    if let Some(relations) = d.get_mut("relations").and_then(|r| r.as_array_mut()) {
        let mut pacts = 0;
        for r in relations.iter_mut() {
            if r["state"] == "Pact" {
                r["state"] = "Peace".into();
                r["security"] = serde_json::Value::Null;
                pacts += 1;
            }
        }
        if pacts > 0 {
            edits.push(format!("{pacts} Pacts (removed 2026-10-03) became Peace"));
        }
    }
    for key in ["proposals", "peace_cooldowns"] {
        if let Some(list) = d.get_mut(key).and_then(|r| r.as_array_mut()) {
            let before = list.len();
            list.retain(|p| p["kind"] != "Pact");
            if list.len() < before {
                edits.push(format!(
                    "dropped {} pending Pact {key}",
                    before - list.len()
                ));
            }
        }
    }
    edits
}

/// Bring a replayed state up to the current rules; returns what was edited.
pub(crate) fn modernize(
    state: &mut GameState,
    seat: Option<PlayerId>,
    rebuild_power: bool,
) -> Vec<String> {
    let mut edits = vec![];
    state.setup.rule_experiments = Default::default();
    for p in &mut state.setup.players {
        p.is_human = false;
        p.difficulty = AiDifficulty::Hard;
    }
    for p in &mut state.players {
        p.is_human = false;
    }
    // Reconstruct public Power for saves recorded before the history existed.
    if rebuild_power {
        let window = usize::from(legatum_core::experiments::POWER_WINDOW_TURNS);
        let powers: Vec<(PlayerId, i32)> = state
            .players
            .iter()
            .filter(|p| p.alive)
            .map(|p| {
                let y = legatum_core::economy::player_income(state, p.id).cities;
                (p.id, y.food + y.production + y.gold + y.science)
            })
            .collect();
        for (p, power) in powers {
            state.player_mut(p).power_history = vec![power; window];
        }
        legatum_core::scoring::refresh_decadence(state);
    }
    for u in state.units.values_mut().filter(|u| Some(u.owner) == seat) {
        u.goto = None;
        u.path.clear();
        u.path_stop_on_block = false;
        u.automation = None;
        u.standing_order = None;
        u.attack_target = None;
        u.needs_orders = false;
    }
    // Units the current rules would not let stand where they are.
    let misplaced: Vec<_> = state
        .units
        .values()
        .filter(|u| {
            !u.owner.is_barbarian()
                && state
                    .map
                    .get(u.pos)
                    .is_some_and(|t| !state.passage_allowed(u.owner, u.kind, t.owner))
        })
        .map(|u| (u.id, u.owner, u.kind, u.pos))
        .collect();
    for (id, owner, kind, pos) in misplaced {
        match legal_tile(state, owner, kind, pos) {
            Some(to) => {
                let u = state.units.get_mut(&id).expect("unit");
                u.pos = to;
                u.goto = None;
                u.path.clear();
                edits.push(format!(
                    "moved {} #{} of player {} from ({},{}) to ({},{}): it stood in foreign land at Peace, which the current rules forbid",
                    legatum_core::data::unit_name(kind),
                    id.0,
                    owner.0,
                    pos.q,
                    pos.r,
                    to.q,
                    to.r
                ));
            }
            None => {
                state.units.remove(&id);
                edits.push(format!(
                    "removed {} #{} of player {} at ({},{}): it stood in foreign land at Peace with no legal tile nearby",
                    legatum_core::data::unit_name(kind),
                    id.0,
                    owner.0,
                    pos.q,
                    pos.r
                ));
            }
        }
    }
    legatum_core::visibility::update_explored(state);
    edits
}

pub(crate) fn describe(state: &GameState, seat: PlayerId) -> Vec<String> {
    let mut notes = vec![format!("turn {}", state.turn)];
    for p in state.players.iter().filter(|p| p.alive) {
        let cities: Vec<String> = state
            .cities_of(p.id)
            .map(|c| {
                format!(
                    "{} (id {}, pop {}, hp {}{})",
                    c.name,
                    c.id.0,
                    c.population,
                    c.hp,
                    if legatum_core::data::has_building(
                        &c.buildings,
                        legatum_core::BuildingKind::Walls
                    ) {
                        ", walls"
                    } else {
                        ""
                    }
                )
            })
            .collect();
        let mut units: std::collections::BTreeMap<&str, u32> = Default::default();
        for u in state.units_of(p.id) {
            *units
                .entry(legatum_core::data::unit_name(u.kind))
                .or_default() += 1;
        }
        let wars: Vec<u8> = state
            .players
            .iter()
            .filter(|q| q.alive && q.id != p.id && state.at_war(p.id, q.id))
            .map(|q| q.id.0)
            .collect();
        notes.push(format!(
            "{}player {} {:?}: gold {}, techs {}, decadence {}%, wars {:?}; cities {}; units {:?}",
            if p.id == seat { "SEAT " } else { "" },
            p.id.0,
            p.faction,
            p.gold,
            p.techs.len(),
            p.decadence_percent,
            wars,
            cities.join(", "),
            units
        ));
    }
    notes
}

fn placeholder(r: &Recipe, state: &GameState) -> ScenarioFile {
    ScenarioFile {
        format: FORMAT.into(),
        version: VERSION,
        id: r.id.into(),
        revision: 1,
        title: r.id.into(),
        faction: state.player(PlayerId(r.seat)).faction,
        seat: PlayerId(r.seat),
        difficulty: AiDifficulty::Hard,
        minutes: 20,
        briefing: "TODO".into(),
        tests: "TODO".into(),
        objective: Objective {
            summary: "TODO".into(),
            turns: r.turns,
            win: vec![],
            end_on_win: false,
            keep: vec![],
            score: vec![],
        },
        predictions: Predictions {
            good_play: "TODO".into(),
            ai_play: "TODO".into(),
            ai_score: ScorePrediction {
                median: 0,
                low: 0,
                high: 0,
                win_percent: 0,
            },
        },
        source: String::new(),
        baseline: None,
        human: None,
        lab: None,
        rusher: None,
        ai_memory: None,
        state: EncodedState {
            encoding: "gzip-base64".into(),
            hash: "0".into(),
            data: String::new(),
        },
    }
}

/// The seat whose orders `modernize` clears. Normally the scenario's seat (the user's own orders
/// are dropped); in the games where the attacker was an AI and the designer sat in seat 0, it is
/// the designer's seat, so the attacker keeps the paths and standing orders its own brain set.
/// In the scenarios that replay the designer's recorded orders on top of carried AI memory
/// nobody is cleared: the AI keeps the paths its brain set, and the designer's units keep the
/// paths he had given them (his recorded orders for the window are issued on top of those).
fn cleared_seat(id: &str, seat: PlayerId) -> Option<PlayerId> {
    if script_for(id).is_some_and(|s| !s.starts_with("jp/")) {
        None
    } else if id.starts_with("war-swiss-tepid-")
        || id == "war-india-chur-ring"
        || id.starts_with("war-hollow-")
    {
        Some(PlayerId(0))
    } else {
        Some(seat)
    }
}

/// Every AI seat's saved memory beside a state (`a1025/T210.json` -> `a1025/memory-T210.json`,
/// written by `war_memory`); None for states extracted without it.
fn saved_memories(state_path: &std::path::Path) -> Result<Option<AiMemories>, String> {
    let name = state_path
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or("bad state path")?;
    let path = state_path.with_file_name(format!("memory-{name}"));
    let Ok(text) = std::fs::read_to_string(&path) else {
        return Ok(None);
    };
    serde_json::from_str(&text)
        .map(Some)
        .map_err(|e| format!("{}: {e}", path.display()))
}

pub fn run(cli: &Cli) -> Result<(), String> {
    for r in recipes() {
        if !cli.ids.is_empty() && !cli.ids.iter().any(|id| id == r.id) {
            continue;
        }
        let path = cli.states.join(r.state);
        let json =
            std::fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
        let mut value: serde_json::Value =
            serde_json::from_str(&json).map_err(|e| format!("{}: {e}", path.display()))?;
        let mut edits = migrate_legacy_json(&mut value);
        let mut state = legatum_wasm::json::parse_state(&value.to_string())
            .map_err(|e| format!("{}: {e}", path.display()))?;
        let seat = PlayerId(r.seat);
        edits.extend(modernize(
            &mut state,
            cleared_seat(r.id, seat),
            !r.current_rules,
        ));
        if let Some((who, gold)) = r.treasury {
            let player = &mut state.players[usize::from(who)];
            edits.push(format!(
                "player {who} ({:?}) treasury {} Gold set to {gold}",
                player.faction, player.gold
            ));
            player.gold = gold;
        }
        let errors = legatum_core::scenario::validate_state(&state);
        if !errors.is_empty() {
            return Err(format!("{}: illegal state: {}", r.id, errors.join("; ")));
        }
        let out = cli.dir.join(format!("{}.json", r.id));
        let mut file = match std::fs::read_to_string(&out) {
            Ok(text) => ScenarioFile::parse(&text)?,
            Err(_) => placeholder(&r, &state),
        };
        let memory = saved_memories(&path)?.map(|m| encode_memories(&m));
        let changed = file.ai_memory != memory
            || file.state.hash != state.hash().to_string()
            || file.objective.win != r.win
            || file.objective.keep != r.keep
            || file.objective.score != r.score
            || file.objective.turns != r.turns;
        if changed && file.state.hash != "0" {
            file.revision += 1;
            file.baseline = None;
        }
        file.seat = seat;
        file.faction = state.player(seat).faction;
        file.state = encode_state(&state);
        file.ai_memory = memory;
        file.objective.turns = r.turns;
        file.objective.win = r.win.clone();
        file.objective.end_on_win = r.end_on_win;
        file.objective.keep = r.keep.clone();
        file.objective.score = r.score.clone();
        file.human = Some(r.human.clone());
        let edit_note = if edits.is_empty() {
            String::new()
        } else {
            format!(" Edits: {}.", edits.join("; "))
        };
        let memory_note = if file.ai_memory.is_some() {
            "Every AI seat restores the memory its brain had in that game (campaigns, plans, \
             assessments, replayed with the recording engine's brains: every recorded AI order \
             list reproduced and the final memory equal to the save's), and keeps the paths and \
             standing orders its own brain had set; the human player's own seat (a Hard AI) starts \
             with none"
        } else {
            "the empires' AI memory starts fresh"
        };
        file.source = if r.current_rules {
            format!(
                "A recorded {} game (recorded on today's rules), replayed to turn {} with every \
                 turn matching the recorded state hash, with its real Power history. \
                 Every seat is a Hard AI; {memory_note}, and the \
                 human player's own orders are gone. Built by `tools/scenarios/extract-war-states.sh` \
                 and `legatum-sim war-build`.{edit_note}",
                r.game, state.turn
            )
        } else {
            format!(
                "A recorded {} game, replayed with the engine it was recorded on to turn {} (every \
                 turn matched the recorded state hash), then loaded into the current rules: old \
                 technologies mapped to the seven-era tree, old units are the A2 units with the same \
                 ids and decadence off, and each empire's 50-turn Power window \
                 filled with its current yields. The \
                 human player's units lose their orders. Built by `tools/scenarios/extract-war-states.sh` \
                 and `legatum-sim war-build`.{edit_note}",
                r.game, state.turn
            )
        };
        if script_for(r.id).is_some() {
            file.source.push_str(
                " The human player's seat replays its recorded orders for the window instead of \
                 playing as an AI (`scenarios/war/scripts`, `examples/war_script`).",
            );
        }
        std::fs::create_dir_all(&cli.dir).map_err(|e| e.to_string())?;
        let mut text = serde_json::to_string_pretty(&file).expect("scenario JSON");
        text.push('\n');
        std::fs::write(&out, text).map_err(|e| e.to_string())?;
        write_script(cli, &r, state.turn)?;
        println!(
            "== {} -> {} (revision {})",
            r.id,
            out.display(),
            file.revision
        );
        for e in &edits {
            println!("edit: {e}");
        }
        for line in describe(&state, seat) {
            println!("{line}");
        }
    }
    Ok(())
}

pub fn main(args: Vec<String>) -> Result<(), String> {
    use clap::Parser;
    let cli = Cli::try_parse_from(args).map_err(|e| e.to_string())?;
    run(&cli)
}
