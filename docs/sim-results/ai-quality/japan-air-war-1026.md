# Japan's air war, 8.9.11 — 2026-10-26

**The AI doesn't know how to handle the air stage because it treats aircraft as optional production, not as a military system it must prepare for, protect against, and move with the front.** All four AIs reached Flight. None built a Fighter. Germany repeatedly interrupted Bomber production, finally completed one 35 turns after researching Flight, fired it twice, and lost it with its city. The AI has a working air-strike selector; it lacks the production commitments, interception coverage and operational movement that make that selector useful.

There are two important qualifications to the designer's account. **Athens did build and use Bombers**, including against Japan. And the decisive attack was **four Bombers plus two Future-era Autonomous Armour**, not Bombers alone. Japan's first attempt failed: Germany retook Leipzig and killed the capturing Infantry. The war lasted T172–221; the fast collapse was its final eleven turns, T211–221. Japan began the war at **2 VP versus Germany's 8**; Germany reached **9 VP after T172**, and again before the final offensive. Japan finished at **14 VP**, winning after T234. Its first three victory holds broke after one turn (T216→217, T219→220, T223→224); the successful ten-turn hold ran T225–234.

This branch adds three experimental genes: `air_defense`, `air_commitment`, and `air_rebase`. **All default to 0.** They change AI decisions, not combat, research, resources, visibility or victory rules. The fixtures demonstrate specific better actions, not a proven improvement in whole-game win rate. No rubber band was changed or credited for these interventions.

## What was reconstructed

- Source copy: `japan-win-bombers-20261026.legatum`, SHA-256 `f9f81664243336424ddc8c519b1e76fb367b930ef42308b8e42c42cda50188e7`. The supplied file was read and copied, never written.
- Recorded engine `d1233d48678a2e89630fa456f710db100084a7de`, seed `3811224394289556`, 48×32, human Japan plus Hard Sparta, Germany, Byzantium and Athens. Player IDs are respectively 0–4.
- **234/234 resolved-turn hashes match.** Every serialized field of the rebuilt T235 snapshot equals the saved snapshot. Tables use **pre-turn states**; a completion on T177 is available at the start of T178.
- Persistent original brains were replayed on each rebuilt, fogged player view. Germany **221/221**, Sparta **234/234**, Athens **234/234** complete order lists match the tape. Byzantium matches **129/230**, first differing at T108; the cause of that planner-memory divergence is not established. Byzantine actual actions below come from the tape, not the divergent replan.
- The deeper trace restores pre-plan memory for Germany T155–221 and eight late positions for each other AI. Germany matches **67/67**; Sparta and Athens each **8/8**. Byzantium matches **1/8**. Production diagnoses apply the same supply filtering as live planning; without it an unavailable Factory can misleadingly appear to beat a legal Bomber.

Evidence: [turn ledger](japan-1026/ledger.json.gz), [original planning and memory](japan-1026/original-plans.json.gz), [production scores, plan candidates and military phases](japan-1026/original-trace.json.gz), [Germany's every-turn account](japan-1026/germany-turns.md), and [each German unit's order and assigning phase](japan-1026/germany-unit-turns.csv.gz). The ledger contains every army roster, HP, position, research, city queue, supply, upgrade offer, visible rival and event. Damage numbers below are the engine's rolled damage, which can exceed remaining HP; they are not net HP removed.

## Germany's war

### Forces and preparation

