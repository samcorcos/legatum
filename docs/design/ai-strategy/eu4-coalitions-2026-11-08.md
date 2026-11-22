# EU4 coalitions, and what Legatum should take from them (2026-11-08)

The designer, 2026-11-08: EU4's coalitions contain a runaway, the war "gets messy really quickly", and the player plays to avoid one. Research only; no code changed.

## 1. EU4's mechanism in brief

- **Aggressive expansion (AE).** Taking land costs opinion with *other* countries: development (cap 30) × term (province 60%) × casus belli (conquest 100%, reconquest 25%); +50% each for the loser's religion and culture, down to 25% with distance, two-thirds for your allies. It decays about 2 a year, faster with "improve relations". [EU4 wiki, Relations/AE; fandomspot guide]
- **When a coalition forms.** A country with AE of −50 or worse toward you, no truce with you and not your ally may join a coalition against you. AI members must also be "outraged", there must be at least 4 eligible countries, and the coalition must be strong enough to threaten you. It dissolves as AE decays, or after its war. [EU4 wiki, Coalition]
- **The defensive trigger.** If the target attacks any member, every member joins at once and cannot decline. With 4 members they may also attack together. No separate peace, and −30 to accepting peace. [EU4 wiki]
- **Visible before any war.** The Coalition map mode shows each country's AE toward you, and the peace screen shows the AE a deal will add and who it would push past −50, before you sign. [EU4 wiki; fandomspot]
- **War leader and goals.** The attacker, or the member attacked, leads and negotiates for everyone. The coalition's war goal ticks warscore from battles won. [EU4 wiki; r/eu4 guides]
- **How the AI fights it.** Dev Diary 43: allies too weak to act alone attach their armies to the war leader's. The October 2018 diary: armies score provinces as destinations, prefer safe home ground, and stand when the war is lost. Players report the AI dodges stronger stacks, hits weaker ones, sieges fast and joins only wars it expects to win; fronts open naturally because each member marches from its own border; attrition punishes stacking. (Mostly forum-sourced; the diaries were read from search excerpts.)
- **How players avoid it.** Pace conquests per region, spread AE across religions and cultures, ally likely members, improve relations, force a truce on a forming member, use the reconquest CB, take Espionage ideas (−20% AE).
- **Why it feels threatening.** A slow, visible fuse you can play around; one greedy peace deal tips it; then the neighbourhood attacks at once from every side and can't be peeled off.
- **Known frustrations.** "Dogpiled", "the AI gangs up on the player" (others reply it judges every nation alike), chasing scattered stacks, allied AIs suiciding into strong stacks. [Steam and Paradox forum threads]

## 2. Where our coalition falls short

**The trigger ignores conquest and isn't something to play around.** The formal coalition (`core/src/diplomacy/coalition.rs::select`) is VP only: progress ≥80%, or ≥60% with a 15-point lead, or a hold. City pressure is off unless the `expansion_coalitions` experiment is set. A conqueror at 4/10 VP is invisible (learnings, Mongol rush: Athens and Byzantium eliminated by T52, every other AI stayed at Peace). The conquest-aware reads (`containment_target`, the branch's `runaway` score) are AI internals that the player never sees, so there is nothing to pace against.

**No coalition exists before the war.** `runaway_alliance_value` (branch `claude/coalition-front`) returns 0 unless the partner is already at War with the runaway, and `bloc_size_penalty` charges −60 per team beyond a pair. EU4's core, a standing pact that any attack on a member triggers, is missing, even though our Alliance rule already brings defenders in at once (§5).

**Coalition wars don't fight.** Playtest 3: 26 AI Alliances named him as the target, 2 AIs declared, 0 battles. In 8.9.2, Germany sat at war for 73 turns and Japan for 41 with no attack. The 8.9.26 Celts ran Contain on him for 78 turns without an attack. `coalition_push`: 19 hits on his cities in 160 turns. Swiss hold probe (`celts-break-the-reserve`, 64 runs): wars declared on the Swiss 0.06 → 0.07 with the gene, second fronts 0, Geneva 0/64. AI-only census (4,096 games): a T100 runaway still wins 59.7% → 59.9%.

