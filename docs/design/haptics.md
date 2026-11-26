# Game event → haptic map

This is the single source of truth for Legatum haptics. Add a row before adding a haptic anywhere. The semantic names are the `feel.*` functions in `app/src/feel/haptics.ts`. "Viewer" means the local player whose screen is shown.

## Direct interactions (fire at the moment of user intent)

| feel call | Trigger | Plugin call | Notes |
| --- | --- | --- | --- |
| `select` | Tap a unit, city or technology to select or inspect it | `selectionFeedback()` | Also used for segmented controls, lobby options and tab switches |
| `select` | Toggle Auto-explore on or off | `selectionFeedback()` | No feedback per automated move |
| `snap` | Pinch zoom crosses a zoom level (detail ↔ overview) | `selectionFeedback()` | Once per level crossing, never continuous |
| `preview` | First tap on a destination shows a path and forecast | `impactFeedback('soft')` | Subtle "armed" feeling |
| `commit` | A unit order is confirmed (move, found, air order, goal action) | `impactFeedback('medium')` | Attack confirm uses `rigid` instead |
| `queue` | A production or research choice is confirmed | `impactFeedback('medium')` | Its own sound (a page turn); a unit order is a cloak swish |
| `commitAttack` | Attack confirmed | `impactFeedback('rigid')` | |
| `purchase` | Gold spent on a Buy | `impactFeedback('medium')` then `notificationFeedback('success')` | Only if the buy validates |
| `invalid` | A tap that can't do anything (unreachable tile, blocked action) | `notificationFeedback('warning')` | Pair it with the visible reason; never error-buzz silently |
| `ready` | Ready / End turn pressed | `impactFeedback('heavy')` | Unready: `impactFeedback('light')` |
| `longPress` | Long-press opens context actions | `impactFeedback('medium')` | At the moment the menu appears |
| `sheet` | A bottom sheet reaches a detent | `selectionFeedback()` | |

## Turn resolution (fire when the turn result is applied, in narrative order, spaced about 120 ms apart, at most 4 per turn — the most important win)

| feel call | Game event (core `Event`) | Plugin call |
| --- | --- | --- |
| `turnResolved` | New planning turn begins | `impactFeedback('soft')` |
| `battleWon` | A viewer's unit won a battle or took no losses | `impactFeedback('medium')` |
| `battleLost` | A viewer's unit was destroyed | `impactFeedback('heavy')` |
| `discovery` | The viewer opened a ruin (all reward kinds and tiers; once per turn) | `impactFeedback('light')` then `notificationFeedback('success')` |
| `cityFounded` | The viewer's city was founded | `notificationFeedback('success')` |
| `cityCaptured` | The viewer captured a city | `notificationFeedback('success')` + `impactFeedback('heavy')` |
| `cityLost` | The viewer lost a city | `notificationFeedback('error')` |
| `techComplete` | The viewer completed research | `notificationFeedback('success')` |
| `buildingComplete` | A viewer building completed (Great Work: see below) | `impactFeedback('light')` |
| `greatWork` | Viewer Great Work, faction goal or milestone VP gained | `notificationFeedback('success')` + `impactFeedback('heavy')` |
| `vpGained` | Viewer VP increased from a title | `impactFeedback('medium')` |
| `vpGained` | A treaty took effect for the viewer's team (an accepted Peace, Alliance or Open Borders offer, with any deal Gold); its moment card's tier sting voices it (`sting.notable`; `sting.major` for an Alliance) | `impactFeedback('medium')` |
| `vpLost` | The viewer lost a title | `notificationFeedback('warning')` |
| `warning` | Bankruptcy disband forecast, a city starving, or an enemy adjacent to a city (first time only) | `notificationFeedback('warning')` |
| `eliminated` | The viewer is eliminated | `notificationFeedback('error')` then `impactFeedback('heavy')` |
| `victory` | The viewer's team wins | `notificationFeedback('success')`, then after 180 ms `impactFeedback('heavy')`, then after 180 ms `impactFeedback('rigid')` |
| `defeat` | Another team wins | `impactFeedback('heavy')` |