| Pre-turn | Japan | Germany |
| --- | --- | --- |
| T172 declaration | 1 Rifleman (117), 1 veteran Field Cannon (47), 1 Scout (2); no aircraft yet; 11 cities; 1,353 Gold | 9 Spearmen, 4 Archers, 2 Riflemen, 1 Field Cannon; 1 Scout; no aircraft; 6 cities; 103 Gold |
| T173 | Bomber 162, bought for 920 Gold on T172, based at Kamakura (19); same ground force | A second Field Cannon completed; upgrades Rifleman 9 to Infantry this turn |
| T185, first capture | Infantry 117 at 10 HP in Leipzig; Field Cannon and Scout; 1 Bomber before buying 175 this turn | 7 Spearmen, 1 Archer, 1 Field Cannon, 1 Artillery; just lost Leipzig |
| T210, decisive push | 1 Autonomous Armour, 1 Tank, Field Cannon; 4 Bombers. Tank 217 upgrades to the second Autonomous Armour this turn | 4 Spearmen, 1 Archer, 2 Artillery, 1 Infantry; no completed aircraft |
| T214, Cologne | Armour 216 at 56 HP and 217 at 100; Field Cannon; 4 Bombers | 2 Spearmen, 1 Archer, 1 Phalangite, 1 Infantry; Bomber 221 at Cologne |

The Field Cannon stayed at Sendai `(16,17)` and defended the Spartan front. It was not a large German invasion army. The opening attack was one capture unit plus one Bomber; the final capture force was two fast armoured units. Between T172 and T221 Japan lost Infantry 117 to Germany (T185), Scout 2 to Athens (T199), and Tank 208 to Athens (T206). **No Japanese aircraft was damaged**: all four remained at 100 HP. Germany lost 34 ground combat units, two Scouts and its Bomber during the war; Athens contributed substantially to those losses.

Germany did see Rifleman 117 at `(9,19)` before the declaration. Its T170 assessment, still in force on T172, recorded Japan's border strength as **61**, against Germany's own army strength **432**. It knew **one** Japanese city location, Nara, versus Japan's public **11-city** count. This was not a visible massed army it ignored: Japan bought its first Bomber on the declaration turn, and rival hangars are private. But public Modern-era status and a known base within reach were available warnings. There was no defensive aircraft production response to either.

By T195 and T200 Germany's assessment called Japan's observed border strength **0**, despite ongoing bombing. The assessment tracks map units; it neither learns an air-threat budget from strikes nor includes hidden aircraft as located units. The correct response is an explicit uncertainty estimate, not reading the opponent's hangar.

### How each German city fell

| Turn | City | How it fell; Germany's response |
| --- | --- | --- |
| 184 | Leipzig (8), first loss | Already at 1 HP. Bomber 162 dealt 182 rolled damage, killed the Archer garrison through overflow; Infantry 117 entered, surviving at 10 HP. German Archer 50 also died contesting the move. |
| 185 | Leipzig retaken by Germany | Field Cannon 102 and Artillery 171 fired on the 10-HP Infantry, killing it; Spearman 82 retook the city. Japan's Bomber killed Artillery 171 in the simultaneous ranged phase, but it still fired. This was a successful tactical counterattack. |
| 211 | Leipzig, permanent loss | Bombers 162, 194 and 210 dealt 141 + 107 + 97; Spearman 82 died with the city's overflow. Armour 216 entered. The city shot the Armour for only 8. |
| 214 | Cologne (18) | Starting at 167 HP, all four Bombers dealt 102 + 87 + 124 + 89. Armour 216 entered. German Bomber 221 and the city each dealt 8 to the Armour; the Bomber was destroyed when its hangar changed hands. |
| 216 | Munich (3), capital | After Japan rebased into Leipzig and Cologne on T215, four Bombers dealt 48 + 48 + 41 + 50, leaving 13 city HP. Armour 217 finished with a 25-damage melee attack. Archer 143's attempted counterattack dealt only 4. |
| 218 | Berlin (17) | Two Bombers dealt 219 + 226 to the city; the other two killed nearby Phalangite 179 and Spearman 218. Armour 216 entered. The new Archer 229 died in the city. |
| 219 | Hamburg (20) | Two Bombers dealt 193 + 219; Armour 216 moved four tiles from Berlin and entered. Germany had no surviving ground units afterward. |
| 221 | Essen (13), last city | Four Bombers dealt 79 + 97 + 73 + 96; Armour 216 entered, eliminating Germany. The city still fired for 14 before falling. |

