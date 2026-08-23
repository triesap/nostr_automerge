#!/usr/bin/env python3
"""Validate and optionally execute exact report-clause and finding proof."""

from __future__ import annotations

import argparse
import copy
import hashlib
import json
import subprocess
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import Any, Sequence

sys.dont_write_bytecode = True

from validate_report_contract_v9 import EXPECTED_CLAUSES, REPORT_PROOFS


ROOT = Path(__file__).resolve().parents[1]
SOURCE_CANDIDATE = "3b3dd73a93cb4e33ab08a600ff6294538a5b91bd"
FINDING_IDS = tuple(f"FINDING_{number:03d}" for number in range(73, 94))
HELD_FINDING = "FINDING_080"


class ClosureError(ValueError):
    """One exact report-clause or finding proof invariant failed."""


@dataclass(frozen=True)
class ClosureProof:
    kind: str
    target: str
    selector: str
    runner: str
    arguments: tuple[str, ...] = ()

    @property
    def identity(self) -> str:
        return f"{self.kind}:{self.target}:{self.selector}"


@dataclass(frozen=True)
class FindingBinding:
    identifier: str
    semantic_category: str
    status: str
    proofs: tuple[ClosureProof, ...]


def rust_test(target: str, selector: str) -> ClosureProof:
    return ClosureProof("rust_test", target, selector, "cargo")


def validator(path: str, *arguments: str) -> ClosureProof:
    return ClosureProof("validator", "validator", path, path, arguments)


def hold_record(path: str, runner: str) -> ClosureProof:
    return ClosureProof("hold_record", "external_hold", path, runner)


FINDING_BINDINGS = (
    FindingBinding("FINDING_073", "checkpoint_verification", "closed", (
        rust_test("public_engine_api", "finding_073_checkpoint_authorization_precedes_history"),
    )),
    FindingBinding("FINDING_074", "change_application", "closed", (
        rust_test("public_engine_api", "finding_074_invalid_carrier_is_independent_of_excluded_hash"),
    )),
    FindingBinding("FINDING_075", "report_contract", "closed", (
        rust_test("rust_lib", "reference::evaluate::tests::finding_075_interrupted_batch_discards_all_canonical_progress"),
    )),
    FindingBinding("FINDING_076", "resource_accounting", "closed", (
        rust_test("rust_lib", "engine::reference_evaluator::tests::finding_076_finalization_rejects_reordered_named_passes"),
    )),
    FindingBinding("FINDING_077", "resource_accounting", "closed", (
        rust_test("rust_lib", "reference::evaluate::tests::finding_077_canonical_raw_bytes_share_one_allocation"),
    )),
    FindingBinding("FINDING_078", "evidence_integrity", "closed", (
        validator("scripts/validate_rust_requirement_proofs_v10.py", "--run-suite"),
    )),
    FindingBinding("FINDING_079", "change_application", "closed", (
        rust_test("public_engine_api", "finding_079_unsupported_carrier_does_not_create_semantic_hash_state"),
    )),
    FindingBinding("FINDING_080", "external_hold", "held", (
        hold_record("reports/external_holds_v8.json", "scripts/validate_assurance_v9.py"),
    )),
    FindingBinding("FINDING_081", "report_contract", "closed", (
        rust_test("rust_lib", "engine::evaluation_report::tests::finding_081_incomplete_report_rejects_canonical_cross_view_state"),
    )),
    FindingBinding("FINDING_082", "resource_accounting", "closed", (
        rust_test("rust_lib", "engine::reference_evaluator::tests::finding_082_reevaluation_stops_before_post_incomplete_alert_work"),
    )),
    FindingBinding("FINDING_083", "resource_accounting", "closed", (
        rust_test("public_engine_api", "finding_083_budget_stop_is_not_relabelled_by_cancellation_requery"),
    )),
    FindingBinding("FINDING_084", "checkpoint_verification", "closed", (
        rust_test("rust_lib", "checkpoint::assemble::tests::finding_084_checkpoint_sort_stops_before_cancelled_work"),
    )),
    FindingBinding("FINDING_085", "checkpoint_verification", "closed", (
        validator("scripts/validate_checkpoint_parity_v9.py"),
    )),
    FindingBinding("FINDING_086", "checkpoint_verification", "closed", (
        validator("scripts/validate_checkpoint_parity_v9.py"),
    )),
    FindingBinding("FINDING_087", "wire_ingress", "closed", (
        validator("scripts/validate_signed_conformance_gate_v10.py"),
    )),
    FindingBinding("FINDING_088", "resource_accounting", "closed", (
        validator("scripts/validate_opaque_resource_gate_v9.py"),
    )),
    FindingBinding("FINDING_089", "resource_accounting", "closed", (
        validator("scripts/validate_opaque_finalization_v9.py"),
    )),
    FindingBinding("FINDING_090", "report_contract", "closed", (
        validator("scripts/validate_report_parity_v9.py", "--run-suite"),
    )),
    FindingBinding("FINDING_091", "evidence_integrity", "closed", (
        validator("scripts/validate_signed_conformance_gate_v10.py"),
    )),
    FindingBinding("FINDING_092", "signed_conformance", "closed", (
        validator("scripts/validate_signed_conformance_gate_v10.py"),
    )),
    FindingBinding("FINDING_093", "evidence_integrity", "closed", (
        validator("scripts/validate_rust_requirement_proofs_v10.py", "--run-suite"),
        validator("scripts/validate_signed_conformance_gate_v10.py"),
    )),
)


