#!/usr/bin/env python3
"""Validate the closed Rust two-tier finalization gate."""

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
REPORT = "reports/rust_finalization_gate_v9.json"
SCHEMA = "tools/validation/rust_finalization_gate_v9.schema.json"
SCHEMA_PROJECTION = "5736b666d4b00dc413e762f98829ce79bc2417a6e35a1dab8ce50abf97162d21"
APPROVED_RESULT_IDENTITY = "ab5f4a6900e8ad6df0dac8f7965c981e9f92782922261f88e156e5fc5ed6759d"
REQUIREMENTS = ("NCRDT-RESOURCE-013", "NCRDT-CONF-010", "NCRDT-EVIDENCE-006")
CANDIDATES = (
    ("step_1218", "b34fc7ce1c46b5100ed8f1514e82066db45a0334", "8e85bd29181ebf36d2cfd7d4ed330b0a0975aa44", 8, "f311271da8592aae3be9a723c15993b3ea00106d4701a614588c926c2bcb3868"),
    ("step_1219", "06c48a96ab0e78e06c5cf8c0f1a99298edf6ece8", "b34fc7ce1c46b5100ed8f1514e82066db45a0334", 11, "8a6ce313f74aca81d4f31c28550a7c5c4d2dbaac34a522868e992a248f0320da"),
    ("step_1220", "74c99e241aa32521846c2f0fcc791803e61c778b", "06c48a96ab0e78e06c5cf8c0f1a99298edf6ece8", 6, "57f1a75a57453c0160db315504a369aa2950790b35338321196d4de6794ccb39"),
    ("step_1221", "1a09181b0db5a0563f699a6483a97a591005578e", "74c99e241aa32521846c2f0fcc791803e61c778b", 6, "1fc0bde370e7690bf8332a72018c391e297db179452a7175c1fc734c61d2d7a2"),
    ("step_1222", "6faf4a0922e6ca33c32b1f503ff29a6f3449f86a", "1a09181b0db5a0563f699a6483a97a591005578e", 6, "8e0fd832957ca92eef2f0115c7db9c233d4ebe133930c5abd8009d82153ca804"),
    ("step_1223", "01c6e9e21b4e51a75fd2012d909b7ae16f77f0ef", "6faf4a0922e6ca33c32b1f503ff29a6f3449f86a", 6, "06e44c79d44b2fdc267048b04b1c7000e8094586f609ac9e2697836911670171"),
    ("step_1224", "eb7300759ffe8262b3eb848ccea0d2dd10f29bc6", "01c6e9e21b4e51a75fd2012d909b7ae16f77f0ef", 6, "e51be648b2669e93b766ddf84cd7219c5bfb6716fd5e60da9a45c8464879d21e"),
    ("step_1225", "66ab2ff05f89638b0dbee66a3962f5ebac768984", "eb7300759ffe8262b3eb848ccea0d2dd10f29bc6", 8, "f94400c26c2f5f8d1fcb6e5a285b790aa267733919f96c916a16d4d5ef4726bb"),
)
COMPLETE_PASSES = (
    "control_records", "semantic_change_records", "change_carrier_events",
    "other_events", "checkpoint_records", "change_classifications",
    "history_digest", "dispositions_digest", "evidence_records",
    "report_invariants", "fixed_overhead",
)
FALLBACK_PASSES = ("digests", "fixed_overhead", "invariants")
BOUNDARIES = ("zero", "n_minus_one", "n", "n_plus_one", "every_pass", "every_cancellation")
MUTATIONS = ("missing", "duplicate", "reordered", "overrun", "underflow", "early_refund", "wrong_tier", "double_settlement")
SOURCE_BINDINGS = (
    ("reference_evaluator", "crates/nostr_automerge/src/engine/reference_evaluator.rs", "d40917502dcb859c6fea94e988ef60917b200d36c38967d63098398f5b55bd39"),
    ("evaluation_report", "crates/nostr_automerge/src/engine/evaluation_report.rs", "fa616b65e518af68cd0219d40b3d510718ba736639bb6b9a8ef0547b40fba708"),
    ("work_budget", "crates/nostr_automerge/src/work_budget.rs", "c1389c0d65ec67c3d40d67a6268eef580fe276002b1b29383eccc130691bf328"),
)
IMPLEMENTATION_IDENTITY = "8897ef5c9358ff895483dbf6a6c94301ac88080c0e1070cccd29716b33eea640"


def git_bytes(*arguments: str) -> bytes:
    result = subprocess.run(("git", *arguments), cwd=ROOT, capture_output=True, check=False)
    require(result.returncode == 0 and result.stderr == b"", "rust_finalization_gate:git")
    return result.stdout


