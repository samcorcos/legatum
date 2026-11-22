"""Tests for the card loop nightly: the shared split, contamination marks and score-only lookups. No simulation."""
import importlib.util
import json
import tempfile
from pathlib import Path
import unittest

spec = importlib.util.spec_from_file_location(
    "card_loop_nightly", Path(__file__).with_name("card_loop_nightly.py")
)
nightly = importlib.util.module_from_spec(spec)
spec.loader.exec_module(nightly)

SPLIT = {"train": ["mongols-ride-to-dyrrachium"], "held_out": ["germany-defend-essen"]}


class CardLoopNightlyTest(unittest.TestCase):
    def test_reads_the_scenario_gate_split(self):
        self.assertEqual(nightly.SPLIT.relative_to(nightly.ROOT).as_posix(),
                         "docs/sim-results/scenario-gate/split.json")
        split = nightly.load_split()
        lab = {p.stem for p in (nightly.ROOT / "scenarios/lab").glob("*.json")}
        self.assertTrue(lab <= set(split["train"]) | set(split["held_out"]),
                        "every Lab card must be in the scenario-gate split")

    def test_side(self):
        self.assertEqual(nightly.side("mongols-ride-to-dyrrachium", SPLIT), "train")
        self.assertEqual(nightly.side("germany-defend-essen", SPLIT), "held out")
        self.assertEqual(nightly.side("new-card", SPLIT), "not in split")

    def test_contaminated_runs_and_tasks_are_found_and_marked_once(self):
        with tempfile.TemporaryDirectory() as d:
            out = Path(d)
            for card in ("germany-defend-essen", "mongols-ride-to-dyrrachium"):
                (out / "runs/2026-11-09" / card).mkdir(parents=True)
            tasks = out / "tasks/2026-11-09"
            tasks.mkdir(parents=True)
            (tasks / "germany-defend-essen-t134.md").write_text("mode: write\n")
            (tasks / "mongols-ride-to-dyrrachium-t37.md").write_text("mode: write\n")
            dirty = nightly.contaminated(out, SPLIT)
            self.assertEqual([p.relative_to(out).as_posix() for p in dirty],
                             ["runs/2026-11-09/germany-defend-essen",
                              "tasks/2026-11-09/germany-defend-essen-t134.md"])
            nightly.mark_task(dirty[1])
            nightly.mark_task(dirty[1])
            text = dirty[1].read_text()
            self.assertEqual(text.count(nightly.CONTAMINATED), 1)
            self.assertTrue(text.endswith("mode: write\n"))

    def test_latest_search_takes_the_newest_day(self):
        with tempfile.TemporaryDirectory() as d:
            out = Path(d)
            for day, wins in (("2026-11-09", 50), ("2026-11-10", 40)):
                run = out / "runs" / day / "card-a"
                run.mkdir(parents=True)
                (run / "result.json").write_text(json.dumps({"line_fresh": {"wins": wins}}))
            (out / "tasks/2026-11-10").mkdir(parents=True)
            (out / "tasks/2026-11-10/card-a-t5.md").write_text("")
            res, task = nightly.latest_search(out, "card-a")
            self.assertEqual(res["line_fresh"]["wins"], 40)
            self.assertEqual(task.name, "card-a-t5.md")
            self.assertEqual(nightly.latest_search(out, "card-b"), (None, None))


if __name__ == "__main__":
    unittest.main()