def require(condition: bool, diagnostic: str) -> None:
    if not condition:
        raise ClosureError(diagnostic)


def load(relative: str) -> dict[str, Any]:
    value = json.loads((ROOT / relative).read_text(encoding="utf-8"))
    require(isinstance(value, dict), f"object:{relative}")
    return value


def clause_rows() -> list[dict[str, Any]]:
    return [
        {
            "id": proof.clause,
            "semantic_category": "report_contract",
            "status": "pass",
            "closure_proofs": [f"rust_test:{proof.target}:{proof.test}"],
        }
        for proof in REPORT_PROOFS
    ]


def finding_rows(bindings: Sequence[FindingBinding]) -> list[dict[str, Any]]:
    return [
        {
            "id": binding.identifier,
            "semantic_category": binding.semantic_category,
            "status": binding.status,
            "closure_proofs": [proof.identity for proof in binding.proofs],
        }
        for binding in bindings
    ]


def projection_identity(
    clauses: list[dict[str, Any]], findings: list[dict[str, Any]]
) -> str:
    value = {
        "candidate": SOURCE_CANDIDATE,
        "requirement_count": 148,
        "report_clauses": clauses,
        "findings": findings,
    }
    return hashlib.sha256(
        json.dumps(value, sort_keys=True, separators=(",", ":")).encode()
    ).hexdigest()


def validate(
    clauses: list[dict[str, Any]],
    findings: list[dict[str, Any]],
    bindings: Sequence[FindingBinding],
) -> None:
    requirements = load("spec/requirements.json").get("requirements")
    source_findings = load("spec/remediation_findings_v9.json").get("findings")
    require(isinstance(requirements, list) and len(requirements) == 148, "requirements")
    require(isinstance(source_findings, list), "source_findings")
    requirement_ids = {row.get("id") for row in requirements}
    require([row["id"] for row in clauses] == list(EXPECTED_CLAUSES), "clause_order")
    require(len(clauses) == len({row["id"] for row in clauses}) == 21, "clause_unique")
    require(all(row["semantic_category"] == "report_contract" for row in clauses), "clause_category")
    require(all(row["status"] == "pass" for row in clauses), "clause_status")
    require(all(len(row["closure_proofs"]) == 1 for row in clauses), "clause_proof")
    for row, proof in zip(clauses, REPORT_PROOFS, strict=True):
        require(
            row["closure_proofs"] == [f"rust_test:{proof.target}:{proof.test}"],
            f"clause_binding:{proof.clause}",
        )
    require([row["id"] for row in findings] == list(FINDING_IDS), "finding_order")
    require([row.get("id") for row in source_findings] == list(FINDING_IDS), "finding_authority")
    require(len(findings) == len({row["id"] for row in findings}) == 21, "finding_unique")
    require(set(requirement_ids).isdisjoint(EXPECTED_CLAUSES), "requirement_clause_overlap")
    require(set(requirement_ids).isdisjoint(FINDING_IDS), "requirement_finding_overlap")
    require(len(bindings) == 21, "binding_count")
    for row, binding in zip(findings, bindings, strict=True):
        require(row["id"] == binding.identifier, f"finding_id:{binding.identifier}")
        require(row["semantic_category"] == binding.semantic_category, f"finding_category:{binding.identifier}")
        require(row["status"] == binding.status, f"finding_status:{binding.identifier}")
        require((row["status"] == "held") == (row["id"] == HELD_FINDING), f"finding_hold:{binding.identifier}")
        require(row["closure_proofs"] == [proof.identity for proof in binding.proofs], f"finding_proofs:{binding.identifier}")
        require(bool(row["closure_proofs"]), f"finding_proof_empty:{binding.identifier}")
        for proof in binding.proofs:
            require(proof.kind in {"rust_test", "validator", "hold_record"}, f"proof_kind:{binding.identifier}")
            if proof.kind == "rust_test":
                require(proof.target in {"rust_lib", "public_engine_api"}, f"test_target:{binding.identifier}")
            else:
                require((ROOT / proof.runner).is_file(), f"runner:{binding.identifier}")
    require(sum(row["status"] == "closed" for row in findings) == 20, "closed_count")
    require(sum(row["status"] == "held" for row in findings) == 1, "held_count")