def scope_observation(parent: str, candidate: str) -> tuple[int, str]:
    fields = git_bytes("diff", "--name-status", "-z", "--no-renames", parent, candidate).split(b"\0")
    require(fields[-1] == b"" and len(fields) % 2 == 1, "rust_finalization_gate:scope_shape")
    rows: list[dict[str, str]] = []
    for index in range(0, len(fields) - 1, 2):
        status = fields[index].decode()
        relative = fields[index + 1].decode()
        require(status != "D", "rust_finalization_gate:scope_deletion")
        digest = hashlib.sha256(git_bytes("show", f"{candidate}:{relative}")).hexdigest()
        rows.append({"status": status, "path": relative, "sha256": digest})
    return len(rows), projection_digest(rows)


def expected_chain() -> list[dict[str, Any]]:
    return [
        {
            "checkpoint": checkpoint,
            "candidate": candidate,
            "parent": parent,
            "scope_entry_count": count,
            "scope_identity_sha256": identity,
            "result": "pass",
        }
        for checkpoint, candidate, parent, count, identity in CANDIDATES
    ]


def validate_report(report: dict[str, Any]) -> None:
    keys = (
        "schema", "checkpoint", "gate_id", "authority_stage", "status",
        "publication_status", "requirement_ids", "candidate_chain",
        "settlement_contract", "boundary_cases", "mutation_families",
        "regressions", "validation", "implementation_identity_sha256",
        "result_classes", "result_identity_sha256",
    )
    require(tuple(report) == keys, "rust_finalization_gate:keys")
    require(report["schema"] == "nostr_automerge.rust_finalization_gate.v9.v1", "rust_finalization_gate:schema")
    require(report["checkpoint"] == "step_1226", "rust_finalization_gate:checkpoint")
    require(report["gate_id"] == "GATE_V9_RUST_FINALIZATION", "rust_finalization_gate:gate")
    require(report["authority_stage"] == "checkpoint_expectations_corrected", "rust_finalization_gate:stage")
    require(report["status"] == "pass" and report["publication_status"] == "held", "rust_finalization_gate:status")
    require(report["requirement_ids"] == list(REQUIREMENTS), "rust_finalization_gate:requirements")
    require(report["candidate_chain"] == expected_chain(), "rust_finalization_gate:chain")
    settlement = report["settlement_contract"]
    require(tuple(settlement) == ("complete_passes", "fallback_passes", "complete_pass_count", "fallback_pass_count", "terminal_states", "reservation", "consumption", "refund", "interruption", "result"), "rust_finalization_gate:settlement_shape")
    require(settlement == {
        "complete_passes": list(COMPLETE_PASSES),
        "fallback_passes": list(FALLBACK_PASSES),
        "complete_pass_count": 11,
        "fallback_pass_count": 3,
        "terminal_states": ["complete", "interrupted", "failed"],
        "reservation": "atomic_checked",
        "consumption": "immediately_before_work",
        "refund": "after_valid_complete_report",
        "interruption": "forfeit_complete_consume_fallback",
        "result": "pass",
    }, "rust_finalization_gate:settlement")
    require(report["boundary_cases"] == list(BOUNDARIES), "rust_finalization_gate:boundaries")
    require(report["mutation_families"] == list(MUTATIONS), "rust_finalization_gate:mutations")
    require(report["regressions"] == {"fixed_count": 9, "open_count": 3, "finding_076": "fixed", "result": "pass"}, "rust_finalization_gate:regressions")
    require(report["validation"] == {
        "focused_resource": "pass", "public_api": "pass", "remediation_harness": "pass", "full_public": "pass",
        "conformance_scenario_count": 180, "delivery_order_count": 8,
        "canonical_output_sha256": "84f370b201945c844396406acfb022faa2bdadb32d96206511474a00218770cb",
        "distribution_run_sha256": "74b24f58fe9c20da082dd9ae4c1b344e8468c00a70dbd710adf724ab70ed14c4",
        "result": "pass",
    }, "rust_finalization_gate:validation")
    require(report["implementation_identity_sha256"] == IMPLEMENTATION_IDENTITY, "rust_finalization_gate:implementation")
    require(report["result_classes"] == [
        {"class": "two_tier_settlement", "result": "pass"},
        {"class": "typed_stop_preservation", "result": "pass"},
        {"class": "mutation_rejection", "result": "pass"},
        {"class": "full_public", "result": "pass"},
    ], "rust_finalization_gate:results")
    projection = copy.deepcopy(report)
    identity = projection.pop("result_identity_sha256")
    require(identity == APPROVED_RESULT_IDENTITY, "rust_finalization_gate:identity")
    require(projection_digest(projection) == identity, "rust_finalization_gate:projection")
    validate_no_leak(report, "rust_finalization_gate:boundary")


def validate_repository_bindings() -> None:
    for checkpoint, candidate, parent, count, identity in CANDIDATES:
        require(git_bytes("rev-parse", f"{candidate}^").decode().strip() == parent, f"rust_finalization_gate:parent:{checkpoint}")
        require(scope_observation(parent, candidate) == (count, identity), f"rust_finalization_gate:scope:{checkpoint}")
    rows = []
    final = CANDIDATES[-1][1]
    for label, relative, expected in SOURCE_BINDINGS:
        actual = hashlib.sha256(git_bytes("show", f"{final}:{relative}")).hexdigest()
        require(actual == expected, f"rust_finalization_gate:source:{label}")
        rows.append({"class": label, "sha256": actual})
    require(projection_digest(rows) == IMPLEMENTATION_IDENTITY, "rust_finalization_gate:implementation_projection")


