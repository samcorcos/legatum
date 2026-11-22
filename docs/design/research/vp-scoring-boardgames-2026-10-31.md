# How VP scoring feels "earned" in board and 4X games (research for Legatum era objectives, 2026-10-31)

Question: what makes a point feel earned rather than automatic, and which mechanisms let a peaceful-leaning empire win while still needing one successful war?

**Method and honesty note.** Web search plus page fetches. Fandom, PC Gamer and several fan wikis returned 402/truncated pages, so some rules below come from search snippets or from my memory of the rulebooks. Those are tagged *(memory)*. TI4 scoring data comes from our own earlier study of 1,974 AsyncTI4 six-player games, `docs/research/ti4-objective-scoring.md`. Player criticism is sampled from forums and reviews, so it is sentiment, not consensus.

**Legatum today** (from `legatum-rules/references/game-rules.md`): seven eras. Each era shows 3 public objectives (Build, Expand, Fight), the same for every empire. Meeting any one earns that era's 1 permanent VP, once per era per empire. There are also 3 live crowns, faction goals and a Great Work. Solo target is 10 VP (20 for a pair), held 10 turns.

---

## 1. Twilight Imperium 4

**Mechanism.**
- 10 public objectives are set up face-down in two decks of 5: Stage I worth 1 VP and Stage II worth 2 VP. Two Stage I cards start revealed, then one more is revealed each Status Phase. All Stage I come out before any Stage II. ([rules summary](https://scottmk.github.io/ti4-reference/rules-summary), [LRR summary gist](https://gist.github.com/OdinsPlasmaRifle/1e0ee0931c153b5df476f130d9a99796))
- Each player may score at most **one public and one secret objective per Status Phase**, in initiative order. (same sources)
- Public cards are **not exclusive**. Every player may score each card once, so a card is a threshold you personally meet, not a race. The scarcity is the per-round cap and the cost.
- Public objectives can only be scored by a player who controls their own home planets *(per the LRR gist)*. That is a built-in "don't lose your base" tax.
- Secret objectives: each player draws two at setup and keeps one *(rules summary)*. They are scored only by their holder, with at most 3 held and 3 scored. More come from the Imperial strategy card.
- Other VP: Mecatol custodians (6 influence plus a ground force on Mecatol, 1 VP to the first claimant), the Imperial card (1 VP if you hold Mecatol, otherwise draw a secret), agenda cards, relics and promissory notes such as Support for the Throne (a gifted 1 VP, a diplomacy chip; the fan wiki gives 1 VP, details unverified). ([BGA help](https://en.doc.boardgamearena.com/Gamehelptwilightimperium))

**Where points come from.** In our 1,974-game sample (10 VP, 6 players), public Stage I gave winners 41% of their VP and Stage II 12%. Secrets gave 27%. Custodians plus Imperial gave 6%, agenda VP 2%, Support for the Throne 5%. Public objectives are a floor everyone reaches (the average player scores 3.7 of them). The winner's edge is about one more public card plus secrets.

**Why it feels earned.**
1. *Revealed one at a time, in the open.* You see the target, then spend a round or two pointing your empire at it. The cost is planning, not luck.
2. *Cost is mostly a choice about resources.* Many cards demand a **spend** in the Status Phase (for example "spend 8 resources" or "spend 8 influence", *memory*), not just a state. Spending means not using that income on ships or votes. Other cards demand a **hold** (control N planets of a type, own N unit upgrades, N non-fighter ships in one system). Both cost something.
3. *Two tiers.* Stage I is reachable by about 70% of players per card. Stage II is scored by about 31% when it was not the last card revealed. Most players skip the hard ones and win on easy ones.
4. *The per-round cap.* You cannot bank five objectives in one turn, so points arrive across the game and the leader cannot instantly cash a lead.
5. *Cheap secrets rebalance the table.* Some secrets are "win a combat against the player with the most VP" or "destroy the last ship of ..." *(memory; exact card text unverified)*. They are the sanctioned way to reward aggression against the leader.

**War without being required.** Many publics are about holding territory, so neighbours' planets are tempting, and Mecatol rewards holding the centre. But spend- and tech-based publics let a builder score steadily, and nothing forces a war.

**Criticism.**
- Secret objectives are the main complaint: "arbitrary", and "the one thing that should not be hidden in a strategy game is the win condition" (FFG forum thread, [ffg-forum-archive](https://ffg-forum-archive.entropicdreams.com/topic/257517-ti4-to-10vp-is-shorter-but-why), plus the search summary of [ubarose](https://ns2.ubarose.com/forum/10-ameritrash/199832-ti-4)). Defenders call the swing intentional and note that you filter two down to one.
- Stage I is nearly automatic for competent players (98.5% of cards get scored; 21% are scored by all six). Our data says that is a floor, not a differentiator.
- Weak Stage II cards are scored only 27-45% of the time they appear, so some are dead cards.

---

## 2. Four other games

### Humankind (era stars and Fame)
- 21 Era Stars per era in seven sets of three. You need at least seven to advance, and **only three stars per category per era** count. Matching your culture's affinity gives double Fame. Stars you receive for being an era ahead give no Fame. ([search summary of guides](https://www.pcgamer.com/humankind-fame-era-stars/), [Amplitude blog](https://community.amplitude-studios.com/amplitudestudios/humankind/blogs/728-humankind-feature-focus-04-shaping-your-legacy?page=12), [PCGamesN](https://www.pcgamesn.com/humankind/fame-victory-win))
- Militarist stars are kill counts. The fan wiki's formula is (era x 2 + 4) kills per star at normal speed *(single fan-wiki source)*. ([Militarist](https://humankind.fandom.com/wiki/Militarist))
- Why it is earned: category caps force breadth, and the era boundary resets the board. Why it is not: every era's stars are earned the same way (techs, territory, gold, population), and "once someone is on top they always get higher fame because they unlock stars first" (player complaints, [Civfanatics](https://forums.civfanatics.com/threads/one-week-post-release-a-review-of-humankind.672733/latest)). Early Steam complaints say Militarist cultures were weak because *winning* wars gave no Fame, only kills. That is the opposite of "war fame is dominant", so I did not find evidence for the latter. The documented failure is **snowball plus repetition**.
- Lesson for us: caps per category are good, but the same-every-era menu goes stale, and a counter that rewards the leader's head start never creates a decision.

### Old World (ambitions)
- Ten ambitions win. Players are offered three at a time. An ambition's **tier = completed so far + 1**, so each completion makes the next harder. A new one is offered at least 5 turns after your last completion. One must be a National Ambition (tier 10). They carry no direct reward; the reward is the victory count plus Legitimacy. ([Old World reference](https://alcaras.github.io/owreference/ambitions), [Soren Johnson's notes](https://www.designer-notes.com/?p=1748))
- Offers are tied to leader and family traits. Military goals go mostly to warrior families. The pool includes "Kill 5 / 10 / 15 / 20 military units", "Capture a foreign capital", and also a **peace National Ambition: "Make Peace with All Tribes and Nations"**, and non-war goals like wonders and techs.
- Why earned: each ambition is the same *kind* of goal as the last but harder, and you decide which of three to chase. Johnson designed it so the target "fit the player's chosen path" instead of requiring commitment from turn one.
- Criticism: ambition victory is easier than VP for humans ("too fast", turn 110) and the AI is excluded because "a surprise ending to a 20-hour 4X game is not a good ending". Players also complain it encourages non-interactive play (buying off rivals and quietly ticking boxes). ([Civfanatics thread](https://forums.civfanatics.com/threads/old-world-designer-notes-11-soren-johnson-on-the-game-end.673707/post-16169691))
- Lesson for us: pace-gating ("not within N turns of the last one") and escalating tier are cheap anti-snowball tools. A peace route that sits next to a conquest route is exactly the designer's brief.

### Terraforming Mars (milestones and awards)
- **Milestones:** 8 MC to claim, 5 VP, only 3 of 5 may be claimed per game, first come first served. **Awards:** the first funded costs 8 MC, the next 14, the last 20; 5 VP for first place, 2 for second, awarded to whoever leads at game end regardless of who funded it. ([BGA help](https://en.doc.boardgamearena.com/Gamehelpterraformingmars), [Fryx Games](https://fryxgames.se/product/terraforming-mars-milestones-awards/))
- Why earned: both are **public from turn 1**, both cost real money, and both are capped at three. Milestones are a race against others to a threshold. Awards are a comparison race, so funding one is a bet that you will lead it. Funding something you will lose subsidises a rival.
- War: none. The lesson is structural: escalating cost per claim, caps, a public menu.

### Scythe (stars)
- Six stars end the game immediately. Goals: all 6 upgrades, all 4 mechs, all 4 structures, all 4 recruits, all 8 workers, one objective card, popularity 18, power 16, and **win combat (up to 2 stars)**. Each goal is once per player and not exclusive. Stars are never lost, and they convert to coins at game end. ([Dized rules](https://rules.dized.com/scythe/rule/concepts/placing-stars), [Board n Bones](https://www.boardnbones.com/2021/03/22/scythe-board-game/))
- Why earned: each star is a **different kind of effort**, and the player decides which six. Wars are *capped* at two of nine slots, so war is a viable part of a win but not the main road. The clock ("someone places 6") creates urgency without a turn limit.
- Criticism: a runaway combat player can still end the game quickly. Reviewers note it is the one repeatable star. The sixth-star trigger makes the end sudden.
- Lesson for us: **lanes plus a hard cap on the war lane** is the cleanest "peaceful win but you need to have fought" structure I found.

### Civ 7 (legacy paths), Millennia, Through the Ages, Eclipse, Dune Imperium, Clash of Cultures
- **Civ 7 legacy paths:** four paths (Science, Culture, Military, Economic), each with milestones worth Legacy Points. Points are spent when the age changes to buy options in the next age. The first two milestones of each path give a point; the third is stronger. Any player's milestone also moves a shared age timer. ([Civfanatics ages](https://civfanatics.com/civ7/civ-vii-gameplay-mechanics/civilization-vii-ages/), [Civ wiki](https://civilization.fandom.com/wiki/Age_(Civ7))). Military milestones centre on conquest and settlement count. Complaints: snowballing persists, ages end "way too fast", milestones are hard to reach early. Lesson: a **common timer driven by everyone's milestones** is a pace regulator, and points that buy *next-era power* feel real because you spend them, not just see them.
- **Millennia:** late "Victory Ages" with a high bar; no scoring formula found ([Paradox diary](https://admin-forum.paradoxplaza.com/forum/developer-diary/developer-diary-ages-part-one.1615781)). No lesson drawn.
- **Through the Ages:** culture decides. Military strength is tallied for wars, colonisation and aggressions, and it can steal culture. A reviewer says skipping military and relying only on culture can lose "after a few wars". ([Meeple Mountain](https://www.meeplemountain.com/reviews/through-the-ages-a-new-story-of-civilization/)). That is the closest published match to "you can't win with zero wars" without forcing conquest: the war pays you or costs you, and you can't opt out of defending.
- **Eclipse:** after combat, each participant draws reputation tiles from a bag (1-4 VP each, 14-20 tiles by player count) and keeps one. ([Dized](https://rules.dized.com/eclipse-second-dawn-for-the-galaxy/rule/resolve-battles/reputation-tiles-1)). War pays a random, capped bounty per battle. It is a model for making each fight worth *something* without scoring raw kills.
- **Dune: Imperium:** 10 VP ends the round, or the 10-card Conflict deck runs out. Conflicts escalate in three levels. Faction alliance tokens (1 VP each) are *stealable*, so ahead means hunted. Reviewers call scoring very tight (largest winning margin 3 VP in a reviewer's dozens of games). ([rules PDF](https://www.direwolfdigital.com/assets/dune/DUNE_IMPERIUM_Rules_2020_10_26.pdf), [Gaming Trend](https://gamingtrend.com/reviews/dune-imperium-review-you-must-not-fear/)). War is one pillar of several.
- **Clash of Cultures and the FFG Civilization board game:** not usable. Reachable sources gave no card rules or criticism.

---

## 3. Principles that recur

1. **Public and visible.** The strongest "earned" games (TI4 publics, TM, Scythe, Humankind) show every target. Hidden targets (TI4 secrets) draw the loudest complaints. Planning needs advance sight.
2. **Revealed incrementally.** TI4 gives one new target per round; Old World offers three at a time, the next tier appearing after you finish. You re-aim as the game goes, which is why points feel like a response to the situation.
3. **A threshold you meet yourself, not a pure race.** TI4 publics and Scythe stars are non-exclusive. TM milestones are races. Races reward the leader and the player who got there first; thresholds put the cost in *effort*. The mix that fits 8 empires is thresholds with a small contest layer.
4. **Cost is a trade-off, not a coincidence.** TI4 spend-in-Status-Phase, TM's 8/14/20, Scythe's all-six-upgrades each force you to give something up. A count you pass by accident (our current 20-55% accident rate) has no cost.
5. **A cap per round or per lane.** TI4's one-public-one-secret, Humankind's three per category, TM's three milestones, Scythe's two combat stars. The cap is what turns "free" points into choices and what keeps a leader from cashing everything.
6. **Escalation.** Stage I to II, Old World's tier = completed + 1, TM's rising award cost. Later points cost more, and the escalation can be per empire, which is a quiet rubber band.
7. **War is one lane among several, capped or conditional.** Scythe (2 of 9), Old World (peace ambition next to conquest ones), TtA (war pays or costs you), Eclipse (a bounty per battle). Pure kill counters (Humankind's Militarist stars) invite farming and snowballing. Relative goals ("against the leader") invite action against the strongest.
8. **Anti-snowball is explicit.** TI4 requires your home to hold; Humankind excludes "ahead an era" stars from Fame; Civ 7 and Humankind both show what happens if you don't (leaders unlock first and keep unlocking). Dune's stealable tokens and Old World's rising tiers are the better answers.
9. **Bounded variance.** Eclipse tiles and TI4 secrets add luck; they work only when capped (Eclipse's bag, TI4's 3-secret cap) and are the most criticised parts when not.

---

## 4. What fits Legatum

Constraints: simultaneous turns; 8 empires; no hidden math (every number with a breakdown); the AI must value it; few new systems; 1 VP about a tenth of a win.

**Rule out:** secret objectives (hidden math, swinginess, the AI would need belief modelling, and one VP is too big a swing); exclusive first-come races as the main source (they reward the leader, and simultaneous turns make ties common); pure kill counters.

### Mechanisms, with fit

**A. Reveal-ahead public cards with Hold and Spend costs (TI4).** Show the next era's cards before it starts (for example when the first empire enters the current era), so planning begins early. Each card has a threshold you meet personally. Half are **Hold** (state at a moment) and half are **Spend** (pay Gold or Production on the card, visible exchange rate). Spend cards make a peaceful empire's economy buy VP at a stated price the AI can value (price versus 1/10 of a win). *Uses existing systems; pure UI and card text. Peaceful: yes. Needs a war: no.*

**B. Lanes with a cap on the war lane (Scythe, Old World).** Group cards by lane (Build, Expand, Economy/Knowledge, Fight). An empire may score at most one card per era (as now), and at most **2** of its era VP may come from Fight. Peaceful win route: 5+ era VP from non-war lanes plus Great Work and goals reaches the target. *Peaceful: yes. Needs a war: only if you add C.*

**C. One required Fight VP (Old World's mandatory National Ambition; TtA).** The winning hold requires at least 1 scored Fight-lane objective, shown on the VP track as a lock icon ("Win needs 1 Fight objective"). This literally encodes "you shouldn't be able to win with zero successful wars" while 9 other VP can be peaceful. Danger: a single required card makes a weak defender a target, so the Fight card must be achievable defensively (D). *Peaceful: yes, with one forced war. Needs a war: yes, exactly one.*

**D. Relative and defensive Fight cards (TI4 secrets like "beat the leader"; Eclipse bounty; Old World "Recapture a city").** Replace raw counts with: **Defend** (destroy an invading army of N value near your cities), **Retake** (recapture a city you lost), **Strike up** (capture a city from, or destroy N of the army of, an empire with more VP or era than you), **Break the hold** (end a rival's hold, which Legatum already tracks as `FightStats`). Defence counts as a *successful war* without being world-domination. Relative goals reward fighting the leader and give a conquest path that is not farm-the-weakest. *Fits: `FightStats` already tracks captures from an empire ahead and from the VP leader and ending a hold. Peaceful: yes (Defend). AI: can price the fight it already plans.*

**E. Per-empire escalation (Old World tier = completed + 1).** The era's card tier or threshold scales with how many era VP *you* already have (or relative to a published median), so the leader's next VP costs more and the laggard's costs less. Visible as "your threshold: 12 (base 10, +2 for 4 era VP)". *Fits no-hidden-math if the breakdown is printed; adds one number per card.*

### Which mechanisms let a peaceful-leaning player win but still need a successful war?

| Mechanism | Peaceful win possible | Forces one successful war | Notes |
| --- | --- | --- | --- |
| A (reveal-ahead, Hold/Spend) | Yes | No | Biggest "earned" gain, zero new systems |
| B (Fight lane capped at 2) | Yes | No | Prevents war-only wins, enables peace wins |
| C (1 required Fight objective) | Yes | **Yes, exactly 1** | Directly encodes the designer's sentence; Old World's National Ambition is the precedent |
| D (Defend / Retake / Strike up) | Yes | Makes the one war achievable defensively | Lets a turtle satisfy C |
| E (per-empire escalation) | Yes | No | Rubber band; needs a visible breakdown |
| World domination (conquest victory) | n/a | Yes | Already a separate victory, per the brief |

Suggested stack (skateboard to car): **A with Hold and Spend cards first** (text and UI only); then **B plus D** (re-lane the existing Fight card and add Defend/Retake/Strike up); then **C** (a gate on the 10-turn hold); then **E** if sims show the leader still sails through.

### Risks and open questions
- *Required Fight VP (C).* A turtle with no neighbour at war has nothing to defend against. Needs a fallback card ("Break a hold", "Clear a camp or ruin") so isolated empires have a route. Decide whether camps count as a war.
- *Spend cards (A).* They put Gold on a visible price; if the price is too cheap the AI hoards less and the economy outscores everything. Tune so 1 VP costs about as much as the best alternative use of the same Gold for one era.
- *Relative cards (D).* "More VP or era than you" is checkable from public state, so no hidden math, but it makes the strongest empire the universal target. That is the intended rubber band (pillar 11), but should be measured so it does not become a gang-up rule.
- *Sample gaps.* Clash of Cultures cards, Millennia scoring and the FFG Civ board game were not covered. Several TI4 card texts are from memory.