def mutation_self_test(
    clauses: list[dict[str, Any]], findings: list[dict[str, Any]]
) -> int:
    mutations: list[tuple[str, list[dict[str, Any]], list[dict[str, Any]]]] = []
    missing_clause = copy.deepcopy(clauses); missing_clause.pop(); mutations.append(("missing_clause", missing_clause, findings))
    reordered_clause = copy.deepcopy(clauses); reordered_clause.reverse(); mutations.append(("reordered_clause", reordered_clause, findings))
    duplicate_clause = copy.deepcopy(clauses); duplicate_clause[-1] = duplicate_clause[0]; mutations.append(("duplicate_clause", duplicate_clause, findings))
    stale_clause = copy.deepcopy(clauses); stale_clause[0]["closure_proofs"] = ["rust_test:public_api:stale"]; mutations.append(("stale_clause", stale_clause, findings))
    missing_finding = copy.deepcopy(findings); missing_finding.pop(); mutations.append(("missing_finding", clauses, missing_finding))
    reordered_finding = copy.deepcopy(findings); reordered_finding.reverse(); mutations.append(("reordered_finding", clauses, reordered_finding))
    duplicate_finding = copy.deepcopy(findings); duplicate_finding[-1] = duplicate_finding[0]; mutations.append(("duplicate_finding", clauses, duplicate_finding))
    false_hold = copy.deepcopy(findings); false_hold[0]["status"] = "held"; mutations.append(("false_hold", clauses, false_hold))
    false_close = copy.deepcopy(findings); false_close[7]["status"] = "closed"; mutations.append(("false_close", clauses, false_close))
    wrong_category = copy.deepcopy(findings); wrong_category[0]["semantic_category"] = "external_hold"; mutations.append(("wrong_category", clauses, wrong_category))
    empty_proof = copy.deepcopy(findings); empty_proof[0]["closure_proofs"] = []; mutations.append(("empty_proof", clauses, empty_proof))
    stale_proof = copy.deepcopy(findings); stale_proof[0]["closure_proofs"] = ["rust_test:public_engine_api:stale"]; mutations.append(("stale_proof", clauses, stale_proof))
    caught = 0
    for name, changed_clauses, changed_findings in mutations:
        try:
            validate(changed_clauses, changed_findings, FINDING_BINDINGS)
        except ClosureError:
            caught += 1
            continue
        raise ClosureError(f"mutation_survived:{name}")
    return caught


def rust_command(proof: ClosureProof) -> list[str]:
    command = ["cargo", "extbuild", "run", "--", "cargo", "test", "-p", "nostr_automerge"]
    if proof.target == "rust_lib":
        command.append("--lib")
    else:
        command.extend(("--test", proof.target))
    command.extend(("--locked", proof.selector, "--", "--exact"))
    return command


def execute_validator(proof: ClosureProof) -> None:
    command = [sys.executable, str(ROOT / proof.runner), *proof.arguments]
    result = subprocess.run(command, cwd=ROOT, capture_output=True, text=True, check=False)
    require(result.returncode == 0, f"validator:{proof.selector}\n{result.stdout}\n{result.stderr}")
    require(result.stdout.startswith("PASS:"), f"validator_result:{proof.selector}")


def execute() -> tuple[int, int]:
    report = subprocess.run(
        [sys.executable, str(ROOT / "scripts/validate_report_contract_v9.py"), "--run-suite"],
        cwd=ROOT,
        capture_output=True,
        text=True,
        check=False,
    )
    require(report.returncode == 0, f"report_suite\n{report.stdout}\n{report.stderr}")
    require("- executed=21\n" in report.stdout, "report_execution")
    unique = tuple(dict.fromkeys(proof for binding in FINDING_BINDINGS for proof in binding.proofs))
    for proof in unique:
        if proof.kind == "rust_test":
            command = rust_command(proof)
            result = subprocess.run(command, cwd=ROOT, capture_output=True, text=True, check=False)
            require(result.returncode == 0, f"test:{proof.selector}\n{result.stdout}\n{result.stderr}")
            require(f"test {proof.selector} ... ok" in result.stdout, f"test_identity:{proof.selector}")
            require(result.stdout.count(" 1 passed;") == 1, f"test_count:{proof.selector}")
        else:
            execute_validator(proof)
    return 21, len(unique)


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--run-suite", action="store_true")
    arguments = parser.parse_args()
    clauses = clause_rows()
    findings = finding_rows(FINDING_BINDINGS)
    validate(clauses, findings, FINDING_BINDINGS)
    mutations = mutation_self_test(clauses, findings)
    report_executed, finding_executed = execute() if arguments.run_suite else (0, 0)
    print("PASS: exact report-clause and finding proof audit v10")
    print("- requirement_rows_unchanged=148")
    print(f"- report_clauses={len(clauses)}")
    print(f"- findings={len(findings)}")
    print("- closed_findings=20")
    print("- held_findings=1")
    print(f"- unique_finding_proofs={finding_executed if arguments.run_suite else len(set(proof for binding in FINDING_BINDINGS for proof in binding.proofs))}")
    print(f"- negative_mutations={mutations}")
    print(f"- projection_sha256={projection_identity(clauses, findings)}")
    print(f"- report_executed={report_executed}")
    print(f"- finding_executed={finding_executed}")
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except ClosureError as error:
        raise SystemExit(f"FAIL: {error}") from error
