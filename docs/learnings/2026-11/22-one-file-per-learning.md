## 2026-11-22: One file per learnings entry

- **An append-only file that every branch writes to conflicts in every open pull request.** With a dozen pull requests open and master landing one every few minutes, each landing made the others unmergeable on `docs/learnings/2026-11.md` alone. The resolution was always "keep both", but each re-merge restarted a ten-minute hosted CI run, so pull requests chased master without landing.
- **Now:** a new entry is its own file, `docs/learnings/<YYYY-MM>/<DD>-<slug>.md`. The month files (`2026-09.md`, `2026-10.md`, `2026-11.md`) are closed and keep the earlier entries. A branch that added an entry to a month file takes master's copy of that file and moves its entry into a new file.
- **GitHub does not apply `merge=union` from `.gitattributes` when it merges a pull request**, so a union merge driver would not have fixed this on the web.
- The Stop hook and `docs-check.sh` already look at every `.md` under `docs/learnings/`, so they needed no change beyond the reminder text.
