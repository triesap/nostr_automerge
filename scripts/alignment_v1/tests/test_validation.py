"""Fail-closed routing regressions, with no Cargo invocation or campaigns."""
import copy
import importlib.util
import os
from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest.mock import patch

MODULE = Path(__file__).resolve().parents[1] / "validation.py"
SPEC = importlib.util.spec_from_file_location("alignment_validation", MODULE)
validation = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(validation)
ROOT = MODULE.parents[2]


class RegistryTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.registry = validation.expected_registry(ROOT)

    def test_exact_original_obligations(self):
        value = self.registry
        self.assertEqual(len(value["declarations"]), 378)
        self.assertEqual(len(value["selected_python_paths"]), 57)
        self.assertEqual(len(validation.validate_registry(ROOT, value)), 55)
        self.assertEqual(len(value["local_jobs"]), 9)

    def test_omitted_current_leaf_and_builtin_rejected(self):
        for field in ("current_python_paths", "current_builtin_obligations", "declarations"):
            value = copy.deepcopy(self.registry)
            value[field].pop()
            with self.subTest(field=field), self.assertRaises(ValueError):
                validation.validate_registry(ROOT, value)

    def test_route_command_anchor_and_duplicates_rejected(self):
        variants = []
        for field, replacement in (("route", "historical"), ("argv", ["python3", "scripts/validate_spec.py", "--write"]), ("id", "unknown")):
            value = copy.deepcopy(self.registry)
            value["declarations"][0][field] = replacement
            variants.append(value)
        value = copy.deepcopy(self.registry)
        value["historical_validation_anchor"] = "0" * 40
        variants.append(value)
        value = copy.deepcopy(self.registry)
        value["declarations"].append(value["declarations"][0])
        variants.append(value)
        for value in variants:
            with self.assertRaises(ValueError):
                validation.validate_registry(ROOT, value)

    def test_authority_candidate_bindings_match_real_plan(self):
        validation.validate_candidate_bindings(ROOT)

    def test_mismatched_baseline_and_reviewed_candidates_rejected(self):
        authority = validation.load(ROOT / "spec/nostr_automerge_alignment_refactor_v1/authority.json")
        plan = validation.load(ROOT / "implementation/nostr_automerge_alignment_refactor_v1/steps.json")
        for field in ("baseline_commit", "reviewed_commit"):
            changed = copy.deepcopy(authority)
            changed[field] = "0" * 40
            with self.subTest(field=field), patch.object(validation, "load", side_effect=[changed, plan]), self.assertRaises(ValueError):
                validation.validate_candidate_bindings(ROOT)

    def test_missing_old_commit_fails_closed(self):
        with patch.object(validation, "ANCHOR", "0" * 40):
            with self.assertRaises(subprocess.CalledProcessError):
                validation.expected_registry(ROOT)

    def test_frozen_tampering_or_missing_artifact_rejected(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            (root / "reports").mkdir()
            path = root / "reports/frozen.json"
            def fake_git(_root, *args):
                if args[0] == "ls-tree":
                    return b"100644 blob abc\treports/frozen.json\0"
                return b"original"
            with patch.object(validation, "git", fake_git), patch.object(validation, "committed", return_value=b"original"):
                for content in (None, b"tampered"):
                    if content is not None:
                        path.write_bytes(content)
                    with self.assertRaises(ValueError):
                        validation.frozen_artifacts(root)
                path.write_bytes(b"original")
                validation.frozen_artifacts(root)


class CandidateTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name).resolve()
        subprocess.run(["git", "init", "--quiet", str(self.root)], check=True)
        (self.root / ".gitignore").write_text(".local/\n")
        (self.root / "source.rs").write_text("source")
        subprocess.run(["git", "add", "."], cwd=self.root, check=True)
        subprocess.run(["git", "-c", "user.name=Validation Test", "-c", "user.email=validation@example.invalid", "-c", "commit.gpgsign=false", "commit", "--quiet", "-m", "fixture"], cwd=self.root, check=True)

    def test_modified_untracked_missing_or_stale_head_inputs_fail(self):
        observation = validation.input_fingerprint(self.root)
        validation.validate_observation(self.root, observation)
        source = self.root / "source.rs"
        for change in (lambda: source.write_text("changed"), lambda: (self.root / "new.txt").write_text("new"), lambda: source.unlink()):
            source.write_text("source")
            (self.root / "new.txt").unlink(missing_ok=True)
            change()
            with self.assertRaises(ValueError):
                validation.validate_observation(self.root, observation)
        source.write_text("source")
        (self.root / "new.txt").unlink(missing_ok=True)
        observation["head"] = "0" * 40
        with self.assertRaises(ValueError):
            validation.validate_observation(self.root, observation)

    def test_ignored_evidence_does_not_change_current_input_digest(self):
        before = validation.input_fingerprint(self.root)
        evidence = self.root / ".local/evidence"
        evidence.mkdir(parents=True)
        (evidence / "observation.json").write_text("ignored")
        validation.validate_observation(self.root, before)

    def test_output_default_and_external_allowed(self):
        self.assertEqual(validation.output_root(self.root, None), self.root / ".local/evidence")
        external = self.root.parent / (self.root.name + "_output")
        self.assertEqual(validation.output_root(self.root, str(external)), external)

    def test_output_source_ancestor_traversal_symlink_rejected(self):
        alias = self.root / "alias"
        alias.symlink_to(self.root.parent, target_is_directory=True)
        for output in (self.root, self.root.parent, self.root / "source.rs", self.root / "target", self.root / "../escape", alias / "evidence"):
            with self.subTest(output=output), self.assertRaises(ValueError):
                validation.output_root(self.root, str(output))

    def test_historical_temporary_directory_inside_source_rejected_before_add(self):
        from contextlib import nullcontext
        temporary = self.root / ".local/history"
        temporary.mkdir(parents=True)
        with patch.object(validation.tempfile, "TemporaryDirectory", return_value=nullcontext(str(temporary))), patch.object(validation.subprocess, "run") as run:
            with self.assertRaisesRegex(ValueError, "physically outside source"):
                with validation.historical_checkout(self.root):
                    self.fail("unsafe historical checkout was accepted")
            run.assert_not_called()

    def test_historical_temporary_alias_into_source_rejected_before_add(self):
        from contextlib import nullcontext
        with tempfile.TemporaryDirectory() as external:
            alias = Path(external) / "alias"
            alias.symlink_to(self.root, target_is_directory=True)
            with patch.object(validation.tempfile, "TemporaryDirectory", return_value=nullcontext(str(alias))), patch.object(validation.subprocess, "run") as run:
                with self.assertRaisesRegex(ValueError, "physically outside source"):
                    with validation.historical_checkout(self.root):
                        self.fail("aliased source checkout was accepted")
                run.assert_not_called()

    def test_historical_failure_cleans_up_registered_worktree(self):
        head = validation.git(self.root, "rev-parse", "HEAD").decode().strip()
        before = validation.git(self.root, "worktree", "list", "--porcelain")
        with patch.object(validation, "ANCHOR", head):
            with self.assertRaisesRegex(RuntimeError, "forced validator failure"):
                with validation.historical_checkout(self.root) as historical:
                    self.assertEqual(validation.git(historical, "rev-parse", "HEAD").decode().strip(), head)
                    raise RuntimeError("forced validator failure")
        self.assertEqual(validation.git(self.root, "worktree", "list", "--porcelain"), before)

    def test_dirty_historical_checkout_is_rejected_and_cleaned(self):
        head = validation.git(self.root, "rev-parse", "HEAD").decode().strip()
        before = validation.git(self.root, "worktree", "list", "--porcelain")
        with patch.object(validation, "ANCHOR", head):
            with self.assertRaises(ValueError):
                with validation.historical_checkout(self.root) as historical:
                    (historical / "source.rs").write_text("tampered")
        self.assertEqual(validation.git(self.root, "worktree", "list", "--porcelain"), before)


if __name__ == "__main__":
    unittest.main()
