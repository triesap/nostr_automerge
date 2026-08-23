#!/usr/bin/env python3
"""Validate the closed Rust target-work and shared-byte resource gate."""

from __future__ import annotations

import copy
import hashlib
import json
import subprocess
from pathlib import Path
from typing import Any

from validate_runtime_ledger_v9 import (
    LedgerError,
    load_object,
    projection_digest,
    require,
    validate_no_leak,
    validate_schema_contract,
)


ROOT = Path(__file__).resolve().parents[1]
REPORT = "reports/rust_resource_gate_v9.json"
SCHEMA = "tools/validation/rust_resource_gate_v9.schema.json"
SCHEMA_PROJECTION = "a8fc46e122b798b590093112700259c78e28e8e892a841d07ecb2f9bf9cc3e99"
APPROVED_RESULT_IDENTITY = "41dbbc04929a1eb431baaa9cdd7c982a3b284a45c442040882875eb21c7dfe6d"
REQUIREMENTS = ("NCRDT-RESOURCE-014", "NCRDT-CONF-010", "NCRDT-EVIDENCE-006")
WORK_COUNTERS = (
    "event", "control", "carrier", "checkpoint", "assertion",
    "decode_byte", "crypto_byte", "document_byte", "checkpoint_byte", "report_byte",
)
BOUNDARIES = (
    "exact", "n_minus_one", "final_cancellation", "scope_isolation",
    "deterministic_counters", "deterministic_output",
)
RESULT_CLASSES = (
    "shared_byte_ownership", "target_work_ownership", "exact_boundaries",
    "cancellation_preservation", "scope_isolation", "deterministic_output",
)
CANDIDATES = (
    ("step_1235", "5b08a2b8d271e2df0ccd1711ba564e7b58d4bbc7", "3bec1ed87f7b2298a7d132dea8c7179b0f9afb20", 7, "713e8df3300bc86ac47e87fac2445e79de5abc4c1576243d411d3a33901d638f"),
    ("step_1236", "57f789e294f2139899ad273cd576d15a12173b91", "5b08a2b8d271e2df0ccd1711ba564e7b58d4bbc7", 15, "647b7c766b1b687ca9afa5f2fba0a9af161fa470fde86e87b6f1bdd8a66b7feb"),
    ("step_1237", "6e9beea4ae0e4ead8af2f1791d21f9952010bee9", "57f789e294f2139899ad273cd576d15a12173b91", 7, "6592f2d4f1dda5cb7f01c76edd1f52b1ad7e5083c23f72eab5b9d3dddd9dc801"),
    ("step_1238", "a863d24247c395e6d1988170ac0eca924a9fd570", "6e9beea4ae0e4ead8af2f1791d21f9952010bee9", 10, "2f64a52292678cad3130f67adbdd43a08f72050875b81cb4bd588f68730110c4"),
    ("step_1239", "ef93f361f16ace0fe0a7bc5c61b020485bb6f287", "a863d24247c395e6d1988170ac0eca924a9fd570", 9, "e82cd0371ff5b99c7577072a62e3e6c81db12b5cdfc46263488a17144cfe9d4a"),
    ("step_1240", "f74a7dae5bcb6a10b67e9596bf368db2b2148936", "ef93f361f16ace0fe0a7bc5c61b020485bb6f287", 16, "f354508ee9858adcdb9888ad76a949478d357b38f3feef0e7577f7442a6c2275"),
    ("step_1241", "627e01f189149592150b47f21ce556b606b70ed9", "f74a7dae5bcb6a10b67e9596bf368db2b2148936", 12, "d8a4d2fe7b1e68ecb295277ba0905b6bb092af63bcc609043c46e121da666284"),
    ("step_1242", "fec9ef4c38c4044902285d9bcfadf2f078dc3a6e", "627e01f189149592150b47f21ce556b606b70ed9", 12, "9ac40520c18748590bb27c404f8f3b0127274640e74eb5884223fde2811155ce"),
    ("step_1243", "7925b9596a2406000009c3341ca0c79eb1fe89b9", "fec9ef4c38c4044902285d9bcfadf2f078dc3a6e", 19, "b5ee954448f7a3ef40a836a71ba817e548ea5144aca00723ca46e527de4e2700"),
)
BUDGETS = (
    ("parent_propagation_exact_budget", 6912, "fixtures/v1_draft/scenarios/resource/parent_propagation_exact_budget.input.json"),
    ("unrelated_control_flood_exact_budget", 110, "fixtures/v1_draft/scenarios/resource/unrelated_control_flood_exact_budget.input.json"),
    ("foreign_claim_flood_exact_budget", 110, "fixtures/v1_draft/scenarios/scope/foreign_claim_flood_exact_budget.input.json"),
    ("unrelated_valid_checkpoints_exact_budget", 264, "fixtures/v1_draft/scenarios/scope/unrelated_valid_checkpoints_exact_budget.input.json"),
)


