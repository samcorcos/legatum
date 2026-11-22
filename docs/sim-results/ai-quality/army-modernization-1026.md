# Army modernization audit — 2026-10-26

**Enabled default: combat valuation and active military-queue reconsideration (`modernization_value = 100`). Upgrade prioritization and return-home routing are implemented but remain at 0.** Valuation passed the paired quality gate and war check. Returning home passed the quality gate alone; its combination with valuation was not tested, and the tested three-gene bundle did not qualify.

The Ancient armies are real, but the causes differ. **Sparta has no strategic supply; Japan has exhausted Iron and very little Gold; Byzantium has funded upgrades it leaves unused; the Mongols actually bought 15 upgrades in the last 40 turns.** The AI also values heads and linear strength too highly, does not routinely reconsider an active military build, and normally upgrades only units that happen to hold in own territory.

Two corrections to the playtest recollection matter. In this exact save, **pink/rose is Japan (seat 5); Sparta is violet (seat 4)** (`app/src/theme.ts`). Japan is at war with Switzerland; Sparta is at war with the Mongols and at peace with Switzerland in this snapshot. The human has researched Ballistics/Field Cannons, but the three surviving guns are **Arquebusiers**, not Field Cannons. These are snapshot facts, not a claim about what the designer saw before or after saving. All five AIs are **Normal**, not Hard.

## Source and method

Read-only source copy: `~/LegatumRuns/bug-saves/staging-ring-chur-20261026.legatum`; SHA-256 `ad4faaa0fd3ef9aff48697c7d3ac056665aa9eeca9d4e2242efc2a15dbe24d9d`. The separate working copy, extracted states and raw work logs live under `~/LegatumRuns/army-modernization-1026/` (`source.legatum`, `states/`, `work-logs/`). Original engine `0b2ff473c291f0c53bb82c4e82a8ec4f4c262c85`, seed `8204795017171929`.

`tools/scenarios/extract-state.sh` replayed all **142 recorded turns, with zero state-hash mismatches**, on the original engine on the build box. Pre-turn states/views T103–142 and final state T143 were retained. The supplementary original-brain replay matched **587/593** full order lists; six Byzantium lists differ at T72, 74, 75, 121, 122, 124. Therefore the historical ledger uses the **recorded orders and hash-verified events**, not regenerated AI orders. The six differences remain unattributed; no diagnosis here treats those regenerated decisions as exact original decision traces.

[Provenance](army-modernization-1026/provenance.json), [complete per-turn ledger](army-modernization-1026/ledger.json.gz), [summary and every unit-class option/blocker](army-modernization-1026/summary.json.gz), and [human-readable 40-turn build/Gold/upgrade history](army-modernization-1026/turns.md). The ledger includes every city queue, all known technologies, every unit's position/HP/level, all independent upgrade blockers, quoted Gold, supply reservations, and recorded orders each turn. Reproduce its summaries with `tools/ai-quality/army-modernization-1026/audit.py STATES SOURCE_SAVE OUTPUT_DIR ORIGINAL_RULES_JSON`.

Final army/Gold/tech figures below are T143. Legal production and upgrade quotes are the **pre-resolution T142** original-engine views. Only Sparta learns a military unlock that last turn (Metallurgy); it has no Coal to use it. Supply “spare” includes construction reservations, so supplied minus live users alone can overstate availability.

## Per-empire facts

The surviving empires and the human share these first 15 technologies: Irrigation, Bronze Working, Masonry, Currency, Writing, Mathematics, Engineering, Iron Working, Code of Laws, Philosophy, Feudalism, Machinery, Chivalry, Education and Crop Rotation. Sparta adds Gunpowder, Metallurgy and Military Engineering. The Mongols, Byzantium and Japan add those three plus Banking, Printing Press and Rifling. Switzerland adds Ballistics (`Cannonry` in the save) to that same set. Germany knows only Irrigation, Bronze Working and Masonry. These are the complete named sets at T143; no AI is secretly still in the Ancient era except eliminated Germany.

