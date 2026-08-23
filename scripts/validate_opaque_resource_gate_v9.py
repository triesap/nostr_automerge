#!/usr/bin/env python3
"""Validate the closed opaque target-work evidence gate."""

from __future__ import annotations

import copy
import hashlib
import json
from pathlib import Path
from typing import Any


ROOT = Path(__file__).resolve().parents[1]
REPORT = ROOT / "reports/opaque_resource_gate_v9.json"
SCHEMA = ROOT / "tools/validation/opaque_resource_gate_v9.schema.json"
REPORT_SHA256 = "8c51df33b0e1cd2af1b886fe13f4add80133cbba32a997373b9444e1f0b86969"
SCHEMA_SHA256 = "ce7684174f5391fbc98a8cba101df2f991583eca8ed0c50e172d2139e2b73bc4"
RESULT_IDENTITY = "730731c61fe5f3002a6db7d5ceedb540991d362f70a560757b45199dbd0d8fde"

REPORT_KEYS = (
    "schema", "checkpoint", "gate_id", "authority_stage", "status",
    "publication_status", "requirement_ids", "candidate_chain", "resource_contract",
    "boundary_results", "scaling", "validation", "result_classes",
    "result_identity_sha256",
)
REQUIREMENTS = ("NCRDT-RESOURCE-001", "NCRDT-RESOURCE-014", "NCRDT-EVIDENCE-006")
CANDIDATES = (
    ("step_1255", "3ebec1cb4f8206c9560386fedb9e5ad6523f00bc", "44f45ef65c6c6a0628d0ffd169ef82c53a9c1b4d", 4, "bab054f7aa8b4ee19bc30136ba912720654a813b854dd52c4b7f5aa4534fcbf8"),
    ("step_1256", "70a1ca45d0bea247ef8784d30febf0db5722d441", "3ebec1cb4f8206c9560386fedb9e5ad6523f00bc", 3, "1d9665a41baa05b0965a9e6bc9f40b74b90dfd9fbcc66203d84ddbd43719bd76"),
    ("step_1257", "66d61287b8786e0ae04aad51bcc30bc77257a4a6", "70a1ca45d0bea247ef8784d30febf0db5722d441", 3, "9a74a7e4be235189b5198e3b5c4bc59336fed9b89ba52b7b36243eb047dea49a"),
    ("step_1258", "bbbcf33c5bcc680400081cc77bdd99e8c6487bf6", "66d61287b8786e0ae04aad51bcc30bc77257a4a6", 4, "dbf0c7cc9b09d1a8c74ee51e0dd06ba9b70ee6143e63536400533521f3cb1068"),
    ("step_1259", "6a6316126691b3be01cb3d6b3ee40a2f9174bd73", "bbbcf33c5bcc680400081cc77bdd99e8c6487bf6", 3, "eb19f7ad3d0dd15715d520be7a55a41d9c3267560f1ca1a75cf1594c2e03cda4"),
    ("step_1260", "5e94ed3d44866ede7bd9cdf3723a01bdc61ceea3", "6a6316126691b3be01cb3d6b3ee40a2f9174bd73", 3, "9fadf36bd113b6d535d1be9683c5ff4316dca78a734d7ccbc5981042ba1ba75a"),
    ("step_1261", "d7d6c21fd3cf095c6296837b66d7665ffa78de6a", "5e94ed3d44866ede7bd9cdf3723a01bdc61ceea3", 1, "aadbd5eb241afd51d6b5973202f3a4a3abc98c495b0bcb9cd1271ad7a40109a4"),
    ("step_1262", "fb585804db1f869014f4d10f57847c081c3635a4", "d7d6c21fd3cf095c6296837b66d7665ffa78de6a", 5, "fbd2f5289806ce2dc36b10bbd9b5c7c43ea1c48d317f7a1d7fe224fe9562d538"),
)
COUNTERS = (
    "control_items", "branch_items", "ancestry_edges", "dependency_edges",
    "carrier_items", "prior_knowledge_items", "checkpoint_items", "automerge_items",
    "closure_edges", "materialized_items", "report_items", "report_bytes",
)
BOUNDARIES = (
    ("exact_pass_budgets", 24),
    ("every_boundary_cancellation", 24),
    ("delivery_permutations", 3),
    ("unrelated_evidence_flood", 32),
)
RESULT_CLASSES = (
    "counter_ownership", "exact_budget_boundaries", "typed_cancellation",
    "unrelated_work_isolation", "deterministic_scaling", "full_private",
)


class GateError(RuntimeError):
    pass


def require(condition: bool, code: str) -> None:
    if not condition:
        raise GateError(code)


def canonical(value: Any) -> bytes:
    return json.dumps(value, sort_keys=True, separators=(",", ":"), ensure_ascii=False).encode()


def sha256(value: bytes) -> str:
    return hashlib.sha256(value).hexdigest()


def load(path: Path) -> dict[str, Any]:
    value = json.loads(path.read_text(encoding="utf-8"))
    require(type(value) is dict, "shape")
    return value


