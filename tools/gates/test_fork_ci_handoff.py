"""Unit tests for the trusted-base fork CI handoff gate (no GitHub writes)."""
import copy
import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import fork_ci_handoff as handoff

REPOSITORY = "samcorcos/legatum"
FORK_REPO = "contributor/legatum"
OTHER_FORK_REPO = "other/legatum"
HEAD = "a" * 40
OTHER_SHA = "b" * 40
RUN_ID = 5001


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


def make_run(
    *,
    run_id=RUN_ID,
    run_attempt=1,
    event="pull_request",
    head_sha=HEAD,
    head_repo=FORK_REPO,
    status="in_progress",
    conclusion=None,
    pull_requests=(),
):
    return {
        "id": run_id,
        "run_attempt": run_attempt,
        "event": event,
        "head_sha": head_sha,
        "head_repository": {"full_name": head_repo} if head_repo is not None else None,
        "status": status,
        "conclusion": conclusion,
        "pull_requests": list(pull_requests),
    }


def make_event(
    action,
    *,
    run_id=RUN_ID,
    run_attempt=1,
    event="pull_request",
    head_sha=HEAD,
    head_repo=FORK_REPO,
    conclusion=None,
    pull_requests=(),
):
    status = "completed" if action == "completed" else "in_progress"
    return {
        "action": action,
        "workflow_run": make_run(
            run_id=run_id,
            run_attempt=run_attempt,
            event=event,
            head_sha=head_sha,
            head_repo=head_repo,
            status=status,
            conclusion=conclusion,
            pull_requests=pull_requests,
        ),
    }


def make_comment(
    body,
    *,
    comment_id=100,
    login="samcorcos",
    author_association="OWNER",
    updated_at="2026-11-28T12:00:00Z",
):
    return {
        "id": comment_id,
        "user": {"login": login},
        "author_association": author_association,
        "updated_at": updated_at,
        "body": body,
    }


def ci_approval_comment(sha=HEAD, *, run_id=None, comment_id=100, login="samcorcos",
                        author_association="OWNER", updated_at="2026-11-28T12:00:00Z"):
    marker = f"Legatum ci-approval: {sha}" + (f" run={run_id}" if run_id is not None else "")
    body = f"{marker}\n\nRead-only safety check passed; awaiting maintainer CI approval."
    return make_comment(
        body,
        comment_id=comment_id,
        login=login,
        author_association=author_association,
        updated_at=updated_at,
    )


