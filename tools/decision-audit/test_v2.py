import importlib.util
import json
import tempfile
import unittest
from pathlib import Path

spec = importlib.util.spec_from_file_location("v2_report", Path(__file__).with_name("v2_report.py"))
report = importlib.util.module_from_spec(spec)
spec.loader.exec_module(report)


class PairedReportTest(unittest.TestCase):
    def test_rival_delta_uses_designer_target_and_rival_vp(self):
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp)
            (path / "fidelity.json").write_text(json.dumps({"hash_mismatches": 0, "salts": 2, "seat": 2}))
            (path / "candidates.json").write_text(json.dumps([{"turn": 4, "arm": "alt:3", "kind": "buy"}]))
            rows = []
            for salt in range(2):
                for arm, designer_chance, rival_chance, rival_vp in [("rec:3", 8500, 2000, 1), ("alt:3", 2500, 4500, 3)]:
                    rows.append({"turn": 4, "arm": arm, "salt": salt,
                                 "h": [{"wc": rival_chance, "vp": rival_vp}],
                                 "target_h": [{"wc": designer_chance, "vp": 5 if designer_chance == 8500 else 4}]})
            (path / "value.jsonl").write_text("\n".join(map(json.dumps, rows)))
            result = report.paired(path, "rival")["results"][0]
            self.assertEqual(result["win_pp"]["mean"], 60)
            self.assertEqual(result["rival_win_pp_gain"]["mean"], 25)
            self.assertEqual(result["rival_vp_gain"]["mean"], 2)

    def test_hash_mismatch_blocks_report(self):
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp)
            (path / "fidelity.json").write_text('{"hash_mismatches":1}')
            with self.assertRaises(ValueError):
                report.paired(path, "window")


if __name__ == "__main__": unittest.main()
