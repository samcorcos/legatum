# How the user reasons during play

The user narrates decisions mid-game. Each entry is a candidate AI behaviour: what was observed, then what was decided. The AI quality work turns these into switchable genes and measures them.

## 2026-10-07, playtest 8.4 (second game), turns 1–8, a food-hungry faction
- **Early scouting to identify neighbours:** "I got lucky with a few scouts early on. I found 2 of my neighbors."
- **The first thing checked is the neighbour's faction, to read the threat:** "the first thing i did was check which faction. both are peaceful so im unlikely to be rushed."
- **Defer commitments until the neighbourhood is known:** "need to see who else is around me (turn 8 so far) before I make decisions."
- **Play to the faction's needs and cover its weakness:**
  - "my faction also needs food, so im building food related things then settlers";
  - "all my tech focus optimizes for food and walls first because i need food and im weak against attacks, so i likely will need walls soon."
- **Research is planned as a queue that serves the plan,** not one tech at a time.
- **What the AI should do:**
  - read each discovered neighbour's faction as a threat prior, and update it as more are found;
  - keep the opening flexible until the neighbourhood is scouted;
  - weight production and research towards the faction's economic need and its known weakness (Walls early for a faction weak against attack).

## 2026-10-07, playtest 8.4 (second game, Japan), turn 17
- **Open land plus no threats means expanding hard:** "there does not appear to be a faction to my right which means a ton of room to expand. im going to build as many settlers as i can to build my hive ASAP."
- **Minimal military when no threat is seen:** "no real military threats, which means at most i might need to build some walls."
- **Keep a Gold reserve as emergency defence instead of a standing army:** "ill plan on keeping cash saved up to buy walls and a ranger if I get surprise attacked."
- **What the AI should do:**
  - scale its Settler push with the open land it has scouted and the absence of a threat;
  - when no threat is seen, swap a standing army for a Gold reserve sized to buy Walls plus one ranged defender in its most exposed city;
  - spend that reserve the turn a threat appears. This needs early warning: the sentries item.

## 2026-10-07, playtest 8.4 (second game, Japan): result
- **The user won, and it was more competitive than the earlier games:** "I had one big attack and the AI defended itself. they counter-attacked at some point and took one of my cities near the end."
- **The user's verdict:** "Overall better performance from the AI when it's a peaceful race to get the most points with few attacks." The AI holds up in a peaceful points race, and collapses against an early military player (the Sparta and Switzerland games).

## 2026-10-07 — Before playtest 8.5 (Mongols, Hard + Search)
"I should probably do at least a little bit of an early rush to take something because they have mobility advantage early." The human's read of Mongols is that early mobility makes a small opening rush correct before switching to a VP race.
Behaviour spec: a Mongols AI should weigh an early limited rush (one target city within reach of its +1 move units) before settling into expansion. Neighbours of a Mongols should treat it as a rush threat from the opening.

## 2026-10-07 — Playtest 8.5, Mongols, partial game (abandoned at turn 54 to restart)
"Overall that last partial playthrough felt good. AIs were behaving wisely. I took too much of a lead and they sought to balance it out."
This is the first playtest with the contest-the-leader AI, and the first time the user called the AIs wise. Protect it: turn the behaviour that balanced the lead into regression tests (analysis queued: `mongols-partial-analysis`).

## 2026-10-07 — Playtest 8.5, Mongols, won at T123 (10 VP, all five titles)
"Walls are a huge deterrent to rushes. That's kind of the point and they worked." The user isn't sure whether walls are overpowered, because the Sparta never attacked.
"Sparta really need to attack / rush before walls start going up because they have a hard time taking damage." When the user played Sparta (8.4), they overcame that easily and crushed the board.
The Sparta AI "built an army, left an unguarded settler (that I stole), and then did nothing", which gave the human time to expand, wall up and defend.
"The other AIs didn't really do a lot either... I basically just walked away with the game." Contrast this with the partial game just before, where the AIs balanced the lead.
"The Courier Hall is pretty cool."
Behaviour spec: an Sparta AI with an army advantage must strike a neighbour before its cities are walled, and must escort its Settlers.

## 2026-10-07 — Conquest is the fastest economy
"When I'm thinking about attacking, I'm thinking 'conquering these cities is the fastest way to grow my economy'. And I bet the math checks out on that. In Civ 6 deity, it's basically impossible to win unless you win an early war and conquer at least a few cities to grow your economy quickly."
Behaviour spec: the AI's plan valuation should count a captured city's yields, population and territory, and the rival's lost output, as economic growth that compounds. An early winnable war should then beat Economy whenever the math says so. Measure this; don't force it.

## 2026-10-07 — Celts's four-city expansion (playtest 8.4)
- The human opened Farm then Granary, queued three capital Settlers, and produced Settlers by T15, T24 and T35; no Gold was spent. A Celts unit captured the Athens Settler at T25 and founded Green Veil T26. Along with Thicket T1 and built Settler cities at T19 and T29, Celts had four cities by T30. Each new city was 4 hexes from the capital. Best AI Switzerland built one Soldier before its first Settler, then alternated Rangers and Settlers: cities at T23 and T38 were 4 and 12 hexes from its capital. No Gold was spent by either seat in T1–T40. Full per-turn queue, yields and treasury trace: `.reviews/human-fast-expansion.md`.
- What the AI should do: pipeline at least two Settlers after essential early food infrastructure when safe sites are open; hold discretionary military production until a real threat; evaluate reachable enemy Settlers as city-denial/founding opportunities; favour a nearby good site when a distant site adds many travel turns.
- Tests: on 16 fixed salts of the Celts map, mean AI cities >=3 at T30 and >=4 at T50 with survival/duel score no worse than baseline; on a replay-derived T25 exposed-Settler scenario, capture and found by T30 in >=80% of Hard+Search runs without losing a city; first two new-city distances median <=6 with site quality within 10% of baseline.

## 2026-10-07 — Walls as Sparta (recalling playtest 8.4)
"I was able to take walled cities as Sparta in the game that I won. It cost me more (I lost at least one unit), but it wasn't impossible. It was a strong deterrent though and forced me to change tactics in that game. If they were built early enough I would have had to end my attack."
Behaviour spec for the Sparta AI and proxy:
- A walled city is a costly target, not off-limits. Take it with a concentrated force and accept losses when the prize is worth it.
- Prefer unwalled cities, and strike before walls go up.
- Abort a campaign when walls finish early in a target's defence.

Design read: walls are working as intended, a strong deterrent that changes tactics without making cities invulnerable. Don't nerf them.

## 2026-10-08 — Winning is zero-sum
"Winning in this game is zero sum, so it's not just about you building your economy (which is important) it's also a benefit to you if your rival loses economy."
Behaviour spec: plan values must count damage to rivals' output (cities taken, tiles pillaged or denied, rivals' Production lost) alongside our own growth. Deep search measured Campaign costing the rival 9.1 Production a turn by T164.

## 2026-10-08 — Playtest (Switzerland, current build), turn 14
"I'm 14 turns in and I'm scouting the map. What I'm seeing is that all my neighbors are passive players: another Swiss, a Japan, and a Athenian."
- **Read the neighbours, then commit:** "I'm probably not vulnerable to early rush so I should start expanding with settlers ASAP."
  - Spec: scout early. Classify each neighbour by its faction's rush potential and what it's been seen doing. With no rush threat, go Settler-first.
- **Escort with spare Scouts:** "I can probably defend the settlers with some of my extra scouts that I got from the villages so they can't be poached."
  - Spec: use spare Scouts (from villages) as cheap Settler escorts against Scout raids.
- **Expand toward the constrained neighbour:** "I probably want to build towards japan because they have the hive situation where they can't really build too far away from their capital, so if I build towards them, I'm crowding them out, which benefits me in the zero sum scoring game."
  - Spec: when picking sites, weight sites that take land from a rival whose faction rules limit its settling range (Japan hive), because denying a rival land counts as our gain.
- **Later in the same game, a self-correction:** "I kind of messed up number 3... I built to the north. And as you can see, Japan built exactly where I should have built and now I'm crowded out myself. That was a mistake. This is recoverable, but you can see how that would have been better."
  - Lesson: contested land between two empires is a race. The side that settles toward the contested land first gets it. Settling away from a neighbour leaves it to them, and they crowd you instead.
  - Spec: score contested sites (those reachable by a rival soon) higher and settle them first. Defer safe, uncontested sites behind you until after the contested ones.

## 2026-10-08 — Switzerland, turn 57
"I've been building cities, poaching settlers (the AI is terrible at defending its settlers. This is embarrassing), and I beelined for markets to get more gold because that's my victory condition. Gold also gives me flexibility to not have to invest in an army or walls because I can always buy them in a pinch, which I've done a couple times now."
- **Settler raids keep working** against the AI, on the 8.5 build. Settler defence is the AI's most embarrassing weakness.
- **Gold as insurance:** a Switzerland economy (rush Markets) replaces a standing army and pre-built Walls. Keep a Gold reserve and buy defence only when a threat appears.
  - Spec for the Switzerland AI: keep army and Walls minimal, build Markets early, hold a reserve sized to buy a defender or Walls within a turn when a threat is seen, and spend on expansion.
  - This is the opposite of what the Hard AI does in the human's Switzerland seat: it walls up and carries military upkeep, scoring 0% on the 8.3 benchmark map.

## 2026-10-09 — Pick one strategy and commit
"These strategies are mutually exclusive. If you're building an army to conquer (Sparta, for example) you want to spend your cash quickly on units to win early combat. You want to rush Markets if you're Switzerland, but maybe not others... Taking the average of all strategies is going to lead to failure. You have to pick one and go with it."
"When you pick one and commit, that doesn't mean that you shouldn't change course if it's obviously not working. The name of the game is being flexible and adapting based on what you learn."

## 2026-10-09 — Playtest 8.6, Germany (Hard + Search, 4 players, Standard), opening plan
"I should get production as fast as possible and I'm going to win by building wonders to get VPs. So I'm going to rush tech that increases production, probably get walls relatively early if I have aggressive neighbors, and then rush to tech that lets me build for victory points."
Strategy commitment (Builder/Wonder archetype for Germany):
1. Production first: beeline Production techs and buildings.
2. Defence depends on the neighbours: get Walls early only if they're aggressive.
3. Beeline the techs that unlock VP buildings (Great Works, wonders) and win by building VP.
Behaviour spec: a Germany AI on a Builder plan should beeline Production, then the VP-unlocking techs, and gate Walls on the neighbours' scouted aggression.
- **Caveat (user):** "That doesn't mean that's the best or only strategy, that's just what I'm thinking and we'll see how it goes." This records one plan, not the answer.
- **Timing the faction goal** (Every City a Forge, an Engine in every city): "I'm not going to build that for a while. Need to settle more cities, get production, then go for that VP." Defer the faction goal until expansion and Production are in place, because the goal scales with the number of cities, so building Engines early is waste.
  - Spec: the Germany AI defers Engines until expansion slows and Production is high. This matches the benchmark finding that early goal spending hurts (goal rites, 2026-10-08).
- **Adapting on scouting:** "It looks like I have Sparta as a neighbor... so I should get walls soon. Just started that research. I should also probably move my scout near their border to see if they're planning to attack me."
  - Spec: when a rush-capable faction (Sparta) is revealed as a neighbour, (1) start the Walls tech promptly, without building Walls yet; (2) post a Scout as a sentry at their border to watch for massing, which gives early warning so Walls or defenders go up just in time. (The sentry gene exists; this ties it to the neighbour's faction.)
- **Outcome: the AI won (T88), with Athens on 10 VP and all five titles.** "The Athenian played well. They legitimately beat me with no gimmicks. They just outplayed me. I assumed they wouldn't build an army because they're science focused, so I focused on building production. They took one of my cities and snowballed with points and won. I over-indexed on settlers for early expansion because I didn't think they would attack me and I paid the price (which is good!)."
  - Lesson: playing against faction stereotypes works. A "science" faction that attacks a greedy neighbour punishes over-expansion. That's pillar 10, a counter to greedy expansion.

## 2026-10-09 — Playtest 8.6, Germany again, losing to Mongols (T125)
"I'm about to lose another game to Mongols playing as Germany. Technically I'm in second place now, but he has an army that's about to wipe me out and he has a bigger army, economy, population, etc. I'm basically screwed. And it doesn't look like my ally the Celts is going to open up another front."
Notes: the second AI win in a row (if it ends this way). The allied Celts AI not helping is a possible weakness: an ally should open a second front against a common enemy that's winning.

## 2026-10-09 — Tech progress feels stuck in the first age
"How many games advance to a meaningful level of technology? Basically none of the games I play made it past the basic part of the game. One of the most fun things about Civ is that you get all the different techs and feel progress there... Civ had several very clear eras of technology and you could feel that progress. This game feels stuck in the first 'age'. Definitely never made it to something like tanks."
Measured from the user's saves: winners end with 5–14 of 32 techs, mostly eras 1–2. Era 3 (Integration) has at most 1 tech, and era 4 (Inheritance) is reached only in the longest games (T150+).

## 2026-10-09 — Settler safety, the human way
"The AIs need to be much smarter about leaving settlers out for capture. You notice how when I play I almost never lose settlers? And also I keep them unguarded most of the time (but not all the time)? In recent games when I sensed that there might be hostile units in an area, I guarded them... Stealing a settler is a game-changing moment... If I get an early settler taken, I'm almost ready to call it game over because it's a big deal. Not only do I lose a city, but they gain one."
Behaviour spec: Settlers move unguarded by default. When hostile units are sensed nearby (visible enemy or rival military, or Scouts within reach of the route, or a rush-capable neighbour near the destination), guard the Settler or wait. Target: Settler losses near zero, as the human achieves, without slowing expansion. A lost Settler is a two-city swing, so price it that way in the risk model.

## 2026-10-09 — Sparta win at T84 by elimination (playtest 8.6)
"Once again, it was easy if you commit to all-out war and crushing all your neighbors. Maybe the AI is just not that good at fighting wars with Sparta? Something to do with the healing rules messing them up? Their win rate should be a lot higher."
Spec: an Sparta AI commits to serial all-out conquest and keeps wounded units fighting, because they heal by kills and captures and can't rest.

## 2026-10-10 — Kiting, and the counter to Mongols
"I kept finding myself getting kited by their ranged units. I attack, they retreat out of range and they get a free attack each time before I can fight back... high mobility cavalry will help counter them... If I were playing against Mongols I would build walls and cavalry. Walls stop city attacks and cavalry helps me chase down their units so I don't get kited."
Spec (AI counter-play): against a Mongols neighbour or a ranged-heavy army, build Walls for cities, and fast mounted units to run down ranged units.

## 2026-10-09 — Germany game, turn 170: war without fighting, research without choices
"I'm at turn 170. And honestly, I'm bored. The AIs are at war with me, but they aren't really attacking me... and I'm stuck at 7 VP. I am a few turns away from getting the 9th VP because that's the world wonder... but the rest of the gameplay is just 'press return and wait'. And all the science at this point is useless. I just pick whatever is closest to click and there's no longer a strategy for what I'm researching."
Spec (AI): an AI at war with a leading human must act on it (march, siege, raid), not sit in a declared war; a declared war with no engagement for N turns is a bug to measure in sims. Spec (design): research choices must matter late, so the era/research overhaul (legatum-design) is the fix, not faster science.

## 2026-10-11: Sparta on 8.8.1 (Hard), conquest worked despite early overreach
- "Even with the early-game penalty... for crowns and income penalty, I was still able to conquer pretty effectively."
- "The AIs performed better this time than they have in the past, but they still didn't take the threat seriously enough once I started winning battles."
- "It sounds like the AI when it plays Sparta doesn't commit to winning like I do."
- Lesson for the AI: once a rival starts winning battles, neighbours should treat it as the main threat (walls, armies to the front, coalitions). A Sparta-like attacker should commit to a winnable war and keep pressing, adapting only when the plan is clearly failing.
- **T56, conquered Germany:** "This was an expensive war, but I decided it was worth it because 1) taking the city healed my units 2) knocked Germany out of the game. It was expensive, and not easy, so I think it was handled well in the balancing." Sparta's capture-heal makes a costly assault worth pushing through, and eliminating a rival is its own payoff.
- **Next target, the last remaining rival:** "If the AI is smart, they should know that I'm obviously trying to win the game by conquering everyone so they'd be building walls and archers to defend themselves." A lone survivor facing a conqueror who has eliminated everyone else should go fully defensive: walls, ranged garrisons, and peace or tribute if possible.
- **What actually happened (trace to T46):** I replayed the save at its engine commit (0f917ec4) and re-ran every AI brain each turn. All 45 turns matched with 0 order mismatches.
  - He trained 2 Spearmen and a Market, then from T22 a Horseman and 2 more Spearmen. He declared on Germany at T25 and took Essen at T27.
  - Japan declared on him at T28 and Byzantium at T33. He took Kamakura at T37 and Japan's walled capital Nara at T43 (Japan eliminated), then Germany's capital at T55 (Germany eliminated).
  - **Every AI did notice him.** Germany and Japan made him their top threat at T28, the turn after his first capture (scores 89 and 69). Byzantium did so at T33 (score 76). All three moved to a Contain war plan against him.
  - **They lacked the means to respond:**
    - **Germany went bankrupt.** It was paying a crown's upkeep (2 Gold a turn) and lost 4 units to "your treasury can't pay your armies" on T32, T33, T38 and T42, while he was invading it. Its army fell from 60 to 3.
    - **The coalition never formed.** Japan and Byzantium proposed Alliances (T30, T32, T49), but each was refused because the 60 Gold signing fee was more than the other AI had (1 to 32 Gold).
    - **Japan flipped from Contain to Peace on T36.** That was a turn before he took Kamakura, because the plan charged a cost for being attacked back to a war plan and none to peace. It also spent 48 Gold on a tile instead of a defender and started a Brood Hall on T37.
    - **Germany proposed Peace the turn after his declaration.**
- **What the AI should do:** once a rival has ended an empire or taken cities and is at war with us, every city within its reach defends as if its army were at the door, with Walls and a shooter. Peace with an attacker that won't stop is not free, so a war plan shouldn't be the only option that carries the cost of being attacked. A coalition against a proven conqueror must be affordable, either through the AI saving for the fee or through the free-alliance rule covering this case. A Sparta AI commits to the war its army can win.

## 2026-10-17: 8.9.2 Byzantium (Hard), the Mongols' first competent attack (T149)
- "The Mongols had an actually competent attack... I was unprepared for such a large assault and they took one of my cities... it was not trivial to fight them off like it usually is. I actually lost a city that I tried to defend."
- This is the first time an AI offensive beat the user's defence. Keep it: the behaviour must not regress, and it should generalise to other AI factions.
- **What it was (traced 2026-10-17):** the user was Switzerland; the Mongols declared on Aarau (founded T77) on T78, sat at war for 58 turns with an army that grew from 10 to 17, then shot Aarau from 200 HP to 0 over T141 to T146 with three or four gunpowder-era shooters while the city had no garrison (Swiss army: one or two units; 2,235 Gold unspent), and walked in on T147. The garrison arrived on T144, too late. Locked in as `war-golden-horde` and `war-golden-horde-hold` (Hard attacker 43 to 50%); details in `docs/sim-results/hard-vs-human/war-scenarios/README.md`.
- **Spec (AI):** keep this behaviour; spread it to other factions' wars (`ai-quality/ideas.md` #15). **Spec (defence, for the user's side):** a walled border city with no garrison is the opening.