def git_bytes(*arguments: str) -> bytes:
    result = subprocess.run(("git", *arguments), cwd=ROOT, capture_output=True, check=False)
    require(result.returncode == 0 and result.stderr == b"", "rust_resource_gate:git")
    return result.stdout


def scope_observation(parent: str, candidate: str) -> tuple[int, str]:
    fields = git_bytes("diff", "--name-status", "-z", "--no-renames", parent, candidate).split(b"\0")
    require(fields[-1] == b"" and len(fields) % 2 == 1, "rust_resource_gate:scope_shape")
    rows = []
    for index in range(0, len(fields) - 1, 2):
        status = fields[index].decode()
        relative = fields[index + 1].decode()
        require(status != "D", "rust_resource_gate:scope_deletion")
        digest = hashlib.sha256(git_bytes("show", f"{candidate}:{relative}")).hexdigest()
        rows.append({"status": status, "path": relative, "sha256": digest})
    return len(rows), projection_digest(rows)


def expected_chain() -> list[dict[str, Any]]:
    return [
        {"checkpoint": step, "candidate": candidate, "parent": parent,
         "scope_entry_count": count, "scope_identity_sha256": identity, "result": "pass"}
        for step, candidate, parent, count, identity in CANDIDATES
    ]


def expected_budgets() -> list[dict[str, Any]]:
    return [{"fixture_id": fixture_id, "max_items": budget, "result": "pass"}
            for fixture_id, budget, _ in BUDGETS]


def validate_report(report: dict[str, Any]) -> None:
    require(tuple(report) == (
        "schema", "checkpoint", "gate_id", "authority_stage", "status",
        "publication_status", "requirement_ids", "candidate_chain",
        "resource_contract", "exact_budget_fixtures", "boundary_cases",
        "regressions", "validation", "result_classes", "result_identity_sha256",
    ), "rust_resource_gate:keys")
    require(report["schema"] == "nostr_automerge.rust_resource_gate.v9.v1", "rust_resource_gate:schema")
    require(report["checkpoint"] == "step_1244", "rust_resource_gate:checkpoint")
    require(report["gate_id"] == "GATE_V9_RUST_RESOURCE", "rust_resource_gate:gate")
    require(report["authority_stage"] == "checkpoint_expectations_corrected", "rust_resource_gate:stage")
    require(report["status"] == "pass" and report["publication_status"] == "held", "rust_resource_gate:status")
    require(report["requirement_ids"] == list(REQUIREMENTS), "rust_resource_gate:requirements")
    require(report["candidate_chain"] == expected_chain(), "rust_resource_gate:chain")
    require(report["resource_contract"] == {
        "work_counters": list(WORK_COUNTERS), "counter_count": 10,
        "ownership": "target_local", "shared_change_bytes": "single_canonical_allocation",
        "charge_order": "immediately_before_work", "cancellation": "sampled_only_at_charge",
        "result": "pass",
    }, "rust_resource_gate:contract")
    require(report["exact_budget_fixtures"] == expected_budgets(), "rust_resource_gate:budgets")
    require(report["boundary_cases"] == list(BOUNDARIES), "rust_resource_gate:boundaries")
    require(report["regressions"] == {"fixed_count": 11, "open_count": 1, "finding_077": "fixed", "finding_084": "fixed", "result": "pass"}, "rust_resource_gate:regressions")
    require(report["validation"] == {
        "focused_resource": "pass", "checkpoint": "pass", "report": "pass",
        "conformance": "pass", "full_public": "pass", "conformance_scenario_count": 180,
        "delivery_order_count": 8, "process_count": 2,
        "fixture_manifest_sha256": "4c6866b91bffbeba9610c4602b99abfc7e5a16c9d262d6e4d624a4e3a9537f9a",
        "canonical_output_sha256": "cfb32cbf0f2248470ae07d7e42f78301df9014afc2822d622e2c260c8c60b5c6",
        "distribution_run_sha256": "edd05b0ee5f09f8b4fda87b3bf15a1988141a371cd4b13f504a49b27ad345ed4",
        "result": "pass",
    }, "rust_resource_gate:validation")
    require(report["result_classes"] == [{"class": value, "result": "pass"} for value in RESULT_CLASSES], "rust_resource_gate:results")
    projection = copy.deepcopy(report)
    identity = projection.pop("result_identity_sha256")
    require(identity == APPROVED_RESULT_IDENTITY, "rust_resource_gate:identity")
    require(projection_digest(projection) == identity, "rust_resource_gate:projection")
    validate_no_leak(report, "rust_resource_gate:boundary")


def validate_bindings() -> None:
    for checkpoint, candidate, parent, count, identity in CANDIDATES:
        require(git_bytes("rev-parse", f"{candidate}^").decode().strip() == parent, f"rust_resource_gate:parent:{checkpoint}")
        require(scope_observation(parent, candidate) == (count, identity), f"rust_resource_gate:scope:{checkpoint}")
    manifest = ROOT / "fixtures/distribution/manifest_v9.json"
    require(hashlib.sha256(manifest.read_bytes()).hexdigest() == "4c6866b91bffbeba9610c4602b99abfc7e5a16c9d262d6e4d624a4e3a9537f9a", "rust_resource_gate:manifest")
    for fixture_id, budget, relative in BUDGETS:
        source = load_object(relative)
        require(source.get("fixture_id") == fixture_id, f"rust_resource_gate:fixture_id:{fixture_id}")
        require(source.get("budget") == {"max_bytes": 1_000_000, "max_items": budget}, f"rust_resource_gate:fixture_budget:{fixture_id}")


