# LAN hardening verification (2026-09-26)

Verified on `astra/lan-server-hardening`, with master merged through `ba3fa6a`.

| Check | Result |
| --- | --- |
| `cargo fmt --all` | Passed, exit 0. |
| `cargo fmt --all -- --check` | Passed, exit 0. |
| `cargo clippy --workspace --all-targets -- -D warnings` | Passed, exit 0. |
| `cargo test --workspace` | Passed, exit 0: 283 passed, 0 failed, 3 ignored. |
| `WASM_PACK_CACHE=/tmp/legatum-lan-wasm-cache ./scripts/build-wasm.sh` | Passed, exit 0; release WASM built and optimized before the app checks. |
| `cd app && npm test` | Passed, exit 0: 117 tests across 12 files, including the real WASM store tests. |
| `cd app && npm run build` | Passed, exit 0; TypeScript and Vite production build. |
| `git diff --check` | Passed, exit 0. |

The ignored Rust tests are the existing opt-in `midgame_speed`, `paired_evaluation`,
and `many_games_summary` performance/evaluation runs. The WASM cache override keeps
the build tools inside a writable directory in the worktree's sandbox.

Regression tests were added and observed failing before the production fixes:

- Cross-origin/Host requests reached the handlers, and POSTs without a JSON media
  type were accepted. Tests now cover the three HTTP admin routes and the real
  WebSocket upgrade, trusted loopback/LAN URLs, default port 80, duplicate headers,
  spoofed forwarding headers, and rejection of non-loopback admin peers.
- Changing only unseen terrain changed MoveUnit validation. Tests now compare
  identical PlayerViews over different hidden terrain/connectivity, and exercise
  generic OrderFailed reporting through a real server turn.
- Welcome omitted retained DisbandUnit orders; the client/store lost the list and
  could enable Ready or replay stale orders. Tests now cover reload, takeover,
  missed turns, replacement games, cancellation, and same-turn unsent edits.
- Snapshot/pruning failures replaced the committed autosave. Tests now inject those
  failures and verify both the autosave bytes and restart behavior after a rejected
  game replacement.

HTTP/WebSocket tests use real ephemeral TCP listeners. The non-loopback router test
injects a non-loopback peer identity so it does not depend on a configured LAN
interface. Header-rejection tests use small bodies because rejection deliberately
happens before body consumption; a large rejected upload can yield a broken pipe
instead of a readable HTTP response. Full save/load tests still use actual saves.

The optional `.reviews/astra-lan-security-review.md` did not exist when this task
started. No physical two-computer LAN session has been verified.

## Staging room / B23 (2026-09-26)

Implemented on `astra/lan-staging-server`, based on `d87f744` (the merged LAN
security fixes). Staging is enabled by default when status advertises support.

| Check | Result |
| --- | --- |
| `cargo fmt --all` and `cargo fmt --all -- --check` | Passed, exit 0. |
| `cargo clippy --workspace --all-targets -- -D warnings` | Passed, exit 0. |
| `cargo test --workspace` | Passed, exit 0: 306 passed, 0 failed, 4 existing ignored. |
| `WASM_PACK_CACHE=/tmp/legatum-staging-wasm-cache scripts/build-wasm.sh` | Passed, exit 0; release WASM built and optimized. |
| `cd app && npm test` | Passed, exit 0: 133 tests across 14 files. |
| `cd app && npm run build` | Passed, exit 0; includes type-checking the Playwright suite. |
| `npx -y playwright@latest test --reporter=line` | Blocked at browser launch in this macOS sandbox; neither of the two scenarios reached a page. |

The real HTTP/WebSocket tests cover staging start and first turns with 1, 2, 3,
and 8 human clients; disconnected-seat reclamation; configuration changes and
re-readiness; occupied-seat refusal; stable slots; close/leave; invalid configs;
AI difficulty, team compaction and Random faction resolution at full u64 seeds;
atomic save failure; and the direct Host Game path with 1-8 humans. Admin tests
cover Host/Origin, content type and non-loopback peers on the new routes, including
DELETE. Unit tests cover host-slot loopback identity, coalesced staging snapshots,
slow-consumer eviction, and clearing stale snapshots on close/start.

Vitest regressions cover exact settings acknowledgment, delayed/serialized writes,
reverted edits, request failures, close ordering, disconnected reclamation, and
closing a row without moving its neighbor. Playwright's checked-in two-context
scenarios exercise those controls against the real Rust server, including a held
settings POST, guest reload, start, and first turn; **they remain unverified**.

