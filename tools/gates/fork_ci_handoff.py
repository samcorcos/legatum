#!/usr/bin/env python3
"""Update handoff labels on fork PRs when hosted CI starts or completes.

Runs only trusted master code on workflow_run events for the "CI" workflow.
Never checks out or executes fork code; treats all PR metadata as untrusted data.

State transitions for open, non-draft fork PRs matching workflow_run.head_sha:
- in_progress: if peer-review is not already success on head_sha and needs-human
  is present, remove needs-human and unassign samcorcos (do not add needs-review yet).
- completed (conclusion == "success"): if peer-review is not already success on
  head_sha, clear any lingering needs-human / samcorcos and add needs-review
  unless reviewing:*, changes-requested, or needs-review is already present.
- completed (conclusion != "success"): no-op (failed CI belongs to the fork author).
"""
import json
import os
import re
import sys
from pathlib import Path
from urllib.error import HTTPError
from urllib.parse import quote
from urllib.request import HTTPRedirectHandler, Request, build_opener

SHA = re.compile(r"[0-9a-f]{40}\Z")
REPO = re.compile(r"[A-Za-z0-9][A-Za-z0-9_.-]*/[A-Za-z0-9][A-Za-z0-9_.-]*\Z")
MAINTAINER = "samcorcos"
PEER_REVIEW_CHECK = "peer-review"


class NoRedirect(HTTPRedirectHandler):
    def redirect_request(self, req, fp, code, msg, headers, newurl):
        raise RuntimeError("GitHub API unexpectedly redirected")


class GitHub:
    def __init__(self, repository, token):
        if not REPO.fullmatch(repository):
            raise ValueError("Invalid GITHUB_REPOSITORY")
        self.repository = repository
        self.root = f"https://api.github.com/repos/{repository}"
        self.token = token

    def request(self, method, path, data=None, allow_404=False):
        if not path.startswith("/") or ".." in path:
            raise ValueError("Invalid API path")
        req = Request(
            self.root + path,
            method=method,
            data=None if data is None else json.dumps(data).encode(),
            headers={
                "Authorization": f"Bearer {self.token}",
                "Accept": "application/vnd.github+json",
                "Content-Type": "application/json",
                "X-GitHub-Api-Version": "2022-11-28",
                "Cache-Control": "no-cache",
            },
        )
        try:
            with build_opener(NoRedirect()).open(req, timeout=30) as response:
                raw = response.read()
                return json.loads(raw) if raw else None
        except HTTPError as error:
            if allow_404 and error.code == 404:
                return None
            raise

    def open_pulls(self):
        result = []
        for page in range(1, 101):
            rows = self.request("GET", f"/pulls?state=open&per_page=100&page={page}")
            if not isinstance(rows, list):
                raise ValueError("Invalid pulls response")
            result.extend(rows)
            if len(rows) < 100:
                return result
        raise RuntimeError("Open pulls pagination exceeded its bound")

    def check_runs(self, sha, check_name):
        if not SHA.fullmatch(sha):
            raise ValueError("Invalid commit SHA")
        encoded = quote(check_name, safe="")
        data = self.request("GET", f"/commits/{sha}/check-runs?check_name={encoded}&per_page=100")
        if not isinstance(data, dict) or not isinstance(data.get("check_runs"), list):
            raise ValueError("Invalid check-runs response")
        return data["check_runs"]

    def remove_label(self, number, label):
        encoded = quote(label, safe="")
        return self.request("DELETE", f"/issues/{number}/labels/{encoded}", allow_404=True)

    def add_labels(self, number, labels):
        return self.request("POST", f"/issues/{number}/labels", {"labels": list(labels)})

    def remove_assignees(self, number, assignees):
        return self.request("DELETE", f"/issues/{number}/assignees", {"assignees": list(assignees)})


def peer_review_passed(api, head_sha):
    runs = [run for run in api.check_runs(head_sha, PEER_REVIEW_CHECK)
            if isinstance(run, dict) and run.get("name") == PEER_REVIEW_CHECK]
    if not runs:
        return False
    latest = max(runs, key=lambda run: int(run.get("id", 0)))
    return latest.get("status") == "completed" and latest.get("conclusion") == "success"


