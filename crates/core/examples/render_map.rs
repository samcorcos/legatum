//! Render a generated planet to a PNG.
//!
//! Usage: `cargo run -p legatum-core --example render_map -- <seed> <out.png>`
//!
//! No image dependencies: hexes are rasterized with 3x supersampling and written with a tiny
//! PNG encoder (zlib "stored" blocks, no compression).

use legatum_core::hex::Hex;
use legatum_core::state::{GameSetup, Map};
use legatum_core::types::{Faction, Terrain};
use std::time::Instant;

/// Hex radius (center to corner) in output pixels.
const RADIUS: f64 = 11.0;
const SUPERSAMPLE: usize = 3;
const PAD: f64 = 14.0;
const SQRT3: f64 = 1.732_050_808;

type Rgb = [f64; 3];

fn hex_rgb(c: u32) -> Rgb {
    [
        ((c >> 16) & 0xff) as f64,
        ((c >> 8) & 0xff) as f64,
        (c & 0xff) as f64,
    ]
}

fn terrain_color(t: Terrain, water_depth: i32) -> Rgb {
    match t {
        Terrain::Ocean => match water_depth {
            ..=2 => hex_rgb(0x24_3a5e),
            3 => hex_rgb(0x1e_3152),
            _ => hex_rgb(0x1a_2946),
        },
        Terrain::Coast => hex_rgb(0x3b_7f8e),
        Terrain::Grassland => hex_rgb(0x7d_a05a),
        Terrain::Plains => hex_rgb(0xae_a15c),
        Terrain::Forest => hex_rgb(0x41_6b3c),
        Terrain::Hills => hex_rgb(0x94_7a56),
        Terrain::Desert => hex_rgb(0xe8_d4a2),
        Terrain::Tundra => hex_rgb(0xc4_cdca),
        Terrain::Mountains => hex_rgb(0x8f_8b88),
    }
}

const PLAYER_COLORS: [u32; 8] = [
    0xff_f4d6, 0xff_d36b, 0xff_6b6b, 0x6b_d4ff, 0xc3_8bff, 0xff_9e4a, 0x7c_ffb2, 0xff_7ad9,
];

fn mix(a: Rgb, b: Rgb, t: f64) -> Rgb {
    [
        a[0] + (b[0] - a[0]) * t,
        a[1] + (b[1] - a[1]) * t,
        a[2] + (b[2] - a[2]) * t,
    ]
}

fn scale(a: Rgb, k: f64) -> Rgb {
    [a[0] * k, a[1] * k, a[2] * k]
}

/// Pixel-space center of a hex.
fn center(h: Hex) -> (f64, f64) {
    let (col, row) = h.to_offset();
    let x = PAD + RADIUS * SQRT3 * (col as f64 + 0.5 + 0.5 * (row & 1) as f64);
    let y = PAD + RADIUS + RADIUS * 1.5 * row as f64;
    (x, y)
}

/// The hex containing pixel-space point (x, y).
fn hex_at(x: f64, y: f64) -> Hex {
    // Undo the origin used by `center` for hex (0, 0).
    let px = x - PAD - RADIUS * SQRT3 * 0.5;
    let py = y - PAD - RADIUS;
    let qf = (SQRT3 / 3.0 * px - py / 3.0) / RADIUS;
    let rf = (2.0 / 3.0 * py) / RADIUS;
    let sf = -qf - rf;
    let (mut q, mut r, s) = (qf.round(), rf.round(), sf.round());
    let (dq, dr, ds) = ((q - qf).abs(), (r - rf).abs(), (s - sf).abs());
    if dq > dr && dq > ds {
        q = -r - s;
    } else if dr > ds {
        r = -q - s;
    }
    Hex::new(q as i32, r as i32)
}

/// Distance from a point (relative to the hex center) to the hex boundary; positive inside.
fn edge_distance(dx: f64, dy: f64) -> f64 {
    let apothem = RADIUS * SQRT3 / 2.0;
    let mut m: f64 = 0.0;
    for k in 0..3 {
        let a = std::f64::consts::PI / 3.0 * k as f64;
        m = m.max((dx * a.cos() + dy * a.sin()).abs());
    }
    apothem - m
}

/// Distance from each water tile to the nearest land (0 for land).
fn water_depth(map: &Map) -> Vec<i32> {
    let mut depth = vec![i32::MAX; map.len()];
    let mut frontier: Vec<usize> = (0..map.len())
        .filter(|&i| map.tiles[i].terrain.is_land())
        .collect();
    for &i in &frontier {
        depth[i] = 0;
    }
    let mut d = 0;
    while !frontier.is_empty() {
        d += 1;
        let mut next = Vec::new();
        for i in frontier {
            for nb in map.hex_at(i).neighbors() {
                if let Some(j) = map.index(nb)
                    && depth[j] == i32::MAX
                {
                    depth[j] = d;
                    next.push(j);
                }
            }
        }
        frontier = next;
    }
    depth
}

