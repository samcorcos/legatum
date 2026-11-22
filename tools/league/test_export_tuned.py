"""Exporter input checks: python3 -m unittest tools.league.test_export_tuned."""
import unittest

from tools.league.export_tuned import genome_list


class GenomeFormatTests(unittest.TestCase):
    def setUp(self):
        self.schema = {"openings": {"Rush": {"genes": [
            {"name": "rush_timeout", "min": 30, "max": 200},
            {"name": "builds.0.count", "min": 0, "max": 8},
        ]}}}

    def test_duel_vector_and_league_map_have_the_same_schema_order(self):
        vector = genome_list("Rush", [30, 3], self.schema)
        mapped = genome_list("Rush", {"builds.0.count": 3, "rush_timeout": 30}, self.schema)
        self.assertEqual(vector, [30, 3])
        self.assertEqual(mapped, vector)

    def test_rejects_wrong_vector_length(self):
        for genome in ([30], [30, 3, 1]):
            with self.subTest(genome=genome), self.assertRaisesRegex(SystemExit, "needs 2 integers"):
                genome_list("Rush", genome, self.schema)

    def test_rejects_missing_or_unknown_map_genes(self):
        for genome in ({"rush_timeout": 30}, {"rush_timeout": 30, "unknown": 3}):
            with self.subTest(genome=genome), self.assertRaisesRegex(SystemExit, "does not fit"):
                genome_list("Rush", genome, self.schema)

    def test_rejects_out_of_bounds_and_non_integer_values(self):
        for genome in ([29, 3], [30, 9], [30, True], [30, 3.0]):
            with self.subTest(genome=genome), self.assertRaisesRegex(SystemExit, "outside"):
                genome_list("Rush", genome, self.schema)

    def test_rejects_other_genome_formats(self):
        for genome in ("30,3", 30, None):
            with self.subTest(genome=genome), self.assertRaisesRegex(SystemExit, "vector or a gene map"):
                genome_list("Rush", genome, self.schema)


if __name__ == "__main__":
    unittest.main()
