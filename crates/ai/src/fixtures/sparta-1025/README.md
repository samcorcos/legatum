# Sparta 8.9.10 historical fixtures

Engine `8e9433e5cf53ea4ffe4e8729d5904c6320cb519a`, setup seed `7732777812994077`.
Source: immutable copy `sparta-win-final-20261025-0902.legatum`; all 207 tape hashes verified.
See `docs/sim-results/ai-quality/sparta-game-1025.md` and `tools/ai-quality/sparta-1025/`.

Views are the actual fogged Japan PlayerViews. The T142 memory is the original brain's
pre-turn memory after reproducing every previous AI plan. The state is the original
WASM JSON; the test converts only setup.seed from its string encoding. The two order
maps are copied from tape turns 142 and 143, in that order. Gzip mtime is zero.

SHA-256 of decompressed JSON bytes (orders were serialized from the two tape maps):

- `T129-p2.view.json.gz`: `f32cf14cdc9871b8dd228cbf1c8825bf8c2413ad8a586aa8c21942b14b7770d0`
- `t142-143.orders.json.gz`: `47395a0216dfb1cc4516c6ea3b6c885b0018f724f02b1ec78c69f6f17604d468`
- `t142-japan.memory.json.gz`: `66e34eab389436ce52bb4f3cb752eeb05780c20670bfd666a7928a9de4626435`
- `t142-japan.view.json.gz`: `7209a63c72db6f545672697da36942677e7807959ebb07a5083493f2da534002`
- `t142.state.json.gz`: `d4104967f0428ae98109cc80a3fca29cae33d4118296a736ed87658bac68234b`
