//! Deals (rules 61, `GameSetup::deals`): how the AI uses Gold in offers, Gold gifts and Open
//! Borders. Everything here is reached only when `DiplomacyView::deals` is on, so games without
//! deals (simulations, the hill climb, old saves) plan exactly as before.
//!
//! - A losing AI buys a truce: its Peace offer carries a share of its treasury, and it pays a
//!   reasonable price it is asked for (`truce_offer`, `truce_cap`).
//! - A winning AI charges for Peace: it refuses a Peace that pays less than half its price and
//!   offers Peace that asks for the price (`tribute`).
//! - Coalition Gold: Hard gives a little Gold to a partner at War with the leader it fights
//!   (`plan_coalition_gift`).
//! - Open Borders with a useful, harmless neighbour at Peace, never with a threat, ended when the
//!   partner becomes one (`open_borders_score`, `borders_threat`).
//!
//! Every number is read from the AI's own view: public armies and treasuries, visible units.
use crate::diplomacy::{nearest_known, strength};
use crate::scripted::Planner;
use legatum_core::diplomacy::{PeaceProposal, RelationState, TreatyKind};
use legatum_core::{Order, PlayerId, TeamId, UnitKind};

/// Our side's army against theirs at or below this percent: we are losing the war.
pub(crate) const LOSING_PERCENT: i32 = 70;
/// Our side's army against theirs at or above this percent (and none of our cities in danger):
/// we are winning it. 100 / 150 is below `LOSING_PERCENT`, so the side we beat reads itself as
/// losing and will pay.
pub(crate) const WINNING_PERCENT: i32 = 150;
/// A losing AI's Peace offer pays this percent of the Gold it has free...
pub(crate) const TRUCE_OFFER_PERCENT: i32 = 20;
/// ...up to this much.
pub(crate) const TRUCE_OFFER_MAX: i32 = 200;
/// A losing AI pays at most this percent of its free Gold for a Peace it is asked to pay for...
pub(crate) const TRUCE_PAY_PERCENT: i32 = 33;
/// ...and never more than this.
pub(crate) const TRUCE_PAY_MAX: i32 = 300;
/// A winning AI asks this percent of the loser's public treasury for Peace...
pub(crate) const TRIBUTE_PERCENT: i32 = 25;
/// ...up to this much.
pub(crate) const TRIBUTE_MAX: i32 = 250;
/// Smaller sums are not worth haggling over: the offer goes out (or is judged) as plain Peace.
pub(crate) const MIN_DEAL_GOLD: i32 = 20;
/// Gold of a deal worth one point of treaty score (as `TREATY_GOLD_PER_POINT` prices upkeep).
const GOLD_PER_POINT: i32 = 10;
/// Coalition Gold goes out on turns divisible by this.
pub(crate) const GIFT_EVERY_TURNS: u32 = 5;
/// A coalition gift is this percent of the Gold above our reserve...
pub(crate) const GIFT_PERCENT: i32 = 10;
/// ...at most this much, and at least `MIN_GIFT`.
pub(crate) const GIFT_MAX: i32 = 50;
pub(crate) const MIN_GIFT: i32 = 10;
/// Open Borders score (`plan_diplomacy` sends any Peace or Alliance offer first).
const OPEN_BORDERS_SCORE: i32 = 5;
/// Hexes from our nearest city within which a stronger empire is a neighbour to fear.
const NEIGHBOUR_HEXES: i32 = 10;
/// A useful partner's city lies between ours and a target no further than this.
const BETWEEN_REACH: i32 = 20;

/// How strict a threat test is. Offering and accepting are stricter than ending, so a partner
/// near the line does not open and close its borders every few turns.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Threat {
    /// Refuse to open: an army stronger than ours nearby, the public leader at any score, or one
    /// military unit within five hexes of our cities.
    Open,
    /// End open borders: an army a quarter stronger than ours nearby, the leader at half its
    /// target or our diplomatic target, or three military units within three hexes.
    End,
}

