# Athens needs a real drawback: design brief (2026-11-05)

## The problem
Athens is the science faction and tops every AI-only census: 20.0% of seats win (4,096 Hard games, 9 factions, fair share about 11%; the worst faction, Germany, is 6.4%). It has held first place for weeks across engine changes (16.9% on 10-22, 20.4% on 11-01).

What it has today (code-verified, `crates/core/src/data.rs::building_stats_for`, `rules/units.rs` ruins, `rules/actions.rs` broadcast):
- **Lyceum** replaces the Library (Laboratory family): +6 Science instead of +3, and costs 70 Production instead of 80.
- **Ruins** give their normal reward plus 20 stored Science.
- **Weak Markets** (the only standing drawback): Markets give +1 Gold instead of +3.
- **Goal, Share the Learning:** a one-time action. Every living rival outside Athens's team gets 3 free technologies of its choice, and Athens gets +2 VP for good. The AI fires it at a median T164 of a median 183-turn game, when the free techs barely matter, so the price is close to zero. Completion is 31%.

Where Athens's VP comes from (census, per seat): Great Works 1.16 (highest of all factions), era objectives 2.73, the Most Advanced crown held 2x as often as most factions, faction goal 0.66. Science also means better units sooner (strict era gate: you research only your current era's techs; finishing an era enters the next), so it is not militarily soft.

## How Legatum designs factions (read these before proposing)
- `CLAUDE.md` pillars, especially 8: asymmetry with compensating drawbacks (TI4, StarCraft, not Eclipse). Each faction leans hard into its play style; its strength is paid for by a weakness elsewhere that pushes it further into that style. Balance by adding or deepening drawbacks and by making weak identities pay off; never by weakening strengths or homogenizing. Pillar 10: every strategy has a counter. Pillar 5: no hidden math.
- `.claude/skills/legatum-design/SKILL.md`: drawbacks are decisions with visible counters and ideally scale themselves (Swiss treasury bounty: rich means targeted); mechanics follow from the story, never arbitrary numbers; bend existing systems instead of adding new ones; every rule fits one plain sentence.
- `.claude/skills/legatum-factions/SKILL.md`: the faction method, all current designs, lessons, and rejected ideas.
- `.claude/skills/legatum-rules/references/game-rules.md`: how the game actually works. Read section 9 ("Systems that do NOT exist") and check every idea against it: one order per unit or city per turn, simultaneous turns, no trade routes, no naval game to speak of, etc.
- Factions are real peoples, grounded in real history, no fantasy.

## Rejected for Athens (do not re-propose)
- **"School of Hellas"** (rivals research techs Athens knows faster): the designer: "it basically just means 'any game athens is in, everyone researches faster.' it lifts all boats that doesn't affect competition." A drawback must change the competition around Athens: cost Athens relative to its rivals, or give rivals a lever they can pull against it. A table-wide effect that applies whether or not anyone engages with Athens fails.
- Plain number nerfs (Lyceum +5 instead of +6, etc.): pillar 8 says deepen drawbacks, don't weaken strengths, and number tweaks feel samey.
- The Delian League is parked for a possible future faction.

## What we want
6 to 10 candidate drawbacks (or a reshaped goal whose price really bites), each:
1. One plain sentence a player reads on the faction sheet.
2. The history it comes from (real Athens: the Peloponnesian War, Spartan invasions of Attica and the Long Walls, the plague of 430 BC, ostracism and the assembly voting on war, the Sicilian Expedition, the Thirty Tyrants, Athenian dependence on imported grain, silver from Laurion, philosophers and schools, etc.).
3. How it changes the competition: what lever it gives rivals, what decision it forces on Athens, how it scales with Athens's lead.
4. Which existing systems it uses (cite game-rules sections), and what (if anything) would be new.
5. How the AI would play it on both sides (Athens and its rivals), since balance is measured on AI-only censuses.
6. The strongest case against it (exploits, griefing, dominance, feel-bad moments, hidden math).
Then rank them, and say which one you would ship and why.
