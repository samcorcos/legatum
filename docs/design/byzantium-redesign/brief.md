# Byzantium: a rule break that makes sense, or delete it. Design brief (2026-11-06)

## The problem
The designer: "I'm just not sure the Byzantium faction makes sense... the rule break is weird and feels forced. Maybe we just delete Byzantium." He is open to a redesign if it is fun and unique; otherwise the faction is deleted. He liked the Theodosian Walls as a hook ("lol i didn't know that. im open to it").

## What Byzantium has today (code-verified; `.claude/skills/legatum-rules/references/game-rules.md`, faction table)
- **Mercenary Companies (the rule break):** hires any armed unit a met empire it is not at War with can field; pays the unit's Production cost in Gold to that seller; 30-turn contract, 2 Gold upkeep, outside the levy cap and strategic supply; can capture; unpaid Companies turn barbarian (`crates/core/src/company.rs`).
- **Thin levies (drawback):** at most one armed unit per city plus one.
- **Tagmatic Soldier:** the Pikeman and later melee line get +4 strength and +10 Production.
- **Goal, the Heir (held):** train and hold a 2 VP civilian; whoever holds him scores.
- The Imperial Scriptorium was cut today (a weaker copy of Athens's science identity).

Rule-off ablation (4,096 Hard AI-only games per arm, `docs/sim-results/faction-ablation/byzantium-2026-11-05/`): Companies are **backwards** (Byzantium wins 20.2% without them vs 11.3% with; the AI hires badly), thin levies **dead** (+0.8 pp; since 11-06 Hard's lean army equals the cap, so the AI never meets it), the Heir **pays** (−1.95 pp). Overall Byzantium wins about 11–14% (fair share about 11%).

## The lead's starting sketch (not binding)
Byzantium is famous for the Theodosian Walls of Constantinople (built 413; breached by land only in 1453 — the 1204 Crusaders came over the sea walls) and for diplomacy by Gold: tribute and subsidies to buy off enemies, playing rivals against each other (Huns, Avars, Rus, Pechenegs), Greek fire, a professional bureaucracy, the solidus, Belisarius's reconquest with small armies, the theme system, Basil II, survival for a thousand years against larger enemies. The sketch: an impregnable capital plus "pay Gold to end a war at once, even when the enemy doesn't want Peace, with a price that grows each time", paid for by a small army. Concern: Switzerland is already the Gold-and-defence faction (tolls, interest, Alpine Redoubt, 3 Gold upkeep). Any design must be clearly distinct from Switzerland.

## How Legatum designs factions (read before proposing)
- `CLAUDE.md` pillars, especially 8 (TI4/StarCraft asymmetry: one core rule broken, a strength paid for by a drawback that pushes the faction further into its style; never homogenize), 10 (every strategy has a counter; expansion is the core loop, "tall" is not viable), 11 (competitive to the end), 2 (familiar to Civ 6 players), 5 (no hidden math), 6 (AI decides better, never cheats).
- `.claude/skills/legatum-design/SKILL.md`, `.claude/skills/legatum-factions/SKILL.md` (method, every current faction, lessons, rejected ideas — e.g. "lifts all boats" drawbacks rejected; a drawback must change the competition around the faction; every rule must move win rate in an on/off census or it goes).
- `.claude/skills/legatum-rules/references/game-rules.md`: how the game actually works; read section 9 ("Systems that do NOT exist"). One order per unit or city per turn, simultaneous turns, diplomacy is only Peace / Alliance / War (Gold fee and upkeep for Alliances, a betrayal record), no trade routes, little naval play, strict era research. Walls now cost 100 Production; Walls count as garrison for Hard AI.
- Real history, no fantasy. Simple: each rule fits one plain sentence on the faction sheet. Prefer bending existing systems over new ones. The AI must be able to play it well (balance is measured on AI-only censuses).

## What we want
1. **Research.** Byzantine history across its whole span (what made it survive and what made it distinct), and how other games modelled Byzantium or similar civilizations (Civ 5/6 Byzantium, Civ 7, Humankind, Old World, Crusader Kings, Europa Universalis, Age of Empires II Byzantines, Total War, TI4 factions with diplomatic or defensive identities). What felt distinct, what players found weak or boring.
2. **6 to 10 candidate kits** (a rule break, the drawback that pays for it, and a goal; keep the Heir if it still fits), each with:
   - one plain sentence per rule, as the faction sheet would read;
   - the history behind it;
   - when in the game it starts paying;
   - how it changes the competition: the decision it forces on Byzantium, the lever rivals have against it, how it pushes Byzantium further into its style;
   - which existing systems it uses (cite game-rules sections) and what would be new;
   - how it differs from Switzerland (and every other faction);
   - how the AI would play it;
   - the strongest case against it.
3. Rank them, say which you would ship, and say plainly whether deleting Byzantium is better than your best kit.
