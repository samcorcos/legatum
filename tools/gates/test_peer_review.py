"""Security boundaries for the trusted-base peer review gate (no GitHub writes)."""
import copy
import os
import json
import sys
import subprocess
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parent))
import peer_review as gate

HEAD = "a" * 40
BASE = "b" * 40
REPOSITORY = "samcorcos/legatum"
DOCS = "legatum-docs-reviewer"


def pr():
    return {"state": "open", "draft": False, "user": {"login": "samcorcos"},
            "labels": [{"name": "author:chatgpt"}],
            "head": {"sha": HEAD, "ref": "chatgpt/task", "repo": {"full_name": REPOSITORY}},
            "base": {"sha": BASE, "ref": "master", "repo": {"full_name": REPOSITORY}}}


def comment(family="gemini", verdict="peer: PASS", number=1, sha=HEAD, day=1):
    return {"id": number, "user": {"login": "samcorcos"},
            "updated_at": f"2026-11-{day:02d}T12:00:00Z",
            "body": f"Legatum review: {family} at {sha}\n{verdict}\n\nChecked the changed paths and required behavior."}


class FakeAPI:
    repository = REPOSITORY
    comments = gate.GitHub.comments

    def __init__(self, comments=None, refreshed_pr=None, refreshed_comments=None):
        self.initial = pr()
        self.refreshed_pr = refreshed_pr
        self.initial_comments = comments if comments is not None else [comment()]
        self.refreshed_comments = refreshed_comments
        self.calls = []
        self.pr_reads = 0

    def request(self, method, path, data=None):
        self.calls.append((method, path, copy.deepcopy(data)))
        if path.startswith("/pulls/"):
            self.pr_reads += 1
            return copy.deepcopy(self.refreshed_pr if self.pr_reads > 1 and self.refreshed_pr is not None else self.initial)
        if path == "/check-runs":
            return {"id": 123}
        if path.startswith("/issues/"):
            page = int(path.rsplit("=", 1)[1])
            rows = self.refreshed_comments if self.pr_reads > 1 and self.refreshed_comments is not None else self.initial_comments
            return copy.deepcopy(rows[(page - 1) * 100:page * 100])
        if method == "PATCH":
            return {}
        raise AssertionError((method, path))


class ParsingTests(unittest.TestCase):
    def test_canonical_and_crlf(self):
        row = comment(verdict=f"peer: PASS\n{DOCS}: PASS")
        self.assertEqual(gate.parse_comment(row), ("gemini", HEAD, {"peer": "PASS", DOCS: "PASS"}))
        row["body"] = row["body"].replace("\n", "\r\n")
        self.assertIsNotNone(gate.parse_comment(row))

    def test_rejects_noncanonical_comments(self):
        original = comment()["body"]
        bodies = ["Example:\n" + original, "```\n" + original + "\n```", "> " + original,
                  original.replace(HEAD, HEAD[:7]), original.replace(HEAD, HEAD.upper()),
                  original.replace("gemini", "unknown"), original.replace("PASS", "pass"),
                  original.replace("peer: PASS", "unknown: PASS"),
                  original.replace("peer: PASS", "peer: PASS\npeer: FAIL"),
                  original.split("\n\n")[0] + "\n\n", None]
        for body in bodies:
            with self.subTest(body=body):
                row = comment()
                row["body"] = body
                self.assertIsNone(gate.parse_comment(row))
        row = comment()
        row["user"]["login"] = "outside-user"
        self.assertIsNone(gate.parse_comment(row))

    def test_event_numbers_and_issue_skip(self):
        self.assertIsNone(gate.pr_number("issue_comment", {"issue": {"number": 4}}))
        self.assertEqual(gate.pr_number("issue_comment", {"issue": {"number": 4, "pull_request": {}}}), 4)
        self.assertEqual(gate.pr_number("pull_request_target", {"pull_request": {"number": 5}}), 5)
        self.assertEqual(gate.pr_number("workflow_dispatch", {}, "6"), 6)
        for value in [True, 0, -1, "1/../../", "1\n", "01"]:
            with self.assertRaises(ValueError):
                gate.pr_number("workflow_dispatch", {}, value)


