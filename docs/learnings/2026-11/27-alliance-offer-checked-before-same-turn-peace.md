# An Alliance offer was validated before the same turn's Peace

An AI could accept a Peace and offer an Alliance on the same turn. Each order passed
`check_order` against the state before resolution, but once the Peace was signed it carried a
guarantee, and the Alliance would have made the proposer defend its new partner against that
same empire. The offer waited in the receiver's inbox, and accepting it failed.

Validating orders one at a time against the pre-turn state cannot see combinations. The fix is
to re-check outstanding Alliance offers against the post-resolution state at the boundary
(the same `check_alliance` the accept path uses) and withdraw the ones that fail, plus an AI
guard so it doesn't make the wasted offer. Any order whose legality depends on other orders
resolved the same turn needs the same post-resolution re-check.