def is_matching_fork_pr(pr, repository, head_sha):
    if not isinstance(pr, dict):
        return False
    if pr.get("state") != "open" or pr.get("draft"):
        return False
    head = pr.get("head") or {}
    base = pr.get("base") or {}
    if head.get("sha") != head_sha:
        return False
    if (base.get("repo") or {}).get("full_name") != repository:
        return False
    if (head.get("repo") or {}).get("full_name") == repository:
        return False
    return True


def matching_fork_prs(api, head_sha):
    return [pr for pr in api.open_pulls() if is_matching_fork_pr(pr, api.repository, head_sha)]


def handle_event(api, event):
    if not isinstance(event, dict):
        raise ValueError("Invalid event payload")
    action = event.get("action")
    if action not in ("in_progress", "completed"):
        return []
    workflow_run = event.get("workflow_run")
    if not isinstance(workflow_run, dict):
        return []
    if workflow_run.get("event") != "pull_request":
        return []
    head_repo = (workflow_run.get("head_repository") or {}).get("full_name")
    if head_repo == api.repository:
        return []
    head_sha = workflow_run.get("head_sha")
    if not isinstance(head_sha, str) or not SHA.fullmatch(head_sha):
        raise ValueError("Invalid workflow_run head_sha")
    if action == "completed" and workflow_run.get("conclusion") != "success":
        return []

    prs = matching_fork_prs(api, head_sha)
    if not prs:
        return []

    peer_ok = peer_review_passed(api, head_sha)
    results = []
    for pr in prs:
        number = pr.get("number")
        if isinstance(number, bool) or not isinstance(number, int) or number < 1:
            raise ValueError(f"Invalid pull request number: {number!r}")
        labels = {
            label["name"]
            for label in pr.get("labels", [])
            if isinstance(label, dict) and isinstance(label.get("name"), str)
        }
        assignees = {
            user["login"]
            for user in pr.get("assignees", [])
            if isinstance(user, dict) and isinstance(user.get("login"), str)
        }
        taken = []
        if action == "in_progress":
            if not peer_ok and "needs-human" in labels:
                api.remove_label(number, "needs-human")
                taken.append("remove:needs-human")
                if MAINTAINER in assignees:
                    api.remove_assignees(number, [MAINTAINER])
                    taken.append(f"unassign:{MAINTAINER}")
        elif action == "completed":
            if not peer_ok and "needs-human" in labels:
                api.remove_label(number, "needs-human")
                labels.discard("needs-human")
                taken.append("remove:needs-human")
                if MAINTAINER in assignees:
                    api.remove_assignees(number, [MAINTAINER])
                    taken.append(f"unassign:{MAINTAINER}")
            has_reviewing = any(label.startswith("reviewing:") for label in labels)
            if (
                not peer_ok
                and not has_reviewing
                and "changes-requested" not in labels
                and "needs-human" not in labels
                and "needs-review" not in labels
            ):
                api.add_labels(number, ["needs-review"])
                taken.append("add:needs-review")
        if taken:
            results.append((number, taken))
    return results


def main():
    event_name = os.environ["GITHUB_EVENT_NAME"]
    if event_name != "workflow_run":
        raise ValueError(f"Unsupported event: {event_name}")
    event = json.loads(Path(os.environ["GITHUB_EVENT_PATH"]).read_text())
    api = GitHub(os.environ["GITHUB_REPOSITORY"], os.environ["GITHUB_TOKEN"])
    results = handle_event(api, event)
    if not results:
        print("No fork PR updates needed.")
    else:
        for number, actions in results:
            print(f"PR #{number}: {', '.join(actions)}")
    return 0


if __name__ == "__main__":
    try:
        sys.exit(main())
    except Exception as error:
        print(f"fork-ci-handoff failed: {type(error).__name__}: {error}", file=sys.stderr)
        sys.exit(1)
