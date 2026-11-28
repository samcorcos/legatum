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
HEADER = re.compile(r"Legatum review: (claude|chatgpt|gemini|human) at ([0-9a-f]{40})\Z")
VERDICT = re.compile(r"([a-z-]+): (PASS|FAIL)\Z")
MERGE_GROUP_REF = re.compile(r"refs/heads/gh-readonly-queue/master/[A-Za-z0-9][A-Za-z0-9._/-]*\Z")
MERGE_GROUP_PR = re.compile(r"/pr-([1-9][0-9]{0,9})(?:-|\Z)")


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
    """Return agent authors, or None when explicit maintainer approval is required."""
    if pr["base"]["repo"]["full_name"] != repository or pr["base"]["ref"] != "master":
        raise ValueError("Only pull requests targeting this repository's master are supported")
    if not SHA.fullmatch(pr["head"]["sha"]) or not SHA.fullmatch(pr["base"]["sha"]):
        raise ValueError("Invalid commit SHA")
    if ((pr["head"].get("repo") or {}).get("full_name") != repository
            or pr["user"]["login"] not in TRUSTED_USERS):
        return None
    labels = {x["name"] for x in pr["labels"]}
    authors = {label.removeprefix("author:") for label in labels if label.startswith("author:")}
    prefix, separator, suffix = pr["head"]["ref"].partition("/")
    if not authors or not authors <= FAMILIES or prefix not in authors or not separator or not suffix:
        return None
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
    peer = any(role == "peer" and (family == "human" or (authors is not None and family not in authors))
               and verdict == "PASS"
               for (family, role), (_, verdict) in latest.items())
    reasons = []
    if failures:
        reasons.append("Outstanding failures: " + ", ".join(sorted(failures)))
    if missing:
        reasons.append("Missing specialist verdicts: " + ", ".join(missing))
    if not peer:
        reasons.append("Missing explicit human peer approval" if authors is None
                       else "Missing different-family peer or explicit human approval")
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
    # The workflow checks out trusted master. GitHub's PR base SHA may lag behind it.
    base = strict_git(repo, "rev-parse", "HEAD").strip()
    head = pr["head"]["sha"]
    if not SHA.fullmatch(base) or not SHA.fullmatch(head):
        raise ValueError("Invalid commit SHA")
    ref = f"refs/peer-review/{number}"
    strict_git(repo, "fetch", "--no-tags", "--no-recurse-submodules", "origin",
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
        metadata[side] = [pr[side]["sha"], pr[side]["ref"], (pr[side].get("repo") or {}).get("full_name")]
    current = sorted((c["id"], c.get("body"), c.get("updated_at"), c.get("user", {}).get("login")) for c in comments)
    return json.dumps([metadata, current], sort_keys=True)


def validate_merge_group(merge_group):
    if not isinstance(merge_group, dict):
        raise ValueError("Invalid merge_group payload")
    head_sha = merge_group.get("head_sha")
    base_sha = merge_group.get("base_sha")
    head_ref = merge_group.get("head_ref")
    base_ref = merge_group.get("base_ref")
    if not isinstance(head_sha, str) or not SHA.fullmatch(head_sha):
        raise ValueError("Invalid merge_group head_sha")
    if not isinstance(base_sha, str) or not SHA.fullmatch(base_sha):
        raise ValueError("Invalid merge_group base_sha")
    if not isinstance(head_ref, str) or not MERGE_GROUP_REF.fullmatch(head_ref) or ".." in head_ref:
        raise ValueError("Invalid merge_group head_ref")
    if base_ref is not None and base_ref != "refs/heads/master":
        raise ValueError("Only merge groups targeting master are supported")
    return head_sha, base_sha, head_ref


def merge_group_prs(api, merge_group, repo="."):
    head_sha, base_sha, head_ref = validate_merge_group(merge_group)
    ref = f"refs/peer-review/merge-group/{head_sha}"
    strict_git(repo, "fetch", "--no-tags", "--no-recurse-submodules", "origin", f"+{head_ref}:{ref}")
    if strict_git(repo, "rev-parse", ref).strip() != head_sha:
        raise ValueError("Merge group head changed during fetch")
    strict_git(repo, "merge-base", "--is-ancestor", base_sha, head_sha)
    lines = [line for line in strict_git(repo, "rev-list", "--first-parent", "--parents",
                                         f"{base_sha}..{head_sha}").splitlines() if line.strip()]
    if not lines:
        raise ValueError("Merge group contains no commits")
    resolved = {}
    for line in lines:
        parts = line.split()
        commit, parents = parts[0], parts[1:]
        if len(parents) == 2:
            target_sha = parents[1]
            require_head_match = True
        elif len(parents) == 1:
            target_sha = commit
            require_head_match = False
        else:
            raise ValueError(f"Unsupported commit parent count in merge group: {commit}")
        if not SHA.fullmatch(target_sha):
            raise ValueError("Invalid commit SHA in merge group")
        pulls = api.request("GET", f"/commits/{target_sha}/pulls?per_page=100")
        if not isinstance(pulls, list):
            raise ValueError("Invalid commit pulls response")
        matches = [
            row for row in pulls
            if row.get("state") == "open"
            and (row.get("base") or {}).get("ref") == "master"
            and ((row.get("base") or {}).get("repo") or {}).get("full_name") == api.repository
            and (not require_head_match or (row.get("head") or {}).get("sha") == target_sha)
        ]
        if len(matches) != 1:
            raise ValueError(f"Cannot uniquely map merge-group commit {target_sha} to an open PR")
        number = matches[0].get("number")
        if isinstance(number, bool) or not isinstance(number, int) or number < 1:
            raise ValueError("Invalid pull request number in commit pulls response")
        if number in resolved:
            if require_head_match and resolved[number] != target_sha:
                raise ValueError(f"Conflicting commits for PR #{number} in merge group")
        else:
            resolved[number] = target_sha
    match = MERGE_GROUP_PR.search(head_ref)
    if match and int(match[1]) not in resolved:
        raise ValueError(f"Merge group ref names PR #{match[1]} which was not resolved from commits")
    return list(resolved.items())


def evaluate_pr(api, number, classify=required_roles, expected_sha=None, pr=None):
    """Evaluate a single PR's approval requirements; returns (reasons, closed)."""
    if pr is None:
        pr = api.request("GET", f"/pulls/{number}")
    if pr["state"] != "open":
        return ["Pull request is not open"], True
    sha = pr["head"]["sha"]
    if not SHA.fullmatch(sha):
        raise ValueError("Invalid PR head")
    if expected_sha is not None and sha != expected_sha:
        return [f"PR head {sha} does not match merge-group commit {expected_sha}"], False
    authors = identity(pr, api.repository)
    labels = {label["name"] for label in pr["labels"]}
    comments = None
    if pr["draft"]:
        reasons = ["Pull request must be ready for review"]
    elif "changes-requested" in labels:
        reasons = ["changes-requested must be resolved before approval"]
    else:
        required = classify(pr, number)
        comments = api.comments(number)
        reasons = evaluate(comments, sha, authors, required)
    # Recheck even when waiting for reviews, so a concurrent close never stamps a failure.
    refreshed = api.request("GET", f"/pulls/{number}")
    closed = refreshed["state"] != "open"
    if not reasons and not closed:
        refreshed_comments = api.comments(number)
        if fingerprint(pr, comments) != fingerprint(refreshed, refreshed_comments):
            reasons = ["PR head, metadata or comments changed during evaluation; rerun required"]
    return reasons, closed


def run(api, number, classify=required_roles):
    pr = api.request("GET", f"/pulls/{number}")
    if pr["state"] != "open":
        print("Closed pull request; no check published.")
        return None
    sha = pr["head"]["sha"]
    if not SHA.fullmatch(sha):
        raise ValueError("Invalid PR head")
    check_run = api.request("POST", "/check-runs", {"name": "peer-review", "head_sha": sha,
                           "status": "in_progress", "output": {"title": "Reviewing current approvals", "summary": "Checking exact-head peer and specialist verdicts."}})
    reasons = []
    operational_error = None
    closed = False
    try:
        reasons, closed = evaluate_pr(api, number, classify=classify, pr=pr)
    except Exception as error:
        operational_error = error
        reasons = [f"Review evaluation failed: {type(error).__name__}: {error}"]
    success = not reasons and not closed
    summary = ("Pull request closed during evaluation; check cancelled." if closed else
               "\n".join(reasons) if reasons else
               "Required specialist verdicts and an eligible peer PASS match the exact current head.")
    api.request("PATCH", f"/check-runs/{check_run['id']}", {"status": "completed",
                "conclusion": "cancelled" if closed else "success" if success else "failure", "output": {
                    "title": "Pull request closed" if closed else "Review requirements satisfied" if success else "Review requirements not satisfied",
                    "summary": summary[:60000]}})
    print(summary)
    if operational_error is not None:
        raise operational_error
    return success


def run_merge_group(api, merge_group, resolve_prs=merge_group_prs, classify=required_roles):
    if not isinstance(merge_group, dict):
        raise ValueError("Invalid merge_group payload")
    head_sha = merge_group.get("head_sha")
    if not isinstance(head_sha, str) or not SHA.fullmatch(head_sha):
        raise ValueError("Invalid merge_group head_sha")
    check_run = api.request("POST", "/check-runs", {"name": "peer-review", "head_sha": head_sha,
                           "status": "in_progress", "output": {"title": "Reviewing merge group approvals",
                           "summary": "Checking exact-head peer and specialist verdicts for all queued pull requests."}})
    reasons = []
    operational_error = None
    checked = []
    try:
        validate_merge_group(merge_group)
        prs = resolve_prs(api, merge_group)
        if not prs:
            raise ValueError("Merge group resolved to zero pull requests")
        for number, expected_sha in prs:
            if isinstance(number, bool) or not isinstance(number, int) or number < 1:
                raise ValueError(f"Invalid constituent PR number: {number!r}")
            if not isinstance(expected_sha, str) or not SHA.fullmatch(expected_sha):
                raise ValueError(f"Invalid expected SHA for PR #{number}")
            pr_reasons, closed = evaluate_pr(api, number, classify=classify, expected_sha=expected_sha)
            checked.append(f"#{number}")
            if closed:
                reasons.append(f"PR #{number}: Pull request is not open")
            elif pr_reasons:
                reasons.append(f"PR #{number}: " + "; ".join(pr_reasons))
    except Exception as error:
        operational_error = error
        reasons = [f"Merge group review evaluation failed: {type(error).__name__}: {error}"]
    success = not reasons
    summary = ("\n".join(reasons) if reasons else
               f"Required specialist verdicts and an eligible peer PASS match the exact reviewed head for all queued PRs ({', '.join(checked)}).")
    api.request("PATCH", f"/check-runs/{check_run['id']}", {"status": "completed",
                "conclusion": "success" if success else "failure", "output": {
                    "title": "Merge group review requirements satisfied" if success else "Merge group review requirements not satisfied",
                    "summary": summary[:60000]}})
    print(summary)
    if operational_error is not None:
        raise operational_error
    return success


def main():
    event = json.loads(Path(os.environ["GITHUB_EVENT_PATH"]).read_text())
    event_name = os.environ["GITHUB_EVENT_NAME"]
    if event_name == "merge_group":
        api = GitHub(os.environ["GITHUB_REPOSITORY"], os.environ["GITHUB_TOKEN"])
        run_merge_group(api, event.get("merge_group"))
        return 0
    number = pr_number(event_name, event, os.environ.get("INPUT_PR"))
    if number is None:
        return 0
    api = GitHub(os.environ["GITHUB_REPOSITORY"], os.environ["GITHUB_TOKEN"])
    run(api, number)
    return 0


if __name__ == "__main__":
    try:
        sys.exit(main())
    except Exception as error:
        print(f"peer-review failed: {type(error).__name__}: {error}", file=sys.stderr)
        sys.exit(1)
