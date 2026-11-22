//! Settlers (pick and found the best site) and scouts (explore the frontier and ruins).

use crate::analysis::{THREAT_RADIUS, UNREACHABLE, forecast_moves};
use crate::scripted::{Planner, tag};
use legatum_core::path::find_path;
use legatum_core::rules::movement::can_step;
use legatum_core::settlement::{settlement_candidates_with, settlement_candidates_with_risky};
use legatum_core::view::UnitView;
use legatum_core::{Faction, Hex, Order, UnitClass, UnitId, UnitKind};

/// Gene `snipe_read`: how long a surprise war marks its declarer as a Settler sniper.
const SNIPE_RECORD_TURNS: u32 = 20;
/// Gene `snipe_read`: a sniper whose known cities and units all lie farther than this from a
/// Settler's route cannot have an unseen unit beside it.
const SNIPE_FOG_RANGE: i32 = 12;

/// A traced Settler, the optional probe tile and the notes recorded for it.
type SettlerTrace = (UnitId, Option<Hex>, Vec<String>);

thread_local! {
    /// Diagnostics only (`ScriptedBrain::settler_diagnosis`): the Settler whose planning is
    /// being explained, and the notes recorded for it. `None` in every live call.
    /// The optional probe is a second tile (such as where a human founded with this Settler)
    /// whose verdict in the site search is recorded too.
    static SETTLER_TRACE: std::cell::RefCell<Option<SettlerTrace>> =
        const { std::cell::RefCell::new(None) };
}

/// Records a note for `unit` when its planning is being traced; free otherwise.
pub(crate) fn settler_note(unit: UnitId, note: impl FnOnce() -> String) {
    SETTLER_TRACE.with(|t| {
        if let Some((id, _, notes)) = t.borrow_mut().as_mut()
            && *id == unit
        {
            notes.push(note());
        }
    });
}

/// The probe tile of `unit`'s trace, if its planning is being traced with one.
fn settler_probe(unit: UnitId) -> Option<Hex> {
    SETTLER_TRACE.with(|t| {
        t.borrow()
            .as_ref()
            .filter(|(id, _, _)| *id == unit)
            .and_then(|(_, probe, _)| *probe)
    })
}

/// Runs `f` with `unit`'s Settler planning traced (and `probe`'s verdict in the site search
/// recorded as "probe tile ...") and returns its notes.
pub(crate) fn trace_settler<R>(
    unit: UnitId,
    probe: Option<Hex>,
    f: impl FnOnce() -> R,
) -> (R, Vec<String>) {
    SETTLER_TRACE.with(|t| *t.borrow_mut() = Some((unit, probe, Vec::new())));
    let r = f();
    let notes = SETTLER_TRACE
        .with(|t| t.borrow_mut().take())
        .map(|(_, _, n)| n)
        .unwrap_or_default();
    (r, notes)
}

/// Gene `border_explained`: `settlement::border_city_risks`, except that a visible foreign
/// tile within a known city's work radius of a city of the same owner is explained by that city
/// and implies no unseen one (as `remembered_border_risks` treats remembered borders). Visible
/// empty land, impassable known terrain and spacing from known cities still rule centers out.
pub(crate) fn unexplained_border_risks(view: &legatum_core::PlayerView) -> Vec<bool> {
    use legatum_core::data::CITY_WORK_RADIUS;
    let gap = view.rule_experiments.min_city_gap(view.faction_of(view.me));
    let hex_at = |i: usize| Hex::from_offset(i as i32 % view.width, i as i32 / view.width);
    let mut centers = vec![false; view.tiles.len()];
    for (i, tile) in view.tiles.iter().enumerate() {
        let Some(owner) = tile.owner.filter(|&o| tile.visible && o != view.me) else {
            continue;
        };
        let pos = hex_at(i);
        if view
            .cities
            .iter()
            .any(|c| c.owner == owner && c.pos.distance(pos) <= CITY_WORK_RADIUS)
        {
            continue;
        }
        for center in pos.within(CITY_WORK_RADIUS) {
            let Some(j) = view.index(center) else {
                continue;
            };
            let possible = &view.tiles[j];
            if !possible.visible
                && possible
                    .terrain
                    .is_none_or(|t| t.is_land() && t.move_cost().is_some())
                && view.cities.iter().all(|c| c.pos.distance(center) > gap)
            {
                centers[j] = true;
            }
        }
    }
    let mut risks = vec![false; view.tiles.len()];
    for (i, possible) in centers.into_iter().enumerate() {
        if possible {
            for site in hex_at(i).within(gap) {
                if let Some(j) = view.index(site) {
                    risks[j] = true;
                }
            }
        }
    }
    risks
}