## Multiplayer

| feel call | Trigger | Plugin call |
| --- | --- | --- |
| `partnerReady` | A teammate or other human marks Ready | `impactFeedback('light')` |
| `allReady` | The last human readies and the turn begins resolving | `impactFeedback('medium')` |
| `seatTaken` | Your seat was taken over on another device | `notificationFeedback('warning')` |
| `teamPing` | A teammate pings a tile (not your own pings) | `impactFeedback('light')` |

## Never

- Pan, drag, scroll or inertia; tooltip open or close; toast dismiss; any AI-only event the viewer can't see (fog: only use the viewer's filtered events); repeated identical events in one turn beyond the first.

## Celebration sound hooks

Each accepted `feel.*` call may carry `sting.minor`, `sting.notable`, `sting.major`
or `fanfare.legendary`. Tier hooks share the same debounce and four-call turn cap;
they do not add a second haptic or sound. A wired tier hook replaces the semantic
sound, otherwise the existing semantic sound plays. `discovery` and all four hooks ship CC0 Kenney sounds in the audio manifest: a
single pluck, short pizzicato, fuller pizzicato, and an orchestral Hit fanfare.
Interrupted celebrations cancel only their decorated beats; losses still play. Partner celebration cards have no haptic.

## Battle and land sound (no haptic)

These are heard, not felt, and are not `feel.*` events: the map schedules them itself (`app/src/feel/battleSound.ts`, `app/src/feel/sound.ts`).

| Sound | Trigger | Limit |
| --- | --- | --- |
| Bow, arrow hit | An archer or crossbow volley in the turn's battle animation leaves, and lands if it did damage | 2 each a turn |
| Catapult throw, stone hit | A stone shot leaves and lands | 1 throw, 2 hits a turn |
| Hooves, clash | A melee clash before gunpowder; hooves lead a mounted charge | 1 hooves, 2 clashes a turn |
| Death cry | A pre-gunpowder unit falls | 2 a turn |
| Soldier falls | A soldier of the gunpowder age or later falls (a second set of cries). A destroyed machine (tank, artillery, aircraft, drone) sounds the explosion instead; a Settler is silent | 2 a turn |
| Gunshot | A melee of rifle-armed units meets (the Industrial era or later) | 2 a turn |
| Musket | A musket volley leaves, or a melee of musket-armed units meets (the Renaissance) | 2 a turn |
| Drone | A combat drone flies in | 1 a turn |
| Cannon | A cannon fires | 2 a turn |
| Shell fire, explosion | Artillery or a tank gun fires, and its shell lands; a bomber's bombs land | 1 firing, 2 explosions a turn |
| Aircraft, jet | An aircraft's run, 1.5 s before its bombs are released: the propeller recording for the Biplane, Fighter and Bomber, the jet recording for the jets and stealth aircraft | 1 each a turn |
| Collapse | A city is razed | every one |
| Land and sea loop (birds, wind, cold wind, waves) | The land or sea under the middle of the map view, read every 1.5 s; two readings must agree | one loop, on the Effects bus at half level |

At most 10 battle sounds a turn, chosen by consequence in the order collapse, death, soldier falls, explosion, clash, gunshot, cannon, musket, shell fire, aircraft, jet, drone, stone hit, arrow hit, bow, throw, hooves. The same sound inside 160 ms is one sound. Battle sound follows the map-animation setting.

Music follows the listener's era as well as its people: tracks marked `eras` in the manifest play only then (the 0 A.D. music in eras 1 to 3, the FreeCol and Unknown Horizons music in eras 4 and 5, the Battle for Wesnoth music in eras 6 and 7, the Warzone 2100 music in eras 8 and 9), except a people's opening track, which stays in rotation all game. A new era never cuts the track that is playing.

The land and sea loop is one hook, `map/useLandSound.ts`, used by both the 2D and the 3D map. Battle sound is scheduled by the 2D map only, because the 3D map does not draw battles yet; when it does, it calls `playBattleSounds(battleSoundCues(plan), elapsed)` against its own animation clock.