class FakeAPI:
    repository = REPOSITORY

    def __init__(
        self,
        *,
        pulls=None,
        live_pr=None,
        comments=None,
        live_comments=None,
        ci_runs=None,
        live_ci_runs=None,
        check_runs=None,
    ):
        self._pulls = list(pulls or [])
        self._live_pr = live_pr
        self._comments = list(comments or [])
        self._live_comments = live_comments
        self._ci_runs = list(ci_runs) if ci_runs is not None else None
        self._live_ci_runs = list(live_ci_runs) if live_ci_runs is not None else None
        self._check_runs = list(check_runs or [])
        self.calls = []
        self._comment_reads = 0
        self._ci_run_reads = 0

    def open_pulls(self):
        self.calls.append(("open_pulls",))
        return copy.deepcopy(self._pulls)

    def pull(self, number):
        self.calls.append(("pull", number))
        if self._live_pr is not None:
            return copy.deepcopy(self._live_pr)
        for pr in self._pulls:
            if pr["number"] == number:
                return copy.deepcopy(pr)
        raise AssertionError(f"Unknown PR #{number}")

    def comments(self, number):
        self.calls.append(("comments", number))
        self._comment_reads += 1
        if self._comment_reads > 1 and self._live_comments is not None:
            return copy.deepcopy(self._live_comments)
        return copy.deepcopy(self._comments)

    def ci_runs(self, head_sha):
        self.calls.append(("ci_runs", head_sha))
        self._ci_run_reads += 1
        if self._ci_run_reads > 1 and self._live_ci_runs is not None:
            return copy.deepcopy(self._live_ci_runs)
        if self._ci_runs is not None:
            return copy.deepcopy(self._ci_runs)
        return []

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
    def test_in_progress_clears_needs_human_with_trusted_marker(self):
        pr = make_pr(labels=("needs-human",), assignees=("samcorcos",))
        api = FakeAPI(
            pulls=[pr],
            comments=[ci_approval_comment(HEAD)],
            ci_runs=[make_run(status="in_progress")],
        )

        results = handoff.handle_event(api, make_event("in_progress"))

        self.assertEqual(results, [(42, ["remove:needs-human", "unassign:samcorcos"])])
        self.assertIn(("remove_label", 42, "needs-human"), api.calls)
        self.assertIn(("remove_assignees", 42, ["samcorcos"]), api.calls)
        self.assertFalse(any(call[0] == "add_labels" for call in api.calls))

    def test_in_progress_clears_needs_human_with_matching_run_id_marker(self):
        pr = make_pr(labels=("needs-human",), assignees=())
        api = FakeAPI(
            pulls=[pr],
            comments=[ci_approval_comment(HEAD, run_id=RUN_ID)],
            ci_runs=[make_run(run_id=RUN_ID, status="in_progress")],
        )

        results = handoff.handle_event(api, make_event("in_progress", run_id=RUN_ID))

        self.assertEqual(results, [(42, ["remove:needs-human"])])
        self.assertFalse(any(call[0] == "remove_assignees" for call in api.calls))

    def test_in_progress_preserves_design_or_final_merge_needs_human_without_marker(self):
        pr = make_pr(labels=("needs-human",), assignees=("samcorcos",))
        design_handoff = make_comment(
            "Needs maintainer design decision on #318 before proceeding.\n\n@samcorcos"
        )
        # peer-review is missing or failing, as is normal for a fork awaiting human decision/approval.
        for check_runs in (
            [],
            [{"id": 10, "name": "peer-review", "status": "completed", "conclusion": "failure"}],
        ):
            with self.subTest(check_runs=check_runs):
                api = FakeAPI(
                    pulls=[pr],
                    comments=[design_handoff],
                    ci_runs=[make_run(status="in_progress")],
                    check_runs=check_runs,
                )
                results = handoff.handle_event(api, make_event("in_progress"))
                self.assertEqual(results, [])
                self.assertFalse(any(c[0] in ("remove_label", "remove_assignees", "add_labels") for c in api.calls))

    def test_completed_success_preserves_unrelated_needs_human_without_marker(self):
        pr = make_pr(labels=("needs-human",), assignees=("samcorcos",))
        final_merge_handoff = make_comment("Read-only review complete. Ready for human merge.\n\n@samcorcos")
        api = FakeAPI(
            pulls=[pr],
            comments=[final_merge_handoff],
            ci_runs=[make_run(status="completed", conclusion="success")],
            check_runs=[{"id": 10, "name": "peer-review", "status": "completed", "conclusion": "failure"}],
        )

        results = handoff.handle_event(api, make_event("completed", conclusion="success"))

        self.assertEqual(results, [])
        self.assertFalse(any(c[0] in ("remove_label", "remove_assignees", "add_labels") for c in api.calls))

    def test_untrusted_fork_author_spoofing_marker_is_ignored(self):
        pr = make_pr(labels=("needs-human",), assignees=("samcorcos",))
        spoofed = ci_approval_comment(
            HEAD,
            login="outside-contributor",
            author_association="FIRST_TIME_CONTRIBUTOR",
        )
        api = FakeAPI(
            pulls=[pr],
            comments=[spoofed],
            ci_runs=[make_run(status="in_progress")],
        )

        results = handoff.handle_event(api, make_event("in_progress"))

        self.assertEqual(results, [])
        self.assertFalse(any(c[0] in ("remove_label", "remove_assignees", "add_labels") for c in api.calls))

    def test_fenced_marker_example_is_ignored(self):
        pr = make_pr(labels=("needs-human",), assignees=("samcorcos",))
        example = make_comment(f"Example only:\n```text\nLegatum ci-approval: {HEAD}\n```")
        api = FakeAPI(
            pulls=[pr],
            comments=[example],
            ci_runs=[make_run(status="in_progress")],
        )

        results = handoff.handle_event(api, make_event("in_progress"))

        self.assertEqual(results, [])
        self.assertFalse(any(c[0] in ("remove_label", "remove_assignees", "add_labels") for c in api.calls))

    def test_newer_trusted_comment_supersedes_older_ci_approval_marker(self):
        pr = make_pr(labels=("needs-human",), assignees=("samcorcos",))
        older = ci_approval_comment(HEAD, comment_id=100, updated_at="2026-11-28T12:00:00Z")
        newer = make_comment(
            "Actually, pausing for a human design decision before merging.",
            comment_id=101,
            updated_at="2026-11-28T12:10:00Z",
        )
        api = FakeAPI(
            pulls=[pr],
            comments=[older, newer],
            ci_runs=[make_run(status="in_progress")],
        )

        results = handoff.handle_event(api, make_event("in_progress"))

        self.assertEqual(results, [])
        self.assertFalse(any(c[0] in ("remove_label", "remove_assignees", "add_labels") for c in api.calls))

    def test_mixed_labels_preserve_needs_human_and_assignment(self):
        for labels in (
            ("needs-human", "reviewing:claude"),
            ("needs-human", "changes-requested"),
        ):
            for action, status, conclusion in (
                ("in_progress", "in_progress", None),
                ("completed", "completed", "success"),
            ):
                with self.subTest(labels=labels, action=action):
                    pr = make_pr(labels=labels, assignees=("samcorcos",))
                    api = FakeAPI(
                        pulls=[pr],
                        comments=[ci_approval_comment(HEAD)],
                        ci_runs=[make_run(status=status, conclusion=conclusion)],
                    )
                    results = handoff.handle_event(api, make_event(action, conclusion=conclusion))
                    self.assertEqual(results, [])
                    self.assertFalse(
                        any(c[0] in ("remove_label", "remove_assignees", "add_labels") for c in api.calls)
                    )

    def test_in_progress_preserves_needs_human_when_peer_review_passed(self):
        pr = make_pr(labels=("needs-human",), assignees=("samcorcos",))
        runs = [{"id": 10, "name": "peer-review", "status": "completed", "conclusion": "success"}]
        api = FakeAPI(
            pulls=[pr],
            comments=[ci_approval_comment(HEAD)],
            ci_runs=[make_run(status="in_progress")],
            check_runs=runs,
        )

        results = handoff.handle_event(api, make_event("in_progress"))

        self.assertEqual(results, [])
        self.assertFalse(any(call[0] in ("remove_label", "remove_assignees", "add_labels") for call in api.calls))

    def test_completed_success_adds_needs_review(self):
        pr = make_pr(labels=(), assignees=())
        api = FakeAPI(
            pulls=[pr],
            ci_runs=[make_run(status="completed", conclusion="success")],
        )

        results = handoff.handle_event(api, make_event("completed", conclusion="success"))

        self.assertEqual(results, [(42, ["add:needs-review"])])
        self.assertIn(("add_labels", 42, ["needs-review"]), api.calls)

    def test_completed_success_clears_lingering_needs_human_with_marker_and_adds_needs_review(self):
        pr = make_pr(labels=("needs-human",), assignees=("samcorcos",))
        api = FakeAPI(
            pulls=[pr],
            comments=[ci_approval_comment(HEAD)],
            ci_runs=[make_run(status="completed", conclusion="success")],
        )

        results = handoff.handle_event(api, make_event("completed", conclusion="success"))

        self.assertEqual(results, [(42, ["remove:needs-human", "unassign:samcorcos", "add:needs-review"])])

    def test_completed_success_skips_when_already_routed_or_draft(self):
        for kwargs in (
            {"labels": ("reviewing:claude",)},
            {"labels": ("changes-requested",)},
            {"labels": ("needs-review",)},
            {"draft": True},
        ):
            with self.subTest(kwargs=kwargs):
                pr = make_pr(**kwargs)
                api = FakeAPI(
                    pulls=[pr],
                    ci_runs=[make_run(status="completed", conclusion="success")],
                )
                results = handoff.handle_event(api, make_event("completed", conclusion="success"))
                self.assertEqual(results, [])
                self.assertFalse(any(call[0] == "add_labels" for call in api.calls))

    def test_completed_failure_or_cancelled_is_noop(self):
        for conclusion in ("failure", "cancelled", "timed_out", None):
            with self.subTest(conclusion=conclusion):
                pr = make_pr(labels=())
                api = FakeAPI(pulls=[pr], ci_runs=[make_run(status="completed", conclusion=conclusion)])
                results = handoff.handle_event(api, make_event("completed", conclusion=conclusion))
                self.assertEqual(results, [])
                self.assertEqual(api.calls, [])

    def test_same_repo_or_null_head_repo_or_mismatched_fork_repo_ignored(self):
        # Same-repo event ignored.
        pr = make_pr(head_repo=REPOSITORY, labels=("needs-human",), assignees=("samcorcos",))
        api = FakeAPI(pulls=[pr], comments=[ci_approval_comment(HEAD)])
        self.assertEqual(handoff.handle_event(api, make_event("in_progress", head_repo=REPOSITORY)), [])
        self.assertEqual(api.calls, [])

        # Null PR head.repo ignored.
        pr_null = make_pr(head_repo=None, labels=())
        api_null = FakeAPI(pulls=[pr_null], ci_runs=[make_run(status="completed", conclusion="success")])
        self.assertEqual(handoff.handle_event(api_null, make_event("completed", conclusion="success")), [])
        self.assertFalse(any(call[0] == "add_labels" for call in api_null.calls))

        # Different fork repo with same SHA ignored.
        pr_other = make_pr(head_repo=OTHER_FORK_REPO, labels=())
        api_other = FakeAPI(pulls=[pr_other], ci_runs=[make_run(status="completed", conclusion="success")])
        self.assertEqual(handoff.handle_event(api_other, make_event("completed", conclusion="success")), [])
        self.assertFalse(any(call[0] == "add_labels" for call in api_other.calls))

    def test_delayed_attempt_1_success_ignored_when_attempt_2_is_running_or_failing(self):
        pr = make_pr(labels=())
        for live_status, live_conclusion in (("in_progress", None), ("completed", "failure")):
            with self.subTest(live_status=live_status, live_conclusion=live_conclusion):
                api = FakeAPI(
                    pulls=[pr],
                    ci_runs=[make_run(run_id=RUN_ID, run_attempt=2, status=live_status, conclusion=live_conclusion)],
                )
                event = make_event("completed", run_id=RUN_ID, run_attempt=1, conclusion="success")
                results = handoff.handle_event(api, event, env_run_id=str(RUN_ID), env_run_attempt="1")
                self.assertEqual(results, [])
                self.assertFalse(any(call[0] == "add_labels" for call in api.calls))

    def test_superseded_by_newer_run_id_is_noop(self):
        pr = make_pr(labels=())
        api = FakeAPI(
            pulls=[pr],
            ci_runs=[
                make_run(run_id=RUN_ID, run_attempt=1, status="completed", conclusion="success"),
                make_run(run_id=RUN_ID + 1, run_attempt=1, status="in_progress", conclusion=None),
            ],
        )

        results = handoff.handle_event(api, make_event("completed", run_id=RUN_ID, conclusion="success"))

        self.assertEqual(results, [])
        self.assertFalse(any(call[0] == "add_labels" for call in api.calls))

    def test_head_sha_changed_between_discovery_and_live_reread_aborts(self):
        initial_pr = make_pr(head_sha=HEAD, labels=())
        live_pr = make_pr(head_sha=OTHER_SHA, labels=())
        api = FakeAPI(
            pulls=[initial_pr],
            live_pr=live_pr,
            ci_runs=[make_run(status="completed", conclusion="success")],
        )

        results = handoff.handle_event(api, make_event("completed", conclusion="success"))

        self.assertEqual(results, [])
        self.assertIn(("pull", 42), api.calls)
        self.assertFalse(any(call[0] == "add_labels" for call in api.calls))

    def test_labels_changed_between_discovery_and_live_reread_aborts(self):
        initial_pr = make_pr(labels=())
        live_pr = make_pr(labels=("needs-human",))
        api = FakeAPI(
            pulls=[initial_pr],
            live_pr=live_pr,
            ci_runs=[make_run(status="completed", conclusion="success")],
        )

        results = handoff.handle_event(api, make_event("completed", conclusion="success"))

        self.assertEqual(results, [])
        self.assertFalse(any(call[0] == "add_labels" for call in api.calls))

    def test_handoff_comment_superseded_between_initial_and_live_reread_aborts(self):
        pr = make_pr(labels=("needs-human",), assignees=("samcorcos",))
        initial_comments = [ci_approval_comment(HEAD, comment_id=100, updated_at="2026-11-28T12:00:00Z")]
        live_comments = initial_comments + [
            make_comment("Hold for design question.", comment_id=101, updated_at="2026-11-28T12:05:00Z")
        ]
        api = FakeAPI(
            pulls=[pr],
            comments=initial_comments,
            live_comments=live_comments,
            ci_runs=[make_run(status="in_progress")],
        )

        results = handoff.handle_event(api, make_event("in_progress"))

        self.assertEqual(results, [])
        self.assertFalse(any(c[0] in ("remove_label", "remove_assignees", "add_labels") for c in api.calls))

    def test_empty_workflow_run_pull_requests_resolves_via_open_pr_head_sha(self):
        pr = make_pr(number=99, head_sha=HEAD, head_repo=FORK_REPO, labels=())
        api = FakeAPI(
            pulls=[pr],
            ci_runs=[make_run(status="completed", conclusion="success")],
        )
        event = make_event("completed", conclusion="success", pull_requests=[])

        results = handoff.handle_event(api, event)

        self.assertEqual(results, [(99, ["add:needs-review"])])
        self.assertIn(("open_pulls",), api.calls)
        self.assertIn(("add_labels", 99, ["needs-review"]), api.calls)

    def test_invalid_head_sha_or_mismatched_env_run_id_raises(self):
        api = FakeAPI()
        with self.assertRaises(ValueError):
            handoff.handle_event(api, make_event("in_progress", head_sha="not-a-sha"))
        with self.assertRaises(ValueError):
            handoff.handle_event(api, make_event("in_progress", run_id=RUN_ID), env_run_id=str(RUN_ID + 1))


if __name__ == "__main__":
    unittest.main()