impl Planner<'_> {
    /// Core refreshes one extra move for any unit starting a turn on a road. The view's
    /// max_moves field contains its faction stats but not this current-tile bonus.
    pub(crate) fn turn_moves(&self, u: &UnitView) -> i32 {
        u.max_moves.max(1) + i32::from(self.view().tile(u.pos).is_some_and(|tile| tile.road > 0))
    }

    pub(crate) fn at_war(&self) -> bool {
        self.view()
            .players
            .iter()
            .any(|p| p.id != self.ctx.me && self.view().at_war(self.ctx.me, p.id))
    }

    /// A wartime settler may not cross land owned by the team it is fighting, even though
    /// ordinary military movement can enter that land during a war.
    pub(crate) fn enemy_war_tile(&self, h: Hex) -> bool {
        self.view().tile(h).is_some_and(|tile| {
            tile.owner
                .is_some_and(|owner| self.view().at_war(self.ctx.me, owner))
        })
    }

    /// Keep settlement sites outside the visible enemy army's threat radius. The shared
    /// settler danger map covers attack reach; THREAT_RADIUS adds the established city-threat
    /// buffer so a site cannot be selected just beyond that immediate reach.
    pub(crate) fn enemy_army_near(&self, h: Hex) -> bool {
        self.ctx
            .threat_units
            .iter()
            .any(|u| u.military && u.pos.distance(h) <= THREAT_RADIUS)
    }

    /// A visible army at Peace is a possible future attack on a new city, not a
    /// wartime threat. Price the two-sided loss of a colony by the local force
    /// ratio and public reasons for a declaration. This reads only this PlayerView.
    fn peace_army_site_risk(&self, site: Hex) -> i32 {
        if self.decisions.peace_army_sites == 0
            || self.difficulty != legatum_core::AiDifficulty::Hard
        {
            return 0;
        }
        let v = self.view();
        let near_power = |u: &UnitView| -> i32 {
            let weight = match u.pos.distance(site) {
                0..=2 => 100,
                3 => 75,
                4 => 50,
                5 => 35,
                6 => 25,
                7 => 20,
                8 => 10,
                _ => 0,
            };
            crate::analysis::threat_power(v, u) * weight / 100
        };
        let our_power: i32 = self
            .ctx
            .own_units
            .iter()
            .filter(|u| u.military && u.kind != UnitKind::Scout)
            .map(|u| near_power(u))
            .sum();
        let our_army: i32 = v
            .players
            .iter()
            .filter(|p| v.same_team(p.id, v.me))
            .map(|p| p.army)
            .sum();
        let mut risk = 0;
        let mut seen_teams = std::collections::BTreeSet::new();
        for rival in v
            .players
            .iter()
            .filter(|p| !p.id.is_barbarian() && !v.same_team(v.me, p.id) && !v.allied(v.me, p.id))
        {
            if !seen_teams.insert(v.team_of(rival.id)) {
                continue;
            }
            let rival_army: i32 = v
                .players
                .iter()
                .filter(|p| v.same_team(p.id, rival.id))
                .map(|p| p.army)
                .sum();
            let visible: Vec<_> = self
                .ctx
                .peace_armies
                .iter()
                .filter(|u| v.same_team(u.owner, rival.id) && u.pos.distance(site) <= 8)
                .collect();
            if visible.is_empty() {
                continue;
            }
            let seen_force: i32 = visible.iter().map(|u| near_power(u)).sum();
            let stronger = if rival_army >= our_army.saturating_mul(2) {
                25
            } else if rival_army > our_army {
                10
            } else {
                0
            };
            let contact = v
                .cities
                .iter()
                .any(|c| v.same_team(c.owner, rival.id) && c.pos.distance(site) <= 7)
                || site.within_iter(2).any(|h| {
                    v.tile(h)
                        .and_then(|t| t.owner)
                        .is_some_and(|owner| v.same_team(owner, rival.id))
                });
            let other_war = v.players.iter().any(|p| {
                p.alive
                    && !v.same_team(p.id, rival.id)
                    && !v.same_team(p.id, v.me)
                    && v.at_war(rival.id, p.id)
            });
            // A seen patrol beside a known rival city is evidence of a border
            // force, even when most of that empire's public army is just beyond
            // our sight. Count only a sixth of that public strength as deployable.
            let force = seen_force + if contact { rival_army / 6 } else { 0 };
            let chance = (15 + stronger + if contact { 25 } else { 0 }
                - if other_war { 20 } else { 0 })
            .clamp(5, 75);
            // Fifteen is the new city's own defence: a single patrol has a
            // price, while a massed army outweighs the city's opening guard.
            let superiority = force.saturating_mul(100) / (force + our_power + 15);
            risk += chance * superiority / 100;
        }
        risk.min(45)
    }

    fn peace_army_site_score(&self, site: Hex, score: i32) -> i32 {
        score - score.max(0) * 2 * self.peace_army_site_risk(site) / 100
    }

    fn wartime_border_near_route(&self, path: &[Hex]) -> bool {
        path.iter().any(|step| {
            step.within(THREAT_RADIUS)
                .into_iter()
                .any(|near| self.enemy_war_tile(near))
        })
    }

    /// Percent chance that a rival at Peace treats a free Settler as worth a surprise war.
    /// The city's distance is public and proxies the retaliation that declaration invites;
    /// a remote army can take the prize without exposing its own towns. Human opponents
    /// receive a stronger prior because the AI cannot read their private aggression.
    pub(crate) fn peace_settler_capture_chance(&self, owner: legatum_core::PlayerId) -> i32 {
        let v = self.view();
        if v.same_team(v.me, owner) || v.allied(v.me, owner) {
            return 0;
        }
        if owner.is_barbarian() && v.barbarians_spare_me() {
            return 0;
        }
        if owner.is_barbarian() || v.at_war(v.me, owner) {
            return 100;
        }
        if !crate::opportunity::legal_declaration(v, v.team_of(owner)) {
            return 0;
        }
        // Gene `snipe_read`: an empire that has just started a war from Peace has shown the
        // whole recipe in public; its units near a Settler are a near-certain capture.
        let record = if self.recent_surprise_war(owner) {
            self.decisions.snipe_read
        } else {
            0
        };
        if self.decisions.peace_settler_capture == 0 {
            return record.max(10);
        }
        let nearest_cities = v
            .cities
            .iter()
            .filter(|c| v.same_team(c.owner, owner))
            .flat_map(|theirs| {
                self.ctx
                    .own_cities
                    .iter()
                    .map(move |ours| theirs.pos.distance(ours.pos))
            })
            .min();
        // No shared frontier means the declaration is cheap for the raider. A city
        // within striking distance makes retaliation and territorial loss plausible.
        let cheap_war = match nearest_cities {
            Some(0..=6) => 20,
            Some(7..=10) => 40,
            _ => 65,
        };
        let human = v.players.iter().any(|p| p.id == owner && p.is_human);
        let aggression = if human {
            15
        } else {
            crate::knobs::Knobs::for_faction(v.faction_of(owner)).aggression / 5
        };
        ((cheap_war + aggression + 15) * self.decisions.peace_settler_capture / 100)
            .max(record)
            .clamp(0, 100)
    }

    /// Gene `snipe_read`: `owner` is a known Settler sniper. Either its team alone declared a war
    /// that is still running and took effect within the last `SNIPE_RECORD_TURNS` turns (public
    /// relations; every declaration is made from Peace, a mutual war or a war an ally joined does
    /// not count), or it has taken one of our Settlers, which it stays for the rest of the game.
    pub(crate) fn recent_surprise_war(&self, owner: legatum_core::PlayerId) -> bool {
        let v = self.view();
        self.decisions.snipe_read > 0
            && !owner.is_barbarian()
            && (self
                .opponents
                .is_some_and(|o| o.threats.settler_snipers.contains(&owner))
                || v.diplomacy.relations.iter().any(|r| {
                    r.state == legatum_core::diplomacy::RelationState::War
                        && r.declarers == [v.team_of(owner)]
                        && r.effective_turn + SNIPE_RECORD_TURNS >= v.turn
                }))
    }

    /// Gene `snipe_read`: the chance an unseen unit of a rival with a recent surprise war
    /// (`recent_surprise_war`) intercepts this trip. Its units are not in sight, so any tile
    /// of the route with an unseen tile within two hexes could hide one; a rival whose known
    /// cities and units all lie more than `SNIPE_FOG_RANGE` hexes from the route is too far
    /// to matter, and one we have never seen could be anywhere (8.9.25 India T17: the Celts
    /// met India only when its Scout took their Settler). Only public relations and our own
    /// map are read.
    pub(crate) fn snipe_fog_risk(&self, from: Hex, to: Hex) -> i32 {
        if self.decisions.snipe_read == 0 {
            return 0;
        }
        let v = self.view();
        let mut route = v
            .movement_path(v.me, UnitKind::Settler, from, to)
            .unwrap_or_default();
        route.push(from);
        if !route.iter().any(|step| {
            step.within_iter(2)
                .any(|near| v.tile(near).is_some_and(|t| !t.visible))
        }) {
            return 0;
        }
        let near_route = |h: Hex| route.iter().any(|step| step.distance(h) <= SNIPE_FOG_RANGE);
        v.players
            .iter()
            .filter(|p| {
                p.alive
                    && !v.same_team(v.me, p.id)
                    && !v.allied(v.me, p.id)
                    && !v.at_war(v.me, p.id)
                    && crate::opportunity::legal_declaration(v, p.team)
                    && self.recent_surprise_war(p.id)
            })
            .filter(|p| {
                let mut known = v
                    .cities
                    .iter()
                    .filter(|c| c.owner == p.id)
                    .map(|c| c.pos)
                    .chain(v.units.iter().filter(|u| u.owner == p.id).map(|u| u.pos))
                    .peekable();
                known.peek().is_none() || known.any(near_route)
            })
            .map(|_| self.decisions.snipe_read)
            .max()
            .unwrap_or(0)
    }

    /// Exposure along the route at the time the Settler would reach each step. An enemy at
    /// war or a barbarian weighs 100; a rival able to declare a surprise war is priced by
    /// its declaration incentive, a remembered threat weighs 10 and a camp 15.
    /// Future peaceful movement is less certain, but counts
    /// while the rival can reach the route before the Settler founds. Only our view is read.
    pub(crate) fn settler_trip_risk(&self, from: Hex, to: Hex) -> i32 {
        if !self.settler_safety() {
            return self.legacy_settler_trip_risk(from, to);
        }
        let v = self.view();
        let moves = self
            .ctx
            .own_units
            .iter()
            .find(|u| u.kind == UnitKind::Settler && u.pos == from)
            .map_or(2, |u| self.turn_moves(u));
        // The current tile is shared by every candidate site. Charge the route for
        // exposure it can actually avoid; the guard pass handles danger at the current
        // tile separately.
        let mut route = if from == to {
            vec![(from, 0)]
        } else {
            Vec::new()
        };
        let mut remaining = moves;
        let mut elapsed = 0;
        for step in v
            .movement_path(v.me, UnitKind::Settler, from, to)
            .unwrap_or_else(|| (from != to).then_some(to).into_iter().collect())
        {
            let cost = self.ctx.move_cost(step).unwrap_or(1);
            if !can_step(remaining, moves, cost) {
                elapsed += 1;
                remaining = moves;
            }
            remaining = (remaining - cost).max(0);
            route.push((step, elapsed));
        }
        let exposure = |pos: Hex, mobility: i32, range: i32, weight: i32, hostile: bool| {
            route
                .iter()
                .filter(|(h, turn)| {
                    self.ctx.reachable(pos, *h)
                        && pos.distance(*h) <= mobility.max(1) * (turn + 1) + range.max(0)
                })
                .map(|(_, turn)| {
                    // A distant army could converge, but its future position is less
                    // certain. We will replan when the Settler gets closer; a rival at
                    // Peace is discounted sooner than one already fighting us.
                    weight / (1 + turn / if hostile { 2 } else { 1 })
                })
                .max()
                .unwrap_or(0)
        };
        let mut risk = 0;
        for u in v
            .units
            .iter()
            .filter(|u| u.military && !v.same_team(v.me, u.owner) && !v.allied(v.me, u.owner))
        {
            let hostile = if u.owner.is_barbarian() {
                !v.barbarians_spare_me()
            } else {
                v.at_war(v.me, u.owner)
            };
            let weight = if hostile {
                100
            } else {
                self.peace_settler_capture_chance(u.owner)
            };
            // A rival can declare war before movement this turn. Even a ranged unit must
            // move onto the civilian to take it, so only wartime fire extends this prior.
            risk += exposure(
                u.pos,
                self.turn_moves(u),
                if hostile { u.range } else { 0 },
                weight,
                hostile,
            );
        }
        for o in &self.ctx.remembered_threats {
            let stats = v
                .rule_experiments
                .unit_stats_for(v.faction_of(o.owner), o.kind);
            risk += exposure(
                o.pos,
                forecast_moves(v, o.owner, o.kind)
                    + i32::from(v.tile(o.pos).is_some_and(|tile| tile.road > 0)),
                stats.range,
                10,
                false,
            );
        }
        for camp in v
            .barbarian_camps
            .iter()
            .filter(|_| !v.barbarians_spare_me())
        {
            risk += exposure(camp.pos, 2, 1, 15, false);
        }
        // A known rival border is a weak prior for an unseen patrol, not the same
        // evidence as a visible Scout that can intercept. Faster Mongol and
        // Spartan units, and Celtic archery, warrant more caution.
        // Ownership is public; this never consults the actual hidden units.
        let border_risk = route
            .iter()
            .flat_map(|(step, _)| {
                step.within_iter(3).filter_map(move |near| {
                    let owner = v.tile(near)?.owner?;
                    if v.same_team(v.me, owner) || v.allied(v.me, owner) {
                        return None;
                    }
                    let (radius, prior) = match v.faction_of(owner) {
                        legatum_core::Faction::Celts => (3, 8),
                        legatum_core::Faction::Mongols | legatum_core::Faction::Sparta => (3, 6),
                        _ => (2, 4),
                    };
                    (step.distance(near) <= radius).then_some(prior)
                })
            })
            .max()
            .unwrap_or(0);
        risk = risk.max(border_risk).max(self.fog_route_risk(from, to));
        risk
    }

    /// The unseen flank of a route is a chance of losing a city-sized civilian,
    /// not evidence of a particular hidden army. A known rival border makes that
    /// uncertainty more consequential; faction speed and past wars are public.
    pub(crate) fn fog_route_risk(&self, from: Hex, to: Hex) -> i32 {
        if self.decisions.fog_escort_percent == 0
            || self.difficulty != legatum_core::AiDifficulty::Hard
            || from == to
        {
            return 0;
        }
        let v = self.view();
        let route = v
            .movement_path(v.me, UnitKind::Settler, from, to)
            .unwrap_or_else(|| vec![to]);
        let unseen_steps = route
            .iter()
            .filter(|step| {
                step.within_iter(2)
                    .any(|near| v.tile(near).is_some_and(|t| !t.visible))
            })
            .count() as i32;
        if unseen_steps == 0 {
            return 0;
        }
        let mut rival_risk = 0;
        for step in &route {
            for border in step.within_iter(4) {
                let Some(owner) = v.tile(border).and_then(|t| t.owner) else {
                    continue;
                };
                if v.same_team(v.me, owner) || v.allied(v.me, owner) {
                    continue;
                }
                let distance = step.distance(border);
                let wars = self
                    .opponents
                    .and_then(|o| o.rivals.get(&owner))
                    .map_or(0, |r| r.declared_wars as i32);
                rival_risk = rival_risk.max(
                    20 + 5 * (4 - distance)
                        + 10 * i32::from(crate::opponents::rush_capable(v.faction_of(owner)))
                        + 5 * wars.min(3),
                );
            }
        }
        let prior = if rival_risk > 0 {
            rival_risk + 8 * unseen_steps.min(3)
        } else {
            5 + 3 * unseen_steps.min(5)
        };
        prior * self.decisions.fog_escort_percent / 100
    }

    fn legacy_settler_trip_risk(&self, from: Hex, to: Hex) -> i32 {
        let v = self.view();
        let near = |pos: Hex, reach: i32| {
            pos.distance(from) + pos.distance(to) <= from.distance(to) + 2 * reach
        };
        let mut risk = 0;
        for u in v
            .units
            .iter()
            .filter(|u| u.military && !v.same_team(v.me, u.owner))
        {
            let hostile = if u.owner.is_barbarian() {
                !v.barbarians_spare_me()
            } else {
                v.at_war(v.me, u.owner)
            };
            let weight = if hostile {
                100
            } else if crate::opportunity::legal_declaration(v, v.team_of(u.owner)) {
                50
            } else {
                0
            };
            let reach = if hostile {
                u.max_moves.max(u.range) + 2
            } else {
                u.max_moves.max(1) + 1
            };
            if weight > 0 && near(u.pos, reach) {
                risk += weight;
            }
        }
        for o in &self.ctx.remembered_threats {
            let moves = legatum_core::data::unit_stats(o.kind).moves;
            if near(o.pos, moves.max(1) + 1) {
                risk += 50;
            }
        }
        for camp in v
            .barbarian_camps
            .iter()
            .filter(|_| !v.barbarians_spare_me())
        {
            if near(camp.pos, 3) {
                risk += 50;
            }
        }
        risk.max(self.fog_route_risk(from, to))
    }

    /// A rival could stand beside this tile after its next move and take a Settler left on
    /// it (an enemy at war or a barbarian can attack it directly).
    pub(crate) fn snipe_exposed(&self, tile: Hex) -> bool {
        let v = self.view();
        if !self.settler_safety() {
            return v.units.iter().any(|u| {
                u.military
                    && !v.same_team(v.me, u.owner)
                    && (u.owner.is_barbarian() && !v.barbarians_spare_me()
                        || !u.owner.is_barbarian()
                            && (v.at_war(v.me, u.owner)
                                || crate::opportunity::legal_declaration(v, v.team_of(u.owner))))
                    && u.pos.distance(tile) <= u.max_moves.max(1) + 1
            });
        }
        v.index(tile).is_some_and(|i| {
            self.settler_intercept.get_or_init(|| {
                let mut exposed = vec![false; v.tiles.len()];
                for u in v.units.iter().filter(|u| {
                    u.military
                        && !v.same_team(v.me, u.owner)
                        && !v.allied(v.me, u.owner)
                        && (self.decisions.peace_settler_capture == 0
                            || (u.kind == UnitKind::Scout && self.decisions.peace_scout_danger > 0)
                            || self.peace_settler_capture_chance(u.owner)
                                >= self.decisions.settler_risk_escort)
                }) {
                    for h in u.pos.within(self.turn_moves(u) + 1) {
                        if let Some(i) = v.index(h)
                            && self.ctx.reachable(u.pos, h)
                        {
                            exposed[i] = true;
                        }
                    }
                }
                exposed
            })[i]
        })
    }

    pub fn plan_settlers(&mut self) {
        if self.decisions.found_here > 0 || self.found_now() {
            let turn = self.view().turn;
            self.campaigns
                .found_tried
                .retain(|&(_, _, t)| t + 1 >= turn);
        }
        let settlers: Vec<&UnitView> = self
            .ctx
            .own_units
            .iter()
            .copied()
            .filter(|u| u.kind == UnitKind::Settler)
            .collect();
        for s in settlers {
            self.plan_settler(s);
        }
        // Gene `rome_colonia`: a level-3 veteran founds the Colonia Victrix first.
        self.plan_colonia();
        if self.faction == Faction::Rome
            && self.hard_defense()
            && self.decisions.roman_founders > 0
            && self.ctx.own_cities.len() < 6
        {
            // Gene `roman_finish_siege`: a siege whose capture still forecasts keeps its
            // committed units; a captured city is worth more than a founded one.
            let siege = self.roman_live_siege();
            let in_siege =
                |u: &UnitView| siege.as_ref().is_some_and(|(_, ids)| ids.contains(&u.id));
            // A veteran cannot retire while wounded. Let a safe one recover before
            // ordinary military assignments spend another turn of its health.
            let healing: Vec<_> = self
                .ctx
                .own_units
                .iter()
                .copied()
                .filter(|u| {
                    u.level >= 1
                        && u.hp < legatum_core::data::UNIT_HP
                        && legatum_core::data::unit_class(u.kind) == Some(UnitClass::Melee)
                        && !self.has_acted(u.id)
                        && self.exposure_at(u.pos) == 0
                        && !self.defending(u.pos)
                        && !in_siege(u)
                })
                .collect();
            for u in healing {
                self.unit_order(u.id, Order::SkipUnit { unit: u.id });
            }
            // Gene `roman_veteran_retreat`: a wounded veteran is a city waiting on its health.
            // Under threat it steps out of reach rather than fighting on at low HP, unless it
            // is the last defender a threatened city has.
            if self.decisions.roman_veteran_retreat > 0 {
                let exposed: Vec<_> = self
                    .ctx
                    .own_units
                    .iter()
                    .copied()
                    .filter(|u| {
                        u.level >= 1
                            && u.hp < legatum_core::data::UNIT_HP
                            && legatum_core::data::unit_class(u.kind) == Some(UnitClass::Melee)
                            && !self.has_acted(u.id)
                            && self.exposure_at(u.pos) > 0
                            && self.roman_unit_can_leave(u)
                            && !in_siege(u)
                    })
                    .collect();
                for u in exposed {
                    self.retreat(u);
                }
            }
        }
        if self.faction == Faction::Rome
            && self.hard_defense()
            && self.decisions.roman_founders > 0
            && self.ctx.own_cities.len() < 6
            && self
                .ctx
                .own_units
                .iter()
                .filter(|u| u.military && u.kind != UnitKind::Scout)
                .count()
                > self.ctx.own_cities.len()
        {
            // Gene `roman_finish_siege`: a veteran the live siege still needs stays in it; a
            // spare one (the capture forecasts without it) may found.
            let siege = self.roman_live_siege();
            let mut founders: Vec<_> = self
                .ctx
                .own_units
                .iter()
                .copied()
                .filter(|u| {
                    u.hp == legatum_core::data::UNIT_HP
                        && u.level >= 1
                        && legatum_core::data::unit_class(u.kind) == Some(UnitClass::Melee)
                        && !self.has_acted(u.id)
                        && self.roman_unit_can_leave(u)
                        // Gene `rome_colonia`: the Colonia's founder, and the veteran closest
                        // to becoming it, found no ordinary city.
                        && !self.colonia_reserved(u)
                        && siege.as_ref().is_none_or(|(city, ids)| {
                            !ids.contains(&u.id) || {
                                let rest: Vec<UnitId> =
                                    ids.iter().copied().filter(|id| *id != u.id).collect();
                                self.roman_capture_holds(city, &rest)
                            }
                        })
                })
                .collect();
            founders.sort_by_key(|u| (self.unit_keep_value(u), u.id));
            if let Some(founder) = founders.first() {
                if self.decisions.roman_found_first > 0 {
                    self.roman_founder = Some(founder.id);
                }
                self.plan_settler(founder);
            }
        }
    }

    /// Gene `roman_promotion_value`: the first promotions a siege of `attack_turns` turns
    /// should give Rome's melee units in `force`. Each attacks the city once a turn
    /// (`XP_CITY_ATTACK`), and the most experienced also takes the capture
    /// (`XP_CITY_DEFEATED`). Every one that reaches level 1 can found a city.
    pub(crate) fn roman_siege_promotions(
        &self,
        force: &[legatum_core::UnitId],
        attack_turns: i32,
    ) -> i32 {
        if self.faction != Faction::Rome
            || !self.hard_defense()
            || self.decisions.roman_founders == 0
            || self.decisions.roman_promotion_value == 0
        {
            return 0;
        }
        let mut xp: Vec<i32> = self
            .ctx
            .own_units
            .iter()
            .filter(|u| {
                force.contains(&u.id)
                    && u.level == 0
                    && legatum_core::data::unit_class(u.kind) == Some(UnitClass::Melee)
            })
            .map(|u| u.xp.unwrap_or(0) as i32)
            .collect();
        xp.sort_unstable_by(|a, b| b.cmp(a));
        let attacks = attack_turns.max(1) * legatum_core::data::XP_CITY_ATTACK;
        xp.iter()
            .enumerate()
            .filter(|(i, x)| {
                let capture = if *i == 0 {
                    legatum_core::data::XP_CITY_DEFEATED
                } else {
                    0
                };
                **x + attacks + capture >= legatum_core::data::PROMOTION_XP[0]
            })
            .count() as i32
    }

    /// Gene `roman_finish_siege`: the target and committed units of Rome's campaign while its
    /// capture forecast holds on that force.
    pub(crate) fn roman_live_siege(&self) -> Option<(legatum_core::view::CityView, Vec<UnitId>)> {
        if self.faction != Faction::Rome
            || !self.hard_defense()
            || self.decisions.roman_founders == 0
            || self.decisions.roman_finish_siege == 0
        {
            return None;
        }
        let active = self.campaigns.active.as_ref()?;
        let v = self.view();
        let city = v.cities.iter().find(|c| c.id == active.target)?;
        (v.at_war(v.me, city.owner)
            && !v.same_team(v.me, city.owner)
            && self.roman_capture_holds(city, &active.committed))
        .then(|| (city.clone(), active.committed.clone()))
    }

    /// Whether `force` still forecasts the capture of `city` (the campaign's own tests). A
    /// visible city shot down to 0 HP falls to the first melee unit that walks in; the siege
    /// forecasts stop pricing it there, which is exactly when the veterans must stay.
    pub(crate) fn roman_capture_holds(
        &self,
        city: &legatum_core::view::CityView,
        force: &[UnitId],
    ) -> bool {
        (city.visible
            && city.hp <= 0
            && force.iter().any(|id| {
                self.ctx
                    .own_units
                    .iter()
                    .any(|u| u.id == *id && crate::composition::charges(u.kind))
            }))
            || self.campaign_can_take(city, force)
            || self.grind_forecast(city, force).is_some()
    }

    /// Gene `roman_soldier_route`: a Roman military founder routes and is priced as a soldier.
    pub(crate) fn roman_soldier_founder(&self, s: &UnitView) -> bool {
        s.military
            && self.faction == Faction::Rome
            && self.hard_defense()
            && self.decisions.roman_founders > 0
            && self.decisions.roman_soldier_route > 0
    }

    /// A veteran is worth a new city only after the threatened city's last field defender
    /// has another unit to cover it. Both promotion hunting and founding use this price.
    pub(crate) fn roman_unit_can_leave(&self, u: &UnitView) -> bool {
        !self.last_garrison_under_attack(u)
            && self.ctx.own_cities.iter().enumerate().all(|(i, city)| {
                if u.pos.distance(city.pos) > 2 || self.ctx.threat[i] == 0 {
                    return true;
                }
                let unit_power = crate::analysis::defense_power(self.view(), u)
                    .max(crate::analysis::attack_power(self.view(), u));
                self.local_defense(city) - unit_power >= self.ctx.threat[i]
            })
            && !self.ctx.own_cities.iter().any(|city| {
                self.exposed_city(city)
                    && u.pos.distance(city.pos) <= 3
                    && self
                        .ctx
                        .own_units
                        .iter()
                        .filter(|guard| {
                            guard.military
                                && guard.kind != UnitKind::Scout
                                && guard.pos.distance(city.pos) <= 3
                        })
                        .count()
                        <= 1
            })
    }

    fn plan_settler(&mut self, s: &UnitView) {
        if self.has_acted(s.id) {
            settler_note(s.id, || "already ordered before plan_settlers".into());
            return;
        }
        if self.unit_exposure_key(s, s.pos).0 > 0 {
            // Gene `found_here`: founding ends the exposure.
            if self.found_here_value(s).is_some() {
                self.found_here(s);
                return;
            }
            settler_note(s.id, || format!("exposed at {:?}: retreat", s.pos));
            if !self.retreat(s) {
                match self.ctx.nearest_own_city(s.pos).map(|c| c.pos) {
                    // As in `move_settler`: holding inside an enemy's reach loses it.
                    Some(home)
                        if (self.hard_defense() || self.settler_safety()) && home != s.pos =>
                    {
                        self.move_or_hold(s, home);
                    }
                    _ => {
                        self.unit_order(s.id, Order::SkipUnit { unit: s.id });
                    }
                }
            }
            return;
        }
        // A Roman veteran has already paid the combat and unit opportunity cost.
        // Once it stands on a safe, legal site, take the city now instead of
        // chasing a newly revealed higher-scoring site across the map.
        if self.faction == Faction::Rome
            && self.hard_defense()
            && self.decisions.roman_founders > 0
            && self.ctx.own_cities.len() < 6
            && self.found_here_value(s).is_some()
        {
            self.found_here(s);
            return;
        }
        let scored = self.choose_site_scored(s);
        settler_note(s.id, || format!("chosen {scored:?} from {:?}", s.pos));
        // Gene `found_here`: the search chose another tile, home or nothing; founding here
        // now is worth at least as much.
        let here = if scored.is_none_or(|(_, site)| site != s.pos) {
            self.found_here_value(s)
        } else {
            None
        };
        // Gene `found_now`: the search's site is worth only what is left once its land may be
        // gone before the Settler founds there.
        if let Some(here) = here
            && scored.is_none_or(|(value, site)| here >= self.found_now_alternative(s, value, site))
        {
            self.found_here(s);
            if let Some(trace) = &self.decision_trace {
                for row in trace.borrow_mut().iter_mut().filter(|r| {
                    r.domain == "settler_site"
                        && r.option.starts_with(&format!("unit={:?} site=", s.id))
                        && r.outcome == "selected"
                }) {
                    row.outcome = "selected_not_executed";
                    row.displaced_by = Some("found_here override".into());
                }
            }
            return;
        }
        let chosen = scored.map(|(_, h)| h);
        match chosen {
            Some(site) if site == s.pos => {
                if self.rescout_founding_site(s) {
                    settler_note(s.id, || "rescout founding site".into());
                    return;
                }
                self.settle_targets.push(site);
                // Full moves and no goto after turn 1 means it stood here last turn: most
                // likely its found order clashed with a settler we can't see (found orders
                // within 3 of each other all fail). Flip a coin so the two stop mirroring.
                let idle = s.goto.is_none() && s.moves_left >= s.max_moves;
                let wait = !self.settlement_timed_out(s)
                    && self.view().turn > 1
                    && idle
                    && self.jitter(tag::SETTLE, s.id.0 as u64, 1) == 0;
                if wait {
                    settler_note(s.id, || "coin-flip wait at site".into());
                    self.unit_order(s.id, Order::SkipUnit { unit: s.id });
                } else {
                    self.unit_order(s.id, Order::FoundCity { unit: s.id });
                }
            }
            Some(site) => {
                self.settle_targets.push(site);
                self.escort_sites.push(site);
                if self.settlement_timed_out(s)
                    && let Some((_, path)) = self.nearby_settler_route(s, site)
                {
                    self.unit_order(
                        s.id,
                        Order::MoveUnitPath {
                            unit: s.id,
                            to: site,
                            path,
                            // The route avoids settler danger: stop if blocked rather than repath into it.
                            stop_on_block: true,
                        },
                    );
                } else {
                    self.move_settler(s, site);
                }
            }
            None => {
                if let Some((ruin, path)) = self.idle_settler_ruin(s) {
                    self.unit_order(
                        s.id,
                        Order::MoveUnitPath {
                            unit: s.id,
                            to: ruin,
                            path,
                            stop_on_block: true,
                        },
                    );
                } else if let Some(home) = self.ctx.nearest_own_city(s.pos).map(|c| c.pos) {
                    self.move_settler(s, home);
                } else {
                    self.unit_order(s.id, Order::SkipUnit { unit: s.id });
                }
            }
        }
        if let Some(trace) = &self.decision_trace {
            let selected_order = self
                .orders
                .iter()
                .rev()
                .find(|order| legatum_core::rules::units::order_unit(order) == Some(s.id))
                .cloned();
            for row in trace.borrow_mut().iter_mut().filter(|r| {
                r.domain == "settler_site"
                    && r.option.starts_with(&format!("unit={:?} site=", s.id))
                    && r.outcome == "selected"
            }) {
                row.selected_order = selected_order.clone();
            }
        }
    }

    /// Gene `found_here`: the value of founding on the Settler's own tile this turn, or None
    /// when the tile is not a legal, preferred site. The site search leaves a tile out for
    /// hazards to a Settler on a trip (an enemy army within the threat radius, danger, its trip
    /// risk, exposure) and for a foreign border that could hide a city within spacing. A Settler
    /// that founds where it stands takes no trip: only an enemy that can reach it this turn
    /// still matters (founding resolves after movement), and a hidden city costs one turn.
    /// What stays is the fresh city's own risk, charged on both sides of the swing (we lose it,
    /// the captor gains it).
    ///
    /// Gene `found_now` uses the same value with three changes: the chance of capture before
    /// the found order resolves is priced per unit (`found_now_capture_percent`) instead of
    /// ruling the tile out, only empires at war count toward the fresh city's loss
    /// (barbarians never assault a city), the search's pre-Walls damage forecast is charged
    /// at war (gene `founding_survival_weight`), and the tile must pass the site search's
    /// border verdicts (`found_now_border_risk`).
    fn found_here_value(&self, s: &UnitView) -> Option<i32> {
        let now = self.found_now();
        if self.decisions.found_here == 0 && !now {
            return None;
        }
        let v = self.view();
        let h = s.pos;
        let gap = v.rule_experiments.min_city_gap(v.faction_of(self.ctx.me));
        let blocked = if self.enemy_war_tile(h) {
            Some("enemy war tile")
        } else if self.turns_remaining() <= 8 {
            Some("too late in the game")
        } else if now && self.found_now_border_risk(h) {
            Some("border risk")
        } else if self.settle_targets.iter().any(|t| t.distance(h) <= gap) {
            Some("another settler's target")
        } else if v.units.iter().any(|u| {
            u.owner != self.ctx.me
                && u.kind == UnitKind::Settler
                && u.pos.distance(h) <= v.rule_experiments.min_city_gap(v.faction_of(u.owner))
        }) {
            Some("foreign settler near")
        } else {
            None
        };
        if let Some(why) = blocked {
            settler_note(s.id, || format!("found here: {why}"));
            return None;
        }
        // A found order here last turn failed: leave the tile to the search (and its rescout).
        if self
            .campaigns
            .found_tried
            .iter()
            .any(|&(unit, at, turn)| unit == s.id && at == h && turn + 1 == v.turn)
        {
            settler_note(s.id, || "found here: failed last turn".into());
            return None;
        }
        let Some(site) = legatum_core::settlement::settlement_site(v, s, &self.settle_targets, h)
        else {
            settler_note(s.id, || "found here: not a known legal site".into());
            return None;
        };
        // Founding resolves after movement and combat: a Settler an enemy can reach this
        // turn is captured before it founds, so this tile needs the Settler safe today.
        let capture = if now {
            self.found_now_capture_percent(h)
        } else if self.settler_capturable_this_turn(h) {
            100
        } else {
            0
        };
        if capture >= 100 {
            settler_note(s.id, || "found here: capturable this turn".into());
            return None;
        }
        // Level 2 also takes land below the preferred quality: the search's alternative for
        // this Settler is then a far fallback or home, a zero-value wait.
        let quality_floor = if self.decisions.found_here >= 2 {
            1
        } else {
            self.decisions.settler_min_site_score
        };
        let roman_site = self.faction == Faction::Rome
            && self.hard_defense()
            && self.decisions.roman_founders > 0
            && site.quality > 0;
        let preferred = roman_site
            || (site.quality >= quality_floor
                && site
                    .nearest_city
                    .is_none_or(|home| home <= self.decisions.settler_max_site_distance));
        if !preferred {
            settler_note(s.id, || {
                format!("found here: not preferred (quality {})", site.quality)
            });
            return None;
        }
        // As in `choose_site`: the Iron and Coal the site claims, and its first ring.
        let score = site.score
            + 3 * crate::supply_value::site_gain(v, h, self.decisions.settlement_claim_timeline)
                * self.decisions.supply_value_percent
                / 100;
        let score = self.peace_army_site_score(h, score);
        let value = score * self.settlement_productivity(h) / 100;
        let loss = self.fresh_city_loss(h, now);
        // Captured before it founds, or founded and then lost: either way both sides swing.
        let lost = capture + (100 - capture) * loss / 100;
        let mut net = value - value.max(0) * 2 * lost / 100;
        // Gene `found_now`: the site search's own war forecast for a new city (gene
        // `founding_survival_weight`) applies to founding here too.
        if now && self.decisions.founding_survival_weight > 0 && self.at_war() {
            net -= self.founding_survival_cost(value, self.founding_damage_before_defense(h, 0));
        }
        settler_note(s.id, || {
            format!("found here: value {value} capture {capture}% loss {loss}% net {net}")
        });
        (net > 0).then_some(net)
    }

    /// Percent chance a city founded at `h` this turn falls to the visible attack: the at-war
    /// military that can reach it by next turn against a new city's defence. `empires_only`
    /// (gene `found_now`) leaves barbarians out: their raiders never assault a city.
    fn fresh_city_loss(&self, h: Hex, empires_only: bool) -> i32 {
        let v = self.view();
        let pressure: i32 = self
            .ctx
            .threat_units
            .iter()
            .filter(|e| {
                e.military
                    && e.kind != UnitKind::Scout
                    && !(empires_only && e.owner.is_barbarian())
                    && v.at_war(self.ctx.me, e.owner)
                    && e.pos.distance(h) <= e.max_moves.max(1) + e.range.max(1)
            })
            .map(|e| crate::analysis::offense_power(v, e))
            .sum();
        if pressure <= 0 {
            return 0;
        }
        let defense = legatum_core::combat::city_defense_for_rules(
            v.rule_experiments,
            v.faction_of(self.ctx.me),
            1,
            false,
            0,
            legatum_core::data::CITY_HP,
            v.era_of(self.ctx.me),
        )
        .total
        .max(0)
            * legatum_core::data::CITY_HP
            / legatum_core::data::UNIT_HP;
        ((pressure - defense).max(0) * 100 / pressure).min(100)
    }

    /// Gene `found_now` is on for this (Hard) planner.
    fn found_now(&self) -> bool {
        self.decisions.found_now > 0 && self.hard_defense()
    }

    /// Gene `found_now`: percent chance a Settler founding on `h` this turn is captured first
    /// (founding resolves after movement), the highest over the foreign military that can
    /// reach `h` this turn. Read from the public rules (`rules/barbarians.rs`): a barbarian
    /// raider takes a lone civilian within its moves, a camp's Scout only wanders and reports
    /// (its random wander could still end on the tile), and its guard holds the camp tile.
    fn found_now_capture_percent(&self, h: Hex) -> i32 {
        /// A barbarian Scout's random wander ending on the Settler's tile.
        const BARBARIAN_SCOUT_CAPTURE: i32 = 5;
        let v = self.view();
        v.units
            .iter()
            .filter(|u| {
                u.military
                    && !v.same_team(v.me, u.owner)
                    && !v.allied(v.me, u.owner)
                    && u.pos.distance(h) <= self.turn_moves(u)
                    && self.ctx.reachable(u.pos, h)
            })
            .map(|u| {
                if u.owner.is_barbarian() {
                    if v.barbarians_spare_me() || v.barbarian_camps.iter().any(|c| c.pos == u.pos) {
                        0
                    } else if u.kind == UnitKind::Scout {
                        BARBARIAN_SCOUT_CAPTURE
                    } else {
                        100
                    }
                } else if v.at_war(v.me, u.owner) {
                    100
                } else if crate::opportunity::legal_declaration(v, v.team_of(u.owner)) {
                    self.peace_settler_capture_chance(u.owner)
                } else {
                    0
                }
            })
            .max()
            .unwrap_or(0)
    }

    /// Gene `found_now`: the site search's border verdicts on `h` (the 8.9.24 Swiss T145
    /// rejection): a remembered foreign border no known city explains, or a visible one
    /// that could hide a city within spacing (only an unexplained one with `border_explained`).
    fn found_now_border_risk(&self, h: Hex) -> bool {
        let Some(i) = self.view().index(h) else {
            return true;
        };
        let visible = if self.decisions.border_explained > 0 {
            self.unexplained_border_risks
                .get_or_init(|| unexplained_border_risks(self.view()))[i]
        } else {
            legatum_core::settlement::border_city_risks(self.view())[i]
        };
        visible || self.remembered_border_risks()[i]
    }

    /// Gene `found_now`: the value of the search's choice `site`, worth `value`, less the chance
    /// its land is gone before the Settler founds there (`site_claim_percent`).
    fn found_now_alternative(&self, s: &UnitView, value: i32, site: Hex) -> i32 {
        if !self.found_now() || site == s.pos {
            return value;
        }
        let moves = s.max_moves.max(1);
        let turns = (s.pos.distance(site) + moves - 1) / moves;
        let claim = self.site_claim_percent(site, turns);
        let left = value - value.max(0) * claim / 100;
        settler_note(s.id, || {
            format!("found now: {site:?} in {turns} turns claim {claim}% value {value} -> {left}")
        });
        left
    }

    /// Gene `found_now`: percent chance the land at `site` is taken before a Settler arriving in
    /// `turns` turns founds there: 50 for a visible foreign Settler that can get within its
    /// city gap of the site by then, and for every other empire whose nearest known city lies
    /// within a Settler's reach of it (twice its city gap plus 2, as `land_contest_value`), the
    /// gene's percent a turn times `(reach + 1 - d) / reach`, over `turns + 1` turns.
    /// Only units and cities in our view count.
    pub(crate) fn site_claim_percent(&self, site: Hex, turns: i32) -> i32 {
        let v = self.view();
        let window = turns.max(0) + 1;
        let settler = v.units.iter().any(|u| {
            u.kind == UnitKind::Settler
                && !v.same_team(v.me, u.owner)
                && u.pos.distance(site)
                    <= self.turn_moves(u) * window
                        + v.rule_experiments.min_city_gap(v.faction_of(u.owner))
        });
        let mut nearest: std::collections::BTreeMap<legatum_core::PlayerId, i32> =
            std::collections::BTreeMap::new();
        for c in v
            .cities
            .iter()
            .filter(|c| !c.owner.is_barbarian() && !v.same_team(v.me, c.owner))
        {
            let d = c.pos.distance(site);
            nearest
                .entry(c.owner)
                .and_modify(|n| *n = (*n).min(d))
                .or_insert(d);
        }
        let per_turn: i32 = nearest
            .iter()
            .map(|(&owner, &d)| {
                let reach = 2 * v.rule_experiments.min_city_gap(v.faction_of(owner)) + 2;
                if d > reach {
                    0
                } else {
                    self.decisions.found_now * (reach + 1 - d) / reach
                }
            })
            .sum();
        (i32::from(settler) * 50 + per_turn * window).min(100)
    }

    /// Gene `found_here`: found on the Settler's own tile now, remembering the order so a
    /// failed found (an unseen city within spacing) is not repeated next turn.
    fn found_here(&mut self, s: &UnitView) {
        settler_note(s.id, || "found here".into());
        let turn = self.view().turn;
        self.campaigns
            .found_tried
            .retain(|&(unit, _, t)| unit != s.id && t + 1 >= turn);
        self.campaigns.found_tried.push((s.id, s.pos, turn));
        self.settle_targets.push(s.pos);
        self.unit_order(s.id, Order::FoundCity { unit: s.id });
    }

    /// Founding a city is worth more than a one-off reward. If no site is available, however,
    /// a Settler can collect an uncontested ruin instead of waiting at home. The route is the
    /// same two-turn, danger-free one we require for a short founding trip.
    fn idle_settler_ruin(&self, s: &UnitView) -> Option<(Hex, Vec<Hex>)> {
        if self.decisions.ruin_collect == 0 {
            return None;
        }
        self.ctx
            .frontier
            .iter()
            .copied()
            .filter(|h| {
                self.view()
                    .tile(*h)
                    .is_some_and(|t| t.ruin && t.owner == Some(self.view().me))
            })
            .filter_map(|h| {
                let (turns, path) = self.nearby_settler_route(s, h)?;
                let value = self.decisions.ruin_bonus - 15 * turns - 5;
                (value > 0).then_some((value, h, path))
            })
            .max_by_key(|(value, h, _)| (*value, std::cmp::Reverse(*h)))
            .map(|(_, h, path)| (h, path))
    }

    pub(crate) fn settlement_timed_out(&self, s: &UnitView) -> bool {
        s.created_turn.is_some_and(|created| {
            self.view().turn.saturating_sub(created) >= self.decisions.settle_patience as u32
        })
    }

    /// A known, safe route arriving within two movement phases. Sum elapsed turns,
    /// not just path costs: a 1/2/1-cost route takes three turns at two moves/turn.
    fn nearby_settler_route(&self, s: &UnitView, to: Hex) -> Option<(i32, Vec<Hex>)> {
        if s.pos.distance(to) > 2 * s.max_moves.max(1) {
            return None;
        }
        // find_path reads only the map's geometry.
        let path = find_path(self.movement_map(), s.pos, to, |h| {
            let i = self.view().index(h)?;
            self.ctx.move_cost_light(h).filter(|_| {
                !self.enemy_war_tile(h)
                    && !self.ctx.settler_danger[i]
                    && (!self.settler_safety() || !self.settler_capturable_this_turn(h))
                    && !self.ctx.hostile_at[i]
                    && !self.ctx.neutral_at[i]
                    && !self
                        .view()
                        .units
                        .iter()
                        .any(|u| u.id != s.id && !u.military && u.pos == h)
            })
        })?;
        let mut turns = i32::from(!path.is_empty());
        let mut moves = if self.settler_safety() {
            self.turn_moves(s)
        } else {
            s.max_moves
        };
        for h in &path {
            let cost = self.ctx.move_cost_light(*h)?;
            if !can_step(moves, s.max_moves, cost) {
                turns += 1;
                moves = s.max_moves;
            }
            if turns > 2 || !can_step(moves, s.max_moves, cost) {
                return None;
            }
            moves = (moves - cost).max(0);
        }
        Some((turns, path))
    }

    /// Do not cut a commitment short for land that will be rejected again as soon
    /// as this route brings a remembered foreign border back into sight. Reuse the
    /// shared border inference, limited to known borders along this short route;
    /// distant remembered claims must not rule out a safe nearby fallback.
    fn route_reveals_settlement_risk(&self, site: Hex, path: &[Hex]) -> bool {
        use legatum_core::data::CITY_WORK_RADIUS as BORDER_RADIUS;
        let v = self.view();
        let gap = v.rule_experiments.min_city_gap(v.faction_of(self.ctx.me));
        path.iter().any(|step| {
            step.within(legatum_core::data::UNIT_SIGHT)
                .into_iter()
                .filter(|border| {
                    v.tile(*border).is_some_and(|tile| {
                        tile.explored && tile.owner.is_some_and(|owner| owner != self.ctx.me)
                    })
                })
                .any(|border| {
                    border.within(BORDER_RADIUS).into_iter().any(|center| {
                        center.distance(site) <= gap
                            && v.tile(center).is_some_and(|tile| {
                                !tile.visible
                                    && tile
                                        .terrain
                                        .is_none_or(|t| t.is_land() && t.move_cost().is_some())
                            })
                            && v.cities.iter().all(|c| c.pos.distance(center) > gap)
                    })
                })
        })
    }

    /// Only issue a destination-only order when its ordinary route is safe. Confirm
    /// detours with stop-on-block so core retains the goal without unsafe rerouting.
    pub(crate) fn move_settler(&mut self, s: &UnitView, to: Hex) {
        let map = self.movement_map();
        let otherwise_safe = |h| {
            self.view().index(h).is_some_and(|i| {
                !self.enemy_war_tile(h)
                    && !self.ctx.settler_danger[i]
                    && !self.ctx.hostile_at[i]
                    && !self.ctx.neutral_at[i]
                    && self.ctx.move_cost_light(h).is_some()
                    && !self
                        .view()
                        .units
                        .iter()
                        .any(|u| u.id != s.id && !u.military && u.pos == h)
            })
        };
        let safe = |h| {
            otherwise_safe(h) && (!self.settler_safety() || !self.settler_capturable_this_turn(h))
        };
        // The movement map reads an unexplored tile as Grassland, but core refuses a route over
        // land the empire has not explored, so a Settler's route stays on known tiles.
        let ordinary = find_path(map, s.pos, to, |h| {
            map.move_cost(h).filter(|_| {
                self.view().index(h).is_some_and(|i| {
                    !self.enemy_war_tile(h)
                        && !self.ctx.hostile_at[i]
                        && self.view().tiles[i].terrain.is_some()
                })
            })
        });
        // A distant hazard is rechecked next turn. Rejecting a whole route because a
        // Scout could occupy a tile many turns away strands Settlers at home even while
        // the first movement phase is safe. Confirm that phase's steps now, then replan.
        let safe_this_turn = |path: &[Hex]| {
            let budget = self.turn_moves(s);
            let mut remaining = budget;
            let mut arrival = s.pos;
            for h in path {
                let Some(cost) = self.ctx.move_cost(*h) else {
                    return false;
                };
                if !can_step(remaining, budget, cost) {
                    break;
                }
                arrival = *h;
                remaining = (remaining - cost).max(0);
            }
            // A home guard can walk beside a Settler through a peaceful Scout's
            // interception radius. Without this, a Scout parked by the capital
            // traps both units there indefinitely even though they could leave
            // together. Wartime danger still requires a route around the army.
            let home_guard = self.ctx.own_units.iter().any(|u| {
                u.military
                    && u.kind != UnitKind::Scout
                    && u.pos == s.pos
                    && u.hp >= self.decisions.recover_hp
                    && self.reaches_this_turn(u, arrival)
                    && self.free_tile_for(u, arrival)
            });
            let mut remaining = budget;
            for h in path {
                let Some(cost) = self.ctx.move_cost(*h) else {
                    return false;
                };
                if !can_step(remaining, budget, cost) {
                    break;
                }
                if !safe(*h) && !(home_guard && otherwise_safe(*h)) {
                    return false;
                }
                remaining = (remaining - cost).max(0);
            }
            true
        };
        // Keep the older Normal policy's wider caution; Hard evaluates the actual
        // route and this turn's arrival instead of every tile around the origin.
        let danger_nearby = !self.settler_safety()
            && s.pos.within(s.max_moves).iter().any(|h| {
                self.view()
                    .index(*h)
                    .is_some_and(|i| self.ctx.settler_danger[i])
            });
        if !danger_nearby
            && let Some(path) = ordinary.filter(|path| {
                if self.settler_safety() {
                    safe_this_turn(path)
                } else {
                    path.iter().all(|h| safe(*h))
                }
            })
        {
            if self.settler_safety() && path.iter().any(|h| !safe(*h)) {
                self.unit_order(
                    s.id,
                    Order::MoveUnitPath {
                        unit: s.id,
                        to,
                        path,
                        stop_on_block: true,
                    },
                );
                return;
            }
            if self.at_war() && self.wartime_border_near_route(&path) {
                self.unit_order(
                    s.id,
                    Order::MoveUnitPath {
                        unit: s.id,
                        to,
                        path,
                        stop_on_block: true,
                    },
                );
                return;
            }
            // A no-route failure clears goto without spending movement. The route can
            // still look open in remembered territory, so repeating a destination-only
            // order may never take even its first safe step. Confirm the known route:
            // the engine checks each step as it arrives, letting us reconnoitre the
            // obstruction. Keep the settlement goal as `to`, not a temporary waypoint.
            // All evidence is in PlayerView, so recovery also works with a fresh brain.
            let stationary = s.goto.is_none() && s.moves_left >= s.max_moves;
            let remembered_route = path
                .iter()
                .any(|h| self.view().tile(*h).is_some_and(|tile| !tile.visible));
            if s.max_moves > 0 && (stationary || !s.path.is_empty()) && remembered_route {
                self.unit_order(
                    s.id,
                    Order::MoveUnitPath {
                        unit: s.id,
                        to,
                        path,
                        stop_on_block: false,
                    },
                );
            } else {
                self.move_or_hold(s, to);
            }
        } else if let Some(path) = find_path(map, s.pos, to, |h| {
            self.ctx
                .move_cost_light(h)
                .filter(|_| safe(h) && self.view().tile(h).is_some_and(|t| t.terrain.is_some()))
        })
        .filter(|path| !path.is_empty())
        {
            // Adjacent waypoints clear goto on arrival, losing choose_site's commitment
            // bonus. Competing sites can then send a settler around the same circuit
            // forever. Keep the destination while committing every safe detour step,
            // and require a new planning decision if that constrained route is blocked.
            self.unit_order(
                s.id,
                Order::MoveUnitPath {
                    unit: s.id,
                    to,
                    path,
                    stop_on_block: true,
                },
            );
        } else if (self.hard_defense() || self.settler_safety())
            && self.snipe_exposed(s.pos)
            && let Some(home) = self
                .ctx
                .nearest_own_city(s.pos)
                .map(|c| c.pos)
                .filter(|h| *h != s.pos && self.ctx.reachable(s.pos, *h))
        {
            // Hard avoids every tile a rival could reach, so a Settler caught inside that
            // reach finds no safe route and would hold there, beside the army that can take
            // it, turn after turn. Standing still is the worst choice: head home.
            self.move_or_hold(s, home);
        } else {
            self.unit_order(s.id, Order::SkipUnit { unit: s.id });
        }
    }

    /// Retreat from an immediate threat while still keeping the route out of known enemy land.
    /// Unlike an expansion trip, an escape may cross the ordinary settler danger map.
    pub(crate) fn retreat_settler(&mut self, s: &UnitView, to: Hex) {
        let map = self.movement_map();
        let enterable = |h| {
            map.move_cost(h).filter(|_| {
                self.view().index(h).is_some_and(|i| {
                    !self.enemy_war_tile(h)
                        && !self.ctx.hostile_at[i]
                        && !self.ctx.neutral_at[i]
                        && self.view().tiles[i].terrain.is_some()
                })
            })
        };
        let route = find_path(map, s.pos, to, enterable);
        let engine_route = find_path(map, s.pos, to, |h| {
            map.move_cost(h).filter(|_| {
                self.view()
                    .index(h)
                    .is_some_and(|i| !self.ctx.hostile_at[i] && !self.ctx.neutral_at[i])
            })
        });
        if engine_route
            .as_ref()
            .is_some_and(|path| path.iter().all(|h| !self.enemy_war_tile(*h)))
        {
            if self.at_war()
                && engine_route
                    .as_ref()
                    .is_some_and(|path| self.wartime_border_near_route(path))
            {
                if let Some(path) = engine_route.filter(|path| !path.is_empty()) {
                    self.unit_order(
                        s.id,
                        Order::MoveUnitPath {
                            unit: s.id,
                            to,
                            path,
                            stop_on_block: true,
                        },
                    );
                } else {
                    self.unit_order(s.id, Order::SkipUnit { unit: s.id });
                }
            } else {
                self.move_or_hold(s, to);
            }
        } else if let Some(path) = route.filter(|path| !path.is_empty()) {
            self.unit_order(
                s.id,
                Order::MoveUnitPath {
                    unit: s.id,
                    to,
                    path,
                    stop_on_block: true,
                },
            );
        } else {
            self.unit_order(s.id, Order::SkipUnit { unit: s.id });
        }
    }

    /// A stationary settler may have tried founding beside a city just outside its sight
    /// radius (city spacing is three, sight is two). Check that border from one step closer
    /// instead of repeating the same failed order forever. This uses only observed borders
    /// and fog, never the actual hidden city or the engine's private rejection reason.
    fn rescout_founding_site(&mut self, s: &UnitView) -> bool {
        if self.ctx.own_cities.is_empty() || s.goto.is_some() || s.moves_left < s.max_moves {
            return false;
        }
        let borders: Vec<_> = s
            .pos
            .within(2)
            .into_iter()
            .filter(|h| {
                self.view()
                    .tile(*h)
                    .is_some_and(|t| t.owner.is_some_and(|o| o != self.ctx.me))
            })
            .collect();
        let gap = self
            .view()
            .rule_experiments
            .min_city_gap(self.view().faction_of(self.ctx.me));
        let unseen: Vec<_> = s
            .pos
            .ring(gap)
            .into_iter()
            .filter(|h| {
                // The target need not be explored or passable: only the step toward it
                // below must be known land. Unexplored tiles can hide the blocking city.
                self.view().tile(*h).is_some_and(|t| !t.visible)
                    && borders.iter().any(|b| b.distance(*h) == 1)
            })
            .collect();
        let step = s
            .pos
            .neighbors()
            .into_iter()
            .filter(|h| {
                self.ctx.move_cost_light(*h).is_some()
                    && !self.enemy_war_tile(*h)
                    && !self.ctx.settler_danger[self.view().index(*h).expect("known tile")]
                    && !self.ctx.hostile_at[self.view().index(*h).expect("known tile")]
            })
            .filter_map(|h| {
                let reveals = unseen
                    .iter()
                    .filter(|u| u.distance(h) <= legatum_core::data::UNIT_SIGHT)
                    .count();
                (reveals > 0).then_some((std::cmp::Reverse(reveals), h))
            })
            .min()
            .map(|(_, h)| h);
        if let Some(to) = step {
            self.move_or_hold(s, to);
            true
        } else {
            false
        }
    }

    /// The best reachable site for this settler, trading site quality against travel time.
    #[cfg(test)]
    pub fn choose_site(&self, s: &UnitView) -> Option<Hex> {
        self.choose_site_scored(s).map(|(_, h)| h)
    }

    /// `choose_site` with the chosen site's value.
    fn choose_site_scored(&self, s: &UnitView) -> Option<(i32, Hex)> {
        let ctx = &self.ctx;
        let gap = self
            .view()
            .rule_experiments
            .min_city_gap(self.view().faction_of(ctx.me));
        // Shared, fog-safe suitability; Hard also values initial productivity. The AI additionally
        // requires a route around remembered hazards, which only the planner tracks.
        let at_war = self.at_war();
        // Gene `roman_soldier_route`: a Roman veteran is a soldier, not a civilian to be
        // captured on the road, so the Settler's danger detours do not apply to it.
        let spearman = self.roman_soldier_founder(s);
        let field = if spearman {
            ctx.distance_field(s.pos, true)
        } else if at_war {
            let mut blocked = ctx.settler_danger.clone();
            for (i, blocked) in blocked.iter_mut().enumerate() {
                *blocked |= self.enemy_war_tile(ctx.hex_at(i));
            }
            ctx.distance_field_avoiding(s.pos, true, Some(&blocked))
        } else {
            ctx.settler_distance_field(s.pos)
        };
        let first_city = ctx.own_cities.is_empty();
        let mut best: Option<(i32, Hex)> = None;
        let mut distant_capital: Option<(i32, Hex)> = None;
        let mut fallback: Option<(i32, Hex)> = None;
        let remembered_risk = self.remembered_border_risks();
        let timed_out = self.settlement_timed_out(s);
        let mut timeout_best = None;
        let mut risky: Option<(i32, i32, Hex)> = None;
        // Gene `fallback_risk`: the preferred site dropped for trip risk with the least of it.
        let mut dropped_for_risk: Option<(i32, i32, Hex)> = None;
        // Diagnostics only: a second tile whose verdict is recorded ("probe tile").
        let probe = settler_probe(s.id);
        let mut probe_seen = false;
        // Gene `border_explained`: border-risk sites come back flagged, and only a border no
        // known city explains still rules one out.
        let explained = self.decisions.border_explained > 0;
        let unexplained_risk: &[bool] = if explained {
            self.unexplained_border_risks
                .get_or_init(|| unexplained_border_risks(self.view()))
        } else {
            &[]
        };
        let candidates = if explained {
            settlement_candidates_with_risky(self.view(), self.ctx.site_score.basics(), s)
        } else {
            settlement_candidates_with(
                self.view(),
                self.ctx.site_score.basics(),
                s,
                &self.settle_targets,
            )
        };
        for mut site in candidates {
            let site_option = format!("unit={:?} site={:?}", s.id, site.pos);
            if site.border_risk
                && self
                    .view()
                    .index(site.pos)
                    .is_none_or(|i| unexplained_risk[i])
            {
                if self.decision_trace.is_some() {
                    let mut row =
                        crate::DecisionOption::new("settler_site", site_option, "gated_out");
                    row.gate = Some("unexplained border risk".into());
                    self.note_option(row);
                }
                continue;
            }
            let Some(turns) = site.travel_turns else {
                if self.decision_trace.is_some() {
                    let mut row =
                        crate::DecisionOption::new("settler_site", site_option, "gated_out");
                    row.gate = Some("no route".into());
                    self.note_option(row);
                }
                if probe == Some(site.pos) {
                    probe_seen = true;
                    settler_note(s.id, || "probe tile not a candidate: no route".into());
                }
                continue;
            };
            // Iron and Coal this site would claim are worth the Forges and Factories their
            // supply would put to work, at the site score's 3 points per Production.
            site.score +=
                3 * crate::supply_value::site_gain(
                    self.view(),
                    site.pos,
                    self.decisions.settlement_claim_timeline,
                ) * self.decisions.supply_value_percent
                    / 100;
            let h = site.pos;
            site.score = self.peace_army_site_score(h, site.score);
            let i = self.view().index(h).expect("known site");
            let here = h == s.pos;
            let label = if here {
                Some("own tile")
            } else if probe == Some(h) {
                Some("probe tile")
            } else {
                None
            };
            if probe == Some(h) {
                probe_seen = true;
                settler_note(s.id, || {
                    format!(
                        "probe tile candidate: score {} quality {:?} travel {turns} nearest city {:?}",
                        site.score, ctx.site_score[i], site.nearest_city
                    )
                });
            }
            let blocked = if field[i] == UNREACHABLE {
                Some("unreachable")
            } else if self.enemy_war_tile(h) {
                Some("enemy war tile")
            } else if self.enemy_army_near(h) {
                Some("enemy army near")
            } else if remembered_risk[i] {
                Some("remembered border risk")
            } else if ctx.in_danger(h) {
                Some("in danger")
            } else if ctx.settler_danger[i] && !spearman {
                Some("settler danger")
            } else if site.threats > 0 {
                Some("site threats")
            } else if self.settle_targets.iter().any(|t| t.distance(h) <= gap) {
                Some("another settler's target")
            } else if self.view().units.iter().any(|u| {
                u.owner != ctx.me
                    && u.kind == UnitKind::Settler
                    && u.pos.distance(h)
                        <= self
                            .view()
                            .rule_experiments
                            .min_city_gap(self.view().faction_of(u.owner))
            }) {
                Some("foreign settler near")
            } else if !first_city && turns + 8 >= self.turns_remaining() {
                Some("too late in the game")
            } else {
                None
            };
            if let Some(why) = blocked {
                if self.decision_trace.is_some() {
                    let mut row =
                        crate::DecisionOption::new("settler_site", site_option, "gated_out");
                    row.gate = Some(why.into());
                    self.note_option(row);
                }
                if let Some(label) = label {
                    settler_note(s.id, || format!("{label} rejected: {why}"));
                }
                continue;
            }
            let founding_damage = if self.decisions.founding_survival_weight > 0 && at_war {
                self.founding_damage_before_defense(h, turns)
            } else {
                0
            };
            if timed_out
                && let Some((arrival, path)) = self.nearby_settler_route(s, h)
                && !self.route_reveals_settlement_risk(h, &path)
            {
                // A nearby safe site overrides distant-site hysteresis and faction
                // detours. Still evaluate normal travel below: a crowded homeland
                // may have no legal site within two turns, and returning home then
                // would trap the settler forever.
                let value = if self.decisions.founding_survival_weight == 0 {
                    // Preserve the original integer evaluation exactly when the gene is off.
                    site.score * self.settlement_productivity(h) / (100 * (arrival + 1))
                } else {
                    let raw = site.score * self.settlement_productivity(h) / 100;
                    (raw - self.founding_survival_cost(raw, founding_damage)) / (arrival + 1)
                };
                if timeout_best.is_none_or(|(b, _)| value > b) {
                    timeout_best = Some((value, h));
                }
            }
            if s.goto == Some(h) && Self::stuck(s) {
                if self.decision_trace.is_some() {
                    let mut row =
                        crate::DecisionOption::new("settler_site", site_option, "gated_out");
                    row.gate = Some("stuck on current route".into());
                    self.note_option(row);
                }
                if let Some(label) = label {
                    settler_note(s.id, || format!("{label} rejected: stuck"));
                }
                continue;
            }
            // A trained settler has already spent a citizen and a city's production. When
            // no preferred site remains (a rival took it, or war closed the approach), a
            // modest safe site nearby still adds a city, territory and a milestone step;
            // holding the settler at home forever also blocks the next one.
            // A border-risk site the gene `border_explained` lets in has no `site_score`.
            let quality = if site.border_risk {
                ctx.site_score.basics().legal_base_score(self.view(), i)
            } else {
                ctx.site_score[i]
            };
            // Rome cannot train Settlers. A spare veteran or a captured Settler
            // is already a paid-for city if it can reach safe land. The ordinary
            // quality and capital-distance floors can strand either of them.
            let roman_founder = self.faction == Faction::Rome
                && self.hard_defense()
                && self.decisions.roman_founders > 0;
            let preferred = first_city
                || (roman_founder && site.score > 0 && turns <= 10)
                || (quality.is_some_and(|score| score >= self.decisions.settler_min_site_score)
                    && site
                        .nearest_city
                        .is_none_or(|home| home <= self.decisions.settler_max_site_distance));
            let risk = if self.settler_safety() && !first_city && !spearman {
                self.settler_trip_risk(s.pos, h)
            } else {
                0
            };
            if self.settler_safety() && risk >= self.decisions.settler_risk_avoid {
                if self.decision_trace.is_some() {
                    let mut row =
                        crate::DecisionOption::new("settler_site", site_option, "gated_out");
                    row.gate = Some(format!(
                        "trip risk {risk} >= {}",
                        self.decisions.settler_risk_avoid
                    ));
                    self.note_option(row);
                }
                if let Some(label) = label {
                    settler_note(s.id, || format!("{label} rejected: trip risk {risk}"));
                }
                if !at_war && risky.is_none_or(|(r, b, _)| (risk, -site.score) < (r, -b)) {
                    risky = Some((risk, site.score, h));
                }
                continue;
            }
            if !preferred {
                if self.decision_trace.is_some() {
                    let mut row =
                        crate::DecisionOption::new("settler_site", site_option, "gated_out");
                    row.gate = Some(format!(
                        "site score below {} or nearest city beyond {} (fallback considered separately)",
                        self.decisions.settler_min_site_score,
                        self.decisions.settler_max_site_distance
                    ));
                    self.note_option(row);
                }
                let safe_score = site.score
                    - Self::settler_loss_cost(site.score, risk)
                    - self.founding_survival_cost(site.score, founding_damage);
                if let Some(label) = label {
                    settler_note(s.id, || {
                        format!(
                            "{label} not preferred (score {} min {}, nearest city {:?} max {}): fallback {safe_score}",
                            quality.unwrap_or(0),
                            self.decisions.settler_min_site_score,
                            site.nearest_city,
                            self.decisions.settler_max_site_distance
                        )
                    });
                }
                if safe_score > 0 && turns <= 8 && fallback.is_none_or(|(b, _)| safe_score > b) {
                    fallback = Some((safe_score, h));
                }
                continue;
            }
            let mut value = if first_city && self.rushing() {
                self.rush_capital_production(h, turns) + site.score / 8
            } else {
                site.score * self.settlement_productivity(h) / 100
                    - turns * self.decisions.settler_travel_penalty
            };
            if roman_founder && s.kind != UnitKind::Settler {
                // Each travel turn keeps a proven founder in the field, unable to
                // replace itself with a city's production. Prefer a nearby legal
                // site over a distant marginally richer one.
                value -= turns * 30;
            }
            value -= self.founding_survival_cost(value, founding_damage);
            value += self.land_contest_value(h, value);
            value += self.site_strategy_value(h, value, s);
            if self.settler_safety() {
                // A captured Settler denies us a city and gives the captor one. Exposure is a
                // bounded loss-risk heuristic, so price both sides of that swing.
                value -= Self::settler_loss_cost(value, risk);
            } else if self.hard_defense() && !first_city && !spearman {
                // Retain the established two-seat policy used by the paired duel.
                let legacy_risk = self.settler_trip_risk(s.pos, h);
                if legacy_risk >= self.decisions.settler_risk_avoid {
                    if let Some(label) = label {
                        settler_note(s.id, || {
                            format!("{label} rejected: legacy risk {legacy_risk}")
                        });
                    }
                    if !at_war && risky.is_none_or(|(r, b, _)| (legacy_risk, -value) < (r, -b)) {
                        risky = Some((legacy_risk, value, h));
                    }
                    if dropped_for_risk.is_none_or(|(r, b, _)| (legacy_risk, -value) < (r, -b)) {
                        dropped_for_risk = Some((legacy_risk, value, h));
                    }
                    continue;
                }
                value -= value.max(0) * legacy_risk / 250;
            }
            // Commit to the site already on the way to. Fog reveals and hides the same
            // hazard as the settler walks, and the travel discount shifts with every step;
            // without hysteresis two sites can trade places each turn and the settler
            // walks back and forth for the rest of the game. A site that turns unsafe or
            // illegal is still dropped by the filters above.
            if s.goto == Some(h) {
                value = value * self.decisions.settler_commit_percent / 100
                    + self.decisions.settler_commit_bonus;
            }
            // Rush prefers to found within two turns, but a cityless settler
            // must keep traveling when every safe legal site is farther away.
            let candidate = if first_city && self.rushing() && turns > 2 {
                &mut distant_capital
            } else {
                &mut best
            };
            if let Some(label) = label {
                settler_note(s.id, || {
                    format!("{label} value {value} (score {})", site.score)
                });
            }
            if candidate.is_none_or(|(b, _)| value > b) {
                *candidate = Some((value, h));
            }
            if self.decision_trace.is_some() {
                let mut row =
                    crate::DecisionOption::new("settler_site", site_option, "considered_rejected");
                row.score = Some(value);
                self.note_option(row);
            }
        }
        if let Some(p) = probe.filter(|_| !probe_seen) {
            settler_note(s.id, || {
                let why = legatum_core::settlement::site_blocker(self.view(), p).map_or_else(
                    || "foreign border could hide a city".to_string(),
                    |b| format!("{b:?}"),
                );
                format!(
                    "probe tile not a candidate: {why}{}",
                    self.border_risk_note(p)
                )
            });
        }
        settler_note(s.id, || {
            format!(
                "timeout {timeout_best:?} best {best:?} distant {distant_capital:?} fallback {fallback:?} risky {risky:?} at_war {at_war}"
            )
        });
        // Keeping a trained Settler for a later, safer opening is the zero-value outside
        // option. Only the new gene compares founding against that option; the established
        // planner still takes its best site, even if its score is negative.
        let worthwhile = |candidate: Option<(i32, Hex)>| {
            if self.decisions.founding_survival_weight > 0 {
                candidate.filter(|(value, _)| *value > 0)
            } else {
                candidate
            }
        };
        let preferred_choice = worthwhile(timeout_best)
            .or(worthwhile(best))
            .or(worthwhile(distant_capital));
        // Gene `fallback_risk`: the far fallback is checked for the trip risk the preferred
        // sites were dropped for; a fallback no safer than the least risky of them loses to it.
        let fallback = match (preferred_choice, worthwhile(fallback), dropped_for_risk) {
            (None, Some((_, site)), Some((dropped_risk, value, near)))
                if self.decisions.fallback_risk > 0 && self.hard_defense() && !first_city =>
            {
                let fallback_risk = self.settler_trip_risk(s.pos, site);
                settler_note(s.id, || {
                    format!(
                        "fallback {site:?} trip risk {fallback_risk} against {near:?} dropped at {dropped_risk}"
                    )
                });
                if fallback_risk >= dropped_risk {
                    Some((value - value.max(0) * dropped_risk.min(250) / 250, near))
                } else {
                    fallback
                }
            }
            _ => fallback,
        };
        // The ordinary quality floor favors a preferred site even over a higher-valued
        // fallback. A Peace army can erase that quality edge; compare the priced
        // alternatives when the preferred site's Peace exposure is real.
        let chosen =
            if preferred_choice.is_some_and(|(_, site)| self.peace_army_site_risk(site) > 0) {
                match (preferred_choice, worthwhile(fallback)) {
                    (Some(preferred), Some(fallback))
                        if fallback.0 > preferred.0
                            && self.peace_army_site_risk(fallback.1)
                                < self.peace_army_site_risk(preferred.1) =>
                    {
                        Some(fallback)
                    }
                    (preferred, fallback) => preferred.or(fallback),
                }
            } else {
                preferred_choice.or(worthwhile(fallback))
            };
        let chosen = chosen.or(if at_war {
            None
        } else {
            worthwhile(risky.map(|(_, value, site)| (value, site)))
        });
        if let Some(trace) = &self.decision_trace {
            let mut trace = trace.borrow_mut();
            let prefix = format!("unit={:?} site=", s.id);
            let mut scores: Vec<i32> = trace
                .iter()
                .filter(|r| r.domain == "settler_site" && r.option.starts_with(&prefix))
                .filter_map(|r| r.score)
                .collect();
            scores.sort_unstable_by(|a, b| b.cmp(a));
            for row in trace
                .iter_mut()
                .filter(|r| r.domain == "settler_site" && r.option.starts_with(&prefix))
            {
                row.winner_score = scores.first().copied();
                row.runner_up_score = scores.get(1).copied();
                if chosen.is_some_and(|(_, h)| row.option == format!("unit={:?} site={h:?}", s.id))
                {
                    row.outcome = "selected";
                    row.gate = None;
                }
            }
        }
        chosen
    }

    /// Gene `land_contest`: what founding on `site` denies a neighbour. The nearest known
    /// city of another team's empire (barbarians aside) would send its next Settlers within
    /// about twice its city gap plus 2; a site inside that reach is land the neighbour takes
    /// if we do not, worth the gene's percent of the site's `value`, more the closer it lies
    /// to that city. Only cities in our view count, so nothing hidden is read.
    pub(crate) fn land_contest_value(&self, site: Hex, value: i32) -> i32 {
        let gene = self.decisions.land_contest;
        if gene == 0 || !self.hard_defense() || value <= 0 || self.ctx.own_cities.is_empty() {
            return 0;
        }
        let v = self.view();
        let Some((d, owner)) = v
            .cities
            .iter()
            .filter(|c| !c.owner.is_barbarian() && !v.same_team(v.me, c.owner))
            .map(|c| (c.pos.distance(site), c.owner))
            .min()
        else {
            return 0;
        };
        let reach = 2 * v.rule_experiments.min_city_gap(v.faction_of(owner)) + 2;
        if d > reach {
            return 0;
        }
        value * gene / 100 * (reach + 1 - d) / reach
    }

    /// Diagnostics: the evidence behind a border-risk verdict on `site` (the unseen tiles that
    /// could hold a city within spacing, and the visible foreign tiles that imply them).
    fn border_risk_note(&self, site: Hex) -> String {
        use legatum_core::data::CITY_WORK_RADIUS;
        let v = self.view();
        let gap = v.rule_experiments.min_city_gap(v.faction_of(v.me));
        let mut centers = 0;
        let mut explored_centers = 0;
        let mut explained = 0;
        let mut unexplained = 0;
        let mut nearest: Option<i32> = None;
        for c in site.within(gap) {
            let Some(t) = v.tile(c) else { continue };
            if t.visible
                || !t
                    .terrain
                    .is_none_or(|t| t.is_land() && t.move_cost().is_some())
                || v.cities.iter().any(|k| k.pos.distance(c) <= gap)
            {
                continue;
            }
            let borders: Vec<(Hex, legatum_core::PlayerId)> = c
                .within(CITY_WORK_RADIUS)
                .into_iter()
                .filter_map(|b| {
                    v.tile(b)
                        .filter(|bt| bt.visible)
                        .and_then(|bt| bt.owner)
                        .filter(|&o| o != v.me)
                        .map(|o| (b, o))
                })
                .collect();
            if borders.is_empty() {
                continue;
            }
            centers += 1;
            explored_centers += i32::from(t.explored);
            for (b, o) in borders {
                let d = v
                    .cities
                    .iter()
                    .filter(|k| k.owner == o)
                    .map(|k| k.pos.distance(b))
                    .min();
                if let Some(d) = d {
                    nearest = Some(nearest.map_or(d, |n: i32| n.max(d)));
                }
                if d.is_some_and(|d| d <= CITY_WORK_RADIUS) {
                    explained += 1;
                } else {
                    unexplained += 1;
                }
            }
        }
        format!(
            " (centers {centers}, explored {explored_centers}; border tiles explained by a known city within 3: {explained}, not: {unexplained}; farthest known-city distance {nearest:?})"
        )
    }

    fn settler_loss_cost(city_value: i32, exposure: i32) -> i32 {
        city_value
            .max(0)
            .saturating_mul(2)
            .saturating_mul(exposure.clamp(0, 100))
            / 100
    }

    /// The fraction of a fresh, unwalled city's HP a visible wartime shooter could remove
    /// before this site can build Walls. Movement uses known passable terrain and current
    /// border permissions; unknown LOS is opaque. A future shooter is charged only when it
    /// can actually reach a firing tile within three turns of founding (five at most).
    /// The site can still win if its net value beats keeping the Settler for later.
    fn founding_damage_before_defense(&self, site: Hex, found_in: i32) -> i32 {
        use legatum_core::{BuildingKind, combat, data, los, path};
        let v = self.view();
        let faction = v.faction_of(v.me);
        let center_prod = v.tile(site).and_then(|t| t.terrain).map_or(0, |terrain| {
            data::terrain_yields(faction, terrain).production
        });
        let mut workers: Vec<i32> = site
            .neighbors()
            .into_iter()
            .filter_map(|h| v.tile(h))
            .filter(|tile| tile.explored && tile.owner.is_none_or(|owner| owner == v.me))
            .filter_map(|tile| tile.terrain)
            .map(|terrain| data::terrain_yields(faction, terrain).production)
            .collect();
        workers.sort_unstable_by(|a, b| b.cmp(a));
        // Optimistic early production makes Walls arrive sooner, so this does not
        // exaggerate exposure on marginal terrain.
        let production = (data::CITY_CENTER_YIELDS.production
            + center_prod
            + workers.into_iter().take(2).sum::<i32>())
        .max(data::CITY_MIN_PRODUCTION);
        let walls_cost = v.rule_experiments.base_production_cost(
            faction,
            legatum_core::ProductionItem::Building {
                building: BuildingKind::Walls,
            },
        );
        let walls_turns = (walls_cost + production - 1) / production;
        let defense = combat::city_defense_strength_for_rules(
            v.rule_experiments.faction_rules_off,
            faction,
            1,
            false,
            data::CITY_HP,
            v.era_of(v.me),
        )
        .total;
        let movement_horizon = (found_in + 3).clamp(3, 5);
        let mut damage = 0;
        for enemy in self.ctx.threat_units.iter().filter(|enemy| {
            enemy.military
                && enemy.ranged > 0
                && enemy.range > 0
                && v.at_war(v.me, enemy.owner)
                && enemy.pos.distance(site)
                    <= movement_horizon * enemy.max_moves.max(1) + enemy.range
        }) {
            let entry = v.entry_table(enemy.owner, enemy.kind);
            let shot = site
                .within(enemy.range)
                .into_iter()
                .filter(|&fire| fire != site)
                .filter(|&fire| {
                    v.tile(fire).is_some_and(|tile| {
                        tile.explored
                            && tile.move_cost().is_some()
                            && entry.can_enter(v, tile.owner)
                    })
                })
                .filter(|&fire| {
                    los::ranged_clear_with_tech(
                        enemy.kind,
                        matches!(
                            enemy.kind,
                            UnitKind::Artillery | UnitKind::RocketArtillery | UnitKind::CombatDrone
                        ),
                        v.cities
                            .iter()
                            .any(|c| c.owner == enemy.owner && c.pos == fire && c.walls),
                        fire,
                        site,
                        |h| v.tile(h).and_then(|tile| tile.terrain),
                    )
                })
                .filter_map(|fire| {
                    if fire == enemy.pos {
                        return Some(0);
                    }
                    let route = v.movement_path_with(&entry, enemy.pos, fire)?;
                    if route.iter().any(|&h| {
                        h == site
                            || !v
                                .tile(h)
                                .is_some_and(|t| t.explored && t.move_cost().is_some())
                            || v.units.iter().any(|u| u.pos == h)
                    }) {
                        return None;
                    }
                    let turns = path::path_turns(
                        &route,
                        |h| v.tile(h).and_then(|t| t.move_cost()).unwrap_or(i32::MAX),
                        (enemy.pos, enemy.max_moves, enemy.max_moves),
                        |h| v.tile(h).is_some_and(|t| t.road > 0),
                        |_| false,
                    );
                    (turns <= movement_horizon).then_some(turns + 1)
                })
                .min();
            let Some(shot) = shot else { continue };
            let shots_before_walls = (walls_turns - (shot - found_in).max(0) + 1).clamp(0, 2);
            if shots_before_walls == 0 {
                continue;
            }
            let attack = combat::unit_ranged_strength_for_rules(
                v.rule_experiments.faction_rules_off,
                v.faction_of(enemy.owner),
                v.made_by(enemy),
                enemy.kind,
                enemy.hp,
                enemy.level,
            )
            .total
                - enemy.unsupplied_loss.unwrap_or(0)
                + data::siege_city_bonus(enemy.kind);
            damage += combat::base_damage(attack, defense) * shots_before_walls;
        }
        (damage * 100 / data::CITY_HP).clamp(0, 100)
    }

    fn founding_survival_cost(&self, value: i32, damage_percent: i32) -> i32 {
        let scaled = i64::from(value.max(0))
            * i64::from(self.decisions.founding_survival_weight)
            * i64::from(damage_percent)
            / 10_000;
        scaled.min(i64::from(i32::MAX)) as i32
    }

    /// The shared scorer infers an unseen foreign city only from borders visible right now.
    /// A settler walking away from such a border forgets it, the site looks legal again and
    /// it turns back; near the site it sees the border and turns away. Apply the same
    /// inference to remembered foreign borders that no known city explains.
    pub(crate) fn remembered_border_risks(&self) -> Vec<bool> {
        use legatum_core::data::CITY_WORK_RADIUS as BORDER_RADIUS;
        let v = self.view();
        let mut risks = vec![false; v.tiles.len()];
        for (i, tile) in v.tiles.iter().enumerate() {
            if tile.visible || !tile.explored || !tile.owner.is_some_and(|o| o != self.ctx.me) {
                continue;
            }
            let pos = self.ctx.hex_at(i);
            if v.cities
                .iter()
                .any(|c| c.pos.distance(pos) <= BORDER_RADIUS)
            {
                continue;
            }
            for center in pos.within(BORDER_RADIUS) {
                let Some(j) = v.index(center) else { continue };
                let t = &v.tiles[j];
                // Cities are remembered once seen: an explored tile without one is no
                // center. Only never-explored land can hide the city behind a border.
                if t.explored
                    || !t
                        .terrain
                        .is_none_or(|t| t.is_land() && t.move_cost().is_some())
                    || v.cities.iter().any(|c| {
                        c.pos.distance(center)
                            <= v.rule_experiments.min_city_gap(v.faction_of(v.me))
                    })
                {
                    continue;
                }
                for site in center.within(v.rule_experiments.min_city_gap(v.faction_of(v.me))) {
                    if let Some(k) = v.index(site) {
                        risks[k] = true;
                    }
                }
            }
        }
        risks
    }

    pub fn plan_scouts(&mut self) {
        let scouts: Vec<&UnitView> = self
            .ctx
            .own_units
            .iter()
            .copied()
            .filter(|u| u.kind == UnitKind::Scout)
            .collect();
        let mut targets: Vec<Hex> = Vec::new();
        // A known leader fighting on another front is a short opportunity. Send one
        // Scout to open a legal city route before ordinary sentry posts occupy both.
        for s in &scouts {
            if self.has_acted(s.id) || self.unit_exposure_key(s, s.pos).0 > 0 {
                continue;
            }
            if let Some(target) = self.busy_leader_scout_target(s)
                && let Some(to) = self.campaign_approach(s, target)
                && to != s.pos
            {
                targets.push(target);
                self.move_or_hold(s, to);
                break;
            }
        }
        // An empire that knows nowhere to settle needs its Scouts' eyes on the land, not on
        // a neighbour: sentry duty from the first turns kept a one-city empire from ever
        // seeing a site (gene `sentry_needs_sites`).
        if !self.scouts_needed_for_sites() {
            self.watch_approaches(&scouts);
            self.post_sentry(&scouts);
        }
        for s in scouts {
            if self.has_acted(s.id) {
                continue;
            }
            if self.unit_exposure_key(s, s.pos).0 > 0 {
                if !self.retreat(s) {
                    self.unit_order(s.id, Order::SkipUnit { unit: s.id });
                }
                continue;
            }
            match self
                .busy_leader_scout_target(s)
                .or_else(|| self.primitive_scout_target(s))
                .or_else(|| self.choose_frontier(s, &targets))
            {
                Some(t) => {
                    targets.push(t);
                    // MoveUnit carries a destination, not our safe route. Reuse army
                    // approach waypoints so resolution cannot shortcut through a sighting.
                    let to = self.campaign_approach(s, t).unwrap_or(s.pos);
                    self.move_or_hold(s, to);
                }
                // A temporarily unsafe frontier is still work to revisit after clearance or
                // expiry. Scouts cost no upkeep, so one with nothing to explore just waits;
                // disbanding it would save nothing and give away its sight.
                None => {
                    self.unit_order(s.id, Order::SkipUnit { unit: s.id });
                }
            }
        }
    }

    /// Gene `sentry_needs_sites`: a small Hard empire with no known good city site and an
    /// unexplored frontier sends its Scouts exploring before any sentry or watch post.
    fn scouts_needed_for_sites(&self) -> bool {
        self.hard_defense()
            && self.decisions.sentry_needs_sites > 0
            && self.ctx.own_cities.len() < 4
            && !self.ctx.frontier.is_empty()
            && self.count_good_sites(1) == 0
    }

    /// Explore toward a unit we actually saw from the public leader's other war. A city
    /// must be found before the second-front plan can pass its reachability gate.
    fn busy_leader_scout_target(&self, scout: &UnitView) -> Option<Hex> {
        if !self.hard_defense() || self.decisions.busy_leader_front == 0 {
            return None;
        }
        let v = self.view();
        let leader = self.runaway_read()?.team;
        if v.teams_at_war(v.team, leader)
            || !v.players.iter().any(|p| {
                p.alive && p.team != v.team && p.team != leader && v.teams_at_war(leader, p.team)
            })
            || v.cities.iter().any(|city| {
                v.team_of(city.owner) == leader
                    && self
                        .ctx
                        .own_cities
                        .iter()
                        .any(|home| home.pos.distance(city.pos) <= 12)
            })
        {
            return None;
        }
        let (at, _) = self
            .opponents?
            .threats
            .approach_sightings
            .iter()
            .filter(|(id, (_, turn))| {
                v.team_of(**id) == leader && v.turn.saturating_sub(*turn) <= 12
            })
            .map(|(id, (at, turn))| (*at, (*turn, std::cmp::Reverse(*id))))
            .max_by_key(|(_, key)| *key)?;
        if !self
            .ctx
            .own_cities
            .iter()
            .any(|home| home.pos.distance(at) <= 18)
        {
            return None;
        }
        // An explored border is a firmer lead than the unit's old sighting. We still
        // stand on permitted land outside it and reveal the city with ordinary sight.
        let border: Vec<_> = v
            .tiles
            .iter()
            .enumerate()
            .filter(|(_, tile)| {
                tile.explored && tile.owner.is_some_and(|owner| v.team_of(owner) == leader)
            })
            .map(|(i, _)| self.ctx.hex_at(i))
            .filter(|h| {
                self.ctx
                    .own_cities
                    .iter()
                    .any(|home| home.pos.distance(*h) <= 12)
            })
            .collect();
        let safe_spot = |h: Hex| {
            v.tile(h).is_some_and(|t| {
                t.explored
                    && t.terrain.and_then(|terrain| terrain.move_cost()).is_some()
                    && v.passage_allowed(v.me, scout.kind, t.owner)
            }) && self.safe_scout_tile(scout, h)
                && self.ctx.reachable(scout.pos, h)
                && self.campaign_approach(scout, h).is_some()
        };
        if let Some(target) = border
            .iter()
            .flat_map(|b| b.within(2).into_iter().map(move |h| (*b, h)))
            .filter(|(b, h)| h.distance(*b) >= 1 && *h != scout.pos && safe_spot(*h))
            .filter_map(|(b, h)| {
                let unseen = h
                    .within(2)
                    .into_iter()
                    .filter(|near| v.tile(*near).is_some_and(|t| !t.explored))
                    .count() as i32;
                (unseen > 0).then_some((h, unseen * 20 - h.distance(b) * 5 - scout.pos.distance(h)))
            })
            .max_by_key(|(h, score)| (*score, *h))
            .map(|(h, _)| h)
        {
            return Some(target);
        }
        at.within(5)
            .into_iter()
            .filter(|h| {
                v.tile(*h).is_some_and(|t| {
                    t.terrain.and_then(|t| t.move_cost()).is_some()
                        && v.passage_allowed(v.me, scout.kind, t.owner)
                }) && self.safe_campaign_tile(*h)
                    && self.ctx.reachable(scout.pos, *h)
            })
            .filter_map(|h| {
                let unexplored = h
                    .within(2)
                    .into_iter()
                    .filter(|near| v.tile(*near).is_some_and(|t| !t.explored))
                    .count() as i32;
                (unexplored > 0).then_some((h, unexplored))
            })
            .max_by_key(|(h, unseen)| (unseen * 10 - h.distance(at), -scout.pos.distance(*h), *h))
            .map(|(h, _)| h)
    }

    /// Keep a cheap observer on the line between our city and a strong rival's known city.
    /// Hills extend the actual core sight radius by one; an old sighting can move the watch
    /// point toward the last known army route, but is never treated as a live target.
    fn watch_approaches(&mut self, scouts: &[&UnitView]) {
        if !self.hard_defense() || self.decisions.sentry_posts == 0 {
            return;
        }
        let v = self.view();
        let mut rivals: Vec<_> = v
            .players
            .iter()
            .filter(|p| p.alive && !v.same_team(v.me, p.id))
            .filter_map(|p| {
                let value = self.information_value(p.id) * self.decisions.sentry_posts / 100;
                let city = v
                    .cities
                    .iter()
                    .filter(|c| c.owner == p.id)
                    .min_by_key(|c| {
                        self.ctx
                            .own_cities
                            .iter()
                            .map(|home| home.pos.distance(c.pos))
                            .min()
                            .unwrap_or(i32::MAX)
                    })?;
                (value >= 60 || self.top_threat_team() == Some(p.team))
                    .then_some((value, p.id, city.pos))
            })
            .collect();
        rivals.sort_by_key(|(value, id, _)| (std::cmp::Reverse(*value), *id));
        for (value, rival, city) in rivals {
            let Some(home) = self
                .ctx
                .own_cities
                .iter()
                .min_by_key(|c| c.pos.distance(city))
            else {
                return;
            };
            let home = home.pos;
            let stale = v
                .observed_threats
                .iter()
                .filter(|o| o.owner == rival && v.turn.saturating_sub(o.seen_turn) >= 3)
                .max_by_key(|o| o.seen_turn);
            let target = stale.map_or(city, |o| o.pos);
            let Some(s) = scouts
                .iter()
                .copied()
                .filter(|s| !self.has_acted(s.id) && self.unit_exposure_key(s, s.pos).0 == 0)
                .min_by_key(|s| (s.pos.distance(target), s.id))
            else {
                return;
            };
            let distance = home.distance(target);
            let post = target
                .within(10)
                .into_iter()
                .filter(|h| {
                    let from_home = h.distance(home);
                    let at_sighting = if stale.is_some() {
                        let sight = legatum_core::data::UNIT_SIGHT
                            + i32::from(
                                v.tile(*h).and_then(|t| t.terrain)
                                    == Some(legatum_core::Terrain::Hills),
                            );
                        (2..=sight).contains(&h.distance(target))
                    } else {
                        h.distance(target) >= 4
                    };
                    (4..=8).contains(&from_home)
                        && at_sighting
                        && from_home + h.distance(target) <= distance + 1
                        && v.tile(*h).is_some_and(|t| {
                            t.explored
                                && t.terrain.and_then(|t| t.move_cost()).is_some()
                                && v.passage_allowed(v.me, s.kind, t.owner)
                        })
                        && self.free_tile_for(s, *h)
                        && self.safe_scout_tile(s, *h)
                        && self.ctx.reachable(s.pos, *h)
                })
                .max_by_key(|h| {
                    let hill =
                        v.tile(*h).and_then(|t| t.terrain) == Some(legatum_core::Terrain::Hills);
                    let sight = legatum_core::data::UNIT_SIGHT + i32::from(hill);
                    let score = value * (h.distance(home) + sight).min(9) / 10
                        + if hill { value / 8 } else { 0 }
                        - 5 * s.pos.distance(*h)
                        - 10 * (8 - h.distance(target)).max(0);
                    (score, std::cmp::Reverse(*h))
                });
            if let Some(post) = post {
                let to = self.campaign_approach(s, post).unwrap_or(post);
                if self.safe_scout_tile(s, to) {
                    self.move_or_hold(s, to);
                }
            }
        }
    }

    /// Hard next to a rusher (`OpponentAssessment::aggressive_neighbor`: rush-capable by its
    /// public rules within striking range, or massing at our border, or a declarer of wars):
    /// one Scout stands sentry on the tile we may enter that is closest to its city nearest us,
    /// so its army is seen while it gathers and the border readiness (Walls, a shooter, the
    /// defended expansion) starts in time (user, 2026-10-07: "I park my scout around their
    /// cities to get visibility on their troop movements... to get an early warning").
    fn post_sentry(&mut self, scouts: &[&UnitView]) {
        if !self.hard_defense() || self.decisions.sentry == 0 {
            return;
        }
        let Some(model) = self.opponents else {
            return;
        };
        let v = self.view();
        let Some(home) = self
            .ctx
            .capital()
            .or_else(|| self.ctx.own_cities.first().copied())
        else {
            return;
        };
        // Each watched rival's city nearest our cities: rushers (`aggressive_neighbor`) always,
        // and with `sentry` 2 every rival within the neighbour radius (research item 1: "one
        // scout per bordering rival"), rushers first.
        let nearest = |c: &&legatum_core::view::CityView| {
            self.ctx
                .own_cities
                .iter()
                .map(|o| o.pos.distance(c.pos))
                .min()
                .unwrap_or(i32::MAX)
        };
        let mut watched: Vec<(bool, i32, Hex)> = Vec::new();
        for p in v
            .players
            .iter()
            .filter(|p| p.alive && !v.same_team(v.me, p.id))
        {
            let rusher = model
                .rivals
                .get(&p.id)
                .is_some_and(|r| r.aggressive_neighbor());
            let Some(c) = v
                .cities
                .iter()
                .filter(|c| c.owner == p.id)
                .min_by_key(|c| (nearest(c), c.id))
            else {
                continue;
            };
            let d = nearest(&c);
            if rusher || self.decisions.sentry >= 2 && d <= crate::opponents::NEIGHBOR_RADIUS {
                watched.push((!rusher, d, c.pos));
            }
        }
        watched.sort();
        let mut free: Vec<&UnitView> = scouts
            .iter()
            .copied()
            .filter(|s| !self.has_acted(s.id))
            .collect();
        for (_, _, target) in watched {
            let Some(i) =
                (0..free.len()).min_by_key(|i| (free[*i].pos.distance(target), free[*i].id))
            else {
                return;
            };
            let s = free.swap_remove(i);
            self.post_sentry_at(s, target, home.pos);
        }
    }

    fn post_sentry_at(&mut self, s: &UnitView, target: Hex, home: Hex) {
        let v = self.view();
        let post = target
            .within(SENTRY_RADIUS)
            .into_iter()
            .filter(|h| {
                v.tile(*h).is_some_and(|t| {
                    t.explored
                        && t.terrain.and_then(|terrain| terrain.move_cost()).is_some()
                        && v.passage_allowed(v.me, s.kind, t.owner)
                }) && (*h == s.pos || self.free_tile_for(s, *h))
                    && self.safe_scout_tile(s, *h)
                    && self.ctx.reachable(s.pos, *h)
            })
            .min_by_key(|h| {
                (
                    h.distance(target),
                    v.tile(*h).and_then(|t| t.terrain) != Some(legatum_core::Terrain::Hills),
                    h.distance(home),
                    *h,
                )
            });
        let Some(post) = post else {
            return;
        };
        if post == s.pos {
            self.unit_order(s.id, Order::SkipUnit { unit: s.id });
        } else {
            let to = self.campaign_approach(s, post).unwrap_or(post);
            self.move_or_hold(s, to);
        }
    }

    /// A worthwhile frontier, ruin or reconnaissance tile with a safe known route.
    /// Use the retreat policy's live reach and unexpired sightings, never hidden units.
    pub fn choose_frontier(&self, s: &UnitView, taken: &[Hex]) -> Option<Hex> {
        let ctx = &self.ctx;
        let blocked: Vec<_> = (0..self.view().tiles.len())
            .map(|i| {
                let h = ctx.hex_at(i);
                !self.safe_scout_tile(s, h) || !self.free_tile_for(s, h)
            })
            .collect();
        let field = ctx.distance_field_avoiding(s.pos, true, Some(&blocked));
        let find = self.value_frontier();
        let mut best: Option<(i32, Hex)> = None;
        for &h in &ctx.frontier {
            if h == s.pos || taken.iter().any(|t| t.distance(h) <= 3) {
                continue;
            }
            let Some(i) = ctx.view.index(h) else { continue };
            let d = field[i];
            if d == UNREACHABLE {
                continue;
            }
            let mut cost = match &find {
                Some(f) => f.score(self, h, d),
                None => crate::exploration::frontier_score(ctx, h, d, self.decisions.ruin_bonus),
            };
            if self.decisions.approach_scout > 0
                && self.view().turn <= self.decisions.approach_scout_turn as u32
                && self
                    .view()
                    .players
                    .iter()
                    .any(|p| p.alive && p.faction == legatum_core::Faction::Byzantium)
                && let Some(home) = ctx.own_cities.first()
                && !ctx.enemy_cities.iter().any(|city| {
                    home.pos.distance(city.pos) <= self.decisions.lone_city_strike_radius
                })
            {
                // Reveal the nearby approach before following distant ruins: an
                // unwalled capital is an opening target only for a short window.
                let from_home = home.pos.distance(h);
                if from_home <= self.decisions.lone_city_strike_radius {
                    cost -= from_home * 20;
                } else {
                    cost += 100;
                }
            }
            if self.decisions.recon_weight > 0 {
                // Their passage makes distant reconnaissance useful before a war opens roads.
                let from_home = ctx
                    .own_cities
                    .iter()
                    .map(|c| c.pos.distance(h))
                    .min()
                    .unwrap_or(0);
                cost -= from_home.min(12) * self.decisions.recon_weight;
            }
            if s.goto == Some(h) {
                if Self::stuck(s) {
                    continue;
                }
                cost -= 15;
            }
            if best.is_none_or(|(b, _)| cost < b) {
                best = Some((cost, h));
            }
        }
        // Exploration is not finished when the map is painted. Revisit remembered enemy
        // cities from a safe tile within sight, so campaigns get current defenders and owners.
        // The top threat's front comes first: cities of empires it is fighting may already be
        // its (public city counts move, but not which city), and its own may be weakly held.
        let threat = self.top_threat_team();
        for city in ctx.enemy_cities.iter().filter(|c| !c.visible) {
            let owner = self.view().team_of(city.owner);
            let front = threat.is_some_and(|t| owner == t || self.view().teams_at_war(owner, t));
            for h in city.pos.ring(legatum_core::data::UNIT_SIGHT) {
                if h == s.pos || taken.iter().any(|t| t.distance(h) <= 3) {
                    continue;
                }
                let Some(i) = ctx.view.index(h) else { continue };
                let d = field[i];
                if d == UNREACHABLE || ctx.friendly_military_at[i] || ctx.hostile_at[i] {
                    continue;
                }
                // Gene `stale_intel`: a city not seen for a long time is looked at before
                // any war is declared on it.
                let stale = if self.stale_city(city) { 300 } else { 0 };
                let cost =
                    d * 10 - self.conquest_pressure() / 2 - if front { 100 } else { 0 } - stale;
                if best.is_none_or(|(b, _)| cost < b) {
                    best = Some((cost, h));
                }
            }
        }
        best.map(|(_, h)| h)
    }
}