The position explains the collapse: Japanese Bombers could erase both city HP and obsolete garrisons without terrain screening or retaliation, while the armoured capture units took little damage. Captured cities then became bases for the next attack. Walls continued to add defensive strength, but **air attacks ignore line of sight and are not halved by Walls** (`los::ranged_clear`, `combat::wall_damage_percent`, `forecast_air_strike`). The effective Bomber strike strength was **114**, plus **36 against city HP**, at range **8**. These are era-scaled values, not the unscaled constants 64 and 20 in the data table.

### What Germany planned, and why

The linked [50-row turn account](japan-1026/germany-turns.md) covers every T172–221 decision. Its companion CSV names the last military phase assigning each unit's order. The strategic sequence is:

- **T172–184: Peace with Athens**, while normal tactical defense handled Japan. On T172 Japan was rejected as a campaign target with **“already at war with another empire, no second front”**; Athens' known cities all had forecast capture odds below **15%**. A Peace strategic label did not prevent defensive moves or shots. Germany upgraded 9 on T173, started Infantry at Cologne on T174, bought a Spearman at Essen on T176 and another at Cologne on T179, and upgraded Rifleman 146 on T181. Both upgraded Infantry died by T182.
- **T185: recover Leipzig.** The campaign forecast was **51%**, two turns, zero march; terms were capture **+2,589**, failed siege **−17**, supply **+10,404**, era deficit **−40**, total **12,936**. It did retake Leipzig that turn. The enormous supply term reflects the Coal/Niter/Oil lost with that city; it is not an air-superiority calculation.
- **T186–189: hold the recovery.** Further campaigns were rejected because the army was holding a captured city. Munich's newly started Bomber was replaced by Infantry on T186. This was the emergency production path, not a deliberate judgment that air power was inferior.
- **T190–221: contain Japan.** Example values: **35** on T192, **53** on T204, **188** on T210, **601** on T214, then **0** on T221. On T214 the terms were VP denied **29**, leader's race **598**, war footing **−15**, attrition **−11**. Peace scored **−197** because it would leave the leader its VP. Nevertheless, offensive campaign eligibility still rejected Japan as a second front. A positive containment score did not supply an operational attack that could stop the Bombers.

Military action continued: Germany made ground attacks, upgrades and purchases and retook a city. The failure was not “the AI does nothing when attacked.” It was defending and valuing the war with the wrong force model and repeatedly funding counters too late.

## Every AI's air layer

| Empire | Flight completed | Aircraft completed, turn and base | Strikes / rebases | Fighter coverage |
| --- | --- | --- | --- | --- |
| Sparta | 181 | None | 0 / 0 | None |
| Germany | 177 | Bomber 221, T212, Cologne | 2 / 0 | None |
| Byzantium | 205 | None; started a Bomber at Nicaea T225, lost Nicaea T229 | 0 / 0 | None |
| Athens | 179 | Bomber 195 T196 at Eleusis; 201 T199 at Piraeus; 228 T217 at Phaleron; 231 T218 at Eleusis | 21 / 0 | None |
| Japan, for comparison | 171 | Bomber 162 T172 Kamakura (bought); 175 T185 Nara (bought); 194 T196 Kyoto; 210 T204 Edo | 123 / 25 | No Fighters needed against the AI's ineffective response |

**There was no interception at all.** Fighters did not leave their guard posts; none existed. No AI ordered a Fighter build or purchase anywhere in the tape. There is no separate ground anti-air unit in this ruleset. Aircraft cannot be attacked as ordinary map units; the available counters are interception and capturing a hangar city. No AI captured a Japanese hangar during the air war. Sparta's Kanazawa capture was T119, before any aircraft existed, and Japan recovered it T124.

