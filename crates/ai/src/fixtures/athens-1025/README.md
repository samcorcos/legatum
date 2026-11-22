# Athens 8.9.10 positions, 2026-10-25

Copied from a hash-checked replay of `athens-win-swiss-tepid-20261025.legatum` on engine
`8e9433e5cf53ea4ffe4e8729d5904c6320cb519a`; setup seed `7970005121204451`.
The source save remains private and unmodified. No `.legatum` is committed.

`Tn-pP.view.json.gz` is player P's fogged view before turn n.
`memory` is its original pre-plan brain memory; `recorded` is its entire recorded order list.
The committed views and state use the current building and goal enums. The source save
and recorded orders remain unchanged.
T80/177/185/210/214/220 Switzerland and T214/219 Germany cover the two Swiss wars,
late siege stalls, Germany's failed hold defense and its mispriced final capture.

`T210.state.json.gz` is the captured full state, for resolution only. The fixture loader converts
the JSON seed string to a u64, as the WASM parser does. `T210-215.orders.json.gz` holds those
six turns' recorded orders for all players. The intervention replaces only Swiss orders and
recomputes its own fogged view/memory each turn. No hidden state enters its planner.

The six-turn regression uses **current rules**, not a claim to reproduce old-engine hashes.
The all-turn hash/brain audit separately uses the original engine. Fixed historical opponent
orders do not model human adaptation. The nearby-Mongol negative test changes only a copy
of the T210 view, adding a visible enemy to verify that genuine home defense still wins.

See `docs/sim-results/ai-quality/athens-game-1025.md` and `tools/ai-quality/athens-1025/`.
