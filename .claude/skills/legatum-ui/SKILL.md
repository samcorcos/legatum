---
name: legatum-ui
description: This skill should be used when designing, building or reviewing any Legatum UI (screens, panels, map rendering, reports, tooltips, art, mobile) or when the user gives UI feedback. It records the user's UI preferences and recurring complaints so new UI meets them the first time.
---

# Legatum UI preferences

Pillars 5 (no hidden math) and 9 (the next action is always obvious) from CLAUDE.md govern everything here.


This file holds the current rules, one line each. The user's words, dates, as-built notes and review findings are in `references/` (index at the bottom); search them before proposing UI that might already be decided.

## Sizes and verification

- The user's laptop window is about 1000x600 CSS pixels: the primary desktop size, not an edge case. Verify at 1000x600 and 1440x900 (and phone width for layout changes) with headless Playwright screenshots, never the app's browser pane.
- Every new screen or decision card joins the screen tour (`app/e2e/tour/`) with a fixture in the same change. Fixtures use shipping names and states the rules can produce; park the mouse before shots.
- Test layouts with two humans as well as one; test controls with their longest label.
- A rewrite that drops an established interaction (click targets, shortcuts) is a regression: list a screen's interactions before rewriting it (user, 2026-10-11).
- Never override a stated user preference for a general guideline; satisfy both or ask (user, 2026-10-17).
- Question every requirement on every screen: review each screen and map piece for what can be removed or merged without losing a decision, and before each release check one icon, word, colour and component per concept (user, 2026-11-22). Art quality belongs to `legatum-art-reviewer`.

## The next action is always obvious (pillar 9)

