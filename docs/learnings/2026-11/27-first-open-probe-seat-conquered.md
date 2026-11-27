# The first-open probe lost its city screen when its seat got conquered (2026-11-27)

The nightly lean check builds a turn-40 save from one fixed seed and opens each screen on it as seat 0. After the
AI learned to finish off a weak neighbour early, seat 0 in that seed was dead by turn 40. The probe could not open
the city screen (no city to select), and the other screens were being measured for an eliminated player.

`tools/perf/lean_check.py` now tries seeds 7000001 to 7000008 and keeps the first where seat 0 is alive with a
city (`seat_holds_city`). On 2026-11-27 that is 7000002; the city screen opens in 261 ms and fetches nothing.

A probe that depends on how one seed plays out breaks whenever the AI changes. Check the precondition in the
save, not the seed.
