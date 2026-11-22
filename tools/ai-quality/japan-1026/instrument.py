#!/usr/bin/env python3
"""Instrument a DISPOSABLE checkout for offline war diagnosis; never the live engine.

Usage: instrument.py CHECKOUT
Prints JSON lines on stderr. Snapshots preserve orders; no decision is changed.
Use with war_position, restored pre-plan memory, and the original engine commit.
"""
import re
import sys
from pathlib import Path

root = Path(sys.argv[1])
p = root / 'crates/ai/src/military.rs'
s = p.read_text()
start = s.index('    pub fn plan_military(')
end = s.index('\n    ///', start)
part = s[start:end]
part = re.sub(r'(?m)^(\s*)self\.(\w+)\(([^;\n]*)\);',
              lambda m: m[0] + f'\n{m[1]}self.audit_war_phase("{m[2]}");', part)
s = s[:start] + part + s[end:]
start = s.index('    pub fn plan_military(')
s = s[:start] + '''    pub(crate) fn audit_war_phase(&self, phase: &str) {
        eprintln!("AUDIT {}", serde_json::json!({
            "turn": self.view().turn, "player": self.view().me,
            "phase": phase, "orders": self.orders,
            "campaign": self.campaigns.active,
            "candidates": (phase == "choose_plan").then(|| self.plan_candidates().into_iter().map(|c| serde_json::json!({"kind": c.kind, "value": c.value, "terms": c.terms})).collect::<Vec<_>>()),
            "outlooks": (phase == "choose_plan").then(|| self.view().players.iter().filter(|p| p.alive && p.team != self.view().team).map(|p| serde_json::json!({"team": p.team, "campaign": format!("{:?}", self.campaign_against(p.team))})).collect::<Vec<_>>()),
        }));
    }

''' + s[start:]
p.write_text(s)
p = root / 'crates/ai/src/scripted.rs'
s = p.read_text().replace('let mut p = self.planner_with(view, true);', 'let affordable = crate::supply::affordable(view); let mut p = self.planner_with(&affordable, true);')
for call in ['choose_plan', 'plan_siege_buys', 'choose_stance', 'dedupe_first_steps', 'enforce_levy']:
    needle = f'        p.{call}();'
    assert needle in s, call
    s = s.replace(needle, needle + f'\n        p.audit_war_phase("{call}");')
p.write_text(s)
p = root / 'crates/ai/src/siege.rs'
s = p.read_text()
needle = '            let margin = mine * 100 / their.max(1);'
assert needle in s
s = s.replace(needle, needle + '''
            eprintln!("AUDIT {}", serde_json::json!({
                "turn": v.turn, "player": v.me, "phase": "siege_ratio",
                "early": early, "city": c.id, "mine": mine, "their": their,
                "margin": margin, "need": need, "shooters": shooters,
                "near": near.iter().map(|u| u.id).collect::<Vec<_>>(),
            }));''')
needle = '            let Some(to) = spot else { continue };'
assert needle in s
s = s.replace(needle, '''            eprintln!("AUDIT {}", serde_json::json!({
                "turn": self.view().turn, "player": self.view().me,
                "phase": "siege_spot", "early": early, "city": city, "unit": u.id,
                "hp": u.hp, "mine": mine, "spot": spot,
                "candidates": rings.iter().flat_map(|r| city.ring(*r)).map(|h| serde_json::json!({
                    "pos": h, "free": self.free_tile_for(u, h),
                    "reachable": h == u.pos || self.ctx.reachable(u.pos, h),
                    "los": !shooter || self.ranged_line_clear(h, city),
                    "damage_now": self.city_damage_now(u),
                    "max_city_damage": self.max_city_damage_at(u, h),
                    "exposure": self.enemy_exposure_key(h).0,
                })).collect::<Vec<_>>(),
            }));
''' + needle)
p.write_text(s)
print('Instrumented offline checkout', root)