| Empire | Techs and current era at T143 | Gold T143 (range T103–142) | Combat army by era, excluding Scouts/civilians |
| --- | --- | ---: | --- |
| Human Switzerland | 22, Industrial; Rifling and Ballistics known | 6,481 (2,869–6,339) | 3 Renaissance Arquebusiers |
| Germany | 3, Ancient; Irrigation, Bronze Working, Masonry; eliminated | 0 (0–0) | None |
| Mongols | 21, Industrial; Rifling known | 31 (19–137) | 3 Ancient, 3 Classical, 5 Medieval, 4 Renaissance, 1 Industrial |
| Byzantium | 21, Industrial; Rifling known | 376 (18–394) | 3 Ancient, 3 Medieval, 1 Renaissance, 1 Industrial |
| Sparta | 18, Renaissance; Gunpowder, Military Engineering, Metallurgy known; Banking/Printing Press missing | 22 (3–102) | 13 Ancient, 2 Medieval |
| Japan | 21, Industrial; Rifling known | 43 (3–64) | 14 Ancient, 2 Classical, 3 Medieval, 1 Renaissance |

| Empire | Best researched melee / ranged / mounted | Best actually buildable T142 | Binding reason |
| --- | --- | --- | --- |
| Human | Rifleman / Field Cannon / Cuirassier | Same | All three supplied; no Flight, so no aircraft |
| Germany | Spearman / Archer / Horseman | None | Eliminated, no city |
| Mongols | Rifleman / Arquebusier / Cuirassier | Rifleman / Archer / Horseman | Iron 12 supplied, 10 live uses, **0 spare** after reservations; Niter 6 supplied, 5 live uses, **0 spare**; Coal 6 spare. Ballistics and Military Science not learned |
| Byzantium | Rifleman / Arquebusier / Cuirassier | None | **Thin Levies 8/8** blocks every new combat unit. Upgrades do not need a new levy slot; Niter 4 spare and Coal 1 spare are usable |
| Sparta | Musketman / Arquebusier / Cuirassier | Hoplite / Archer / Horseman | **0 Iron, 0 Niter, 0 Coal**; two Pikemen consume missing Iron and suffer the supply penalty. Cuirassier becomes researched only at T142 resolution |
| Japan | Rifleman / Arquebusier / Cuirassier | Same | Coal 8 spare and Niter 2 spare allow direct new builds; **0 spare Iron** blocks taking most Ancient units onto the upgrade ladder |

Timing matters: the Mongols, Byzantium and Japan all learn Rifling only at T140 resolution; Switzerland learns it at T141 and had learned Ballistics at T134. Those three AIs have had only two completed production turns with Riflemen unlocked by this snapshot. The older gunpowder options have been available much longer: Byzantium learns Gunpowder at T105, Japan at T108, the Mongols at T109, and Sparta at T120. Ancient completions late in the window therefore cannot all be explained by a brand-new unlock.

No AI knows Flight. The Mongol Keshig replaces the Classical Cataphract and loses its bow on upgrading to Knight; the legacy `upgrade_gains` accounts for this dual role. Sparta's Hoplite adjacency bonus ends at Phalangite, but **XP and promotion level persist through upgrades**. Byzantium's levy cap makes upgrading especially valuable. Germany's absence is elimination, not a modernization decision.

| AI | Completed combat units T103–142 | Actual upgrade purchases | Upgrade Gold spent | Legal unit-turn upgrade offers |
| --- | --- | ---: | ---: | ---: |
| Germany | None | 0 | 0 | 0 |
| Mongols | 4 Spearmen, 1 Keshig, 1 Arquebusier | 15 | 884 | 83 |
| Byzantium | 1 Pikeman, 1 Crossbowman, 1 Phalangite | 4 | 238 | 106 |
| Sparta | 3 Hoplites, 4 Archers | 1 | 84 | 6 |
| Japan | 4 Spearmen, 4 Archers, 1 Arquebusier | 5 | 216 | 17 |

An offer is a legal unit-turn, **not an independently affordable simultaneous purchase**; several compete for the same Gold and supply. Civilians and Scouts are in the detailed history. Sparta also built four Scouts and one Settler; Japan built one Scout and two Settlers. The table counts completed production, not merely queues selected.

