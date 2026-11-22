//! Renders the road layout (`legatum_core::roads`) on real AI games as SVG, for eyeballing.
//!
//!   cargo run --release -p legatum-sim --example roads_debug -- <out_dir> <players> <width> <height> <turn> <seed> [hub=capital|far|late] [later_turns]
//!
//! Plays the scripted AIs to `turn`, lays roads rooted at a hub city, writes `<out_dir>/roads-*.svg`.
//! With `later_turns`, it plays on that many more turns and extends the network (cities founded later).
use legatum_core::hex::Hex;
use legatum_core::roads;
use legatum_core::state::GameState;
use legatum_core::types::Terrain;
use legatum_core::{AiDifficulty, Faction, GameSetup};
use legatum_wasm::session::{BrainKind, Session};

fn colour(t: Terrain) -> &'static str {
    match t {
        Terrain::Ocean => "#9db6c0",
        Terrain::Coast => "#bdd0d4",
        Terrain::Grassland => "#c9d3a0",
        Terrain::Plains => "#dcd29c",
        Terrain::Forest => "#9fb27a",
        Terrain::Hills => "#c4ad84",
        Terrain::Desert => "#e6d6a4",
        Terrain::Tundra => "#d7dad2",
        Terrain::Mountains => "#8f877b",
    }
}

fn pos(h: Hex, r: f64) -> (f64, f64) {
    let (c, row) = h.to_offset();
    let w = 3f64.sqrt() * r;
    (
        w * (c as f64 + if row & 1 == 1 { 0.5 } else { 0.0 }) + w,
        1.5 * r * row as f64 + r * 1.2,
    )
}

/// Adjacent road tiles with no link between them: roads running side by side.
fn parallel_pairs(state: &GameState) -> usize {
    let mut n = 0;
    for h in state.map.hexes() {
        for (d, nb) in h.neighbors().into_iter().enumerate() {
            if d < 3
                && state.map.is_road(h)
                && state.map.is_road(nb)
                && !roads::linked(&state.map, h, nb)
            {
                n += 1;
            }
        }
    }
    n
}

