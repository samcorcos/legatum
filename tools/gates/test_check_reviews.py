#!/usr/bin/env python3
"""Tests for the reviewer gate in tools/gates/check.py. Run: python3 tools/gates/test_check_reviews.py"""
import json
import os
import sys
import tempfile
import unittest

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


if __name__ == "__main__":
    unittest.main()