Germany's two strikes, T213 and T214, targeted Autonomous Armour 216 for **10 and 8** damage. Athens made **nine strikes against Germany and twelve against Japan**. Its useful attacks included Leipzig for 162 on T197, Japan's Scout for 246 on T199, German Artillery for 129 + 122 on T202, and Japan's Tank for 75 on T206. Late attacks against Autonomous Armour generally dealt **7–10**, while its T225 strike on Japanese-held Eleusis dealt **134**. The strike selector was functional; its ability to select the best current target did not compensate for obsolete ground forces or unprotected bases.

Athens lost both Eleusis Bombers on T223, Piraeus' Bomber on T225 and Phaleron's on T226 when Japan took those cities. It never rebased any of them. Germany similarly lost Bomber 221 at Cologne. Japan used captured airbases immediately and made **25 successful rebase orders**.

The other empires expose the same gates in different positions:

- **Sparta:** at T185 the idle city Pylos was ten hexes from Sendai, beyond the production rule's nine-hex reach, so its Bomber value was zero. It chose a Spearman at **2,527**, ahead of a Phalangite at **2,378**. Front bases existed, but they were slow and committed elsewhere: Sparta Colony 9 quoted **154 turns for a Bomber** while building Walls; Colony 7 quoted 42 while building Roadworks. The new rebase policy does not yet make a safe rear city value a Bomber for later deployment; that requires production and basing to be planned together.
- **Athens:** T180 Piraeus and Marathon selected Bombers at **1,296 raw**, discounted to **980** and **960**. It therefore did use the unlocked unit. After its first completion, Eleusis T197 still chose Artillery **2,610** over Bomber **1,105** and Fighter **566**. At T224 it chose Bombers again: Athens **929 versus Fighter 491**, Piraeus **1,192 versus Fighter 517**. These are matching original-brain decisions.
- **Byzantium:** it completed Flight at peace on T205, so neither the war-only Bomber value nor the own-Bomber prerequisite for Fighters admitted a defensive air investment. Its actual tape starts Research Labs in both cities at T214; those finish T221 and T224. It declared on Japan T216, then started a Bomber at Nicaea T225 with a **16-turn** quote, four turns before losing the city. This statement uses tape and core production quotes; the divergent Byzantine military replan is not used as evidence of its private strategic reasoning.

### Exact gates and scores

1. **Fighters depend on owning Bombers.** `crates/ai/src/air.rs::production_value_with_policy` retains the old zero-gene path: Bomber **1,800** when a war is within range + 1; Fighter **900** only if own Bombers > own Fighters; otherwise **0**. The empire cap is aircraft count ≥ own city count, and each physical hangar holds two. Before its first Bomber, Germany therefore valued all Fighters at zero, regardless of incoming bombs. This is a production failure, not a guard-order failure.
2. **A Fighter loses the production competition even after one Bomber exists.** At Essen on **T214**, the original planner scored Fighter **1,107 raw / 945 discounted**, Bomber **2,214 / 2,800**, Research Lab **1,204 / 1,044**, Scout **246 / 250**. The raw air numbers include Germany's **123% military production strategy weight**; the Bomber's banked progress adds to its score. The ordinary scorer uses `value * 60 / (turns + 60)`, up to a half-value banked-progress bonus, and jitter of at most 15. It chose the Bomber. At Munich T185, Bomber was **2,214 / 1,686**, Star Fort **741 / 614**, Fighter absent because its value was zero.
3. **Useful aircraft builds are treated as interruptible civilian work.** `production.rs::plan_emergency_production` preserves a current ground military build, but `is_military_unit` calls `composition::is_combat`, which excludes aircraft. The emergency candidate set also excludes them. Munich: Bomber T185 → Infantry T186; Cologne: Bomber T192 → Artillery T198 → resume Bomber T209 → complete T212. The emergency ground selector ranks roughly `max(strength,ranged) * 1000 / (turns + 2)`, with +20 for a needed shooter. It never compares the pending Bomber's future sorties or a Fighter's interceptions.
4. **Buying does not reproduce Japan's timing.** Japan spent 920 Gold to get an immediate Bomber on T172 and again T185. Germany had 103 and 108 Gold at those pre-turns. Emergency and routine military purchase paths filter to ground combat units. Adding planes to production alone does not create a financed aircraft purchase strategy.
5. **Guard logic exists but had no units to run on.** With `fighter_strike=1`, the old `air.rs::bombers_likely` keeps a Fighter guarding if a rival at war is in the Flight era and has a known city within **8 + 4 = 12** hexes, unless a strike value exceeds fixed **120**. A valid Fighter strike spends its interception for that turn. Fighter guard radius is **4**, strike range **5**. The existing scorer values forecast HP damage times target Production cost / 100 plus a kill term. It does not place Fighters to cover particular assets or react to recorded bomb sorties.
6. **There was no rebase planner.** The old air function emitted only `AirStrike`. Neither an empty target list nor a threatened hangar generated `Rebase`; it simply did nothing or fired once more. Interceptions are not forecast by the ordinary air-strike forecast because rival hangars are hidden.
7. **Macro and research remain ground-centric.** `opportunity.rs::strike_force_within` / `forecast_siege` build their force from map units; the original campaign forecast does not price a joint bomber-and-capture operation. `plan.rs::campaign_against` rejects second fronts before value comparison. `research.rs` gives Flight the generic unit-line term **20 + military/2 + 60 if arms-racing**, shared with Steel, Combustion and Replaceable Parts. There is no special value for obtaining first air superiority or urgently unlocking interception after bombing.

