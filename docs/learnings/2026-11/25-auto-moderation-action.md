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
