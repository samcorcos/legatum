# Celebration overlay slots

The game screen declares these slots with `data-overlay-slot`; the geometry is in
`app/src/styles.css` and `app/src/touch.css`. `celebrate.css` owns only celebration styling.
All celebration tiers use the same reserved story lane.

| Slot | Wide viewports | Compact viewports (960 px and below) |
| --- | --- | --- |
| `top-bar` | Gold, Science, Research, Turn, Menu | Five bounded controls below 700 px; stored Science and total population stay out of the phone bar |
| `victory-race` | 320 px right column | 100 px strip below the top bar, with every team's crest, score and track; tap for the breakdown |
| `story` | Left of the VP column; beyond the left selection dock on wide desktop | Compact banner below the VP strip, at most 184 px / 25% of viewport height; major adds a gold band and crest glow, legendary a gold caption strip and rays |
| `story-report` | Same horizontal reservation as story; below any remaining card and overflow chip | Below the card/chip while they remain, otherwise below the VP strip |
| `story-hint` | Existing centered hint slot; waits for celebrations | Below the VP strip |
| `counter-flight` | Pointer-transparent layer above the cards and HUD, clipped out of the VP race | Same clipping; hidden counters are never flight destinations |
| `selection-dock` | Left column / bottom unit bar | Below the card/chip while they remain, otherwise below the VP strip; choosing a unit interrupts celebrations |
| `turn-dock` | Bottom right | Existing bottom turn controls remain usable |
| `map-preview`, `map-compass` | Map-local context and decorative compass | A map gesture interrupts the story before opening a preview |

**Tech unlock card (the one exception to the lane).** Every technology you finish earns a centred
card (`.moment-card.unlock`), not a lane card: it is `position: fixed` over the map, raised above every dock
(`.moment-lane:has(.moment-card.unlock)` takes z-index 12), with a pointer-transparent scrim, and it declares no
overlay slot. It is held until the player acts: no timer, and no click, map pan or opened report closes it
(`interrupt` is a no-op while it is up). Only its own controls, Esc and Return close it, and each closes only
itself, never the cards queued behind. Return takes its primary ("Show <item> in <city>"; "Choose research" when
research is idle and nothing can be built; else "Continue"); with idle research and a building to show, "Choose
research (T)" sits beside Later. Return and Esc are consumed, so End Turn and the map do not also act, and the
dock's own Return badge is hidden meanwhile. End Turn stays clickable under it. While any `<dialog>` is open the
card waits (it is not rendered, so it never hides behind the dialog, and Esc goes to the dialog first), then
returns. The turn report's "researched X · unlocks Y" line reopens it (`unlockReopen` in the store; it plays
alone). A research finishes at most once per empire per turn, so there is at most one such card per turn; a
partner's tech stays a small lane card. There is no "No benefit" card: a tech that gives a faction nothing is skipped at game start (rules 49, `data::skipped_techs`), so every researched tech has something to show.
Reduced motion removes the arrival and ray animation like every other card.

**Era-up takeover (the second exception).** Your own `EraEntered` holds a full-screen takeover
(`.era-takeover`, `position: fixed; inset: 0`; `.moment-lane:has(.era-takeover)` takes z-index 80) that covers the HUD and
declares no overlay slot. It follows the unlock card's contract: no timer, a click or map pan never closes it, Return takes
its primary ("Choose research" when nothing is queued, else "Continue"; research picked in pending orders while the card waited counts as queued, `celebrate/researchSettled.ts`, which does the same for the unlock card) and Esc is Later, each closing only itself, and it
waits behind any open `<dialog>` and returns. It is local to the viewer's screen, so it never blocks a teammate's turn.
`handleOverlayKey` (`gameplayKeys.ts`) treats it like an unlock card, so the open turn report does not eat its Esc or Return.
The animation is under 3 s and every key or the button skips it at once; reduced motion shows the finished scene. The
sting is the organic japan (`fanfare.legendary`) through `soundHooksForMoments`.

The report's requested-open state is retained while the provider holds the story slot;
it becomes visible after the **first** card expires or on any tap/Escape. Later cards continue
in the lane above the report. Tapping details or Show on map releases the report without ending
the sequence. Opening the report explicitly ends the entire sequence. First-game hints wait
until all cards end, even when no report was requested. Outside controls interrupt on click,
so moving the phone dock cannot steal an in-progress tap; map gestures interrupt on pointer-down. The overflow chip exists
only during a sequence and counts only notable-and-above moments.

Cards do not steal focus. Show on map is always a bordered primary link, including on
phones; it uses the existing interruptible 380 ms camera pan and leaves the card visible.
Legendary alone can also pan automatically (380 ms) to keep its celebrated tile clear of the
measured cards, report and controls; an already-visible tile stays put. Reduced motion snaps
that adjustment. Reduced motion uses static rays, a fixed ruin mark/reveal boundary, the Scout
at its real neighbor, and static counter deltas. No camera or counter flight is animated in that mode.

`app/e2e/celebrate.spec.ts` checks both looks at 390, 430, 1024 and 1440 px, tests card/text
bounds throughout arrival, captures every visible tier, and checks real first-turn
ruin feedback, report deferral, interruption and reduced motion. These browser specs
must be run by the reviewer: this worktree's sandbox cannot launch browsers.
