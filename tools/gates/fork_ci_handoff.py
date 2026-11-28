#!/usr/bin/env python3
"""Update handoff labels on fork PRs when hosted CI starts or completes.

Runs only trusted master code on workflow_run events for the "CI" workflow.
Never checks out or executes fork code; treats all PR metadata as untrusted data.

Fail-closed state transitions for open, non-draft fork PRs matching both
workflow_run.head_sha and workflow_run.head_repository.full_name, after
confirming the event is the live latest CI run/attempt for that head and
re-reading live PR state immediately before any write:
- in_progress: if no reviewing:*/changes-requested label is held, peer-review
  is not already success on head_sha, needs-human is present, and the latest
  trusted PR comment carries the canonical marker
  `Legatum ci-approval: <head_sha>` (optionally `run=<run_id>`), remove
  needs-human and unassign samcorcos (do not add needs-review yet).
- completed (conclusion == "success"): if no reviewing:*/changes-requested
  label is held and peer-review is not already success on head_sha, clear
  needs-human / samcorcos only when backed by the same active trusted
  ci-approval marker, then add needs-review if neither needs-human nor
  needs-review is present.
- completed (conclusion != "success"): no-op (failed CI belongs to the fork author).
"""
import json
import os
import re
import sys
from datetime import datetime, timezone
from pathlib import Path
from urllib.error import HTTPError
from urllib.parse import quote
from urllib.request import HTTPRedirectHandler, Request, build_opener

SHA = re.compile(r"[0-9a-f]{40}\Z")
REPO = re.compile(r"[A-Za-z0-9][A-Za-z0-9_.-]*/[A-Za-z0-9][A-Za-z0-9_.-]*\Z")
POS_INT = re.compile(r"[1-9][0-9]{0,18}\Z")
CI_APPROVAL = re.compile(r"Legatum ci-approval: ([0-9a-f]{40})(?: run=([1-9][0-9]{0,18}))?\Z")
TRUSTED_ASSOCIATIONS = frozenset({"OWNER", "MEMBER", "COLLABORATOR"})
MAINTAINER = "samcorcos"
PEER_REVIEW_CHECK = "peer-review"
CI_WORKFLOW = "ci.yml"


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

    def pull(self, number):
        data = self.request("GET", f"/pulls/{number}")
        if not isinstance(data, dict):
            raise ValueError("Invalid pull response")
        return data

    def comments(self, number):
        result = []
        for page in range(1, 1001):
            rows = self.request("GET", f"/issues/{number}/comments?per_page=100&page={page}")
            if not isinstance(rows, list):
                raise ValueError("Invalid comments response")
            result.extend(rows)
            if len(rows) < 100:
                return result
        raise RuntimeError("Comment pagination exceeded its bound")

    def ci_runs(self, head_sha):
        if not SHA.fullmatch(head_sha):
            raise ValueError("Invalid commit SHA")
        data = self.request(
            "GET",
            f"/actions/workflows/{CI_WORKFLOW}/runs?head_sha={head_sha}&event=pull_request&per_page=20",
        )
        if not isinstance(data, dict) or not isinstance(data.get("workflow_runs"), list):
            raise ValueError("Invalid workflow runs response")
        return data["workflow_runs"]

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


def parse_positive_int(value, name):
    if isinstance(value, bool) or value is None:
        raise ValueError(f"Invalid {name}")
    text = str(value)
    if not POS_INT.fullmatch(text):
        raise ValueError(f"Invalid {name}")
    return int(text)


def extract_labels(pr):
    return {
        label["name"]
        for label in pr.get("labels", [])
        if isinstance(label, dict) and isinstance(label.get("name"), str)
    }


def extract_assignees(pr):
    return {
        user["login"]
        for user in pr.get("assignees", [])
        if isinstance(user, dict) and isinstance(user.get("login"), str)
    }


def has_blocking_review_state(labels):
    return "changes-requested" in labels or any(label.startswith("reviewing:") for label in labels)