fn render(state: &GameState, hub: Hex, label: &str) -> String {
    let r = 9.0;
    let w = 3f64.sqrt() * r * (state.map.width as f64 + 1.5);
    let h = 1.5 * r * state.map.height as f64 + 3.0 * r;
    let mut s = format!(
        "<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 {w:.0} {:.0}' width='{w:.0}' height='{:.0}'>",
        h + 14.0,
        h + 14.0
    );
    s += &format!(
        "<rect width='100%' height='100%' fill='#efe6cf'/><text x='6' y='{:.0}' font-size='11' font-family='sans-serif'>{label}</text>",
        h + 8.0
    );
    let palette = [
        "#b3402e", "#2e6bb3", "#3a8a4a", "#8a4ab3", "#c08a1e", "#2eaab3", "#b33a7a", "#555555",
    ];
    for hex in state.map.hexes() {
        let t = state.map.get(hex).unwrap();
        let (x, y) = pos(hex, r);
        let pts: Vec<String> = (0..6)
            .map(|i| {
                let a = std::f64::consts::PI / 180.0 * (60.0 * i as f64 - 30.0);
                format!("{:.1},{:.1}", x + r * a.cos(), y + r * a.sin())
            })
            .collect();
        let stroke = t
            .owner
            .map(|o| palette[o.0 as usize % 8])
            .unwrap_or("#00000010");
        s += &format!(
            "<polygon points='{}' fill='{}' stroke='{}' stroke-opacity='0.35' stroke-width='0.6'/>",
            pts.join(" "),
            colour(t.terrain),
            stroke
        );
    }
    for hex in state.map.hexes() {
        let t = state.map.get(hex).unwrap();
        let (x, y) = pos(hex, r);
        for (d, nb) in hex.neighbors().into_iter().enumerate() {
            if t.road & (1 << d) != 0 && d < 3 {
                let (nx, ny) = pos(nb, r);
                s += &format!(
                    "<line x1='{x:.1}' y1='{y:.1}' x2='{nx:.1}' y2='{ny:.1}' stroke='#3b2a1a' stroke-width='2' stroke-linecap='round'/>"
                );
            }
        }
    }
    for c in state.cities.values() {
        let (x, y) = pos(c.pos, r);
        let fill = palette[c.owner.0 as usize % 8];
        let road = state.map.is_road(c.pos);
        s += &format!(
            "<circle cx='{x:.1}' cy='{y:.1}' r='{}' fill='{fill}' stroke='{}' stroke-width='1.6'/>",
            if c.pos == hub { 6.5 } else { 4.5 },
            if road { "#111" } else { "#ff00ff" }
        );
        if c.pos == hub {
            s += &format!(
                "<text x='{x:.1}' y='{:.1}' font-size='9' text-anchor='middle' fill='white'>H</text>",
                y + 3.0
            );
        }
    }
    s + "</svg>"
}

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let (out, players, w, h, turn, seed) = (
        &a[1],
        a[2].parse::<usize>().unwrap(),
        a[3].parse::<i32>().unwrap(),
        a[4].parse::<i32>().unwrap(),
        a[5].parse::<u32>().unwrap(),
        a[6].parse::<u64>().unwrap(),
    );
    let mode = a.get(7).map_or("capital", String::as_str);
    let later: u32 = a.get(8).map_or(0, |s| s.parse().unwrap());
    let mut setup = GameSetup::all_ai(seed);
    setup.players.truncate(players);
    let mut factions = Faction::ALL;
    factions.rotate_left((seed % 8) as usize);
    for (p, f) in setup.players.iter_mut().zip(factions) {
        p.faction = f;
        p.difficulty = AiDifficulty::Hard;
    }
    (setup.width, setup.height) = (w, h);
    let mut session = Session::new(&setup, |_| BrainKind::Scripted);
    session.history = None;
    while session.state.turn < turn && !session.state.finished {
        let orders = session.ai_orders();
        session.step(&orders);
    }
    let mut state = session.state.clone();
    let hub = {
        let cities: Vec<_> = state.cities.values().collect();
        let cap = cities.iter().find(|c| c.owner.0 == 0).unwrap().pos;
        match mode {
            "far" => {
                cities
                    .iter()
                    .max_by_key(|c| (c.pos.distance(cap), c.id))
                    .unwrap()
                    .pos
            }
            _ => cap,
        }
    };
    let positions = |s: &GameState| s.cities.values().map(|c| c.pos).collect::<Vec<_>>();
    let pos_all = positions(&state);
    let n = roads::connect(&mut state.map, hub, &pos_all);
    let label = format!(
        "{players}p {w}x{h} seed {seed} turn {} {mode}: {} cities, {n} joined, {} road tiles, {} unlinked adjacent pairs",
        state.turn,
        state.cities.len(),
        state.map.tiles.iter().filter(|t| t.is_road()).count(),
        parallel_pairs(&state)
    );
    std::fs::create_dir_all(out).unwrap();
    let name = format!("{out}/roads-{players}p-{w}x{h}-{seed}-{mode}");
    std::fs::write(format!("{name}.svg"), render(&state, hub, &label)).unwrap();
    if later > 0 {
        // Play on with the roads in place: cities founded later join.
        session.state = state;
        for _ in 0..later {
            let orders = session.ai_orders();
            session.step(&orders);
        }
        let mut state = session.state.clone();
        let before = state.map.tiles.iter().filter(|t| t.is_road()).count();
        let pos_all = positions(&state);
        let n = roads::connect(&mut state.map, hub, &pos_all);
        let label = format!(
            "{players}p {w}x{h} seed {seed} turn {} later: {} cities, {n} joined, {} -> {} road tiles",
            state.turn,
            state.cities.len(),
            before,
            state.map.tiles.iter().filter(|t| t.is_road()).count()
        );
        std::fs::write(format!("{name}-later.svg"), render(&state, hub, &label)).unwrap();
    }
}
