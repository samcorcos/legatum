#!/usr/bin/env python3
"""Tests for tools/gates/public-claims.py. Run: python3 tools/gates/test_public_claims.py"""
import importlib.util
import os
import pathlib
import unittest

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(os.path.dirname(HERE))
spec = importlib.util.spec_from_file_location("public_claims", os.path.join(HERE, "public-claims.py"))
pc = importlib.util.module_from_spec(spec)
spec.loader.exec_module(pc)

TYPES = """impl Faction {
    pub const ALL: [Faction; 3] = [
        Self::Switzerland,
        Self::Celts,
        Self::Rome,
    ];
}"""
ERAS = """pub const ERA_COUNT: usize = 2;
pub static TECHS: [TechDef; TechId::ALL.len()] = {
    use TechId::*;
    [
        // Ancient
        t(Irrigation, "Irrigation", 0, 24),
        t(Masonry, "Masonry", 0, 30),
        // Classical
        t(Mathematics, "Mathematics", 1, 48),
        t(Engineering, "Engineering", 1, 48),
    ]
};"""
GOALS = """        Faction::Celts => GoalRule::Camps,
        Faction::Rome => GoalRule::None,"""
DATA = 'export const FACTIONS: Faction[] = ["Switzerland", "Celts", "Rome"];'
README = """You research through two eras.
- **Three peoples, two of them finished.** Switzerland and the Celts are done. Rome is early. <!-- peoples-count -->
- **Two eras, 4 technologies.** You finish an era's two technologies to enter the next. <!-- eras-count -->
"""


def run(readme=README, types=TYPES, eras=ERAS, goals=GOALS, data=DATA):
    return pc.findings(readme, types, eras, goals, data)


class PublicClaims(unittest.TestCase):
    def test_a_true_readme_passes(self):
        self.assertEqual(run(), [])

    def test_wrong_people_count(self):
        found = run(README.replace("Three peoples", "Two peoples"))
        self.assertTrue(any("says 2 peoples" in f and "has 3" in f for f in found), found)

    def test_missing_people_name(self):
        found = run(README.replace("Rome is early.", "One more is early."))
        self.assertTrue(any("does not name Rome" in f for f in found), found)

    def test_wrong_finished_count(self):
        found = run(README.replace("two of them finished", "all finished"))
        self.assertTrue(any("2 peoples are finished" in f for f in found), found)

    def test_no_finished_count_needed_when_every_people_has_a_goal(self):
        readme = README.replace("Three peoples, two of them finished.", "Three peoples.")
        self.assertEqual(run(readme, goals="Faction::Celts => GoalRule::Camps,"), [])

    def test_wrong_era_count_anywhere(self):
        found = run(README.replace("through two eras", "through seven eras"))
        self.assertTrue(any('"seven eras"' in f for f in found), found)

    def test_wrong_technology_count(self):
        found = run(README.replace("4 technologies", "35 technologies"))
        self.assertTrue(any("says 35 technologies" in f and "has 4" in f for f in found), found)

    def test_a_removed_marker_is_a_finding(self):
        found = run(README.replace(" <!-- peoples-count -->", "").replace(" <!-- eras-count -->", ""))
        self.assertEqual(len(found), 2, found)

    def test_app_and_core_people_lists_must_agree(self):
        found = run(data='export const FACTIONS: Faction[] = ["Switzerland", "Celts"];')
        self.assertTrue(any("FACTIONS lists" in f for f in found), found)

    def test_unreadable_source_is_a_finding_not_a_pass(self):
        found = run(eras="pub const ERAS: usize = 2;")
        self.assertTrue(any("cannot read ERA_COUNT" in f for f in found), found)

    def test_the_repository_readme_agrees_with_the_code(self):
        texts = [pathlib.Path(ROOT, p).read_text(encoding="utf8") for p in pc.SOURCES]
        self.assertEqual(pc.findings(*texts), [])


if __name__ == "__main__":
    unittest.main()