#[cfg(test)]
mod sparta_1025_founding_tests {
    use super::*;
    use crate::{Brain, Knobs, ScriptedBrain};
    use flate2::read::GzDecoder;
    use legatum_core::{AiDifficulty, Faction, Hex, PlayerView, UnitId};
    use std::io::Read;

    // Original setup.seed in sparta-win-final-20261025-0902.legatum.
    const SEED: u64 = 7_732_777_812_994_077;

    fn view() -> PlayerView {
        let mut data = Vec::new();
        GzDecoder::new(include_bytes!("fixtures/sparta-1025/T129-p2.view.json.gz").as_slice())
            .read_to_end(&mut data)
            .expect("historical view gzip");
        serde_json::from_slice(&data).expect("historical India view")
    }

    fn brain(v: &PlayerView, weight: i32) -> ScriptedBrain {
        let mut brain = ScriptedBrain::with_knobs(
            Faction::India,
            SEED,
            v.me,
            Knobs::for_faction(Faction::India),
        );
        brain.difficulty = AiDifficulty::Hard;
        brain.decisions.founding_survival_weight = weight;
        brain
    }

    fn settler_89_order(orders: &[Order]) -> Order {
        orders
            .iter()
            .find(|order| match order {
                Order::FoundCity { unit }
                | Order::MoveUnit { unit, .. }
                | Order::MoveUnitPath { unit, .. }
                | Order::SkipUnit { unit } => *unit == UnitId(89),
                _ => false,
            })
            .expect("unit 89 receives an order")
            .clone()
    }