At T142, Byzantium has **345 Gold**, +31/turn, and three legal Pikeman→Musketman upgrades (units 24, 147, 163), **58 Gold each**. None is bought that turn. It ends T143 with 376 Gold. Japan starts T142 with **29 Gold**, +14/turn: its home Pikemen need 58 Gold; its Phalangites need 42; most Ancient units additionally need Iron. The Mongols start T142 with 80 Gold and buy Rifleman upgrade 23 for all 80; unit 50's 42-Gold Pikeman offer is a competing use, not evidence of forgotten free money. Sparta starts T142 with 23 Gold and has no supply-legal upgrade at all. Its sole window upgrade is unit 15→Pikeman, T122, 84 Gold; a same-resource step does not repair the existing supply deficit.

Byzantium bought 420 Gold of nonmilitary improvements/buildings at T104/108/111/115/120 (Workshop, two Quarries, Mine, Granary). The other three surviving AIs made **no ItemBought purchases** in this 40-turn window. “They spent all the Gold elsewhere” is thus not a general explanation. Japan's net income was only +2 at T103 and +3 at T123, rising to +14 at T142; Sparta alternated tiny surpluses and deficits, ending around +1 before another completion.

## Why the old policy does this

- **The main resource block is real.** `supply::affordable` removes supply-blocked production; `resources::upgrade_blocker` and `rules::units::check_upgrade` enforce supply, tech, Gold and own territory. Sparta owns **no Iron, Niter or Coal resource tiles at T143**, so its zero supply is not merely an unbuilt Mine. Coal cannot skip an Iron-requiring intermediate upgrade. Directly building a current unit can avoid that intermediate resource, making replacement a distinct option.
- **Production scores quantity and fixed bonuses.** `production.rs::item_base_value` uses army/ranged head counts, base military values such as 2,400 for a funded campaign, and `composition::quality` bonuses of 0/200/250/400/550/700/850 across eras. `choose_item` discounts by `60 / (turns + 60)` and credits banked progress. Those fixed differences do not represent the enemy's combat power. `composition::obsolete` exempts the entire melee line; ranged obsolescence is a research veto rather than a comparison with a supplied alternative. Active military production normally bypasses `choose_item` altogether.
- **Emergency production gives cheap ranged units a flat bonus.** The recorded Japan order at T116 switches Hiroshima from Walls to an Archer; that Archer completes at T136. The original view offered an Archer in 19 turns, Composite Archer in 24, Crossbowman in 31, and Arquebusier in 39. `plan_emergency_production` ranks `(max(strength,ranged) + 20) × 1000 / (turns + 2)` when it needs a shooter: **1,666 / 1,615 / 1,545 / 1,585**, respectively. The Ancient Archer wins despite researched, supplied alternatives. This ranking explains the recorded choice; the original-brain replay matches Japan's entire T116 order list. The fixed +20 matters more to its 15 strength than to the Arquebusier's 45, and the active-queue rule then preserves the choice for twenty turns.
- **Several Gold paths use linear strength per cost.** `buy_surplus_army`, `buy_ready_defenders`, and frontline defender ranking use `max(strength,ranged) × 1000 / Gold`. A healthy Archer contributes only **11** standing combat points under the Strongest Army square, versus **211** for a Field Cannon (15²/20 vs 65²/20). Treating them as linear 15 vs 65 compresses the quality gap substantially. A Spearman costs 80 Production and a Rifleman 290: linear strength/Production is **0.250 vs 0.210**, favoring the cheap Spearman; squared combat points/Production are **0.250 vs 0.641**, favoring the Rifleman by 2.56×. The matching full-price Gold comparison has the same ratio. Combat-point ratios are a planning estimate, not a claim that every duel has exactly that win probability.
- **Upgrades are incidental and late.** `scripted.rs::planner_with` plans cities before military. `military.rs::hold_or_upgrade` upgrades only a unit assigned to hold, after Gold reserve/runway checks and `upgrade_supply_worth`. Moving, shooting, defending or campaigning can consume the order first. The older `frontline_upgrade_reserve` only protects a healthy unit standing in an exposed own city. It does not reserve a whole multi-step refit or bring a border army home.
- **Own territory is an independent obstacle.** At T142 the Mongols' Keshigs 86/127 can take the next military step for 40 Gold without extra Iron but stand away from home. Japan's Phalangite 18 is away; even at 42 Gold it must come home. The ledger lists territory, tech, cash and supply separately, so a resource failure is not misreported as a pathfinding failure.
- **Faction identity is situational value.** A level-0 adjacent Hoplite has 25 effective strength, equal to a Phalangite's base 25, so spending for only that step can have no immediate combat payoff. Later researched steps can justify crossing it. Promotions and veteran eligibility survive; upgrading does not reset the ladder. The levy cap and Keshig's bow are separate constraints, not reasons to keep all Ancient troops forever.

