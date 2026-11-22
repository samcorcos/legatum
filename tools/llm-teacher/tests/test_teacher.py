"""Run: python3 -m unittest discover -s tools/llm-teacher/tests"""
import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import teacher as t  # noqa: E402

W = {
    "game": "g", "turn": 20,
    "state": "Rivals\n- Mongols: alive, at Peace\n    city Sarai (pop 1, no Walls)\n    city Karakorum (pop 3)\n",
    "candidates": [{"name": "hard", "label": "hard", "line": "l"}, {"name": "strike:Sarai", "label": "strike:Sarai", "line": "l"}],
    "events": ["x"], "hard_plan": "p", "notes": "n",
    "tape": {"declared": ["Mongols"], "marched_on": {"Sarai": 4}, "bought": [], "founded": 0, "produced": [], "accepted": [], "refused": []},
}


class Parse(unittest.TestCase):
    def test_chooser_strips_numbers(self):
        r = t.parse_chooser({"choice": "2. strike:Sarai", "ranking": ["2. strike:Sarai", "hard", "nope"]}, ["hard", "strike:Sarai"])
        self.assertTrue(r["valid"])
        self.assertEqual(r["ranking"], ["strike:Sarai", "hard"])

    def test_chooser_rejects_unknown(self):
        self.assertFalse(t.parse_chooser({"choice": "dance"}, ["hard"])["valid"])

    def test_proposer_grammar(self):
        j = {"proposals": [
            {"template": "short_strike", "target": "Karakorum"},
            {"template": "short_strike", "target": "Atlantis"},
            {"template": "strike", "target": "Sarai"},
        ], "other_ideas": ["x"]}
        r = t.parse_proposer(j, W)
        self.assertEqual([p["name"] for p in r["proposals"]], ["short_strike:Karakorum", "strike:Sarai"])
        self.assertEqual(r["dropped"], 1)
        self.assertTrue(r["proposals"][0]["new"])
        self.assertFalse(r["proposals"][1]["new"])
        self.assertEqual(t.parse_proposer({"proposals": [{"template": "teleport"}]}, W)["dropped"], 1)

    def test_tape_label(self):
        self.assertEqual(t.tape_label(W)[0], "strike:Sarai")

    def test_json_in_fences(self):
        self.assertEqual(t.parse_json('```json\n{"a": 1}\n```'), {"a": 1})


class Prompt(unittest.TestCase):
    def test_notes_only_in_notes_variant_and_no_referee(self):
        plain, _ = t.build_prompt(W, "chooser", "plain", 0)
        notes, _ = t.build_prompt(W, "chooser", "notes", 0)
        self.assertNotIn("How the player you advise reasons", plain)
        self.assertIn("How the player you advise reasons", notes)
        for p in (plain, notes):
            self.assertNotIn("gain_pp", p)
            self.assertNotIn("declared", p)  # the tape digest never reaches a prompt

    def test_shuffle_is_stable_per_repeat(self):
        a = [c["label"] for c in t.shuffled(W, 1)]
        self.assertEqual(a, [c["label"] for c in t.shuffled(W, 1)])


class Stats(unittest.TestCase):
    def test_rank_corr(self):
        self.assertEqual(t.rank_corr(["a", "b", "c"], ["a", "b", "c"]), 1.0)
        self.assertEqual(t.rank_corr(["a", "b", "c"], ["c", "b", "a"]), -1.0)


if __name__ == "__main__":
    unittest.main()
