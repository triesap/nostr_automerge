#!/usr/bin/env python3
"""Validate the closed opaque private-finalization evidence record."""

from __future__ import annotations

import copy
from typing import Any

from validate_runtime_ledger_v9 import (
    LedgerError,
    load_object,
    projection_digest,
    require,
    validate_no_leak,
    validate_schema_contract,
)


REPORT = "reports/opaque_finalization_v9.json"
SCHEMA = "tools/validation/opaque_finalization_v9.schema.json"
SCHEMA_PROJECTION = "aeb8b64aed5422938e5fb96d41115a64ee2f5479887dbdd0ebfedb7ec0b46361"
RESULT_IDENTITY = "557e37981f1a196e29ff9dabab647b732ec15745b26b066a9df13aee2696c2e0"
REQUIREMENTS = ("NCRDT-RESOURCE-014", "NCRDT-CONF-010", "NCRDT-EVIDENCE-006")
CANDIDATES = (
    ("step_1227", "0994aeebb6fdb6d8d1814250b4771841a3daee9c"),
    ("step_1228", "24bcc0a46ecc9ea6297a55a8a84c41a1ba2029f3"),
    ("step_1229", "2aa4077905e9ad9af3c37ed01a3ea6b948b71aa9"),
    ("step_1230", "7ceb364ce5fbfd77f7a7d5d2bacf145f1122f8be"),
    ("step_1231", "b981a06011abbc46d1faca5aa5c3a2348918da95"),
    ("step_1232", "e83da2c052c985ce8af160c954a472d0bf2055c8"),
    ("step_1233", "3f0a571081e22d9f018f9803bb2efcb248d1e9ec"),
)
COUNTS = {
    "complete_passes": 11,
    "fallback_passes": 3,
    "boundary_cases": 4,
    "stop_causes": 2,
    "interrupted_prefixes_per_cause": 12,
    "callback_error_cases": 11,
    "mutation_families": 10,
}
IDENTITIES = {
    "implementation_identity_sha256": "cd8d956eb7e665527e41d154ce775a7d7d601bbbcec9c79d8277f636a24e5bc4",
    "private_report_sha256": "d58f3724d4b1671a6754e2f680681177f0c2303906fbb6881a9893488c1c578e",
    "private_schema_sha256": "3f585b05b5258c55f979a1a7cc7ab17d29aa26dfabe3579b8059ab255d3cb087",
    "private_result_identity_sha256": "6885351cf8012236597a3115d8a832db55f4c18bb7633b59a240dd4e4b9a5a0d",
}
RESULTS = (
    {"class": "two_tier_settlement", "result": "pass"},
    {"class": "typed_stop_preservation", "result": "pass"},
    {"class": "mutation_rejection", "result": "pass"},
    {"class": "full_private", "result": "pass"},
)


def expected_chain() -> list[dict[str, str]]:
    return [
        {"checkpoint": checkpoint, "candidate": candidate, "result": "pass"}
        for checkpoint, candidate in CANDIDATES
    ]


def validate_report(report: dict[str, Any]) -> None:
    keys = (
        "schema", "checkpoint", "gate_id", "stage", "status",
        "publication_status", "requirement_ids", "candidate_chain",
        "settlement_counts", "regressions", "identities", "result_classes",
        "execution_class", "execution_result", "result_identity_sha256",
    )
    require(tuple(report) == keys, "opaque_finalization:keys")
    require(report["schema"] == "nostr_automerge.opaque_finalization.v9.v1", "opaque_finalization:schema")
    require(report["checkpoint"] == "step_1233", "opaque_finalization:checkpoint")
    require(report["gate_id"] == "GATE_V9_PRIVATE_FINALIZATION", "opaque_finalization:gate")
    require(report["stage"] == "private_finalization_complete", "opaque_finalization:stage")
    require(report["status"] == "pass" and report["publication_status"] == "held", "opaque_finalization:status")
    require(report["requirement_ids"] == list(REQUIREMENTS), "opaque_finalization:requirements")
    require(report["candidate_chain"] == expected_chain(), "opaque_finalization:candidates")
    require(report["settlement_counts"] == COUNTS and tuple(report["settlement_counts"]) == tuple(COUNTS), "opaque_finalization:counts")
    require(report["regressions"] == {"fixed_count": 14, "open_count": 9, "result": "pass"}, "opaque_finalization:regressions")
    require(report["identities"] == IDENTITIES and tuple(report["identities"]) == tuple(IDENTITIES), "opaque_finalization:identities")
    require(report["result_classes"] == list(RESULTS), "opaque_finalization:results")
    require(report["execution_class"] == "environment_independent" and report["execution_result"] == "pass", "opaque_finalization:execution")
    projection = copy.deepcopy(report)
    identity = projection.pop("result_identity_sha256")
    require(identity == RESULT_IDENTITY, "opaque_finalization:identity")
    require(projection_digest(projection) == RESULT_IDENTITY, "opaque_finalization:projection")
    validate_no_leak(report, "opaque_finalization:boundary")


def mutation_self_test(report: dict[str, Any]) -> int:
    mutations: list[dict[str, Any]] = []
    for key in report:
        mutation = copy.deepcopy(report); mutation.pop(key); mutations.append(mutation)
    mutation = copy.deepcopy(report); mutation["note"] = "held"; mutations.append(mutation)
    mutation = copy.deepcopy(report); mutation["schema"] = mutation.pop("schema"); mutations.append(mutation)
    for field in ("requirement_ids", "candidate_chain", "result_classes"):
        mutation = copy.deepcopy(report); mutation[field].reverse(); mutations.append(mutation)
        mutation = copy.deepcopy(report); mutation[field].pop(); mutations.append(mutation)
        mutation = copy.deepcopy(report); mutation[field].append(copy.deepcopy(mutation[field][-1])); mutations.append(mutation)
    for field in COUNTS:
        mutation = copy.deepcopy(report); mutation["settlement_counts"][field] += 1; mutations.append(mutation)
    for field in IDENTITIES:
        mutation = copy.deepcopy(report); mutation["identities"][field] = "f" * 64; mutations.append(mutation)
    mutation = copy.deepcopy(report); mutation["regressions"]["fixed_count"] += 1; mutations.append(mutation)
    mutation = copy.deepcopy(report); mutation["candidate_chain"][0]["candidate"] = "f" * 40; mutations.append(mutation)
    mutation = copy.deepcopy(report); mutation["result_identity_sha256"] = "f" * 64; mutations.append(mutation)
    coordinated = copy.deepcopy(report)
    coordinated["settlement_counts"]["mutation_families"] += 1
    projection = copy.deepcopy(coordinated); projection.pop("result_identity_sha256")
    coordinated["result_identity_sha256"] = projection_digest(projection)
    mutations.append(coordinated)
    for index, mutation in enumerate(mutations):
        try:
            validate_report(mutation)
        except LedgerError:
            continue
        raise LedgerError(f"opaque_finalization:mutation:{index}")
    return len(mutations)


def main() -> None:
    report = load_object(REPORT)
    schema = load_object(SCHEMA)
    validate_schema_contract(
        schema, "opaque_finalization_schema", SCHEMA_PROJECTION
    )
    validate_report(report)
    count = mutation_self_test(report)
    print("PASS: opaque private finalization v9")
    print(f"- candidates={len(CANDIDATES)}")
    print(f"- mutations={count}")
    print(f"- result_identity_sha256={RESULT_IDENTITY}")


if __name__ == "__main__":
    main()