The [Athens report](athens-game-1025.md) independently demonstrated obsolete units stalled by lethal city fire while a modern garrison stayed home. This save adds exact resource and treasury evidence; it does not establish that every Athens unit had the same blocker.

## Implementation and validation

Three independent genes, each **0 = the previous policy**, implement combat valuation, refit investment, and returning home. The shared square helper is imported unchanged from `astra/fix-strongest-army-measure:crates/core/src/army.rs`; this branch does **not** switch the Strongest Army crown's scoring rule.

`modernization_value` uses squared effective strength against visible nearby opponents, with the unit's class bonus, supply, promotions and actual Hoplite adjacency. It discounts obsolete troops in army quotas, permits resource-constrained alternatives to compete by value, reconsiders running military production with banked progress credited, and uses combat value per Gold in routine purchases. No-contact estimates use legal own production, not hidden enemy technology.

`modernization_upgrade` compares legal researched upgrade chains with alternative purchases, charges every intermediate Gold cost and action turn, keeps solvency reserves, and spends before optional city investment. Several same-turn orders share the existing supply budget. It reserves toward worthwhile refits rather than repeatedly spending the small income installment. It recognizes that a same-resource upgrade of an unsupplied unit remains unsupplied.

`modernization_return` can replace an outside-territory staging hold after tactical orders with an explicit safe path home. It charges round-trip travel and lost service and only sends a funded set of units. It does not replace a shot or attack already selected. Hired companies and veterans already able to retire are excluded from refits.

The full original state also validates all 16 complete planner order lists (four surviving AI seats × Normal/Hard × genes off/on) against the core. Save-derived fixture tests cover the real supply block, funded Byzantine refits, the Ancient-vs-Industrial square, phalanx preservation, zero-gene behavior, stronger build selection, and a controlled home-routing counterfactual. These are AI changes; no resource grants, combat nerfs, extra vision or rule bypasses.

The square is a planning approximation: it does not simulate terrain, firing opportunities, the opponent's matchup-specific bonus or complete tactical duels. Production quotas estimate quality by unit kind; refit comparisons use the individual unit's HP, level and supply status. Future upgrade steps are reconsidered each turn; supply is reserved for emitted immediate orders, not promised across future turns. This change does not grant missing strategic resources or change the resource-acquisition policy.

### Quality gate

[The frozen protocol and reproduction commands](army-modernization-1026/gate/README.md) define four candidate arms, three planned looks, fresh salts, a repeated control and the stricter acceptance policy for this AI optimization. The combined policy passed the Spot war check: **38/64 captures, 59% reported, versus the 30% floor**, with zero scenario bug reports. [Native report](army-modernization-1026/gate/war-all.json.gz) and [binary/source provenance](army-modernization-1026/gate/war-all-provenance.json).

All four arms reached the final planned look: **4,096 games per arm**, paired with the same 4,096-game control. The independent repeat matched **all 1,024 complete game records exactly**. Across control, repeat and four candidates, this is **21,504 completed full games**; failed capacity launches contribute no observations. Every intermediate look is retained in the gate directory.