def mutation_self_test(report: dict[str, Any]) -> int:
    mutations = []
    for key in report:
        candidate = copy.deepcopy(report); candidate.pop(key); mutations.append(candidate)
    candidate = copy.deepcopy(report); candidate["note"] = "pass"; mutations.append(candidate)
    candidate = copy.deepcopy(report); candidate["candidate_chain"].reverse(); mutations.append(candidate)
    candidate = copy.deepcopy(report); candidate["candidate_chain"].pop(); mutations.append(candidate)
    candidate = copy.deepcopy(report); candidate["candidate_chain"].append(copy.deepcopy(candidate["candidate_chain"][-1])); mutations.append(candidate)
    candidate = copy.deepcopy(report); candidate["resource_contract"]["work_counters"].reverse(); mutations.append(candidate)
    candidate = copy.deepcopy(report); candidate["resource_contract"]["ownership"] = "global"; mutations.append(candidate)
    candidate = copy.deepcopy(report); candidate["exact_budget_fixtures"].reverse(); mutations.append(candidate)
    for index in range(4):
        candidate = copy.deepcopy(report); candidate["exact_budget_fixtures"][index]["max_items"] += 1; mutations.append(candidate)
    for field in ("fixed_count", "open_count", "finding_077", "finding_084"):
        candidate = copy.deepcopy(report); value = candidate["regressions"][field]
        candidate["regressions"][field] = value + 1 if isinstance(value, int) else "open"; mutations.append(candidate)
    for field in ("fixture_manifest_sha256", "canonical_output_sha256", "distribution_run_sha256"):
        candidate = copy.deepcopy(report); candidate["validation"][field] = "0" * 64; mutations.append(candidate)
    candidate = copy.deepcopy(report); candidate["result_classes"].reverse(); mutations.append(candidate)
    coordinated = copy.deepcopy(report); coordinated["exact_budget_fixtures"][0]["max_items"] += 1
    projection = copy.deepcopy(coordinated); projection.pop("result_identity_sha256")
    coordinated["result_identity_sha256"] = projection_digest(projection); mutations.append(coordinated)
    caught = 0
    for candidate in mutations:
        try:
            validate_report(candidate)
        except LedgerError:
            caught += 1
            continue
        raise LedgerError("rust_resource_gate_mutation_survived")
    return caught


def binding_self_test() -> int:
    final = CANDIDATES[-1]
    observed = (git_bytes("rev-parse", f"{final[1]}^").decode().strip(), *scope_observation(final[2], final[1]))
    require(observed == (final[2], final[3], final[4]), "rust_resource_gate:binding_positive")
    mutations = (("0" * 40, observed[1], observed[2]), (observed[0], observed[1] + 1, observed[2]), (observed[0], observed[1], "0" * 64))
    return sum(1 for candidate in mutations if candidate != (final[2], final[3], final[4]))


def schema_self_test(schema: dict[str, Any]) -> int:
    mutations = []
    candidate = copy.deepcopy(schema); candidate["additionalProperties"] = True; mutations.append(candidate)
    candidate = copy.deepcopy(schema); candidate["required"].pop(); mutations.append(candidate)
    candidate = copy.deepcopy(schema); candidate["properties"]["candidate_chain"]["items"]["additionalProperties"] = True; mutations.append(candidate)
    candidate = copy.deepcopy(schema); candidate["properties"]["resource_contract"]["additionalProperties"] = True; mutations.append(candidate)
    candidate = copy.deepcopy(schema); candidate["required"].reverse(); mutations.append(candidate)
    caught = 0
    for candidate in mutations:
        try:
            validate_schema_contract(candidate, "rust_resource_gate_schema", SCHEMA_PROJECTION)
        except LedgerError:
            caught += 1
            continue
        raise LedgerError("rust_resource_gate_schema_mutation_survived")
    return caught


def main() -> int:
    report = load_object(REPORT)
    schema = load_object(SCHEMA)
    validate_schema_contract(schema, "rust_resource_gate_schema", SCHEMA_PROJECTION)
    validate_report(report)
    validate_bindings()
    print("PASS: Rust target-work resource gate")
    print(f"- candidates={len(CANDIDATES)}")
    print(f"- work_counters={len(WORK_COUNTERS)}")
    print(f"- exact_budget_fixtures={len(BUDGETS)}")
    print(f"- negative_mutations={mutation_self_test(report)}")
    print(f"- binding_negative_mutations={binding_self_test()}")
    print(f"- schema_negative_mutations={schema_self_test(schema)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