    #[test]
    fn exposed_kyoto_is_deferred_for_a_walled_home_city() {
        let v = view();
        let settler = v.units.iter().find(|u| u.id == UnitId(89)).unwrap();
        assert_eq!(
            settler_89_order(&brain(&v, 0).plan(&v)),
            Order::FoundCity { unit: settler.id }
        );

        let mut p = Planner::new(&v, Knobs::for_faction(Faction::India), Faction::India, SEED);
        p.difficulty = AiDifficulty::Hard;
        p.decisions.founding_survival_weight = 200;
        assert_eq!(p.founding_damage_before_defense(settler.pos, 0), 100);
        assert_eq!(p.founding_damage_before_defense(Hex::new(17, 13), 2), 100);
        assert_eq!(p.founding_survival_cost(100, 50), 100);
        assert_eq!(p.founding_survival_cost(100, 100), 200);

        let improved = settler_89_order(&brain(&v, 200).plan(&v));
        let home = Hex::new(13, 14);
        let path = [Hex::new(15, 12), Hex::new(14, 13), home];
        assert_eq!(
            improved,
            Order::MoveUnitPath {
                unit: settler.id,
                to: home,
                path: path.to_vec(),
                stop_on_block: true,
            }
        );
        assert!(
            v.cities
                .iter()
                .any(|c| c.pos == home && c.owner == v.me && c.walls)
        );
        let mut from = settler.pos;
        for to in path {
            assert_eq!(from.distance(to), 1);
            assert!(
                v.tile(to)
                    .is_some_and(|t| t.explored && t.move_cost().is_some())
            );
            assert_eq!(
                v.movement_path(v.me, settler.kind, from, to),
                Some(vec![to])
            );
            from = to;
        }

        // Same historical view with only the visible shooter removed: the gene still founds.
        let mut no_shooter = v.clone();
        no_shooter.units.retain(|u| u.id != UnitId(40));
        assert_eq!(
            settler_89_order(&brain(&no_shooter, 200).plan(&no_shooter)),
            Order::FoundCity { unit: settler.id }
        );
    }
}