**Where it worked.** `byzantium-empty-horde` with `runaway_coalition` 3: Byzantine cities held 0.50 → 3.12, Peace with the Mongols 1.25 → 0.59, Sparta opened 41 second fronts. Committed operations that the plan layer can't cancel are what moved it.

**Our gates can't see it.** The human-seat gate changed only 154 of 2,048 games (+0.54 pp, CI −0.88 to +1.90), because the Hard seat is rarely the runaway. Coalition work has to be measured in scenarios where the human is the runaway.

## 3. The strongest case against copying EU4

1. **The real bottleneck is battle tactics, not coalition logic.** `siege_oracle` made Hard braver and worse (seat −7.3 pp; units lost per capture 7.67 → 10.29). The tactics Lab cards are 0/50. The Tepid Swiss went 47 turns of war with 16 units and took nothing; Bombers cleared Athens and nobody walked in. EU4's coalition frightens because its armies win sieges. A trigger that starts more wars Hard then fights badly adds wars, not captures (10-18: token wars cost the leader a little; they took nothing).
2. **It taxes the core loop.** Pillar 10 makes expansion by conquest the core loop and rules out tall. AE is a per-capture tax on exactly that, and it hits Sparta, the Mongols and the Aztecs hardest. That weakens a signature strength, which pillar 8 forbids.
3. **The opinion layer is hidden math.** "Outraged" attitude and "the coalition could threaten you" are opaque AI judgements, against pillar 5. We removed hidden trust on 2026-10-03.
4. **The dogpile with two humans.** Six AIs on a permanent human team, wars joined automatically, no separate peace: EU4's top complaint, amplified, and the co-op partner is a target player.
5. **It doesn't port.** We have no peace terms, relations, religion, culture or espionage (§9), so most of EU4's counterplay has nothing to map to.
6. **Rules are frozen** until the 11-13 to 11-18 AI checkpoint.

Points 1 and 3 are decisive for sequencing, not for the idea: fix how coalition AIs fight with what already exists, and only then add a visible rule.

## 4. Proposals

### (a) AI only, possible under the freeze

All build on `claude/coalition-front`. Gates, in order: human-runaway scenarios (`celts-break-the-reserve`, `byzantium-empty-horde`, `runaway-conqueror`, `war-break-the-hold-t168`, `celts-stop-the-horde`; 64 reseeded runs), the human-seat gate (not significantly worse), the designer seat (must improve for strategy changes; noninferiority for tactics).

