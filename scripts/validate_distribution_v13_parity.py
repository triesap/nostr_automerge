#!/usr/bin/env python3
"""Validate opaque cross-implementation distribution-v13 parity."""

from __future__ import annotations

import copy
import hashlib
import json
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[1]
OPAQUE_PATH = ROOT / "reports/opaque_compatibility_v13.json"
REPORT_PATH = ROOT / "reports/distribution_v13_parity.json"
SCHEMA_PATH = ROOT / "tools/validation/distribution_v13_parity.schema.json"
CONTRACT_PATH = ROOT / "spec/distribution_v13_compatibility_contract.json"
MANIFEST_PATH = ROOT / "fixtures/distribution/manifest_v13.json"
RUST_PATH = ROOT / "reports/rust_conformance_v13.json"
REPORT_SHA = "25f6f4b4f032d04d8aec3a67dfc434a8dc274ab249542a01b32d22de82d9b4bd"
SCHEMA_SHA = "27502a5eaa53f0797176b7f3eaa4e4035eee7e3755f377e2ae1503744a96fe44"
OPAQUE_SHA = "8fcedef517036392343066206916a9439f73161ffbacc4a33d37e623a12b859b"
HOLDS = ["external_assurance", "event_kind_allocation", "nip_submission", "production_qualification", "publication", "release", "remote_mutation"]
OPAQUE_FIELDS = ["candidate", "predecessor", "result_identity_sha256", "execution_identity_sha256", "signed_input_projection_sha256", "report_projection_sha256", "work_contract_identity_sha256", "scenario_count", "signed_event_count", "delivery_permutations", "process_count", "canonical_output_sha256", "serialized_run_sha256", "byte_mismatch_count", "deliberate_expectation_mismatch", "result"]
REPORT_FIELDS = ["schema", "status", "contract_sha256", "opaque_record_sha256", "rust_result_identity_sha256", "typescript_result_identity_sha256", "signed_input_projection_sha256", "report_projection_sha256", "work_contract_identity_sha256", "scenario_count", "signed_event_count", "delivery_orders", "process_count", "canonical_output_sha256", "byte_mismatch_count", "finding", "finding_status", "holds", "result", "result_identity_sha256"]

class ParityError(RuntimeError): pass
def require(value: bool, code: str) -> None:
    if not value: raise ParityError(code)
def digest(path: Path) -> str: return hashlib.sha256(path.read_bytes()).hexdigest()
def canonical(value: Any) -> bytes: return json.dumps(value, ensure_ascii=False, sort_keys=True, separators=(",", ":")).encode()
def identity(value: dict[str, Any]) -> str: return hashlib.sha256(canonical({key: item for key, item in value.items() if key != "result_identity_sha256"})).hexdigest()

def work_identity(manifest: dict[str, Any]) -> str:
    rows = []
    for entry in manifest["fixtures"]:
        scenario = json.loads((ROOT / entry["input_paths"][0]).read_text())
        rows.append({"fixture_id": entry["fixture_id"], "budget": scenario["budget"], "cancel_after": scenario["cancel_after"], "expected_completion": scenario["expected_report"]["completion"]})
    return hashlib.sha256(canonical(rows)).hexdigest()