def mutation_self_test(report: dict[str, Any]) -> int:
    mutations: list[dict[str, Any]] = []
    for key in report:
        candidate = copy.deepcopy(report)
        candidate.pop(key)
        mutations.append(candidate)
    extra = copy.deepcopy(report); extra["note"] = "held"; mutations.append(extra)
    reordered = copy.deepcopy(report); reordered["schema"] = reordered.pop("schema"); mutations.append(reordered)
    for field in ("requirement_ids", "candidate_chain", "boundary_cases", "mutation_families", "result_classes"):
        candidate = copy.deepcopy(report); candidate[field].reverse(); mutations.append(candidate)
        candidate = copy.deepcopy(report); candidate[field].pop(); mutations.append(candidate)
        candidate = copy.deepcopy(report); candidate[field].append(copy.deepcopy(candidate[field][-1])); mutations.append(candidate)
    for field in report["settlement_contract"]:
        candidate = copy.deepcopy(report)
        value = candidate["settlement_contract"][field]
        candidate["settlement_contract"][field] = value + 1 if isinstance(value, int) else "fail" if isinstance(value, str) else list(reversed(value))
        mutations.append(candidate)
    for field in report["regressions"]:
        candidate = copy.deepcopy(report); value = candidate["regressions"][field]
        candidate["regressions"][field] = value + 1 if isinstance(value, int) else "fail"; mutations.append(candidate)
    for field in report["validation"]:
        candidate = copy.deepcopy(report); value = candidate["validation"][field]
        candidate["validation"][field] = value + 1 if isinstance(value, int) else "0" * 64 if field.endswith("sha256") else "fail"; mutations.append(candidate)
    coordinated = copy.deepcopy(report); coordinated["regressions"]["fixed_count"] = 10
    projection = copy.deepcopy(coordinated); projection.pop("result_identity_sha256")
    coordinated["result_identity_sha256"] = projection_digest(projection); mutations.append(coordinated)
    leak = copy.deepcopy(report); leak["result_classes"][0]["class"] = "private_workspace"; mutations.append(leak)
    caught = 0
    for candidate in mutations:
        try:
            validate_report(candidate)
        except LedgerError:
            caught += 1
            continue
        raise LedgerError("rust_finalization_gate_mutation_survived")
    return caught


def binding_self_test() -> int:
    row = CANDIDATES[-1]
    observed = (git_bytes("rev-parse", f"{row[1]}^").decode().strip(), *scope_observation(row[2], row[1]))
    require(observed == (row[2], row[3], row[4]), "rust_finalization_gate:binding_positive")
    mutations = (("0" * 40, observed[1], observed[2]), (observed[0], observed[1] + 1, observed[2]), (observed[0], observed[1], "0" * 64))
    caught = 0
    for candidate in mutations:
        try:
            require(candidate == (row[2], row[3], row[4]), "rust_finalization_gate:binding_mutation")
        except LedgerError:
            caught += 1
            continue
        raise LedgerError("rust_finalization_gate_binding_mutation_survived")
    return caught


def schema_self_test(schema: dict[str, Any]) -> int:
    mutations = []
    opened = copy.deepcopy(schema); opened["additionalProperties"] = True; mutations.append(opened)
    missing = copy.deepcopy(schema); missing["required"].pop(); mutations.append(missing)
    candidate = copy.deepcopy(schema); candidate["properties"]["candidate_chain"]["items"]["additionalProperties"] = True; mutations.append(candidate)
    settlement = copy.deepcopy(schema); settlement["properties"]["settlement_contract"]["additionalProperties"] = True; mutations.append(settlement)
    reordered = copy.deepcopy(schema); reordered["required"].reverse(); mutations.append(reordered)
    caught = 0
    for candidate in mutations:
        try:
            validate_schema_contract(candidate, "rust_finalization_gate_schema", SCHEMA_PROJECTION)
        except LedgerError:
            caught += 1
            continue
        raise LedgerError("rust_finalization_gate_schema_mutation_survived")
    return caught


def main() -> int:
    report = load_object(REPORT)
    schema = load_object(SCHEMA)
    validate_schema_contract(schema, "rust_finalization_gate_schema", SCHEMA_PROJECTION)
    validate_report(report)
    validate_repository_bindings()
    mutations = mutation_self_test(report)
    bindings = binding_self_test()
    schemas = schema_self_test(schema)
    print("PASS: Rust two-tier finalization gate")
    print(f"- candidates={len(CANDIDATES)}")
    print(f"- complete_passes={len(COMPLETE_PASSES)}")
    print(f"- fallback_passes={len(FALLBACK_PASSES)}")
    print(f"- negative_mutations={mutations}")
    print(f"- binding_negative_mutations={bindings}")
    print(f"- schema_negative_mutations={schemas}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
