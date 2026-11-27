# Egypt: "The army eats Food": 3 Food per unit

Rule: each Egyptian land military unit (Scouts and aircraft aside) eats 3 Food a turn, shared across Egypt's cities
(total = 3 x units; each city pays total / cities, the first `total % cities` cities by id pay 1 more). Applied in
`economy.rs::nile` after the doubling, as the breakdown line "The Nile: the army eats Food". Keeps double Food, no
Production, Grow. A Food deficit starves as usual.

Starting point: this branch already has the sim-only experiment `egypt-army-food=N`, plus egypt-buy, egypt-war-food,
egypt-army-food-era and `offense_ladder --rules`). To ship as a real rule:
1. Make it the default for Egypt as a `FactionRule` (ablation `egypt_army_food`), bump RULES_VERSION, drop the unused experiments.
2. Player text: faction sheet drawback line ("Your army eats 3 Food per unit, shared by your cities."), Guide entry,
   city panel breakdown line already reads "The Nile: the army eats Food". Plain-language reviewer.
3. AI: Hard Egypt already sees it through yields; re-check egypt_peace_army (PR 322) still sensible.
4. Measure: 2,000-game all-Hard faction census (best/worst <= 1.5x target) + 300-game Egypt ladder (seed 5001: 26.0% with
   improved AI and city upkeep; base 47.3%).
5. Gates: tools/gates/check.py; reviewers: docs, plain-language, legatum-design-reviewer (core rules gate, PR 325),
   advisor if it teaches Egypt. Update legatum-rules factions.md and legatum-factions skill (chosen over:
   buy +50%, war Food x1.5, era-scaled). Delete this file before the PR merges.