def parse_ci_approval_marker(body):
    if not isinstance(body, str):
        return None
    in_fence = None
    matches = []
    for line in body.replace("\r\n", "\n").split("\n"):
        stripped = line.lstrip()
        if stripped.startswith("```") or stripped.startswith("~~~"):
            fence = stripped[:3]
            if in_fence is None:
                in_fence = fence
            elif in_fence == fence:
                in_fence = None
            continue
        if in_fence is not None:
            continue
        match = CI_APPROVAL.fullmatch(line)
        if match:
            run_id = int(match[2]) if match[2] is not None else None
            matches.append((match[1], run_id))
    if len(matches) != 1:
        return None
    return matches[0]


def is_trusted_comment(comment, repository):
    if not isinstance(comment, dict):
        return False
    owner = repository.split("/", 1)[0]
    login = (comment.get("user") or {}).get("login")
    assoc = comment.get("author_association")
    return login == owner or assoc in TRUSTED_ASSOCIATIONS


def comment_order(comment):
    raw_ts = comment.get("updated_at") or comment.get("created_at")
    if isinstance(raw_ts, str) and raw_ts:
        stamp = datetime.fromisoformat(raw_ts.replace("Z", "+00:00"))
    else:
        stamp = datetime.min.replace(tzinfo=timezone.utc)
    return stamp, parse_positive_int(comment.get("id"), "comment.id")


def has_active_ci_approval_handoff(comments, repository, head_sha, run_id):
    trusted = [c for c in comments if is_trusted_comment(c, repository)]
    if not trusted:
        return False
    latest = max(trusted, key=comment_order)
    marker = parse_ci_approval_marker(latest.get("body"))
    if marker is None:
        return False
    marker_sha, marker_run_id = marker
    if marker_sha != head_sha:
        return False
    if marker_run_id is not None and marker_run_id != run_id:
        return False
    return True


def peer_review_passed(api, head_sha):
    runs = [
        run
        for run in api.check_runs(head_sha, PEER_REVIEW_CHECK)
        if isinstance(run, dict) and run.get("name") == PEER_REVIEW_CHECK
    ]
    if not runs:
        return False
    latest = max(runs, key=lambda run: int(run.get("id", 0)))
    return latest.get("status") == "completed" and latest.get("conclusion") == "success"


def is_latest_ci_run(api, head_sha, head_repo, run_id, run_attempt, action):
    applicable = [
        run
        for run in api.ci_runs(head_sha)
        if isinstance(run, dict)
        and run.get("event") == "pull_request"
        and run.get("head_sha") == head_sha
        and (run.get("head_repository") or {}).get("full_name") == head_repo
    ]
    if not applicable:
        return False
    latest = max(
        applicable,
        key=lambda run: (
            parse_positive_int(run.get("id"), "workflow_run.id"),
            parse_positive_int(run.get("run_attempt", 1), "workflow_run.run_attempt"),
        ),
    )
    latest_id = parse_positive_int(latest.get("id"), "workflow_run.id")
    latest_attempt = parse_positive_int(latest.get("run_attempt", 1), "workflow_run.run_attempt")
    if latest_id != run_id or latest_attempt != run_attempt:
        return False
    if action == "in_progress":
        return latest.get("status") == "in_progress"
    if action == "completed":
        return latest.get("status") == "completed" and latest.get("conclusion") == "success"
    return False


def is_matching_fork_pr(pr, repository, head_sha, head_repo):
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
    pr_head_repo = (head.get("repo") or {}).get("full_name")
    if not isinstance(pr_head_repo, str) or not pr_head_repo or pr_head_repo == repository:
        return False
    return pr_head_repo == head_repo


def matching_fork_prs(api, head_sha, head_repo):
    return [
        pr
        for pr in api.open_pulls()
        if is_matching_fork_pr(pr, api.repository, head_sha, head_repo)
    ]