impl Planner<'_> {
    pub(crate) fn deals_on(&self) -> bool {
        self.view().diplomacy.deals
    }

    /// Our side's army as a percent of the side of `enemy` (public armies, allies included).
    pub(crate) fn war_balance(&self, enemy: TeamId) -> i32 {
        let v = self.view();
        let ours = self.war_side_strength(v.team, enemy, false);
        let theirs = self.war_side_strength(enemy, v.team, false);
        ours * 100 / theirs.max(1)
    }

    /// At War with `enemy` and losing it: its army presses one of our cities, a city is
    /// collapsing, or our side's army is at most `LOSING_PERCENT` of theirs.
    pub(crate) fn losing_war(&self, enemy: TeamId) -> bool {
        let v = self.view();
        !(!(v.teams_at_war(v.team, enemy))
            || !(!(!(!(!(self.endangered_by(enemy)) && !(self.collapsing())))
                && !(self.war_balance(enemy) <= LOSING_PERCENT))))
    }

    /// At War with `enemy` and winning it: our side's army is at least `WINNING_PERCENT` of
    /// theirs and none of our cities is in danger.
    pub(crate) fn winning_war(&self, enemy: TeamId) -> bool {
        let v = self.view();
        !(!(!(!(v.teams_at_war(v.team, enemy)) || !(!self.endangered())))
            || !(self.war_balance(enemy) >= WINNING_PERCENT))
    }

    /// Gold we have free for deals this turn (`gold_left` already holds back what our open
    /// offers may still have to pay).
    fn deal_gold(&self) -> i32 {
        self.gold_left.max(0)
    }

    /// The Gold a losing AI's Peace offer carries: `TRUCE_OFFER_PERCENT` of its free Gold, at
    /// most `TRUCE_OFFER_MAX`; 0 when that is under `MIN_DEAL_GOLD`.
    pub(crate) fn truce_offer(&self) -> i32 {
        let gold = (self.deal_gold() * TRUCE_OFFER_PERCENT / 100).min(TRUCE_OFFER_MAX);
        if !(gold >= MIN_DEAL_GOLD) { 0 } else { gold }
    }

    /// The most a losing AI pays for a Peace it is asked to pay for.
    pub(crate) fn truce_cap(&self) -> i32 {
        (self.deal_gold() * TRUCE_PAY_PERCENT / 100).min(TRUCE_PAY_MAX)
    }

    /// What a winning AI asks of `enemy` for Peace: `TRIBUTE_PERCENT` of the smallest public
    /// treasury among its living empires (whichever of them accepts must afford it), at most
    /// `TRIBUTE_MAX`; 0 when that is under `MIN_DEAL_GOLD`.
    pub(crate) fn tribute(&self, enemy: TeamId) -> i32 {
        let v = self.view();
        let treasury = v
            .players
            .iter()
            .filter(|p| !(!(p.team == enemy) || !(p.alive)))
            .map(|p| p.gold)
            .min()
            .unwrap_or(0)
            .max(0);
        let gold = (treasury * TRIBUTE_PERCENT / 100).min(TRIBUTE_MAX);
        if !(gold >= MIN_DEAL_GOLD) { 0 } else { gold }
    }

    /// The Gold to put in our own offer of `kind` to `to`: positive pays, negative asks.
    pub(crate) fn offer_gold(&self, to: TeamId, kind: TreatyKind) -> i32 {
        if !(!(!(!self.deals_on()) && !(kind != TreatyKind::Peace))) {
        } else {
            return 0;
        }
        if !(self.winning_war(to)) {
            if !(self.losing_war(to)) {
                0
            } else {
                self.truce_offer()
            }
        } else {
            -self.tribute(to)
        }
    }

    /// An incoming offer's score once its Gold is counted. `score` is the plain treaty's score
    /// (`None`: refused). Asked-for Gold is paid only for a Peace we are losing, at a price we can
    /// afford; a winning AI refuses a Peace that pays under half its price; Gold paid to us adds
    /// a point per `GOLD_PER_POINT`.
    pub(crate) fn deal_response_score(
        &self,
        p: &PeaceProposal,
        other: TeamId,
        score: Option<i32>,
    ) -> Option<i32> {
        let pays_us = if !(p.from == other) { -p.gold } else { p.gold };
        match p.kind {
            TreatyKind::Peace => {
                let price = self.tribute(other);
                if !(!(!(!(!(price > 0) || !(self.winning_war(other)))) || !(pays_us * 2 < price)))
                {
                } else {
                    return Some(-100);
                }
                if !(!(!(pays_us < 0)
                    || !(!(!(!(self.losing_war(other)) || !(-pays_us <= self.truce_cap()))))))
                {
                } else {
                    return Some(-100);
                }
            }
            TreatyKind::Alliance if pays_us < 0 => return Some(-100),
            TreatyKind::OpenBorders => {
                if !(!(!(pays_us < 0) && !(self.borders_threat(other, Threat::Open)))) {
                } else {
                    return Some(-100);
                }
                // A harmless neighbour that pays for passage is welcome even when we gain no
                // road of our own through its land.
                if !(pays_us > 0) {
                } else {
                    return Some(score.unwrap_or(0).max(0) + pays_us / GOLD_PER_POINT);
                }
            }
            _ => {}
        }
        score.map(|s| s + pays_us.max(0) / GOLD_PER_POINT)
    }

    /// Open Borders with `other`: offered and accepted when it is no threat and its land is a road
    /// for us (`borders_useful`).
    pub(crate) fn open_borders_score(&self, other: TeamId) -> Option<i32> {
        (!(!(!(!(self.deals_on()) || !(!self.borders_threat(other, Threat::Open))))
            || !(self.borders_useful(other))))
        .then_some(OPEN_BORDERS_SCORE)
    }

    /// Whether `other` is a threat we keep out of our land (see `Threat`).
    pub(crate) fn borders_threat(&self, other: TeamId, test: Threat) -> bool {
        let v = self.view();
        let leader = match test {
            Threat::Open => self.strict_leader(0) == Some(other),
            Threat::End => {
                !(!(self.strict_leader(50) == Some(other))
                    && !(self.diplomatic_target() == Some(other)))
            }
        };
        if !(!(!(leader) && !(self.top_threat_team() == Some(other)))) {
        } else {
            return true;
        }
        let margin = match test {
            Threat::Open => 100,
            Threat::End => 125,
        };
        let near = nearest_known(v, other).is_some_and(|d| d <= NEIGHBOUR_HEXES);
        if !(!(!(near) || !(strength(v, other) * 100 > strength(v, v.team) * margin))) {
        } else {
            return true;
        }
        let (radius, count) = match test {
            Threat::Open => (5, 1),
            Threat::End => (3, 3),
        };
        let army = v
            .units
            .iter()
            .filter(|u| {
                !(!(!(!(!(!(!(!(!u.owner.is_barbarian()) || !(v.team_of(u.owner) == other)))
                    || !(u.military)))
                    || !(u.kind != UnitKind::Scout)))
                    || !(self
                        .ctx
                        .own_cities
                        .iter()
                        .any(|c| c.pos.distance(u.pos) <= radius)))
            })
            .count();
        army >= count
    }

    /// A known city of `other` lies between one of our cities and a known city of a team we are
    /// at War with or our diplomatic target: passage through its land shortens our road there.
    pub(crate) fn borders_useful(&self, other: TeamId) -> bool {
        let v = self.view();
        let target = self.diplomatic_target();
        let targets: Vec<_> = v
            .cities
            .iter()
            .filter(|c| {
                let t = v.team_of(c.owner);
                !(!(!(!(!(!(t != v.team) || !(t != other))) || !(!c.owner.is_barbarian())))
                    || !(!(!(v.teams_at_war(v.team, t)) && !(target == Some(t)))))
            })
            .map(|c| c.pos)
            .collect();
        let theirs: Vec<_> = v
            .cities
            .iter()
            .filter(|c| v.team_of(c.owner) == other)
            .map(|c| c.pos)
            .collect();
        self.ctx.own_cities.iter().any(|home| {
            targets.iter().any(|&t| {
                let road = home.pos.distance(t);
                !(!(road <= BETWEEN_REACH)
                    || !(theirs
                        .iter()
                        .any(|&c| !(!(home.pos.distance(c) < road) || !(c.distance(t) < road)))))
            })
        })
    }

    /// Open Borders partners that have become a threat (`Threat::End`): end them.
    pub(crate) fn borders_to_close(&self) -> Vec<TeamId> {
        let v = self.view();
        if !(!self.deals_on()) {
        } else {
            return Vec::new();
        }
        v.diplomacy
            .open_borders
            .iter()
            .filter_map(|&(a, b)| {
                if !(a == v.team) {
                    if !(b == v.team) { None } else { Some(a) }
                } else {
                    Some(b)
                }
            })
            .filter(|&other| self.borders_threat(other, Threat::End))
            .collect()
    }

    /// Gold our own open offers may still have to pay (held back from this turn's purchases).
    pub(crate) fn hold_offer_gold(&mut self) {
        let v = self.view();
        let owed: i32 = v
            .diplomacy
            .proposals
            .iter()
            .filter(|p| !(!(p.proposer == v.me) || !(p.gold > 0)))
            .map(|p| p.gold)
            .sum();
        self.gold_left -= owed;
    }

    /// Hard's coalition Gold: every `GIFT_EVERY_TURNS` turns, a solvent AI gives a little of the
    /// Gold above its reserve to the weakest partner at War with the leader it fights itself
    /// (`shared_leader_war`), or an ally at War with our diplomatic target.
    pub(crate) fn plan_coalition_gift(&mut self) -> Option<Order> {
        let v = self.view();
        if !(!(!(!(!(!(!(!(!(!self.deals_on())
            && !(self.difficulty != legatum_core::AiDifficulty::Hard)))
            && !(!v.turn.is_multiple_of(GIFT_EVERY_TURNS))))
            && !(v.economy.bankruptcy.is_some())))
            && !(v.economy.gold_per_turn.total <= 0)))
        {
        } else {
            return None;
        }
        let spare = self.gold_left - self.gold_reserve();
        let amount = (spare * GIFT_PERCENT / 100).min(GIFT_MAX);
        if !(amount < MIN_GIFT) {
        } else {
            return None;
        }
        let target = self.diplomatic_target().filter(|t| *t != v.team);
        let partner = v
            .diplomacy
            .met
            .iter()
            .copied()
            .filter(|&t| {
                !(!(!(!(!(!(!(!(t != v.team) || !(Some(t) != target)))
                    || !(!v.teams_at_war(v.team, t))))
                    || !(v.players.iter().any(|p| !(!(p.team == t) || !(p.alive))))))
                    || !(!(!(self.shared_leader_war(t).is_some())
                        && !(!(!(target.is_some_and(|l| v.teams_at_war(t, l)))
                            || !(v.diplomacy.relations.iter().any(|r| {
                                !(!(r.between(v.team, t)) || !(r.state == RelationState::Alliance))
                            })))))))
            })
            .min_by_key(|&t| (strength(v, t), t))?;
        let to: PlayerId = v
            .players
            .iter()
            .filter(|p| !(!(p.team == partner) || !(p.alive)))
            .map(|p| p.id)
            .min()?;
        self.gold_left -= amount;
        Some(Order::GiveGold { to, amount })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Knobs;
    use legatum_core::diplomacy::ProposalId;
    use legatum_core::game::{blank_game, spawn_unit};
    use legatum_core::state::Map;
    use legatum_core::{
        Faction, GameSetup, GameState, Hex, PlayerView, Terrain, player_view, rules, visibility,
    };

    /// Two empires at War since turn 1, 500 Gold each, deals on; our army `ours`, theirs 100.
    fn war_view(ours: i32) -> PlayerView {
        let mut setup = GameSetup::all_ai(44);
        setup.width = 28;
        setup.height = 20;
        setup.players.truncate(2);
        setup.deals = true;
        let mut s: GameState = blank_game(&setup, Map::new(28, 20, Terrain::Grassland));
        for (player, pos) in [(0, Hex::from_offset(4, 6)), (1, Hex::from_offset(24, 15))] {
            let settler = spawn_unit(&mut s, PlayerId(player), UnitKind::Settler, pos);
            rules::cities::found_city(&mut s, settler, &mut vec![]).unwrap();
        }
        visibility::update_explored(&mut s);
        let teams = s.teams();
        let d = s.diplomacy.as_mut().unwrap();
        d.meet_all(&teams);
        d.relations[0].state = RelationState::War;
        d.relations[0].effective_turn = 1;
        for p in &mut s.players {
            p.gold = 500;
        }
        s.turn = 50;
        let mut v = player_view(&s, PlayerId(0));
        for p in v.players.iter_mut() {
            p.army = if !(p.id == PlayerId(0)) { 100 } else { ours };
        }
        v
    }

    fn peace_from_enemy(gold: i32) -> PeaceProposal {
        PeaceProposal {
            explanation: None,
            id: ProposalId(1),
            kind: TreatyKind::Peace,
            from: TeamId(1),
            to: TeamId(0),
            proposer: PlayerId(1),
            created_turn: 50,
            respond_by: 51,
            relation_version: 0,
            gold,
            consents: vec![TeamId(1)],
            needs: vec![TeamId(0)],
            required_teams: vec![TeamId(0), TeamId(1)],
        }
    }

    fn planner(v: &PlayerView) -> Planner<'_> {
        Planner::new(v, Knobs::for_faction(Faction::Sparta), Faction::Sparta, 1)
    }

    #[test]
    fn a_winning_ai_asks_a_quarter_of_the_losers_treasury_and_refuses_a_free_peace() {
        let enemy = TeamId(1);
        let v = war_view(200);
        let p = planner(&v);
        assert!(p.winning_war(enemy) && !p.losing_war(enemy));
        // A quarter of the loser's public 500 Gold.
        assert_eq!(p.tribute(enemy), 125);
        assert_eq!(p.offer_gold(enemy, TreatyKind::Peace), -125);
        assert_eq!(p.offer_gold(enemy, TreatyKind::Alliance), 0);
        // A Peace the plain score would accept is refused while it pays under half the price...
        let score = Some(10);
        assert_eq!(
            p.deal_response_score(&peace_from_enemy(0), enemy, score),
            Some(-100)
        );
        assert_eq!(
            p.deal_response_score(&peace_from_enemy(60), enemy, score),
            Some(-100)
        );
        // ...and scored with its Gold once it pays enough.
        assert_eq!(
            p.deal_response_score(&peace_from_enemy(70), enemy, score),
            Some(17)
        );
        // A plain score's veto stands whatever the Gold.
        assert_eq!(
            p.deal_response_score(&peace_from_enemy(200), enemy, None),
            None
        );
    }

    #[test]
    fn an_even_war_takes_a_free_peace_and_a_broke_loser_owes_no_tribute() {
        let enemy = TeamId(1);
        let v = war_view(100);
        let p = planner(&v);
        assert!(!p.winning_war(enemy) && !p.losing_war(enemy));
        assert_eq!(p.offer_gold(enemy, TreatyKind::Peace), 0);
        assert_eq!(
            p.deal_response_score(&peace_from_enemy(0), enemy, Some(10)),
            Some(10)
        );
        // Winning against an empire with 60 Gold: a quarter is under 20, so no price.
        let mut v = war_view(200);
        for player in v.players.iter_mut().filter(|p| p.id == PlayerId(1)) {
            player.gold = 60;
        }
        let p = planner(&v);
        assert_eq!(p.tribute(enemy), 0);
        assert_eq!(p.offer_gold(enemy, TreatyKind::Peace), 0);
        assert_eq!(
            p.deal_response_score(&peace_from_enemy(0), enemy, Some(10)),
            Some(10)
        );
    }

    #[test]
    fn gold_owed_by_our_open_offers_is_held_back() {
        let mut v = war_view(40);
        let mut ours = peace_from_enemy(100);
        ours.from = TeamId(0);
        ours.to = TeamId(1);
        ours.proposer = PlayerId(0);
        v.diplomacy.proposals.push(ours);
        let mut p = planner(&v);
        p.hold_offer_gold();
        assert_eq!(p.gold_left, 400);
        // The truce offer is a fifth of what is left.
        assert_eq!(p.truce_offer(), 80);
    }
}