## Modern composition and Oil

Germany was technologically competitive, but not militarily modern. It entered Modern at T160, researched Combustion T166, Replaceable Parts T172, Flight T177 and Steel T183. Japan prioritized Flight first and later researched Autonomous Systems T201, then upgraded two Tanks to Autonomous Armour. Germany never completed Autonomous Systems before elimination. It researched Quantum Computing and Recursive AI while remaining heavily dependent on ancient troops.

At T172 Germany had **9 Iron supply, all 9 used by Forges and Universities**, leaving **0** for the Spearman → Phalangite or Archer → Composite Archer upgrade step. Coal had **8 supply, 3 active users and 5 reservations**, also **0 spare**. Oil had **8 supply, 0 users, 0 reservations**. Spare Oil cannot skip the intermediate upgrade steps. The core requires territory, tech, Gold and supply (`rules::units::check_upgrade`, exposed in the view); AI upgrade valuation additionally compares the upgrade against industry demand (`military.rs::upgrade_supply_worth`, `supply_value::unit_outbids_industry`).

| Pre-turn | Japan Oil supplied / used / spare | Germany | Sparta | Athens | Byzantium |
| --- | --- | --- | --- | --- | --- |
| 172 | 10 / 0 / 10 | 8 / 0 / 8 | 2 / 0 / 2 | 9 / 1 / 8 | 2 / 0 / 2 |
| 185 | 12 / 1 / 11 | 6 / 1 / 3 | 4 / 0 / 3 | 9 / 1 / 8 | 3 / 0 / 3 |
| 210 | 14 / 1 / 10 | 9 / 3 / 6 | 4 / 1 / 1 | 9 / 1 / 5 | 3 / 0 / 3 |

Spare excludes reservations, explaining why it need not equal supplied minus used. Between T160 and T221 Germany upgraded two Infantry, three Artillery and one Phalangite; Athens upgraded two Infantry; Sparta made more intermediate upgrades but remained mostly pre-Modern until very late; Byzantium upgraded three intermediate units. By T210 Germany still had five ancient combat units out of eight, Athens three Spearmen plus one Artillery, and Byzantium no Modern combat unit.

**Aircraft used no Oil in this save.** `resources::unit_need` explicitly excludes class Air. The air scorer's “unmet supply” veto was therefore inactive for these planes; it did not explain their absence. Simply adding Oil would constrain the human's free air arm but would not make the AI buy Fighters or preserve its builds. The open `astra/aircraft-use-oil` branch (`d4dd8ee20`) adds aircraft supply and replaces the blanket unmet-demand veto with value-based allocation. Integrate and re-test that code with these genes; this branch deliberately does not duplicate the resource rule change.

