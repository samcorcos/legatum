# Payoff moments on the map

The user (2026-10-24): "all payoff / investment / building / conquering etc moments should have a rewarding animation." The code is `app/src/map/momentFx.ts` (the plan, one per resolved turn, a pure function of time) and `momentFxDraw.ts` (the drawing), on the combat animation's layer. Everything follows "Map animations" (On or Off; Off shows the new state at once) and `prefers-reduced-motion`, and nothing blocks input. Clips are real turns from the designer's 8.9.6 game, filmed frame by frame at 60 fps (`app/e2e/media/moments-fx.media.ts`, on the cloud box).

## What ships

- **Borders:** any tile changing hands (growth, a new city, a capture, a release) fills from the side facing its city, swept around it, before the real border takes it.
- **Capture:** the banner turns over to its new owner and the land floods outward, both when combat-fx's capture lands (`captureTime`), not at the start of the turn.
- **VP and crowns:** "+N VP" set small in the house serif on a vellum slip, over a fine gilt rule that draws out from its centre; a crown names its title in italic under it. Merged per empire per turn.
- **Treaties and wars:** a small wax seal pressed in quietly over each capital (a ring for Peace, linked rings for an Alliance, a cross for War).
- **Roads** run out from the city that laid them.
- **Units:** a soft band of light passes over a promoted or upgraded unit; marching units stride and leave a little dust.
- **Idle life:** smoke over towns of city size and up and glints on the water, redrawn at 15 fps.
- **Cards:** the technology unlock card's seal settles in and its painted unlocks wash in; the era takeover lays its painting in as an ink wash, with parallax and the rule drawn out.
- **Great Work landmarks (`landmarks.ts`):** a finished Great Work stands on its chosen tile, with an icon badge when zoomed out. The public city view also exposes an unfinished work and its stage.

| Clip | What happens |
| --- | --- |
| `capture-lugano-t200.mp4` | Lugano falls: combat-fx plays the assault; the banner turns over and the land floods in the taker's colour; the turn's VP slip and seals over capitals. |
| `tech-unlock-card-t151.mp4` | The technology unlock card. |
| `era-takeover-industrial-t151.mp4` | The era takeover. |

## Construction: a site advances each turn

A queued Great Work or capturable faction-goal wonder stands on its chosen tile immediately; its public city record includes the tile, building, progress and stage, so a rival's explored city shows its site through fog. The site passes from plot to footing at 10%, half built at 40%, roofline at 75%, then the finished landmark. These are the aligned painted construction stages, with ink stock, a crane beside the painting, workers and scaffold on the painted outline. Walls grow clockwise round the town from the gate. A city founding uses the painted hamlet stages. Ordinary buildings have a one-second banner meter and icon beat; they do not grow a map miniature.

`constructionSites.ts` derives the site from the view each frame. `momentFx.ts` plans a 1.2-second step for any own site whose production rises, even when no completion event fires, and holds the preceding site until that step. A finished work takes 2.5 seconds: last stage laid upward, furniture struck, dust settled and its name lettered under the painting. `landmarks.ts` keeps the name under the finished work. The shared map animation layer controls the step; Map animations Off and reduced motion show the resulting site or landmark instantly.

Review clips (archived outside the repository): wonder151.mp4 (in docs/design/construction/clips) follows a staged Great Library from the T112 save through the build with one locked camera; `walls63.mp4` shows Winterthur's staged Walls from T61; and `found63.mp4` follows the new city. The saves are copies of the designer's 8.9.6 game; the original orders tape is unavailable, so the clips reconstruct the needed founding order and stage Bellinzona's Great Library on its chosen tile. Film them in Chromium on the cloud box with `npx playwright test -c playwright.media.config.ts e2e/media/moments-fx.media.ts`; `MOMENT_FRAMES` sets the frames per turn.