Playwright 1.63.0 was installed via `npx -y playwright@latest`. Chromium fails with
`bootstrap_check_in ... MachPortRendezvousServer ... Permission denied (1100)`;
Firefox aborts with SIGABRT; WebKit aborts with exit 134. All fail before opening a
page. Browser/cache paths were under `/tmp`; no sandbox relaxation was attempted.
Rerun the README browser commands in an environment that permits launching a
browser. No physical two-computer session was run.

The fourth ignored Rust test is the pre-existing opt-in
`recorded_corpus_states_and_serialized_views_match` cross-build corpus check.

## Protocol connection reset (2026-09-27)

Investigated on `astra/server-test-flake`, starting from `130d59e`, using Rust
1.98.1 on macOS. The unchanged protocol suite was run 60 times: 50 default
invocations, five with `--test-threads=1`, and five alongside
`cargo test --workspace`. Separate repetition lanes ran concurrently; every
invocation ran all 34 tests, including the real 30-second pre-join deadline.

One loaded invocation failed in
`staging_credentials_survive_disconnect_handoff_and_authenticated_takeover`.
The backtrace points to the displaced staging client's `resumed.until("error")`
(original `protocol.rs:1758`, through `Peer::next` at line 206):

```text
Io(Os { code: 54, kind: ConnectionReset, message: "Connection reset by peer" })
test result: FAILED. 33 passed; 1 failed
```

This is a server race between the close notification and connection work. The
outer biased `select!` could poll the close channel Pending, then poll the work
future. A takeover on the blocking pool could publish `seat_taken` and drop the
outbox senders in between. The inner loop then observed a closed admission or
delivery channel and returned; the outer work branch returned `None`, discarding
the terminal reason. This skipped both the error and Close frame and dropped TCP.
Unread heartbeat traffic can turn such a drop into the reset described in
[RFC 6455 section 1.4](https://www.rfc-editor.org/rfc/rfc6455.html#section-1.4).
Biased polling does not prevent a notification arriving between branch polls.

An intermediate handshake-only build exposed this as Close without `seat_taken`
(four failures in 132 completed protocol invocations; eight in-flight invocations
were interrupted). The deterministic same-poll regression reproduced the missing
reason without timing, sleeps, or retries. The server now reads the terminal
reason after either branch completes, so closing another outbox channel cannot
discard it. No listener readiness, fixed-port, quota, or pre-join timeout change
was needed.

The server also finishes the WebSocket closing handshake instead of dropping TCP
immediately. It detaches from the game before waiting on the peer, retains admission
through the handshake, and releases it before TCP EOF. The handshake has a
five-second deadline for an unresponsive peer. Incoming application frames during
closing are drained without processing. Peer-initiated closes flush their queued
Close reply, which was previously discarded too.

All three new regressions were observed failing against the corresponding old
behavior: the same-poll close-reason race, a delayed heartbeat/Close acknowledgment,
and the closing deadline using virtual time. The strengthened admission test also failed with
`Protocol(ResetWithoutClosingHandshake)` before the fix. Both quota tests now
wait for the Close acknowledgment and TCP EOF and attempt the replacement
connection once; their connection retry loops were removed. No retry or race-hiding
sleep was added.

Raw logs, backtraces, the repetition driver, and per-invocation JSON records with
commands, timestamps, exit codes, and SHA-256 checksums are retained locally in
`.reviews/server-test-flake/` (gitignored).

Final verification on the complete fix:

| Check | Result |
| --- | --- |
| `cargo test -p legatum-server --test protocol`, 180 default invocations across eight repetition lanes | 180 passed, 0 failed; all 34 tests in every invocation. |
| Same command with `-- --test-threads=1`, ten invocations | 10 passed, 0 failed. |
| Same command alongside `cargo test --workspace`, ten invocations | 10 passed, 0 failed; timestamp overlap verified. |
| Total consecutive protocol invocations after the final fix | **200 passed, 0 failed; 6,800 tests passed, none ignored or filtered.** |
| `cargo fmt --all` and `cargo fmt --all -- --check` | Passed, exit 0. |
| `cargo clippy --workspace --all-targets -- -D warnings` | Passed, exit 0. |
| `cargo test --workspace` | Passed, exit 0: **482 passed, 0 failed, 4 existing ignored**. |
| `git diff --check` | Passed, exit 0. |

The verification batch used concurrent repetition lanes, with sequential Cargo
invocations within each lane. No failed invocation was retried or excluded from
the batch. `verified/results.jsonl` and `verified/audit.json` record the final
200 invocations and confirm all log checksums and unchanged source hashes. The
workspace result includes the three new unit regressions; no tests were added to
the ignore list. `docs/learnings.md` was not changed.

The retained driver command was:

```sh
export PATH="$HOME/.cargo/bin:/opt/homebrew/bin:$PATH"
python3 -u .reviews/server-test-flake/stress.py verified \
  --normal 180 --serial 10 --loaded 10 --jobs 8
```
