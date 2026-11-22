# Eras, tech and catch-up: a design research brief

Researched 2026-10-09 for the "turn 170 and bored" problem (design doc: mid-late game boredom, research and
era overhaul requested by the user). Nothing here is a decision; it is evidence and ranked options.

**The problem in numbers (from our own rules and sims).** 32 techs in four eras (`data.rs::tech_era`:
Recovery 7, Consolidation 9, Integration 9, Inheritance 7). Winners finish with 5-14 techs, so most games
never leave eras 1-2. Five contested titles (1 VP each) all follow size. The win is 10 VP held for 10 turns
at Standard, so the last third of a game is a countdown with nothing to decide. "Most Advanced" compares
science over the last 5 turns (`SCIENCE_HISTORY_TURNS`), not techs known. Tech diffusion is -10% per
known empire, max -30%.

**Evidence quality.** DuckDuckGo HTML served a CAPTCHA (not bypassed); CivFanatics and Reddit were walled.
Fandom pages were read via the public MediaWiki `api.php`, the rest via plain `curl`. Soren Johnson's and
Jon Shafer's own talks and essays could **not** be retrieved; their views come only from the secondary
sources named. **[memory]** marks background knowledge that was not re-verified.

---

## 1. Era and age systems

### Civ 6: eras, era score, ages (read: Fandom Era (Civ6), Age (Civ6))
- You enter the next individual era when you own one tech or civic from it, or all of your current era.
  So the gate is "first tech of the next era", not "finish the era". Beelining is allowed.
- Rise and Fall added a **World Era** that moves on the median progress of all civs (10-turn warning,
  with min and max turn counts). Techs from earlier eras cost **20% less**, later eras **20% more**. This is
  a built-in diffusion and anti-beeline lever. Warmonger penalties also grow each World Era (note: that
  discourages the war we want more of).
- **Era score** comes from Historic Moments (first contact, natural wonders, first unique unit, etc.). At
  each World Era change, a low score gives a **Dark Age**, a middle score a Normal Age, and a high score a
  **Golden Age**. A Golden Age raises the next thresholds (+5 per Golden Age, +1 per city); a Dark Age lowers them
  and unlocks Dark Age policy cards. Filling the Golden meter *from* a Dark Age gives a **Heroic Age**
  (three Dedications instead of one). Points above the Golden threshold are wasted.
- Good: a public, scheduled re-scoring moment and a self-correcting comeback loop. Weak (**[memory]**): the
  effects are mostly modifiers (loyalty, a policy slot), so players feel eras mainly through units.

### Civ 7: three Ages (read: Firaxis Dev Diary #1, Fandom Age/Legacy Path/Crisis (Civ7), Wikipedia, Steam thread)
- Firaxis's diagnosis: late-game Civ is "just not that fun"; causes were **snowballing** ("your choices and
  decisions have little effect on the ultimate outcome"), micromanagement growth, and civs strong in only
  one phase. Over half of Civ 6 players never finished a campaign.
- Each Age (75-125 turns per the wiki; the diary said 150-200) brings its own civs, resources, techs, civics,
  units, buildings, wonders and **game systems**. **Age Progress** rises every turn and jumps with Legacy
  milestones; researching future techs "ahead of their time" speeds the Age, "penalizing civilizations that
  have fallen behind"; eliminating a civ also advances it.
- **Legacy Paths**: four per Age (Science, Culture, Economy, Military), up to 3 Legacy Points each. Each
  milestone also advances the Age, so "it is often impossible to complete all four": forced specialisation, a
  race to be first, and players delaying milestones to control the clock. A full path gives a Golden Age Legacy
  for the next Age; missing a milestone gives a Dark Age Legacy.
- **Crisis** starts at 70% Age Progress: a themed event, then **mandatory negative Crisis Policies** (1 at 70%,
  2 at 80%, 3-4 at 90%) and escalating negative events. Firaxis wanted "a jolt of variety" so the end "feels
  climactic". **Transition**: all players move together, pick a new civ and which Legacies to carry; tech trees
  restart, old buildings are "overbuilt" at reduced yield, leaders persist.
- **Reception.** Wikipedia: "mixed" Steam reviews, 48% recommending months after launch, with the "heaviest
  criticism" aimed at "the abruptness of the ages system and the immersion breaking of changing civilizations";
  also a tech tree ending around the 1960s. A Steam post: "two separate games followed by a tiebreaker";
  "what I did in one age doesn't matter as much as it should in the next". The "Test of Time" update (19 May
  2026) reportedly lets players keep one civ. The wiki says Legacy bonuses often lead "to a snowballing effect".