/// Color of one sample point, or None for background.
fn shade(map: &Map, depth: &[i32], starts: &[Hex], x: f64, y: f64) -> Option<Rgb> {
    let h = hex_at(x, y);
    let tile = map.get(h)?;
    let i = map.index(h)?;
    let (cx, cy) = center(h);
    let (dx, dy) = (x - cx, y - cy);
    let edge = edge_distance(dx, dy);
    let base = terrain_color(tile.terrain, depth[i]);

    // Soft bevel: a touch lighter toward the upper left, darker at the rim.
    let rim = (1.0 - edge / (RADIUS * 0.87)).clamp(0.0, 1.0);
    let light = 1.0 + 0.06 * (-(dx + dy) / RADIUS) - 0.10 * rim * rim;
    let mut c = scale(base, light);

    // Glyphs.
    match tile.terrain {
        Terrain::Mountains => {
            // A peak with a lit left face, shadowed right face and a snow cap.
            let top = -RADIUS * 0.62;
            let base_y = RADIUS * 0.45;
            if dy > top && dy < base_y {
                let half = (dy - top) / (base_y - top) * RADIUS * 0.62;
                if dx.abs() < half {
                    c = if dx < 0.0 {
                        hex_rgb(0xb8_b4ae)
                    } else {
                        hex_rgb(0x6e_6a67)
                    };
                    if dy < top + (base_y - top) * 0.32 {
                        c = if dx < 0.0 {
                            hex_rgb(0xf2_f2ee)
                        } else {
                            hex_rgb(0xcf_d3d6)
                        };
                    }
                }
            }
        }
        Terrain::Hills => {
            for (ox, oy, rr) in [(-0.3, 0.15, 0.36), (0.3, 0.2, 0.3)] {
                let (ex, ey) = (dx - ox * RADIUS, dy - oy * RADIUS);
                let d = (ex * ex + ey * ey).sqrt();
                if ey < 0.0 && (d - rr * RADIUS).abs() < 0.9 {
                    c = hex_rgb(0x6a_5438);
                }
            }
        }
        Terrain::Forest => {
            for (ox, oy) in [(-0.33, 0.18), (0.33, 0.18), (0.0, -0.28)] {
                let (ex, ey) = (dx - ox * RADIUS, dy - oy * RADIUS);
                let d = (ex * ex + ey * ey).sqrt();
                if d < RADIUS * 0.24 {
                    c = mix(
                        hex_rgb(0x2a_4a28),
                        hex_rgb(0x5a_8a4e),
                        (-(ex + ey) / RADIUS).max(0.0) * 2.0,
                    );
                }
            }
        }
        Terrain::Desert => {
            let ripple = ((dy + 0.25 * dx) / RADIUS * 3.0 * std::f64::consts::PI).sin();
            if ripple > 0.7 && edge > 2.0 {
                c = scale(c, 0.88);
            }
        }
        Terrain::Tundra => {
            let (ex, ey) = (dx.rem_euclid(5.0) - 2.5, dy.rem_euclid(5.0) - 2.5);
            if ex * ex + ey * ey < 0.8 && edge > 2.0 {
                c = hex_rgb(0xee_f2f2);
            }
        }
        _ => {}
    }

    // Ruin: a gold diamond with a dark outline.
    if tile.ruin {
        let d = dx.abs() + dy.abs();
        if d < RADIUS * 0.5 {
            c = if d < RADIUS * 0.37 {
                mix(
                    hex_rgb(0xff_e08a),
                    hex_rgb(0xc9_8f1c),
                    ((dx + dy) / RADIUS + 0.4).clamp(0.0, 1.0),
                )
            } else {
                hex_rgb(0x3a_2a10)
            };
        }
    }

    // Start: a bright ring in the seat's color.
    if let Some(seat) = starts.iter().position(|s| *s == h) {
        let d = (dx * dx + dy * dy).sqrt();
        let ring = RADIUS * 0.62;
        if (d - ring).abs() < 1.6 {
            c = hex_rgb(PLAYER_COLORS[seat % PLAYER_COLORS.len()]);
        } else if (d - ring).abs() < 2.6 {
            c = hex_rgb(0x10_1418);
        } else if d < RADIUS * 0.2 {
            c = hex_rgb(PLAYER_COLORS[seat % PLAYER_COLORS.len()]);
        }
    }

    // Hairline between tiles: stronger where terrain changes.
    if edge < 0.7 {
        let other = hex_at(cx + dx * 1.15, cy + dy * 1.15);
        let differs = map.get(other).is_none_or(|t| t.terrain != tile.terrain);
        c = scale(c, if differs { 0.72 } else { 0.9 });
    }
    Some(c)
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 3 {
        eprintln!("usage: render_map <seed> <out.png>");
        std::process::exit(2);
    }
    let seed: u64 = args[1].parse().expect("seed must be an integer");
    let out = &args[2];

    let setup = GameSetup::standard(seed, [Faction::Switzerland, Faction::India]);
    let t0 = Instant::now();
    let ((map, starts), retries) = legatum_core::mapgen::generate_counted(&setup);
    let gen_time = t0.elapsed();

    let depth = water_depth(&map);
    let background = hex_rgb(0x11_1a2c);
    let width = (PAD * 2.0 + RADIUS * SQRT3 * (map.width as f64 + 0.5)).ceil() as usize;
    let height =
        (PAD * 2.0 + RADIUS * 2.0 + RADIUS * 1.5 * (map.height - 1) as f64).ceil() as usize;
    let mut pixels = vec![0u8; width * height * 3];
    let ss = SUPERSAMPLE as f64;
    for py in 0..height {
        for px in 0..width {
            let mut acc = [0.0; 3];
            for sy in 0..SUPERSAMPLE {
                for sx in 0..SUPERSAMPLE {
                    let x = px as f64 + (sx as f64 + 0.5) / ss;
                    let y = py as f64 + (sy as f64 + 0.5) / ss;
                    let c = shade(&map, &depth, &starts, x, y).unwrap_or(background);
                    for k in 0..3 {
                        acc[k] += c[k];
                    }
                }
            }
            for k in 0..3 {
                let v = acc[k] / (ss * ss);
                pixels[(py * width + px) * 3 + k] = v.round().clamp(0.0, 255.0) as u8;
            }
        }
    }
    std::fs::write(out, encode_png(width, height, &pixels)).expect("write png");

    let land = map.tiles.iter().filter(|t| t.terrain.is_land()).count();
    let ruins = map.tiles.iter().filter(|t| t.ruin).count();
    let mut counts = std::collections::BTreeMap::new();
    for t in &map.tiles {
        *counts.entry(format!("{:?}", t.terrain)).or_insert(0) += 1;
    }
    println!(
        "seed {seed}: {}x{}, land {land} ({}%), ruins {ruins}, retries {retries}, generated in {:?}",
        map.width,
        map.height,
        land * 100 / map.len(),
        gen_time
    );
    println!("terrain: {counts:?}");
    println!("wrote {out} ({width}x{height})");
}