/// Gene `scout_find`: what exploring a frontier tile is worth to the Scout in the opening.
/// Every unexplored tile it would reveal counts 3, as in `frontier_score`. While some rival's
/// capital is still unseen, each such tile also counts one point per hex of reach it lies
/// inside our nearest city, scaled by the gene's percent: the land around home is where the
/// neighbours, the sites and the ruins that matter are, and a Scout that chases the farthest
/// edge first learns them late.
pub(crate) struct ValueFrontier {
    /// The near-home weight applies on this turn.
    near: bool,
    /// The distance from our cities beyond which unexplored land adds nothing (the map's
    /// expected spacing between capitals, with room for the neighbour next door).
    reach: i32,
    /// `scout_find`, in percent.
    weight: i32,
}

impl ValueFrontier {
    pub(crate) fn score(&self, p: &Planner<'_>, h: Hex, distance: i32) -> i32 {
        let ctx = &p.ctx;
        let home = |t: Hex| {
            ctx.own_cities
                .iter()
                .map(|c| c.pos.distance(t))
                .min()
                .unwrap_or(self.reach)
        };
        let info: i32 = h
            .within(legatum_core::data::UNIT_SIGHT)
            .into_iter()
            .filter(|n| p.view().tile(*n).is_some_and(|t| !t.explored))
            .map(|n| {
                3 + if self.near {
                    (self.reach - home(n)).max(0) * self.weight / 100
                } else {
                    0
                }
            })
            .sum();
        let bonus = p.decisions.ruin_bonus;
        let ruin = p.view().tile(h).is_some_and(|t| t.ruin);
        distance * 10 - info - if ruin { bonus } else { 0 }
    }
}