- **Game end, about T205+: the AI beat the user.** "The AI legitimately beat me! That's great. And it felt pretty competitive too. Whereas in Civ 6, the harder difficulties only beat you by cheating (having different starting conditions, bonuses, etc), it felt like I was just outplayed." This is the first playtest loss to Hard. It was the north star, reached without AI bonuses (pillar 6).

## 2026-10-17, 8.9.2, Sparta against 7 Hard AIs, T87 (in progress)
- The user feels he is "crushing everyone pretty easily" with an early-unit, keep-conquering Sparta. He says he isn't playing especially well and the AIs aren't doing especially badly; the strategy just works.
- His question is why more AIs don't take this strategy if it works this easily. His hypotheses:
  - the AI's Sparta doesn't spam units from turn 1;
  - AIs don't persist in conquering;
  - AIs don't keep wars going;
  - AIs can't coordinate units to win fights and take cities.
- Save: `~/LegatumRuns/bug-saves/p892-sparta-crush-t87.legatum`. The coach review goes in `docs/playtests/reviews/`.
- Follow-up, the same day: "it seems like the Sparta full-on attack and rush is a seriously dominant strategy... that doesn't necessarily mean that Sparta is overpowered... just that it seems the AI doesn't play optimally when it's Sparta, and other players don't defend themselves properly." He plans to replay the rush once to see if it repeats. His working theory has two AI gaps: the AI's Sparta under-plays the rush, and AI defenders don't defend against it. Faction strength is not his suspect.
## 2026-10-17: 8.9.2 Switzerland (Hard), won at T202
- "Just won the game as the swiss. It wasn't a walk in the park, but it wasn't too hard. I'd like the 'hard' setting to be something where I usually don't win (but the AI shouldn't cheat to make it extra hard specifically for me)."
- **What he did (from the tape, he didn't narrate it):** Granary, a Ranger, then three Settlers from the capital; Walls first in almost every new city, then Market and Granary; markets beelined for the era-2 objective (Markets in 4 cities, T74, 40 turns before any AI scored era 2); a small army (army strength 40 at T80 against 174 and 178 for the Mongols and Sparta) with Gold as insurance (13 purchases, mostly Walls, Libraries and emergency ranged units); 3 Settlers stolen, none lost; accepted every peace offer and turned down most alliance offers.
- **His view of the Swiss hoard (same day):** a big hoard is a fair trade-off because "it makes you a bigger target for attack (because you can capture that gold) and it means you haven't been spending your money on things".
- **His question about the rubber bands:** "A lot of why the AI might be more competitive is the rubber-band effect of weakening the top player for overextension, more than the AI actually getting better. How much of it came from the AI getting smarter, and how much from just making the person in the lead (me) weaker?" Every coach review now answers this with numbers.
- **What the AI should do:** treat an economic leader (public VP lead, Largest Economy, a public Power lead that already triggers decadence) as the main threat when he passes about 40% of the target, not 45–50%; go after the hoard where it lives (the capital), with one concentrated force rather than five token wars; and contest the hold.
- **What the replay showed (coach review, `docs/playtests/reviews/2026-10-17-8.9.2-sparta-t87.md`):** both of his gaps are real, but the first isn't unit count.
  - The AI's Sparta in his seat declares on T18 with 3 units to his 4. It takes 1.1 cities by T87 to his 7, because its first target is hard-coded to Japan, it makes peace after captures, and it doesn't storm unwalled cities.
  - AI defenders swapped half-built Walls for Rangers, and fielded one unit per city. The same defenders held against AI rushers from the same turns.
  - Where the counter was built (walled Tara, the Mongols' shooters at Sarai), he lost 10 units and stopped.
  - His own method: Hoplites only, no Settlers, the softest neighbour first, focus fire, and never accept peace.
- 2026-10-17: "The Byzantium rule break of forced peace feels off." The user asked for creative alternatives, researched from other games (TI4 and more) and history, and for a wider look at which core mechanics could be broken in a fun way. The research is in `docs/design/research/rule-breaks-2026-10-17.md`.

## 2026-10-18, 8.9.4, Sparta on a 4-player small map against Hard Mongols, Sparta and Athens
- He won by eliminating everyone at about T40: "really easy... I just spammed hoplites and ran the map with almost no real resistance." That was despite the 8.9.4 AI defence changes (`walls_under_way`, value targeting, coalitions). Save: `~/LegatumRuns/bug-saves/p894-sparta-win-t40-1308.legatum`. The coach review goes in `docs/playtests/reviews/`.

## 2026-10-18, 8.9.4, builder game (testing how the AI treats a leader)
- Mid-game: "I just looked down and noticed that the cities are pink now. That tells me that Sparta is on the war path, and I didn't realize they were near me before. That means I need to defend myself. I'm building walls and archers ASAP." His reflex against a rush neighbour is Walls plus Archers. The T40 coach review found Spearmen beat Archers against a Hoplite rush (Archers did nothing for Karakorum), so the game should teach that. He only noticed the Spartan neighbour late, from border colour, so first contact with a rush faction should be unmissable.
- His next move: send the Scout over to see what is going on. That is the scout-first reaction the design intends.
- Result: he won as Switzerland at T247. "It went well! I won and it felt reasonably competitive. The AI made some weird choices, but it didn't feel like it made completely stupid choices." Save: `~/LegatumRuns/bug-saves/p894-swiss-win-1621.legatum`.
- He reached the Future era for the first time. The game took about 3 h 10 min of wall clock (13:10 to 16:20 local, 247 turns, about 45 s a turn, including chat breaks); 8.9.4 has no per-turn timing yet. At the 150% objective scale that's about 3.5 to 4 h, inside the 3–6 h target, so keep 150%.
- His read (2026-10-18): "it seems that the AI is pretty decent at playing the game for points as it relates to economy... but maybe it's just not so good with war decisions, troop movements, and other strategic considerations... Japan attacking me when it had basically no army was strange." The lead's view: mostly agree. War is the weakest part (unwinnable declarations, no follow-through, no counterattacks, idle coalitions), but the economy is also behind a strong human (he had about 1.5x the best AI's economy at T100 on 8.9.2, and expanded faster); it's just less dramatic. He paused playtesting until the Engineer build (8.9.5) lands.
- **What he did (from the tape, coach review `docs/playtests/reviews/2026-10-18-8.9.4-switzerland.md`):** a slow builder start (4 cities at T100, 0 VP until T131) with three Settlers bought with Gold (T80, T98, T128); Walls in five cities by T140; refused Japan's peace three times after it attacked (T145, T152, T159) and took Sapporo and Okayama, eliminating Japan; answered the AI Swiss call to arms against Sparta (T159); bought four emergency units during the coalition war (T163–T190) instead of standing an army; offered the Mongols peace twice (T162, T193) and accepted every other peace; hoarded to 2,869 Gold, then bought the Relay Uplink (1,050) and two Tanks (1,798) at T227, which cost him the reserve goal for ten turns.

## 2026-10-18, 8.9.5, Athens against 7 Hard AIs (a loss)
- He lost to Sparta: "Sparta started conquering and out-maneuvered me and I wasn't paying enough attention." He was set back by the Bomber not working (a big investment; the fix is in progress). "I had a hard time figuring out how to get more points, but that second part is fine for now." Overall: "that game felt pretty reasonable." He'll try Athens again. First loss to the AI since the 8.9.2 Mongol game.
- **What the save shows** (coach review `docs/playtests/reviews/2026-10-18-8.9.5-athens.md`):
  - **The race:** he was a builder with the Science lead all game (Most Advanced crown from T91) and 9 cities by T140, and lost no city. He sat at 5 VP from T150 to T230 with 1,000–1,913 Gold and 2 military units. He bought 3 items in 230 turns (a Marksman, the Bomber at T166 for about 780 Gold, a Drone). He never used Share the Learning.
  - **His wars:** he declared on Japan at T170 to use the Bomber and proposed peace two turns later. He joined Japan's and Germany's alliances against Sparta at T223–T224, three turns into Sparta's hold.
  - **"Out-maneuvered" meant outscored, not out-fought:** Sparta's war on him (T219) gave up in a turn. Sparta's points came from eating the Mongols.
  - **The open points he didn't find:** Share the Learning (+2), Banks in 6 cities (two eras), 9 more tiles (two eras), and the kill objectives his Bomber was built for.
- **Lesson for the game:** "How you win" should list the next VP with costs, across every open era, and name the faction goal as one order (UI proposal in the review).
- **Lesson for the AI:** an empire that fights for the leader's prey slows a conqueror more than a coalition that only declares.

## 2026-10-19: won as Athens on 8.9.6, "honestly pretty easy"
- "It was honestly pretty easy... there were a couple small wars where i briefly lost some territory, but it wasn't that hard. could have something to do with the recent changes and the AI not being as good with the latest updates."
- Earlier in the same game: the triangle felt wrong once Tanks arrived (infantry shouldn't beat tanks; tanks should move further; bigger late-era jumps), and the Bomber's 5-tile range was too short to reach a city two away. Both are now decided (decisions log 2026-10-19).
- Save: `~/LegatumRuns/bug-saves/p896-athens-win-1019-1800.legatum` (autosave, turn 234). Coach review queued for Tuesday.

## 2026-10-19: the Mongols took one of his cities around T180 (8.9.6), "well played by the ai"
- "right around turn 180, the mongols attacked me and took one of my cities... it was a smart move because i was winning, I had a big gold pile, and it was right after my units were injured so he killed some of my units and took the city. not sure if that was intentionally smart or an accident, but it was well played by the ai."
- What he read as smart: hitting the leader, when his army was wounded, with a city to take. Worth checking whether the plan layer chose it on purpose (runaway read, value targeting, the wounded defenders in the forecast) so we keep it.

## 2026-10-19: won as the Swiss (8.9.6) while Japan led most of the game
- "japan definitely should have won. they were *way* ahead of me for pretty much the whole game, but seemingly couldn't convert that into a victory... i suspect if we replay that game from turn 100 on as Japan with a different strategy, japan should crush, but im not sure."
- Same game: the Mongols took a city of his around T180 right after his units were wounded ("well played"), and the city attack forecast showed only the walls, not the garrison (T187, Lugano).

## 2026-10-20: what the two 8.9.6 saves show (coach reviews)
- **Athens (`docs/playtests/reviews/2026-10-20-8.9.6-athens.md`):** his read that 8.9.6 weakened the AI is right in one way. No AI built a Military Engineer in 233 turns, so his walled, mostly empty cities held. Turning on `walls_as_cost` from his T200 position would have beaten his own line in 30 of 30 games. He also applied the 8.9.5 lesson: after the same 5 VP plateau (T166–T209) he took Castles, 29 kills with 65 Bomber strikes, Mills and Share the Learning as the closer (T224). One Gold purchase all game. Hard in his seat from T120 won 30 of 30, so the position was already won.
- **Switzerland (`docs/playtests/reviews/2026-10-20-8.9.6-switzerland.md`):** "Japan should have won" is right only with the rubber bands off. Japan lost 39% of its output to decadence (35–63%), and with decadence off it wins 24–30 of 30 from any turn T100–T196. None of the different strategies for Japan worked under the bands (0–4 of 30), apart from containing him from T150 (his line 19 → 0 of 30, though Athens won most of those). On VP Japan led only T159–T219, and at 9 VP it was 2 tiles short of 10. His own play beat Hard's from his seat (Hard 2–5 of 30). The Mongol T179 attack was a deliberate value-scored strike on a just-captured Lugano at 0 HP with a wounded garrison, not a reaction to his lead.
- **His method:** he bought 4 Military Engineers and took walled Lugano from Sparta with them. The AI never builds Engineers.


## 2026-10-22: "Japan's Century" scenario run (Japan from T100 of the 8.9.6 Swiss game)
- Score 1240 (Hard baseline median 1060, max 1320, no Hard run won): +6 VP, 12 cities, all 8 starting cities held. "I was definitely about to win. I just had a couple units to kill, get my 2 VP, then I would win the game." He played it to test his earlier hunch that Japan "should have won" the Swiss game; this run informs the decadence decision. Coach review queued (`coach-japan`).
- **What the run shows (coach review `docs/playtests/reviews/2026-10-22-japans-century.md`):** he kept and grew rather than fought. He declared no war (offered peace 10 times), lost 2 cities in 150 turns against Hard Japan's 7.3 from the same start, grew Food from 189 to 347 and reached 12 cities and 109 population, so the Modern (89 population) and Future (12 cities) objectives scored the turn he entered each era; one 144-tile count scored two eras at T166. He took the Athenian capital at 20 HP (T171, held to T193). He closed the country late (T217), so the vow was his last 2 VP; at T250 six German and Athenian units stood in his land with four empires at War with him. From that position Hard in his seat reaches 10 VP next turn every time and loses the hold within 1–2 turns every time (wins 14 of 100 eventually). He paid 44–52% decadence (49% of his Production, Gold and Science), the most anyone has paid, and played on permanent VP that decadence doesn't touch. About 51 minutes of active play for 148 turns (median 14 s a turn) over three sittings.

## 2026-10-22: strategic supply doesn't create any pull (8.9.7, Celts, half-way)
- "I'm about half-way through another game now and to be honest, I haven't once thought 'oh, I need to get <resource x>'. and it didn't occur to me last game either. a big part of civ 6 is fighting over strategic and luxury resources. this game doesn't have that. it just kind of happens and you can play the game all the way through without even thinking about it."
- Also agreed: Siege as a Gold upgrade that uses 1 Iron ("that sounds fine").

## 2026-10-22: won as the Celts on 8.9.7 (4 players), "crushed everyone"
- "I just played a game as the celts and crushed everyone. I'm not sure the advantage or disadvantage of the celts made much of a difference. I played a pretty standard game." The AI Celts win 2.4% of balanced 8-empire games. Save `~/LegatumRuns/bug-saves/p897-celts-win-1022-2028.legatum` (T250). Coach review running (`coach-celts`).
- Same day: "I seem to have no problem winning no matter the faction... so I suspect the AI is just not very good, and the reason byzantium keeps winning is because of some sort of rules gimmick that the AI can't figure out rather than it being actually overpowered."
- **What the save shows (coach review `docs/playtests/reviews/2026-10-22-8.9.7-celts.md`):** a builder game with one war. He settled 12 cities (11 Settlers, six cities after T100), queued Walls from T11 and walled them all, kept an army of about 135 from T70 to T115, and lost no city and 7 units all game. He scored the Great Work first (T161) and four era objectives late, from eras he had entered long before (Market ×9 at T190, 155 turns after entering era 2; Walls ×11, Bank ×7, Forge ×7). He accepted Sparta's Alliance against Japan while Japan was at War with him (T193–T203), let Japan's army bleed from 490 to 119 against his walls and Field Cannons, took Japan's peace (T206), then declared on Japan six turns later with three bought Bombers and sacked five cities including the capital (T214–T232, 1,813 Gold). He raised the Warbands once, after his last city (T222). He sat on 1,500–4,700 Gold for 60 turns and bought 12 items all game. 52 minutes of active play, one sitting.
- **His read that the faction didn't matter is right for 200 turns:** Celtic land wounded invaders for 132 HP all game and his hidden archers fired 42 shots; only the sacks mattered. The AI Celts never attack a city at all (a hard-coded gate), which is most of their 2.4%; with every Celtic rule removed the AI Celts still win 2.0%.
- Same day, after the Celts review: remove the Celtic homeland damage and the hidden-hunter rule ("does not appear to be meaningful"), keep the sack, and propose new Celtic rule breaks. Raise the Warbands is fine: "it's easy to counter, but people just don't do it. all someone has to do is capture an encampment." On the AI: "your analysis... suggests you have a bunch of hard-coded rules and you're not iterating or exploring them. That's probably not good. Also, you seem unable to replicate even my level of success. I feel like what we need is a hill climbing exercise where you iterate on the AIs on our 64 cpu box until they at least get to the performance that I got to in terms of points and stuff and win percentage, then use that winning model to compete against the other winning models."

## 2026-10-25, 8.9.10 Sparta game: strategic resources finally create pull
- "this is the first game where I actually thought to myself, 'man, I need to find some niter' so that's good!" Era resources now drive his decisions: Niter for the Renaissance, Iron for Siege. This is the pull that was missing on 10-22 ("I haven't once thought 'oh, I need to get resource X'").
- Friction: only the current era's resource (Coal) shows in the top bar, so he struggled to see his Iron when a Siege upgrade needed it.

## 2026-10-25, 8.9.10 Athens game: won again; the AI still can't fight
- "it was pretty competitive with Germany, especially near the end there. but i ultimately won because the AIs don't know how to fight wars. switzerland declared war but was tepid in attacking me even though they had a huge army at my border. i have no idea why. they really just kind of let me win at the end."
- Second game in a day with the same verdict as the Sparta game: the gap is war (attacking with a big army it already has) and stopping the leader at the end, not the economy. Analysis: Astra task `analyze-athens-game-1025` (save `~/LegatumRuns/bug-saves/athens-win-swiss-tepid-20261025.legatum`).
- Why the Swiss were tepid (war suite, 2026-10-27, `war-swiss-tepid-t180-replay`): replaying his own Athens orders from T180 (a College and Workshops, almost nothing on defence), the Swiss AI took Athens in 0 of 64 runs with or without the new campaign planner. Its Spearmen and Archers can't bring a walled 200-HP city to 0 before his unit arrives. The huge army at his border was mostly obsolete, so the stall was army quality first (the target of 8.9.13's army modernization) and operations second.
- How he'd decide a war (same day): weigh what armies each side and its allies can actually commit; an enemy busy across the map is weak where you are, so poach one city and take a peace deal. The AI should reason the same way (Astra task `ai-local-opportunism`, after the Athens analysis).

## 2026-10-26, 8.9.11 Japan game: won with a late bomber war
- "germany was playing pretty well, honestly. i was getting worried they might beat me to 10 vp, but i declared war on them and took them over quickly. I was able to get a few bombers and just dominate everyone. not sure why nobody else did the same or attempted to counter it... I don't think the AI has any concept of how to handle that stage of combat."
- Germany was a real threat (good sign for the economy AI). He won by a decisive war on the leader when behind, using bombers nobody countered. Analysis: Astra task `analyze-japan-air-war-1026` (save `~/LegatumRuns/bug-saves/japan-win-bombers-20261026.legatum`).
- 2026-10-26, 8.9.12 (Switzerland): an enemy army of Hoplites and Archers stood fortified in a ring along his border around Chur without attacking. "they are just circling me but not attacking... feels like a behavioral bug." Same as the Swiss army in the Athens game: staging becomes a permanent destination. Task `fix-staging-ring` (save `staging-ring-chur-20261026.legatum`).
- Same game: the AI keeps building and fielding Ancient units (Archers, Hoplites) while he has Field Cannons. "they're building a lot of weak units for some reason and not upgrading them... it's not going to work." Units should be valued by combat strength against the enemies they'll actually meet. Task `ai-army-modernization`.

## 2026-10-27, 8.9.13 Athens game: crushed everyone, "it was easy"
- Setup: 5 empires (Athens, Byzantium, Sparta, Celts, Mongols), every AI Hard (checked in the save), game to T178. Save: `~/LegatumRuns/bug-saves/athens-win-bombers-20261027.legatum`.
- His plan, in his words: scout; found Sparta right next door, so "change of plans, get walls and archers"; defended against Sparta, then expanded rapidly; "since im athens, rush science, and specifically rush to get bombers before anyone else"; meanwhile prioritized buildings that earn VP; "get a bomber, take over cities with VPs."
- His read: "a pretty reasonable and expected strategy for athens. Athens is a late game bloomer, so late game tech is what saves them. Athens (by flavor) should be thinking 'i just need to survive until the late game'. the AIs were not aggressive enough and I just walked all over them."
- Second bomber-led win in a row (after Japan 10-26): the air stage is still the AI's blind spot, and early aggression (Sparta next door) didn't punish a walls-and-archers opening.

- What the save shows (coach review, `docs/playtests/2026-10-27-athens-bombers-review.md`): "walls and archers" meant Walls first in every city and exactly one Archer; he never had more than 3 military units, and at his hold 14 of 16 cities had nobody inside. He played it in 27 active minutes with about 350 orders, leaning on automation. "Take over cities with VPs" was literal: the Mongol and Celtic Great Work cities (Sarai T165, Bibracte T171) gave 4 of his last 8 VP, and Share the Learning gave 2 more. He trusted that the AI would not punish a tiny army, and it didn't (one AI attack on his cities all game).

## 2026-10-27, 8.9.13 Sparta game: about to win at T126, "really easy"
- Setup: 5 empires on a 48x32 map (Sparta, Germany, Japan, Switzerland, Athens), every AI Hard (checked in the save). He stopped at T126 with the win in hand. Save: `~/LegatumRuns/bug-saves/sparta-win-20261027.legatum`.
- His words: "im about to win this game with sparta -- no reason for me to contineu playing... it was also really easy. the ai still sucks at fighting wars."
- Second easy win on 8.9.13 in one evening (after Athens), this time with a war faction, so the war gap shows on both sides: the AI can't attack him (Athens) and can't defend against him (Sparta).
- What the save shows (coach review, `docs/playtests/2026-10-27-sparta-review.md`): six Hoplites queued on turn 2, one city until his first conquest at T33, then Switzerland (T30), Japan (T42), Germany (it declared, T45) and Athens (T46) in turn; Siege on four Hoplites T50–T63 so Walls stopped mattering; from T70 he ran the war with two units and left all his cities empty, trusting that the AI would not counterattack, and it didn't. 24 active minutes, 460 orders.

## 2026-10-28, 8.9.15 Swiss game: "won another game pretty easily"
- Setup: 4 empires on a 40x26 map (Switzerland, Japan, Sparta, Mongols), every AI Hard. Won T180, 10 VP to 2. Save: `~/LegatumRuns/bug-saves/swiss-win-2026-10-28/autosave.legatum`.
- The designer's words: "won another game pretty easily". Asked about Swiss balance during the review: "i don't think the swiss are overpowered at all. it feels like i win whether I play a defensive faction playing defense or whether i play an offensive faction like sparta and just go offense all game, i still win every time quite easily."
- So the target is faction-independent: Athens (defend, then bombers), Sparta (all-out offence) and Switzerland (hoard, then one war) all won easily on 8.9.13 to 8.9.15. Reviews should rank AI fixes by their effect across factions, not by the matchup.
- What the save shows (coach review, `docs/playtests/2026-10-28-swiss-review.md`): Walls first in every city and 1 or 2 military units for up to eight cities from T30 to T137, trusting the AI not to attack an empty empire (third game in a row). When an AI army walked at a town he bought one defender there (Chur T138, Solothurn T159), and both Mongol campaigns quit within two turns. Then one short war on the weakest neighbour: 3 Field Cannons and 3 Riflemen took all five Spartan cities and its Great Work (T153 to T171), refusing three Spartan Peace offers. 28 active minutes, 405 orders.

## 2026-10-30, 8.9.17 Mongols game: "i just ran away with it easily"
- Setup: 5 empires on a 48x32 map (Mongols, Byzantium, Celts, Athens, Switzerland), every AI Hard. He won T172 by holding 13 of 10 VP for 10 turns. Save: `~/LegatumRuns/bug-saves/mongols-win-2026-10-30/autosave.legatum`.
- His words: "ok just finished the game as the mongols... i just ran away with it easily."
- Fourth easy win in a row on the newest AI, and the first on 8.9.17, which was built to punish a rich, empty leader. At T96 he had 10 cities, 5 units and 1,432 Gold, and at the end 18 cities, 7 units and 2,165 Gold. He never lost a city. Switzerland (at war with him from T56) and Byzantium (from T86) took nothing. He took four Swiss cities from T157 to T168.
- Why the AI didn't punish him (hollow-attack trace, `docs/learnings.md` 2026-10-30):
  - the Swiss wanted the war but garrisoned 5 of 6 units at home;
  - a defender check stopped all their buying, so their Gold piled up to 3,257;
  - their banked settings never allow a walled city to be taken;
  - Byzantium was busy elsewhere and blocked by Swiss land.

  Coach review pending (`docs/playtests/2026-10-30-mongols-review.md`).

## 2026-10-30, 8.9.17 Mongols game: "i just ran away with it easily"
- Setup: 5 empires on a 48x32 map (Mongols, Byzantium, Celts, Athens, Switzerland), every AI Hard. Won T172, 13 VP to 5, holding from T163. 18 cities at the end, never lost one. Saves: `~/LegatumRuns/bug-saves/mongols-win-2026-10-30/autosave.legatum` (T96 to the end) and `mongols-hollow-8917-20261030.legatum` (T1 to T95).
- His question during the review: "how does my build order compare to the AIs? ... I win on population, science, gold ... I don't understand how that's possible."
- What the save shows (coach review, `docs/playtests/2026-10-30-mongols-review.md`): in the first 60 turns his build was like theirs (2 military units against 2 to 4; everyone 3 to 4 Settlers). The AIs built Walls and Quarries; he built Markets, Farms and Mines. The gap opened after T50 in cities: 10 at T96 against 4 to 6, from 12 trained and 3 bought Settlers. He paid 76 Gold of unit upkeep in 100 turns against the AIs' 225 to 405, and their armies never attacked him. He kept 3 to 6 military units for 10 to 14 cities, most of them empty (fourth game in a row). Then one war on the weakest neighbour with a public Great Work: two bought Bombers and a Field Cannon took four Swiss cities, including the capital and its Great Library, T157 to T168. 43 active minutes, about 570 orders.

## 2026-10-31, 8.9.17 Sparta game (in progress, T29): "they actually build walls and an army"
- Setup: Sparta (him), Germany, Switzerland, Japan and Byzantium, every AI Hard. Snapshot copied at T29.
- His words: "i thought I was going to easily roll into switzerland and take their city, but then I noticed that actually build walls and an army -- maybe even because they noticed that sparta (me) was right next to them and they maybe said '[uh oh], im probably goign to get attacked' and they defended themselves."
- His read is right. The Swiss AI's memory shows adaptation `DefendedExpand`, triggered by its assessment of him:
  - Sparta `rush_capable`;
  - nearest city 5 hexes away;
  - one war already declared.
  - Basel (pop 3) had a Granary and Walls by T29, with 2 Spearmen and an Archer; Swiss army strength rose 21 to 33 while his was 71.
- First time he has called out an AI reacting sensibly to a threat. It's the behaviour pillar 10 wants: an opening is a bet, and a neighbour adapts to it.

## 2026-10-31, Playtest 8.9.20, turn 52: the Settler snipe on the Mongols
- He saw an unescorted Mongol Settler next to his unit, declared war and ordered the capture, reasoning "if the units are adjacent and I say to capture, I get to capture."
- It escaped: moving Mongol civilians were exempt from the capture pin (part of slip-past). Nothing on screen said so.
- Reaction: "[what?] that's dumb. please fix." He still values Settler sniping as the counter to greedy expansion (2026-10-04), so the AI-side lesson is that escorting matters, not that Settlers should be slippery.

## 2026-10-31, Playtest 8.9.21 (Aztecs), turns 22 to 26: a stolen Settler and a surprise Peace
- T22 he declared war on Byzantium to take an unescorted Settler with his Scout, 2 hexes away. A barbarian Soldier next to it took it first; his Scout stopped. "I would have expected a battle to happen at the least." Expectation: whoever reaches a prize first or fights for it, never a silent bounce.
- T24 the Mongols allied with Athens (whom he'd attacked on T19) and joined the war; he read it as "the Mongols declared war on me". T26 he accepted Athens's Peace and the Mongols' war ended too: "im confused". Peace across a whole Alliance is invisible unless the card names every war it ends.

## 2026-10-31, Playtest 8.9.21 (Aztecs), turns 84 to 89: Switzerland retakes its city
- He took Swiss city 27 on T84 and lost it, with most of his army, on T89. His verdict: "sucks for me, but that's a good sign... usually they suck." First Hard counterattack he has called competent.
- What the Swiss did (tape): a retake campaign on the lost city from T85; a Composite Archer shooting the same Phalangite every turn; Spearmen and a Horseman feeding in; two Cataphracts arriving T88, one charging his Crossbowman for 77 damage (mounted +40% against ranged), then both finishing his last wounded Phalangite and retaking the city T89. Losses 5 for 5, but the Swiss got the city back.
- Likely enablers (all landed 2026-10-31, 8.9.19 to 8.9.20): `strike_kill` (a unit that can kill a wounded field unit alone attacks instead of staging), `capture_hold`, and the mounted rebalance (riders 130% of the melee line, class bonus +40%), which made Cataphracts a real answer to ranged units.

## 2026-11-01, Playtest 8.9.22: the Celts open a second front
- Mid-game, at war with Switzerland, he found the Celts had opened a second front against him: "the celts have done a good job opening up a second front in a war against me and it looks like they will probably make some progress, so that's good. that means the AI is getting smarter."
- His plan: "I'm likely going to have to end my war against switzerland soon and shift my focus back to my western front after i lose a city or two." The AI forced him to choose between fronts, which is the pressure pillar 10 and the "Hard = human" goal want: punishing a player who is committed elsewhere.
- Second positive verdict on Hard's war play in two days (after the Swiss retake on 2026-10-31). Worth a coach review of the save once the game ends: did the Celts time the war to his Swiss commitment (`second_front_temperament`, coalition logic), and does the sack (burning his cities) cost them VP they could have held?
- What the save shows (coach review, `docs/playtests/reviews/2026-11-01-8.9.22-aztecs.md`): he won on T181, 11 VP to the Celts' 8. He never made Peace with Switzerland: he finished it instead (five cities and its Great Library, T147 to T166), refusing three Swiss Peace offers. His "lose a city or two" came true five times over, but only small northern towns: the Celts sacked Chalco, Tlatelolco (twice), Colony 10 and Emain Macha. The Celts timed the war to the coalition, not to his Swiss commitment: they declared on T130, the same turn as Switzerland. Then their plan went quiet for 17 turns (T149 to T165), exactly while his army was in the south, because after burning the two Aztec towns they had seen they knew no other Aztec city. They broke his first hold through the Strongest Army crown (T170).
- Flower Wars (after the game): "ranged units don't take slaves and bombers that kill units also obviously don't take them either." All 16 Flower Wars events in the game had an Aztec melee unit taking part. The one he likely noticed (T168, Tlacopan) was a shared kill: a Bomber and a Field Cannon did most of the damage and a Rifleman took part. He agreed shared and defensive melee kills count.
- 62 active minutes in one session, about 840 orders. Expansion again (20 cities at the end against the best AI's 7) with 4 to 9 ground military units; the closing VP were two captured Great Works, the fifth game in a row.

## 2026-11-01, Playtest 8.9.22: a second Aztec win, "pretty easy"
- After the close morning win (11 to 8, the Celts broke a hold), his second Aztec game of the day: "i just played a game as aztecs and crushed. it was pretty easy." He had to hunt for the New Fire button ("it was hard to find").
- Two easy-to-close Aztec wins in a row raise the question whether the Aztecs (Flower Wars population on melee kills plus the 30-population New Fire) snowball for a player who fights constantly. Coach review asked to separate faction strength from AI weakness (save: `~/LegatumData/playtests/8.9.22-aztecs-win2-20261101-1441/`).
- What the save shows (coach review, `docs/playtests/reviews/2026-11-01-8.9.22-aztecs-2.md`): he won T153, 11 VP to 4, in 35 minutes (0.2 minutes a turn). His lead was expansion (9 cities at T100, 14 at T130, against 6 and 7) with 3–6 military units, at War with four AIs for most of the game; they hit his cities 13 times. Flower Wars gave 13 population; the New Fire (T144) was his closer and started the hold. He also took Byzantium's Heir with Corinth (T130, +2 VP). He declared only two wars (Switzerland T18, India T23) and eliminated both; the others declared on him.

## 2026-11-01, Playtest 8.9.23: Germany, and re-rolling starts for production
- Early in a Germany game: "it feels like as germany im already winning." Coach comparison against AI Germany queued (`docs/playtests/reviews/2026-11-01-8.9.23-germany-start.md`).
- **He re-rolls starts:** "when playing a faction that focuses a lot on war and production, I usually just re-roll the start position until I am next to a forrest and get a lot of production... Sometimes im in a plain with just food and no production, and there's basicalyl no way to win as sparta if that's what you're stuck with because you need production early to build military."
- Why it matters: start quality (`mapgen::site_quality`) adds Food and Production together, so a food-only plain scores like a forest start. The AI never re-rolls, so his re-rolls are an edge Hard doesn't get, and they inflate the human-seat comparison. He asked whether starts should be more balanced or lean toward what each faction wants.

## 2026-11-02, Playtest 8.9.24: an easy Swiss win
- "i just played a game as the swiss and crushed everyone easily. is that because the swiss are especially good? probably not, but it was easy to win."
- His plan in his words: "the early bonus for extra walls defense is helpful to keep me safe, and then i used my gold hoard to buy armies when it mattered." Turtle early behind the Swiss walls bonus, bank Gold, then convert the hoard into an army at the decisive moment (the playbook's "gold economy that buys the army" strategy).
- Context: AI-only Switzerland is second-worst on this build (6.9% of 6,000 games), so the AI plays the Swiss far below how he does. This is the first playtest with the new expansion and coalition AI genes. Save copy: `~/LegatumRuns/bug-saves/swiss-win-8924-20261102/`.
- What the save shows (coach review, `docs/playtests/reviews/2026-11-02-8.9.24-swiss.md`): he won T161, 12 VP to 4 (Germany), in 40 minutes. The walls and the hoard worked as he said, with a correction: Lugano, the capital, had no Walls until T63, and it was two bought Composite Archers (T58, T59) that stopped the Mongol raid (capital at 30 of 200 HP; skip either buy and it falls). His walls (Thun T40, Winterthur T45, Biel T50) ended later campaigns. He made only 7 purchases all game, ended with 7,387 Gold and 2 to 5 military units, and his decisive buy was one Bomber at T140 (Flight T139, nobody else had it). The edge he did not mention: at T41 a Soldier shadowed an unescorted Mongol Settler for three turns and he declared and took it (Biel T44), which held the Mongols at 3 cities for 40 turns; Hard from the T40 position wins 4%, from T42 46%.

## 2026-11-03, Playtest 8.9.25: the reworked Celts win "pretty easily"
- He won as the Celts against five Hard AIs on T159, 10 VP to 7 (Aztecs), and called it "pretty easily", his second easy win in two days after the Swiss (8.9.24). From the same game: "since the barbarians are not hostile, the cities should not shoot at them... units should not wake up from sleep / fortify if a barbarian is nearby" (fix on `claude/celts-barb-peace`).
- **What the save shows** (coach review, `docs/playtests/reviews/2026-11-03-8.9.25-celts.md`): his first game in which an AI out-built him. The Aztecs had 12 cities to his 10, four times his army and all three crowns, and led the VP race until T115. He won on conversion. He scored all five eras he entered with the cheapest card on offer (Capture 2 cities, Castle ×2, Bank ×2, University ×4, Gain 2 cities). He built Hagia Sophia (T104) and took Sparta's capital (T82). He called the Wild at T127, when it took him from 7 to 9 VP, and again three times, each time new cities lapsed the goal or a camp was cleared. His closing move was three bought Settlers (T136, T145) and four new cities for the Modern "Gain 2 cities" card. He kept 2 to 7 military units, fought only where he chose (Sparta T69–T82, and the Aztecs at his walls, about 15 Aztec units killed for none of his), and played 29 minutes, about 11 seconds a turn.
- Open questions for him (from the decision diff): why the Settler surge at T136/T145 (the objective, and a planned re-call?); why Sparta at T69 (Sellasia nearly empty, or the capital point and the Capture-2 card?); did he expect anyone to clear his unguarded camps during the hold?
- **His own observations (told to the lead before the review, held back from the coach as a test, 2026-11-03):**
  1. "i can use my +2vp faction thing as an advantage because they can almost act as a defensive shield since they don't attack me. But i didn't want to trigger it too early otherwise id get too many points and people would gang up on me. by the late game, the production cost of the space that the barbarians take up isn't a huge deal, which is fine." He timed Call the Wild late on purpose, to avoid scoring early and drawing a coalition, and used the camps as a shield.
  2. "the +2 bonus above is strong, but it's also easy to counter. it's not that hard to capture an encampment (but the AI never bothered to do so). They could have stopped me from winning at several points but they didnt." (This answers the coach's third question: he expected a camp to be cleared.)
  3. "my cities were basicalyl undefended for almost the entire game and nobody attacked me with a serious force for the whoel game."
  4. "the aztecs shoudl have beaten me but they didnt. their army was way bigger than mine by end of game and they didnt use it."
- **Canary grading (lead):** the coach found 2 (its top finding and top what-if: camp denial first takes his recorded orders from 32/32 to 0/32 wins) and 4 (four times his army, sent one unit at a time into his walls). It found 3 only in part: it saw no serious attack and two 0-HP cities left untaken, but described his defence as walls and ranged garrisons rather than "basically undefended". It found 1 in part: camps raid only his rivals and act as a free army (a design question), but it missed the shield use and his deliberate late timing, and instead asked him about the Wild's timing.
- **His answers to the coach's three questions (2026-11-03):**
  1. Settler surge T136/T145: "Yes, I was going for 'gain 2 cities' but also... for some reason, I had to build more than 2 cities to trigger it. then yes, call wild again." (Replay, 2026-11-03: he founded three cities in the era, T139, T140 and T147, and lost none. He entered Modern at the end of T139, and the era's count starts from a snapshot taken after that turn's orders, so the T139 city never counted; the card read 0/2 at T140 and scored at T147. Not the net count, as the lead first guessed.)
  2. Why Sparta at T69: "because it was undefended and they had a weak army. i considered the mongols too because they didn't have walls yet." He picks the war target by its visible defence (garrison, army, Walls), not by VP or objectives.
  3. Camps during the hold: "I assumed they would [clear them] and I would have to go back and forth for a while." He planned on re-calling the Wild against camp clearing, not on nobody trying.

## 2026-11-03, Playtest 8.9.25: India, stopped at T37
- Stopped at T37 (falling asleep): "i do think that im starting out in a position that practically guarantees im going to win. how often does the AI have a start like the one I had here? i assume not a lot. that's likely a problem with the AI. either 1) these kinds of starts are to be expected and the AI needs to get better or 2) something about the behavior of the AI made it easy for me to take advantage of." Early-game review asked to separate the start, his play and rival behaviour (`docs/playtests/reviews/2026-11-03-8.9.25-india-start.md`). Save copy: `~/LegatumRuns/bug-saves/8.9.25-india-unfinished-20261103-2100.legatum`.
- **What the save shows** (`docs/playtests/reviews/2026-11-03-8.9.25-india-start.md`): a good, roomy start (58th percentile quality, 94th room; Hard from T1 on his map 25%), not re-rolled (the archive has no abandoned India start before it). Two Soldiers first, then he parked them beside Germany's first Settler, declared T16, took it T17, took the Celtic Settler with a Scout T18 the turn he declared, and took the empty Leipzig T22 (Germany eliminated). Three of his five cities came from the AIs. He held the 160 Gold of wonder rewards unspent (305 at T37). Hard in his seat: 8% from T16, 56% from T22, 77% from T37.
- **Open questions for him:** whether the T13-T15 Soldier placement was a plan to snipe or a reaction to the empty capital; whether the Scout was hunting the Celtic Settler; why the Gold stayed unspent.

## 2026-11-05, Playtest 8.9.25: India "just crushed everyone"
- He won as India against four Hard AIs (Mongols, Byzantium, Sparta, Celts) on T177, 15 VP to 3, in 48 minutes: "i just crushed everyone" and India "almost feels overpowered if played correctly". AI India wins 8.7% of seats in the 4,096-game Hard census.
- **What the save shows** (coach review, `docs/playtests/reviews/2026-11-05-8.9.25-india.md`): 22 Settlers (AI India 7.5), Walls first in 16 of the 21 cities he founded, five ground units from T60 to T150 kept current by 22 upgrades, 2 units lost all game, no city lost while the Mongols and Byzantium were at War with him for 89 turns. Seven cities 14 to 18 hexes from the capital at 100% corruption (population 1, no output) still counted as territory for three era objectives. He never queued the Imperial Diwan. He accepted Peace from all four AIs at T130 to T140, then declared once, on the 8-VP Mongols at T149, with Bombers (first to Flight by 17 turns), and took the Heir, the Great Library city and the Mongol capital.
- **Open questions for him:** (1) why accept the Mongols' Peace at T130, which freed their army to eat the Celts and reach 8 VP; (2) whether skipping the Diwan was deliberate; (3) whether five units for twenty cities was a bet that the AIs would not attack, and what would have made him build more.
- **His answers (2026-11-05):**
  - **The T130 Peace:** "i was still building cities and didn't have a lot of troops to fight a war. i didn't realize they would eat the celts. once they took the capital I said '[uh oh], i need to do something about that'". He took Peace to keep expanding, without modelling what a freed Mongol army would do to a third empire; the leader threat only registered when a capital fell.
  - **The Diwan:** "not deliberate, I just didn't have time to get markets in all the cities." He skipped its prerequisites for lack of time, not by choice.
  - **Five units for twenty cities:** "it was a bet they wouldn't attack and that I wouldn't have to fight a two front war... the other bet is that I had so many cities that they could take one or two (maybe even 3) and it wouldn't affect me that much, so by the time they got to cities that mattered, i could shift production to get more units and fight back. I had buffer." **Width is his defence:** peripheral towns are an expendable buffer that buys the time to convert Production into an army; he values a frontier town far below a core city, and he prices a two-front war as unlikely.
  - **Frontier towns are walled buffers (2026-11-05):** "especially for india since the frontier towns are 100% corruption. they're basically buffer cities with walls that poke and delay enemy troops as they come through." A town that yields nothing still earns territory, holds objectives, shoots at passing armies from its Walls and costs the enemy turns, so its value is defensive depth, not output.
  - **Corruption:** "I barely even noticed the corruption penalty even though it was 100%. probably because the core of my economy was so developed by that point." (In AI hands, switching corruption off is worth +6.5 pp, so it binds the AI and not him: he puts his economy in the core and uses the far towns only for territory and objectives.)

## 2026-11-06, Playtest 8.9.25: the Mongol horde
- At T84, six empires, Hard: "i just wiped the map with them. extreme mobility, ranged or melee attack... they're extremely good... I suspect if the AI played the way I play them, the mongols will be a terrifying force to deal with." Germany out T28, India out T83, 13 cities, 6 VP to 2, three units lost all game.
- **His army is the win condition (his words, same day):** "if an AI came up to my cities with an army like the one i assembled with the mongols (lots of keshigs, highly experienced) I'd say '[we're in serious trouble]'. That army is basically enough to win the game." His plan from T84: "keep rolling up cities, going up through sparta... then at some point if i start losing units, I'll retreat for a minute until I have tanks then upgrade them all to tanks and start dominating again." He keeps a veteran stack alive and upgrades it through the eras (retreat to save experience, upgrade rather than replace).
- **What the save shows** (coach review, `docs/playtests/reviews/2026-11-06-8.9.25-mongols.md`): Horseman first (T10) and five by T35, Iron Working sixth (T31), four promoted Horsemen upgraded to Keshigs (T37–T55) who are his veterans now (92–120 XP), five more Keshigs built, all 11 Iron on Keshigs and one Siege, no Forge. Keshigs made 94 shots (59 from two tiles, 55 at walls) and 17 charges (12 into 0-HP cities); 5 of his 7 captures were a Keshig riding in. Targets were whoever had the least at the gates (Germany 1 unit, Dyrrachium 1 Spearman, India 0 units); the army stood 2–3 tiles out before each declaration, and he took Peace with one enemy to fight the next.
- Hard Mongols do none of this: about 0.7 Keshigs a game, 0 Horseman upgrades in 32 games on his map, half the army Spearmen, Keshigs idle in 54% of turns next to an enemy.
- He reported that accepting Byzantium's Peace on T86 also ended his war with Sparta (Byzantium's ally); the T87 tape shows one proposal (17) accepted by him and by Sparta, and both wars ended that turn. Under investigation by the lead.
- **Open questions:** (1) T37: upgrade rather than build, to keep the promotions? (2) T55: did he see India had no military unit? (3) Did he ever charge a city that still had HP?

## 2026-11-06, 8.9.25 Mongols (answers to the coach's questions)
- **Why upgrade Horsemen into Keshigs instead of building new ones (T37):** "i did it because it was faster. i had the money and the units were already there." Keeping their promotions was a bonus, not the reason. Upgrade-when-rich is about tempo: the unit is already at the front.
- **India's empty army (T55):** "i noticed it was easy to conquer them, but i assumed it was because i killed them all." He didn't read India's collapse; he pressed because resistance was weak.
- **Charging a city with HP:** "yeah, but not when they had walls." Unwalled cities: charge in directly. Walled cities: shoot them down from range 2 first, ride in at 0 HP. (Tape: 17 Keshig charges, 12 at cities, 5 of those captures into 0-HP cities.)
- **How rivals should have beaten his horde (user, 2026-11-06):** "probably a bunch of spearmen, but also walls and rushing to get tech for advanced melee units to be able to counter them. But honestly, if you have a mongol horde of keshig coming at you like the one I had, you're probably [finished] if you don't act quickly. Everyone else on the board should have seen that I was conquering and attacked and built up a defense. otherwise it's unstoppable." The counter is collective and early: read a conqueror by what it is doing (cities taken, an empire eliminated), not by its VP, and arm and attack before the army arrives.

## 2026-11-07, how he would defend a capital whose fall ends the empire (Byzantium design)
- **His defence plan, in order (user, 2026-11-07):** "1) get walls up in the capital 2) ensure there are buffer cities around it with walls to slow things down 3) have a garrison unit in the capital at all times 4) later in the game, have melee units near the capital and even later, fighter jets to make sure there are no sneak attacks." So: the capital's Walls come first, then walled buffer cities, then a permanent garrison, then a mobile reserve, then air cover. This is the target behaviour for Byzantium's AI under the capital-collapse rule.

## 2026-11-07, Switzerland feels easy to him
- **"im still confused why switzerland doesn't win more. when i play as switzerland it feels like I always win easily."** (User, 2026-11-07, on AI Swiss at 6.7% in the nightly census.) The evidence so far says the AI plays the Swiss badly, not that the kit is weak. He wins with every faction. Hard in his 8.9.24 seat from T1 wins 0 of 32. The kit multiplies a lead: without it his T90 position drops from 94% to 16%. AI Swiss went from 11.58% with `swiss_playbook` 2 (11-05) to 6.7% (11-07), so a later landing may have undone it. He plans another Swiss playtest.

## 2026-11-07, 8.9.26 Switzerland: a game he was going to lose
- **"i allied with aztecs which put me at war with sparta. that was a bad idea and sparta was going to beat me. that's an ok outcome. i should not have joined a war against sparta and i paid for it. this is a good thing."** (User, 2026-11-07.) He counts it a fair loss: the Alliance pulled him into the Aztecs' war with Sparta (the Alliance offer's "You would join the war against Celts and Sparta" line warned him), and Sparta's army punished it. He wants that to stay possible: choosing your wars matters, and Hard can make you pay for a bad one. Save copied to `~/LegatumRuns/bug-saves/swiss-8926-sparta-20261107/`.
- **What the game shows** (coach review, `docs/playtests/reviews/2026-11-07-8.9.26-switzerland.md`): a 30-turn game (about 4 active minutes), abandoned the turn Zürich fell. The Alliance was the third of three steps: Zürich founded T19 three tiles from the Aztec capital as Sparta declared on it with three Hoplites already there; his only Soldier 12 tiles east at a barbarian camp; then the Aztec Alliance accepted T24 with Tlatelolco at 0 HP, which made him a defender in both Aztec wars at once (not a call to arms). The Aztecs were eliminated T25 and the wars stayed. Hard in his seat declines the offer and keeps Zürich (48/48 to T40); with it signed, 0/48. In the continuations from T30 Athens wins 42/48 and Sparta 0–2, so Sparta was not on course to win the game.
- **Open questions for him:** (1) On T19, did he see Sparta's three Hoplites at Tlatelolco when he founded Zürich three tiles away? (2) On T24, did he see Tlatelolco at 0 HP, and what did he expect the Alliance to give him? (3) Where was the Soldier going after the barbarian camp (T16–T27)?

## 2026-11-07, 8.9.26 Switzerland, second game: an easy win
- **"again, super easy. i absolutely crushed everyone else on the board and it didn't even feel hard. the AIs are still bad."** (User, 2026-11-07.) This is the second Swiss game on 8.9.26. The first was the Sparta Alliance-war loss, which he counted as fair.
- **India's Ancient army in the Modern era:** "while im in the modern era with bombers and stuff, india was still building warriors and archers. no idea why. they're basically useless at that point in the game." His screenshot shows India's Spearmen, Archers and Siege, all wearing the era-I plaque, next to his Modern units. To check: whether India lacked era resources (every land unit from the Classical era on needs its era's resource), whether it fell behind in tech, or whether the AI's build valuation prefers cheap units. Save: `~/LegatumRuns/bug-saves/swiss-8926-win-20261107/`.
  - **Answer (coach, 2026-11-07):** not tech (India was first into the Renaissance); India had no Niter T81–T135 (he had settled both nearby deposits, Biel and Bern, before Niter was visible) and the AI's army tally kept asking for units it could only fill with era-I ones. Review: `docs/playtests/reviews/2026-11-07-8.9.26-switzerland-win.md`. Open questions for him: did he place Biel and Bern for the Niter; were the T131–T135 Field Cannons and Bomber bought to take Strongest Army and start the hold; why accept Sparta's Peace (T136) but refuse India's (T137)?
- **How he plays resources (user, 2026-11-07):** "I usually get the 'discover the resource' tech first because i need to know where they are so i can focus my effort there." He researches the tech that reveals the next era's resource before anything else, then aims settling and armies at the deposits it reveals. He wants deposits placed unfairly, so each era opens with "a rush to find the next era's resource".
- **His answers on the lost game (user, 2026-11-07):** (1) At T19 he didn't see Sparta's 3 Hoplites next to Tlatelolco when he founded Zürich 3 tiles away. (2) At T24 he didn't notice Tlatelolco was at 0 HP when he accepted the Aztec Alliance. (3) His Soldier "was heading back to defend. i didn't realize i was about to get rolled by sparta." The loss came from information he never saw, not from a choice he weighed. A war between two other empires next door, and an ally on the brink, were both invisible to him.
- **His verdict on why (user, 2026-11-07):** "i don't think that's on the UI, I think that's due to a lack of scouting... I made a bad decision and i was punished for it. this was a good outcome." Scouting is part of the game. Don't build warnings that replace it.
- **His answers on the 8.9.26 Swiss win (user, 2026-11-07):** (1) Placement for Niter is answerable from the tape: Biel (T60) and Bern (T69) were founded before Niter was visible to his seat, so they weren't placed for it. (2) The T131–T135 Field Cannons and Bomber "were meant to get strongest army, retake my city, and then start conquering my neighbors": one purchase served the crown, the recapture and the next offensive. (3) Sparta's Peace at T136 but not India's at T137: "I did not want to fight a two front war if I could avoid it. I probably could've done both at the same time, but I didn't think it was necessary to win the game and could've caused distraction." Near a win, he closes fronts he doesn't need and keeps the one he's using.

## 2026-11-07, how he places cities and uses scouting (general strategy, his words condensed)
- **Site value is more than yields.** He looks for high Production, Food and overall resources. He also thinks competitively: block rivals from territory that could cause him problems later, and weigh how aggressive each neighbour is.
- **Don't settle toward an aggressive neighbour.** With Sparta on one border and open land between them, he won't put a city near Sparta. It would likely be taken: the Settler is wasted, Sparta gets a city, Sparta gets a reason to declare war, and that city then threatens all his others. He settles away from them, even on the far side of the map. **Exceptions:** he settles toward them only with a whole army there, knowing he can wall it quickly, or while that neighbour is distracted by another war, or once he can defend it.
- **Denial is a risk-reward bet.** Biel and Bern blocked India and denied it Niter (he didn't know the Niter was there). Safer sites existed on the other side of the map, but cities that cut a rival off from land it would need later pay off. A rival that's blocked won't go around the map to settle behind him; it may not even be able to.
- **Information is worth a unit.** He scouts a lot early. On the Sparta border he put units on hills at the mountain chokepoints to watch for an attack instead of garrisoning them: "If I have 5 turns of warning, I can start building units that would defend my cities better than if I have a garrison in there and then I have to rapidly change things." A buffer of information about where enemy troops are drives his strategy and his production.

## 2026-11-07, 8.9.26 Sparta: conquering with 5–6 soldiers
- **"im winning easily. im just conquering everyone with a relatively small army of about 5-6 soldiers. nobody is offering real resistance and the coalition isn't doing anything... conquering the world shouldn't be this easy."** (User, 2026-11-07, mid-game.) A small veteran army is enough. Rivals don't build up against a visible conqueror, and the coalition against the leader produces no real military pressure.
- **Final (user, 2026-11-07):** "game just ended. absolute domination. probably my easiest victory yet. and yet somehow they're the third worst." AI Sparta sits around 9–10% in AI-only censuses, yet in his hands it's his easiest win. The gap is AI play, plus possibly the kit since today's slow-heal cut.
- **From the tape (coach, 2026-11-07):** 5 Hoplites by T29, no new military unit until T97 (Settlers and Markets), the same five upgraded to Riflemen, 1 Hoplite lost, 14 Peace offers refused. Hard Sparta trains more units, storms cities 5x less and keeps no veterans (review `docs/playtests/reviews/2026-11-07-8.9.26-sparta.md`).

## 2026-11-07, 8.9.26 Celts: an easy win
- **"i crushed everyone, easily. in your estimation they seem weak, but in my play I almost worry they're overpowered! I won that game more easily than i won the sparta game."** (User, 2026-11-07.) This is the third faction in a day (Swiss, Sparta, Celts) that sits mid-to-low in AI-only censuses yet gives him a crushing win. To check: whether the Celts kit (barbarians never attack the Celts, the 60-Gold sack) is strong in human hands, separately from AI weakness.
- **From the tape (coach, 2026-11-07):** 16 towns (two from other empires' Settlers: Germany's at T17, which founded Gergovia, his Great Work city; Byzantium's taken back from barbarians at T36) behind 3–4 military units; the Wild called T88 with 11 towns; one Bomber bought T122 that, with 2–3 Riflemen and barbarians, razed five German cities in 11 turns. The sack's 2,520 Gold arrived after his only purchase and ended unspent (6,075 Gold). Review `docs/playtests/reviews/2026-11-07-8.9.26-celts.md`.
- **Open questions for him:** (1) T17: was the war on Germany for the Settler it had just captured, or planned anyway? (2) T88: was the early Wild for the 2 VP or for the camps as a screen against Sparta and Germany? (3) T122: why Germany rather than Sparta as the Bomber's target?

## 2026-11-07, 8.9.27 Germany: another crushing win
- **"i just played another game as germany and absolutely crushed everyone final score was like 13vp to 3 in second place. the ai is still bad against every civ."** (User, 2026-11-07; Hard AIs, not Training.) That's the fourth faction in a day to crush the AI: Swiss, Sparta, Celts, Germany. The weakness isn't faction-specific.
- **From the tape (coach, 2026-11-07):** Hard in his seat can't win this one (designer-seat 5.4%; the Aztecs win most runs), unlike the Swiss, Sparta and Celts games (56–88%). His edge was a late plan: Essen on the Industry Science project from T100 (11% of his Science), a 1,884-Gold hoard behind 1–3 military units, Flight on T134, two bought Bombers, and five cities taken T144–T151 with no unit lost. The Mongols took Hamburg twice (T60, T76) after his T35 declaration. Review `docs/playtests/reviews/2026-11-07-8.9.27-germany.md`.
- **Open questions for him:** (1) T99: was the Science project in Essen aimed at Flight and the Bombers, or just the best use of a capital with nothing left to build? (2) T134: why the Aztecs (the VP leader, its army busy with Switzerland) rather than the Mongols, who had just declared on him again?
- **His answers on the 8.9.27 Germany win (user, 2026-11-07):** (1) Essen's Science project from T99 was "to get flight faster, but also because it had nothing better to do". A finished capital becomes a tech engine aimed at the next power spike (Flight). (2) He struck the Aztecs, not the Mongols who had just declared on him, "because they had a city with a wonder that would get me points faster". He picks targets by VP per capture: Chalco's Hagia Sophia (+3 VP) beat the empire actually at war with him. Target value is points, not grievance.
- **On the "Keep Zürich" scenario (user, 2026-11-07):** "in that scenario, you're not going to keep zurich. the only way to keep zurich would be to have not gone to war with sparta and build units and walls earlier." The position at T25 is already lost. The decision that mattered came earlier: decline the Alliance that drags you into Sparta's war, and build defenders and Walls before a strong neighbour is next door. A scenario that starts after the decisive choice can't teach anything.

## 2026-11-07, "The Horde Looks East" scenario (8.9.27, Byzantium vs the Mongols)
- **"i felt like i'm missing something... im way outmatched by the mongol army and i don't have enough time to build defenses or a counter, and within a few turns it looks like they make peace or something and then come to attack me almost immediately? so the whole idea of 'they're distracted in the east' isn't accurate."** The scenario's premise was false: the Mongols ended their eastern war and turned on him. (The scenario was deleted in the 11-07 sweep.)
- **The coalition gap, in his words:** "the mongols are clearly running away with this game and none of the other civs are willing to ally with me to counter them. And also, sparta, my one ally, has a big army just sitting off to the side when the mongols leave to attack me and they don't counter attack." What he expects: "when the mongols have such a large army and are clearly on track to dominate the world... other factions are willing to ally and actually attack to open up new fronts and make it harder for the mongols." Against a runaway leader, AIs should form alliances with anyone fighting the leader, and allies should open a second front when the leader's army is committed elsewhere.

## 2026-11-08, "Hold Biel" scenario (8.9.28, Switzerland vs India)
- **"i mean it technically worked but it was extremely easy. am i missing something? is that something that the AI doesn't win 100% of the time?"** He won in 8 turns (felt: too easy, fun 1). From his run: on the first turn he bought a Rifleman in Biel out of 6,104 Gold, fired the Field Cannon on the nearest Cavalry and walked the nearby Field Cannon and Rifleman toward Biel. Biel's own strike killed a Spearman; India never got the city below 140 HP.
- **Correction (2026-11-08, same day):** the version he played was broken by rules 25. The state was cut on rules 22; India knew Metallurgy but not Industrialization, so under the new reveal it lost sight of its Coal, its two Coal Cavalry lost supply and India pulled back after T132. In that version Hard also holds Biel 50/50. The 0/50 below is the original (and the fair, India-given-Industrialization) version, where Hard still loses Biel 50/50 on master.
- **The gap it shows:** Hard from the same start met the objective in 0 of 50 runs (every run 1080, losing a city) on the pre-11-08 AI. For a human the answer is obvious (a threatened city with a huge treasury buys a defender at once); Hard does not do it. Under investigation on `claude/hold-biel-defense`. Lab scenarios that are trivial for him but 0% for Hard are the cheapest AI targets we have, and he does not need to replay them.
- **"Six Spartans and a Wall" (user, 2026-11-08): "just played another secnario with the spartans... it was extremely easy."** Won in 6 of 12 turns. From his run: all six units adjacent to walled Nicomedia by T66, then combined assaults on three consecutive turns (2, then 3, then 4 Hoplites plus a Phalangite), losing one Phalangite to the city's fire; captured T69. He ignored nearby unwalled Syracuse. Hard from the same start: 0/50 (pre-11-08 AI). The human answer is concentrate everything and keep attacking every turn while the split counterattack is cheap; re-measured on current master in the Hold Biel investigation.

## 2026-11-08, four cards mined from his saves (task `mine-easy-for-human-scenarios`)

What he did in the original games, from the task report (his actions, not his words; the cards are cut on rules 25 and no human has played these exact versions yet). Hard from the same start, 50 runs each:

- **`germany-defend-essen` (8.9.27, T134, Hard 0/50):** Mongol riders approached Essen with 1,884 Gold in the treasury and one Rifleman in the field. He bought two Bombers and held all eight cities.
- **`celts-steal-a-settler` (T16, Hard 0/50):** he took a German Settler that crossed near his Scout and founded his second city, Gergovia.
- **`india-strike-the-leader` (T148, Hard 0/50):** with Flight and 1,286 Gold he bought a Bomber, then took Gergovia and Nicomedia.
- **`mongols-ride-to-dyrrachium` (T37, Hard 2/50):** he upgraded two Horsemen and captured Dyrrachium, an unwalled city, while Nicomedia next to it had Walls.

The pattern matches Hold Biel: he spends Gold on the one unit that decides the fight and strikes the soft target; Hard keeps Gold and splits its force.

## 2026-11-08, 8.9.29 Aztecs: a turn-116 win (hold began T107)
- **"ok just won as aztecs in 107 turns... it was super easy."** First game on the new Hard AI (training round 2, siege stance, hidden-wariness coalition). Fifth faction in two days to crush Hard (Swiss, Sparta, Celts, Germany, Aztecs).
- **On the coalition:** "no they didn't really do much." The sidebar said "Coalition against you" by T102, but he felt no pressure.
- **When he knew he'd won:** "pretty early. i jsut kept conquering and getting points and nobody stopped me." A conquest chain with no counter: the coalition's job (pillar 11, EU4-style deterrence) did not happen. Coach review: `docs/playtests/reviews/2026-11-08-8.9.29-aztecs.md`.
- **From the tape (coach, 2026-11-08):** the win was 116 turns (the New Fire on T107 started the hold), 29 active minutes. He conquered before he settled: five Spearmen took all three Swiss towns T27-T42 (Switzerland had one military unit), and his first Settler came on T55. Then four walled cities T59-T69, 6 to 8 melee units all game, batch upgrades with Gold before each siege (Pikemen T72, Musketmen T94-T102), and 13 turns waiting outside walled Therapne for the Musketmen before a four-turn storm (taken T101). One purchase all game. Hard in his seat: 0/48 from T2 and T20, 10% from T55, 67% from T65, 94% from T70 (designer seat 23.3%).
- **Why "they didn't really do much" was true:** every AI read him as the runaway from T42, with wariness 48 to 82 from T50. Byzantium joined Sparta's war the day he declared (T63), the Celts were pulled in by an Alliance on T98, and India never joined (the Alliance fee and its own war with Sparta). Three strike operations expired with 1 attack between them; seven Celtic units stood three tiles from St. Gallen (no Walls, one Spearman) for 15 turns without attacking. Byzantium took his two newest colonies during the hold, worth 0 VP, instead of empty, walled Solothurn, worth 3.
- **Open questions for him (intent only):** (1) T26: Switzerland because it was the weakest neighbour (one military unit for two towns) or because it was nearest? (2) T55: was conquest his expansion plan from the start, or did Settlers only start once the land ran out? (3) T63: declare on Sparta then but storm Therapne only on T97: was the plan always to wait for Gunpowder?

### Three cards cut from the 8.9.29 Aztecs game (2026-11-09)

Positions come from the save's chronicle, cut on rules 25; no human has played these versions yet. Hard from the same start, 50 runs each:

- **`aztecs-swiss-spearmen` (T24, Hard 0/50):** with one city and four Spearmen he declared War on Switzerland at T26, took Schaffhausen at T27 and Solothurn at T34.
- **`aztecs-eight-by-seventy` (T56, Hard 4/50):** with four cities (three taken from Switzerland) he founded four more by T69, reaching eight; the card gives 15 turns.
- **`byzantium-break-the-fire` (T108, Hard 0/50):** the position he never faced: the Aztecs lit the New Fire at T107 and stood at 11 of 10 VP with Byzantium at War with them since T64. Seated as Byzantium, Hard never takes empty, walled Solothurn.

## 2026-11-10: Hold Biel on 8.9.30 (war variant `war-swiss-hold-biel` r1)

His words: "it was insanely easy. i basically just got a rifleman and then that was it. how is it possible that the AI can't do that scenario?" Two runs that evening (`~/LegatumData/scenarios/war-swiss-hold-biel-2026-11-11T00-10-11Z.json` and `…T00-50-30Z.json`): the first lost a city by T136 (score 1060; Riflemen bought in two cities on T132, one more on T133, none on T134); the second won at T140 (score 1150) by buying one Rifleman in the same city on each of T132, T133 and T134 and nothing else. The card's "Hard 0/50" was measured on 11-08, before threatened-city defence landed; Hard on the Lab copy measured 34/50 and 35/50 on 11-09. In the 11-08 baseline Hard bought a Cavalry and then a Horseman in Biel, where he bought Riflemen.

Correction from him the same evening: "the one loss was a misclick. I intended to move the rifleman into the city but it didn't happen. that's not a real loss." The tape agrees: on T134 Rifleman 174 left Biel (23,10) for (23,11) and Rifleman 172 moved from (23,11) to (22,11), a tile adjacent to both, instead of into Biel; the city stood empty and fell on T135. Count his record on this card as 1 real attempt, 1 win; do not treat the first run as evidence that the card is hard for a human.

His reasoning for the card, in his words: "I just don't understand how the AI could possibly lose... you have $6000 gold and a bunch of troops. just buy riflemen?" The line is: the strongest defender available, bought in the threatened city, every turn, and kept in the city. Current Hard on this copy (master, 2026-11-10, 50 runs): 21/50. It buys a Rifleman in Biel on T132, then a Phalangite on T133 (50/50), mostly nothing on T134, Rangers and Phalangites later, Cavalry in Basel on T132, and from T136 its bought units leave Biel to attack Cavalry in the field. Cause of the unit choice not yet traced.

## 2026-11-10: Ride to Dyrrachium on 8.9.30 (`mongols-ride-to-dyrrachium` r1)

His words: "it was also extremely easy... what am i missing?" One run (`~/LegatumData/scenarios/mongols-ride-to-dyrrachium-2026-11-11T01-11-15Z.json`): won on T40 with four turns to spare, score 790. T37 and T38 every Horseman and Soldier 3 marched straight to the tiles around Dyrrachium (10,25), no upgrades; T39 he declared War on Byzantium with five units adjacent and all five attacked; T40 four attacked again and the city fell. He is missing nothing: the card says so itself ("This should be easy for a human").

Current Hard from the same start (master, 2026-11-10, 50 runs): 2/50, the same as the 11-08 baseline, so this number is not stale. Hard spends T37 upgrading two Horsemen, stages the army at (16,21)/(17,21) beside Nicomedia (15,20), declares War on T39 and in 48 of 50 runs sends Horsemen into walled Nicomedia from T40 to T44; only 2 runs go to Dyrrachium, and both win. Hard is not told a card's objective (nothing in `crates/ai` reads the win condition), so this measures which city Hard picks by itself: the walled one beside its staging point over the open one a two-turn ride away, with a mounted army that cannot crack Walls.

Same pattern as Hold Biel the same evening: the human line is one obvious sentence (there, "buy the best defender in the threatened city and keep it there"; here, "ride everything at the open city and declare when adjacent") and Hard misses it on target and unit choice, not on tactics.

His read after both cards (2026-11-10): "it feels like the AI simply does not understand how to set and commit to an objective of any kind...? the task was 'do this thing' in the scenario and somehow it decided to do something else?" What the runs show: Hard is never given the card's task, so on Dyrrachium it set its own objective (Nicomedia) and did commit to it for five turns in 48 of 50 runs; the error is which objective it set. Hold Biel is the commitment failure: units bought to hold the city leave it to chase Cavalry. Proposed to him: a second measurement arm where Hard is handed the card's objective, to split "picks the wrong goal" from "cannot execute a given goal".

## 2026-11-10: Two Cities, Two Bombers on 8.9.30 (`germany-two-cities-one-turn` r2)

His words: "i think I beat it in one turn.....??? [what?] how did the AI succeed 0 times?" One run (`~/LegatumData/scenarios/germany-two-cities-one-turn-2026-11-11T01-16-34Z.json`): won on T143, score 1310. T142: Infantry 137 to (23,8) beside Xochimilco, Cavalry 202 to (22,12) beside Chalco, Bomber 192 rebased from Berlin to the forward city. T143: one Bomber on each city (Chalco −246 HP, Xochimilco −208 HP, both to 0 from full) and each ground unit walked in the same turn. Chalco was the Aztec original capital and held the Hagia Sophia, so the turn was also worth +3 VP and put him at 10.

Current Hard from the same start (master, 2026-11-10, 50 runs): 0/50, median 880, same as the 11-08 baseline. It never attacks either city: T142 it proposes Peace to the Mongols and pulls the Cavalry back toward Cologne, T143 the Infantry goes to Colony 8, and the cities queue Tanks and Artillery; the Bombers sit, and in 11 runs one strikes (8,19), away from both targets. Hard never pairs "a Bomber empties a city in one strike" with "a ground unit beside it takes it that turn", so it sees no capture on offer and plays defence. Third card of the evening with the same shape: the winning line is one sentence and Hard does not consider it. Design flag for him: one Bomber takes a full-HP city with a Castle to 0 in a single strike.

## 2026-11-11: four intent answers after the mining of his games (`designer-mining-1111`)

Asked what the tapes could not show. His answers, in his words where it matters.

**Why he keeps building with an enemy army near a city.** "Yes, I trust the walls. I also generally have units nearby, so I can rush them in." And it depends on the city: "If it's not a city that I'm super afraid to lose, then I just don't worry that much about it. I consider them buffer cities. It's one of the reasons why I build so many cities. Cities with walls are a pretty good buffer on an attack on cities that actually matter." For the AI: a threat is priced by three things Hard ignores today: whether the Walls hold, whether units can reach the city in time, and what the city is worth. Losing a buffer city is an accepted cost, not an emergency.

**The Gold reserve.** Not an emergency fund but "a strategic reserve... it gives you a lot of strategic flexibility": buy one or two units on whichever front needs them, buy next-generation units the turn they unlock, upgrade quickly. "It's almost exclusively for units." Rarely, he spends it to rush a victory point (e.g. four Universities). For the AI: the reserve's value is option value on units, sized to about two purchases plus the next upgrade wave; it is not spent on buildings.

**A second front.** Only opened on purpose when "the person that I'm fighting is so weak that another front is a very easy thing for me to win." For the AI: the charge for a second war should fall toward zero as the current war becomes a walkover, instead of a flat 4% of win chance per war.

**City spacing on the frontier.** "It's more useful to just maximize the number of cities": he packs cities at the minimum distance (4 apart, 3 for India) and counts how many fit. The cities form a wall: with a city between the frontier and the capital, "a wall of cities with range units and walls makes it pretty resilient to attacks." For the AI: a frontier site is worth more than its yields, because it screens the cities behind it; this is the value Hard's founding risk refusal (19 of his sites) leaves out.

## 2026-11-11, Switzerland on the 8.9.30 build: a T140 win ("won easily")

From the tape (his actions, not his words; review `docs/playtests/reviews/2026-11-11-switzerland-review.md`):
- **He picked the busy, unwalled strongest neighbour.** The Mongols held Strongest Army and were the likeliest winner (Hard-seat continuations from T30–T45: Mongols 15–16 of 24), but their army was at Tenochtitlan in a war with the Aztecs. He built three Spearmen and an Archer T40–T50, parked them two tiles from Shangdu at Peace, refused the Aztec (T48) and Celtic (T50) Alliances against the same target, declared alone on T51 and upgraded six Spearmen to Phalangites the turn after the first capture (T54–T55, 360 Gold). Four cities and the Mongol capital by T69; Mongols eliminated T70; VP 2 → 7.
- **Then economy behind Walls through four wars.** 6–7 military units for 12 cities, Walls in 10, one purchase all game (Roadworks, T59) and 15 upgrades; Markets, Mills, Libraries and Universities where Hard in his seat builds units. Treasury 5,151 at T130 (the Reserve and the hold), 6,956 at the end.
- **A buffer town.** Lucerne, founded forward on T88, lost to Germany T107, refounded on the same tile T119, burned by the Celts T131; seven of the coalition's thirteen campaigns went at it. This matches his 11-11 answer about buffer cities.
- **Open questions for him (intent only):** (1) T40–T51: the Mongols because their army was busy, because their cities had no Walls, or because they were the strongest AI? (2) T48/T50: why refuse the Alliances against the Mongols when he was about to attack them anyway? (3) T88/T119: was Lucerne a buffer he expected to lose, and why found it again on the same tile?

### His answers (2026-11-11)

**Why the Mongols.** Mainly the distracted army: "their army was distracted in the south, quite far away from where I was fighting. So I knew that opening up a second front in the north would put them in a really tight spot." Then the faction's power curve: "I knew that they cannot build walls in the early game... relatively vulnerable in the very early game until they get their Keshig horde. Then they become really dangerous. So they're sort of like a mid-game peak." Seeing them roll up the Aztecs, "this would be a huge problem for me pretty soon if I didn't do something about it"; "usually I don't attack people like that as the Swiss, but... the mid-game Mongol horde would probably crush me." It was a preventive strike inside a closing window, not a hunt for the leader. For the AI: a war's value should include the rival's future strength if left alone (a faction that peaks later and is snowballing off a neighbour), and its window (no Walls yet, army away); today the plan prices only the present position.

**Terrain and counter-units.** "All of these battles are happening in the forest, which tends to limit the mobility advantage that the Mongols have." He started Archers, then switched to more Spearmen "because they have an advantage against mounted units." For the AI: pick the army's unit mix for the chosen enemy's roster and the terrain of the front, after the target is chosen.

**Why no Alliance with the Aztecs or Celts.** Not strategy: "there's been so many weird issues with alliances in this game... trying to avoid bugs where I have an alliance with them and then they declare peace with the Mongols and then suddenly I'm at peace with the Mongols." Otherwise "I would have happily allied with them to get the visibility": the value of an Alliance to him is map sight of the enemy's units, not military help. So his refusals here are not evidence about Alliance value; do not train on them. Product problem: he does not trust Alliances. For the AI: price shared vision of the war target into an Alliance.

**Lucerne.** A deliberate buffer: "I would not say that I necessarily expected to lose it, but I was willing to lose it... tried to build walls there, and packed a bunch of units in to make it so that we're fighting over territory that's not important to me... it would get razed and I could just rebuild it, it just delays the advance." For the AI, both sides: as defender, a cheap forward town that draws the siege is worth more than its yields; as attacker, a population-1 buffer is a decoy and the campaign should go past it (matches the coalition's 16 of 20 attacks on Lucerne).

### His answers to the Fable review's questions (2026-11-11)

**Refusing the Mongol Peace at T59.** "I wasn't under any threat... There was no real benefit to stopping. I could keep pushing and see until I get some sort of resistance, and then maybe if... it's no longer worth the effort, or if too many other fronts open up... I could change my strategy." "There's no point in ending my war when I'm winning just fine. I think calling for peace would have just decreased my probability of winning." For the AI: a winning war continues by default; Peace needs a reason (resistance met, another front, a threat at home), not the other way round. Hard's "Peace with the Mongols" after each capture is the opposite rule.

**4–7 units for 12 cities at War with four empires.** He gauges the threat as it arrives: "Once people started pushing against border cities... I would gauge that risk, and then if they started putting a real threat towards anything that mattered to me, then I would have to start retreating and building more units." Nothing threatening reached a city that mattered, so he bought nothing. He expects the occupation lock to change this: "I would have had to either buy more units or call for peace earlier because my units would have been trapped in those cities."

**The Reserve margin at T130.** He knew. His plan was the reverse of the review's worry: "I would much rather buy units than risk losing a city... if needed, I'll spend money to dip below the threshold in order to defend my cities so that I can get back up to it fairly easily. My gold per turn was very high, so it would just delay my victory by a few turns." For the AI: a goal threshold is not sacred; with high income, dropping below it to save a city costs a few turns, and losing the city costs more.

**Razing your last city.** No rule: "no human would ever do that... an AI just shouldn't do because it's stupid." AI fault to fix (the Mongols at T70).

### Occupation lock, first game (Sparta, Hard, 8.9.31, 2026-11-11)

**The lock slows a conqueror, as intended.** "Already as Sparta... I'm feeling the pain from the locked units. This is good! It's slowing down my advance. If I had those units unlocked I would have taken more cities. This is promising." For the AI: a capturing unit is spent for the rest of the war, so the choice of capturer and the size of the follow-up force are priced decisions.

**It must read as a rule, with a reason.** "There needs to be some kind of UI indicator that the unit is locked, otherwise it just feels like a bug." "However it's communicated, it should be clear that the units are locked to suppress the rebellion or what not... just need to make it more clear." Locked units should also stay out of the action queue: "they're just automatically fortified and you can't take actions with them."

## Rival near victory, first game on 8.9.31 (2026-11-11)

Playing Sparta on Hard: "ok the celts just got 10vp and i was surprised. there should prboably be a warning of some kind once someone gets to 8 or somehting". The VP track was on screen the whole time and the hold alarm fired at 10, but he was not watching the track while fighting his own war. Lesson: a visible track is not a warning; a rival reaching target minus 2 needs its own interrupt. For the AI: a human notices a runaway late, so Hard should not assume rivals react to a leader before the warning point.

## Game length and VP sources, same game (2026-11-11)

"in general it feels like there are too many ways to get points. im on turn 123 and the celts are about to win. a typical game should be more like 250 turns. i think we made too many ways to get VPs so the games go too fast." Context: Hard, Sparta, both experiments on, so World Ages era cards (up to 6 VP) were live alongside goals, wonders, crowns and capitals. One game; the Celts' VP breakdown has not been read yet. His target: a typical game ends near turn 250, not 120.

## World Ages: one card per Age (2026-11-11)

Looking at the Industrial Age row (Gain 3 cities / Demobilise / Tribute 400, which he scored with Tribute on turn 115): "for each age as this gets revealed... it should really only be one, randomly decided option per age, not 3 to choose from". With three cards he took the cheapest (pay Gold) and the other two were never a decision. Same game as his "too many ways to get points" verdict.

## Coalitions and simplifying (2026-11-11)

After asking how to "join a coalition" and learning it is a label, not an order: "it feels like the 'coalition' thing doesn't need to be an explicit mechanic... you can always just declare war on them. maybe we simplify? are there any other opportunities to simplify?" His lean: drop the player-facing coalition concept, keep declaring war as the answer. A simplification audit (usage counts from his tapes) was started the same day; nothing is cut until he sees it.

## Simplification round 2 (2026-11-11)

After the simplification audit he chose: "go with A, yes delete decadence, yes fold calls to arms." A = cut the three crowns as a VP source (target stays 10) to push games toward 250 turns. Decadence is deleted outright. Calls to arms become a notice with a Declare War button; Alliance exit cost and betrayal record go with them. Still open: Sparta's Honour the Veterans goal (replace or cut) and which lenses he uses.

## Triumphal Arch is a VP giveaway (2026-11-11, 8.9.31, Classical Age)

Screenshot of the Arch card: 80 Production, 3 turns, +1 VP kept for good. "there really should be no way to get 1vp with 3 turns of production this early in the game... something is off if we're giving away victory points like this." A VP must never be a cheap purchase: an Age card has to cost a real sacrifice or a contested act. Arch prices at the time were 80/100/120/160/200/240 Production for Ages 2 to 7. Fix pending his choice (cut the Monument card, or reprice it).

## Occupation lock, second game (2026-11-11, Mongols, Hard, 8.9.31)

"still won pretty easily, but it was a little bit more challenging because of the unit lock, which is a good thing. it locked the best unit I attacked with and then required me to do more planning and i couldn't roll everyone as easily." The lock works as intended: the capturer is usually the strongest unit, so each capture costs the army its spearhead and forces planning. Hard is still too easy for him.

## Athens on 8.9.32, and halving captured cities (2026-11-11, Hard, crowns cut)

"just played as athens... was also super easy as you can tell from the score." Cutting the crowns did not make Hard a contest. His proposal against snowballing: "what if for cities you take, the population halves? i think that's how civ 6 does it." Today a capture costs the city 1 population (`battle.rs::capture_city`) plus resistance. Not decided yet: the lead's reply recommends trying it as a lobby rule experiment (round up, no loss when a city returns to its founder) and says the main cause of easy wins is still Hard not punishing a conqueror.

His answers on the Athens game (2026-11-11):
- Empty cities T105–T120: "They hadn't come yet, so i assumed they wouldn't. if they did, i would react." He prices garrison need from the AIs' observed behaviour, not from their capability. The AI lesson is the other side: a human who has never been attacked strips his cities, so one real strike on an open city changes how he must play the whole game.
- Second Arch queued 13 turns late: "i didnt know it was there. would have gotten it earlier." A UI miss, not a decision: a second copy of a paid Age card was not visible as available. His real pace on that VP was faster than the tape shows.
- T129 Peace offer to the Swiss: yes, to unfreeze the captured cities (occupation lock). He uses Peace as the lock's release.

## Aztecs on 8.9.32.1, and the role-reversal scenario (2026-11-11, Hard)

Won 17/10; Germany 3, everyone else eliminated. "Won by a lot again." His idea: "i was not in the lead near the end, and then i snowballed and took over the game. it might be useful to have a scenario where I play as Athens or something before the Aztec snowball so that I can show you how Athens could have won the game?" Role reversal: after a game he wins from behind, cut a scenario from the seat that was ahead (or best placed to stop him) just before his takeover, with Hard playing his old seat, so his play in that seat shows what the AI should have done. Adopted as a standard coach output.

Follow-up (same day): "to the extent possible.. if you could convince Aztecs to play like the way i played, that would help!" The old seat should reproduce his takeover: drive it from his recorded orders where the tooling allows (Hard takes over when a taped order is no longer valid), otherwise cut late enough that his army and wars are already in place. Measure the scenario's Aztec captures and VP by turn against his real game.

His answers on the Aztecs game (2026-11-11):
- Gold held at T129 (1,301, of which 1,160 bought two Riflemen the turn after Athens declared): "deliberate". He keeps a treasury as a defence fund in place of a standing garrison: units are bought when an attack appears, not before. An attacker has to price his Gold as defenders he can buy next turn.
- Why Athens at T152: "No, but it worked out that way. I picked them because their capital was close to me, and they were a threat because they could take my capital quickly with no buffer. they were a threat." Target choice is threat removal by proximity to his capital, not the prize; the free techs were a bonus he did not plan around.
- Captures during the hold: he did not understand the question as asked ("i don't understand the question"). Re-asked in plain words.
- Captures during the hold, re-asked: "i was just carrying on and not thinking about the countdown". The 7 VP past the target were momentum, not insurance. The hold countdown did not change his play at all; no AI threatened it, so it never needed attention.

## Occupation lock comes back after Peace (2026-11-11, 8.9.32.1, bug report)

"i took a city from mongols. we then had peace. they later declared war on me again... and now this city is 'held' and i can't do stuff in there anymore. its like after the peace and then war again it went back to the old state" (Almaliq, population 7, banner "Held"). The code did what the rules reference said ("the record stays until the city changes hands"), but he reads it as a bug and it is one: the former owner can re-freeze a city at will by declaring again, and the holder cannot end it without the attacker's consent. Fix: Peace with the civ the city was taken from clears the record for good (as its elimination does). A later war with that civ does not lock or freeze the city.

- **"i feel like there has to be a way to throw a ton of fable 5.2 tokens at this to make the AI good, no? this model is so smart! how is it that hte AI in this game is still so bad"** then, on the proposal to put Fable in the seat as an offline teacher: **"can we try this overnight and just see what happens? im about to go to bed, so it can do this overnight. I have 2 resets, so feel free to play a whole game."** (User, 2026-11-11.) Approved: Fable plays the game itself, offline, from the seat's own player view; a whole game is allowed and the token cost is accepted for this run. The shipped Hard still never calls a model; Fable's play is a teacher and a benchmark. First run: his scenario cards, then a whole game against Hard seats, with token cost and results reported in the morning.

## Back from travel: four answers (2026-11-13)

Asked four numbered decisions after the war-pricing and Fable-in-the-seat runs, he answered: "1. only if its winning 2. sure 3. ok 4. use your judgment".

1. **Teach Hard the capture tactics Fable used** (two melee attackers on the capture turn, clear adjacent units first, no attack the preview says loses, buy the unit that can capture): build them, but they turn on **only if they win** on the After the Riflemen card (Hard's baseline 21/50) without losing ground on the other cards. No default flips on a tie.
2. **Public treasuries: yes.** Every empire's Gold is visible to all; the AI reads it from its player view. Rules version 31.
3. **Rerun Fable's whole game properly: yes.** One turn per call, with the missing facts added to its view. Later the same day: "we still have a lot of fable credits, so please run overnight to continue your training run with the fable agent if you think that makes sense." Fable tokens are not the constraint this week.
4. **The open items are the lead's to decide.** Decided 2026-11-13: cut the paid World Age cards (Triumphal Arch and Tribute); a captured city loses half its population; the occupation lock becomes the rule and its lobby switch goes; the simplification cleanup (decadence, calls to arms as a notice, free Alliance exit) goes ahead. Rows are in the design doc's decisions log.

## The plane era arrives as too big a jump (2026-11-13, 8.9.33)

"Pacing feels good… But it accelerates too fast with the tech era that has planes. Too much of a jump with that tech era. Maybe we need more eras? perhaps something like 10? it just feels like it jumps really fast at that point. what do you think". The pace up to the Modern era is right; the Modern era (Flight: Fighter and Bomber) is where it breaks. Not decided. The lead's reply: Modern stacks three things in one era (the 45% strength step, the end of the class triangle, and aircraft that strike from 8 hexes with no retaliation), and late eras may also be shorter in turns; measure turns per era from his tapes first, then prefer spreading Modern out (Flight later and dearer, or one extra era splitting ground from air) over going to 10 eras, which costs units, art, wonders and Age cards for every new era.

### His answer on the plane era (2026-11-13)
"1. ok, let's increase tech costs. 2. im ok with flight in the era it's in, just that things seem to accelerate at that point 3. use your judgment. -- it's nto that planes are too strong. they're supposed to be strong. it's more that right around the time planes show up, the whole game accelerates way too fast."
- Decided: raise Modern tech costs so the era lasts like the earlier ones. Flight is not moved. Aircraft are not weakened. No extra eras.
- The complaint is the pace of the whole game from that point, not air power. Watch the Future era and the end of the game for the same speed-up after the Modern change.
- Hard using aircraft: the lead's call; it goes ahead as AI work.

## After the Riflemen, played on 8.9.34 (2026-11-14)

"i immediately recognized that hte army in place was not going to be able to take hte city, so i retreated to regroup, heal, wait until i got field cannons, upgrade, then attacked again. I then looked through all my cities and prioritized getting more coal (via mines), which i needed for more current-era units, and then markets and banks if they can't do that or build units so that I can get more income. Then I pushed the attack and took the city."
- He judged the army against the city first and saw it could not win, so he pulled back at once: no attack, no purchase to patch the siege.
- The wait had a named end: Field Cannons. Heal while waiting, upgrade when the technology lands, then attack again.
- City priorities while waiting, in order: Coal Mines (Coal gates current-era units), then Markets and Banks where a city can do neither Mines nor units, for income.
- Result file: `~/LegatumData/scenarios/athens-after-the-riflemen-2026-11-14T18-42-39Z.json`.

### Go-ahead on the AI queue and more model-in-the-seat games (2026-11-14)
"you can kick off yours too. and if you think it's beneficial, we still have a lto of fable credits. you said it takes 45 minutes per fable playthrough, so feel free to do that until we are at 20% remaining on our token budget. keep a supervisor of some kind to ensure that we're still getting useful learnings from the experience. im not looking to use tokens arbitrarily but we do have tokens to spare. We also have [a huge number] of tokens left on OpenAI, so feel free to use Astra to do similar playthroughs."
- The four queued AI items start now (refit gate, capture priced by VP, buy to close an objective, Mongols baseline).
- Fable whole-game playthroughs continue until the weekly budget is 80% used, and stop sooner when games stop producing new learnings. The lead supervises: usage and novelty are checked after every batch.
- Astra (Codex) plays the same kind of games on the OpenAI budget.

### Go-ahead on regroup, synchronised strike and retreat-to-heal; city walls fire (2026-11-14)
"yes go ahead. it shoudl also be aware (if it's not already) that city walls attack, so be mindful when units are put in harms way"
- Build the three Hard changes from the After the Riflemen analysis (regroup as a priced plan, synchronised strike, retreat towards healing), each behind an off switch.
- Hard must price a walled city's fire for every unit it places within range: staging, waiting, retreating and healing, not only on the assault turn. The Fable games found city fire unpriced, so it is not already aware.

## Athens on Hard, 8.9.34, World Ages on: thinking at T113 (2026-11-14)

"still on athens game. it's a pretty close game on paper, and im now on turn 113 and im thinking about end game. so im building som eunits i can disband to get the VP for that... im also thinking about how im going to betray my athens alliance and take their capital and wonders right next to me... probably once I get flight, so i need to save up to buy a bomber quickly and rush that since im athens and im ahead on science. no action, just note to self"
- End-game planning starts at T113 of a close game: he reads the VP sources still open and works back from them.
- He builds units for the purpose of disbanding them to score the Age card (Demobilise). The card is scored by preparing for it, not by changing his plan. Check against the tape whether this is the "scored without changing my plan" problem again.
- He plans to betray his ally (he wrote "my athens alliance"; he is Athens, so take the ally's name from the tape) to take the capital and wonders beside him: the target is picked by the VP it moves and by being adjacent, and the alliance is priced as a temporary convenience.
- The timing is tied to a technology: wait for Flight, save Gold now to buy a Bomber the turn it lands. Athens's science lead is what makes the timing his.
- No action asked.

## Athens game finished (user, 2026-11-14)

He reported the Athens game on Hard with World Ages on (8.9.34, rules 31) finished. Save copied to `~/LegatumData/backups/athens-1114/athens-hard-world-ages-2026-11-14.legatum` for the coach review. This lifts the hold on the rules-32 build (auto-explore for every military unit).

## Athens game result and the betrayal as played (user, 2026-11-14)

"yes i won by a lot. the betrayal went well. in fact, i ended up deciding to go after byzantium first because switzerland was busy with their army fighting sparta, so i snuck in and attacked byzantium, took their capital and the heir, then turned around to take switzerland."

- He changed the target order on the turn from what he planned at T113: the first victim was the one whose army was away (Switzerland's was tied up against Sparta, so Byzantium was hit first and Switzerland after). Target choice by where the defender's army is, not only by VP and adjacency.
- On the era: "the tech jump to modern is too big. tanks + flight is OP. it feels like we're missing a tech era to pace things better. maybe something that's the equivalent of ww1 tech. we jump from basically the civil war up to modern tanks and infantry... it's almost like we skipped over the entire 20th century". Second time he has raised the Modern jump (first: 2026-11-13, answered with Modern tech costs +30%).

## 2026-11-14 — Tara bounce (8.9.36, Celts, turns 62-64)

His words: "it would appear that there are two civs that are trying to take my city, but they are bouncing. that's not really what's intended here right. i thought we resolved this already with the tie breaker going to the one with the most power on the move?"

Intent confirmed: when several empires move on a 0-HP city, the one bringing the most force takes it, whatever those empires' relations with each other. Cause found by replaying his save: India and Sparta were at Peace with each other, and the 2026-11-11 rule only ran for attackers at war with each other. Fix goes into rules 34 on `claude/special-units`.

## 2026-11-14 — Siege stall at Celts Colony 7 (8.9.36, Celts, turn 144)

His words: "on turn 144 of the current game, the AI [fell apart]. no idea why. it was winning the siege and I think i may have just barely turned the tide... and now its units are just sitting there doing nothing". His screenshot: the city (1 tile SE of Gergovia) at 0/200 HP with a damaged garrison inside, and four magenta melee units (two Knights, a Spartan Hoplite, a Cavalry) adjacent, fortified.

What he expects: an attacker that has a city at 0 HP with melee units adjacent finishes the job, or pulls back deliberately; it does not stand still. Diagnosis from the save copy is in progress (`idle144` in the session scratchpad).

## 2026-11-14 — Landship: name, art, class (8.9.36 build list)

His words: "obviously we need to fix the art for 'landship' and can we also just call it 'early tank'? nobody knows what a landship is. and i thought tanks were supposed to be in the mounted class but i see both cavalry and landship here?"

- Decided: the Landship's player-facing name is **Early Tank** (the `UnitKind::Landship` identifier stays). Its placeholder art (a sepia modern tank) is replaced by a WWI tank painting.
- The class question is a readability miss, not a rules bug: the Landship is mounted and Cavalry upgrades to it, but the build list folds an older unit under a newer one only when the newer one is at least as good in every stat (`app/src/ui/productionChoices.ts::unitLines`), and the Landship has 3 moves to the Cavalry's 4, so both get a card. Nothing on the card says "mounted", which is why he could not tell.

## 2026-11-14 — Celts retaking their own city burned it (8.9.36, Celts, turn 156)

His words: "i took back my own city, and it razed it? i feel like if i take back my own city it shouldn't be razed as celts right?"

- Confirmed in code: `rules/battle.rs` sacks every city a Celtic unit captures, with no check on who founded it. Decided: a city the Celts founded comes home intact when they retake it (no sack, no Gold, no camp), like any other founder's recapture keeps its full population.

## 2026-11-14 — Bomber recon request (8.9.36, Celts, Bomber in Lugdunum)

His words: "is there a way to have bombers to 'recon' or something that lets them show tiles? that would allow me to then attack them the following turn if i dont have units nearby."

- Today: no. A strike needs a target his team already sees; only the Biplane gives sight (4 tiles around its city), and that sight is lost when it upgrades to a Fighter or Bomber.
- **Decided 2026-11-15: "both steps on bomber recon"** (base sight for Fighters and Bombers, and the Recon order).
- What he wants: to use an aircraft's turn to find targets where he has no ground units, then strike the turn after. Not decided. The lead's proposal: a Recon order that costs the aircraft's order for the turn (a strike given up for sight), plus Fighters and Bombers keeping the Biplane's 4-tile sight around their base; the case against is that needing a spotter is the Bomber's only real limit and his late wins already run on one or two bought Bombers.

## 2026-11-14 — Celts game abandoned; asks for the extra era (8.9.36, Celts, turn 204)

His words: "[oh no]. since i cant get my cities back and athens just took my capital... taking it back means it's going to be razed... so i have to give up on this game. oh well. will try again with celts later. I'm pretty convinced that we still need the extra tech level for the ww2 era tech. can you help me come up with that and do the new techs, cards, units, and stuff?"

- He quit because of the sack rule, not because the position was lost on the board: retaking Alesia (taken by Athens on T203) would have burned it. Rules 34 fixes this for cities the Celts founded. Save copied to `~/LegatumRuns/bug-saves/celts-8936-abandoned-20261114/`.
- From the tape: his era entries were T109, T136, T161, T191 (Industrial to Future), so eras ran 25 to 30 turns; Athens entered the Future on T165. He lost seven colonies from T120 and razed six cities himself (five retaken or taken from India and Switzerland).
- He no longer wants to wait for the rules-34 playtest before the extra era. Proposal: `docs/design/ninth-era-proposal-2026-11-14.md` (awaiting his rulings).

## 2026-11-14 — Ninth era approved

His words, on the four rulings in `docs/design/ninth-era-proposal-2026-11-14.md`: "after modern is fine, go with your names, both in"

- Placement: the new era goes after today's Modern.
- Names: today's Modern becomes World War; the new era is Modern.
- Air Defence and Rocket Artillery at range 3: both in.

## 2026-11-14 — Look-alike art pairs: delegated

Asked whether to accept the seven pairs the art check flagged on the rules-34 branch or repaint first, his words: "use your judgment". The lead accepted all seven in `tools/art/distinctness.json` (reasons on each entry) so the build ships, and queued a Panzer repaint so Panzer and Tank stand apart. This is a delegation for these seven pairs, not a standing permission to add accepted pairs.

## 2026-11-14 — Byzantium, 8.9.37, T81: the Swiss Settler that sat unguarded

His words, with a screenshot of the captured Settler: "turn 81 i took the swiss settler... but it was basicalyl just sitting there naked for a long time...? weird choice?" He declared on Switzerland on T80, took the Settler on T81 and founded Heraclea with it on T82, on the very tile the Swiss wanted. The tape shows why it sat there (see `docs/learnings.md`, same date): a Settler frozen one step from its site for 11 turns, its escort gone home. He read it as a free city, correctly.

## 2026-11-15 — Byzantium, 8.9.40 AI (8.9.41 was installed mid-session; replay matches the 8.9.40 tag), finished (a win): Athens first, economy behind a distant Sparta

His words: "i didn't really use any special abilities for byzantium this time around... but that's probably ok. my focus this time was making sure athens doesn't snowball, so i focused my early attention on them. i noticed that sparta was pretty far away, so I wasn't at risk of an early attack. that let me focus on building population, science, and gold. then i took cities from athens, jumped in tech, and then crushed everyone."

- He picks the rival most likely to snowball (Athens) and spends his early attention on it, before it compounds.
- He reads the distance to the rush faction (Sparta) to choose between defence and economy: far away means no early defence, so population, Science and Gold come first.
- The order was economy, then Athens's cities, then a tech jump, then everyone else.
- He won without Byzantium's specials, a signal for `legatum-factions` that they are not pulling him into a Byzantine style.

Same message, on the end of the game: "can we make the "conquest" victory condition just "take all the capitals"? otherwise, the late game can get pretty tedious tracking down every city and every settler."

## 2026-11-15 — Byzantium, 8.9.40 AI (not 8.9.41): Settler snipes still work

His words: "i was still able to do a few settler snipes early on. those have huge swings for the game... im surprised that's still a thing given how many times we've tried to teach the ai to be more careful with settler snipes." Each snipe in this tape gets traced to the turn, the Settler's escort and the code path that let it walk (coach review of this game), before another fix is written: the earlier fixes were each for one cause.

## 2026-11-15 — Capitals victory: the saving is in how he would play, not the turns after the last capital fell
- The coach review said a capitals rule would have saved 4 turns in his Byzantium win (T177 against T181), holding his play fixed. He corrected it: "that's not accurate, because it would have also changed how I played the game. I would have spent less time thinking about all of the other random cities, and I would have just beeline for the capital. It probably would have saved me 30 turns."
- Lesson for reviews: a rule change is judged by the play it would produce, not by replaying the old tape under the new end condition. Say so when a counterfactual holds his orders fixed.
- Consequence for Hard: under this rule the winning line is a beeline for capitals, so Hard must defend its own capital and count capitals when it ships, not in a later increment.

## 2026-11-15 — Supported defence: "we do this aggregate attack, but we don't do a supported or aggregate defense"
- His words: "it woudl make combat feel more like Diplomacy where units "support each other". maybe nothing in the ui even needs to change... there isn't really a good way to prevent... all of them ganging up and taking that city, especially given that the defense attack back, the reply, is distributed among the other attackers. So this gives a huge advantage to attack and makes defense nearly impossible." His example: a walled city, three Riflemen attacking from above, three Riflemen below all ordered into the city. "Those should all be combined as a combined defense as well, as opposed to they move into the city when the unit dies, one of them will move in. Otherwise, they just bounce and nothing else happens."
- What exists (rules 27, relief chains in): units ordered into their own attacked city absorb the assault in turn and the first survivor becomes the garrison; unattacked, they stay out. What does not exist: the relief fights back (the defending side counterattacks once, split among the attackers), and none of this applies outside a city.
- His answer (2026-11-15), which decides it: "the city walls absorb all the damage first (as expected) then the damage is spread among the units supporting the city. if the unit in the city itself dies, one of the supporting units takes its place in the city. once the walls, the unit in the city, and all supporting units are defeated, then the attacker takes the city. it's basically the same result as if you put all the attacking units on one hex and all the defending unnits on one hex and then had them attack each other."
- Backfill picks by strength as well as HP (2026-11-15): "i woudl say "healthiest and strongest"... a warrior in the late game might have hte most HP, but they can't absorb it." The supporter that enters is the one that can absorb the most. He also asked for "a very small simulation for this that I can play in a scenario": two Lab scenarios (hold, take) ship with the rule's build.
- Status: decided (design doc decisions log, 2026-11-15); being built as a rules change on `claude/supported-defence`. The lead's reading of "attack each other", still for him to check: every defender replies and the replies are pooled and split among the melee attackers; a supporter that is itself attacked gives no support; open ground is a second increment; Hard learns to support and to cut support with the rule.

## 2026-11-15 — Panzer costs 1 Oil; faction cards get another pass
- Panzer: "can we have this use 1 oil? why did we make this 2 oil? there are no other units that require double supply." Decided: 1 Oil, like every other unit (design doc decisions log, 2026-11-15). The 2 Oil was the lead's first-draft price for the Panzer's +1 move, approved untested with the six special units on 2026-11-14; it was the only double-supply unit and needed its own supply code. Peltast and Reisläufer are also plain upgrades, so nothing replaces the drawback. Built on `claude/panzer-one-oil` (rules bump: ships between his games).
- Faction cards (Choose your empire, Germany): "can we have fable and astra take another pass at these cards? we have art availbale that we're not showing, like for units. this is better than it was a while back, but it could use another iteration". Rule for `legatum-ui`: a faction sheet shows the painted art of its special unit and special building, not a text row with a generic icon. Astra review queued (`faction-cards-pass`); Fable is at its weekly cap until 2026-11-17, so the lead's own notes stand in for the Fable pass until then.
- Art (same day): "is the art for everything in good shape? we have a lot of chatgpt credits, so maybe we do a pass at all of our art to make sure it meets our standards?" No full pass has covered every class since the 2026-10-10 audit; units were swept 2026-11-14. Queued a Codex audit of every art class (`art-audit-all`: report, ranked repaint list, phone contact sheets), then Codex repaints from that list, most visible first, one class per build.

## 2026-11-15 — World Ages still feels contrived; the VP on offer is forgotten mid-game
- His words: "It still feels... off. This is more the direction I think we shoudl go, but it still feels contrived. e.g. disbanding 2 units just means i save up gold and purchase the units -- so it could hav ejus tbeen gold? it needs to be something that causes the player to take an action they otherwise wouldn't necessarily take. conquer a thing. get units to a certain level of experience. do seomthign else... we've done several iterations on this."
- The test for every Age card from now on: it makes the player take an action he would not otherwise take, and it cannot be met by spending Gold or Production. Demobilise fails it (it survived the rules-31 cut of Tribute and the Triumphal Arch only because it was priced, which is the same thing): cut. Replaced first by a Veterans card (his "get units to a certain level of experience").
- Visibility: "the victory points and victory conditions aren't front-and-center, or are not refreshed in the palayers memory enough to make sense. I often get 3/4 into the game and then think "oh right, I'm supposed to do something this era to get points". maybe the era-turn takeover animation shoudl also show the victory points that are now available for the era?" Built with the card change: the era takeover shows the open Age cards and their VP, and the open card stays on screen every turn (the lead's addition: a takeover seen once does not fix forgetting 40 turns later).

## 2026-11-15 — Germany, 8.9.43: won in 169 turns; the extra eras fixed the "god tech" jump, the AI is still easy
- His words: "i just played another game as germany and crushed everyone... adding the additional eras has improved the game meaningfully. i still won in 169 turns... which is too fast... but the pacing felt better. it didn't feel like i suddenly got "god tech" like before and just ran away with the game with flight and tanks. it took more time and iteration. but still, the AIs are dumb and easy to conquer".
- Read: the nine-era curve is confirmed as the right direction (keep; finish increments 2-3). Game length is still short of the 250-turn target, and the cause he names is not pacing but Hard losing wars. The next work is Hard's defence and war play, not more VP or cost tuning. Coach review of this tape queued the same turn.

## 2026-11-15 — Treasury: he saves and spends in bursts; the AI sits flat near its minimum
- His words, looking at the game chart: "i do tihngs very differently in almost every game, my money spent is smarter than their. it's as if the AI just wants to have the minimum amount of moeny in savings to survive, and I use it strategically. the chart shows that i save up and spend strategically, and the AI is basically flat all game."
- A valuation error that holds across his games, not one game's gene: Hard spends Gold as it arrives (or holds a floor), so it never has a treasury when a decisive moment comes (a war starting, a city about to fall, a Settler or army to buy at the front). He banks Gold toward a planned purchase and spends it all at the moment it swings something.
- To measure before changing anything (coach, across all his tapes): his Gold curve against each Hard seat's; what he bought on each spike and what it decided; what Hard bought with the same Gold in the same turns; which code decides Hard's spending and reserve. Candidate direction: Hard saves toward a named purchase tied to its committed plan (an army for a campaign, an emergency defence fund sized to the nearest threat) and spends in a burst, instead of a flat floor.

## 2026-11-15 — Treasury, measured (coach review of the Germany 8.9.43 win)
- His claim holds across 25 games: median treasury 461 Gold against Hard's 72, peak 2,842 against 241; he spends in bursts (2.0 a game; 30 of 51 followed by a capture within 10 turns), Hard in single upgrades (0.27 bursts a game; 1 of 33).
- What his bursts buy: an army the turn a war opens or a target is in reach (three Cavalry for 2,041 Gold on T113, two turns after the Aztecs declared), an upgrade wave the turn a unit tech lands, and two or three defenders the turn a city is threatened (two Riflemen on T138 against India's strike). Almost no buildings (11 in 25 games).
- In this game he held 1,500 to 2,200 Gold with 2 military units for 9 cities from T90 to T112 and nobody attacked; the tape cannot say whether that Gold was earmarked for a war before the Aztecs declared (question in the review).

## 2026-11-15, Germany 8.9.43 (won T169): why he held Gold, and the T138 Riflemen

- **T90 to T113, 1,500 to 2,200 Gold with 2 military units** (user): "I was mostly holding it in a strategic reserve in case I was attacked, which i had to do a few times this game. but also so I could upgrade units when a big tech jump came like cavalry / tanks." His bank is first an emergency defence reserve and second an upgrade-wave fund for a big unit tech; it was not saved for a planned war. The army he bought at T113 was the reserve answering the Aztec declaration.
- **T138, last 1,100 Gold on two Riflemen, not Cavalry** (user): "that was to get the victory point." The purchase was aimed at a VP, not at the Indian strike the review credited it with. Which VP is to be read from the tape (objective or World Age card at T138), not asked.
- **For Hard:** the named fund's default order is emergency defence, then upgrade wave, then campaign army. **Corrected by him the same day:** a VP purchase does not outrank a military one ("I'm not sure i would say it outranks... its just that i was already winning, so i didn't need to use that money to buy more military units"). The rule is conditional: when the war is already won or safe, spare Gold goes to the VP; when it is not, the army comes first.

## 2026-11-15, Athens 8.9.47 (abandoned about T65): Sparta's early war, and what he expects next

- **He quit because the position was lost** (user): "i basically think my current game is [lost] so im going to quit." Sparta declared at T37 and broke his army; he accepted peace at T64 ("Sparta's army crushed me and im now in a bad position. i can't afford to be at war right now. i need to rebuild my military and get settlers. im way behind right now", his coach-question answer).
- **What he expects of a good Hard here** (user): "the Aztecs should probably attack me right now too since they have crossbowmen and I have no way to defend myself." A neighbour with a tech edge should take the opening a third party created.
- **His test for Sparta's war** (user): "good work by the AI if Sparta is able to compete in this game... otherwise this is basically just sparta going kamikaze into me and killing us both, which i guess is fine (if they picked me fairly) but not a great strategy." A rush is good play only if the attacker comes out able to win; a war that ruins both is not. The target must be picked on the merits, not because he is the human (pillar 6).
- **To answer from the save, not from him:** did Sparta come out ahead of the other six after T64; why Sparta chose him; whether the Aztecs' Hard valued the opening.

## 2026-11-15, Athens 8.9.47, second game (won): his answers and his verdict on the coach questions

- **Verdict on the questions** (user): "it was interesting to see what Hard focused on. it focused on what I was producing near the end of the game when it didn't matter all that much and i set all my cities to 'auto'. it seems to have missed the decisions that actually mattered." Four of ten questions were about cities on automation. Questions must rank by what decides the game, skip automated choices, and name the city and tile ("Next time can you specify which tile and which city you're referring to?").
- **T87, a city founded far from the best site** (Hard rated it 29 against 98): "I think Celts are about to attack me with those units on my border, so I wanted to found something far away from them." He discounts a site by the army standing next to it; Hard's site value did not.
- **T101, 640 Gold unspent at war:** "i definitely wouldn't buy a composite archer when im about to get attacked by a mounted unit. terrible idea. if anything I should buy a musketman... but i have a new tech coming any turn now and i want to upgrade my units, plus i have enough units to defend marathon anyway." Three rules: buy the counter to what is coming, not the cheapest ranged unit; hold Gold when an upgrade tech is a few turns out; do not buy when the forecast already holds.
- **T131, 522 Gold unspent:** "i could probably buy something though i would probably buy a tank to get the extra mobility." Hard wanted an Assault Trooper and a Soldier; he values mobility for the attack over two cheap bodies.
- **T143, 924 Gold on a Tank Hard would not buy:** "i want to keep pushing my attack on the celts." Gold goes to the campaign that is winning.
- **T147, no war on India although Hard's plan was to contain it:** "im already at war with enough civs. I can't afford to open another front."
- **Automation differs from Hard** (T129): "it's weird that 'auto' is different than what 'hard' would do." City automation and Hard's production choice should agree or the difference should be explained.

## 2026-11-15, why he keeps settling through a war (Athens 8.9.47 win, T80 to T100)

Asked about building 3 Settlers and 2 Walls during the Celtic war where Hard in his seat built none (user):
- **Strategic resources:** "a lot of the expansion for me is 1) access to strategic resources. some of those expansions I think got me lots of oil and stuff which I needed."
- **Buffer cities:** "2) they create buffer cities. cities that have walls and units in them that shoot back slow down and give me time to react."
- **For Hard:** a new city at war is valued for the deposits it brings that the empire lacks (now and one era ahead) and for the time a walled, garrisoned city on the enemy's approach buys the cities behind it, not only for its yields. Walls go up early in such a city. This is why he does not stop expanding when a war starts.

## 2026-11-15, the faction sheet redesign was not done when he next opened it
- Looking at the Sparta sheet in 8.9.57 (user): "what ever happened to the screen redesign i asked you for on this page?" He asked for it that morning on the Germany sheet. Astra's review was written by 11:06 and then sat unread for nine hours behind AI and faction work. A request of his that has a finished review and no implementation is the next thing to start, not a backlog item; say where it stands before he has to ask.

## 2026-11-15, Sparta 8.9.57 (won about T117, World Ages on, Coach questions on): "the most dominant game ive played yet"

- **Verdict** (user): "this one was absolute domination. in fact, i think this was the most dominant game ive played yet. not sure if any rule changes affected that or if i got better at playing as sparta." His VP line reaches 10 at about T108 and 11 by T110; no rival is above 2 VP at the end. Eleven captures and five wars declared by T100. To answer from the tape: the rule and AI changes of the day against his play.
- **T65, the Pyramids in the middle of a war:** "Because i have a sufficiently large army at this point. my real constraint is iron. so if anything i need VPs because i have 4 now. that would give me 6. so it was either pyramids or settlers to get more iron." Once the army is large enough for the war, Production goes to VP or to the binding resource, not to more units.
- **T79, offering India peace:** "their cities are so close together and their walls are killing some of my best units, which is not ideal." Overlapping walled cities are a reason to stop.
- **T83, refusing India's peace four turns later:** "they already killed my most experienced unit and i was about to take their city with no additional losses and all my units would heal. basically because i was winning, so why would i take peace?" A capture that heals the army (Sparta) is worth finishing.
- **T105 and T110, Gold unspent:** "at this point, im just moving the 3 military units to retire them and get my faction 2vp and then i win"; "doesnt matter, im about to win in 7 turns." No purchases once the win is in hand.
- **Coach questions, what was wrong in this game:**
  - T40, T49, T68 garrison questions fired on a barbarian Scout, a single Horseman and an Indian Scout ("the barbarian? i don't think they're a threat to a city with a wall that's actively building more units"; "scouts don't attack with very much damage. im not threatened by it"). A garrison question needs a force that can actually hurt the city (Walls, HP, what the city can build or buy next turn), not any unit in range.
  - T40 printed "team 255" for the barbarians.
  - T79 said "you made peace with India" when he had only offered it, and named Hard's unrelated plan ("make peace with Switzerland").
  - T105 and T110 asked about purchases while he was about to win; he skipped five of twelve questions.

## 2026-11-15, his reading of the Sparta 8.9.57 review (what Hard should have done)

- **Obsolete purchases** (on Switzerland saving 1,067 Gold and buying seven Spearman-class units in era 5): "It's weird that it still does that... if you look through my games, i basically never buy obsolete units. on rare occasion, ill buy something one generation back if i dont have the strategic resource available." Rule for Hard, from his play: buy the best unit it can train; one generation back only when the strategic resource for the best one is missing; never further back.
- **The named fund is not the fault** (correcting the lead, who switched it off): "well im not sure the named fund is the problem, its that it spent the money poorly on bad units and then used them poorly." Saving and spending in a burst is right (it is how he plays); what to fix is which units the burst buys and what they then do. The fund comes back on once those are fixed and measured.
- **A neighbour's fall is a warning** (on the victims seeing his stack only one to three turns out): "after i rolled up athens, and then after celts... surely India knew i was coming for them next. if I were next to Athens and saw them get attacked by a big spartan army, I would immediately say '[uh oh], i need to gear up for a big war with sparta'." An empire that sees a neighbour conquered by a strong army prepares for that army: walls, units, alliances, before it turns up.
- **India naming him top threat and doing nothing for 37 turns:** "weird. why? that seems like the thing that could have been the biggest counter-balance." The second front on a leader who is busy elsewhere is the counter he expects.
- **The coalition arriving one empire at a time:** "yeah, definitely a failure mode."
- **Nobody contesting the hold:** "by that point it was too late and there was nothing they could do. there was only one civ left." Not a fault.
- **Veterans card:** "could be more specific with language. something about an earned promotion or something."
- **War-flavoured Age cards going to the empire winning by war:** "war flavored cards are fine." Decided; do not reopen.

## 2026-11-15, the Egypt and Rome unit art was accepted too easily
- The lead accepted the first contact sheets ("I'm accepting the set"). He asked (user): "can you look at the unit art with a more critical eye? are you sure this looks reasonable? e.g. the things they're holding, realism, etc?" On a second look the Legionary's "pilum" is an ordinary spear with a metal collar (no long iron shank), its sandals have studs on the sides, and the figure is in a throwing pose; the Medjay wears a bronze cuff nobody wore, Roman-style strap sandals, carries a bow with no string, and crouches where every other unit stands; the Roman portrait holds a bronze diploma (issued to auxiliaries from the 50s AD, not to a legionary in 30 BC) and the Egyptian a painter's palette, not a scribe's.
- **Rule for art review:** before accepting a figure, check each thing it holds and wears against what that soldier actually carried (name the object and its real form), check the pose and scale against the standing figures already in the set, and do not repeat the painter's own description of a piece without looking for it in the image.

## 2026-11-16, first Rome game (8.9.67, Hard, Coach questions on; in progress at about T68)

- **Rome's rule** (user): "it feels like a real decision... but also, that's a lot of money... im still able to attack and win, but having an army means that I don't have a lot of cash to pay for the founding of the cities. I think we should just make it 'veteran soldiers can found cities'." Decided: no Gold price (design doc decisions log; `legatum-factions`). On escalation: "this is already implicit because the units that are veterans later in the game are more expensive than early units."
- **T16, war on Athens:** "I thought I could poach their settler, but the settler ran back into the city. immediately going to try to take their capital now and get the settler." A failed Settler snipe turns straight into a capital strike: the war is already declared, so he uses it.
- **T39, not yet declaring on the Mongols although Hard would:** "I need to move some more units closer to the mongols before declaring war and i need my units to heal after that unfortunate encounter with barbarians where i lost 2 units. will probably declare war on them within 10 turns." He declares after the army is in position and healed, not when the plan first values the war.
- **T55, refusing Mongol peace:** "because im about to take their capital. why would i make peace with them?"
- **T68, the Pyramids mid-war:** "because i can't afford more military units right now because the current gold cost to settle new cities is so high for Rome... so i might as well do something else. i do wonder that if we change the rule and make it no gold cost if i would have just built more military units." The founding price was distorting his production, not only his Gold.
- **Coach questions this game:** four asked, four answered, none skipped, all on wars, peace and a Great Work. The T16 text contradicted itself ("Hard would not have declared war... Hard plans a campaign against Athens").

## Rome on 8.9.71: storming India's walled capital (user, 2026-11-17)

His words: "i took a city from india, then i tried to storm their capital which had walls... and i got crushed. i lost 5 units... I think that outcome is probably ok beacsue that's why the walls exist, but i want to make sure it's a reasonable outcome. is it because of the reinforcement from their other unit? or was it really just the walls?"

What the tape shows (exact replay): he declared on T22, two turns after India finished Walls in its capital (T20). Two Spearmen took unwalled city 7 on T26. From T38 four Spearmen (strength 20) attacked the capital (200 HP, strength 30: City 10, Population 10, Walls 10): 8 to 11 damage per hit, 15 to 32 back per attacker, and one wall strike a turn (38 to 59 damage; four of the five kills). The city never fell below 63 HP, so India's garrison Spearmen never fought. He could not tell whether the garrison or the walls beat him: the result was right, the cause was not readable. Open question put to him: did the attack forecast say the storm was hopeless before he committed?

## 2026-11-20, Switzerland on 8.9.72 (won T150, Coach questions on; 8 asked, 8 answered)

Answers from `<app data>/training/answers/web-1795232768791-….jsonl`:
- **T35, Settler held:** "the city that i wanted to found was taken by bibracte, so i had to move to another spot." A site lost to a rival is re-chosen, not settled on the next-best tile at once.
- **T77, war on the Celts (Hard would not declare):** "it didn't have walls. they had cavalry and i had special pikemen so i thought i could take it." He declares on a border town he can take with the units he has, and picks the unit that counters the garrison (Pikemen against mounted). Gergovia's Walls finished that same turn; it still fell on T79.
- **T85-T94, at War and buying nothing with 2,116 Gold:** "im not under threat from the Celts and i don't think i can take more of their cities in this war because i dont have enough strategic resources." He banks during a war that isn't pressing him and that he can't yet push; spending waits for the units that can take cities (Cuirassiers T115, Cavalry T130).
- **T113, Settler held one turn:** "i want to occupy more space and get more production from the hills." He settles for land and Production even at War.
- **T120, Grand Bazaar mid-war:** "i don't feel threatened by Celts and i need to start getting points. its turn 121 and i only have 2 points." He tracks VP against the clock and starts the Great Work when the war is safe.
- **T129, Hard would buy Cavalry for 595:** "yeah, i probably shoudl have bought cavalry. that would have been smart." (He bought two the next turn.)
- **T131, refusing Athens's Peace:** "i didnt even notice. they're irrelevant to me." A rival with no army is ignored, peace or war.
- **T144, five Athenian units near empty Bern:** "theyre warriors. im not worried about it." He reads unit quality, not count, and leaves cities empty against obsolete armies.
