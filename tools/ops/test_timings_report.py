#!/usr/bin/env python3
"""Tests for the supervisor timing report using an isolated synthetic ledger."""
import csv
import datetime
import json
import os
import subprocess
import tempfile
import unittest
from pathlib import Path


REPORT = Path(__file__).with_name("timings-report.py")
TIMED = Path(__file__).with_name("timed.py")
COLUMNS = ["ts", "kind", "name", "label", "ref", "rc", "wall_s", "cpu_s", "load1", "cores"]


class TimingsReportTests(unittest.TestCase):
    def test_unwritable_ledger_preserves_child_output_and_status(self):
        env = dict(os.environ, LEGATUM_TIMINGS_DIR="/dev/null/cannot-write")
        result = subprocess.run(["python3", str(TIMED), "build", "probe", "--", "sh", "-c",
                                 "printf output; printf error >&2; exit 23"],
                               capture_output=True, text=True, env=env)
        self.assertEqual(result.returncode, 23)
        self.assertEqual(result.stdout, "output")
        self.assertEqual(result.stderr, "error")

    def test_kind_filter_and_json_summary(self):
        with tempfile.TemporaryDirectory() as temp:
            ledger = Path(temp)
            today = datetime.date.today().isoformat()
            now = datetime.datetime.now().replace(microsecond=0).isoformat()
            with (ledger / f"{today}.tsv").open("w", newline="") as f:
                writer = csv.DictWriter(f, fieldnames=COLUMNS, delimiter="\t", lineterminator="\n")
                writer.writeheader()
                writer.writerow(dict(ts=now, kind="build", name="wasm", label="profile=release",
                                     ref="abc", rc=0, wall_s=120, cpu_s=90, load1=1, cores=8))
                writer.writerow(dict(ts=now, kind="sim", name="duel", label="salts=1",
                                     ref="def", rc=0, wall_s=30, cpu_s=15, load1=1, cores=8))

            env = dict(os.environ, LEGATUM_TIMINGS_DIR=temp)
            result = subprocess.run(["python3", str(REPORT), "--kind", "build", "--json"],
                                    check=True, capture_output=True, text=True, env=env)
            report = json.loads(result.stdout)
            self.assertEqual(report["kind"], "build")
            self.assertEqual(report["last_days"]["runs"], 1)
            self.assertEqual(report["last_days"]["cpu_minutes"], 1.5)
            self.assertEqual(report["last_days"]["steps"][0]["name"], "wasm")
            self.assertEqual(report["last_days"]["steps"][0]["median_wall_minutes"], 2.0)

            markdown = subprocess.run(["python3", str(REPORT), "--kind", "build"],
                                      check=True, capture_output=True, text=True, env=env).stdout
            self.assertIn("build:wasm", markdown)
            self.assertNotIn("sim:duel", markdown)

    def test_unavailable_exit_is_not_a_flaky_test_result(self):
        with tempfile.TemporaryDirectory() as temp:
            ledger = Path(temp)
            today = datetime.date.today().isoformat()
            now = datetime.datetime.now().replace(microsecond=0).isoformat()
            with (ledger / f"{today}.tsv").open("w", newline="") as f:
                writer = csv.DictWriter(f, fieldnames=COLUMNS, delimiter="\t", lineterminator="\n")
                writer.writeheader()
                for ref, rc in (("test-ref", 0), ("test-ref", 1), ("cap-only", 3), ("cap-only", 0)):
                    writer.writerow(dict(ts=now, kind="box", name="remote", label="verify", ref=ref,
                                         rc=rc, wall_s=1, cpu_s=0, load1=1, cores=8))
            env = dict(os.environ, LEGATUM_TIMINGS_DIR=temp)
            result = subprocess.run(["python3", str(REPORT), "--json"], check=True,
                                    capture_output=True, text=True, env=env)
            report = json.loads(result.stdout)
            self.assertEqual(report["flaky"], [{"kind": "box", "name": "remote", "refs": 1}])

    def test_stubbed_verify_steps_do_not_pollute_the_ledger(self):
        with tempfile.TemporaryDirectory() as temp:
            env = dict(os.environ, LEGATUM_TIMINGS_DIR=temp, VERIFY_STEP_STUB="1")
            result = subprocess.run(["python3", str(TIMED), "--record", "verify", "fmt", "feature",
                                     "synthetic-ref", "1", "0", "0", "0"],
                                    check=True, capture_output=True, text=True, env=env)
            self.assertEqual(result.stdout, "")
            self.assertEqual(list(Path(temp).glob("*.tsv")), [])


if __name__ == "__main__":
    unittest.main()