fn crc32(chunks: &[&[u8]]) -> u32 {
    let mut crc = 0xffff_ffffu32;
    for data in chunks {
        for &b in *data {
            crc ^= b as u32;
            for _ in 0..8 {
                crc = if crc & 1 != 0 {
                    (crc >> 1) ^ 0xedb8_8320
                } else {
                    crc >> 1
                };
            }
        }
    }
    !crc
}

fn chunk(out: &mut Vec<u8>, kind: &[u8; 4], data: &[u8]) {
    out.extend_from_slice(&(data.len() as u32).to_be_bytes());
    out.extend_from_slice(kind);
    out.extend_from_slice(data);
    out.extend_from_slice(&crc32(&[kind, data]).to_be_bytes());
}

/// RGB8 PNG using uncompressed deflate blocks.
fn encode_png(width: usize, height: usize, rgb: &[u8]) -> Vec<u8> {
    let mut raw = Vec::with_capacity((width * 3 + 1) * height);
    for row in rgb.chunks(width * 3) {
        raw.push(0); // filter: none
        raw.extend_from_slice(row);
    }
    let mut z = vec![0x78, 0x01];
    let blocks: Vec<&[u8]> = raw.chunks(65_535).collect();
    for (i, block) in blocks.iter().enumerate() {
        z.push(u8::from(i + 1 == blocks.len()));
        let len = block.len() as u16;
        z.extend_from_slice(&len.to_le_bytes());
        z.extend_from_slice(&(!len).to_le_bytes());
        z.extend_from_slice(block);
    }
    let (mut a, mut b) = (1u32, 0u32);
    for &byte in &raw {
        a = (a + byte as u32) % 65_521;
        b = (b + a) % 65_521;
    }
    z.extend_from_slice(&((b << 16) | a).to_be_bytes());

    let mut ihdr = Vec::new();
    ihdr.extend_from_slice(&(width as u32).to_be_bytes());
    ihdr.extend_from_slice(&(height as u32).to_be_bytes());
    ihdr.extend_from_slice(&[8, 2, 0, 0, 0]);

    let mut png = vec![0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a];
    chunk(&mut png, b"IHDR", &ihdr);
    chunk(&mut png, b"IDAT", &z);
    chunk(&mut png, b"IEND", &[]);
    png
}
