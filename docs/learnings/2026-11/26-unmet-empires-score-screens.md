# Unmet empires leaked through the score model, not the screens

The rivals table and Diplomacy each hid unmet empires with their own local check, but the Score ledger, the Age score check and the near-win warning read names, crests and faction goals straight from `buildVpModel`, so on turn 1 the ledger showed every rival's faction and goal. Hiding per screen misses each new screen. The fix moves the check into one place: `unmetTeam` in `app/src/empires.ts`, used by `buildVpModel`, which now names an unmet empire "Unknown empire" (row "Unknown") and flags it `unmet`; screens only decide how to draw the flag. A new score surface should read `VpEmpire.unmet` / `VpRow.unmet` rather than `view.diplomacy.met`.

Still naming unmet empires (left for a follow-up): world-event moment cards in `app/src/celebrate/moments.ts` (Royal Tomb, wonders, captures) and the Share the Learning prompt in `FreeTechs.tsx` use `empireOfPlayer` directly.