| Candidate | Control / candidate wins | Paired change | Adjusted interval | Decision |
| --- | ---: | ---: | --- | --- |
| Valuation | 1536 / 1628 | +2.246 pp | [-0.403, +4.895] pp | Noninferior; **enable 100** |
| Upgrade prioritization | 1536 / 1535 | -0.024 pp | [-2.334, +2.286] pp | Inconclusive; keep 0 |
| Return home | 1536 / 1560 | +0.586 pp | [-1.426, +2.598] pp | Noninferior alone; keep 0 in the selected configuration |
| All three | 1536 / 1474 | -1.514 pp | [-4.126, +1.099] pp | Inconclusive; do not enable bundle |

The intervals account for three looks and four arms (critical z = 2.865); the floor is −1.5 pp. The source gate's final `ship-monitor` classification is deliberately **not an enable result for this speculative AI optimization**. That exception is for already-decided designer rules, as explained in `astra/gate-noise-audit:docs/sim-results/ai-quality/gate-noise-1026.md`, “Protocol and decision.” The enable criterion here was declared before observing results: noninferiority, a positive point estimate, functional checks and the war check. A positive point estimate does not establish superiority after the family correction.

The selected arm is valuation alone, the strongest tested passing configuration. Do not infer that valuation plus routing would pass: that pair is not the three-gene bundle, and was not tested. [Machine-readable decision and all final comparisons](army-modernization-1026/gate/decision.json). The valuation-only war check also passed **38/64 (59% reported)** against the 30% floor; [report](army-modernization-1026/gate/war-value.json.gz), [binary provenance](army-modernization-1026/gate/war-value-provenance.json).

Because parameters apply to both the Hard human seat and its Search opponents, the relative win rate does not by itself prove an absolute gain or loss in tactical strength. These eight human seats include no Byzantium: improving the funded Byzantine upgrades demonstrated by the fixture can benefit opponents without a matching Byzantine human-seat benefit. Salt intervals describe these fixed maps, not unseen-map generalization. The standalone routing arm also passed its war check, **38/64 (59% reported)**; [report](army-modernization-1026/gate/war-return.json.gz) and [provenance](army-modernization-1026/gate/war-return-provenance.json). It remains off solely to avoid composing untested configurations.

### Final code checks

- `cargo fmt --all`: exit 0.
- `tools/sim/build-slot.sh -- cargo clippy --workspace --all-targets -- -D warnings`: exit 0 on the build box.
- `tools/sim/build-slot.sh -- cargo test --workspace --profile release-checked`: exit 0 on the build box, which automatically runs nextest plus doctests; **1,987 passed, 10 explicitly ignored tests skipped**, zero doctest failures. Overflow checks and debug assertions remained enabled. This includes determinism and pinned tapes.
- The imported gate tests: **21/21 passed**.
- Eight focused modernization tests are included in the workspace pass, including the 16 full-planner order lists described above. One additional focused test added afterward passed on the box: the exact T117 Hiroshima view retains its Archer at gene 0 and switches away at gene 100. It adds test coverage only; the frozen runtime is unchanged. A follow-up `cargo clippy -p legatum-ai --all-targets -- -D warnings`, also through the build slot on the box, passed with that additional test.

After promoting the valuation default, `cargo test -p legatum-ai --profile release-checked` ran **791 tests**: 787 passed and four old-policy assertions failed. The Hall and recorded-siege tests now explicitly isolate their historical genes with modernization at 0; the two opening-reinforcement tests assert both the old Archer choice and the new squared-value Spearman choice. The **four exact tests passed on rerun**; six existing ignored tests remained skipped. No runtime algorithm was changed after the frozen gate. A final AI-only Clippy pass with all targets and warnings denied passed.

`LEGATUM_REPIN_PROFILE=release-checked tools/sim/build-slot.sh -- tools/sim/repin.sh` regenerated the three mixed tapes and runner digests, with no manual fixture or pin editing. **3 pin checks and 12 determinism tests passed**. Final formatting and whitespace checks passed. Master remained at `08d6525b83acce196ee10c37433382da9a43ce4d`, so no merge from master was needed. No merge into master was performed.

[Machine-readable check results](army-modernization-1026/checks/results.json) and compressed logs in the same directory preserve the commands and results. The original save SHA-256 was rechecked and still matches the untouched copy.
