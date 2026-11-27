#!/usr/bin/env python3
"""Publish the current-head peer-review check, using trusted master code only.

Canonical issue comment (no indentation, quote or Markdown fence):
Legatum review: gemini at <40 lowercase hexadecimal digits>
peer: PASS
legatum-docs-reviewer: PASS

Substantive findings explaining the review.

One or more unique verdict lines are required. Specialist-only comments are valid.
Current comments are the source of truth; edits/deletions revoke their old content.
Model identity is a shared-account convention, not cryptographic attestation.
"""
import json
import os
import re
import subprocess
import sys
from datetime import datetime
from pathlib import Path
from urllib.request import Request, build_opener, HTTPRedirectHandler

import check
import common

FAMILIES = frozenset({"claude", "chatgpt", "gemini"})
TRUSTED_USERS = frozenset({"samcorcos"})
ROLES = frozenset(check.REVIEWERS.values()) | {"peer"}
SHA = re.compile(r"[0-9a-f]{40}\Z")
HEADER = re.compile(r"Legatum review: (claude|chatgpt|gemini) at ([0-9a-f]{40})\Z")
VERDICT = re.compile(r"([a-z-]+): (PASS|FAIL)\Z")


class NoRedirect(HTTPRedirectHandler):
    def redirect_request(self, req, fp, code, msg, headers, newurl):
        raise RuntimeError("GitHub API unexpectedly redirected")


class GitHub:
    def __init__(self, repository, token):
        if not re.fullmatch(r"[A-Za-z0-9][A-Za-z0-9_.-]*/[A-Za-z0-9][A-Za-z0-9_.-]*", repository):
            raise ValueError("Invalid GITHUB_REPOSITORY")
        self.repository = repository
        self.root = f"https://api.github.com/repos/{repository}"
        self.token = token

    def request(self, method, path, data=None):
        if not path.startswith("/") or ".." in path:
            raise ValueError("Invalid API path")
        request = Request(self.root + path, method=method,
                          data=None if data is None else json.dumps(data).encode(), headers={
                              "Authorization": f"Bearer {self.token}",
                              "Accept": "application/vnd.github+json", "Content-Type": "application/json",
                              "X-GitHub-Api-Version": "2022-11-28", "Cache-Control": "no-cache",
                          })
        with build_opener(NoRedirect()).open(request, timeout=30) as response:
            return json.load(response)

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


def pr_number(event_name, event, dispatch=None):
    if event_name == "pull_request_target":
        value = event["pull_request"]["number"]
    elif event_name == "issue_comment":
        if "pull_request" not in event.get("issue", {}):
            return None
        value = event["issue"]["number"]
    elif event_name == "workflow_dispatch":
        value = dispatch or event.get("inputs", {}).get("pr")
    else:
        raise ValueError("Unsupported event")
    if isinstance(value, bool) or not re.fullmatch(r"[1-9][0-9]{0,9}", str(value)):
        raise ValueError("Invalid pull request number")
    return int(value)


def parse_comment(comment):
    if comment.get("user", {}).get("login") not in TRUSTED_USERS:
        return None
    if not isinstance(comment.get("body"), str):
        return None
    lines = comment["body"].replace("\r\n", "\n").split("\n")
    header = HEADER.fullmatch(lines[0]) if lines else None
    if not header or "" not in lines[1:]:
        return None
    separator = lines.index("", 1)
    verdicts = {}
    for line in lines[1:separator]:
        match = VERDICT.fullmatch(line)
        if not match or match[1] not in ROLES or match[1] in verdicts:
            return None
        verdicts[match[1]] = match[2]
    findings = "\n".join(lines[separator + 1:]).strip()
    if not verdicts or not findings:
        return None
    return header[1], header[2], verdicts


def identity(pr, repository):
    """Validate supported provenance and return all recorded author families."""
    if pr["head"]["repo"]["full_name"] != repository or pr["base"]["repo"]["full_name"] != repository:
        raise ValueError("Fork pull requests require the manual review route")
    if pr["user"]["login"] not in TRUSTED_USERS:
        raise ValueError("Outside authors require the manual review route")
    if pr["base"]["ref"] != "master":
        raise ValueError("Only master-targeted pull requests are supported")
    if not SHA.fullmatch(pr["head"]["sha"]) or not SHA.fullmatch(pr["base"]["sha"]):
        raise ValueError("Invalid commit SHA")
    labels = {x["name"] for x in pr["labels"]}
    authors = {label.removeprefix("author:") for label in labels if label.startswith("author:")}
    prefix, separator, suffix = pr["head"]["ref"].partition("/")
    if not authors or not authors <= FAMILIES or prefix not in authors or not separator or not suffix:
        raise ValueError("Recognized author labels must include the branch's model family")
    if pr["state"] != "open" or pr["draft"]:
        raise ValueError("Pull request must be open and ready for review")
    if "changes-requested" in labels:
        raise ValueError("changes-requested must be resolved before approval")
    return authors


