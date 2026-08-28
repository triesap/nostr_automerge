#!/usr/bin/env python3
"""Validate the closed remediation-v12 runtime-operation inventory."""

from __future__ import annotations

import copy
import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
REPORT = ROOT / "reports/remediation_v12_operation_inventory.json"
SCHEMA = ROOT / "tools/validation/remediation_v12_operation_inventory.schema.json"
FIELDS = ["schema","status","candidate","requirements","operations","counts","result"]
ROW_FIELDS = ["id","family","source_path","source_symbol","owner_mode","test","artifact_sha256"]
FAMILIES = ["actor_state","epoch_engine","authorization","dependency_closure","scheduling","quarantine","reference_publication","report_finalization","compatibility_evidence"]
MODES = {"item_metered","exact_reserved","sealed_constant_time"}
IDS = ["actor_predecessor","causal_next_operation","frontier_validation","combined_candidate_semantics","epoch_ancestry","writer_authorization","dependency_closure","candidate_schedule","quarantine_traversal","quarantine_publication","candidate_storage","epoch_result_publication","zero_post_stop","report_finalization","opaque_compatibility"]

class InventoryError(RuntimeError): pass
def require(value: bool, code: str) -> None:
    if not value: raise InventoryError(code)
def sha(path: Path) -> str: return hashlib.sha256(path.read_bytes()).hexdigest()

def validate(report: object, schema: object) -> None:
    require(type(report) is dict and list(report) == FIELDS, "report:shape")
    require(report["schema"] == "nostr_automerge.remediation_v12_operation_inventory.v1" and report["status"] == report["result"] == "pass", "report:state")
    require(report["candidate"] == "81f7ad6f2e803760e3562051eac9b62f1401db46", "report:candidate")
    rows = report["operations"]
    require(type(rows) is list and len(rows) == 15, "operations:count")
    require([row["id"] for row in rows] == IDS, "operations:order")
    require(len({row["id"] for row in rows}) == 15, "operations:unique")
    require(sorted({row["family"] for row in rows}) == sorted(FAMILIES), "operations:families")
    for index, row in enumerate(rows):
        require(type(row) is dict and list(row) == ROW_FIELDS, f"operation:{index}:shape")
        require(row["owner_mode"] in MODES, f"operation:{index}:owner")
        path = ROOT / row["source_path"]
        source = path.read_text()
        require(sha(path) == row["artifact_sha256"], f"operation:{index}:hash")
        require(row["source_symbol"] in source and row["test"] in source, f"operation:{index}:anchor")
    require(report["counts"] == {"operations":15,"item_metered":12,"exact_reserved":1,"sealed_constant_time":2,"unowned":0}, "counts")
    require(type(schema) is dict and schema.get("additionalProperties") is False and schema.get("required") == FIELDS, "schema")

def self_test(report: dict, schema: dict) -> int:
    cases = []
    for label, mutate in (
        ("missing", lambda value: value["operations"].pop()),
        ("extra", lambda value: value["operations"].append(copy.deepcopy(value["operations"][-1]))),
        ("duplicate", lambda value: value["operations"].__setitem__(1, copy.deepcopy(value["operations"][0]))),
        ("order", lambda value: value["operations"].reverse()),
        ("owner", lambda value: value["operations"][0].update(owner_mode="unowned")),
        ("hash", lambda value: value["operations"][0].update(artifact_sha256="0"*64)),
        ("symbol", lambda value: value["operations"][0].update(source_symbol="missing_symbol")),
        ("test", lambda value: value["operations"][0].update(test="missing_test")),
        ("candidate", lambda value: value.update(candidate="0"*40)),
    ):
        changed = copy.deepcopy(report); mutate(changed); cases.append((label, changed, schema))
    changed_schema = copy.deepcopy(schema); changed_schema["additionalProperties"] = True; cases.append(("schema", report, changed_schema))
    for label, changed, changed_schema in cases:
        try: validate(changed, changed_schema)
        except InventoryError: continue
        raise InventoryError("mutation_survived:" + label)
    return len(cases)

def main() -> int:
    report = json.loads(REPORT.read_text()); schema = json.loads(SCHEMA.read_text())
    validate(report, schema); mutations = self_test(report, schema)
    print(f"PASS: remediation-v12 operation inventory operations=15 mutations={mutations}")
    return 0

if __name__ == "__main__": raise SystemExit(main())
