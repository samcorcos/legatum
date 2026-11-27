# Diplomacy

Part of the Legatum rules reference (index: `index.md`). Answers: How do Peace, Alliance and War, calls to arms, offers, leaving treaties, first contact, surprise wars and coalitions work?
Moved verbatim from `game-rules.md` on 2026-11-22, in the original order; the original section headings are kept inside.

## 5. Diplomacy

**Peace, Alliance or War** (since 2026-10-03, user decision; Pacts, minimum terms, exit notice
periods, trust and call commitments were removed; Call to Arms was removed 2026-10-18 for shared wars, below). `RelationState`: `Peace | War | Alliance`
(`diplomacy.rs`); all start at Peace. `TreatyKind`: `Peace | Alliance | OpenBorders` (Open Borders only with deals on, see "Deals" below).

| | Cost (Gold) | Notes |
|---|---|---|
| Alliance upkeep | 1 per owned city per turn for each empire on both teams (minimum 1); none on a turn either ally is at War with a team close to winning (rules 50) | `TreatyKind::upkeep_for_cities` in `economy::treaty_upkeep`; a "Treaty upkeep: Alliance with X" line in the Gold breakdown, and a waiver line beside it |
| Breaking inside the protected window | 120 | first 10 turns (`ALLIANCE_PROTECTED_TURNS`, `leave_cost`) |
| Peace (ending a war) | free | 10 turns of guaranteed Peace (`PEACE_TRUCE_TURNS`); if Peace begins on turn T, no War may take effect through T+9, and War is possible from T+10. |

The truce belongs only to its two signers, so a war with an ally continues. It blocks direct declarations (including orders queued on the last protected turn), surprise wars, defensive Alliance joins, accepted calls to arms, and an Alliance signing that would join that protected pair in war (`diplomacy::check_order`, `join_blocked`, `call_join_blocked`, `check_alliance`). A surprise declaration's defensive joins are checked against the current resolution, not the next boundary. Signing or leaving an Alliance with the former enemy does not erase the truce's recorded deadline. The UI displays the end turn in the empire row and Declare War refusal; the AI prices the ten-turn loss of its best available war plan when considering Peace (`crates/ai/src/plan.rs::plan_candidates`).

Alliances cost per turn only (designer decision 2026-10-31). Each empire on both teams pays the per-city upkeep after activation; the 120 Gold early-exit cost remains. Offers and acceptance require no treasury balance, and `DiplomaticAction.upkeep` quotes the ongoing cost (`diplomacy.rs::check_order,view,resolve_boundary`; `economy.rs::treaty_upkeep`).

