import tempfile
import unittest
from pathlib import Path

from tools.sim import freshness


class FreshnessTest(unittest.TestCase):
    def test_dirty_readme_is_not_attributed_to_clean_commit(self):
        with tempfile.TemporaryDirectory() as directory:
            report = Path(directory) / "README.md"
            report.write_text("Engine commit `24b39619-dirty`; 4,096 games.\n")
            self.assertIsNone(freshness.recorded_commit(report))

    def test_clean_readme_commit_is_found(self):
        with tempfile.TemporaryDirectory() as directory:
            report = Path(directory) / "README.md"
            report.write_text("Engine commit `e3a4b6fb`; 4,096 games.\n")
            self.assertEqual(freshness.recorded_commit(report), "e3a4b6fb")

    def test_dirty_source_stamp_remains_unknown(self):
        self.assertEqual(freshness.current_stamp("24b39619-dirty"), freshness.UNKNOWN)

    def test_mixed_ledger_preserves_old_rules_version(self):
        with tempfile.TemporaryDirectory() as directory:
            ledger = Path(directory) / "history.jsonl"
            ledger.write_text('{"rules_version":14}\n{"rules_version":15}\n')
            self.assertEqual(freshness.ledger_versions(ledger), [14, 15])


if __name__ == "__main__":
    unittest.main()
