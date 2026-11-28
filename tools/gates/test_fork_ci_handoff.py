"""Unit tests for the trusted-base fork CI handoff gate (no GitHub writes)."""
import copy
import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import fork_ci_handoff as handoff

REPOSITORY = "samcorcos/legatum"
FORK_REPO = "contributor/legatum"
HEAD = "a" * 40
OTHER_SHA = "b" * 40


def make_pr(
    number=42,
    *,
    state="open",
    draft=False,
    head_sha=HEAD,
    head_repo=FORK_REPO,
    base_repo=REPOSITORY,
    labels=(),
    assignees=(),
):
    return {
        "number": number,
        "state": state,
        "draft": draft,
        "head": {"sha": head_sha, "repo": {"full_name": head_repo} if head_repo is not None else None},
        "base": {"repo": {"full_name": base_repo}},
        "labels": [{"name": name} for name in labels],
        "assignees": [{"login": login} for login in assignees],
    }


def make_event(
    action,
    *,
    event="pull_request",
    head_sha=HEAD,
    head_repo=FORK_REPO,
    conclusion=None,
    pull_requests=(),
):
    return {
        "action": action,
        "workflow_run": {
            "event": event,
            "head_sha": head_sha,
            "head_repository": {"full_name": head_repo} if head_repo is not None else None,
            "conclusion": conclusion,
            "pull_requests": list(pull_requests),
        },
    }


class FakeAPI:
    repository = REPOSITORY

    def __init__(self, pulls=None, check_runs=None):
        self._pulls = list(pulls or [])
        self._check_runs = list(check_runs or [])
        self.calls = []

    def open_pulls(self):
        self.calls.append(("open_pulls",))
        return copy.deepcopy(self._pulls)

    def check_runs(self, sha, check_name):
        self.calls.append(("check_runs", sha, check_name))
        return copy.deepcopy(self._check_runs)

    def remove_label(self, number, label):
        self.calls.append(("remove_label", number, label))

    def add_labels(self, number, labels):
        self.calls.append(("add_labels", number, list(labels)))

    def remove_assignees(self, number, assignees):
        self.calls.append(("remove_assignees", number, list(assignees)))


