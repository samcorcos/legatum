"""Tests for publish.py: finding the installed rules version, refusing a stale file, copying a good one.
Run: python3 -m unittest tools/scenarios/test_publish.py   (the engine and copy lint are stubbed)."""
import json
import os
import stat
import sys
import tempfile
import unittest
from unittest import mock

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import publish  # noqa: E402


def scenario(version, sid="demo"):
    return {"format": "legatum-scenario", "version": 1, "id": sid, "title": "Demo", "briefing": "Take the city.",
            "objective": {"summary": "Take it.", "turns": 5, "score": [{"metric": "ObjectiveMet", "weight": 1}]},
            "lab": {"question": "Can you?", "blurb": "Two lines."}, "baseline": {"runs": 5},
            "state": {"encoding": "json", "hash": "1", "data": json.dumps({"save_version": version})}}


class PublishTest(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.mkdtemp()
        self.app = os.path.join(self.tmp, "Legatum.app")
        os.makedirs(os.path.join(self.app, "Contents", "MacOS"))
        self.exe = os.path.join(self.app, "Contents", "MacOS", "legatum-desktop")
        open(self.exe, "w").close()
        self.data = os.path.join(self.tmp, "LegatumData")
        os.makedirs(self.data)
        self.dest = os.path.join(self.data, "scenarios", "lab")
        stub = os.path.join(self.tmp, "ok.sh")
        with open(stub, "w") as f:
            f.write("#!/bin/sh\nexit 0\n")
        os.chmod(stub, os.stat(stub).st_mode | stat.S_IXUSR)
        env = {"LEGATUM_SCENARIO_ENGINE": stub, "LEGATUM_SCENARIO_LINT": stub, "LEGATUM_DATA_DIR": self.data}
        patcher = mock.patch.dict(os.environ, env)
        patcher.start()
        self.addCleanup(patcher.stop)

    def write(self, version):
        path = os.path.join(self.tmp, "demo.json")
        with open(path, "w") as f:
            json.dump(scenario(version), f)
        return path

    def test_app_info_written_after_the_install_names_the_version(self):
        info = os.path.join(self.data, "app-info.json")
        json.dump({"rules_version": 23}, open(info, "w"))
        os.utime(self.exe, (1, 1))
        version, how = publish.installed_rules_version(self.app, self.data)
        self.assertEqual(version, 23)
        self.assertIn("app-info.json", how)

    def test_stale_app_info_is_ignored_and_a_missing_app_is_named(self):
        info = os.path.join(self.data, "app-info.json")
        json.dump({"rules_version": 1}, open(info, "w"))
        os.utime(info, (1, 1))  # written before the executable: the app was reinstalled since
        version, _ = publish.installed_rules_version(self.app, self.data)
        self.assertNotEqual(version, 1)
        none, why = publish.installed_rules_version(os.path.join(self.tmp, "Missing.app"), self.data)
        self.assertIsNone(none)
        self.assertIn("no installed app", why)

    def test_publishes_a_file_made_for_the_installed_version_and_refuses_another(self):
        good = self.write(23)
        self.assertEqual(publish.main([good, "--rules-version", "23", "--app", self.app]), 0)
        self.assertTrue(os.path.exists(os.path.join(self.dest, "demo.json")))
        os.remove(os.path.join(self.dest, "demo.json"))
        stale = self.write(22)
        self.assertEqual(publish.main([stale, "--rules-version", "23", "--app", self.app]), 1)
        self.assertFalse(os.path.exists(os.path.join(self.dest, "demo.json")))

    def test_dry_run_copies_nothing(self):
        self.assertEqual(publish.main([self.write(23), "--rules-version", "23", "--dry-run"]), 0)
        self.assertFalse(os.path.exists(self.dest))


if __name__ == "__main__":
    unittest.main()
