## 2026-11-27: Garrison age is not in the live view

- Cause: `UnitView.occupation_lock` identifies the rival holding a unit in place, but the live `PlayerView` has no capture turn or lock start turn. The chronicle is available only after the game ends.
- Evidence: `app/src/types.ts` exposes the lock on units and cities; `PlayerView.history` is documented as finished-game data.
- Change: The advisor stores the turn when this browser first sees each locked unit. Its ten-turn Peace reminder uses that observed duration, so a lock already present when a game is opened may take longer to prompt. A second locked unit triggers the reminder immediately.