- Analyst read (ultimatewarlord.com): a chapter break only fights snowballing if (1) the leader must make a
  *choice*, (2) the trailing player gets a *new contest*, not a free handout, (3) the old chapter still
  matters, and (4) conversion rules are visible. Prefer **conversion** to reset.

### Humankind: era stars and fame (read: Fandom Era Star, Fame)
- Victory is **Fame** only, after a set turn count or end-game trigger. Fame is earned mostly from **Era Stars**:
  generated objectives in 7-8 categories (Builder, Militarist, Scientist...) with 3 stars per category per era;
  the star matching your culture affinity gives **double Fame**. You need **7 stars to enter the next era**
  and may stay longer to farm more, but star Fame **decays each turn**.
- **Competitive Spirit**: when someone else enters a new era first, everyone else gets a catch-up star
  category that grants stars passively each turn (faster as more empires arrive) but gives no Fame.
  This is the cleanest "catch up on access, not on score" mechanic found.
- Era-stage scoring means different empires lead different eras, and the category structure forces breadth.

### Old World, Millennia, Ara, Through the Ages, Stellaris
- **Old World** (Soren Johnson): per-turn *orders* are scarce, so a big empire does not mean a big turn; scope
  was limited to early eras on purpose (Wikipedia). Random tech-draw details are **[memory]**, unverified.
- **Millennia**: 10 ages that all players enter together when techs are researched; each age after II offers
  **Classical / Variant / Crisis / Victory** versions chosen by your achievements (e.g. a "Crisis" age caused by
  low sanitation or too many conquests). Reviews were mixed overall, but the "variant age system" was
  "praised for its uniqueness" (Wikipedia). Closest existing model to "eras that react to what players did".
- **Through the Ages** (board game): four ages; cards, events and military come from the current age's deck;
  culture points come from many sources and can be stolen by aggression (Wikipedia; deck detail **[memory]**).
- **Stellaris**: **galactic crises** exist "to address a common late-game problem... one faction is so
  powerful that their eventual victory is inevitable" (Wikipedia); a player can become the crisis.

### Lessons for Legatum (ranked)
1. **Make an era a shared, public event that changes the questions** (Civ 7 per-age systems; Humankind
   Competitive Spirit; Endless Legend deeds). Entering era N should unlock a new unit class, a new building
   family, and a new set of public goals for *everyone*, with a "first in" reward. This answers "we have no
   planes or tanks" and "when someone gets ahead it's a big deal".
2. **Convert, don't reset.** Civ 7's reset is its most criticised feature and Firaxis is walking it back.
   Legatum has 32 techs in one tree; keep it, and add per-era VP stages (section 4) rather than a new civ.
3. **Drive era change from the median or first mover, with a visible countdown** (Civ 6: 10-turn warning,
   +/-20% cost). A clock tied to the leader's tech lets the leader end the Age on everyone else (Civ 7).
4. **Make the last third a staged crisis** (Civ 7 at 70%, Stellaris), not a passive timer. Our 10-turn
   hold is already one; give it decisions (section 3).
5. **Catch-up grants access, not points** (Humankind). Our diffusion (-10% per empire, max -30%) has the
   right shape but is too small and invisible to matter.

---

## 2. Tech tree structure

### Gating rules across the genre (read)
| Game | Gate to the next tier | Cost shape | Late-game choice mechanism |
|---|---|---|---|
| Civ 6 | Own one tech from the next era (or all of your era) | Fixed per tech; World Era +/-20%; **Eureka/Inspiration** -40% (-50% before Rise and Fall) | Eurekas steer order; few real dead ends |
| Endless Legend | **9 techs from any earlier era** opens the whole next era; no per-tech prereqs | `Base * (N^2 + N + 6)`, N = techs already known | Free choice inside an open era |
| Stellaris | **6 techs from the preceding tier in the same area** make a tech a candidate | +0.1% per empire-size point over 100 | **Hand of 3-4 random cards**, weights (rare techs 1/8, repeats x0.5) |
| Beyond Earth | Tech **web**: one prereq per tech, so you can ignore whole directions | n/a | Affinities unlock unique units; no tech trading |

Endless Legend is the nearest precedent for the "fractional gate": it asks for breadth (nine techs) but
allows free ordering, and its rising cost per tech known slows leaders. It also makes any empire reaching an
era unlock that era's **Legendary Deeds for everyone** (read), which doubles as catch-up and a public event.

