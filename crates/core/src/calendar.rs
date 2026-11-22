//! The display calendar (designer, 2026-11-14): a year shown beside the turn number, "Turn 81 ·
//! 1040 AD". It is a pure function of the turn number and never feeds a rule, so it adds no
//! game state and no save shape. The anchors live in `calendar.json`; years between two anchors
//! are interpolated linearly in integer maths, BC years are negative and there is no year 0 (1 BC
//! is followed by 1 AD).

use crate::types::TechId;
use std::sync::OnceLock;

const ANCHORS_JSON: &str = include_str!("calendar.json");

/// The last anchored turn; beyond it the calendar runs one year per turn.
pub const CALENDAR_END_TURN: u32 = 300;

fn anchors() -> &'static [(i64, i64)] {
    static ANCHORS: OnceLock<Vec<(i64, i64)>> = OnceLock::new();
    ANCHORS.get_or_init(|| {
        let v: serde_json::Value =
            serde_json::from_str(ANCHORS_JSON).expect("calendar.json is valid JSON");
        v["anchors"]
            .as_array()
            .expect("calendar.json has anchors")
            .iter()
            .map(|a| {
                (
                    a["turn"].as_i64().expect("anchor turn"),
                    a["year"].as_i64().expect("anchor year"),
                )
            })
            .collect()
    })
}

/// The calendar year of a turn as a historical year: negative is BC, there
/// is no 0 (the step from -1 to 1 takes one year).
pub fn calendar_year(turn: u32) -> i32 {
    let a = anchors();
    let t = i64::from(turn.max(1));
    let (first_t, first_y) = a[0];
    let (last_t, last_y) = a[a.len() - 1];
    if t <= first_t {
        return first_y as i32;
    }
    if t >= last_t {
        return unaxis(axis(last_y) + (t - last_t)) as i32;
    }
    let i = a.iter().rposition(|&(at, _)| at <= t).unwrap_or(0);
    let ((at, ay), (bt, by)) = (a[i], a[i + 1]);
    let (ya, yb) = (axis(ay), axis(by));
    unaxis(ya + (yb - ya) * (t - at) / (bt - at)) as i32
}

/// A year on a continuous axis that has a year 0 (1 BC = 0 is not used: -1 stays -1 and the AD
/// years shift down one), so interpolation across the BC/AD boundary skips no year or day.
fn axis(y: i64) -> i64 {
    if y > 0 { y - 1 } else { y }
}

fn unaxis(y: i64) -> i64 {
    if y >= 0 { y + 1 } else { y }
}

/// "4000 BC" or "1040 AD".
pub fn format_year(year: i32) -> String {
    if year < 0 {
        format!("{} BC", -year)
    } else {
        format!("{year} AD")
    }
}

/// The label for a turn: "1040 AD".
pub fn calendar_label(turn: u32) -> String {
    format_year(calendar_year(turn))
}

/// The year each technology really arrived (BC negative), for the designers' tech-pace gauge:
/// the sim report sets the year a tech is learned in a typical game beside it. Only years that
/// are well established are listed; every technology missing here has no clean single year
/// (or is Future) and reads as unknown. Never read by the rules.
const TECH_REAL_YEARS: &[(TechId, i32)] = &[
    (TechId::Irrigation, -6000),
    (TechId::Writing, -3200),
    (TechId::BronzeWorking, -3300),
    (TechId::Masonry, -3000),
    (TechId::IronWorking, -1200),
    (TechId::Currency, -600),
    (TechId::Philosophy, -500),
    (TechId::Mathematics, -300),
    (TechId::CodeOfLaws, -1754),
    (TechId::Education, 1088),
    (TechId::Banking, 1397),
    (TechId::PrintingPress, 1440),
    (TechId::Industrialization, 1769),
    (TechId::Railways, 1825),
    (TechId::Steel, 1856),
    (TechId::Combustion, 1876),
    (TechId::Electricity, 1879),
    (TechId::ReplaceableParts, 1800),
    (TechId::Flight, 1903),
    (TechId::ArmouredWarfare, 1916),
    (TechId::Rocketry, 1926),
    (TechId::JetEngines, 1939),
];

/// The year a technology really arrived, if it is listed above.
pub fn tech_real_year(tech: TechId) -> Option<i32> {
    TECH_REAL_YEARS
        .iter()
        .find(|(t, _)| *t == tech)
        .map(|(_, y)| *y)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn anchors_are_exact() {
        for &(t, y) in anchors() {
            assert_eq!(i64::from(calendar_year(t as u32)), y, "turn {t}");
        }
    }

    #[test]
    fn monotonic_and_no_year_zero() {
        let mut prev = i32::MIN;
        for t in 1..=300 {
            let y = calendar_year(t);
            assert!(y >= prev, "turn {t}");
            assert_ne!(y, 0, "turn {t}");
            prev = y;
        }
    }

    #[test]
    fn continues_one_year_per_turn() {
        assert_eq!(calendar_year(301), calendar_year(300) + 1);
        assert_eq!(calendar_year(400), calendar_year(300) + 100);
    }

    #[test]
    fn real_years() {
        assert_eq!(tech_real_year(TechId::Flight), Some(1903));
        assert_eq!(tech_real_year(TechId::FusionPower), None);
    }

    #[test]
    fn labels() {
        assert_eq!(calendar_label(1), "4000 BC");
        assert_eq!(calendar_label(250), "2026 AD");
        assert_eq!(format_year(-1), "1 BC");
    }

    #[test]
    fn bc_to_ad_skips_zero() {
        // Between turn 30 (1000 BC) and 60 (500 AD) the years cross the BC/AD boundary.
        let years: Vec<i32> = (30..=60).map(calendar_year).collect();
        assert!(years.contains(&-1) || years.contains(&1));
        assert!(!years.contains(&0));
    }
}
