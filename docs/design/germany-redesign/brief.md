# Germany: what should its identity be? Design brief (2026-11-05)

## The problem
Germany is the weakest faction: 6.4% of seats win in the latest AI-only census (4,096 Hard games, 9 factions, fair share about 11%; Athens is best at 20.0%). It has been bottom or near it for weeks (8.1% on 10-22).

The designer has played Germany "quite a few" times and believes its signature strength is meaningless: "the germany advantage (ruhr foundry instead of factory) is extremely late game, to the point where I suspect it's meaningless." The census agrees on timing: Hard's Germany learns Industrialization at a median T141 (p25 T137) and only in 85% of seats; games end at a median T183 (p25 T167). So the payoff covers roughly the last quarter of the game, before the Foundry is even built, while the drawback runs the first three quarters. A rule-off ablation test (each rule switched off, paired census) is running now and will report which German rules move its win rate at all.

## What Germany has today (code-verified, game-rules faction table)
- **Ruhr Foundry** replaces the Factory (Industrialization tech, Industrial era, 5th of 7 eras): +10 Production instead of +5. Chosen by the designer on 2026-10-31: "give it a big late-game push with factory instead of an early game advantage." Before that it replaced the Ancient Workshop (+3 vs +2).
- **The first-building discount was removed on 2026-11-07.** Its 2026-11-05 ablation measured −0.38 pp [−1.34, +0.57], a dead strength.
- **Industry projects:** a city can run a continuous project instead of building: each turn 50% of its Production becomes Gold, or 25% becomes Science.
- **Drawback:** every unit has −1 movement until Industrialization (minimum 1). (A trial of "−1 move only outside its own land and roads" was considered.)
- **Goal, Five Ruhr Foundries (live):** own five cities with a Ruhr Foundry; +2 VP only while the count stays at five or more. Completed in 6% of seats, median T192.
- Faction sheet identity: "Nineteenth-century engineering and industrial cities." Counter listed: "Early raids."

## How Legatum designs factions (read before proposing)
- `CLAUDE.md` pillars, especially 8 (asymmetry with compensating drawbacks, TI4/StarCraft not Eclipse; a strength paid for by a weakness that pushes the faction further into its style; balance by deepening drawbacks or making weak identities pay off, never homogenizing), 10 (every strategy has a counter; expansion is the core loop, "tall" is not viable), 11 (competitive to the end), 2 (familiar to Civ 6 players), 5 (no hidden math).
- `.claude/skills/legatum-design/SKILL.md`: TI4-style rule breaks over number tweaks; mechanics follow from the story; bonuses must scale across eras (percentages, existing progression); bend existing systems rather than add new ones; every rule fits one plain sentence.
- `.claude/skills/legatum-factions/SKILL.md`: the method, all current designs, lessons, rejected ideas (e.g. "+50% unit cost until Steam" for Germany was rejected as arbitrary). New: every rule must earn its place, measured by an on/off census; a drawback must change the competition around the faction, never a table-wide effect.
- `.claude/skills/legatum-rules/references/game-rules.md`: how the game actually works. Read section 9 ("Systems that do NOT exist"): one order per unit or city per turn, simultaneous turns, no trade routes, no naval game to speak of. The tech tree is strict by era (you research only your current era's five techs).
- Factions are real peoples, grounded in real history, no fantasy. "Germany = Germany" (the whole sweep of German history is fair game, not only the 19th century).

## What we want
1. **Research.** Germany's history and what it is known for, across eras: the Holy Roman Empire's patchwork of cities, the Hanseatic League, guilds and master craftsmen, the printing press (Gutenberg), Prussia's army and general staff, railways and Moltke's mobilisation, Krupp and the Ruhr, Bismarck's social insurance, the Autobahn, engineering and precision manufacturing, chemistry, the Wirtschaftswunder, the Mittelstand, apprenticeships, etc. And how other games model an industrial or engineering civilization: Civ 6 Germany (Hansa, extra district), Civ 5 Germany, Civ 7, Humankind's industrial cultures and affinities, Age of Empires Teutons, Anno, Old World, Endless Legend/Space, Northgard, StarCraft's Terran or Protoss as asymmetric designs, TI4 factions with production identities, Factorio-likes, and any others that fit. Note what made those designs feel distinct and what players found weak or boring.
2. **6 to 10 candidate identities for Germany** (a strength plus the drawback that pays for it, or a whole reworked kit), each:
   - one plain sentence per rule, as a player reads it on the faction sheet;
   - the history behind it;
   - when in the game it starts paying (it must matter from the early or mid game: the late-only Ruhr Foundry is the problem);
   - how it changes the competition: what decision it forces on Germany, what lever rivals have against it, and how it pushes Germany further into its style;
   - which existing systems it uses (cite game-rules sections) and what would be new;
   - how the AI would play it (balance is measured on AI-only censuses);
   - the strongest case against it.
3. Rank them and say which one you would ship and why.
