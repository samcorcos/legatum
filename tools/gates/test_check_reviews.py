#!/usr/bin/env python3
"""Tests for the reviewer gate in tools/gates/check.py. Run: python3 tools/gates/test_check_reviews.py"""
import json
import os
import sys
import tempfile
import unittest
from unittest import mock

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import check  # noqa: E402

TIP = "8d5a6c66cc8e21c322407fcafe02c4f6139f6534"
KINDS = {"code": ["crates/core/src/x.rs"], "copy": ["app/src/data.ts"], "ui": ["app/src/ui/A.tsx"],
         "engine": [], "art": [], "scenario": []}


class Reviews(unittest.TestCase):
    def test_required_reviewers_follow_the_changed_paths(self):
        self.assertEqual([r for _, r, _ in check.required_reviews(KINDS)],
                         ["legatum-docs-reviewer", "legatum-plain-language-reviewer", "legatum-ui-reviewer"])
        docs_only = dict(KINDS, code=[], copy=[], ui=[])
        self.assertEqual(check.required_reviews(docs_only), [])

    def test_art_and_art_related_code_need_the_art_reviewer(self):
        art = dict(KINDS, code=[], copy=[], ui=[], art=["app/src/assets/units/tank.png"])
        self.assertEqual([r for _, r, _ in check.required_reviews(art)], ["legatum-art-reviewer"])
        impact = dict(KINDS, code=["crates/core/src/data.rs"], copy=[], ui=[], artimpact=["crates/core/src/data.rs"])
        self.assertEqual([r for _, r, _ in check.required_reviews(impact)],
                         ["legatum-docs-reviewer", "legatum-art-reviewer"])
        files = ["app/src/unitArt.ts", "app/public/terrain/grass.png", "app/src/unitArt.test.ts"]
        kinds = check.classify(".", "HEAD", "HEAD", files)
        self.assertEqual(kinds["artimpact"], ["app/src/unitArt.ts"])
        self.assertEqual(kinds["art"], ["app/public/terrain/grass.png"])

    def test_position_only_branch_uses_fast_scenario_gate(self):
        files = ["scenarios/lab/probe.json", "docs/sim-results/scenarios/probe-baseline.md"]
        kinds = check.classify(".", "HEAD", "HEAD", files)
        self.assertEqual(kinds["scenario"], ["scenarios/lab/probe.json"])
        self.assertEqual(kinds["code"], [])
        self.assertEqual(kinds["copy"], [])

    def test_controls_and_rules_the_advisor_teaches_need_the_advisor_reviewer(self):
        files = ["app/src/map/MapCanvas.tsx", "app/src/ui/Advisor.tsx", "crates/core/src/rules/cities.rs",
                 "app/src/ui/advisor.test.ts", "app/src/ui/DiplomacyPanel.tsx", "crates/ai/src/scripted.rs"]
        kinds = check.classify(".", "HEAD", "HEAD", files)
        self.assertEqual(kinds["advisor"], ["app/src/map/MapCanvas.tsx", "app/src/ui/Advisor.tsx",
                                            "crates/core/src/rules/cities.rs"])
        advisor = dict(KINDS, code=["app/src/ui/Advisor.tsx"], copy=[], ui=[], advisor=["app/src/ui/Advisor.tsx"])
        self.assertEqual([r for _, r, _ in check.required_reviews(advisor)],
                         ["legatum-docs-reviewer", "legatum-advisor-reviewer"])

    def test_the_front_door_needs_the_docs_and_art_reviewers(self):
        kinds = check.classify(".", "HEAD", "HEAD", ["README.md", "docs/media/banner.jpg", "docs/learnings/2026-11.md"])
        self.assertEqual(kinds["code"], [])
        self.assertEqual(kinds["front"], ["README.md", "docs/media/banner.jpg"])
        self.assertEqual([r for _, r, _ in check.required_reviews(kinds)],
                         ["legatum-docs-reviewer", "legatum-art-reviewer"])
        docs = check.classify(".", "HEAD", "HEAD", ["docs/learnings/2026-11.md"])
        self.assertEqual(check.required_reviews(docs), [])

    def test_a_comment_must_name_the_reviewer_and_the_head_sha(self):
        reviews = check.required_reviews(KINDS)
        comments = ["legatum-docs-reviewer: PASS at 8d5a6c66c",
                    "legatum-plain-language-reviewer: pass at 1234567",  # an older head
                    "legatum-ui-reviewer verdict for 8d5a6c6: BLOCKER, the panel overflows"]
        status = {k: ok for k, ok, _ in check.comment_status(reviews, comments, TIP)}
        self.assertEqual(status, {"docs": True, "plain": False, "ui": False})

    def test_the_last_matching_comment_decides(self):
        reviews = [("ui", "legatum-ui-reviewer", "UI changed")]
        comments = ["legatum-ui-reviewer: FAIL at 8d5a6c66c", "legatum-ui-reviewer: ship at 8d5a6c66c"]
        self.assertTrue(check.comment_status(reviews, comments, TIP)[0][1])

    def test_comment_files_in_json_and_plain_text(self):
        with tempfile.TemporaryDirectory() as d:
            gh = os.path.join(d, "gh.json")
            with open(gh, "w") as f:
                json.dump({"comments": [{"body": "a"}, {"body": "b"}]}, f)
            self.assertEqual(check.load_comments(gh), ["a", "b"])
            api = os.path.join(d, "api.json")
            with open(api, "w") as f:
                json.dump([{"body": "c"}, "d"], f)
            self.assertEqual(check.load_comments(api), ["c", "d"])
            txt = os.path.join(d, "c.txt")
            with open(txt, "w") as f:
                f.write("one\n\ntwo\n")
            self.assertEqual(check.load_comments(txt), ["one", "two"])

    def test_rule_and_mapgen_changes_need_the_design_reviewer(self):
        files = ["crates/core/src/rules/cities.rs", "crates/core/src/resources/placement.rs", "crates/core/src/lib.rs",
                 "crates/core/src/rules/cities_tests.rs", "crates/ai/src/production.rs", "app/src/ui/A.tsx"]
        kinds = check.classify(".", "HEAD", "HEAD", files)
        self.assertEqual(kinds["design"], ["crates/core/src/rules/cities.rs", "crates/core/src/resources/placement.rs",
                                           "crates/core/src/lib.rs"])
        design = dict(KINDS, code=["crates/core/src/rules/cities.rs"], copy=[], ui=[],
                      design=["crates/core/src/rules/cities.rs"])
        self.assertIn("legatum-design-reviewer", [r for _, r, _ in check.required_reviews(design)])

    def test_prose_stops_at_closing_quotes(self):
        # Object literals and short quoted identifiers must not match across a closing quote.
        non_prose = [
            "kind: 'Mountains', sprite, scale, tile: i",
            "const a = 'Rome', b = 'Athens', c = 'Sparta', d = 'Carthage', e = 'Egypt';",
            "name: \"Stone\", count: 1, tile: i, next: j",
            "label: `Iron`, x: 1, y: 2, z: 3, w: 4",
            "// Select a tile to found your new city here.",
        ]
        for line in non_prose:
            self.assertFalse(check.adds_prose(({1: line}, {1})), f"unexpected PROSE match: {line}")

        # Real 5+ word sentences inside quotes or JSX text must still match.
        real_prose = [
            "const msg = 'Select a tile to found your city.';",
            "const msg = 'You don\\'t have enough gold to build this.';",
            'const msg = "Move your settler to a fertile river valley.";',
            'const msg = `Your city of ${name} has grown to size ${size}.`;',
            "<p>Click any tile to inspect its yield.</p>",
        ]
        for line in real_prose:
            self.assertTrue(check.adds_prose(({1: line}, {1})), f"expected PROSE match: {line}")

    def test_readme_prose_edit_does_not_require_art_reviewer(self):
        prose_lines = [
            "Legatum is a historical 4X strategy game.",
            '<p align="center"><b><a href="https://playlegatum.com">Play free in your browser</a></b></p>',
            "There is no stable release yet; see [Status](#status).",
        ]
        with mock.patch.object(check, "diff_changed_lines", return_value=prose_lines):
            kinds = check.classify(".", "HEAD", "HEAD", ["README.md"])
        self.assertEqual(kinds["front"], ["README.md"])
        self.assertEqual(kinds["frontmedia"], [])
        self.assertEqual([r for _, r, _ in check.required_reviews(kinds)], ["legatum-docs-reviewer"])

    def test_readme_media_ref_or_front_media_requires_art_reviewer(self):
        for line in [
            "![Banner](docs/media/banner.jpg)",
            '<img src="app/public/og.jpg" alt="Legatum" />',
            "Watch the clip at docs/media/intro.mp4",
            '  <a href="https://playlegatum.com/media/legatum-teaser-v2.mp4">',
            '  <a href="https://example.com/other-clip.webm">',
            "[banner]: https://example.com/banner-v2.png",
            "[banner]: https://cdn.example.com/hero",
        ]:
            with mock.patch.object(check, "diff_changed_lines", return_value=[line]):
                kinds = check.classify(".", "HEAD", "HEAD", ["README.md"])
            self.assertEqual(kinds["frontmedia"], ["README.md"], f"expected frontmedia for: {line}")
            self.assertEqual([r for _, r, _ in check.required_reviews(kinds)],
                             ["legatum-docs-reviewer", "legatum-art-reviewer"])

        # Removed media references (including the real README teaser link) must also trigger art review.
        removed_diff = (
            "diff --git a/README.md b/README.md\n"
            "--- a/README.md\n"
            "+++ b/README.md\n"
            "@@ -15 +14,0 @@\n"
            '-  <a href="https://playlegatum.com/media/legatum-teaser.mp4">\n'
        )
        with mock.patch.object(check, "git", return_value=removed_diff):
            kinds = check.classify(".", "HEAD", "HEAD", ["README.md"])
        self.assertEqual(kinds["frontmedia"], ["README.md"])
        self.assertEqual([r for _, r, _ in check.required_reviews(kinds)],
                         ["legatum-docs-reviewer", "legatum-art-reviewer"])

        # Unreadable diff fails closed to art review.
        with mock.patch.object(check, "diff_changed_lines", return_value=None):
            kinds = check.classify(".", "HEAD", "HEAD", ["README.md"])
        self.assertEqual(kinds["frontmedia"], ["README.md"])

        og = check.classify(".", "HEAD", "HEAD", ["app/public/og.jpg"])
        self.assertEqual(og["frontmedia"], ["app/public/og.jpg"])
        self.assertEqual([r for _, r, _ in check.required_reviews(og)],
                         ["legatum-docs-reviewer", "legatum-ui-reviewer", "legatum-art-reviewer"])

    def test_ci_art_distinct_selector_covers_runtime_and_workflow_inputs(self):
        import re
        ci_path = os.path.join(HERE, "..", "..", ".github", "workflows", "ci.yml")
        with open(ci_path, encoding="utf-8") as f:
            text = f.read()
        m = re.search(r"grep -qE '([^']+)'[^\n]*\n\s+echo \"art=true\"", text)
        self.assertIsNotNone(m, "art-distinct selector not found in ci.yml")
        pat = re.compile(m.group(1))
        for path in [
            "app/src/assets/units/tank.png",
            "app/public/wonders/pyramids.png",
            "tools/art/distinctness.py",
            "app/src/artDistinct.test.ts",
            "app/package.json",
            "app/package-lock.json",
            "app/vite.config.ts",
            "app/vitest.config.ts",
            "app/src/test/setupWasm.ts",
            ".github/workflows/ci.yml",
        ]:
            self.assertIsNotNone(pat.search(path), f"expected art-distinct to run for {path}")
        for path in ["crates/core/src/lib.rs", "README.md", "app/src/ui/TopBar.tsx"]:
            self.assertIsNone(pat.search(path), f"unexpected art-distinct match for {path}")


if __name__ == "__main__":
    unittest.main()