def validate(report: dict[str, Any], *, bind_bytes: bool = False) -> None:
    require(tuple(report) == REPORT_KEYS, "report_keys")
    require(report["schema"] == "nostr_automerge.opaque_resource_gate.v9.v1", "schema")
    require(report["checkpoint"] == "step_1263", "checkpoint")
    require(report["gate_id"] == "GATE_V9_PRIVATE_RESOURCE", "gate")
    require(report["authority_stage"] == "checkpoint_expectations_corrected", "stage")
    require(report["status"] == "pass" and report["publication_status"] == "held", "status")
    require(tuple(report["requirement_ids"]) == REQUIREMENTS, "requirements")

    chain = report["candidate_chain"]
    require(type(chain) is list and len(chain) == len(CANDIDATES), "candidate_count")
    for row, expected in zip(chain, CANDIDATES, strict=True):
        require(row == {
            "checkpoint": expected[0], "candidate": expected[1], "parent": expected[2],
            "scope_entry_count": expected[3], "scope_identity_sha256": expected[4],
            "result": "pass",
        }, "candidate_row")

    require(report["resource_contract"] == {
        "counter_families": list(COUNTERS), "counter_count": 12, "pass_count": 24,
        "charge_order": "immediately_before_work", "cancellation": "sampled_only_at_charge",
        "reservation_classification": "classified_at_execution_boundary", "result": "pass",
    }, "resource_contract")
    require(report["boundary_results"] == [
        {"class": name, "count": count, "result": "pass"} for name, count in BOUNDARIES
    ], "boundaries")
    require(report["scaling"] == {
        "classification": "indexed_linear_or_log_linear",
        "sample_sizes": [8, 16, 64, 256],
        "property_read_counts": [8, 16, 64, 256],
        "quadratic_regression": "fixed", "result": "pass",
    }, "scaling")
    require(report["validation"] == {
        "mutation_count": 25, "pass_count": 351, "intentional_skip_count": 15,
        "fixed_regression_count": 21, "open_regression_count": 2,
        "full_check": "pass", "result": "pass",
    }, "validation")
    require(report["result_classes"] == [
        {"class": name, "result": "pass"} for name in RESULT_CLASSES
    ], "result_classes")
    projected = copy.deepcopy(report)
    identity = projected.pop("result_identity_sha256")
    require(identity == RESULT_IDENTITY == sha256(canonical(projected)), "result_identity")

    encoded = canonical(report).lower()
    for forbidden in (b"domains/labs", b"nostr_automerge_typescript", b"/users/", b"file://", b"github.com"):
        require(forbidden not in encoded, "opaque_boundary")
    if bind_bytes:
        require(sha256(REPORT.read_bytes()) == REPORT_SHA256, "report_bytes")
        require(sha256(SCHEMA.read_bytes()) == SCHEMA_SHA256, "schema_bytes")


def self_test(report: dict[str, Any]) -> int:
    mutations = (
        lambda value: value.update(extra=False),
        lambda value: value.pop("status"),
        lambda value: value.update(status="fail"),
        lambda value: value["requirement_ids"].reverse(),
        lambda value: value["candidate_chain"].pop(),
        lambda value: value["candidate_chain"].reverse(),
        lambda value: value["candidate_chain"][0].update(candidate="0" * 40),
        lambda value: value["candidate_chain"][1].update(parent="0" * 40),
        lambda value: value["candidate_chain"][2].update(scope_entry_count=4),
        lambda value: value["candidate_chain"][3].update(scope_identity_sha256="0" * 64),
        lambda value: value["candidate_chain"][4].update(result="fail"),
        lambda value: value["resource_contract"]["counter_families"].reverse(),
        lambda value: value["resource_contract"].update(counter_count=11),
        lambda value: value["resource_contract"].update(pass_count=23),
        lambda value: value["resource_contract"].update(charge_order="before_batch"),
        lambda value: value["resource_contract"].update(cancellation="polled_after_work"),
        lambda value: value["boundary_results"].reverse(),
        lambda value: value["boundary_results"][0].update(count=23),
        lambda value: value["boundary_results"][3].update(count=31),
        lambda value: value["scaling"].update(classification="quadratic"),
        lambda value: value["scaling"]["sample_sizes"].reverse(),
        lambda value: value["scaling"]["property_read_counts"].__setitem__(3, 257),
        lambda value: value["validation"].update(pass_count=350),
        lambda value: value["result_classes"].pop(),
        lambda value: value.update(result_identity_sha256="0" * 64),
    )
    for index, mutate in enumerate(mutations):
        candidate = copy.deepcopy(report)
        mutate(candidate)
        try:
            validate(candidate)
        except GateError:
            continue
        raise GateError(f"mutation_survived:{index}")
    return len(mutations)


def main() -> None:
    report = load(REPORT)
    validate(report, bind_bytes=True)
    mutations = self_test(report)
    print(
        "PASS: opaque target-work gate "
        f"({len(CANDIDATES)} candidates, {len(COUNTERS)} counters, {mutations} mutations)"
    )


if __name__ == "__main__":
    main()