impl Planner<'_> {
    /// Gene `scout_find` for this planner, with the facts it needs this turn.
    pub(crate) fn value_frontier(&self) -> Option<ValueFrontier> {
        if !self.hard_defense() || self.decisions.scout_find == 0 {
            return None;
        }
        let v = self.view();
        let majors = v
            .players
            .iter()
            .filter(|p| p.alive && !p.id.is_barbarian())
            .count()
            .max(2) as i32;
        let spacing = (v.width * v.height / majors).max(1).isqrt();
        let in_window = v.turn <= self.decisions.scout_find_turns as u32;
        let unseen_capital = v.players.iter().any(|p| {
            p.alive
                && !p.id.is_barbarian()
                && !v.same_team(v.me, p.id)
                && !v.cities.iter().any(|c| c.owner == p.id)
        });
        Some(ValueFrontier {
            near: in_window && unseen_capital,
            reach: (spacing * 13 / 10).clamp(10, 24),
            weight: self.decisions.scout_find,
        })
    }
}

/// How far from the watched city a sentry post may be (`post_sentry`).
const SENTRY_RADIUS: i32 = 6;

#[cfg(test)]
mod timeout_tests {
    use super::*;
    use crate::{
        Knobs,
        fixture::{ENEMY, ME, ViewBuilder, off},
    };
    use legatum_core::Terrain;
    use legatum_core::{Faction, UnitId};

