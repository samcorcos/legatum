"""Portable analysis/recovery tests; gzip fixtures also run before Python 3.14."""
import gzip
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

SCRIPT = Path(__file__).with_name("archive-stats")


class ArchiveStatsTest(unittest.TestCase):
    def test_counts_csv_and_crash_repair_without_replaying_old_rules(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            folder = root / "2026" / "09"
            folder.mkdir(parents=True)
            meta = {
                "schema_version": 1, "game_id": "example", "created_at": "2026-09-27T12:00:00Z",
                "finished_at": "2026-09-27T13:00:00Z", "source": "human-local", "speed": "Standard",
                "seed": "18446744073709551615", "outcome": "vp", "turn_count": 1,
                "rules_fingerprint": "old-unavailable-engine",
                "seats": [{"player": 0, "name": "=Name", "faction": "Japan", "final_vp": 10}],
            }
            recording = {"metadata": meta, "telemetry": [{"turn": 1, "players": [{"player": 0, "vp": 10}]}]}
            with gzip.open(folder / "example.tape.json.gz", "wt") as file:
                json.dump(recording, file)
            # Crash left only a partial UTF-8 line in the index.
            (root / "index.jsonl").write_bytes(b'{"name":"\xc3')
            def run(*args):
                return subprocess.run([sys.executable, str(SCRIPT), "--archive-dir", str(root), *args], text=True, capture_output=True, check=True)
            result = run("--repair-index", "--csv", str(root / "csv"))
            self.assertIn("Recovered 1 missing index entries", result.stdout)
            self.assertIn("Games: 1", result.stdout)
            self.assertIn("Japan: 1", result.stdout)
            self.assertIn("'" + "=Name", (root / "csv" / "players.csv").read_text())
            self.assertIn(meta["seed"], (root / "csv" / "games.csv").read_text())
            self.assertIn("example,1,0,10", (root / "csv" / "turns.csv").read_text())
            self.assertIn("Recovered 0 missing index entries", run("--repair-index").stdout)
            self.assertEqual(json.loads((root / "index.jsonl").read_text()), meta)

    def test_corrupt_compression_fails_instead_of_omitting_the_game(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            folder = root / "2026" / "09"
            folder.mkdir(parents=True)
            (folder / "bad.tape.json.gz").write_bytes(b"corrupt")
            result = subprocess.run([sys.executable, str(SCRIPT), "--archive-dir", str(root), "--repair-index"], capture_output=True, text=True)
            self.assertNotEqual(result.returncode, 0)
            self.assertIn("archive-stats:", result.stderr)


if __name__ == "__main__":
    unittest.main()
