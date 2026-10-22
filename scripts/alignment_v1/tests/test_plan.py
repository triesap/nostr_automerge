"""Executable negative controls for alignment evidence and ordered completion."""

import copy
import importlib.util
import json
from pathlib import Path
import subprocess
import tempfile
import unittest

MODULE = Path(__file__).resolve().parents[1] / "validate_plan.py"
SPECIFICATION = importlib.util.spec_from_file_location("validate_plan", MODULE)
plan_validator = importlib.util.module_from_spec(SPECIFICATION)
SPECIFICATION.loader.exec_module(plan_validator)
REPOSITORY = MODULE.parents[2]


class PlanValidationTests(unittest.TestCase):
    def setUp(self):
        self.directory = tempfile.TemporaryDirectory(prefix="alignment-plan-")
        self.addCleanup(self.directory.cleanup)
        self.root = Path(self.directory.name)
        self.authority = json.loads((REPOSITORY / plan_validator.SPEC / "authority.json").read_text())
        self.ledger = json.loads((REPOSITORY / plan_validator.IMPL / "ledger.json").read_text())
        # The fixture models the initial state even after the real ledger advances.
        self.ledger["active_step"] = self.ledger["next_step"] = "NA-001"
        self.ledger["bootstrap"]["source_commit"] = None
        self.ledger["bootstrap"]["receipt_commit"] = None
        for step in self.ledger["steps"]:
            step["status"] = "active" if step["id"] == "NA-001" else "planned"
            step["report"] = None
        self.plan = json.loads((REPOSITORY / self.authority["steps"]).read_text())
        for key in ("governing_plan", "validation_registry", "bootstrap_deviation"):
            self.write(self.authority[key], "fixture evidence\n")
        for key in ("steps", "requirements"):
            self.write(self.authority[key], (REPOSITORY / self.authority[key]).read_text())
        self.write("spec/remediation_v19_authority.json", json.dumps({"holds": plan_validator.HOLDS}))
        self.save()
        self.git("init", "--quiet")
        self.anchor = self.commit()
        self.authority["reviewed_commit"] = self.authority["baseline_commit"] = self.anchor
        self.save()
        self.source = self.commit()

    def git(self, *arguments):
        result = subprocess.run(["git", "-C", str(self.root), *arguments], text=True,
                                capture_output=True, check=True)
        return result.stdout.strip()

    def commit(self):
        self.git("add", ".")
        self.git("-c", "user.name=Alignment Fixture", "-c", "user.email=fixture@example.invalid",
                 "-c", "commit.gpgsign=false", "commit", "--quiet", "-m", "fixture")
        return self.git("rev-parse", "HEAD")

    def write(self, path, content):
        target = self.root / path
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_text(content)

    def save(self):
        self.write(f"{plan_validator.SPEC}/authority.json", json.dumps(self.authority))
        self.write(self.authority["ledger"], json.dumps(self.ledger))

    def errors(self):
        self.save()
        return plan_validator.validate_plan(self.root)

    def assert_rejected(self, message):
        errors = self.errors()
        self.assertTrue(any(message in error for error in errors), errors)

    def complete_bootstrap(self):
        for row, original in zip(self.ledger["steps"][:5], self.plan["steps"][:5]):
            row["status"] = "complete"
            row["report"] = {
                "source_commit": self.source, "observation_base": self.source,
                "files": [self.authority["ledger"]],
                "commands": [{"command": command, "kind": "executed", "status": "passed",
                              "exit_code": 0, "result": "synthetic validator fixture only",
                              "counts": {"tests": 1, "scenarios": 1},
                              "evidence": "synthetic command evidence", "toolchain": "fixture"}
                             for command in original["commands"]],
                "acceptance": [{"criterion": criterion, "status": "passed", "evidence": "synthetic evidence"}
                               for criterion in original["tests"]],
                "self_review": {"status": "passed", "evidence": "synthetic fixture self-review"},
                "review": {"status": "passed", "evidence": "synthetic fixture independent review", "kind": "fresh_adversarial"},
                "deviations": [self.authority["bootstrap_deviation"]], "unverified": [],
                "next_step_safety": "safe"}
        self.ledger["steps"][5]["status"] = "active"
        self.ledger["active_step"] = self.ledger["next_step"] = "NA-006"
        self.ledger["bootstrap"]["source_commit"] = self.source

    def test_initial_ledger(self):
        self.assertEqual([], self.errors())

    def test_valid_following_observation(self):
        self.complete_bootstrap()
        self.assertEqual([], self.errors())

    def test_duplicate_identity(self):
        self.ledger["steps"][1]["id"] = "NA-001"
        self.assert_rejected("original id changed")

    def test_duplicate_json_status_key(self):
        self.save()
        target = self.root / self.authority["ledger"]
        target.write_text(target.read_text().replace('"status": "active"', '"status": "planned", "status": "active"', 1))
        errors = plan_validator.validate_plan(self.root)
        self.assertTrue(any("duplicate JSON key: status" in error for error in errors), errors)

    def test_duplicate_ledger_top_level_key(self):
        self.save()
        target = self.root / self.authority["ledger"]
        target.write_text(target.read_text().replace('{', '{"holds": [],', 1))
        errors = plan_validator.validate_plan(self.root)
        self.assertTrue(any("duplicate JSON key: holds" in error for error in errors), errors)

    def test_simultaneous_plan_ledger_mutation_cannot_drop_obligations(self):
        self.plan["steps"][0]["commands"] = []
        self.plan["steps"][0]["tests"] = []
        self.plan["steps"][0]["requirements"] = []
        self.ledger["steps"][0]["requirements"] = []
        self.write(self.authority["steps"], json.dumps(self.plan))
        self.assert_rejected("approved semantic contract digest mismatch")

    def test_requirement_mutation_cannot_drop_acceptance(self):
        target = self.root / self.authority["requirements"]
        requirements = json.loads(target.read_text())
        requirements["requirements"][0]["acceptance"] = "always passes"
        target.write_text(json.dumps(requirements))
        self.assert_rejected("approved semantic contract digest mismatch")

    def test_formatting_neutral_contract_digest(self):
        target = self.root / self.authority["steps"]
        target.write_text(json.dumps(self.plan, indent=4, sort_keys=True))
        self.assertEqual([], self.errors())

    def test_boolean_ordinal_is_not_integer_ordinal(self):
        self.ledger["steps"][0]["ordinal"] = True
        self.assert_rejected("invalid plan ordinal")

    def test_failed_review_cannot_complete(self):
        self.complete_bootstrap()
        self.ledger["steps"][0]["report"]["review"]["status"] = "failed"
        self.assert_rejected("review must have passed")

    def test_review_not_run_cannot_complete(self):
        self.complete_bootstrap()
        self.ledger["steps"][0]["report"]["review"] = "NOT RUN"
        self.assert_rejected("review: expected closed keys")

    def test_failed_self_review_cannot_complete(self):
        self.complete_bootstrap()
        self.ledger["steps"][0]["report"]["self_review"]["status"] = "failed"
        self.assert_rejected("self_review must have passed")

    def test_two_active_steps(self):
        self.ledger["steps"][1]["status"] = "active"
        self.assert_rejected("one active cursor")

    def test_skipped_prerequisite(self):
        self.ledger["steps"][1]["status"] = "complete"
        self.assert_rejected("skipped prerequisite")

    def test_false_complete_claim(self):
        self.complete_bootstrap()
        self.ledger["steps"][2]["report"]["acceptance"][0]["status"] = "planned"
        self.assert_rejected("unverified acceptance")

    def test_missing_required_command(self):
        self.complete_bootstrap()
        self.ledger["steps"][0]["report"]["commands"].pop()
        self.assert_rejected("missing required command")

    def test_unexecuted_current_command(self):
        self.complete_bootstrap()
        self.ledger["steps"][0]["report"]["commands"][0]["kind"] = "historical_receipt_validation"
        self.assert_rejected("actual execution")

    def test_zero_tests_cannot_complete(self):
        self.complete_bootstrap()
        command = next(c for c in self.ledger["steps"][0]["report"]["commands"] if "cargo test" in c["command"])
        command["counts"]["tests"] = 0
        self.assert_rejected("zero/missing executed test count")

    def test_zero_scenarios_cannot_complete(self):
        self.complete_bootstrap()
        command = next(c for c in self.ledger["steps"][0]["report"]["commands"] if "run_distribution" in c["command"])
        command["counts"]["scenarios"] = 0
        self.assert_rejected("zero/missing executed scenario count")

    def test_circular_completion_source(self):
        self.complete_bootstrap()
        self.save()
        containing = self.commit()
        self.ledger["bootstrap"]["source_commit"] = containing
        for row in self.ledger["steps"][:5]:
            row["report"]["source_commit"] = row["report"]["observation_base"] = containing
        self.assert_rejected("circular source")

    def test_future_source(self):
        self.complete_bootstrap()
        self.ledger["steps"][0]["report"]["source_commit"] = "0" * 40
        self.assert_rejected("unavailable/future source")

    def test_receipt_cannot_be_source(self):
        self.complete_bootstrap()
        self.ledger["bootstrap"]["receipt_commit"] = self.source
        self.assert_rejected("circular source/receipt")

    def test_recorded_receipt_requires_actual_observations(self):
        self.complete_bootstrap()
        self.ledger["bootstrap"]["receipt_commit"] = self.anchor
        self.assert_rejected("receipt must follow source")

    def test_later_recording_of_real_receipt(self):
        self.complete_bootstrap()
        self.save()
        receipt = self.commit()
        self.ledger["bootstrap"]["receipt_commit"] = receipt
        self.assertEqual([], self.errors())

    def test_missing_hold(self):
        self.ledger["holds"].pop()
        self.assert_rejected("preserved v19")

    def test_wrong_deviation_owner(self):
        self.ledger["bootstrap"]["deviation"] = "../private-approval.md"
        self.assert_rejected("wrong deviation ownership")

    def test_deviation_cannot_hide_missing_acceptance(self):
        self.complete_bootstrap()
        self.ledger["steps"][4]["report"]["acceptance"].pop()
        self.assert_rejected("exact per-step acceptance")

    def test_mutated_original_prerequisite(self):
        self.ledger["steps"][3]["depends_on"] = []
        self.assert_rejected("original depends_on changed")

    def test_alternate_ledger_input(self):
        alternate = self.root / "mutant.json"
        mutant = copy.deepcopy(self.ledger)
        mutant["holds"] = []
        alternate.write_text(json.dumps(mutant))
        errors = plan_validator.validate_plan(self.root, ledger_path=alternate)
        self.assertTrue(any("holds" in error for error in errors), errors)


if __name__ == "__main__":
    unittest.main()
