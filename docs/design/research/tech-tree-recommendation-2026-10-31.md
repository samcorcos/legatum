# Tech tree pass: the lead's recommendation (2026-10-31)

Compares Astra's proposal (`.reviews/astra-tech-tree-astra.md`) and Fable's (`tech-tree-fable-2026-10-31.md`), both written independently from `tech-tree-brief-2026-10-31.md`. Base: Fable's tree, which changes less, uses existing systems and keeps the designer's own Future items. Four of Astra's ideas replace Fable's where they're better.

## Where they agree (take it)

| Change | Both say |
|---|---|
| Structure | Keep seven eras, five techs each, resource reveals |
| Philosophy | Keep the Great Library, add +1 Science in every city |
| Aircraft | A Future tech replaces Quantum Computing's "+4 Science per Lab" with a Fighter and Bomber upgrade (longer range, better guard) |
| Humanoid Robot | Replaced by a human infantry unit (Exoskeleton or Mechanized Infantry) |
| Fusion Power | Build the decided +5 Production; the Future wonder is a fusion reactor (ITER, real and under construction) |
| Horse techs | Chivalry and Military Science must give the Aztecs something |
| Masonry | Needs a payoff the Mongols can use before they're allowed Walls |
| Science at the end | No research VP; after the last tech Science becomes Gold at 50% (decided) |

## Where they differ, and the pick

| Question | Astra | Fable | Pick and why |
|---|---|---|---|
| Last tech (Recursive AI) | Rename to Networked Logistics: road moves +3 | Keep it; the whole map is revealed to your team and upgrades cost no Gold | **Fable.** Recursive AI was on the designer's own Future list ("superintelligence and recursive self-improvement"), and a revealed map plus a free re-equip is a climax everyone sees. Road +3 is one more number |
| Aircraft upgrade name and supply | Jet Fighter / Jet Bomber; needs Oil plus Silicon (a new dual-resource rule) | Stealth Fighter / Stealth Bomber; the upgrade switches the plane from Oil to Silicon, like every other era change | **Fable's supply** (an existing rule). **Stealth names**, since jets were flying by 1945 and our Modern era already covers them. Add one rule so the name means something: **only a Stealth Fighter can intercept a Stealth Bomber**. That makes the Future air war a real upgrade race, and it fits his 10-27 ruling that air superiority is the counter to Bombers |
| Hangars | 2 aircraft | 3 under the stealth tech | **Fable.** A Future airbase becomes a target worth a campaign |
| Medieval Aztec fix | Rename Chivalry to Military Organization; 25% cheaper upgrades, which must not stack with Military Engineering's 50% | Castle moves to Chivalry, Mill to Feudalism, Granary +1 Food to Crop Rotation | **Fable.** Three data moves, no interaction rule, and Castles and Knights are one historical package |
| Industrial Aztec fix (Military Science) | Land units cost 10% less Production | Units heal 30 a turn in own territory, not 20, with Sparta excepted | **Astra's effect, keeping the Civ name Military Science.** Cheaper armies suit the Aztecs, an attacking faction; the healing needs a Sparta exception, and hidden exceptions read as bugs |
| Printing Press | +1 Gold per Market | Techs a rival knows cost 10 points less | **Astra.** The diffusion discount is Byzantium's Scriptorium identity; giving it to everyone flattens their signature (pillar 8) |
| Where wonders sit | Hagia Sophia on Education, Royal Arsenal on Industrialization, Eiffel on Electricity | A wonder and a resource reveal never share a tech: Hagia Sophia on Machinery, Royal Arsenal on Ballistics, Eiffel on Steel | **Fable.** His favourite decision is "get the reveal first". If the wonder is on the reveal tech there's no choice, so separating them gives every era a reveal-or-wonder race |
| Future land units | Rename to Main Battle Tank and Drone Battery | Keep Combat Drone and Autonomous Armour | **Fable.** Military drones were on his list, they're real today, and keeping them means no new art |
| Code of Laws | Existing cities claim the ring-2 tile (S3, decided) | The same, optionally plus gating Alliances behind it | **S3 only.** Alliances were just reworked (S12); a diplomacy change doesn't belong in a tech pass |
| Air Defence building | — | Optional; a city intercepts one strike a turn | **Not now.** It cuts against his 10-27 ruling; revisit only if Bombers dominate in play |
| Other Astra additions | Currency +1 Gold, Writing Scouts +1 sight, Railways road +2 | Railways Roadworks reach 12 and Mines +1 | **Fable on Railways** (a payoff you feel). **Skip the Currency and Writing extras**: Market and Library already carry those techs |

