# Frozen scripted AI v1

Source: `b638adfb0a72990cd78b59c4177c171d9aa39855` (worktree starting HEAD, contains v1 merge `94f9404`). `git diff 94f9404..b638adfb0a72990cd78b59c4177c171d9aa39855 -- crates/ai` was empty at freeze time.

The eight policy modules were copied before any B9 policy edits, including the two reviewed defects. The compatibility adaptation replaces `crate::` with `crate::ai_v1_baseline::` so internal module imports resolve inside this integration test. `mod.rs` reuses the public `legatum_ai::Brain` trait. Rustfmt also wrapped the longer `military.rs` import. No policy or core compatibility changes. Both policies link the same `legatum-core` in one test binary.

Eight-faction compatibility: the exhaustive knobs match now gives new factions neutral (50) knobs. The original three policies remain unchanged; this is not an eight-faction v1 balance baseline.

Full-tree compatibility (2026-09-27): exhaustive matches give Assault Trooper and Inference Core the same zero value as other post-v1 additions. No v1 strategy is added.
