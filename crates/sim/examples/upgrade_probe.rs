//! upgrade_probe <tape.json> <turn> <player> [modern-army-level] [view-out.json.gz] [memory-out.json.gz]:
//! replay the tape with a Hard shadow brain warmed
//! on the seat's own views, then print what each idle city weighed at <turn> and its orders.
use legatum_ai::{AiParams, Brain, ScriptedBrain};
use legatum_core::{AiDifficulty, Order, PlayerId, player_view, resolve_turn};
use legatum_wasm::tape::Tape;
fn main() {
    let a: Vec<String> = std::env::args().collect();
    let tape = Tape::from_json(&std::fs::read_to_string(&a[1]).unwrap()).unwrap();
    let turn: u32 = a[2].parse().unwrap();
    let me = PlayerId(a[3].parse().unwrap());
    let mut params = AiParams::builtin();
    if let Some(level) = a.get(4) {
        params.decisions.modern_army = level.parse().unwrap();
    }
    println!(
        "probe T{turn} P{} modern_army={}",
        me.0, params.decisions.modern_army
    );
    let mut state = tape.initial_state();
    let mut b =
        ScriptedBrain::with_params(state.player(me).faction, 1, me, AiDifficulty::Hard, &params);
    for t in tape.turns.iter().filter(|t| t.turn < turn) {
        let _ = b.orders(&player_view(&state, me));
        state = resolve_turn(&state, &t.orders).0;
    }
    let v = player_view(&state, me);
    if let Some(path) = a.get(5) {
        let file = std::fs::File::create(path).unwrap();
        let mut gzip = flate2::write::GzEncoder::new(file, flate2::Compression::best());
        serde_json::to_writer(&mut gzip, &v).unwrap();
        gzip.finish().unwrap();
    }
    if let Some(path) = a.get(6) {
        let file = std::fs::File::create(path).unwrap();
        let mut gzip = flate2::write::GzEncoder::new(file, flate2::Compression::best());
        serde_json::to_writer(
            &mut gzip,
            &serde_json::json!({"strategy": b.strategy, "memory": b.opponent_memory()}),
        )
        .unwrap();
        gzip.finish().unwrap();
    }
    let (_, trace) = b.production_diagnosis(&v);
    for t in &trace {
        let mut opts = t.options.clone();
        opts.sort_by_key(|o| -o.2);
        let top: Vec<String> = opts
            .iter()
            .take(5)
            .map(|(i, v, s)| format!("{i:?} {v}/{s}"))
            .collect();
        println!(
            "city {:?} chose {:?} gate {:?} gates {:?}\n    {}",
            t.city,
            t.chosen,
            t.early_gate,
            t.gates.iter().take(3).collect::<Vec<_>>(),
            top.join("\n    ")
        );
    }
    let (orders, full) = b.order_diagnosis(&v);
    println!("city passes {:?}", full.city_passes);
    let rec: Vec<_> = tape
        .turns
        .iter()
        .find(|t| t.turn == turn)
        .unwrap()
        .orders
        .get(&me)
        .cloned()
        .unwrap_or_default();
    println!(
        "shadow orders {} upgrades {}",
        orders.len(),
        orders
            .iter()
            .filter(|o| matches!(o, Order::UpgradeUnit { .. }))
            .count()
    );
    for o in &orders {
        if !matches!(
            o,
            Order::MoveUnit { .. } | Order::MoveUnitPath { .. } | Order::SkipUnit { .. }
        ) {
            println!("  {o:?}");
        }
    }
    println!("recorded:");
    for o in &rec {
        if !matches!(
            o,
            Order::MoveUnit { .. } | Order::MoveUnitPath { .. } | Order::SkipUnit { .. }
        ) {
            println!("  {o:?}");
        }
    }
}