### Civ 6 Eurekas and Inspirations
A boost is a small objective tied to one tech or civic (build a Quarry for Masonry) that grants 40% of its cost
(50% before Rise and Fall) and can finish the tech. It gives a cheap, local reason to pick one tech over another,
which is what "click the closest" lacks. Weakness **[memory]**: once known, boosts become a checklist.

### Diffusion and catch-up costs
- Ours: -10% per known empire, max -30%. Civ 6: +/-20% by World Era. Endless Legend: cost grows with *your own*
  tech count. Stellaris: with empire size (punishes breadth; **conflicts with pillar 10**). Relative position
  (Civ 6, ours) is the safest lever.

### The user's idea: finish every tech in an era before the next
**For.** Eras become real (everyone shares a toolset for a long time), the leader cannot skip to a late unit on a
thin economy, the tree is fully visible, and it is easy to teach.
**Against.**
1. *It does not fix "click the closest".* If every tech must be researched, order is the only choice left,
   and with 7-9 techs per era most orders are equivalent. It makes the click *more* arbitrary.
2. *Reach.* Winners today finish 5-14 techs. A strict gate would put most games in era 1 or 2 (era 2 needs 7
   techs, era 3 needs 16, era 4 needs 25). Either science must roughly triple or the eras never appear.
3. *Diversity.* Pillar 10 (strategic diversity, each faction leans into a play style) wants specialised
   research, not a forced march. A rush faction needs Ballistics early, not Hydroponics first.