- **A1. A standing bloc before the war (EU4's pact, in our Alliance rule).** While a runaway scores ≥60, empires within reach of it value an Alliance with each other at Peace by the share of its army the pair can bring, with no `bloc_size_penalty` for that bloc. Our defence rule then makes the bloc a coalition: attack one member and all join at once. Alliances are public, so the bloc is visible. Gate: Alliances signed before the first war on the runaway (Swiss probe: 0); the runaway's captures in `runaway-conqueror` down; AI-only wars per game not up (free blocs of 5 to 6 once meant 36 wars a game against 22.5).
- **A2. Strike a hollow runaway.** The branch's named next step: commit an operation on a weakly held runaway city without the army-away test. Gate: wars on the Swiss ≥0.5 a run and Geneva > 0/64 in `celts-break-the-reserve`.
- **A3. One war leader, one target.** The strongest member at war with the runaway picks one city; others attach (stage adjacent, strike the same turn) or take bit 2's second front, whichever costs the runaway more. Gate: same-target strikes within 2 turns (a third of pairs today) and battles on runaway cities, both up.
- **A4. Hunt the field army when it's beatable.** When the runaway's main army is away and outweighed by a member's force in reach, the operation targets the army. Gate: combat lab and war suite.

### (b) Rules and UI, after the checkpoint

- **B1. A public Alarm meter.** Each empire has an Alarm number toward each other team: +10 per city that team captures within 10 hexes of the empire's cities, +20 per elimination, −1 a turn. Each point has a breakdown line on the diplomacy row. The warning fires when 3 empires (or half the others) reach 50, for AI leaders exactly as for humans, and the capture forecast says "Taking Nicaea: Alarm +10 with Athens and Sparta → coalition 3 of 3". It is AE without hidden opinion, and it prices the *rate* of conquest, not the total, so conquest stays the core loop. It replaces the VP-only trigger in `coalition::select`; holds still name the target.
- **B2. The defensive trigger, inside Peace/Alliance/War.** An Alliance between two Alarmed empires is the coalition; defenders already join at once. The only possible rule change is waiving its upkeep.
- **Not recommended:** no separate peace for members (it reverses the 2026-10-23 pairwise-peace decision and builds in the dogpile), and opinion or relations stats (pillar 5).

## 5. Recommendation: build A1 first

A1 is the skateboard of EU4's coalition, built from rules we already ship: a standing bloc that an attack on any member sets off, which the player can see (Alliances are public) and play around by pacing conquests or striking before it forms. No new rule. It partly sidesteps the tactics bottleneck: defenders join automatically, and each one is a front for bit 2. Its gap is pillar 5: the player sees the Alliances, not the runaway score behind them, until B1 shows the number. Build A2 in the same branch; a bloc that never strikes a hollow leader is the Swiss probe's 0/64 again. B1 waits for the checkpoint and for evidence that a visible bloc changes how the designer plays.

## Sources

- EU4 wiki, Coalition (Alliance page): https://eu4.paradoxwikis.com/Coalition
- EU4 wiki, Aggressive expansion (Relations page, verified to patch 1.25): https://eu4.paradoxwikis.com/Aggressive_expansion
- Dev Diary 43, "Artificially Improved" (search excerpt): https://forum.paradoxplaza.com/forum/threads/europa-universalis-iv-developer-diary-43-artificially-improved.705072/
- EU4 dev diary, 16 October 2018, army AI (search excerpt): https://admin-forum.paradoxplaza.com/forum/developer-diary/eu4-development-diary-16th-of-october-2018.1124066/
- FandomSpot, Handling AE and coalitions: https://www.fandomspot.com/eu4-handling-aggressive-expansion-coalitions/
- Player complaints and counter-arguments: https://steamcommunity.com/app/236850/discussions/0/3201496916529769380 ; https://forum.paradoxplaza.com/forum/threads/dev-diary-11-stopping-the-snowball.896247/page-4
- Legatum: `crates/core/src/diplomacy/coalition.rs`; `crates/ai/src/diplomacy.rs::bloc_size_cost`; `claude/coalition-front:crates/ai/src/runaway.rs` and `docs/sim-results/ai-quality/coalition-front-1107/README.md`; `docs/learnings.md` (2026-09-29, 2026-10-18, 2026-11-01, 2026-11-07); `legatum-rules` §5 and §9.

## Addendum: the designer's decision (2026-11-08)

- **Hidden wariness, not a meter.** Each AI keeps a wariness score toward each empire, for its own decisions only, built from public information: nearby conquests (3 to 5 cities or a whole civ, not 1 or 2), eliminations, a military build-up and a VP lead, strongest when it looks like the next target. Responses scale smoothly or are drawn at random, so nobody can game a threshold.
- **Responses:** seek an ally next door or one able to open another front, accept alliances against the threat, and build defences.
- **Coalition wars go all in at once.** He sees this as the biggest gap.
- **Dogpiles are fine.** B1 (the visible Alarm meter) is rejected.
- Building on `claude/coalition-bloc`: wariness replaces A1's trigger, and an all-in coalition commit is added to A2.