    #[test]
    fn idle_settler_values_safe_own_border_ruin() {
        let ruin = off(7, 5);
        let view = ViewBuilder::new(12, 12)
            .city(1, ME, off(5, 5), 3)
            .owner(ruin, ME)
            .ruin(ruin)
            .unit(1, ME, UnitKind::Settler, off(6, 5))
            .build();
        let planner = Planner::new(
            &view,
            Knobs::for_faction(Faction::Switzerland),
            Faction::Switzerland,
            7,
        );
        let (target, path) = planner
            .idle_settler_ruin(&view.units[0])
            .expect("safe ruin should beat waiting");
        assert_eq!(target, ruin);
        assert_eq!(path.last(), Some(&ruin));
    }

    #[test]
    fn old_settler_drops_distant_commitment_at_standard_deadline() {
        // Coach g10 native-1790589094877907000-38765-53, P5 T31:
        // fourteen unfounded turns despite available settlement land.
        let far = off(10, 3);
        let mut view = ViewBuilder::new(24, 18)
            .city(1, ME, off(6, 8), 3)
            .patch(far, 2, Terrain::Forest)
            .unit(1, ME, UnitKind::Settler, off(6, 8))
            .goto(1, far)
            .build();
        view.turn_limit = None;
        view.units[0].created_turn = Some(3);
        view.turn = 3 + 20 - 1;
        let mut p = Planner::new(
            &view,
            Knobs::for_faction(Faction::Switzerland),
            Faction::Switzerland,
            7,
        );
        assert_eq!(p.choose_site(&view.units[0]), Some(far));
        // The gene changes patience, not the legal-site checks.
        p.decisions.settle_patience = 8;
        assert_ne!(p.choose_site(&view.units[0]), Some(far));
        view.turn += 1;
        let p = Planner::new(
            &view,
            Knobs::for_faction(Faction::Switzerland),
            Faction::Switzerland,
            7,
        );
        let site = p.choose_site(&view.units[0]).expect("nearby legal land");
        assert_ne!(site, far);
        assert!(site.distance(view.units[0].pos) <= view.units[0].max_moves * 2);
        assert!(legatum_core::settlement::known_legal_site(&view, site));
    }

    #[test]
    fn timed_out_settler_founds_without_random_wait_but_never_into_danger() {
        let mut view = ViewBuilder::new(20, 16)
            .unit(1, ME, UnitKind::Settler, off(8, 8))
            .build();
        view.turn = 21;
        view.units[0].created_turn = Some(1);
        for seed in 0..20 {
            let mut p = Planner::new(
                &view,
                Knobs::for_faction(Faction::Switzerland),
                Faction::Switzerland,
                seed,
            );
            p.plan_settlers();
            assert_eq!(p.orders, vec![Order::FoundCity { unit: UnitId(1) }]);
        }
        view = ViewBuilder { view }
            .unit(2, ENEMY, UnitKind::Spearman, off(9, 8))
            .build();
        let mut p = Planner::new(
            &view,
            Knobs::for_faction(Faction::Switzerland),
            Faction::Switzerland,
            7,
        );
        p.plan_settlers();
        assert!(
            !p.orders
                .iter()
                .any(|o| matches!(o, Order::FoundCity { .. }))
        );
    }
    #[test]
    fn timeout_keeps_traveling_when_home_is_surrounded_by_cities() {
        let center = Hex::new(12, 10);
        let mut builder =
            ViewBuilder::new(40, 24)
                .city(1, ME, center, 3)
                .unit(1, ME, UnitKind::Settler, center);
        for (i, direction) in center.neighbors().into_iter().enumerate() {
            let delta = Hex::new(direction.q - center.q, direction.r - center.r);
            builder = builder.city(
                i as u32 + 2,
                ME,
                Hex::new(center.q + 5 * delta.q, center.r + 5 * delta.r),
                2,
            );
        }
        let mut view = builder.build();
        view.turn_limit = None;
        view.turn = 30;
        view.units[0].created_turn = Some(1);
        let mut p = Planner::new(
            &view,
            Knobs::for_faction(Faction::Switzerland),
            Faction::Switzerland,
            7,
        );
        assert!(
            legatum_core::settlement::settlement_candidates(&view, &view.units[0], &[])
                .iter()
                .all(|site| p.nearby_settler_route(&view.units[0], site.pos).is_none())
        );
        let site = p
            .choose_site(&view.units[0])
            .expect("safe distant expansion land still exists");
        assert_ne!(site, center);
        p.plan_settlers();
        assert!(
            p.orders
                .iter()
                .any(|o| matches!(o, Order::MoveUnit { unit, to }
            | Order::MoveUnitPath { unit, to, .. } if *unit == UnitId(1) && *to != center))
        );
    }
}

#[cfg(test)]
mod scouting_tests {
    use super::*;
    use crate::{
        Knobs,
        fixture::{ENEMY, ME, ViewBuilder, off},
    };
    use legatum_core::{AiDifficulty, Faction};

    fn hard(v: &legatum_core::PlayerView, weight: i32) -> Planner<'_> {
        let mut p = Planner::new(v, Knobs::for_faction(Faction::Sparta), Faction::Sparta, 3);
        p.difficulty = AiDifficulty::Hard;
        p.decisions.scout_find = weight;
        p
    }

    /// Our city on the left of a 48x32 map for 8 players, with unexplored land to the east.
    fn view_with_rivals_unfound() -> legatum_core::PlayerView {
        ViewBuilder::new(48, 32)
            .city(1, ME, off(6, 16), 3)
            .unexplored_from_col(14)
            .unit(1, ME, UnitKind::Scout, off(8, 16))
            .build()
    }

    #[test]
    fn unfound_rivals_make_the_land_near_home_worth_more() {
        let v = view_with_rivals_unfound();
        let p = hard(&v, 100);
        let find = p.value_frontier().expect("gene on");
        assert!(find.near, "three rivals, none of their cities known");
        let near = find.score(&p, off(13, 16), 30);
        let far = find.score(&p, off(13, 2), 30);
        assert!(
            near < far,
            "land beside home beats the same view far away ({near} vs {far})"
        );
        // A larger weight values the near land more; the usual score is 3 a tile.
        let heavy = hard(&v, 400);
        assert!(
            heavy
                .value_frontier()
                .unwrap()
                .score(&heavy, off(13, 16), 30)
                < near,
            "a bigger weight pulls harder"
        );
        // Every rival located: the weight stops and the tile is worth the flat 3 a tile.
        let found = ViewBuilder { view: v.clone() }
            .city(9, ENEMY, off(20, 16), 3)
            .city(10, legatum_core::PlayerId(2), off(22, 10), 3)
            .city(11, legatum_core::PlayerId(3), off(22, 22), 3)
            .build();
        let pf = hard(&found, 100);
        let f = pf.value_frontier().unwrap();
        assert!(!f.near);
        assert!(near < f.score(&pf, off(13, 16), 30));
    }

    #[test]
    fn the_weight_ends_with_its_window_and_the_gene_off_changes_nothing() {
        let mut v = view_with_rivals_unfound();
        v.turn = 30;
        assert!(hard(&v, 100).value_frontier().unwrap().near);
        v.turn = 90;
        assert!(
            !hard(&v, 100).value_frontier().unwrap().near,
            "past turn 40 at Standard"
        );
        assert!(hard(&v, 0).value_frontier().is_none(), "gene off");
        let mut off_planner = hard(&v, 100);
        off_planner.difficulty = AiDifficulty::Normal;
        assert!(off_planner.value_frontier().is_none(), "Hard only");
    }
}

#[cfg(test)]
mod found_here_tests {
    //! Gene `found_here` (decision audit round 3, 2026-10-31).
    use super::*;
    use crate::{
        Knobs,
        fixture::{ENEMY, ME, ViewBuilder, off},
    };
    use legatum_core::{Faction, PlayerView};

    fn planner(view: &PlayerView, gene: i32) -> Planner<'_> {
        let mut p = Planner::new(
            view,
            Knobs::for_faction(Faction::Switzerland),
            Faction::Switzerland,
            7,
        );
        p.decisions.found_here = gene;
        p
    }

    fn founds(orders: &[Order]) -> bool {
        orders
            .iter()
            .any(|o| matches!(o, Order::FoundCity { unit } if *unit == UnitId(1)))
    }

    /// A Settler on a legal site with enemy Spearmen (2 moves) at `distance` from it.
    fn settler_with_spearmen(distance: i32, spearmen: usize) -> PlayerView {
        let settler = off(9, 8);
        let mut b = ViewBuilder::new(20, 16).city(1, ME, off(3, 8), 3).unit(
            1,
            ME,
            UnitKind::Settler,
            settler,
        );
        for (i, h) in settler
            .ring(distance)
            .into_iter()
            .take(spearmen)
            .enumerate()
        {
            b = b.unit(2 + i as u32, ENEMY, UnitKind::Spearman, h);
        }
        b.build()
    }

    #[test]
    fn a_settler_founds_beside_an_army_that_cannot_reach_it_this_turn() {
        // Within the threat radius (3), beyond a Spearman's two moves.
        let view = settler_with_spearmen(3, 1);
        let mut off_planner = planner(&view, 0);
        off_planner.plan_settlers();
        assert!(
            !founds(&off_planner.orders),
            "the site search drops a tile with an enemy army near"
        );
        let mut on = planner(&view, 1);
        on.plan_settlers();
        assert!(founds(&on.orders), "one Spearman can't take a fresh city");
        // The order is remembered, so a failed found is not repeated next turn.
        assert_eq!(
            on.campaigns.found_tried,
            vec![(UnitId(1), off(9, 8), view.turn)]
        );
    }

    #[test]
    fn a_settler_an_enemy_can_reach_this_turn_does_not_found_there() {
        // Founding resolves after movement: the Spearman would capture it first.
        let view = settler_with_spearmen(1, 1);
        let mut p = planner(&view, 1);
        let ((), notes) = trace_settler(UnitId(1), None, || p.plan_settlers());
        assert!(!founds(&p.orders));
        assert!(
            notes.iter().any(|n| n.contains("capturable this turn")),
            "{notes:?}"
        );
    }

    #[test]
    fn an_army_that_would_take_the_fresh_city_keeps_the_settler_off_the_tile() {
        let view = settler_with_spearmen(3, 6);
        let mut p = planner(&view, 1);
        let ((), notes) = trace_settler(UnitId(1), None, || p.plan_settlers());
        assert!(!founds(&p.orders));
        let loss = notes.iter().find_map(|n| {
            n.split("loss ")
                .nth(1)?
                .split('%')
                .next()?
                .parse::<i32>()
                .ok()
        });
        assert!(
            loss.is_some_and(|l| l >= 50),
            "the fresh city is forecast to fall: {notes:?}"
        );
    }

    #[test]
    fn a_found_that_failed_here_last_turn_is_not_repeated() {
        let view = settler_with_spearmen(3, 1);
        let mut p = planner(&view, 1);
        p.campaigns
            .found_tried
            .push((UnitId(1), off(9, 8), view.turn.saturating_sub(1)));
        p.plan_settlers();
        assert!(!founds(&p.orders));
    }
}

#[cfg(test)]
mod settler_shelter_tests {
    //! Gene `settler_shelter` (decision audit round 5, 2026-10-31).
    use super::*;
    use crate::{
        Knobs,
        fixture::{ENEMY, ME, ViewBuilder, off},
    };
    use legatum_core::Faction;

    #[test]
    fn a_settler_in_a_city_one_spearman_cannot_break_stays_inside() {
        // A Spearman two hexes away reaches the city this turn but cannot take a full city.
        let city = off(6, 8);
        let view = ViewBuilder::new(20, 16)
            .city(1, ME, city, 3)
            .unit(1, ME, UnitKind::Settler, city)
            .unit(2, ENEMY, UnitKind::Spearman, off(8, 8))
            .build();
        let notes = |gene| {
            let mut p = Planner::new(
                &view,
                Knobs::for_faction(Faction::Switzerland),
                Faction::Switzerland,
                7,
            );
            p.decisions.settler_shelter = gene;
            assert_eq!(p.city_shield(city), crate::tactics::CityShield::Full);
            let ((), notes) = trace_settler(UnitId(1), None, || p.plan_settlers());
            (notes, p.orders)
        };
        let (off_notes, _) = notes(0);
        assert!(
            off_notes.iter().any(|n| n.starts_with("exposed at")),
            "gene off: the Spearman's reach makes the Settler retreat: {off_notes:?}"
        );
        let (on_notes, _) = notes(1);
        assert!(
            !on_notes.iter().any(|n| n.starts_with("exposed at")),
            "gene on: the held city shelters it: {on_notes:?}"
        );
        assert!(
            on_notes.iter().any(|n| n.starts_with("chosen")),
            "gene on: the site search plans it (a safe route or a wait): {on_notes:?}"
        );
    }
}

