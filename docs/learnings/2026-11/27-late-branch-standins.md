# Late-game branch measurement needs adaptive leader orders

Replaying fixed recorded orders after changing the rival AI diverges at once: orders that were legal or useful in the recorded state can miss their targets in the changed state. It is not a reliable measure of late-game resistance.

For this study, both allied leader seats use the same Hard stand-in policy in the base and candidate arms. The six rival seats alone receive the candidate genes. Each arm starts from the recorded T130 or T150 state, uses 32 paired salts, and runs through T170. Count leader-bloc city captures including razes, rival seats alive, and rival cities remaining. The snapshot source matched all 170 recorded turn hashes on its original engine. The current engine reproduces the T150 snapshot hash but changes the hash when it loads the T130 snapshot, so the T130 rows are qualified within-engine comparisons.
