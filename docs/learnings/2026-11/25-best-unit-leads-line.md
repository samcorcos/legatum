# 2026-11-25: a unit line leads with the unit you can build, not the one you lack a resource for

`unitLines` (app/src/ui/productionChoices.ts) folds a unit's upgrade chain under its newest available
unit. "Available" came from the core's options list, which includes units short of their strategic
resource (they carry `resource_blocker`). So with Feudalism and no Iron, the Pikeman led the line,
greyed out, and the buildable Spearman hid under "Also: ...". A first-time playtester took
the line as "you can't build an army here".

Fix: when the line's newest unit is resource-blocked, the newest startable unit leads, and the
blocked one shows as one line under it (`better`). "What it still does better" (`keeps`) compares
only with startable successors. This matches the AI, which already trains the best unit it can
supply (`crates/ai/src/composition.rs::best_buildable_in_line`).

Also: the advisor's ring finds a build card by `data-item`; a card inside a closed `<details>`
has no box, so the ring silently vanished. It now points at the "Also" summary until it is opened.