#[cfg(test)]
mod border_explained_tests {
    //! Gene `border_explained` (decision audit round 4, 2026-10-31).
    use super::*;
    use crate::fixture::{ENEMY, ME, ViewBuilder, off};
    use legatum_core::PlayerView;
    use legatum_core::settlement::{border_city_risks, known_legal_site};

    fn hex(view: &PlayerView, i: usize) -> Hex {
        Hex::from_offset(i as i32 % view.width, i as i32 / view.width)
    }

    /// Our city in the west, a known enemy city whose border (radius 2) we see, and the land
    /// past that border explored but out of sight now.
    fn behind_a_known_border() -> PlayerView {
        let enemy_city = off(11, 8);
        let mut b = ViewBuilder::new(24, 16)
            .city(1, ME, off(3, 8), 3)
            .city(2, ENEMY, enemy_city, 3)
            .unit(1, ME, UnitKind::Settler, off(6, 8));
        for h in enemy_city.within(2) {
            b = b.owner(h, ENEMY);
        }
        let mut view = b.build();
        let width = view.width;
        for (i, t) in view.tiles.iter_mut().enumerate() {
            if (i as i32) % width >= 13 {
                t.visible = false;
            }
        }
        view
    }

    fn flagged_legal_sites(view: &PlayerView) -> Vec<Hex> {
        let core = border_city_risks(view);
        (0..view.tiles.len())
            .filter(|&i| core[i] && known_legal_site(view, hex(view, i)))
            .map(|i| hex(view, i))
            .collect()
    }

    #[test]
    fn a_border_a_known_city_explains_implies_no_unseen_city() {
        let view = behind_a_known_border();
        let flagged = flagged_legal_sites(&view);
        assert!(
            !flagged.is_empty(),
            "the shared rule flags legal sites here"
        );
        let mine = unexplained_border_risks(&view);
        assert!(flagged.iter().all(|&h| !mine[view.index(h).unwrap()]));
        // Without the city in view, the same border is evidence again.
        let mut unknown = view.clone();
        unknown.cities.retain(|c| c.owner != ENEMY);
        assert_eq!(
            unexplained_border_risks(&unknown),
            border_city_risks(&unknown)
        );
    }

    #[test]
    fn the_gene_lets_the_site_search_consider_a_site_behind_an_explained_border() {
        let view = behind_a_known_border();
        let probe = *flagged_legal_sites(&view)
            .iter()
            .min_by_key(|h| (h.distance(off(6, 8)), **h))
            .expect("a flagged legal site");
        let notes = |gene: i32| {
            let mut p = crate::fixture::planner(&view, 7);
            p.decisions.border_explained = gene;
            trace_settler(UnitId(1), Some(probe), || p.plan_settlers()).1
        };
        let off_notes = notes(0);
        assert!(
            off_notes
                .iter()
                .any(|n| n.starts_with("probe tile not a candidate")),
            "{off_notes:?}"
        );
        let on_notes = notes(1);
        assert!(
            on_notes
                .iter()
                .any(|n| n.starts_with("probe tile candidate")),
            "{on_notes:?}"
        );
    }
}

#[cfg(test)]
mod fallback_risk_tests {
    //! Gene `fallback_risk` (decision audit round 4, 2026-10-31).
    use super::*;
    use crate::fixture::{ENEMY, ME, ViewBuilder, off};

    fn chosen(notes: &[String]) -> Option<Hex> {
        let n = notes.iter().find(|n| n.starts_with("chosen "))?;
        let q = n
            .split("q: ")
            .nth(1)?
            .split(',')
            .next()?
            .trim()
            .parse()
            .ok()?;
        let r = n
            .split("r: ")
            .nth(1)?
            .split(' ')
            .next()?
            .trim()
            .parse()
            .ok()?;
        Some(Hex { q, r })
    }

    #[test]
    fn a_far_fallback_no_safer_than_the_near_site_loses_to_it() {
        // At war: an enemy Spearman four rows south of the Settler (out of reach this turn)
        // puts every site's trip at risk, near and far alike; the far sites (beyond the
        // preferred distance) are the fallback the search never charged for it.
        let city = off(3, 8);
        let view = ViewBuilder::new(30, 18)
            .city(1, ME, city, 3)
            .unit(1, ME, UnitKind::Settler, off(5, 8))
            .unit(2, ENEMY, UnitKind::Spearman, off(5, 12))
            .build();
        let notes = |gene: i32| {
            let mut p = crate::fixture::planner(&view, 7);
            p.difficulty = legatum_core::AiDifficulty::Hard;
            p.decisions.fallback_risk = gene;
            trace_settler(UnitId(1), None, || p.plan_settlers()).1
        };
        let max = crate::DecisionParams::default().settler_max_site_distance;
        let off_notes = notes(0);
        let far = chosen(&off_notes).expect("a site");
        assert!(far.distance(city) > max, "{off_notes:?}");
        let on_notes = notes(1);
        let near = chosen(&on_notes).expect("a site");
        assert!(near.distance(city) <= max, "{on_notes:?}");
        assert!(
            on_notes.iter().any(|n| n.starts_with("fallback ")),
            "{on_notes:?}"
        );
    }
}

#[cfg(test)]
mod land_contest_tests {
    //! Gene `land_contest` (land race, 2026-11-02).
    use super::*;
    use crate::{
        Knobs,
        fixture::{ENEMY, ME, ViewBuilder, off},
    };
    use legatum_core::{AiDifficulty, Faction, PlayerView};

    fn hard(v: &PlayerView, gene: i32) -> Planner<'_> {
        let mut p = Planner::new(v, Knobs::for_faction(Faction::Germany), Faction::Germany, 3);
        p.difficulty = AiDifficulty::Hard;
        p.decisions.land_contest = gene;
        p
    }

    #[test]
    fn land_a_neighbour_would_take_is_worth_more_the_closer_it_lies() {
        let v = ViewBuilder::new(30, 12)
            .city(1, ME, off(4, 6), 3)
            .city(2, ENEMY, off(16, 6), 3)
            .build();
        let p = hard(&v, 100);
        let near = p.land_contest_value(off(13, 6), 200);
        let mid = p.land_contest_value(off(10, 6), 200);
        assert!(near > mid && mid > 0, "near {near}, mid {mid}");
        assert_eq!(
            p.land_contest_value(off(6, 6), 200),
            0,
            "out of the rival's reach"
        );
        assert!(hard(&v, 200).land_contest_value(off(13, 6), 200) > near);
        assert_eq!(
            hard(&v, 0).land_contest_value(off(13, 6), 200),
            0,
            "gene off"
        );
        let mut normal = hard(&v, 100);
        normal.difficulty = AiDifficulty::Normal;
        assert_eq!(normal.land_contest_value(off(13, 6), 200), 0, "Hard only");
    }
}

#[cfg(test)]
mod found_now_tests {
    //! Gene `found_now` (decision diff patterns, 2026-11-03).
    use super::*;
    use crate::{
        Knobs,
        fixture::{ENEMY, ME, ViewBuilder, off},
    };
    use legatum_core::{AiDifficulty, Faction, PlayerId, PlayerView};

    fn hard(view: &PlayerView, gene: i32) -> Planner<'_> {
        let mut p = Planner::new(
            view,
            Knobs::for_faction(Faction::Switzerland),
            Faction::Switzerland,
            7,
        );
        p.difficulty = AiDifficulty::Hard;
        p.decisions.found_now = gene;
        p
    }

    fn founds(p: &mut Planner<'_>) -> (bool, Vec<String>) {
        // As the brain plans a turn: the Hard danger map first.
        p.prepare_hard_defense();
        let ((), notes) = trace_settler(UnitId(1), None, || p.plan_settlers());
        let found = p
            .orders
            .iter()
            .any(|o| matches!(o, Order::FoundCity { unit } if *unit == UnitId(1)));
        (found, notes)
    }

    /// Our city, a Settler on a legal site six hexes out and one barbarian unit beside it.
    fn settler_and_barbarian(kind: UnitKind, distance: i32, on_camp: bool) -> PlayerView {
        let settler = off(9, 8);
        let barbarian = settler.ring(distance)[0];
        let mut v = ViewBuilder::new(20, 16)
            .city(1, ME, off(3, 8), 3)
            .unit(1, ME, UnitKind::Settler, settler)
            .unit(2, PlayerId::BARBARIAN, kind, barbarian)
            .build();
        if on_camp {
            v.barbarian_camps = vec![legatum_core::view::BarbarianCampView {
                pos: barbarian,
                visible: true,
            }];
        }
        v
    }

    #[test]
    fn a_barbarian_scout_beside_the_settler_does_not_stop_the_founding() {
        let view = settler_and_barbarian(UnitKind::Scout, 1, false);
        let (off_found, off_notes) = founds(&mut hard(&view, 0));
        assert!(
            !off_found,
            "gene off: the Scout's reach sends it away: {off_notes:?}"
        );
        let (on_found, notes) = founds(&mut hard(&view, 10));
        assert!(on_found, "a camp's Scout only wanders: {notes:?}");
        assert!(notes.iter().any(|n| n.contains("capture 5%")), "{notes:?}");
        let mut normal = hard(&view, 10);
        normal.difficulty = AiDifficulty::Normal;
        assert!(!founds(&mut normal).0, "Hard only");
    }

    #[test]
    fn a_camp_guard_does_not_stop_it_but_a_raider_does() {
        let guard = settler_and_barbarian(UnitKind::Spearman, 2, true);
        let (found, notes) = founds(&mut hard(&guard, 10));
        assert!(found, "the guard holds its camp: {notes:?}");
        let raider = settler_and_barbarian(UnitKind::Spearman, 2, false);
        let (found, notes) = founds(&mut hard(&raider, 10));
        assert!(!found, "a raider takes a lone civilian within its moves");
        assert!(
            notes.iter().any(|n| n.contains("capturable this turn")),
            "{notes:?}"
        );
    }

    #[test]
    fn a_rival_at_peace_beside_it_is_priced_not_a_veto() {
        let settler = off(9, 8);
        let mut v = ViewBuilder::new(24, 16)
            .city(1, ME, off(3, 8), 3)
            .city(2, ENEMY, off(20, 8), 3)
            .unit(1, ME, UnitKind::Settler, settler)
            .unit(2, ENEMY, UnitKind::Archer, settler.ring(1)[0])
            .build();
        v.diplomacy.legacy = false;
        let enemy_team = v.team_of(ENEMY);
        v.diplomacy
            .relations
            .push(legatum_core::diplomacy::Relation {
                a: v.team,
                b: enemy_team,
                state: legatum_core::diplomacy::RelationState::Peace,
                effective_turn: 1,
                guarantee_until: 0,
                version: 1,
                security: None,
                war_id: None,
                declarers: Vec::new(),
            });
        assert!(
            hard(&v, 10).found_now_capture_percent(settler) <= 10,
            "at most the prior for a surprise war (`peace_settler_capture` 0)"
        );
        let (found, notes) = founds(&mut hard(&v, 10));
        assert!(
            found,
            "a surprise war for one Settler is a chance: {notes:?}"
        );
    }

    #[test]
    fn a_remembered_foreign_border_keeps_the_settler_off_the_tile() {
        // The 8.9.24 Swiss T145 case: an enemy border seen before, out of sight now, with
        // unexplored land behind it that could hold a city within spacing of the site.
        let settler = off(9, 8);
        let mut v = ViewBuilder::new(20, 16)
            .city(1, ME, off(3, 8), 3)
            .unit(1, ME, UnitKind::Settler, settler)
            .unit(2, PlayerId::BARBARIAN, UnitKind::Scout, settler.ring(1)[0])
            .unexplored_from_col(12)
            .build();
        let border = off(11, 8);
        let i = v.index(border).unwrap();
        v.tiles[i].owner = Some(ENEMY);
        v.tiles[i].visible = false;
        let p = hard(&v, 10);
        assert!(p.remembered_border_risks()[v.index(settler).unwrap()]);
        let (found, notes) = founds(&mut hard(&v, 10));
        assert!(!found, "{notes:?}");
        assert!(
            notes.iter().any(|n| n.contains("found here: border risk")),
            "{notes:?}"
        );
    }

    #[test]
    fn land_near_a_rival_or_its_settler_may_be_gone_before_arrival() {
        let v = ViewBuilder::new(30, 12)
            .city(1, ME, off(4, 6), 3)
            .city(2, ENEMY, off(16, 6), 3)
            .build();
        let p = hard(&v, 10);
        let near = p.site_claim_percent(off(13, 6), 2);
        assert!(near > p.site_claim_percent(off(10, 6), 2));
        assert!(
            p.site_claim_percent(off(13, 6), 4) > near,
            "longer trips lose more"
        );
        assert_eq!(
            p.site_claim_percent(off(6, 6), 3),
            0,
            "beyond a rival's reach"
        );
        let mut with_settler = v.clone();
        with_settler.units.push(
            ViewBuilder::new(30, 12)
                .unit(9, ENEMY, UnitKind::Settler, off(6, 9))
                .build()
                .units[0]
                .clone(),
        );
        assert_eq!(
            hard(&with_settler, 10).site_claim_percent(off(6, 6), 1),
            50,
            "a rival Settler within reach"
        );
    }
}

#[cfg(test)]
mod peace_army_site_tests {
    use super::*;
    use crate::fixture::{ENEMY, ME, ViewBuilder, planner, set_armies};
    use legatum_core::diplomacy::{Relation, RelationState};
    use legatum_core::{AiDifficulty, Faction, TeamId};

    // T87's geometry and public army strengths, cut from the Athens 8.9.47 tape:
    // Hard's preferred (19,22) is two hexes from the Celtic Horseman and four
    // from the Celtic city; the human founded at (15,24).
    fn t87_border() -> legatum_core::PlayerView {
        let mut view = ViewBuilder::new(48, 32)
            .faction(Faction::Athens)
            .city(1, ME, Hex::new(12, 23), 3)
            .city(2, ENEMY, Hex::new(23, 18), 4)
            .unit(1, ME, UnitKind::Settler, Hex::new(15, 24))
            .unit(2, ENEMY, UnitKind::Horseman, Hex::new(18, 21))
            .build();
        view.diplomacy.legacy = false;
        view.diplomacy.relations.push(Relation {
            a: TeamId(0),
            b: TeamId(2),
            state: RelationState::Peace,
            effective_turn: 1,
            guarantee_until: 0,
            version: 1,
            security: None,
            war_id: None,
            declarers: vec![],
        });
        set_armies(&mut view, &[(ME, 131), (ENEMY, 410)]);
        view
    }

    #[test]
    fn t87_visible_peace_army_prices_the_exposed_site_and_gene_zero_restores_it() {
        let view = t87_border();
        let mut p = planner(&view, 7);
        p.difficulty = AiDifficulty::Hard;
        let exposed = Hex::new(19, 22);
        let distant = Hex::new(15, 24);
        assert_eq!(exposed.distance(Hex::new(18, 21)), 2);
        assert_eq!(exposed.distance(Hex::new(23, 18)), 4);
        assert!(p.ctx.threat_units.is_empty());
        assert_eq!(p.ctx.peace_armies.len(), 1);
        assert!(p.peace_army_site_risk(exposed) > p.peace_army_site_risk(distant));
        assert!(p.peace_army_site_score(exposed, 98) < 29);
        p.decisions.peace_army_sites = 0;
        assert_eq!(p.peace_army_site_score(exposed, 98), 98);
        assert_eq!(p.peace_army_site_score(distant, 29), 29);
    }

    #[test]
    fn t87_border_patrol_signals_the_public_army_two_turns_away() {
        let mut view = t87_border();
        view.units
            .iter_mut()
            .find(|u| u.owner == ENEMY)
            .unwrap()
            .pos = Hex::new(16, 18);
        let mut p = planner(&view, 7);
        p.difficulty = AiDifficulty::Hard;
        let exposed = Hex::new(19, 22);
        assert_eq!(exposed.distance(Hex::new(16, 18)), 7);
        assert!(p.peace_army_site_score(exposed, 98) < 29);
        p.decisions.peace_army_sites = 0;
        assert_eq!(p.peace_army_site_score(exposed, 98), 98);
    }

    #[test]
    fn alliance_and_another_war_remove_or_reduce_the_peace_price() {
        let mut view = t87_border();
        let exposed = Hex::new(19, 22);
        let mut p = planner(&view, 7);
        p.difficulty = AiDifficulty::Hard;
        let peace = p.peace_army_site_risk(exposed);
        view.diplomacy.relations[0].state = RelationState::Alliance;
        let mut ally = planner(&view, 7);
        ally.difficulty = AiDifficulty::Hard;
        assert_eq!(ally.peace_army_site_risk(exposed), 0);
        view.diplomacy.relations[0].state = RelationState::Peace;
        view.diplomacy.relations.push(Relation {
            a: TeamId(2),
            b: TeamId(3),
            state: RelationState::War,
            effective_turn: 1,
            guarantee_until: 0,
            version: 1,
            security: None,
            war_id: Some(1),
            declarers: vec![TeamId(2)],
        });
        let mut busy = planner(&view, 7);
        busy.difficulty = AiDifficulty::Hard;
        assert!(busy.peace_army_site_risk(exposed) < peace);
    }
}