class VerdictTests(unittest.TestCase):
    def test_independent_peer_and_author_specialist(self):
        rows = [comment("chatgpt", f"{DOCS}: PASS"), comment(number=2)]
        self.assertEqual(gate.evaluate(rows, HEAD, {"chatgpt"}, {DOCS}), [])
        self.assertTrue(gate.evaluate(rows, HEAD, {"chatgpt", "gemini"}, {DOCS}))
        self.assertTrue(gate.evaluate([comment("chatgpt")], HEAD, {"chatgpt"}, set()))
        self.assertTrue(gate.evaluate([comment(sha=BASE)], HEAD, {"chatgpt"}, set()))

    def test_latest_per_family_role_and_outstanding_failures(self):
        rows = [comment(), comment(verdict="peer: FAIL", number=2, day=2), comment("claude", number=3, day=3)]
        self.assertIn("Outstanding failures", gate.evaluate(rows, HEAD, {"chatgpt"}, set())[0])
        rows.append(comment(number=4, day=4))
        self.assertEqual(gate.evaluate(rows, HEAD, {"chatgpt"}, set()), [])
        # Editing an older failure later revokes its family's newer PASS.
        rows[1]["updated_at"] = "2026-11-05T12:00:00Z"
        self.assertTrue(gate.evaluate(rows, HEAD, {"chatgpt"}, set()))

    def test_deletion_and_edit_revoke_approval(self):
        self.assertTrue(gate.evaluate([], HEAD, {"chatgpt"}, set()))
        row = comment()
        row["body"] = "Approval withdrawn"
        self.assertTrue(gate.evaluate([row], HEAD, {"chatgpt"}, set()))

    def test_identity_and_human_required_provenance(self):
        self.assertEqual(gate.identity(pr(), REPOSITORY), {"chatgpt"})
        rows = []
        for branch in ["chatgpt", "chatgpt/", "claude/task", "fix/task"]:
            row = pr()
            row["head"]["ref"] = branch
            rows.append(row)
        for labels in [[], [{"name": "author:unknown"}]]:
            row = pr()
            row["labels"] = labels
            rows.append(row)
        for repo in [{"full_name": "outsider/fork"}, None]:
            row = pr()
            row["head"]["repo"] = repo
            rows.append(row)
        row = pr()
        row["user"]["login"] = "outside-user"
        rows.append(row)
        for row in rows:
            with self.subTest(row=row):
                self.assertIsNone(gate.identity(row, REPOSITORY))
        for field, value in [("ref", "other"), ("repo", {"full_name": "outsider/repo"}), ("sha", "bad")]:
            row = pr()
            row["base"][field] = value
            with self.assertRaises(ValueError):
                gate.identity(row, REPOSITORY)
        row = pr()
        row["labels"].append({"name": "author:claude"})
        self.assertEqual(gate.identity(row, REPOSITORY), {"chatgpt", "claude"})

    def test_human_approval_still_requires_specialists_and_no_failures(self):
        human = comment("human")
        self.assertEqual(gate.parse_comment(human), ("human", HEAD, {"peer": "PASS"}))
        for authors in [None, {"chatgpt", "gemini", "claude"}]:
            self.assertEqual(gate.evaluate([human], HEAD, authors, set()), [])
            self.assertIn("Missing specialist", gate.evaluate([human], HEAD, authors, {DOCS})[0])
            rows = [human, comment("chatgpt", f"{DOCS}: PASS", number=2)]
            self.assertEqual(gate.evaluate(rows, HEAD, authors, {DOCS}), [])
            rows.append(comment("gemini", "peer: FAIL", number=3))
            self.assertIn("Outstanding failures", gate.evaluate(rows, HEAD, authors, {DOCS})[0])
        self.assertTrue(gate.evaluate([comment()], HEAD, None, set()))
        human["user"]["login"] = "outside-user"
        self.assertIsNone(gate.parse_comment(human))
        self.assertTrue(gate.evaluate([human], HEAD, None, set()))
        self.assertTrue(gate.evaluate([comment("human", sha=BASE)], HEAD, None, set()))


