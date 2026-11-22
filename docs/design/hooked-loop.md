# The Hooked loop in Legatum: reward moments and celebration

Playtest: a ruin find was one plain "Economy" line; health bars and battle outcomes were unclear; research barely moved (0 Science before population 2; +1 Science at turn 38, with 44–160-turn estimates). The good moments exist, but the game neither shows them nor sets up the next one.

## 1. Research summary (Nir Eyal)

Pages read (September 2026): [The Hooked Model: How to Manufacture Desire in 4 Steps](https://www.nirandfar.com/how-to-manufacture-desire/), [Variable Rewards: Want To Hook Users? Drive Them Crazy](https://www.nirandfar.com/want-to-hook-your-users-drive-them-crazy/), [User Investment: Make Your Users Do the Work](https://www.nirandfar.com/makeyourusersdothework/), [The Art of Manipulation](https://www.nirandfar.com/the-art-of-manipulation/) and [How to Trigger Product Usage that Sticks](https://www.nirandfar.com/how-to-trigger-product-usage-that-sticks/).

- **The loop.** Trigger → Action → Variable Reward → Investment. Repeated passes turn *external* triggers (a notification, an icon) into *internal* ones (a feeling or routine that brings the user back unprompted).
- **Action = motivation + ability.** Eyal frames the action as two "pulleys": make it as easy as possible while boosting motivation. In the book this is BJ Fogg's model, B = MAT (behavior happens when motivation, ability and a trigger coincide). The pages above give only the motivation/ability framing; the formula is from the book. For us, "ability" is pillar 9: one obvious primary action.
- **Variable reward.** Predictable feedback (a fridge light) does not create wanting; variability does. There are three types: **tribe** (social acceptance and inclusion), **hunt** (resources and information), and **self** (mastery, completion, competence). Eyal notes that the pull fades once the mystery is gone: an emptied inbox stops tugging.
- **Investment.** This is a bit of work that makes the next pass better: time, data, effort, social capital or money. It is about *anticipated* reward, and it raises the odds of another pass when the next trigger arrives ("loading the next trigger" in the book). **Stored value** is the result: games store scores, levels and cultivated goods, which are lost if the player stops.
- **Manipulation Matrix.** Two questions: "Will I use the product myself?" and "Will the product help users materially improve their lives?" The quadrants are **Facilitator** (yes/yes), **Peddler** (no to using it, claims it helps), **Entertainer** (would use it, can't claim it materially improves lives; he treats entertainment as art, valuable in itself) and **Dealer** (neither). The Dealer offers a good time until addiction takes over.

**Game practice we borrow.** *Juice*: layered, proportionate feedback (map effect, number flying to its counter, sound, haptic). *Civ's wonders and techs*: ceremony scaled to rarity, with a quote. *Slay the Spire relics*: a named object, one line of flavor, an exact effect. *Hades boons*: variety that interacts with what your build already needs.

## 2. The loop at three timescales

Rule for every row: **the variability comes from real game state** (where you explored, what you need, what rivals are doing), and the core engine decides it deterministically. Nothing is bought, nothing is a loot box, and every reward shows its real numbers (pillar 5).

| | Moment to moment (within a turn) | Turn to turn ("one more turn") | Game to game |
| --- | --- | --- | --- |
| **Trigger** | External: an unexplored-ruin marker in sight, an enemy adjacent, a city at 1 turn to growth. Internal, over time: curiosity about the fog. | External: the **Next Turn strip** (see below) names the 1–3 things that land next turn. Internal: "I want to see what the ruin gives / whether we take the title." | External: the title screen's chronicle card ("Last game: the Byzantium fell 2 VP short"). Internal: "we could win with Athens this time"; couple time. |
| **Easiest action (pillar 9)** | One primary call to action on the thing that matters: a ruin in reach shows "Explore ruin: 2 moves" on the unit panel, preview then commit. | Every strip item is a button that selects its subject (the tech, the city, the Scout). End Turn stays the one primary action. | "Continue" or "New game with the same partner" is the primary action on the title screen. |
| **Variable reward** | **Hunt**: the ruin's reward depends on the site and your empire (see §4). **Self**: a battle won, shown with clear HP loss and outcome. | **Hunt**: what completes. **Self**: a title taken, a milestone. **Tribe**: your teammate's moments shown to you ("Alex's Scout found an archive fragment"). | **Self**: faction mastery record, new codex pages. **Tribe**: a shared team chronicle for the couple. |
| **Investment** | Moving a Scout deeper, choosing research, naming or placing a city: each makes the next find more likely or more useful. | Queued research and production, and the goto orders already set, load next turn's trigger. The strip shows the payoff date. | The game archive and chronicle entries, fragments collected, the faction mastery record. Stored value is cosmetic and informational only: **no power carries between games** (pillar 8 and fairness between partners). |

### Turn-to-turn anticipation: the Next Turn strip

At the start of each planning turn, show a compact strip of at most three forecasts, chosen by priority: *completes next turn* > *crosses a threshold within 3 turns* > *a reachable discovery*. Examples: "Ballistics: next turn", "Tessel Market grows next turn (pop 3)", "Ruin 2 hexes from your Scout", "Vessa is 1 VP from taking Largest Economy". All come from forecasts the engine already exposes; nothing is invented.

**Battle clarity** (playtest finding): each battle event animates the real HP change on both health bars (300–500 ms drain, with a numeric "−38" chip), then an outcome chip at the tile: "Won: enemy Soldier destroyed; yours 62/100". This is a *minor* moment (§3) and needs no rules change.

**Early science pacing: a rules decision for the user.** With 0 Science before population 2 and +1 at turn 38, there is nothing to anticipate: the loop's "next trigger" is 44+ turns away. Eyal's point that predictable, unreachable rewards create no wanting applies directly. **Recommendation (for the lead and the user to decide, then log in the decisions log):** give the capital a **base +2 Science**, as Civ 6's Palace does, and target the first technology in 8–12 turns with every Recovery-era estimate at 15 turns or fewer at typical play. Validate with a simulation run (research timing plus win-rate spread per pillar 8) before adopting. Until then, ruin Science should say what it buys in time ("+25 Science: Ballistics 6 turns sooner"), which is truthful and makes the find feel as large as it is.

## 3. Celebration system

One pure mapping, `momentsForTurn(events, viewer, before, after) → Moment[]`, turns the viewer's fog-filtered events into tiered moments. The core decides *what happened*; the UI only decides *how it looks*. Celebrations never block orders, any tap dismisses them, and each respects `useReducedMotion`.

| Tier | Examples | Visual | Duration | Haptic (`feel.*`) | Sound hook |
| --- | --- | --- | --- | --- | --- |
| **Minor** | Battle won, city grew, building complete, tile claimed | Map-space pulse at the tile; the number flies to its counter; report line | 600 ms, no card | Existing `battleWon` / `buildingComplete`; none for growth | `sting.minor` |
| **Notable** | A ruin find, first tech of an era, a city founded | Map burst at the tile, then a **reward card** in the moment lane (top-center, below the top bar): icon, title, exact effect, one lore line; numbers fly to counters | Card 3.5 s, auto-dismiss, tap for detail and "Show on map" | Proposed `discovery` (ruin) = `impactFeedback('light')` then `notificationFeedback('success')`; existing `techComplete` / `cityFounded` | `sting.notable` |
| **Major** | A rare or pivotal ruin (§4), first city, a tech that unlocks a Great Work, a title won, a milestone VP | Larger card with the faction frame; the VP chip flies to the public VP track; the tile glows until dismissed | Card 5 s; tap dismisses | Existing `vpGained` / `greatWork` | `sting.major` |
| **Legendary** | Great Work completed, faction goal, crossing to 1 VP from victory, victory | Full-width banner over a dimmed map with quoted lore (the Civ wonder moment); victory hands off to Game Over | 6 s banner; the map stays interactive underneath; victory itself is the only takeover | Existing `greatWork` / `victory` | `fanfare.legendary` |

- **Stacking.** Highest tier first, at most 3 cards per turn; the rest collapse into a "+N more" chip that opens the turn report there. Haptics follow haptics.md (at most 4 per turn, about 120 ms apart).
- **Voice.** Lore lines are short, in the chronicle voice of the Astrarchy's heirs ("The vault remembered its keepers. It opened for yours."). Numbers are never inside the lore line; they sit in the effect row, exact and tappable for a breakdown.
- **Reduced motion.** No translate, scale or particles: cards crossfade in at most 120 ms, counters update instantly with a static "+40" delta badge, banners appear without a sweep, and haptics still fire.
- **Sound** (later): the same `feel.*` call plays the haptic and the sound hook, so no call site changes. It is off when the device is silent.
- **Partner moments (tribe).** A teammate's notable-or-higher moment shows to you as a smaller card ("Alex's Soldier opened a ruin: +25 Science"), with no haptic and within fog rules.
- **New haptics.md rows needed before implementation:** `discovery`, and optionally `fragmentSetComplete` (= `notificationFeedback('success')`).

## 4. Ruins: varied, truthful finds

**Today** (`crates/core/src/rules/units.rs`, `explore_ruin`): a deterministic roll from `(seed, turn, RUINS, unit)` gives each of these a one-in-three chance: 40 Gold, 25 stored Science, or +1 population in the nearest city (Gold if there is no city). Byzantium doubles Gold and Science; Athens always get 30 Science. The event carries the reward as a **free-form string**, so the UI cannot fly numbers to counters or tier the moment. The first engineering fix is a structured reward, which changes data only, not rules.

**Within today's rules**, variety and tier come from the reward's *consequence* in real state:

- **Notable**: any ordinary find.
- **Major**: the find crosses a threshold this turn. Science that completes the current tech chains into a tech moment; population that makes a city reach a growth or building gate; Gold that makes a queued Buy affordable ("Now affordable: Trooper, 60 Gold").
- **Every card states what the reward buys**: turns saved on research, the new population's yield, Gold in "Buys" terms.

**Proposed rule additions (decisions for the lead and the user, flagged, not adopted):**

1. **Cartographer's cache**: reveal the map within 4 hexes of the ruin (no units or cities revealed, matching Deep Survey's rule).
2. **Dormant sentinel**: a free Scout at the ruin, rare (roughly 1 in 8). This is the "major because rare" find.
3. **Faction-flavored salvage**: keep Byzantium and Athens as they are, since they are identity (pillar 8).
4. All weights stay deterministic and seed-derived; simulate for win-rate spread before adopting.

**The Astrarchy Fragments codex (investment across games).** Every ruin also yields one **lore fragment**. The core picks it deterministically from `(seed, ruin hex)` and puts it in the event, so both partners' clients agree. There are 24 fragments in four sets of six (for example *The Relay Keepers*, *The Last Census*, *The Silent Fleet*, *The Commission*). The app keeps the codex in the local profile across games. Duplicates show as "already known" with the set's progress, and a completed set unlocks the set's full story page and a chronicle entry. Fragments grant **no gameplay power**. This is completion (a *self* reward), and the mystery narrows over many games instead of being bought.

## 5. Ethics guardrails

By Eyal's matrix we are honestly the **Entertainer**: we make it for ourselves, two people who choose to play together, and we don't claim it materially improves lives. We aim toward the Facilitator by protecting what the players value, their shared time. Rules:

- No loot boxes, paid randomness or purchasable advantage. All variability is seeded game state, shown with real numbers.
- No artificial scarcity or FOMO outside the game: no timed events, daily rewards, limited cosmetics or "come back before X".
- No streaks and no penalties for stopping; codex progress never decays.
- No notification spam. Only a real turn or partner event triggers anything outside the app (for example "Your turn: Alex is ready"), at most one per turn, with a Settings toggle.
- **Healthy stopping points:**
  - An **end-of-session summary** on Save & Quit: this session's moments, the VP change and one "next time" hook.
  - A **"good place to stop" cue** after a natural chapter: an era entered, a Great Work done, or 60+ minutes played, with "Save and stop" as the primary action.
  - **Saving is always safe**: autosave every turn and on backgrounding, plus resume exactly where you left off.

## 6. Implementation plan (skateboard first)

**1. Skateboard: ruin celebration plus the reusable framework.**
- *Core*: structured `RuinExplored { reward: RuinReward }` (Gold, Science or Population with the city and amount) in `crates/core/src/events.rs`, `rules/units.rs` and `event_visibility.rs`, mirrored in `app/src/types.ts` and `app/src/format.ts`. Filter ruin lines under a new "Discoveries" report category.
- *App*: `app/src/celebrate/` (`moments.ts` pure mapping, `MomentLane.tsx`, `celebrate.css`), a map pulse in `app/src/map/render.ts`, counter fly-to in `TopBar.tsx`, and a `discovery` row in haptics.md plus `app/src/feel/haptics.ts` and `turnFeedback.ts`.
- *Tests*:
  - Rust: each roll yields the matching structured reward; pinned seeds; fog.
  - Vitest: ruin maps to notable, a threshold find maps to major, tier ordering, collapse above 3, a rival's ruin is hidden.
  - `feel` log asserts exactly one `discovery`.
  - Playwright: a screenshot of the notable card; a reduced-motion run (`emulateMedia({ reducedMotion: 'reduce' })`) asserting `getAnimations()` has no transforms; End Turn stays clickable while the card shows.

**2. Next Turn strip and battle clarity.**
- `app/src/ui/NextTurnStrip.tsx` built from existing forecasts; HP drain and outcome chips in `app/src/map`.
- *Tests*: strip selection from fixtures; screenshots; each strip item selects its subject.
- *Separately, if the user approves*: capital base Science in `crates/core/src/data.rs` / `economy.rs`, plus a `docs/sim-results/` run and a decisions-log row.

**3. All tiers wired.** Tech, city, title, milestone, Great Work, faction goal and victory, plus partner moments.
- *Tests*: one Playwright screenshot per tier from injected-event fixtures; a mapping test per `Event` variant, so a new event without a tier fails; haptic cap of 4 per turn.

**4. Ruin variety and the Fragments codex.**
- Fragment id in the event (core, seed + hex); codex store and screen in `app/src/ui/Codex.tsx`; the flagged rule additions only after a decision.
- *Tests*: determinism (same seed and hex give the same fragment); codex persistence; set-completion moment.

**5. Stopping points and chronicle.**
- Session summary, "good place to stop" cue, and chronicle entries into the game archive.
- *Tests*: summary contents from a fixture session; cue timing with a fake clock; autosave-on-hide.

"Feel" is verified without ears or hands: the mapping is a pure function with unit tests, `window.__feelLog` records haptic and sound-hook names in order, screenshots show each tier's layout, and reduced-motion runs assert that nothing moves. Real haptics still need a device (plans/mobile.md M4).