def validate(opaque: object, report: object, schema: object) -> None:
    require(type(opaque) is dict and list(opaque) == OPAQUE_FIELDS, "opaque:shape")
    require(type(report) is dict and list(report) == REPORT_FIELDS, "report:shape")
    require(digest(OPAQUE_PATH) == OPAQUE_SHA and report["opaque_record_sha256"] == OPAQUE_SHA, "opaque:hash")
    require(digest(REPORT_PATH) == REPORT_SHA and digest(SCHEMA_PATH) == SCHEMA_SHA, "evidence:hash")
    contract = json.loads(CONTRACT_PATH.read_text())
    manifest = json.loads(MANIFEST_PATH.read_text())
    rust = json.loads(RUST_PATH.read_text())
    require(list(opaque) == contract["opaque_evidence_fields"], "opaque:contract")
    require(not any(token in json.dumps(opaque).lower() for token in contract["prohibited_private_fields"]), "opaque:leak")
    require(opaque["scenario_count"] == 204 and opaque["signed_event_count"] == 771 and opaque["delivery_permutations"] == 8 and opaque["process_count"] == 2, "opaque:counts")
    require(opaque["signed_input_projection_sha256"] == contract["authority"]["signed_input_projection_sha256"], "opaque:input")
    require(opaque["report_projection_sha256"] == contract["authority"]["expected_report_projection_sha256"], "opaque:reports")
    require(opaque["work_contract_identity_sha256"] == work_identity(manifest), "opaque:work")
    require(opaque["canonical_output_sha256"] == rust["canonical_output_sha256"] and opaque["byte_mismatch_count"] == 0, "opaque:bytes")
    require(identity(opaque) == opaque["result_identity_sha256"], "opaque:identity")
    expected = {
        "schema": "nostr_automerge.distribution_v13_parity.v1", "status": "pass",
        "contract_sha256": digest(CONTRACT_PATH), "opaque_record_sha256": OPAQUE_SHA,
        "rust_result_identity_sha256": rust["result_identity_sha256"],
        "typescript_result_identity_sha256": opaque["result_identity_sha256"],
        "signed_input_projection_sha256": opaque["signed_input_projection_sha256"],
        "report_projection_sha256": opaque["report_projection_sha256"],
        "work_contract_identity_sha256": opaque["work_contract_identity_sha256"],
        "scenario_count": 204, "signed_event_count": 771, "delivery_orders": 8,
        "process_count": 2, "canonical_output_sha256": rust["canonical_output_sha256"],
        "byte_mismatch_count": 0, "finding": "FINDING_102", "finding_status": "closed",
        "holds": HOLDS, "result": "pass", "result_identity_sha256": report["result_identity_sha256"],
    }
    require(report == expected and identity(report) == report["result_identity_sha256"], "report:identity")
    require(type(schema) is dict and schema.get("additionalProperties") is False and schema.get("required") == REPORT_FIELDS and list(schema.get("properties", {})) == REPORT_FIELDS, "schema:closed")

def self_test(opaque: dict[str, Any], report: dict[str, Any], schema: dict[str, Any]) -> int:
    mutations = [
        ("opaque", lambda value: value.update(scenario_count=203)),
        ("opaque", lambda value: value.update(work_contract_identity_sha256="0" * 64)),
        ("opaque", lambda value: value.update(canonical_output_sha256="0" * 64)),
        ("opaque", lambda value: value.update(byte_mismatch_count=1)),
        ("opaque", lambda value: value.update(paths=["private"])),
        ("report", lambda value: value.update(finding_status="open")),
        ("report", lambda value: value.update(result_identity_sha256="0" * 64)),
        ("report", lambda value: value["holds"].pop()),
        ("schema", lambda value: value.update(additionalProperties=True)),
        ("schema", lambda value: value["required"].pop()),
    ]
    caught = 0
    for target, mutate in mutations:
        changed_opaque, changed_report, changed_schema = copy.deepcopy(opaque), copy.deepcopy(report), copy.deepcopy(schema)
        mutate({"opaque": changed_opaque, "report": changed_report, "schema": changed_schema}[target])
        try: validate(changed_opaque, changed_report, changed_schema)
        except ParityError: caught += 1; continue
        raise ParityError("mutation:survived")
    return caught

def main() -> None:
    opaque, report, schema = (json.loads(path.read_text()) for path in (OPAQUE_PATH, REPORT_PATH, SCHEMA_PATH))
    validate(opaque, report, schema)
    mutations = self_test(opaque, report, schema)
    print(f"PASS: distribution-v13 parity (204 scenarios, 771 Events, 8 deliveries, 2 processes, {mutations} mutations)")

if __name__ == "__main__": main()