## The one call for the designer

**Dropping Quantum Computing.** It was on his Future list (2026-10-10), as Humanoid Robots were before he cut them. Its "+4 Science per Lab" is the clearest placeholder in the tree, and the aircraft upgrade needs its slot. The alternative is a sixth Future tech, which breaks five per era. Recommendation: drop it.

## Build order (skateboard first)

1. **Skateboard:** data and rules only, with no new art; the new units reuse existing sprites and cards.
   - Future rebuild: Stealth Aircraft with its two upgrades, the stealth-only intercept and 3-plane hangars; Exoskeleton Infantry; Fusion's +5 Production with Factories and Labs freed from Coal and Oil; Recursive AI's map reveal and free upgrades.
   - Earlier eras: Philosophy +1 Science; the Medieval chain; Military Science −10%; Printing Press +1 Gold per Market.
   - Then gate it on the same-seed human-seat benchmark against master, as with the revived rules. Aircraft supply cost Hard about 10 points on 10-23, so watch that number.
2. **Scooter:** once `world-wonders-1031` lands, move the wonders to the techs above. Also: the Masonry Quarry step, Workshop +1 at Machinery, Railways reach and Steam Pumps, Factory +2 and Oil Well +1.
3. **Bicycle:** art for the Exoskeleton, the stealth aircraft cards, the four new wonders and the tech icons.

Rough size: about a day of agent time for rules and AI across the skateboard and scooter. Art is the slowest part, limited by image generation and review.

## Designer feedback (2026-11-01) and revised picks

Nothing is built until he agrees. His notes, then the lead's revised pick.

| Item | His note | Revised pick |
|---|---|---|
| Robotics unit | "these shouldnt be exoskeletons. at that point they shoudl be autonomous killer robots of some kind" | **Hunter-Killer**: an armed autonomous ground robot (four-legged, like today's armed quadrupeds; not humanoid) on the melee line; Infantry upgrades to it. With the Combat Drone and Autonomous Armour, the whole Future army is autonomous, one unit per class. Needs its own sprite |
| Aircraft tech | No real link between quantum and stealth | Quantum Computing is gone entirely; the tech is named for what it gives: **Stealth Technology** (Civ 6's name) |
| Recursive AI | Map reveal fine; free upgrades "feels odd... we need more of a rubber band, not a climax" | Keep the map reveal for the researcher. Replace free upgrades with **the leak**: once any empire knows Recursive AI, every empire that doesn't researches Future techs 50% faster. The leader gains the map but speeds everyone else's catch-up, so taking it is a real choice |
| Aircraft supply | "aircraft shoudl require oil. its a flaw in ai gameplay, not the oil requirement" | Modern aircraft keep Oil; the AI gets fixed (Codex `air-oil-ai`). Open: stealth planes on Oil (fuel, his rule) or Silicon (the Future era rule) |

### Second round (2026-11-01)

- Stealth aircraft use **Silicon**; units use their era's resource wherever possible.
- Recursive AI: the leak is rejected ("research tech shouldnt be a tradeoff like that"). It reveals the map to your team and gives **+20% Production in every city** (lead's number; a percentage so it scales).
- After the last tech: the player chooses what each city's Science becomes, at the decided 50%: **Gold** (to the treasury), **Food** or **Production** (in that city). Lead's view on a fourth option: Veterans (Science becomes unit experience) is the only one that isn't another yield; otherwise three is enough.
