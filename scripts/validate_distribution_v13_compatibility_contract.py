#!/usr/bin/env python3
"""Validate the neutral distribution-v13 compatibility and opaque boundary."""

from __future__ import annotations

import copy
import hashlib
import json
from pathlib import Path
from typing import Any


ROOT = Path(__file__).resolve().parents[1]
CONTRACT_PATH = "spec/distribution_v13_compatibility_contract.json"
SCHEMA_PATH = "tools/validation/distribution_v13_compatibility_contract.schema.json"
MANIFEST_PATH = "fixtures/distribution/manifest_v13.json"
LOCK_PATH = "fixtures/distribution/manifest_v13.lock.json"
RUST_PATH = "reports/rust_conformance_v13.json"
CONTRACT_SHA256 = "e43324b6893a60d59b2fb84f759ae77607d94f32a3eddc3c6bfecae5614ded2d"
SCHEMA_SHA256 = "724471beef2ae7cd60c5ce094d6656ca9df5086a83bcf9995940cd44304cfc0c"
FIELDS = ("schema", "status", "protocol_revision", "authority", "counts", "opaque_evidence_fields", "prohibited_private_fields", "holds", "result")
AUTHORITY_FIELDS = ("manifest_sha256", "manifest_lock_sha256", "rust_result_identity_sha256", "signed_input_projection_sha256", "expected_report_projection_sha256")
COUNT_FIELDS = ("scenarios", "signed_events", "delivery_orders", "processes", "fixture_rebindings")
OPAQUE_FIELDS = (
    "candidate", "predecessor", "result_identity_sha256", "execution_identity_sha256",
    "signed_input_projection_sha256", "report_projection_sha256",
    "work_contract_identity_sha256", "scenario_count", "signed_event_count",
    "delivery_permutations", "process_count", "canonical_output_sha256",
    "serialized_run_sha256", "byte_mismatch_count", "deliberate_expectation_mismatch", "result",
)
PROHIBITED = ("commands", "credentials", "logs", "package_layout", "paths", "source", "urls", "workflows")
HOLDS = ("external_assurance", "event_kind_allocation", "nip_submission", "production_qualification", "publication", "release", "remote_mutation")


class ContractError(RuntimeError):
    pass


def require(condition: bool, code: str) -> None:
    if not condition:
        raise ContractError(code)


def digest(relative: str) -> str:
    return hashlib.sha256((ROOT / relative).read_bytes()).hexdigest()


def canonical(value: Any) -> bytes:
    return json.dumps(value, ensure_ascii=False, sort_keys=True, separators=(",", ":")).encode()


def require_keys(value: object, keys: tuple[str, ...], code: str) -> dict[str, Any]:
    require(type(value) is dict and tuple(value) == keys, code)
    assert isinstance(value, dict)
    return value


def projections(manifest: dict[str, Any]) -> tuple[str, str, int]:
    inputs = []
    expected = []
    event_count = 0
    for row in manifest["fixtures"]:
        scenario = json.loads((ROOT / row["input_paths"][0]).read_text())
        report = json.loads((ROOT / row["expected_path"]).read_text())
        raw_events = scenario["raw_events"]
        event_count += len(raw_events)
        inputs.append({"fixture_id": row["fixture_id"], "raw_events": raw_events})
        expected.append({"fixture_id": row["fixture_id"], "expected_report": report})
    return hashlib.sha256(canonical(inputs)).hexdigest(), hashlib.sha256(canonical(expected)).hexdigest(), event_count