## Strategy: could an AI make the same comeback?

The AI does consider attacking leaders. Germany declared on Athens at T54 when Athens led, pricing “leader's race” at **+1,008** and choosing containment at **+587**. Athens declared on Germany at **T135** with **5 VP versus Germany's 6**. Germany stayed in that war through Japan's invasion. Athens then declared on Japan at T185 (a four-way tie at 5), and Byzantium declared on Japan at T216 when Japan had 9 and Byzantium 3. Sparta had already been at war with Japan since T101.

The missing concept is the **decisive, achievable military operation**, not merely a “declare on the leader” trigger. Germany's T214 planner explicitly priced denying Japan's lead, yet its candidate generator rejected a Japanese campaign because Athens was another front. It could not represent “protect these cities with Fighters, use Bombers to strip that city, then capture it with this fast unit,” nor “capture the base that keeps bombing us.” Japan won with exactly that operational sequence.

- **VP-race work is already active.** `winchance.rs`, `victory.rs`, `plan.rs`, `hold.rs` and `diplomacy.rs` price leader progress, denial and refusing a bad Peace. The trace shows those terms firing. They cannot select a plan that the force estimator and second-front admission rules never offer.
- **Local opportunism is the right interface.** `astra/ai-local-opportunism` adds `local_war_force`, `limited_war` and `exposed_flank`, uses committable theatre forces rather than whole-empire counts, and includes own available Bombers plus a public enemy-air uncertainty allowance. This directly addresses the mismatch between Japan's small map army and its ability to take a local city. Its documented enabled stack failed the Golden Horde gate (**0/64 versus control 40/64**); do not infer that integrating it unchanged solves this game or enable it by default.
- **Air/Oil and strategy must be measured together.** The Oil branch closes a resource loophole; these genes make defensive air and rebasing available; local-force work can make an air-supported campaign eligible. None alone provides a validated Modern war policy. The next strategy-lab arms should compare a Flight rush with financed Bombers and capture units, a Fighter screen plus ground push, and an airbase-capture plan, including opponents-only and human-seat-only benchmarks.

## Implemented fixes and what the fixtures prove

| Gene, default | Candidate setting | Change |
| --- | --- | --- |
| `air_defense=0` | 100 | Fighter production and guard value come from the largest uncovered own city or ground asset within a known non-allied Modern base's Bomber range. Counts existing coverage and Fighter queues; avoids double-valuing overlapping guards. A needed Fighter can exceed the old empire aircraft quota, but never a hangar cap. |
| `air_commitment=0` | 1 | A useful Bomber/Fighter already being built survives emergency ground-production replacement unless the city is forecast to fall now. |
| `air_rebase=0` | 1 | Idle aircraft move to a strictly better reachable own base; endangered aircraft can evacuate before ground capture. Bases reserve inbound slots in aircraft-ID order, without spending same-turn departures that validation does not credit. |

The coverage model deliberately uses **potential** hostile air reach from public era and known cities. It does not pretend to know Flight specifically, aircraft counts, enemy build queues or unseen bases. City guard value is `120 + 20 * population`; a troop is its Production cost times remaining HP / 100. The greatest uncovered asset sets the value because one Fighter stops one strike. Production multiplies that by 10 and the gene percentage; guard duty compares the asset value directly with the strike score. These weights need full-game tuning.

Fixtures originate in the rebuilt game, with original memory and recorded orders. Explicit interventions are labeled in the [fixture README](../../../crates/ai/src/fixtures/japan-1026/README.md):