class ForkCIHandoffTests(unittest.TestCase):
    def test_in_progress_clears_needs_human_and_unassigns_maintainer(self):
        pr = make_pr(labels=("needs-human",), assignees=("samcorcos",))
        api = FakeAPI(pulls=[pr], check_runs=[])
        event = make_event("in_progress")

        results = handoff.handle_event(api, event)

        self.assertEqual(results, [(42, ["remove:needs-human", "unassign:samcorcos"])])
        self.assertIn(("remove_label", 42, "needs-human"), api.calls)
        self.assertIn(("remove_assignees", 42, ["samcorcos"]), api.calls)
        self.assertFalse(any(call[0] == "add_labels" for call in api.calls))

    def test_in_progress_clears_needs_human_without_assignee(self):
        pr = make_pr(labels=("needs-human",), assignees=())
        api = FakeAPI(pulls=[pr], check_runs=[])

        results = handoff.handle_event(api, make_event("in_progress"))

        self.assertEqual(results, [(42, ["remove:needs-human"])])
        self.assertFalse(any(call[0] == "remove_assignees" for call in api.calls))

    def test_in_progress_preserves_needs_human_when_peer_review_passed(self):
        pr = make_pr(labels=("needs-human",), assignees=("samcorcos",))
        runs = [{"id": 10, "name": "peer-review", "status": "completed", "conclusion": "success"}]
        api = FakeAPI(pulls=[pr], check_runs=runs)

        results = handoff.handle_event(api, make_event("in_progress"))

        self.assertEqual(results, [])
        self.assertFalse(any(call[0] in ("remove_label", "remove_assignees", "add_labels") for call in api.calls))

    def test_in_progress_clears_when_latest_peer_review_failed_after_earlier_pass(self):
        pr = make_pr(labels=("needs-human",), assignees=("samcorcos",))
        runs = [
            {"id": 10, "name": "peer-review", "status": "completed", "conclusion": "success"},
            {"id": 11, "name": "peer-review", "status": "completed", "conclusion": "failure"},
        ]
        api = FakeAPI(pulls=[pr], check_runs=runs)

        results = handoff.handle_event(api, make_event("in_progress"))

        self.assertEqual(results, [(42, ["remove:needs-human", "unassign:samcorcos"])])

    def test_completed_success_adds_needs_review(self):
        pr = make_pr(labels=(), assignees=())
        api = FakeAPI(pulls=[pr], check_runs=[])

        results = handoff.handle_event(api, make_event("completed", conclusion="success"))

        self.assertEqual(results, [(42, ["add:needs-review"])])
        self.assertIn(("add_labels", 42, ["needs-review"]), api.calls)

    def test_completed_success_clears_lingering_needs_human_and_adds_needs_review(self):
        pr = make_pr(labels=("needs-human",), assignees=("samcorcos",))
        api = FakeAPI(pulls=[pr], check_runs=[])

        results = handoff.handle_event(api, make_event("completed", conclusion="success"))

        self.assertEqual(results, [(42, ["remove:needs-human", "unassign:samcorcos", "add:needs-review"])])

    def test_completed_success_skips_when_blocked_or_already_routed(self):
        for labels in (
            ("reviewing:claude",),
            ("changes-requested",),
            ("needs-review",),
        ):
            with self.subTest(labels=labels):
                pr = make_pr(labels=labels)
                api = FakeAPI(pulls=[pr], check_runs=[])
                results = handoff.handle_event(api, make_event("completed", conclusion="success"))
                self.assertEqual(results, [])
                self.assertFalse(any(call[0] == "add_labels" for call in api.calls))

    def test_completed_success_skips_draft_pr(self):
        pr = make_pr(draft=True)
        api = FakeAPI(pulls=[pr], check_runs=[])

        results = handoff.handle_event(api, make_event("completed", conclusion="success"))

        self.assertEqual(results, [])
        self.assertFalse(any(call[0] == "add_labels" for call in api.calls))

    def test_completed_success_preserves_needs_human_when_peer_review_passed(self):
        pr = make_pr(labels=("needs-human",), assignees=("samcorcos",))
        runs = [{"id": 20, "name": "peer-review", "status": "completed", "conclusion": "success"}]
        api = FakeAPI(pulls=[pr], check_runs=runs)

        results = handoff.handle_event(api, make_event("completed", conclusion="success"))

        self.assertEqual(results, [])
        self.assertFalse(any(call[0] in ("remove_label", "remove_assignees", "add_labels") for call in api.calls))

    def test_completed_failure_or_cancelled_is_noop(self):
        for conclusion in ("failure", "cancelled", "timed_out", None):
            with self.subTest(conclusion=conclusion):
                pr = make_pr(labels=())
                api = FakeAPI(pulls=[pr], check_runs=[])
                results = handoff.handle_event(api, make_event("completed", conclusion=conclusion))
                self.assertEqual(results, [])
                self.assertEqual(api.calls, [])

    def test_same_repo_pr_is_ignored(self):
        pr = make_pr(head_repo=REPOSITORY, labels=("needs-human",), assignees=("samcorcos",))
        api = FakeAPI(pulls=[pr], check_runs=[])

        for action, conclusion in (("in_progress", None), ("completed", "success")):
            with self.subTest(action=action):
                event = make_event(action, head_repo=REPOSITORY, conclusion=conclusion)
                results = handoff.handle_event(api, event)
                self.assertEqual(results, [])
                self.assertEqual(api.calls, [])

    def test_non_pull_request_workflow_event_is_ignored(self):
        pr = make_pr(labels=("needs-human",))
        api = FakeAPI(pulls=[pr], check_runs=[])

        results = handoff.handle_event(api, make_event("in_progress", event="push"))

        self.assertEqual(results, [])
        self.assertEqual(api.calls, [])

    def test_stale_head_sha_is_ignored(self):
        pr = make_pr(head_sha=OTHER_SHA, labels=("needs-human",), assignees=("samcorcos",))
        api = FakeAPI(pulls=[pr], check_runs=[])

        for action, conclusion in (("in_progress", None), ("completed", "success")):
            with self.subTest(action=action):
                results = handoff.handle_event(api, make_event(action, head_sha=HEAD, conclusion=conclusion))
                self.assertEqual(results, [])
                self.assertFalse(any(call[0] in ("remove_label", "remove_assignees", "add_labels") for call in api.calls))

    def test_empty_workflow_run_pull_requests_resolves_via_open_pr_head_sha(self):
        pr = make_pr(number=99, head_sha=HEAD, head_repo=FORK_REPO, labels=())
        api = FakeAPI(pulls=[pr], check_runs=[])
        event = make_event("completed", conclusion="success", pull_requests=[])

        results = handoff.handle_event(api, event)

        self.assertEqual(results, [(99, ["add:needs-review"])])
        self.assertIn(("open_pulls",), api.calls)
        self.assertIn(("add_labels", 99, ["needs-review"]), api.calls)

    def test_invalid_head_sha_raises(self):
        api = FakeAPI()
        with self.assertRaises(ValueError):
            handoff.handle_event(api, make_event("in_progress", head_sha="not-a-sha"))


if __name__ == "__main__":
    unittest.main()
