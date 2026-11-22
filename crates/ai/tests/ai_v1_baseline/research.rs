//! Research choice: knob-weighted value per science point.

use crate::ai_v1_baseline::scripted::{Planner, tag};
use legatum_core::{Faction, Order, TechId};

impl Planner<'_> {
    pub fn plan_research(&mut self) {
        let r = &self.view().research;
        if r.current.is_some() {
            return;
        }
        let mut best: Option<(i32, TechId)> = None;
        for opt in &r.available {
            if r.known.contains(&opt.tech) {
                continue;
            }
            let cost = opt.cost.total.max(1);
            let score = self.tech_value(opt.tech) * 1000 / cost
                + self.jitter(tag::RESEARCH, opt.tech as u64, 40);
            // Strictly greater: ties keep the earlier tech in the view's (ordered) list.
            if best.is_none_or(|(s, _)| score > s) {
                best = Some((score, opt.tech));
            }
        }
        if let Some((_, tech)) = best {
            self.orders.push(Order::SetResearch { tech });
        }
    }

    pub fn tech_value(&self, tech: TechId) -> i32 {
        self.tech_base_value(tech)
    }

    fn tech_base_value(&self, tech: TechId) -> i32 {
        let k = self.knobs;
        let threatened = self.ctx.threat.iter().any(|t| *t > 0);
        let f = self.faction;
        match tech {
            TechId::Irrigation => 40 + k.economy / 2 + k.expansion / 4,
            TechId::BronzeWorking => 40 + k.economy / 2 + k.military / 4,
            TechId::Masonry => 20 + k.military / 2 + if threatened { 60 } else { 0 },
            TechId::Currency => {
                30 + k.economy * 3 / 4 + if f == Faction::Switzerland { 20 } else { 0 }
            }
            TechId::Writing => 30 + k.science * 3 / 4,
            TechId::Feudalism => 20 + k.military / 2 + k.aggression / 2,
            TechId::Philosophy => 20 + k.science / 2 + if f == Faction::Byzantium { 80 } else { 0 },
            TechId::CodeOfLaws => 30 + k.expansion / 4 + k.economy / 4 + k.science / 4,
            _ => 0, // Archived planner predates the expanded tree.
        }
    }
}
