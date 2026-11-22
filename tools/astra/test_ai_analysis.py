import copy
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest import mock

SPEC = importlib.util.spec_from_file_location("ai_analysis", Path(__file__).with_name("ai_analysis.py"))
M = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(M)


class AnalysisTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        (self.root / "crates/ai/src").mkdir(parents=True)
        (self.root / "crates/ai/src/military.rs").write_text("fn military() {}\n")
        self.report = {
            "game_id": "test-game", "schema_version": 1, "detector_version": 2,
            "rules_fingerprint": "abc", "engine_commit": "engine", "complete_history": True,
            "finished": True, "has_winner": True, "map_size": [20, 14],
            "seats": [{"player": 0, "faction": "Switzerland", "difficulty": "Hard", "human": False, "won": True}],
            "turns": [{"turn": 1, "players": [{"player": 0, "outcomes": ["researched Writing"]}]}],
            "incidents": [{"id": "I000001", "turn": 1, "player": 0, "behavior": "unanswered_attack"}],
        }
        self.finding = {"title": "Held under fire", "priority": 1, "confidence": "high", "behavior": "unanswered_attack", "incident_ids": ["I000001"], "code_causes": [{"path": "crates/ai/src/military.rs", "line": 1, "explanation": "Chooses Hold before threat response."}], "rationale": "Two observed turns and confirmed code branch.", "proposal": "Prefer a legal response when attacked repeatedly.", "tests": "Construct a two-turn ranged exposure fixture.", "dedupe_key": "hold-before-threat-response"}
        self.finding["expected_effect"] = "Reduce avoidable damage without abandoning safe city defense."
        self.evidence = [{"game_id": "test-game", "player": 0, "turns": [1]}]
        self.review = {
            "summary": "One finding", "findings": [self.finding], "lead_notes": [],
            "what_worked": [{"behavior": "Continued research", "why_succeeded": "Completed Writing despite the attack.",
                             "confidence": "medium", "evidence": self.evidence}],
            "what_worked_limitations": "",
            "comparison": {"summary": "No matching loss available.", "pairs": [], "limitations": ["Only one verified game."]},
            "proposals": [
                {"kind": kind, "description": description, "evidence": self.evidence,
                 "expected_effect": "Improve decisions while preserving effective research.", "confidence": "medium"}
                for kind, description in [
                    ("tunable_parameter", "Expose defensive reserve turns (1-4); trade spending for readiness."),
                    ("exploit", "Guard repeated ranged kiting with a regression for safe defenders."),
                ]
            ],
        }

    def test_review_validation_rejects_hallucinated_evidence_paths_and_confidence(self):
        self.assertEqual(len(M.validate_review(self.review, self.report, self.root)), 1)
        for field, value in [("incident_ids", ["I-do-not-exist"]), ("confidence", "certain"), ("code_causes", []), ("dedupe_key", "../escape")]:
            bad = copy.deepcopy(self.review)
            bad["findings"][0][field] = value
            with self.assertRaises(ValueError):
                M.validate_review(bad, self.report, self.root)
        for path, line in [("../../../bad.rs", 1), ("crates/ai/src/missing.rs", 1), ("crates/ai/src/military.rs", 22)]:
            bad = copy.deepcopy(self.review)
            bad["findings"][0]["code_causes"][0].update(path=path, line=line)
            with self.assertRaises(ValueError):
                M.validate_review(bad, self.report, self.root)

    def test_queue_deduplicates_across_games_and_running_and_done(self):
        args = (self.finding, "test-game", self.root / "review.md", self.root / "game.json", self.root)
        result = M.queue_proposal(*args)
        self.assertTrue(result.startswith("queued:"))
        pending = next((self.root / ".progress/astra-queue/pending").glob("*.md"))
        self.assertTrue(pending.read_text().startswith("mode: write\nname: ai-fix-"))
        for stage in ("running", "done"):
            moved = self.root / ".progress/astra-queue" / stage / pending.name
            moved.parent.mkdir(parents=True)
            pending.rename(moved)
            pending = moved
            self.assertTrue(M.queue_proposal(*args).startswith("already queued:"))

    def mock_run(self, args, **kwargs):
        response_path = Path(args[args.index("-o") + 1])
        self.assertIn("read-only", args)
        self.assertIn("gpt-6-astra", args)
        self.assertIn("WHAT WORKED", kwargs["input"])
        self.assertIn("tunable_parameter", kwargs["input"])
        response_path.write_text(json.dumps(self.review))
        return subprocess.CompletedProcess(args, 0)

    def fake_detector(self, args, **kwargs):
        if args[0] == "git":
            return "code-revision\n"
        artifacts = self.root / ".reviews/ai-analysis"
        artifacts.mkdir(parents=True, exist_ok=True)
        incidents = artifacts / "test.incidents.json"
        narrative = artifacts / "test.narrative.md"
        incidents.write_text(json.dumps(self.report))
        narrative.write_text("T1: bombardment\n")
        return json.dumps({"game_id": "test-game", "incidents": str(incidents), "narrative": str(narrative)})

    def test_end_to_end_only_high_confidence_queues_and_receipt_skips_model(self):
        tape = self.root / "test.json"
        tape.write_text("{}")
        medium = copy.deepcopy(self.finding)
        medium.update(confidence="medium", dedupe_key="uncertain-hold")
        self.review["findings"].append(medium)
        with mock.patch.object(M.subprocess, "check_output", side_effect=self.fake_detector), mock.patch.object(M.subprocess, "run", side_effect=self.mock_run) as run:
            path = M.review_game(tape, self.root)
            self.assertTrue(path.exists())
            self.assertIn("medium confidence", path.read_text())
            self.assertEqual(len(list((self.root / ".progress/astra-queue/pending").glob("*.md"))), 1)
            proposals = next((self.root / ".reviews/ai-proposals").glob("*.json"))
            bundle = json.loads(proposals.read_text())
            M.ai_review.validate_proposals(bundle, {"test-game": self.report})
            self.assertEqual({p["kind"] for p in bundle["proposals"]}, {"new_behavior", "tunable_parameter", "exploit"})
            self.assertIn("What worked", path.read_text())
            self.assertIn("Winning and losing games", path.read_text())
            self.assertIn(proposals.name, path.read_text())
            M.review_game(tape, self.root)
            self.assertEqual(run.call_count, 1)
            proposals.rename(proposals.with_suffix(".missing"))
            M.review_game(tape, self.root)
            self.assertEqual(run.call_count, 2)

    def test_source_change_during_review_never_queues_stale_findings(self):
        tape = self.root / "test.json"
        tape.write_text("{}")
        def changed(args, **kwargs):
            result = self.mock_run(args, **kwargs)
            (self.root / "crates/ai/src/military.rs").write_text("fn changed() {}\n")
            return result
        with mock.patch.object(M.subprocess, "check_output", side_effect=self.fake_detector), mock.patch.object(M.subprocess, "run", side_effect=changed):
            with self.assertRaisesRegex(ValueError, "sources changed"):
                M.review_game(tape, self.root)
        self.assertFalse((self.root / ".progress/astra-queue/pending").exists())
        self.assertFalse((self.root / ".reviews/ai-analysis-test-game.receipt.json").exists())
        self.assertFalse((self.root / ".reviews/ai-proposals").exists())

    def test_failed_model_leaves_no_receipt_no_tasks_and_retry_works(self):
        tape = self.root / "test.json"
        tape.write_text("{}")
        with mock.patch.object(M.subprocess, "check_output", side_effect=self.fake_detector), mock.patch.object(M.subprocess, "run", side_effect=subprocess.CalledProcessError(1, "codex")):
            with self.assertRaises(subprocess.CalledProcessError):
                M.review_game(tape, self.root)
        self.assertFalse((self.root / ".progress/astra-queue/pending").exists())
        self.assertFalse((self.root / ".reviews/ai-analysis-test-game.receipt.json").exists())
        with mock.patch.object(M.subprocess, "check_output", side_effect=self.fake_detector), mock.patch.object(M.subprocess, "run", side_effect=self.mock_run):
            M.review_game(tape, self.root, no_queue=True)
        self.assertFalse((self.root / ".progress/astra-queue/pending").exists())

    def row(self, game_id, human=False, turn_count=150):
        return {"schema_version": 1, "rules_fingerprint": "abc", "game_id": game_id, "source": "human-local" if human else "sim", "outcome": "vp", "finished_at": "2026-09-27T13:00:00Z", "created_at": "2026-09-27T12:00:00Z", "turn_count": turn_count, "seats": [{"human": human}]}

    def fake_watcher_review(self, archive, root, **kwargs):
        game_id = archive.name.split(".tape")[0]
        path = root / "review.md"
        path.write_text("done")
        proposals = root / ".reviews/ai-proposals" / f"2026-09-28-{game_id}.json"
        M.atomic_json(proposals, json.loads(json.dumps(self.bundle()).replace("test-game", game_id)))
        M.atomic_json(root / ".reviews" / f"ai-analysis-{game_id}.receipt.json", {
            "pipeline_version": M.VERSION, "proposals": str(proposals.relative_to(root)),
            "proposals_sha256": hashlib.sha256(proposals.read_bytes()).hexdigest(),
        })
        return path

    def test_daily_sample_is_20_stable_deduplicated_and_does_not_select_humans(self):
        rows = [self.row(f"g{i}") for i in range(40)] + [self.row("human", True)]
        a = M.daily_sample(rows, "2026-09-27")
        self.assertEqual(len(a), 20)
        self.assertEqual(M.daily_sample(rows, "2026-09-27", fingerprint="incompatible"), [])
        self.assertEqual(M.daily_sample(rows, "2026-09-27", reviewed={r["game_id"] for r in rows}), [])
        self.assertEqual(a, M.daily_sample(list(reversed(rows)), "2026-09-27"))
        self.assertNotIn("human", [r["game_id"] for r in a])
        self.assertNotEqual(a, M.daily_sample(rows, "2026-09-28"))

    def test_watcher_skips_short_human_games_from_automated_tests(self):
        archive = self.root / "archive"
        (archive / "2026/09").mkdir(parents=True)
        rows = [self.row("e2e", True, turn_count=1), self.row("unfinished", True, turn_count=None)]
        for r in rows:
            (archive / f"2026/09/{r['game_id']}.tape.json.gz").write_text("placeholder")
        (archive / "index.jsonl").write_text("".join(json.dumps(r) + "\n" for r in rows))
        with mock.patch.object(M, "review_game") as review:
            self.assertEqual(M.watch_once(archive, self.root, daily=False), 0)
        review.assert_not_called()
        # An earlier failed review of a test game must not be retried either.
        state_path = self.root / ".progress/ai-analysis/watcher.json"
        M.atomic_json(state_path, {"games": {"e2e": {"done": False, "attempts": 1, "retry_after": 0}}})
        with mock.patch.object(M, "review_game") as review, mock.patch.object(M, "daily_sample", return_value=[]), \
                mock.patch.object(M, "engine_fingerprint", return_value="abc"), mock.patch.object(M.subprocess, "run"):
            M.watch_once(archive, self.root, daily=True)
        review.assert_not_called()

    def test_watcher_recovers_finished_humans_skips_checkpoints_and_retries_failures(self):
        archive = self.root / "archive"
        (archive / "2026/09").mkdir(parents=True)
        row = self.row("human", True)
        (archive / "2026/09/human.tape.json.gz").write_text("placeholder")
        (archive / "index.jsonl").write_text(json.dumps(row) + "\n" + '{"unfinished":')
        with mock.patch.object(M, "review_game", side_effect=ValueError("incompatible")):
            self.assertEqual(M.watch_once(archive, self.root, daily=False), 1)
        state_path = self.root / ".progress/ai-analysis/watcher.json"
        state = json.loads(state_path.read_text())
        self.assertFalse(state["games"]["human"]["done"])
        state["games"]["human"]["retry_after"] = 0
        M.atomic_json(state_path, state)
        (self.root / "review.md").write_text("done")
        with mock.patch.object(M, "review_game", side_effect=self.fake_watcher_review) as review:
            self.assertEqual(M.watch_once(archive, self.root, daily=False), 0)
            M.watch_once(archive, self.root, daily=False)
            self.assertEqual(review.call_count, 1)
        self.assertTrue(json.loads(state_path.read_text())["games"]["human"]["done"])

    def test_watcher_skips_short_missing_history_and_unfinished_human_games(self):
        archive = self.root / "archive"
        (archive / "2026/09").mkdir(parents=True)
        rows = [dict(self.row(f"short-{i}", True), turn_count=count)
                for i, count in enumerate([0, 19, None])]
        missing = self.row("missing-history", True)
        missing.pop("turn_count")
        rows.extend([missing, dict(self.row("unfinished", True), finished_at=None)])
        for row in rows:
            (archive / "2026/09" / (row["game_id"] + ".tape.json.gz")).write_text("placeholder")
        (archive / "index.jsonl").write_text("".join(json.dumps(row) + "\n" for row in rows))
        with mock.patch.object(M, "review_game") as review:
            self.assertEqual(M.watch_once(archive, self.root, daily=False), 0)
            review.assert_not_called()

    def test_daily_manifest_freezes_twenty_and_resumes_without_repeating_reviews(self):
        archive = self.root / "archive"
        (archive / "2026/09").mkdir(parents=True)
        rows = [self.row(f"g{i}") for i in range(25)]
        for row in rows:
            (archive / "2026/09" / (row["game_id"] + ".tape.json.gz")).write_text("placeholder")
        (archive / "index.jsonl").write_text("".join(json.dumps(row) + "\n" for row in rows))
        (self.root / "review.md").write_text("done")
        with mock.patch.object(M, "engine_fingerprint", return_value="abc"), mock.patch.object(M, "review_game", side_effect=self.fake_watcher_review) as review, mock.patch.object(M.subprocess, "run") as sim:
            self.assertEqual(M.watch_once(archive, self.root, day="2026-09-27"), 0)
            selected = json.loads((self.root / ".progress/ai-analysis/daily-2026-09-27.json").read_text())["game_ids"]
            self.assertEqual(len(set(selected)), 20)
            M.watch_once(archive, self.root, day="2026-09-27")
            self.assertEqual(review.call_count, 20)
            sim.assert_not_called()
            proposals = self.root / ".reviews/ai-proposals" / f"2026-09-28-{selected[0]}.json"
            proposals.write_text("{}")
            M.watch_once(archive, self.root, day="2026-09-27")
            self.assertEqual(review.call_count, 21, "a damaged proposals artifact must be retried")

    def test_failed_daily_topup_does_not_prevent_finished_human_review(self):
        archive = self.root / "archive"
        (archive / "2026/09").mkdir(parents=True)
        (archive / "index.jsonl").write_text(json.dumps(self.row("human", True)) + "\n")
        (archive / "2026/09/human.tape.json.gz").write_text("placeholder")
        (self.root / "review.md").write_text("done")
        with mock.patch.object(M, "engine_fingerprint", return_value="abc"), mock.patch.object(M.subprocess, "run", side_effect=subprocess.CalledProcessError(1, "sim")), mock.patch.object(M, "review_game", return_value=self.root / "review.md") as review:
            self.assertEqual(M.watch_once(archive, self.root), 1)
            self.assertEqual(review.call_count, 1)

    def test_compaction_represents_rare_behaviors_and_discloses_omissions(self):
        report = copy.deepcopy(self.report)
        report["incidents"] = [{"id": f"I{i}", "player": 0, "behavior": "production_idle"} for i in range(1000)]
        report["incidents"].append({"id": "rare", "player": 1, "behavior": "scout_not_fleeing"})
        compact = M.compact_evidence(report, byte_limit=1000)
        self.assertIn("rare", [i["id"] for i in compact["examples"]])
        self.assertGreater(compact["omitted_incidents"], 0)

    def bundle(self):
        return M.ai_review.proposals_document(
            self.review, [self.finding], self.report, {"test-game": self.report},
            "reviewed-revision", "a" * 64, "2026-09-28T12:00:00+00:00",
        )

    def loss(self):
        loss = copy.deepcopy(self.report)
        loss["game_id"] = "lost-game"
        loss["seats"][0]["won"] = False
        return loss

    def with_comparison(self):
        loss = self.loss()
        reports = {"test-game": self.report, "lost-game": loss}
        self.review["comparison"]["pairs"] = [{
            **M.ai_review.available_pairs(self.report, reports)[0],
            "explanation": "Research continued in the win but stalled during the loss.",
            "evidence": self.evidence + [{"game_id": "lost-game", "player": 0, "turns": [1]}],
        }]
        return reports

    def test_proposals_schema_rejects_invalid_types_enums_missing_and_extra_fields(self):
        bundle = self.bundle()
        M.ai_review.validate_shape(bundle, M.ai_review.PROPOSALS_SCHEMA)
        mutations = [
            lambda b: b.update(schema_version=True),
            lambda b: b.update(schema_version=2),
            lambda b: b.update(unexpected="extra"),
            lambda b: b.update(proposals={}),
            lambda b: b["proposals"][0].pop("expected_effect"),
            lambda b: b["proposals"][0].update(kind="bug"),
            lambda b: b["proposals"][0].update(confidence="certain"),
            lambda b: b["proposals"][0].update(description="  "),
            lambda b: b["proposals"][0].update(evidence=[]),
            lambda b: b["proposals"][0]["evidence"][0].update(player=True),
            lambda b: b["proposals"][0]["evidence"][0].update(turns=[0]),
            lambda b: b["proposals"][0]["evidence"][0].update(turns=[1.0]),
            lambda b: b["proposals"][0]["evidence"][0].update(turns=[1, 1]),
            lambda b: b["proposals"][0]["evidence"][0].update(turns="1"),
        ]
        for mutate in mutations:
            bad = copy.deepcopy(bundle)
            mutate(bad)
            with self.subTest(document=bad), self.assertRaises(ValueError):
                M.ai_review.validate_proposals(bad, {"test-game": self.report})

    def test_proposal_and_success_evidence_must_exist_in_verified_ai_turns(self):
        for field, value in [("game_id", "invented"), ("player", 1), ("turns", [99])]:
            for section in ("what_worked", "proposals"):
                bad = copy.deepcopy(self.review)
                bad[section][0]["evidence"] = [{"game_id": "test-game", "player": 0, "turns": [1], field: value}]
                with self.subTest(section=section, field=field), self.assertRaises(ValueError):
                    M.validate_review(bad, self.report, self.root)
        bad = self.bundle()
        bad["games"][0]["engine_commit"] = "today"
        with self.assertRaisesRegex(ValueError, "provenance"):
            M.ai_review.validate_proposals(bad, {"test-game": self.report})
        human = copy.deepcopy(self.report)
        human["seats"][0]["human"] = True
        with self.assertRaisesRegex(ValueError, "AI seat"):
            M.ai_review.validate_proposals(self.bundle(), {"test-game": human})

    def test_comparison_requires_real_opposite_outcomes_of_matched_version_and_settings(self):
        reports = self.with_comparison()
        M.validate_review(self.review, self.report, self.root, reports)
        for field, value in [
            ("engine_commit", "different"), ("rules_fingerprint", "different"),
            ("map_size", [30, 20]), ("complete_history", False),
            ("finished", False), ("has_winner", False), ("engine_commit", "unknown"),
        ]:
            bad = copy.deepcopy(reports)
            bad["lost-game"][field] = value
            with self.subTest(field=field), self.assertRaises(ValueError):
                M.validate_review(self.review, self.report, self.root, bad)
        for field, value in [("won", True), ("human", True), ("faction", "Celts"), ("difficulty", "Easy")]:
            bad = copy.deepcopy(reports)
            bad["lost-game"]["seats"][0][field] = value
            with self.subTest(field=field), self.assertRaises(ValueError):
                M.validate_review(self.review, self.report, self.root, bad)
        bad = copy.deepcopy(self.review)
        bad["comparison"]["pairs"] = []
        with self.assertRaisesRegex(ValueError, "must discuss"):
            M.validate_review(bad, self.report, self.root, reports)
        bad = copy.deepcopy(self.review)
        bad["comparison"]["pairs"][0]["evidence"] = self.evidence
        with self.assertRaisesRegex(ValueError, "both seats"):
            M.validate_review(bad, self.report, self.root, reports)

    def test_incident_free_game_still_reviews_successes_and_publishes_no_queue_proposals(self):
        self.report["incidents"] = []
        self.review["findings"] = []
        tape = self.root / "test.json"
        tape.write_text("{}")
        with mock.patch.object(M.subprocess, "check_output", side_effect=self.fake_detector), mock.patch.object(M.subprocess, "run", side_effect=self.mock_run):
            report = M.review_game(tape, self.root, no_queue=True)
        self.assertIn("Continued research", report.read_text())
        self.assertEqual(len(list((self.root / ".reviews/ai-proposals").glob("*.json"))), 1)
        self.assertFalse((self.root / ".progress/astra-queue/pending").exists())
        self.review["what_worked"] = []
        with self.assertRaisesRegex(ValueError, "evidence limitation"):
            M.validate_review(self.review, self.report, self.root)
        self.review["what_worked_limitations"] = "The excerpt contains no attributable successful behavior."
        M.validate_review(self.review, self.report, self.root)

    def test_invalid_proposals_publish_nothing_and_never_queue(self):
        tape = self.root / "test.json"
        tape.write_text("{}")
        self.review["proposals"][0]["evidence"] = [{"game_id": "invented", "player": 0, "turns": [1]}]
        with mock.patch.object(M.subprocess, "check_output", side_effect=self.fake_detector), mock.patch.object(M.subprocess, "run", side_effect=self.mock_run):
            with self.assertRaises(ValueError):
                M.review_game(tape, self.root)
        self.assertFalse((self.root / ".reviews/ai-proposals").exists())
        self.assertFalse((self.root / ".reviews/ai-analysis-test-game.md").exists())
        self.assertFalse((self.root / ".reviews/ai-analysis-test-game.receipt.json").exists())
        self.assertFalse((self.root / ".progress/astra-queue/pending").exists())

    def test_explicit_comparison_replays_and_cites_both_games(self):
        self.with_comparison()
        tape, other = self.root / "test.json", self.root / "other.json"
        tape.write_text("{}")
        other.write_text("{}")
        original = copy.deepcopy(self.report)
        def detector(args, **kwargs):
            self.report = self.loss() if str(other.resolve()) in args else original
            try:
                result = json.loads(self.fake_detector(args, **kwargs)) if args[0] != "git" else None
                if result is None:
                    return "code-revision\n"
                result["game_id"] = self.report["game_id"]
                return json.dumps(result)
            finally:
                self.report = original
        with mock.patch.object(M.subprocess, "check_output", side_effect=detector), mock.patch.object(M.subprocess, "run", side_effect=self.mock_run):
            report = M.review_game(tape, self.root, no_queue=True, compare=[other])
        self.assertIn("lost-game P0", report.read_text())
        proposals = json.loads(next((self.root / ".reviews/ai-proposals").glob("*.json")).read_text())
        self.assertEqual({g["game_id"] for g in proposals["games"]}, {"test-game", "lost-game"})

    def test_compact_success_evidence_spans_whole_game_and_discloses_omissions(self):
        report = copy.deepcopy(self.report)
        report["turns"] = [
            {"turn": i, "players": [{"player": 0, "outcomes": ["researched Writing"]}]}
            for i in range(1, 101)
        ]
        compact = M.ai_review.compact_game(report, byte_limit=700)
        included = {t["turn"] for t in compact["turns"]}
        self.assertTrue({1, 50, 100} <= included)
        self.assertEqual(compact["omitted_turns"], 100 - len(included))

    def test_comparison_discovers_verified_cache_and_invalidates_single_game_receipt(self):
        tape = self.root / "test.json"
        tape.write_text("{}")
        with mock.patch.object(M.subprocess, "check_output", side_effect=self.fake_detector), mock.patch.object(M.subprocess, "run", side_effect=self.mock_run) as run:
            M.review_game(tape, self.root, no_queue=True)
            reports = self.with_comparison()
            M.atomic_json(self.root / ".reviews/ai-analysis/peer/lost-game.incidents.json", reports["lost-game"])
            result = M.review_game(tape, self.root, no_queue=True)
            self.assertIn("lost-game P0", result.read_text())
            self.assertEqual(run.call_count, 2)
            M.review_game(tape, self.root, no_queue=True)
            self.assertEqual(run.call_count, 2)

    def test_index_shortlist_replays_peer_before_first_review_and_excludes_bad_replay(self):
        archive_root = self.root / "archive"
        archive = archive_root / "2026/09/test-game.tape.json"
        archive.parent.mkdir(parents=True)
        archive.write_text("{}")
        other = archive.with_name("lost-game.tape.json")
        other.write_text("{}")
        loss = self.loss()
        row = {
            **self.row("lost-game"), "engine_commit": "engine", "winner": 1,
            "complete_history": True, "map": {"width": 20, "height": 14},
            "seats": [{**loss["seats"][0], "team": 0}],
        }
        (archive_root / "index.jsonl").write_text(json.dumps(row) + "\n")
        with mock.patch.object(M, "detect", return_value=({"incidents": "verified-peer.json"}, loss)) as detect:
            reports, paths, _ = M.comparison_reports(self.report, archive, self.root, [])
            self.assertEqual(set(reports), {"test-game", "lost-game"})
            self.assertEqual(paths["lost-game"], "verified-peer.json")
            self.assertEqual(detect.call_count, 1)
        with mock.patch.object(M, "detect", side_effect=ValueError("hash diverged")):
            reports, _, notes = M.comparison_reports(self.report, archive, self.root, [])
            self.assertEqual(set(reports), {"test-game"})
            self.assertTrue(any("hash diverged" in n for n in notes))


if __name__ == "__main__":
    unittest.main()
