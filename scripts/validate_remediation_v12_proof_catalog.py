#!/usr/bin/env python3
"""Validate the exact remediation-v12 proof catalog."""

from __future__ import annotations

import argparse
import copy
import hashlib
import json
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
REPORT = ROOT / "reports/remediation_v12_proof_catalog.json"
SCHEMA = ROOT / "tools/validation/remediation_v12_proof_catalog.schema.json"
INVENTORY = ROOT / "reports/remediation_v12_operation_inventory.json"
FIELDS = ["schema","status","candidate","operation_inventory_sha256","report_suite_identity_sha256","requirements","findings","fixture_families","stop_laws","counts","result"]
REQUIREMENTS = ["NCRDT-RESOURCE-017","NCRDT-RESOURCE-018","NCRDT-RESOURCE-019","NCRDT-EVIDENCE-007"]
FIXTURES = ["deep_actor_predecessor","empty_frontier","many_actor_causal_next","post_epoch_stop","wide_epoch_ancestry","writer_authorization","distribution_v13"]
STOPS = ["charge_before_work","first_stop_preserved","n_minus_one_stops","zero_post_stop_work"]
PREFIXES = {
    "crates/nostr_automerge/src/graph/actor_state.rs":"graph::actor_state::tests::",
    "crates/nostr_automerge/src/graph/epoch.rs":"graph::epoch::tests::",
    "crates/nostr_automerge/src/control/authorize.rs":"control::authorize::tests::",
    "crates/nostr_automerge/src/graph/closure.rs":"graph::closure::tests::",
    "crates/nostr_automerge/src/graph/schedule.rs":"graph::schedule::tests::",
    "crates/nostr_automerge/src/graph/equivocation.rs":"graph::equivocation::tests::",
    "crates/nostr_automerge/src/reference/epoch_engine.rs":"reference::epoch_engine::tests::",
    "crates/nostr_automerge/src/engine/evaluation_report.rs":"engine::evaluation_report::tests::",
}

class CatalogError(RuntimeError): pass
def require(value: bool, code: str) -> None:
    if not value: raise CatalogError(code)
def sha(path: Path) -> str: return hashlib.sha256(path.read_bytes()).hexdigest()

def validate(report: object, schema: object) -> None:
    require(type(report) is dict and list(report) == FIELDS, "report:shape")
    require(report["schema"] == "nostr_automerge.remediation_v12_proof_catalog.v1" and report["status"] == report["result"] == "pass", "report:state")
    require(report["candidate"] == "7d518f3e2c057e4c265b4a66416c0eb3de25dad4", "candidate")
    inventory = json.loads(INVENTORY.read_text())
    require(report["operation_inventory_sha256"] == sha(INVENTORY), "inventory:hash")
    require(report["report_suite_identity_sha256"] == "f911bcb863106be48017734dce12d398fa66794c73d3ca7d1d692d897d42b7ca", "suite:identity")
    require(report["requirements"] == REQUIREMENTS and report["fixture_families"] == FIXTURES and report["stop_laws"] == STOPS, "coverage")
    require(report["findings"] == [
        {"id":"FINDING_100","proof":"remediation_v12_operation_inventory","status":"closed"},
        {"id":"FINDING_101","proof":"report_contract_v9","status":"open"},
        {"id":"FINDING_102","proof":"distribution_v13_parity","status":"closed"},
        {"id":"FINDING_103","proof":"distribution_v13_parity","status":"open"},
    ], "findings")
    require(report["counts"] == {"operation_proofs":len(inventory["operations"]),"report_contract_proofs":21,"finding_proofs":4,"fixture_families":7,"stop_laws":4}, "counts")
    require(len({row["test"] for row in inventory["operations"]}) == 15, "operations:tests")
    require(type(schema) is dict and schema.get("additionalProperties") is False and schema.get("required") == FIELDS, "schema")

def self_test(report: dict, schema: dict) -> int:
    cases = []
    for label, mutate in (
        ("requirement", lambda value: value["requirements"].pop()),
        ("finding", lambda value: value["findings"].pop()),
        ("finding_duplicate", lambda value: value["findings"].__setitem__(1, copy.deepcopy(value["findings"][0]))),
        ("fixture", lambda value: value["fixture_families"].pop()),
        ("fixture_order", lambda value: value["fixture_families"].reverse()),
        ("stop", lambda value: value["stop_laws"].pop()),
        ("inventory", lambda value: value.update(operation_inventory_sha256="0"*64)),
        ("suite", lambda value: value.update(report_suite_identity_sha256="0"*64)),
        ("candidate", lambda value: value.update(candidate="0"*40)),
    ):
        changed=copy.deepcopy(report); mutate(changed); cases.append((label,changed,schema))
    changed_schema=copy.deepcopy(schema); changed_schema["additionalProperties"]=True; cases.append(("schema",report,changed_schema))
    for label,changed,changed_schema in cases:
        try: validate(changed,changed_schema)
        except CatalogError: continue
        raise CatalogError("mutation_survived:"+label)
    return len(cases)

def run_suite() -> int:
    first = subprocess.run([sys.executable, "scripts/validate_report_contract_v9.py", "--run-suite"], cwd=ROOT, text=True, capture_output=True)
    require(first.returncode == 0 and "executed=21" in first.stdout, "suite:report_contract")
    inventory = json.loads(INVENTORY.read_text())
    executed = 21
    for row in inventory["operations"]:
        if row["family"] == "compatibility_evidence":
            result = subprocess.run([sys.executable, row["source_path"]], cwd=ROOT, text=True, capture_output=True)
            require(result.returncode == 0 and result.stdout.startswith("PASS:"), "suite:compatibility")
        else:
            full_test = PREFIXES[row["source_path"]] + row["test"]
            result = subprocess.run(["cargo","test","-p","nostr_automerge","--lib",full_test,"--locked","--","--exact"], cwd=ROOT, text=True, capture_output=True)
            require(result.returncode == 0 and "1 passed" in result.stdout and ("test "+full_test+" ... ok") in result.stdout, "suite:"+row["id"])
        executed += 1
    return executed

def main() -> int:
    parser=argparse.ArgumentParser(); parser.add_argument("--run-suite",action="store_true"); args=parser.parse_args()
    report=json.loads(REPORT.read_text()); schema=json.loads(SCHEMA.read_text())
    validate(report,schema); mutations=self_test(report,schema); executed=run_suite() if args.run_suite else 0
    print(f"PASS: remediation-v12 proof catalog proofs=40 mutations={mutations} executed={executed}")
    return 0

if __name__ == "__main__": raise SystemExit(main())