def validate_contract(value: object, manifest: dict[str, Any], rust: dict[str, Any]) -> None:
    record = require_keys(value, FIELDS, "contract:keys")
    require(record["schema"] == "nostr_automerge.distribution_v13_compatibility_contract.v1", "contract:schema")
    require(record["status"] == "approved" and record["protocol_revision"] == "draft_2026_08", "contract:status")
    authority = require_keys(record["authority"], AUTHORITY_FIELDS, "contract:authority_keys")
    input_hash, expected_hash, event_count = projections(manifest)
    require(authority == {
        "manifest_sha256": digest(MANIFEST_PATH),
        "manifest_lock_sha256": digest(LOCK_PATH),
        "rust_result_identity_sha256": rust["result_identity_sha256"],
        "signed_input_projection_sha256": input_hash,
        "expected_report_projection_sha256": expected_hash,
    }, "contract:authority")
    counts = require_keys(record["counts"], COUNT_FIELDS, "contract:count_keys")
    require(counts == {"scenarios": 204, "signed_events": event_count, "delivery_orders": 8, "processes": 2, "fixture_rebindings": 4}, "contract:counts")
    require(tuple(record["opaque_evidence_fields"]) == OPAQUE_FIELDS, "contract:opaque")
    require(tuple(record["prohibited_private_fields"]) == PROHIBITED, "contract:prohibited")
    require(not set(OPAQUE_FIELDS).intersection(PROHIBITED), "contract:overlap")
    require(tuple(record["holds"]) == HOLDS and record["result"] == "pass", "contract:result")
    require(digest(CONTRACT_PATH) == CONTRACT_SHA256, "contract:hash")


def validate_schema(value: object) -> None:
    schema = require_keys(value, ("$schema", "$id", "type", "additionalProperties", "required", "properties", "$defs"), "schema:keys")
    require(digest(SCHEMA_PATH) == SCHEMA_SHA256, "schema:hash")
    require(schema["type"] == "object" and schema["additionalProperties"] is False, "schema:closed")
    require(schema["required"] == list(FIELDS) and tuple(schema["properties"]) == FIELDS, "schema:fields")
    require(schema["properties"]["authority"]["additionalProperties"] is False, "schema:authority")
    require(schema["properties"]["counts"]["additionalProperties"] is False, "schema:counts")
    require(schema["properties"]["opaque_evidence_fields"]["maxItems"] == 16, "schema:opaque")


def mutation_self_test(contract: dict[str, Any], schema: dict[str, Any], manifest: dict[str, Any], rust: dict[str, Any]) -> int:
    mutations = []
    for mutate in (
        lambda value: value.update(extra=False),
        lambda value: value.pop("authority"),
        lambda value: value["authority"].update(manifest_sha256="0" * 64),
        lambda value: value["authority"].update(signed_input_projection_sha256="0" * 64),
        lambda value: value["counts"].update(scenarios=203),
        lambda value: value["counts"].update(signed_events=770),
        lambda value: value["opaque_evidence_fields"].pop(),
        lambda value: value["opaque_evidence_fields"].append("paths"),
        lambda value: value["opaque_evidence_fields"].reverse(),
        lambda value: value["prohibited_private_fields"].pop(),
        lambda value: value["holds"].pop(),
        lambda value: value.update(result="fail"),
    ):
        changed = copy.deepcopy(contract)
        mutate(changed)
        mutations.append(changed)
    for changed in mutations:
        try:
            validate_contract(changed, manifest, rust)
        except ContractError:
            continue
        raise ContractError("mutation:contract")
    schema_mutations = []
    for mutate in (
        lambda value: value.update(additionalProperties=True),
        lambda value: value["required"].pop(),
        lambda value: value["properties"]["authority"].update(additionalProperties=True),
        lambda value: value["properties"]["opaque_evidence_fields"].update(maxItems=17),
    ):
        changed = copy.deepcopy(schema)
        mutate(changed)
        schema_mutations.append(changed)
    for changed in schema_mutations:
        try:
            validate_schema(changed)
        except ContractError:
            continue
        raise ContractError("mutation:schema")
    return len(mutations) + len(schema_mutations)


def main() -> None:
    contract = json.loads((ROOT / CONTRACT_PATH).read_text())
    schema = json.loads((ROOT / SCHEMA_PATH).read_text())
    manifest = json.loads((ROOT / MANIFEST_PATH).read_text())
    rust = json.loads((ROOT / RUST_PATH).read_text())
    validate_contract(contract, manifest, rust)
    validate_schema(schema)
    mutations = mutation_self_test(contract, schema, manifest, rust)
    print("PASS: distribution-v13 compatibility contract")
    print("- scenarios=204 signed_events=771 opaque_fields=16")
    print(f"- negative_mutations={mutations}")


if __name__ == "__main__":
    main()