def evaluate(comments, sha, authors, required):
    latest = {}
    for comment in comments:
        parsed = parse_comment(comment)
        if not parsed or parsed[1] != sha:
            continue
        family, _, verdicts = parsed
        # Editing an older comment is a new verdict, rather than retaining its old approval.
        stamp = datetime.fromisoformat(comment["updated_at"].replace("Z", "+00:00"))
        order = (stamp, int(comment["id"]))
        for role, verdict in verdicts.items():
            key = (family, role)
            if key not in latest or order > latest[key][0]:
                latest[key] = (order, verdict)
    failures = [f"{family}/{role}" for (family, role), (_, verdict) in latest.items() if verdict == "FAIL"]
    missing = [role for role in sorted(required) if not any(
        r == role and verdict == "PASS" for (_, r), (_, verdict) in latest.items())]
    peer = any(role == "peer" and family not in authors and verdict == "PASS"
               for (family, role), (_, verdict) in latest.items())
    reasons = []
    if failures:
        reasons.append("Outstanding failures: " + ", ".join(sorted(failures)))
    if missing:
        reasons.append("Missing specialist verdicts: " + ", ".join(missing))
    if not peer:
        reasons.append("Missing different-family peer approval")
    return reasons


def strict_git(repo, *args, binary=False):
    args = list(args)
    if args and args[0] == "diff":
        args[1:1] = ["--no-ext-diff", "--no-textconv"]
    result = subprocess.run(["git", "-C", repo, "-c", "core.hooksPath=/dev/null",
                             "-c", "core.quotePath=false", *args], capture_output=True, timeout=90)
    if result.returncode:
        raise RuntimeError(f"git {args[0]} failed")
    return result.stdout if binary else result.stdout.decode("utf-8", "strict")


def required_roles(pr, number, repo="."):
    base, head = pr["base"]["sha"], pr["head"]["sha"]
    if strict_git(repo, "rev-parse", "HEAD").strip() != base:
        raise ValueError("Evaluator checkout must be the current trusted master commit")
    ref = f"refs/peer-review/{number}"
    strict_git(repo, "fetch", "--no-tags", "--no-recurse-submodules", "origin", base,
               f"+refs/pull/{number}/head:{ref}")
    if strict_git(repo, "rev-parse", ref).strip() != head:
        raise ValueError("Pull request head changed during fetch")
    merge_base = strict_git(repo, "merge-base", base, head).strip()
    if not SHA.fullmatch(merge_base):
        raise ValueError("Cannot determine merge base")
    files = strict_git(repo, "diff", "--name-only", "--no-renames", "-z", merge_base, head).split("\0")[:-1]
    if any(not f.isprintable() or '"' in f or "\\" in f for f in files):
        raise ValueError("Unsupported filename in review diff")
    # common.git normally turns errors into empty diffs. In this security gate, errors must raise.
    previous = common.git
    common.git = strict_git
    try:
        kinds = check.classify(repo, merge_base, head, files)
        return {role for _, role, _ in check.required_reviews(kinds)}
    finally:
        common.git = previous


def fingerprint(pr, comments):
    metadata = {key: pr[key] for key in ("state", "draft")}
    metadata["author"] = pr["user"]["login"]
    metadata["labels"] = sorted(label["name"] for label in pr["labels"])
    for side in ("head", "base"):
        metadata[side] = [pr[side]["sha"], pr[side]["ref"], pr[side]["repo"]["full_name"]]
    current = sorted((c["id"], c.get("body"), c.get("updated_at"), c.get("user", {}).get("login")) for c in comments)
    return json.dumps([metadata, current], sort_keys=True)


def run(api, number, classify=required_roles):
    pr = api.request("GET", f"/pulls/{number}")
    sha = pr["head"]["sha"]
    if not SHA.fullmatch(sha):
        raise ValueError("Invalid PR head")
    check_run = api.request("POST", "/check-runs", {"name": "peer-review", "head_sha": sha,
                           "status": "in_progress", "output": {"title": "Reviewing current approvals", "summary": "Checking exact-head peer and specialist verdicts."}})
    reasons = []
    try:
        authors = identity(pr, api.repository)
        required = classify(pr, number)
        comments = api.comments(number)
        reasons = evaluate(comments, sha, authors, required)
        if not reasons:
            refreshed = api.request("GET", f"/pulls/{number}")
            refreshed_comments = api.comments(number)
            if fingerprint(pr, comments) != fingerprint(refreshed, refreshed_comments):
                reasons = ["PR head, metadata or comments changed during evaluation; rerun required"]
    except Exception as error:
        reasons = [f"Review evaluation failed: {type(error).__name__}: {error}"]
    success = not reasons
    summary = "\n".join(reasons) if reasons else "Required specialist verdicts and a different-family peer PASS match the exact current head."
    api.request("PATCH", f"/check-runs/{check_run['id']}", {"status": "completed",
                "conclusion": "success" if success else "failure", "output": {
                    "title": "Review requirements satisfied" if success else "Review requirements not satisfied",
                    "summary": summary[:60000]}})
    print(summary)
    return success


def main():
    event = json.loads(Path(os.environ["GITHUB_EVENT_PATH"]).read_text())
    number = pr_number(os.environ["GITHUB_EVENT_NAME"], event, os.environ.get("INPUT_PR"))
    if number is None:
        return 0
    api = GitHub(os.environ["GITHUB_REPOSITORY"], os.environ["GITHUB_TOKEN"])
    return 0 if run(api, number) else 1


if __name__ == "__main__":
    try:
        sys.exit(main())
    except Exception as error:
        print(f"peer-review failed: {type(error).__name__}: {error}", file=sys.stderr)
        sys.exit(1)
