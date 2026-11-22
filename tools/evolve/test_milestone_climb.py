"""Unit tests for the milestone climb's gene plumbing (no simulator needed)."""

import json
import re
import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import genome  # noqa: E402
import milestone_climb as mc  # noqa: E402

GENESETS = Path(__file__).resolve().parent / "genesets"


class MilestoneClimbTest(unittest.TestCase):
    def test_catalog_matches_the_python_allowlist(self):
        catalog = mc.gene_catalog()
        decisions = {k.split(".", 1)[1]: v for k, v in catalog.items() if k.startswith("decision.")}
        self.assertEqual(set(decisions), set(genome.DECISION_RANGES))
        for name, (_, lo, hi) in decisions.items():
            self.assertEqual((lo, hi), genome.DECISION_RANGES[name], name)
        factions = re.findall(r"Faction::(\w+) => Knobs::new", mc.KNOBS_RS.read_text())
        self.assertEqual(sum(k.startswith("knob.") for k in catalog), len(mc.KNOB_ORDER) * len(factions))

    def test_genesets_name_real_genes(self):
        catalog = mc.gene_catalog()
        for path in GENESETS.glob("*.json"):
            data = json.loads(path.read_text())
            names = data["genes"] if isinstance(data, dict) else data
            self.assertEqual(len(names), len(set(names)), path.name)
            unknown = [n for n in names if n not in catalog and n not in {
                f"knob.{knob}" for knob in mc.KNOB_ORDER
            }]
            self.assertEqual(unknown, [], path.name)

    def test_defaults_round_trip_and_params_are_complete(self):
        catalog = mc.gene_catalog()
        names = json.loads((GENESETS / "sparta-rush.json").read_text())
        defaults = [catalog[n][0] for n in names]
        self.assertEqual(mc.decode(names, catalog, mc.encode(names, catalog, defaults)), defaults)
        params = mc.to_params(names, defaults, "t")
        self.assertEqual(set(params["factions"]["Sparta"]), set(mc.KNOB_ORDER))
        self.assertEqual(params["decisions"]["siege_window"], catalog["decision.siege_window"][0])

    def test_faction_target_writes_decision_genes_only_to_that_override(self):
        params = mc.to_params(["decision.siege_window", "knob.Switzerland.military"], [6, 73], "t", "Sparta")
        self.assertNotIn("decisions", params)
        self.assertEqual(params["faction_overrides"], {"Sparta": {"siege_window": 6}})
        self.assertEqual(params["factions"]["Switzerland"]["military"], 73)
        self.assertNotIn("faction_overrides", mc.to_params(["decision.siege_window"], [6], "t"))

    def test_decode_clips_to_the_gene_range(self):
        catalog = mc.gene_catalog()
        names = ["decision.siege_window", "knob.Sparta.military"]
        self.assertEqual(mc.decode(names, catalog, [-3.0, 7.0]), [0, 100])


if __name__ == "__main__":
    unittest.main()