**Alliance upkeep waiver (Sam, 2026-11-22, via the Fable review; rules 50).** An Alliance pays no upkeep, for both allies, on any turn in which either allied team is at War with a third team that is **close to winning**: alive and on a victory hold (`GameState::vp_holds`) or within `scoring::NEAR_VICTORY_VP` (2) VP of its target (`scoring::near_victory`; the same line as the app's public near-victory warning, `celebrate/alarms.ts` `victoryNear`). `diplomacy::upkeep_waived_by(state, a, b)` names the lowest such team id. It is judged when Gold is paid (the income snapshot at the start of `process_cities`), with VP counted live and holds as of the last turn end. The Gold breakdown keeps the "Treaty upkeep: Alliance with X" line and adds "Treaty upkeep: Alliance with X (free while at War with Y, close to winning)" for the same amount back (`economy::treaty_upkeep`), so nothing is hidden. `DiplomaticAction.upkeep` still quotes the ordinary price. Hard's `diplomacy.rs::treaty_cost` prices what the Alliance really costs now: `treaty_upkeep` is 0 while the waiver applies, read from public data by `PlayerView::alliance_upkeep_waived_by(a, b)` (standings, holds and public wars; mirrors the core rule).

**Alliances from turn 1 (2026-11-22; the app stopped setting it 2026-11-26, `setup.md`):** an older save or tape whose setup has `ai_alliances` on starts its AI-only teams as three Alliances of two (`DiplomacyState::start_alliance`; see `setup.md`). They have met, run the usual protected window from turn 1 and pay upkeep like any Alliance; they are not teams, so VP is not shared and either partner can leave or break the Alliance.

**An Alliance** means: the two teams cannot attack each other (not at War), share vision, may pass through each other's territory (`GameState::passage_allowed`), and defend each other or answer calls to arms (next section). Allied city passage is transit only; see §1 Movement. A coalition does not change the Alliance price.

### Defenders join at once; an ally's own war is a call to arms (user, 2026-10-23)

"Allies auto join defensive wars but they get prompted for whether they want to join offensive wars. It should be possible to ally someone, they declare war on someone, and you say you don't want to join, and you still stay allied." This replaces "allies fight their wars together" (2026-10-18, `share_wars`) for every new game. Guide sentence: **"Allies defend each other, and join wars by choice."**

- **Calls are the only treaty rule.** `DiplomacyState::calls: Vec<WarCall>` is present in every game; there are no free Alliances or automatic offensive wars (`diplomacy.rs::DiplomacyState`, `new`, `resolve_boundary`, `signing_fee`). `start-met` still makes teams start in contact for treaty fixtures (`game.rs::blank_game`).
- **Ally** means a direct Alliance (`DiplomacyState::partners`). Teams joined through chains are still one side (`side_of`) for `declare_conflict` and `alliance_conflict`, but a Peace binds only its two named teams; nothing joins or is called through a chain.
- **Defence is automatic, in the same turn as the declaration.** When team D declares War on A, every direct ally X of A that is alive, is not D, is not on D's side through any Alliance and is not already at War with D is at War with D at once: a surprise declaration brings the defenders in at the start of resolution, before ranged fire; a war on an ally (`breach`) takes effect next turn for all. The joined pair copies the war's `war_id` and has `declarers = [D]`, and emits `Event::WarJoined{team: X, ally: A}` (`diplomacy::start_war_effects`, `DiplomacyState::declare_wars`). No cascade: X's own allies do not join. A team that is allied to both D and A is never dragged in: it gets the call below, and Join is blocked for it. `DiplomaticAction.new_wars` on a declaration lists the defenders (team = X, enemy = D); a guaranteed Peace between X and D refuses the declaration (`join_blocked`: "X would have to join the war on D, but their peace is guaranteed through turn N").
- **Offensive wars are a call to arms.** Every direct ally Y of D that is alive, is not the target and is not already at War with it gets a `WarCall { id, team: Y, ally: D, target: A, created_turn, respond_by, alliance_version, war_version }` (`raise_call`, `Event::CallToArms`, visible to the two teams; the id comes from the proposal counter). **Timing:** the call is created at the boundary that begins D's war and first appears in the planning turn after the declaration (`created_turn`); it stays open `CALL_WINDOW_TURNS` = 2 planning turns (`respond_by = created_turn + 1`). The answer is an order, `Order::RespondToProposal { proposal: <call id>, accept }` (Join = accept, Decline), one diplomatic order per other team per turn like any (keyed to the calling ally). It resolves at the turn boundary like a declaration that does not start at once: **Join puts Y at War with A from the next turn** (`declarers = [Y]`, the ally's `war_id`, `Event::CallAnswered{joined: true}` and `WarJoined{team: Y, ally: D}`), and A's allies then defend against Y at once (`start_war_effects` without calls: joining raises no calls of its own). **Decline** keeps the Alliance and the Peace with A: no betrayal record, no Gold, no cooldown, nothing remembered (`CallAnswered{joined: false}`); an unanswered call lapses at the end of its window and counts as a decline (`CallLapsed`). A call also lapses when a team is eliminated, when the Alliance with the caller ended or changed (`alliance_version`), when the caller's war ended or changed (`war_version`) or when Y is already at War with A (`call_void_reason`). Checks (`check_call_answer`): only the called team, inside the window, call not void; Join is refused when A is also Y's ally ("leave that Alliance first, or decline"), a Peace guarantee with A holds, or a defender A would bring in holds guaranteed Peace with Y (`call_join_blocked`, `join_blocked`), and a closed country (Sakoku) cannot Join (`closed_refusal`); Decline is always legal. Thus accepting a call respects every truce the resulting wars would break. `DiplomaticAction.calls` on a declaration lists the allies who would be asked (team = Y, enemy = A).
- **Signing during a war** (`DiplomacyState::alliance_wars`, `alliance_effects`). For each war a partner fights: where the partner is the **defender** (`defender_in`: its enemy declared it, the partner did not) the new ally is at War with that enemy at once, same `war_id` (`new_wars`); where the partner declared or joined the war, the new ally gets a call (`calls`), both ways. Refusals as before: the two are already one side, the Alliance would make an empire fight its own ally (`alliance_conflict`), or a defender join would break a guaranteed Peace (`join_blocked`).
- **Peace is between two teams only** (`DiplomacyState::peace_parties`, `required_teams`). The proposer and recipient must agree (`awaiting`, `PeaceProposal.consents`); `make_peace` ends only their pairwise War and gives that pair a 10-turn guarantee. Both teams keep their Alliances, and every war either team or its allies has with anyone else continues. A separate Peace is no betrayal. A permanent teammate shares the same TeamId and is bound with its team. A later call to arms against the former enemy cannot be joined during this pair's truce; another pair's truce is independent.
- **Peace UI:** each incoming Peace offer opens a full-screen peace-talks plate once per seat, after the turn's held moments. The Inbox keeps the offer and can reopen the plate. Accept and refuse queue the existing reply with Undo; Decide later leaves it pending. Offer Peace opens a terms plate with only the supported one-time Gold term, subject to the treasury forecast and 999 cap. The core refuses a new proposal while any offer between the pair is open, so there is no counter-offer action.
- **UI:** a call is a decision in Diplomacy's Inbox tab: `CallCard` in `app/src/ui/DiplomacyPanel.tsx` (headline "X, your ally, is at War with Y and calls you to arms", Join the war on Y as the one primary action, Decline beside it, "Declining keeps your Alliance and your Peace", a For-you list with the War it starts, the defenders it brings against you and the note that declining is no betrayal). A call is a pending decision (`turnFlow.pendingDecisions`, "Answer call to arms" in the seat bar) answered with A and D; answering is queued and confirmed with Undo like an offer. The report says "came to the defence of", "joined X's war", "called to arms", "declined" and why a call ended (`format.ts`, `CallToArms`/`CallAnswered`/`CallLapsed`), and an attacked ally raises the "Your ally is attacked" alarm. Covered by WASM behavior checks in `app/src/test/diplomacy.wasm.test.ts`.
- **AI:** answers each call by value, `Planner::call_value` = `join_war_value(target)` (the war's price: the leader or a team counting down +, the odds, our other wars, danger at home, a hopeless or unreachable war, the army at our gates) + `CALL_SHARED_ENEMY_BONUS` (10) per team of the target's side it already fights (up to 3); Join at 0 or more, Decline otherwise, and never when another answer to the same ally uses the turn's one order. Alliance scoring counts the wars a new ally brings in at once (`new_wars`) and notes the calls (we may decline); it no longer prices wars we would be asked to join. The human-seat proxies join a call when the target is at 80% victory progress or a campaign against it is affordable and reachable (`proxy_v4.rs`).
- **Kept as they were:** leaving (protected window, Gold, betrayal records), war on an ally as a betrayal, the permanent human team (same `TeamId`, never at War, shares everything), first contact, Sakoku (a closed country cannot sign an Alliance, so it never has allies to defend).
- **Tests:** `crates/core/tests/diplomacy.rs` (defenders join at once, the call and its answers, lapse, void calls, signing in wartime, guarantees, Peace without the decliner, chains), `crates/ai/tests/proxy_alliances.rs`, WASM behavior checks in `app/src/test/diplomacy.wasm.test.ts`, the card in `app/src/ui/AlliesFight.test.tsx`.

**Offers end when answered or overtaken (2026-10-17, playtest 8.9.2 T20 loop).** An offer (`PeaceProposal`, any kind) records
the pair's `relation_version`; `resolve_boundary` drops it when declined (`ProposalRejected`, 5-turn cooldown), withdrawn,
accepted, expired, or when the pair's relation changed this turn: `ProposalInvalidated` with reason `diplomacy::WAR_ENDS_OFFERS`
("War was declared") when the pair is now at War (a surprise war at the start of resolution or a boundary declaration), else
"Relationship changed" (an Alliance left, bought peace). So **declaring War on an empire withdraws every pending offer between
the two**, and a declined offer never survives the turn. Core takes **one diplomatic order per other team per turn**
(`validate_orders`: "One diplomatic order per other team per turn"; `resolve_boundary` keeps the first by PlayerId), so a
decline plus a war toward the same empire is invalid: the war alone settles the offer. The client mirrors this: `queueDiplomacy`
replaces the order toward that team, except that declining an offer from a team the plan already declares War on, leaves or pays
tribute to is a no-op (`endsOffers`), and `turnFlow.pendingDecisions` counts an offer as settled by a reply or by such an order
toward its sender (`offerSettledBy`), so it is no longer "Respond to offer" and its Inbox card reads "Ended by your War on X".

**A closed country (Japan's Sakoku vow, §8):** a closed empire cannot declare war, and no Alliance can be proposed or signed by a team with a closed member or with such a team (`diplomacy::closed_refusal` in `check_order`; the reason reads "X has closed the country (Sakoku): ..."), Peace is unaffected, and `passage_allowed` lets nobody but the closed empire's own team into its land at Peace (no Alliance passage or open road).

**Leaving** (`BreakTreaty`) takes effect next turn. Inside the protected window it costs the breach
Gold and is a **betrayal**; after it, leaving is free. **War on an ally** needs `DeclareWar{breach:
true}`, takes effect next turn, and is always a betrayal (plus the Gold inside the window).
An existing unit inside former allied land is not expelled when the Alliance ends (`rules/mod.rs` keeps
its position and clears fortification). At Peace in closed foreign land it cannot heal, fortify,
shoot, or attack from there; it may leave. A later declaration makes that land open as wartime
territory, so the staged unit may attack (`rules/movement.rs`, `rules/battle.rs`,
`core/tests/diplomacy.rs::closed_borders_require_progress_to_exit_and_forbid_healing_fortifying_or_attacking`).
This passage and the 10-turn free exit are costs in AI Alliance scoring.

A betrayal is one visible record, `Betrayal{by,against,turn,until = turn + 20}`
(`BETRAYAL_TURNS`, `DiplomacyState::betrayals`, public in `DiplomacyView.betrayals`, event
`AllianceBetrayed`): while it lasts the betrayer cannot propose an Alliance to its victim ("X won't
ally with Y until turn N: betrayed on turn M"); the victim may still offer one. There is no hidden
trust stat.
When both allies declare a breach in the same turn, each pays the protected-window cost and gets
their own betrayal record; the shared Alliance ends once (`diplomacy::resolve_boundary`).


**First contact (2026-10-03)**: an Alliance can be neither offered nor accepted until the two
teams have **met**: one team's own units or cities have seen a unit, city or owned tile of the
other. Contact is mutual and permanent; only native sight counts (an ally's borrowed sight
introduces nobody) and barbarians never count. Peace and War don't
need it. Recorded at game start and at the end of every resolution
(`crates/core/src/visibility.rs::contact_pairs,update_explored`) into `DiplomacyState::contacts`;
checked in `crates/core/src/diplomacy.rs::check_order` (ProposeTreaty and accepting
RespondToProposal, error `diplomacy::NOT_MET`) via `DiplomacyState::met`; views list met teams in
`DiplomacyView::met`. Saves from before the rule have no record (`contacts: None`) and treat every
pair as met; tapes before version 5 replay without it (`wasm/src/tape.rs::initial_state`).
 The Diplomacy panel hides an empire not met yet, as the rivals table does (`offerStatus.ts::hiddenEmpire`: unless the two are at War or a declaration on it is queued): no row, name or actions, one line "N empires not met yet. Explore to meet them: offers need contact first." (2026-11-25). The "You haven't met X yet" line remains only for an unmet empire shown because of a War. The turn a team first meets another, each of its human seats gets the first-contact plate (`celebrate/MeetingTakeover.tsx`, from the growth of `DiplomacyView.met` between two views in `moments.ts::meetingsForTurn`; no core event, so loaded games and replays never show it). Continue (or Return) closes it and takes the map to where they were met (their nearest seen city, else their unit); Esc closes it without moving the map.

The Diplomacy action and an attack that requires a declaration open the same full-screen War plate. It lists the public army points, defenders, calls to arms, Alliance costs and any refusal; the attack entry retains its attack order when confirmed. Keep the Peace, Return and Escape cancel, and a refused declaration has no confirm button.

**War**: declaring against a team at plain Peace is a **surprise war** — effective immediately,
same turn, before ranged fire/movement; a unit can be ordered into enemy territory with the
declaration in one submission (G20). **At Peace a unit cannot enter the other empire's territory**
(except with Open Borders, deals only, below; and except Mongols's Scouts and civilians, §8); allies may, but not into a closed country (§8, Sakoku). Only an Alliance shares vision.
The AI uses that same opening turn for ranged attacks and eligible melee attacks. Its combat
forecast projects valid surprise declarations before planning the assault
(`crates/ai/src/scripted.rs::attack_view`, `city_assault.rs::city_assault_succeeds`,
`military.rs::press_strike`). The aircraft planner uses the projected view after diplomacy
orders are chosen (`scripted.rs::projected_attack_view`, `air.rs::air_orders`). Decision gene
`declare_and_strike` defaults to 1; setting it to 0
restores the former one-turn attack delay for comparison. A declaration that breaches an Alliance
still takes effect next turn and is never projected as a surprise war.

**Coalitions** (engine and AI only; the player UI no longer shows the word, 2026-11-11: it says plain facts instead, see below) are a derived warning, not a mechanic: flags when one team's VP pressure (and
optionally city count) pulls far enough ahead, and lists Alliances signed against it
(`coalition_target`). A team holding its VP target (counting down, §6) is always the target: it
outranks every non-holder even on equal progress; of two holders the one with fewer turns left
(then higher pressure) is, and holders tied on both leave no target (`coalition::select`;
`pressure_view` adds `HOLD_PRESSURE` 1000 minus turns left). The view still carries `DiplomacyView::coalition` and `SecurityTreaty::coalition_target`, which nothing in the app reads. No coalition order exists, and the target changes no Alliance price (`diplomacy/coalition.rs`). Separately, on Hard,
`Planner::containment_target` can name a conqueror from public conquest evidence (captures,
eliminations, wars, armies at the border, momentum) well before VP or city-count would trigger a
formal coalition, and that target feeds the same Alliance-forming logic below
(`ai::diplomacy::containment_target`).

**AI** (`crates/ai/src/diplomacy.rs`): `peace_pair_scope` (default 1, all difficulties) prices the named enemy's danger rather than danger from every war, gives a small value to peeling one member off a hostile Alliance while the ally's war continues, and keeps `plan.rs::peace_would_be_broken` pair-specific. Alliances are value-based: score = −10 + 25 shared enemy +
25 shared coalition target + 15 when losing a war + the army it keeps off us (`alliance_security`:
their public army against ours, ×reach by distance between nearest known cities or units: ≤6 hexes
100%, ≤10 60%, ≤14 30%, beyond 0) − the wars it forecloses (`alliance_opportunity`: aggression and
army edge, same reach) − 80 if they are the leader − upkeep (`treaty_cost`: 10 turns of upkeep, one point per 10 Gold, `TREATY_GOLD_PER_POINT`; none in an old save) − 6 per Alliance already
held, accepted this turn or offered (`TREATY_HELD_PENALTY`, `TreatyBudget`) − 60 per team the Alliance adds to our bloc beyond a pair
(gene `bloc_size_penalty`, `bloc_size_cost`: free Alliances had produced blocs of 5 to 6 of 8 empires) + the wars a new ally brings in at once
(`alliance_war_value` over `DiplomaticAction.new_wars`: a war we would join costs `join_war_value`; an ally who would join our war is worth
`10 + min(20, edge/5)` (+10 endangered), scaled by how far it can reach the enemy, `helper_reach_percent`: all within 8 hexes, none past 16).
Offers to us are answered best first against the budget earlier acceptances leave. Refused when the partner is at
≥60% victory progress, has an active betrayal against us, or the treasury cannot cover
20 turns of any deficit the upkeep leaves (`TreatyCost::affordable`). **Against a shared
leader (Hard, gene `coalition_alliance`, 2026-10-07, playtest 8.4)**: when both teams are at war
with a team counting down, with our diplomatic target, or with the strict public VP leader at 50%
or more (`diplomacy.rs::shared_leader_war`), the 20-turn treasury test is relaxed for a shared leader (a bankrupt treasury still refuses), the score gains
`leader_war_value` ((leader progress − 40)/2, 0..30), and a partner past 60% is still accepted while
the leader is 10 points further ahead and the partner under 70%. The AI leaves an Alliance
only when leaving is free: partner ≥70% progress, a growing aggressor, a lapsed coalition reason,
or a forecast bankruptcy. War on an ally is reserved for a coalition leader at ≥90%. An AI waits
`ALLIANCE_ACQUAINTANCE_TURNS` (15) before offering; human recipients need no signing treasury (`crates/ai/src/diplomacy.rs::alliance_acquainted,treaty_cost`). **Calls to arms (2026-10-23):** an AI answers each call by
`Planner::call_value` (§5, "Defenders join at once"): `join_war_value(target)` plus 10 per team of the target's side it already fights. `join_war_value` also charges the front the war
would open at home (`inherited_front_cost`: up to 40 for the share of the army near our cities that would be theirs, and 20 or more more when a campaign against someone else is under way). The shared settlement scorer discounts a colony's
site value by 40% per travel turn after the first city, up from 10%, so both UI advice and AI
value reaching a city before rival borders close (`core::settlement::analyze_sites`,
`ai::civilians::choose_site`).

## Deals (rules 61, behind GameSetup::deals)

Sam approved the first slice of "more ways to deal with other empires than war or peace" on 2026-11-25. Everything here
needs `GameSetup::deals` (serde default false, omitted when false); off, each deal is refused with
`diplomacy::DEALS_OFF` ("Deals are off in this game") and nothing else changes: simulations, the hill climb, old saves,
tapes and fixtures serialize and resolve exactly as before. The app turns it on for new games. `DiplomacyView::deals`
says whether it is on.

- **Gold in offers.** `Order::ProposeTreaty { to, kind, gold }` and `PeaceProposal::gold` (both omitted when 0) apply to
  Peace, Alliance and Open Borders offers. Positive: the proposing empire pays that much Gold to the accepting empire;
  negative: the accepting empire pays |gold| to the proposer. At most 999 either way (`MAX_DEAL_GOLD`). Checks
  (`check_order`): the proposer must afford a positive amount now (`afford`, "This offer costs N Gold; ..."), and the
  accepting player must afford an asked amount now ("Accepting this offer costs N Gold; ..."). At resolution
  (`resolve_boundary`, after the turn's purchases), when the treaty would take effect, the payer must still have the
  Gold; if not, the offer ends with `ProposalInvalidated` reason "<Faction> could not pay N Gold" and nothing is signed.
  Otherwise the Gold moves and `Event::GoldGiven { from, to, amount }` follows `ProposalAccepted`. A Peace pays only when
  the Peace is made. `DiplomaticAction::cost` quotes the Gold the ordering empire would pay (an accept of an offer that
  asks for Gold). Deal Gold moves after income, so it is not part of this turn's Swiss interest.
- **Open Borders** (`TreatyKind::OpenBorders`, no upkeep, no breach cost). Offered and accepted only between two teams
  that have met, are at Peace (not War, not Alliance) and don't already have them; never with a closed country
  (Sakoku, `closed_refusal`: "a closed country can't open its borders"; `vow_conflict` likewise). State:
  `DiplomacyState::open_borders` (team pairs, lower id first; omitted when empty); public in
  `DiplomacyView::open_borders`. Accepting emits `ProposalAccepted` (then `GoldGiven` if any) and
  `Event::BordersOpened { a, b, effective_turn }`. Effect: `GameState::passage_allowed` (and its mirror
  `PlayerView::passage_allowed`) let a unit enter land of a team it has Open Borders with; never a closed country, never
  barbarians. No vision, no war duties.
- **Open Borders end** (`Event::BordersClosed { a, b, effective_turn, reason }`, public) when either side orders
  `BreakTreaty { with, agreement: Agreement::OpenBorders }` (free, no betrayal, next turn; "No Open Borders to end"
  otherwise), when the two go to War by any path (surprise war, boundary declaration, defenders joining, calls answered)
  or sign an Alliance (which includes passage), or when a team is eliminated (`close_lapsed_borders`, run after
  `declare_surprise_wars` and at the end of `resolve_boundary`). Units inside stay, like units left in former allied
  land (not expelled; closed-land limits apply). Closing borders does not change the relation's `version`.
- **Gifts:** see `turn-and-orders.md` "Co-op Gold gifts": with deals on, Gold may go to any met empire not at War.
- The one-diplomatic-order-per-other-team-per-turn rule holds for all of it.
- **View:** with deals on, `DiplomacyView::actions` adds, per other living team, `ProposeTreaty{kind: OpenBorders,
  gold: 0}` and `BreakTreaty{agreement: OpenBorders}` with their `blocked_reason`; Gold amounts are the client's to set.
- **UI** (`app/src/ui/DiplomacyPanel.tsx`, `app/src/deals.ts`, shown only when `DiplomacyView.deals`): an empire's
  Alliance and Open Borders offers share one "Gold with your offer" field; the Peace terms plate uses the same control (`DealGold`: No Gold, You pay, You ask, and the shared `GoldStepper` of
  10 up to 999), with the forecast "You pay 40 Gold when they accept. You have 120 Gold." from the treasury less the Gold
  queued orders already promise (`deals.ts::promisedGold`: gifts, offers we pay for, accepting an offer that asks us for
  Gold); paying more than that disables the offers with "You have N Gold, so offer at most N." in place. "You pay" puts
  a red cost chip on each offer, "You ask" a green "+N if accepted" chip. With deals on an empire's detail runs: the
  Gold field, the offers, Give Gold (`RivalGift`, a met empire not at War, a gift to the team's first living member,
  same `GoldStepper`), Close borders or Leave the Alliance, then Declare War below a rule. Offer Open Borders shows at
  Peace while closed ("If they accept, it starts on turn N."), Close borders while open. A queued action keeps its
  button pressed with a "Queued: Offer Open Borders. You ask 40 Gold. Undo" line under it, as gifts do. An incoming
  offer's headline carries its Gold ("Switzerland asks 40 Gold for Peace", "… offers Peace and 40 Gold") and its
  For-you terms say "You pay 40 Gold." or "You get 40 Gold."; the turn button and the advisor name it by kind
  (`turnFlow.ts::offerDecisionLabel`: "Open Borders with X: your units may enter each other’s land · asks 40 Gold ·
  respond by turn N"). Open Borders are drawn as emblem plus word, like Peace, on every surface (panel row, rivals
  table, report lines): `TreatyEmblem kind="OpenBorders"` is Peace's ring with a two-way arrow in place of the star.
  The report says "Open Borders with X begin.", "X closed the borders." (or "Open Borders with X end: War."), "You
  gave Sparta 40 Gold for Open Borders" (a `GoldGiven` right after a `ProposalAccepted` with Gold), and an offer line
  adds "they ask 40 Gold". App passage (`diplomacy.ts::canEnter`, used by routes, reach and the unit card) honours
  `open_borders`; cities stay closed. Closing borders ends no pending offer (`endsOffers`), and the same treaty with
  other Gold counts as the same offer (`sameDiplomaticAction`). New local games and online rooms set `deals: true`
  (`lobby.ts`, `staging.ts::configFromLobby`, `StagingConfig::deals` in `crates/wasm/src/host/staging.rs`); a loaded save
  keeps its own setup. Polish from live play (2026-11-25, all from the view and the session's turn reports, no core
  state; `app/src/offerStatus.ts`): while an offer between the pair is open, the empire's other offers read, where
  they would stand (above Give Gold), "Not now: Offer Alliance, Offer Open Borders. Waiting for Athens’ answer to your
  Open Borders offer, due turn 6." (or "Athens’ Peace offer waits for your answer in the Inbox, due turn N.") in place
  of core's "This pair already has an outstanding proposal"; our offer they rejected, let lapse or that was
  invalidated shows on their row for 5 turns ("Rejected your Open Borders offer · turn 5", "Didn’t answer your …",
  "Your Peace offer ended: War was declared · …"; `recentOfferOutcomes` over `reportHistory`, so only this session's
  last 20 turns) until the same offer is open again; with a diplomatic order toward an empire queued, each other
  action button says "Replaces your Open Borders offer" under it (`replacesText`). A treaty taking effect for the
  viewer's team (`ProposalAccepted` naming it, then any `GoldGiven`) plays a moment card (`moments.ts::treatyMoment`;
  no kicker, the title "Open Borders with Athens" names the event): our crest, the treaty emblem and their crest, the
  one-line rule, the deal Gold in green "+40 Gold from Athens" or red "−40 Gold to Athens" ("Your teammate: " first
  when it was the teammate's) with the treasury counter tick. Peace and Open Borders are the small notable card; an
  Alliance is the framed major card and adds its upkeep in red, "Upkeep −3 Gold a turn" (`allianceTerms`: one per viewer city,
  minimum one, as Diplomacy prices it); its sound is the `vpGained` feel call voiced by the card's tier (`sting.notable`, `sting.major` for an Alliance). Tour:
  `app/e2e/tour/deals.spec.ts`.
- **AI** (`crates/ai/src/deals.rs`, wired into `diplomacy.rs::plan_diplomacy`; every scripted difficulty unless noted,
  only when `DiplomacyView::deals` is on, so games without deals plan exactly as before). "Losing" a war: an enemy army
  presses one of our cities (`endangered_by`), a city is collapsing, or our side's public army is at most 70% of theirs;
  "winning": at least 150% and none of our cities in danger.
  - A losing AI's Peace offer pays 20% of its free Gold, at most 200 (nothing if that is under 20). It pays for a Peace
    it is asked to pay for only while losing, and only up to a third of its free Gold, at most 300.
  - A winning AI asks 25% of the smallest public treasury among the enemy team's empires, at most 250 (no price under
    20): its own Peace offers carry that ask, and it refuses a Peace that pays under half of it (it must still want
    Peace: with no plan, an army at 130% of the enemy's or more wants it only to leave a coalition's war).
  - Gold paid to the AI adds a point of treaty score per 10 Gold; it never pays for an Alliance or for Open Borders.
  - Gold its open offers may still pay is held back from the turn's purchases (`Planner::gold_left`), as is Gold it
    agrees to pay.
  - Open Borders: offered and accepted when a known city of the partner lies between one of ours and a known city (within
    20 hexes) of a team we are at War with or our diplomatic target, and the partner is no threat: not the strict VP
    leader or our top threat, no army stronger than ours within 10 hexes, no military unit within 5 hexes of our
    cities. A harmless partner that pays for them is accepted anyway. Ended (`BreakTreaty`) when the partner becomes a
    threat with a margin: the strict leader at half its target or our diplomatic target, an army 25% stronger nearby,
    or three military units within 3 hexes of our cities.
  - Coalition Gold (Hard only): every fifth turn, a solvent AI (positive income, not bankrupt) gives 10% of its Gold
    above its reserve, 10 to 50, to the weakest met partner at War with the leader it fights itself
    (`shared_leader_war`), or an ally at War with its diplomatic target (`Order::GiveGold`).
  - The proxies (`proxy_v4`..`v8`, sim benchmarks where deals are off) still decline Open Borders and offers that ask
    them for Gold.
- **Tests:** `crates/core/tests/deals.rs`.