def resolve_run_identity(workflow_run, env_run_id=None, env_run_attempt=None):
    payload_id = workflow_run.get("id")
    payload_attempt = workflow_run.get("run_attempt")
    run_id = parse_positive_int(
        payload_id if payload_id is not None else env_run_id, "workflow_run.id"
    )
    if payload_id is not None and env_run_id is not None:
        if run_id != parse_positive_int(env_run_id, "WORKFLOW_RUN_ID"):
            raise ValueError("WORKFLOW_RUN_ID disagrees with event payload")
    run_attempt = parse_positive_int(
        payload_attempt if payload_attempt is not None else (env_run_attempt or 1),
        "workflow_run.run_attempt",
    )
    if payload_attempt is not None and env_run_attempt is not None:
        if run_attempt != parse_positive_int(env_run_attempt, "WORKFLOW_RUN_ATTEMPT"):
            raise ValueError("WORKFLOW_RUN_ATTEMPT disagrees with event payload")
    return run_id, run_attempt


def handle_event(api, event, env_run_id=None, env_run_attempt=None):
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
    if not isinstance(head_repo, str) or not head_repo or head_repo == api.repository:
        return []
    head_sha = workflow_run.get("head_sha")
    if not isinstance(head_sha, str) or not SHA.fullmatch(head_sha):
        raise ValueError("Invalid workflow_run head_sha")
    if action == "completed" and workflow_run.get("conclusion") != "success":
        return []
    run_id, run_attempt = resolve_run_identity(workflow_run, env_run_id, env_run_attempt)

    prs = matching_fork_prs(api, head_sha, head_repo)
    if not prs:
        return []
    if not is_latest_ci_run(api, head_sha, head_repo, run_id, run_attempt, action):
        return []

    peer_ok = peer_review_passed(api, head_sha)
    if peer_ok:
        return []

    results = []
    for initial_pr in prs:
        number = parse_positive_int(initial_pr.get("number"), "pull_request.number")
        initial_labels = extract_labels(initial_pr)
        initial_assignees = extract_assignees(initial_pr)
        if has_blocking_review_state(initial_labels):
            continue

        clear_needs_human = False
        add_needs_review = False
        if "needs-human" in initial_labels:
            initial_comments = api.comments(number)
            clear_needs_human = has_active_ci_approval_handoff(
                initial_comments, api.repository, head_sha, run_id
            )
            if action == "in_progress" and not clear_needs_human:
                continue
        if action == "completed":
            effective_labels = set(initial_labels)
            if clear_needs_human:
                effective_labels.discard("needs-human")
            if "needs-human" not in effective_labels and "needs-review" not in effective_labels:
                add_needs_review = True

        if not clear_needs_human and not add_needs_review:
            continue

        # Revalidate live PR state, comments, and CI run freshness immediately before any write.
        live_pr = api.pull(number)
        if not is_matching_fork_pr(live_pr, api.repository, head_sha, head_repo):
            continue
        live_labels = extract_labels(live_pr)
        live_assignees = extract_assignees(live_pr)
        if live_labels != initial_labels or live_assignees != initial_assignees:
            continue
        if has_blocking_review_state(live_labels):
            continue
        if clear_needs_human:
            live_comments = api.comments(number)
            if not has_active_ci_approval_handoff(live_comments, api.repository, head_sha, run_id):
                continue
        if not is_latest_ci_run(api, head_sha, head_repo, run_id, run_attempt, action):
            continue

        taken = []
        if clear_needs_human:
            api.remove_label(number, "needs-human")
            taken.append("remove:needs-human")
            if MAINTAINER in live_assignees:
                api.remove_assignees(number, [MAINTAINER])
                taken.append(f"unassign:{MAINTAINER}")
        if add_needs_review:
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
    results = handle_event(
        api,
        event,
        env_run_id=os.environ.get("WORKFLOW_RUN_ID"),
        env_run_attempt=os.environ.get("WORKFLOW_RUN_ATTEMPT"),
    )
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
