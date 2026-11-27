#!/usr/bin/env python3
"""Tests for tools/gates/e2e-strings.py. Run: python3 tools/gates/test_e2e_strings.py"""
import importlib.util
import os
import subprocess
import tempfile
import unittest

HERE = os.path.dirname(os.path.abspath(__file__))
spec = importlib.util.spec_from_file_location("e2e_strings", os.path.join(HERE, "e2e-strings.py"))
gate = importlib.util.module_from_spec(spec)
spec.loader.exec_module(gate)


def run(repo, *args):
    subprocess.run(["git", "-C", repo, *args], check=True, capture_output=True)


def write(repo, path, text):
    full = os.path.join(repo, path)
    os.makedirs(os.path.dirname(full), exist_ok=True)
    with open(full, "w") as f:
        f.write(text)


class Strings(unittest.TestCase):
    def test_literals_jsx_text_and_template_pieces(self):
        self.assertEqual(gate.strings("title={`AI · ${level}. Open Diplomacy`}"), [". Open Diplomacy"])
        self.assertEqual(gate.strings("<span>Not met yet</span>"), ["Not met yet"])
        self.assertEqual(gate.strings("                    Host this save"), ["Host this save"])
        self.assertEqual(gate.strings("import { x } from '../store';"), [])
        self.assertEqual(gate.strings("// 'Old words here' in a comment"), [])


class Check(unittest.TestCase):
    def setUp(self):
        self.repo = tempfile.mkdtemp()
        run(self.repo, "init", "-q", "-b", "master")
        run(self.repo, "config", "user.email", "t@example.com")
        run(self.repo, "config", "user.name", "t")
        write(self.repo, "app/src/ui/Panel.tsx", "export const A = () => <button title=\"Open the Guide\">Host this game</button>;\n"
                                                 "export const B = 'Still here today';\n")
        write(self.repo, "app/e2e/panel.spec.ts", "await expect(b).toHaveAttribute('title', /^Open the Guide$/);\n"
                                                  "await page.getByRole('button', { name: 'Host this game' });\n"
                                                  "// Host this game is only named in a comment here\n"
                                                  "expect(x).toBe('Still here today');\n")
        run(self.repo, "add", "-A")
        run(self.repo, "commit", "-qm", "base")
        self.base = subprocess.run(["git", "-C", self.repo, "rev-parse", "HEAD"], capture_output=True, text=True).stdout.strip()

    def commit(self, path, text):
        write(self.repo, path, text)
        run(self.repo, "commit", "-qam", "change")
        return subprocess.run(["git", "-C", self.repo, "rev-parse", "HEAD"], capture_output=True, text=True).stdout.strip()

    def test_a_reworded_label_flags_the_spec_lines_that_still_expect_it(self):
        tip = self.commit("app/src/ui/Panel.tsx", "export const A = () => <button title=\"Open Guide\">Host a game</button>;\n"
                                                  "export const B = 'Still here today';\n")
        found = gate.check(self.repo, self.base, tip)
        self.assertEqual(len(found), 2, found)
        self.assertTrue(found[0].startswith("app/e2e/panel.spec.ts:1:") and "Open the Guide" in found[0])
        self.assertTrue(found[1].startswith("app/e2e/panel.spec.ts:2:") and "Host this game" in found[1])

    def test_words_moved_to_another_file_or_kept_in_a_spec_comment_pass(self):
        write(self.repo, "app/src/ui/Other.tsx", "export const C = 'Open the Guide' + 'Host this game';\n")
        run(self.repo, "add", "-A")
        tip = self.commit("app/src/ui/Panel.tsx", "export const B = 'Still here today';\n")
        self.assertEqual(gate.check(self.repo, self.base, tip), [])

    def test_gate_allow_on_the_spec_line(self):
        self.commit("app/e2e/panel.spec.ts", "// gate-allow: checks the old wording is gone\n"
                                             "await expect(page.getByText('Host this game')).toHaveCount(0);\n")
        base = subprocess.run(["git", "-C", self.repo, "rev-parse", "HEAD"], capture_output=True, text=True).stdout.strip()
        tip = self.commit("app/src/ui/Panel.tsx", "export const A = () => <button title=\"Open the Guide\">Host a game</button>;\n"
                                                  "export const B = 'Still here today';\n")
        self.assertEqual(gate.check(self.repo, base, tip), [])


if __name__ == "__main__":
    unittest.main()
