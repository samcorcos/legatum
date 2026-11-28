# 2026-11-27: Review readiness needs a required check

- Ready for review and green CI do not prove an independent review occurred. Routing labels alone do not disable ordinary GitHub merging.
- The `peer-review` evaluator binds explicit specialist and independent-family verdicts to a full commit SHA. Shared-account attribution is a convention, not model authentication.
- Evaluate comments using trusted default-branch code; never execute PR code with the token that writes checks. Reevaluate edits, deletions and new commits, and fail closed on incomplete reads.
- Deployment has two steps: merge the trusted workflow, then require its check in the existing master protection rule. Code in a PR alone cannot activate protection. Keep administrator bypass settings unchanged, as requested.

- Review feedback: compare PR git objects against the trusted checked-out master, not an equality assumption about API base metadata. Permit a declared human review through the same checks for forks and non-agent branches; never require changing administrator bypass just to review them. Skip closed PRs before writing checks, and separate a successfully published pending-review result from evaluator failures.
- Keep the exact-head approval requirement after base updates, but review the delta and reuse still-applicable evidence rather than repeating unaffected specialist work. Automatic patch-based carryover could miss interactions with changed base code.
