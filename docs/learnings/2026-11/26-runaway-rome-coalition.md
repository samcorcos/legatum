# Rivals already fight a runaway Rome; declaring more wars doesn't stop it

2026-11-26. Measured with `crates/sim/examples/rush_ladder.rs`.

**Arming against a visible build-up** (`buildup_match`, before T60 train the cheapest melee unit while a nearby rival's public army is bigger than ours): on 300 rusher games it raised survival to T100 from 39% to 43% (significant) with capital loss 67% to 64% (noise). In 400 full games (1 arm seat vs 5 built-in Hard) wins were flat (15.5% vs 15.8%). Dropped.

**Capital-taker coalition** (`capital_coalition`, declare on whoever took an original capital in the last 15 turns): with every seat on, 300 full games, capital-takers won 16% either way. Dropped from consideration.

**From real runaway positions** (`--tape SAVE --handover T`: replay a recorded game's orders to turn T, then every seat is Hard AI): on two recorded Rome games that reached 11-12 cities by T100, handing over at T80 or T100 Rome won 128 of 128 runs, with or without the coalition (results identical: it never fired). Instrumenting it showed why: from T80 every rival is already at war with Rome, so "no other war" blocked it every turn. Rome ends with 25-29 cities.

Lesson: against a runaway conqueror the AI's problem is not declaring war, it is that the rivals at war with it don't take anything back. The next lever is how rivals fight a war they are already in against the leader (combined fronts, retaking captured cities), not diplomacy.

The handover replay differs from the recorded hashes on every turn (the engine has moved on since), but the replayed empire's city counts track the original game, so the positions are close.
