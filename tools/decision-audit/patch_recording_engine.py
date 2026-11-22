#!/usr/bin/env python3
"""Add the audit-only dice stream to a temporary recording-engine worktree.

Keep that engine's actual rules: copying current core files into an older tag would
change the game being measured. Fail if its RNG sites no longer match this patch.
"""
from pathlib import Path
import sys


root = Path(sys.argv[1])
if "pub dice_seed: Option<u64>" in (root / "crates/core/src/state.rs").read_text():
    raise SystemExit(0)


def edit(path, old, new, count=1):
    file = root / path
    source = file.read_text()
    actual = source.count(old)
    if actual != count:
        raise SystemExit(f"{path}: expected {count} instances of {old!r}, found {actual}")
    file.write_text(source.replace(old, new))


edit(
    "crates/core/src/state.rs",
    "pub struct GameState {\n",
    "pub struct GameState {\n"
    "    #[serde(default, skip_serializing_if = \"Option::is_none\")]\n"
    "    pub dice_seed: Option<u64>,\n",
)
edit(
    "crates/core/src/state.rs",
    "impl GameState {\n",
    "impl GameState {\n"
    "    pub fn dice_seed(&self) -> u64 {\n"
    "        self.dice_seed.unwrap_or(self.setup.seed)\n"
    "    }\n",
)
edit("crates/core/src/game.rs", "    GameState {\n", "    GameState {\n        dice_seed: None,\n")
for path, count in (
    ("crates/core/src/rules/battle.rs", 2),
    ("crates/core/src/rules/movement.rs", 1),
    ("crates/core/src/rules/units.rs", 1),
):
    edit(path, "state.setup.seed,", "state.dice_seed(),", count)
edit(
    "crates/core/src/rules/barbarians.rs",
    "Rng::derive(state.setup.seed, &all)",
    "Rng::derive(state.dice_seed(), &all)",
)
