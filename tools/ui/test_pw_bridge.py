"""Browser-free checks for the bridge's trust boundary and result handling."""

import importlib.util
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

MODULE = Path(__file__).with_name("pw-bridge.py")
spec = importlib.util.spec_from_file_location("pw_bridge", MODULE)
bridge = importlib.util.module_from_spec(spec)
spec.loader.exec_module(bridge)


class BridgeTest(unittest.TestCase):
    def setUp(self):
        self.request = {
            "id": "abc_123",
            "worktree": str(bridge.REPO),
            "specs": ["app/e2e/layout.spec.ts"],
        }

    def test_valid_request(self):
        self.assertEqual(bridge.validate(self.request, "abc_123")[1], self.request["specs"])

    def test_rejects_paths_and_mismatched_id(self):
        changes = (
            {"id": "other"},
            {"worktree": "/tmp"},
            {"worktree": "relative/path"},
            {"specs": ["app/e2e/../playwright.config.ts"]},
            {"specs": ["app/e2e/not-there.spec.ts"]},
            {"specs": ["app/e2e/layout.spec.ts", "--config=/tmp/x"]},
            {"project": "../../evil"},
            {"grep": ["bad"]},
        )
        for change in changes:
            with self.subTest(change=change), self.assertRaises(ValueError):
                bridge.validate({**self.request, **change}, "abc_123")

    def test_report_extracts_titles_and_error_snippets(self):
        with tempfile.TemporaryDirectory() as directory:
            report = Path(directory) / "report.json"
            report.write_text(json.dumps({"suites": [{"title": "layout.spec.ts", "specs": [
                {"title": "works", "tests": [{"outcome": "expected", "results": []}]},
                {"title": "fails", "tests": [{"outcome": "unexpected", "results": [{"errors": [{"message": "bad layout"}]}]}]},
            ]}]}))
            passed, failed, failures = bridge.parse_report(report)
            self.assertEqual((passed, failed), (1, 1))
            self.assertEqual(failures, [{"title": "layout.spec.ts › fails", "error": "bad layout"}])

    def test_invalid_request_produces_result_and_moves_to_done(self):
        with tempfile.TemporaryDirectory() as directory, patch.object(bridge, "QUEUE", Path(directory)):
            for name in ("requests", "results", "done"):
                (Path(directory) / name).mkdir()
            request = Path(directory) / "requests/abc_123.json"
            request.write_text(json.dumps({**self.request, "worktree": "/tmp"}))
            bridge.handle(request)
            result = Path(directory) / "results/abc_123"
            summary = json.loads((result / "summary.json").read_text())
            self.assertEqual(summary["exit_code"], 1)
            self.assertIn("outside", summary["error"])
            self.assertTrue((result / "output.log").exists())
            self.assertTrue((Path(directory) / "done/abc_123.json").exists())


if __name__ == "__main__":
    unittest.main()