- One unmistakable primary action per screen, visible at 1000x600 in every seat setup; anything clickable looks clickable before hover; one live Return primary at a time.
- Keyboard first: Return confirms or advances, Esc closes; Escape never throws away something he has to answer, and the turn report (L) brings back whatever it hides (user, 2026-11-21).
- Invalid choices are greyed out with the reason next to them, in one line, where the price would be; never accept an order that will fail (offer "Declare war on X?" instead); a broken order is the next decision ("Fix order").
- A card held until the player acts closes only by its own controls; acting on a decision card closes it; a card that announces a finished mechanic offers the next idle decision as a button.
- A background failure (upload, save, network) is a small corner notice, never a card over a decision (2026-11-07).
- A held card's call to action follows live state when it shows, not the state when it was planned (PR #32, 2026-11-22).
- Every new Age opens a score check for every player (user, 2026-11-22; `celebrate/ScoreCheck.tsx`): the VP track plus your sheet of ways to score, fitting 1000x600 whole; any VP you have is a filled box, held vs kept a second mark; a "how to score" line names the action. Detail: `references/review-findings-2026-11.md`.
- The hosted title screen (uploads on) has about 10px to spare at 1000x600: the upload notice stays 2 lines; any further hosted-only line needs a layout change. Shoot it separately from the standard tour build, which has uploads off (PR #33, 2026-11-22).
- The tour shoots turn 1 once with the first-game hints on (`e2e/tour/first-run-hints.spec.ts`; every other fixture disables them), and a hint names only what exists at 1000x600: "Press T or click Research", not "Open Technologies" (PR #30, 2026-11-22).
- Never a blank page: a boot plate before React mounts, and one full-screen crash card with Reload as the primary (`ui/CrashScreen.tsx`). Saves never fail silently: a failed write shows one notice whose action is Save to file; replacing a saved game asks once with focus on the safe choice (2026-11-22).
- A confirm dialog opens on the safe choice and Esc means the safe choice; a dialog Esc closes also shows a visible close control; a button's label says what the click does (2026-11-22 reviews). A turn-failure dialog (`TurnFailureDialog`) follows that: the X sits top-right with `title="Close (Esc)"`, its destructive button says whose orders it drops (several human seats on one engine: "End the turn with all orders cleared"; alone: "with my orders cleared"), and a status note ("Saved.") clears when the dialog reopens for the next state (PR #30 review, 2026-11-22).
- A refusal from the game server is a short plain notice with one next step, never "relay", "socket" or a code; offer a retry only when it can work (relay review, 2026-11-22).
- A new order kind goes into `ORDER_KINDS` (`app/src/orderDecoding.ts`) in the same change, with a tour shot of the pending-orders line (PR 31 review, 2026-11-22).
- Online: an action that ends a shared game says so and asks first, with the label branching on host or guest; status text is written from the viewer's role; durability text matches the storage behind it (no "saved" for a tab-lifetime copy) (co-op review, 2026-11-22).
- Big moments get a full interstitial: war declared on you, entering an era, the end of the game; a rival closing on victory and a city under attack are never a surprise (user, 2026-10-08, 2026-11-11).
- A menu action that can fail shows its busy and failed state on the control the player pressed, never on a neighbour. A flow that resumes a shared game says who still has to come back, and how.

## No hidden math (pillar 5)

- Every number has a breakdown on hover or click; hover is for reading, click opens breakdowns; a hoverable bubble never covers its trigger.
- A number shown is the number in force, and a limit that applies is named in the same line (user, 2026-10-17). Label gross versus net; each number once per card; one source per number.
- Forecast headlines include every consequence (walled-city fire, Siege); a line that changes a number shows both numbers.
- Costs are loud (red), including ongoing upkeep on every build card; neutral "nothing yet" lines are muted ink, never green; colour is from the viewer's side.
- Anything that ticks over time has a progress bar with turns left; a goal's progress is a meter with its number.
- Every way to score is visible, with ownership unmistakable (user, 2026-10-06); a scoring option that becomes available is announced every time (user, 2026-11-11).

## Map

- Territory is readable at a glance; borders and cities stay live through fog; overlays and lenses are translucent with the terrain readable under them (user, 2026-10-08/09, 2026-10-04).
- Map options are stackable overlays (Yields, Resources, Borders) plus one exclusive lens; the Settle lens shows every legal site on an absolute scale in shades of one green, with a plain one-sentence hover, and never depends on a Settler (user, 2026-10-11/17).
- The map never moves under a mouse that is aiming (user, 2026-11-11). Clicks never shift content under the pointer (user, 2026-10-06).
- Map art never hides yields; every map bonus is visible on the tile and named wherever its number lands (user, 2026-10-11). Tile tooltips lead with the most valuable thing: wonder, resource, terrain.
- Every unit names its owner in words and emblem, never by colour alone; each faction has its preferred colour in every game (user, 2026-10-18, 2026-11-11).
- No raw coordinates anywhere: name places relative to cities.
- A tile-chosen build is placed on the map like Civ's districts; buying a tile starts from one obvious button (user, 2026-10-29, 2026-11-20).

## Cities, units and lists

- Build lists are dense; no line holds one short item; locked upgrades nest under the building they follow (user, 2026-10-28, 2026-10-18).
- Faction-accurate everywhere: show only what helps your faction, with its replacements; when a faction loses a yield, hide it (user, 2026-11-22, Egypt).
- A unit has two idle buttons: Fortify (civilians: Wait; F) and Skip this turn (Space), plus Heal (H) when damaged (the internal order is still named Sleep); every military unit can auto-explore with E (user, 2026-10-20, 2026-11-01, 2026-11-14).
- A unit a rule holds in place shows it on the map and card before you try to move it; a released garrison returns to the queue (user, 2026-11-11, 2026-11-15).
- Aircraft are units in the turn rotation (user, 2026-10-18). A unit with two ways to attack shows both and picks nothing silently.
- A long list of cards has type-to-filter and shows nothing that cannot be used (user, 2026-11-14).
- Eliminated empires leave active lists; the race panel always shows the race (extras shrink first) (user, 2026-10-06, 2026-10-17).
- Lobby: Easy, Normal and Hard only; a new Single Player lobby starts at the map size's default count (6 on Standard); choosing a map size snaps the count; no game-type preset bar (2026-11-21, user 2026-11-03, 2026-10-06, 2026-10-27). Two settings never share a name (user, 2026-11-15).
- No "coalition" in the UI; say the plain facts (user, 2026-11-11).

## Look and feel

- Premium means imagery and iconography: painted art, portraits where a leader is the subject, celebrations for rewards, a staged reveal at the end of a game (user, 2026-10-06, 2026-10-17). Art direction lives in `legatum-art`.
- Every faction sheet has the same shape: strengths left, costs right, a drawback beside the ability it pays for, the special unit and building painted, each rule once, the goal in one block, ending "Full rules in How to Play" (user, 2026-11-15, 2026-11-21).
- Finishing a technology feels like an unlock (Civ 6 style); unlock hovers sell the unlock with big art and a comparison (user, 2026-10-05, 2026-10-11).
- Sound is organic and in-world, never robotic; frequent clicks need a quieter sound (target player, 2026-10-09, 2026-11-20).
- Small layout rules (details in `references/feedback-rules-2.md`): one glyph, one meaning; an idle city's first build card is whole at scroll 0 at 1000x600; a relation badge is never cut; a status word comes from one shared check; auto-scrolling a panel keeps its identity header in view; pickers start on the map; per-tile action lists go after the main groups.
- Amounts carry their yield icon and are upright lining figures; text fits its box, wraps rather than truncating meaning, and never sits under a badge.

## Game text: plain language

Every word a player reads (sheets, cards, the Guide, tooltips, breakdowns, report lines, refusals) follows this guide (user, 2026-10-17: "'per complete pair' is not normal language... is there a rule... to make sure we use normal language"). The cause of the bad wording was agents translating code literally (`floor(pop / 2)` became "per complete pair"). Write what the player gets, not how the code computes it.

- **Tone:** a good board-game rulebook, or Civ's Civilopedia at its clearest. Plain, warm, exact.
- **The card says the rule; the Guide says the fine print (user, 2026-11-21, on the Celts sheet line "Barbarians never attack the Celts, all game. No camp, scout or raider touches your units, cities or Settlers, and the warband camps you raise raid your rivals only."):** "a typical board game ... would tell you the simple explanation and then in the rules, there would be a much more detailed explanation in the rule book. e.g. this one might say 'Barbarians don't attack the Celts' and that's about it. Then all the nuance of other things (can i attack them? can i take camps? when i attack them, does it still do damage?) get handled in the rule book." A sheet row, card, tooltip, tip, notice or report line is one short sentence (about 15 words, two at most) that a player can act on. Exceptions, what it doesn't cover, interactions and edge cases go in the Guide (`GuideContent.tsx`; a faction sheet row's longer `detail` is what the Guide prints). Drop emphasis that adds no rule ("all game", "always"). The headline must still be true: for every faction that sees it ("up to level 5" is false for Sparta), in every era it applies to (Mongols build Walls from the Medieval era), and everywhere a shared string is reused (a unit's rule on a tech card must still name the unit). `legatum-plain-language-reviewer` checks this on every PR that changes player text.
- **A card's way to the rulebook (UI review, 2026-11-21):** a faction sheet ends with "Full rules in How to Play", which opens the Guide at that faction's own rules. A link-style button is at least 13px in the accent colour with a `:focus-visible` ring, so it reads as clickable before hover. A deep link into a modal scrolls only the modal's content pane, never the dialog shell: the title and Close stay in view.
- **A number the player can change is never stated as fixed (user, 2026-11-22, on the title screen's "You against five AI empires": "it could be more or less than that"):** say the range ("1 to 7 AI empires", built from the constants) or show the live count; "five" is only true as "five by default".
- **Numbers first, in the player's terms:** "+1 Gold for every 2 population", not "per complete pair of population". "Up to 100 HP", not "(max 100)". "At least 1 move", not "(minimum 1)".
- **No mechanics-speak or code terms,** unless the game teaches the word: floor, rounded down, complete, held (for Gold: say "in your treasury"), accrue, tick, live or permanent VP source, base (strength, cost), adjusted, multiplier, stage, view, excepted, retain, provided that, mutually exclusive, Σ, max/min, i.e./e.g. A rounding rule is never worth a clause: state the number or say "about".
- **In-theme words for mechanics:** interest (Swiss Gold from the treasury), tribute (bought peace), levy (Byzantine armies), the agoge, a vow (Sakoku). Lore may be colourful; rules lines are plain.
- **Every rule fits one plain sentence** (user, `legatum-design`). If a rule needs a paragraph of exceptions to say, flag it as a design smell rather than writing the paragraph.
- **One idea per sentence,** at most about 30 words, no more than one semicolon. Active voice, second person ("you", "your cities"). No stacked qualifiers ("each military unit beyond your first two, Scouts excepted, per turn"): split them into sentences.
- **Say what it means for the player and when it matters,** not how it is implemented. "Your units deal half damage to city HP", not "half damage, rounded down after the attack roll and any siege bonus".
- **A live goal says how it is lost** in a sentence: "You lose the VP while your treasury is below 1,000 Gold", never a "Live:" tag.
- **Text shown for another empire is third person** ("its land"), and shared rule text built by one function never says "you" for a rival.
- **The same term everywhere** for the same thing. The glossary below is canonical; its synonyms are lint failures.
- **A drawback sits beside the ability it pays for** (user, 2026-10-17, Germany's sheet: unrelated rows side by side read as related). A faction sheet row pairs a strength with the price that offsets it; a strength with no matching price stands alone across the row (`FactionPair.price` is optional). A "drawback" that only restates its ability is not a drawback: cut it.
- **A faction sheet row is the rule, not its fine print (user, 2026-11-05, Swiss sheet: "i think we can simplify this"):** one idea per card ("Only Switzerland can buy a world wonder" (superseded 2026-11-06: Swiss wonder buying is cut and nobody buys wonders); "Military unit upkeep is triple: 3 Gold a turn instead of 1"). Leave out exemptions every empire shares (Scouts pay no upkeep for anyone), consequences the player can work out ("can finish a contested wonder immediately", "can delay the reserve goal"), and one example item's price standing in for a class (a Great Library price when every era has its own wonder). Two unrelated drawbacks are two cards, not one paragraph.
- **Compare a replacement with what it replaces in the same breath:** "Ruhr Foundry: +6 Production (a Forge gives +3)", never a bare "+3" that leaves the player asking whether it is on top.
- **No standing explainer notes** that restate what each line already says (a "A live goal gives VP only while..." note under goals that each say how they are lost). Say it once, where it applies.
- **Every form of a template reads well:** check the one-item and many-item forms ("moved 1 tile, now 1 tile E of Kyoto" as well as "moved 3 tiles, now ..."), the at-Peace and at-War forms, and your own and a rival's.
- **A note under a breakdown uses the breakdown's own split** (a growth note saying "30 + 16 per population" over a table of "Population growth + Game pace" is two stories).

**Glossary (canonical terms):**

| Term | Means | Never |
| --- | --- | --- |
| tile | one map cell; distances are in tiles ("within 5 tiles") | hex, hexes (code and docs may say hex) |
| population | a city's size | pop, citizens |
| Food, Production, Gold, Science | the four yields, always capitalised | food, hammers, gold |
| city HP | a city's own health bar (200 at full) | city health, "the walls" |
| city strength | the city's combat strength; Walls add to it | city defence strength |
| Walls | the building only (Castle and Star Fort upgrade it); "walled city" is fine | walls (lowercase) for city HP |
| military unit | any unit that fights; say "except Scouts" when a rule leaves them out | armed unit, regiment, troops |
| VP | victory points (the Guide spells it out once) | victory points elsewhere |
| World Age card | the Age's one card, worth 1 VP (rules 50; "era objectives" is retired) | era objectives, era goals |
| decadence | the cut to a leader's yields | lead penalty |
| at Peace, at War, Alliance | treaty states, capitalised | at peace, at war |
| supply | the Iron or Coal your deposits give; each building or unit that needs it uses 1 while it exists ("Iron 5 / 6") | stockpile, stored Iron/Coal, Iron per turn, Coal per turn |
| interest | Swiss Gold earned from the treasury | treasury bonus |
| Grow | Egypt's project that grows a city with its surplus Food instead of building (2026-11-21); "surplus Food" is Food beyond 2 per population | growth project, grow mode |
| tribute | Byzantium buying peace | bribe, peace payment |
| level | a unit's promotion level ("level 4 or higher") | rank, promotion tier |
| technology | in rules text; "tech diffusion" is the one fixed phrase | techs |
| British spelling | defence, labour, centre, armour, honour, colour, favour | defense, labor, center (in player text; code identifiers may differ) |

**Enforced:** the copy lint (`app/src/test/copyRules.ts`) runs from `ui/textArtifacts.test.tsx` and `ui/copySource.test.ts`; the `legatum-plain-language-reviewer` checks player text on every PR. Before-and-after examples: `references/plain-language-examples.md`.

## References (open only when needed)

| File | Open it when |
| --- | --- |
| `references/feedback-rules-1.md` | You need the user's exact words or as-built details for a rule above (first half: Escape, sheets, lists, map, sound, goals, orders, forecasts, banners, faction selection) |
| `references/feedback-rules-2.md` | The same for the second half (unlock cards, disabled reasons, menus, rewards, tour, overlays, Settle lens, premium tiers, lobby, VP surfaces, alarms, diplomacy, aircraft, end of game) |
| `references/process-and-screens.md` | Working on a specific screen as built: research, era objectives and World Age cards, Settle lens, overlays, Cities overview, race panel, unlock hovercards, piece comparison, Siege, Byzantium screens |
| `references/review-findings-2026-10.md` | Checking what an October UI review already ruled (8.9.6 to train-1101, Mercenary Companies) |
| `references/review-findings-2026-11.md` | Checking what a November UI review already ruled (train-1103 to the game-upload notice) |
| `references/plain-language-examples.md` | Rewriting player text (before/after table), the copy lint, the Scenario Lab screen |