class EvaluationTests(unittest.TestCase):
    def run_gate(self, api, classifier=None):
        result = gate.run(api, 381, classifier or (lambda *_: set()))
        self.assertEqual(api.calls[1][2]["status"], "in_progress")
        self.assertEqual(api.calls[1][2]["head_sha"], HEAD)
        self.assertEqual(api.calls[-1][2]["conclusion"], "success" if result else "failure")
        return result

    def test_paginated_success(self):
        rows = [{**comment(number=i), "body": "Ordinary discussion"} for i in range(1, 101)] + [comment(number=101)]
        api = FakeAPI(rows)
        self.assertTrue(self.run_gate(api))
        self.assertEqual(sum("page=2" in call[1] for call in api.calls), 2)

    def test_changed_head_labels_and_revoked_comments_fail_closed(self):
        for change in ["head", "labels", "delete", "edit"]:
            api = FakeAPI()
            if change in {"head", "labels"}:
                api.refreshed_pr = pr()
                if change == "head":
                    api.refreshed_pr["head"]["sha"] = "c" * 40
                else:
                    api.refreshed_pr["labels"].append({"name": "changes-requested"})
            else:
                api.refreshed_comments = [] if change == "delete" else [comment(verdict="peer: FAIL")]
            with self.subTest(change=change):
                self.assertFalse(self.run_gate(api))

    def test_irrelevant_repository_metadata_does_not_invalidate(self):
        refreshed = pr()
        refreshed["head"]["repo"]["stargazers_count"] = 12
        self.assertTrue(self.run_gate(FakeAPI(refreshed_pr=refreshed)))

    def test_classifier_failure_and_missing_specialists_fail_closed(self):
        def broken(*_):
            raise RuntimeError("git diff failed")
        api = FakeAPI()
        with self.assertRaisesRegex(RuntimeError, "git diff failed"):
            gate.run(api, 381, broken)
        self.assertEqual(api.calls[-1][2]["conclusion"], "failure")
        self.assertFalse(self.run_gate(FakeAPI(), lambda *_: {DOCS}))

    def test_comments_read_failure_completes_check_as_failure(self):
        api = FakeAPI()
        with patch.object(api, "comments", side_effect=RuntimeError("GitHub unavailable")):
            with self.assertRaisesRegex(RuntimeError, "GitHub unavailable"):
                gate.run(api, 381, lambda *_: set())
        self.assertEqual(api.calls[-1][2]["conclusion"], "failure")

    def test_closed_pr_skips_post_and_concurrent_closure_cancels(self):
        api = FakeAPI()
        api.initial["state"] = "closed"
        self.assertIsNone(gate.run(api, 381))
        self.assertEqual(len(api.calls), 1)
        for rows in [[], [comment()]]:
            closed = pr()
            closed["state"] = "closed"
            api = FakeAPI(rows, refreshed_pr=closed)
            self.assertFalse(gate.run(api, 381, lambda *_: set()))
            self.assertEqual(api.calls[-1][2]["conclusion"], "cancelled")

    def test_main_waiting_is_success_but_operational_failure_raises(self):
        with tempfile.TemporaryDirectory() as directory:
            event = Path(directory) / "event.json"
            event.write_text(json.dumps({"inputs": {"pr": "381"}}))
            env = {"GITHUB_EVENT_PATH": str(event), "GITHUB_EVENT_NAME": "workflow_dispatch",
                   "GITHUB_REPOSITORY": REPOSITORY, "GITHUB_TOKEN": "test"}
            api = FakeAPI()
            api.initial["draft"] = True
            with patch.dict(os.environ, env), patch.object(gate, "GitHub", return_value=api):
                self.assertEqual(gate.main(), 0)
            self.assertEqual(api.calls[-1][2]["conclusion"], "failure")
            api = FakeAPI()
            run = gate.run
            with patch.dict(os.environ, env), patch.object(gate, "GitHub", return_value=api), \
                    patch.object(api, "comments", side_effect=RuntimeError("API unavailable")), \
                    patch.object(gate, "run", side_effect=lambda api, number: run(api, number, lambda *_: set())):
                with self.assertRaisesRegex(RuntimeError, "API unavailable"):
                    gate.main()
            self.assertEqual(api.calls[-1][2]["conclusion"], "failure")

    def test_human_only_route_and_noncanonical_approval(self):
        for body in [comment()["body"], "Example:\n" + comment("human")["body"], comment("human", sha=BASE)["body"]]:
            api = FakeAPI([{**comment(), "body": body}])
            api.initial["head"]["repo"]["full_name"] = "outside/fork"
            self.assertFalse(self.run_gate(api))
        api = FakeAPI([comment("human", f"peer: PASS\n{DOCS}: PASS")])
        api.initial["head"]["repo"]["full_name"] = "outside/fork"
        self.assertTrue(self.run_gate(api, lambda *_: {DOCS}))

    def test_strict_git_errors_do_not_become_empty_diffs(self):
        with patch.object(gate.subprocess, "run") as process:
            process.return_value.returncode = 1
            with self.assertRaises(RuntimeError):
                gate.strict_git(".", "diff", BASE, HEAD)
            args = process.call_args.args[0]
            self.assertIn("--no-textconv", args)
            self.assertIn("--no-ext-diff", args)

    def test_real_git_classifies_data_without_executing_pr_code(self):
        with tempfile.TemporaryDirectory() as directory:
            source = Path(directory) / "source"
            checkout = Path(directory) / "checkout"
            def git(where, *args):
                return subprocess.check_output(["git", "-C", str(where), *args], stderr=subprocess.DEVNULL, text=True).strip()
            source.mkdir()
            git(source, "init", "-b", "master")
            git(source, "config", "user.name", "Test")
            git(source, "config", "user.email", "test@example.com")
            (source / "README.md").write_text("Base\n")
            git(source, "add", ".")
            git(source, "commit", "-m", "Base")
            base = git(source, "rev-parse", "HEAD")
            git(source, "checkout", "-b", "chatgpt/test")
            (source / "app/src").mkdir(parents=True)
            (source / "app/src/example.ts").write_text('const message = "Automate your city to improve its economy";\n')
            (source / "tools/gates").mkdir(parents=True)
            (source / "tools/gates/check.py").write_text('raise RuntimeError("PR code must never execute")\n')
            git(source, "add", ".")
            git(source, "commit", "-m", "Untrusted proposed changes")
            head = git(source, "rev-parse", "HEAD")
            git(source, "update-ref", "refs/pull/381/head", head)
            git(source, "checkout", "master")
            (source / "README.md").write_text("Trusted master advanced\n")
            git(source, "add", ".")
            git(source, "commit", "-m", "Advance trusted master")
            trusted = git(source, "rev-parse", "HEAD")
            self.assertNotEqual(trusted, base)
            subprocess.check_call(["git", "clone", "--quiet", "--branch", "master", str(source), str(checkout)])
            row = pr()
            row["base"]["sha"], row["head"]["sha"] = base, head
            row["head"]["repo"]["full_name"] = "outside/fork"
            self.assertIsNone(gate.identity(row, REPOSITORY))
            roles = gate.required_roles(row, 381, str(checkout))
            self.assertIn(DOCS, roles)
            self.assertIn("legatum-plain-language-reviewer", roles)
            self.assertEqual(git(checkout, "rev-parse", "HEAD"), trusted)
            self.assertFalse((checkout / "tools/gates/check.py").exists())

    def test_classifier_uses_strict_git_and_restores_helper(self):
        original = gate.common.git
        outputs = [BASE, "", HEAD, BASE, "docs/example.md\0"]
        def classify(*_):
            self.assertIs(gate.common.git, strict)
            raise RuntimeError("diff failed")
        with patch.object(gate, "strict_git", side_effect=outputs) as strict, patch.object(gate.check, "classify", side_effect=classify):
            with self.assertRaises(RuntimeError):
                gate.required_roles(pr(), 381)
        self.assertIs(gate.common.git, original)


if __name__ == "__main__":
    unittest.main()