4. *Slow players are punished twice* (they cannot advance, and they get the cheapest era's units).
**Known middle grounds** (all read above): (a) **N of M** gate (Endless Legend 9 of the previous tiers; Stellaris
6 per area); (b) **first tech of the next era** (Civ 6) plus a World Era cost swing; (c) **capstone**
techs: a short list inside each era that must be owned to enter the next, leaving other techs optional;
(d) gate by **objectives** rather than techs (Humankind's 7 stars).

### How games keep late research meaningful (and avoid "closest")
- Make the *effect* a new verb, unit, building or doctrine, not "+10% science" (Beyond Earth, Civ 7).
- Offer **mutually exclusive forks** or a limited hand so that choosing is also declining (Stellaris cards;
  Old World bonus cards **[memory]**). With random draws, keep *which cards are in the deck* and the odds public
  to satisfy "no hidden math".
- Tie the choice to *who you are* (Humankind affinity x2; Beyond Earth affinities; our factions) and to
  *who you fight* (counter-tech); Eureka-style local objectives keep players looking at the map.

### Lessons for Legatum (ranked)
1. **Do not force full-era completion.** Adopt a fractional or capstone gate instead (e.g. 6 of 9 techs, or the
   era's 2 capstones) and retune science so that era 3 is reached by a median winner around turn 110-130.
   Validate with the sim "techs known at win" histogram, not by feel.
2. **Make techs unlock things you can see on the map** (unit classes, building families, a verb), and make
   at least one fork per era exclusive. Today's early techs are mostly +1 yield or a building; that is why order is
   arbitrary.
3. **Add local Eureka-style discounts** (-40% if you built X / captured Y). Cheap to explain, forecastable,
   and it turns the research screen into a to-do list tied to the map. Fits pillar 5 (every choice a forecast).
4. **Keep diffusion relative, and make it visible**: a "behind the world era" discount of 20-30% beats
   -10% per empire. Add a line in the tech card ("-20%: 3 empires know this").
5. **If you add randomness, make it a visible hand of 3 from a public pool**, not a hidden draw.

---

## 3. The 4X late-game problem

### Evidence (read)
- **Firaxis (Ed Beach, Civ 7 Dev Diary #1)**: late game is not fun; three causes (snowballing, micromanagement
  growth, civs strong only in one phase); metric: more than half of Civ 6 players never finish. Their fix was
  structural: split the game into chapters with their own systems.
- **Wikipedia, 4X**: players criticise "excessive micromanagement"; late turns can take "several minutes";
  AI governors were criticised for "poor decisions".
- **Jon Shafer, At the Gates** (Wikipedia): a "mixed" 4X whose headline flaw was "passive AI and a flawed economy"
  (PC Gamer). Shafer also criticised his own AI design for Civ 5. Relevance: an AI that is at war but does not
  attack is a *known* failure mode, not only our bug; a passive AI removes the mid-game threat that makes
  research and armies matter.
- **Soren Johnson**: his own view could not be retrieved (a Substack "summary" read as machine-generated and
  was not used). Secondary: Old World bounds turns with scarce orders and focuses on early eras.
- Civ 7's own rewards can feed the leader (Legacy bonuses "snowball"; see section 1), so a catch-up system
  needs its own audit for who benefits.

### Remedies that exist in shipped games
| Remedy | Examples | Cost / risk |
|---|---|---|
| **Decision density** (a scarce per-turn resource) | Old World orders; Legatum's pillar 3 | Needs UI to show the budget |
| **New mechanics per era** | Civ 7 per-age systems; Beyond Earth affinities | Content cost; AI must learn each |
| **Crises** | Civ 7 (70%/80%/90%); Stellaris; Millennia crisis ages | Random negatives feel punitive; Civ 7 mixed |
| **Coalitions / leader-bashing** | TI4 agendas **[memory]** | Can feel like punishing skill |
| **Staged objectives** | TI4 Stage I/II; Humankind stars; Civ 7 Legacy | Needs tuning so no one path dominates |
| **Delegation / automation** | Civ 7 goal; 4X governors (criticised) | Poor automation is worse than none |

### Lessons for Legatum (ranked)
1. **The passive-AI problem comes first.** If AIs at war do not attack, no catch-up rule matters. The Hold
   countdown should be a *trigger* for them: when a team has held target for N turns, every other team's AI
   gets a "break the hold" objective with a forecasted payoff (capture a city or lose a title = hold reset).
   The rules already reset the hold on any VP loss; check in the AI whether that is valued enough (unverified).
2. **Make the hold phase a crisis with choices**, not a wait. Each hold turn the holder picks one of two
   costs (Civ 7 Crisis Policies), and each rival picks one of two pressure options. Public and forecast,
   not random events.
3. **Coalition as a mechanic, not a flag.** Today `coalition_target` is only a derived warning. Give an
   active coalition a concrete benefit (shared vision, -25% to attack the leader, tech diffusion from the
   leader) so that "gang up on the leader" is something a human can *do*.
4. **Every era needs one new thing to do each turn** (a verb or system), not bigger numbers; swap, don't stack
   clicks (pillar 3). Delegate the routine (auto-queue, standing orders) so the turn stays the same size.

---

## 4. Victory-point structures that resist correlated leads

### How shipped designs do it
- **Twilight Imperium 4** (Fandom Objectives, Wikipedia): target 10 VP. **Public objectives** are revealed
  one per round: Stage I (1 VP, five of them) then Stage II (2 VP, the first after all Stage I are out). A player
  may score **at most one public objective per Status Phase**, and must hold every planet in their home system
  to score. **Secret objectives** (1 VP; keep one of two at setup, max 3) are personal. So the *rate* of scoring
  is capped (no one can take everything the turn it appears), the pool *changes every round*, objectives are of
  different kinds (resources, tech, fleets, planets), and secret objectives mean different players chase
  different things. Agendas (voting weighted by planets) add a political layer **[memory]**.
- **Civ 7 Legacy Paths** (section 1): milestones advance the Age so you cannot do everything; rewards are
  paid in next-Age options, not an immediate lead. **Through the Ages**: culture comes from many sources and the
  card decks change per age **[memory]**.
- **Humankind Fame**: points come from per-era generated objectives in 7-8 categories, the category that fits
  your culture pays double, and Fame decays. You are scored on *what you did this era*, not stock.
- **Civ 6 era score**: thresholds rise after a Golden Age and fall after a Dark Age, a built-in negative
  feedback loop on the "who is ahead" state.

### Why Legatum's titles correlate
LargestEconomy, MostPopulous, MostTerritory, StrongestArmy and MostAdvanced are *stocks* that grow with city
count, and "strict highest wins, incumbent keeps ties" favours the incumbent. A leader can hold up to 5 VP from
titles, so needs only 5 more from milestones and goals. Milestones (FourCities, SixTechs, TwentyPopulation)
are *thresholds* that every competent player hits, so they do not separate teams.

### What would break the correlation (design rules from the above)
1. **Cap the rate**: at most one (or two) public goals scorable per team per era or per N turns (TI4).
2. **Score flows, not stocks**: "most techs *gained this era*", "most cities captured this era" (Humankind,
   Civ 7). A big, stable empire gains no extra.
3. **Normalise by size** where a stock is unavoidable: per-city science, tech per city, territory per
   city, army value per population.
4. **Stage it**: era N goals only exist in era N; title-style awards are fixed at era end and become permanent,
   so different teams can win different eras.
5. **Different offers per player**: a small public hand drawn per faction (Humankind affinity bonus, TI4 secret).
   Secret goals clash with pillar 4; a *public* per-faction hand keeps it visible.
6. **Pay in options, not points** where possible (Civ 7 Legacies): the trailing team picks first.

### Lessons for Legatum (ranked)
1. **Replace the five live titles with era-staged public goals** (3-5 per era, 1 VP each, one scoring slot per
   team per era; a 2-VP "capstone" per era). Keep 1-2 live titles at most, and only the size-independent ones.
2. **Redefine titles as per-capita or per-era**: "Most Advanced" = techs gained in this era; "Largest Economy" =
   income per city; "Strongest Army" = army value per city. Cuts the size correlation without nerfing size.
3. **Add a catch-up category that pays access** (Humankind Competitive Spirit): when a rival enters an era first,
   everyone behind accrues partial progress toward that era's goals each turn.
4. **Reveal next stage's goals one era ahead** on the VP track (pillars 4 and 5).
5. **Re-derive the target after staging** (10 VP + 10-turn hold may be too hard) by sim, about 2,000 games per
   speed (pillar 8), not by feel.

---

## 5. Warfare across eras

### What the references show
- **Endless Legend** (Fandom): reaching an era trains units at higher level, unlocks equipment and reveals
  resources and tiles; the era also opens a deeds list for all players. Units get better *because the world
  entered the era*.
- **Beyond Earth**: three affinities unlock unique units; the tech web lets a player ignore directions,
  so armies differ by research path (Wikipedia). No tech trading, and techs can be stolen.
- **Alpha Centauri**: customisable units, native life as a third party, Secret Projects as race goals
  (Wikipedia); weapon, armour and chassis chosen separately **[memory]**.
- **Civ 6** **[memory]**: era units (spearman to musketman to rifleman to infantry; aircraft and armour)
  create the "tanks versus spearmen" window, which is what makes a tech lead *feel* decisive and gives the
  leader a reason to attack and the laggard a reason to fear.

### Where Legatum stands
Combat units are Soldier, Ranger, Trooper, Marksman, Siege Engine and Assault Trooper (`UnitKind`); no naval,
air or armour. Combined Arms, Siege Engineering and Powered Armor exist but barely change the shape of war.
If every army is the same generation, defence is cheap and attacking does not pay, which may add to the AI's
passivity.

### Lessons for Legatum (ranked)
1. **One signature unit class per era, with a counter** (e.g. era 2: mounted or mobile infantry; era 3: armour
   walkers; era 4: aircraft or drones). Each beats the previous era's best at near-equal cost, so a tech lead
   creates an attack window. Rock-paper-scissors across eras (pillar 10).
2. **Make the window a public clock**: when a team gets an era unit first, others see it ("first in Integration")
   and get a 10-20 turn catch-up discount for that unit class's tech. Fits pillars 4 and 11.
3. **Convert, don't reset**: old units upgrade for gold, so a trailing team can leap with one big spend and
   the leader keeps experience.
4. **Do not escalate the war penalty with era.** Civ 6 raises warmonger penalties each World Era; Legatum
   needs the opposite (the AI must be willing to attack). Keep war cost flat; make *holding* the target costly.
5. **Cheap first slice**: one new unit class plus the tech that unlocks it, then measure attacks per game in
   the sim. Armour and drones are cheap; naval and air need new movement rules.

---

## Overall: design patterns ranked by fit to Legatum's pillars

Legend: pillars F=familiar to Civ 6, T=turn size, V=victory race visible, N=no hidden math, S=simultaneous,
D=strategic diversity, C=competitive to the end.

| # | Pattern | Source | Fit | Clashes / risks |
|---|---|---|---|---|
| 1 | **Era-staged public goals** (3-5 per era, one scoring slot per team per era; replaces live titles) | TI4, Humankind, Civ 7 Legacy | V, N, D, C strong; T neutral; S fine | Needs sim retune of the 10 VP + 10-turn hold; if goals are secret, clashes with V |
| 2 | **Fractional or capstone era gate + world-era clock** (6 of 9, or 2 capstones; 10-turn warning; +/-20% cost vs the world era) | Civ 6 World Era, Endless Legend, Stellaris | F, N, D, C strong; T neutral | Clock tied to the leader hands the leader the pace; use the median or a floor |
| 3 | **Era-signature unit class with a counter and a public "first in" window** | Endless Legend, Civ 6 units, Beyond Earth | F, D, C strong; V good | Content cost; AI must learn; roster growth adds clicks (T) unless delegated |
| 4 | **Catch-up for access, not points** (Competitive Spirit stars, world-era discount, EL deeds for all) | Humankind, Civ 6, Endless Legend | C, N, V strong; S fine | Must stay small, else leader feels punished; no free yields (pillar 6) |
| 5 | **Hold phase as a crisis with visible choices, plus a mechanical coalition** | Civ 7 Crisis, Stellaris, TI4 agendas | C strong, V strong, D good | Random negative events feel punitive; keep choices public; risk of anti-leader "punish skill" feel |
| 6 | **Techs unlock verbs and Eureka-style discounts; at least one exclusive fork per era** | Civ 6 Eurekas, Beyond Earth | F strong, N strong, D strong, T neutral | More data per tech card; needs UI; the discount list becomes a checklist |
| 7 | **Per-capita and per-era titles** (techs gained this era, income per city) | Humankind, Civ 6 era score | V, C, D good; F fine | Harder to explain than "largest"; weakens the size reward if overdone (pillar 10) |
| 8 | **Visible hand of 3 research options from a public pool** | Stellaris, Old World **[memory]** | N ok if odds shown; D good | Unfamiliar to Civ 6 players (F weak); randomness vs pillar 5 |

### Patterns that clash with the pillars (avoid or contain)
- **Hard reset / civ switching (Civ 7)**: breaks F, hurts D (a lead is erased) and is the most criticised
  element (48% Steam recommendation, Firaxis reversing). Do not copy; convert instead.
- **Force every tech in an era** (user's idea, strict form): fails D, makes "closest" worse, and is out of
  reach at 5-14 techs. A fractional gate keeps the benefit (real eras) without these costs.
- **Empire-size research tax (Stellaris)**: punishes expansion, contradicting pillar 10 ("tall is not viable").
  Tax techs known or relative lead instead.
- **Secret objectives (TI4)**: add variety but clash with V and N. Use a public per-faction hand.
- **Age clock driven by the leader (Civ 7 future-tech acceleration)**: the leader controls when the
  game ends. Use the median and a floor.
- **Random negative events (Civ 7 crises)**: feel like the game cheats; prefer public, chosen costs.
- **Escalating warmonger penalty (Civ 6 World Era)**: the opposite of what a passive AI needs.

### Suggested order (skateboard, per `rules/skateboard-delivery.md`)
1. *Skateboard*: sim-only. Change the gate to 6 of 9 per era and add the world-era cost swing; measure techs
   at win, era reached at win, and attack counts. No UI change.
2. *Scooter*: one signature unit class (era 2 or 3) with its counter and a "first in" banner.
3. *Bicycle*: era-staged public goals replacing three of the five titles; retune the VP target by sim.
4. *Car*: hold-phase choices, mechanical coalition, per-capita titles, exclusive forks and the Eureka list.

## Sources actually read (all fetched with `curl`)
- Firaxis, Civilization VII Dev Diary #1: Ages: https://civilization.2k.com/civ-vii/news/dev-diary-1-ages/
- Fandom wikis via `https://<wiki>.fandom.com/api.php?action=parse&page=<page>`: civilization (Era, Age, Eureka
  (Civ6); Age, Legacy Path, Crisis (Civ7); Technology (CivBE)), humankind (Era Star, Fame), endlesslegend
  (Technologies), stellaris (Technology), millennia (Ages), twilight-imperium (Objectives)
- Wikipedia: Civilization VII and VI, Humankind, Old World, Through the Ages, Twilight Imperium, At the Gates,
  Stellaris, Beyond Earth, Alpha Centauri, Millennia, Ara, Polytopia, Endless Legend, 4X, Technology tree
- https://steamcommunity.com/app/1295660/discussions/2/598519066984975760/ (Civ 7 age-transition feedback)
- https://ultimatewarlord.com/articles/what-civilization-vii-ages-teach-about-snowballing/
- Not readable (walled), worth a logged-in pass: CivFanatics Civ 7 snowball and Legacy Point threads; Reddit
  r/OldWorldGame "Soren Johnson's designer notes"; designer-notes.com interviews.
