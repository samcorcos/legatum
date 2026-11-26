# End Turn was one long task; most of it could happen before the click (2026-11-26)

Second of the fixes for the "very slow UI" report. Measured with `?perf=1` (the store's `legatum:endTurn`,
`readViews`, `applyTurn`, `writeAutosave` measures) and a long-task observer, in headless Chromium with a
development WASM build, three End Turns from a turn-121 and a turn-280 save.

Before: End Turn was a single task of 133 to 156 ms at turn 121 and 229 to 308 ms at turn 280 (the first after a
load more: 438 and 605 ms). Inside it, `end_turn` was 53 to 131 ms and the autosave 34 to 154 ms. Natively,
`turn_profile` puts two thirds of `end_turn` in the AI seats planning (21.5 of 31.6 ms at turns 220 to 280).

Why not a worker: the page asks the engine questions synchronously on hover (validate, forecast, assault), so the
main thread needs its own engine at the current state. A worker that resolves the turn would have to hand the
state back, and `Game.load` of a late save is 40 to 110 ms on the main thread, about what End Turn costs. Moving
the whole engine behind an asynchronous API is a rewrite of every caller.

What changed instead:

- **The AI plans while the player does.** A seat's orders depend only on the state at the start of the turn, and
  nothing changes that state until the turn resolves. `Session::plan_ahead` plans one seat per call and keeps the
  orders; `take_ai_orders` uses what is planned and plans the rest. The app calls it from idle callbacks, starting
  1.2 s into a turn. However often it is called, the game is the same: a test plays 25 turns planning all, some
  or no seats ahead and compares orders and state hash with a plain game each turn.
- **The autosave is written after the new turn's first frame**, in its own task. The next End Turn and the page
  going away (`pagehide`, hidden) write it first, so the recovery save is never a turn behind when it is needed.

After: `end_turn` is 13 to 26 ms, and the steady-state End Turn has no task over 60 ms at either save. The first
End Turn after a load is 148 to 156 ms (the turn's views and playback, not the engine).

Gotchas:

- Only the local game's `end_turn` path uses the planned orders. Anything that calls `Session::ai_orders` directly
  (the online host, the sims) plans every seat afresh, so `plan_ahead` must not be called on those sessions: a
  brain asked twice in one turn is not guaranteed to answer the same.
- A panic while planning ahead leaves the WASM module unusable, as one in End Turn does. The app reloads the last
  good save and plans nothing more that turn, so End Turn meets the same failure where the player is told.
- Loading a game drops what was planned; a probe that calls `client.load` mid-turn measures an unplanned turn.
- A training game plans every seat at the start of the turn already (`pending_ai_orders`); `plan_ahead` then has
  nothing to do.