- **T185 Germany, unchanged position:** zero genes reproduce the recorded full order list; enabling defense makes Munich choose a Fighter before Germany owns any Bomber.
- **T186 Germany, unchanged position:** the original planner replaces Munich's Bomber with Infantry; commitment retains the useful Bomber queue.
- **T214 Germany, aircraft-type intervention:** changing its one completed Bomber to a Fighter makes it guard Cologne; core resolution cancels exactly one of Japan's four recorded targets, leaving three city strikes. One Fighter is not claimed to stop a four-plane wave or reverse the war.
- **T214 Germany, actual Bomber:** enabling rebasing evacuates aircraft 221 before the recorded Japanese ground capture; the baseline loses it with Cologne. The city still falls in both arms.
- **T197 Athens, actual Bomber:** it legally strikes Leipzig and deals over 100 damage with the new policy enabled. Offensive air use remains available.
- Synthetic tests isolate an idle rear Bomber rebasing then striking, last-slot hangar reservation, and suppression of redundant Fighter production by an existing or queued defender.

The [six candidate position comparisons](japan-1026/candidate-positions.json.gz) preserve the exact resulting orders and production scores. At T185 the new Munich Fighter scores **4,920 raw / 3,842 discounted**, beating the unchanged Bomber at **2,214 / 1,686**; at T214 Bomber 221 rebases to Munich (3). T178 Germany, T197 Athens and T198 Athens retain their recorded full order lists. These are local interventions, not continuation wins.

The diagnostic production API now applies the live supply filter. That fixes audit accuracy only; it changes no live orders. The state extractor and `war_position` example are ported from the Athens/Sparta analysis branches, with recorded and reconstructed orders stored separately.

**Still needed:** first-air-threat research valuation; Gold plans for aircraft purchases; multi-interceptor coverage against several possible Bombers; remembered strike evidence; affordability across the whole upgrade chain; aircraft contributions to city capture forecasts; selecting enemy bases as objectives without omniscience; multi-hop rebasing; and paired continuation/full-game experiments. The public-era prior may overbuild Fighters against a peaceful Modern neighbour with no aircraft. The evacuation heuristic is conservative and can sacrifice a useful sortie. Those risks are why the genes remain off.

## Reproduction and validation

```sh
# Use a copy; never point an output path at the supplied save.
tools/scenarios/extract-state.sh COPY.legatum --out STATES --every 1 --from 1 --to 235 --audit-plans
python3 tools/ai-quality/japan-1026/analyze.py COPY.legatum STATES docs/sim-results/ai-quality/japan-1026
tools/ai-quality/japan-1026/trace.sh STATES TRACE_OUT
# Copy TRACE_OUT/trace.json.gz to japan-1026/original-trace.json.gz, then:
python3 tools/ai-quality/japan-1026/summarize.py docs/sim-results/ai-quality/japan-1026
```

Final checks on implementation `81055d8fe` (the cloud snapshot had identical Rust sources), all exit **0**:

- `cargo fmt --all`: passed; `git diff --check`: passed.
- `tools/sim/build-slot.sh -- cargo clippy --workspace --all-targets -- -D warnings`: passed, no warnings (box job `20261026T050035Z-d463c7`).
- `tools/sim/build-slot.sh -- cargo test --workspace --profile release-checked`: **1,968 passed, 10 skipped**, no failures; all seven crate doctest steps passed (zero doctests). The automatic cloud adapter ran nextest plus doctests, retaining this profile's overflow checks and debug assertions (job `20261026T050036Z-0e5bb4`). This includes determinism and pin tests; pins did not change.
- Final focused air run: **14/14 passed**, including eight new regression tests. Earlier focused failures caught an off-by-one mismatch between public era numbering and the zero-based helper; it was corrected before these final checks.
- Python scripts compile; both shell scripts pass `bash -n`.
- Original replay **234/234 hashes**, final snapshot equality, and source-copy SHA-256 equality all verified.

[Clippy log](japan-1026/clippy.log.gz), [workspace test log](japan-1026/workspace-tests.log.gz). Local master remained `acc7ddfc0`, the branch's original base, through final validation; no merge or pin regeneration was necessary. No whole-game win-rate claim is made from these one-turn fixtures. The genes remain off pending paired continuation and full-game validation.

Co-Authored-By: GPT-6 Astra (Codex) <noreply@openai.com>
