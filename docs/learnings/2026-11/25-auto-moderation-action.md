# Moderating a public repo: scheduled agents are too slow and too noisy, an event Action is not (2026-11-25)

During a wave of political issues and README-vandalism PRs, Sam asked for moderation every minute. A scheduled
Claude task can run at most hourly here, and each run opens a new project thread, so a one-minute schedule would
have meant 1,440 threads a day. What works instead: `.github/workflows/moderate.yml` runs on `issues: opened` and
`pull_request_target: opened` (seconds, free on a public repo) and asks GitHub Models (`models: read`, the
workflow's own token, no secret) for a verdict; `tools/moderation/moderate.py` closes only a confident
`off_topic`. The hourly Claude sweep stays as the backstop and the log.

Gotchas: under `pull_request_target`, never check out or run the PR (check out the base branch, read the PR as
text through the API); give the token `issues`/`pull-requests: write` but not `contents: write`, so a manipulated
verdict can at worst close something (reversible), never merge. Strip `<untrusted>` tags a stranger writes so they
cannot end the quoted block early. `author_association` CONTRIBUTOR and above are never judged.

Live result (same day): from Actions, `POST https://models.github.ai/inference/chat/completions` with the workflow
token answered HTTP 200 `text/plain` "OK" instead of a completion, and the old `models.inference.ai.azure.com`
address no longer resolves. So the check falls back to a short keyword list for the wave (Treasury Department,
FinCEN, vibecoding, taxpayer...; not plain "treasury", which game text uses). Dry runs: #165 and the README PR #160
close, the real bug #196 and feature PR #191 stay open. Test any change with a manual run on the branch
(`workflow_dispatch` checks out the ref it was started on) before merging.

Comments (same day): vandals moved to comments on real issues (#198). GitHub has no comment-hiding call in the
GitHub MCP tools, but the workflow token can `minimizeComment` (GraphQL, classifier OFF_TOPIC) with
`issues: write`, so the workflow also runs on `issue_comment: created`, and a manual run with `hide_comments` lets
the hourly sweep hide comments by dispatching it.

Insults (same day): a run of bare-profanity titles ("eat shit", "fuck off") sailed past the wave keyword list. The
rule now also closes on insults aimed at people anywhere, and on profanity when the author wrote almost nothing
else (fewer than 40 letters outside the issue template), so a frustrated player's real report with a swear stays
open. Profanity matches whole words with known endings ("shitty" yes, "shitake" no).
