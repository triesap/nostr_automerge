#!/usr/bin/env python3
"""Reproduce v16 provisional inventory acceptance without claiming closure."""

import copy, hashlib, json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
REPORT = ROOT / "reports/causal_projection_inventory_reproduction_v17.json"
INVENTORY = ROOT / "reports/causal_projection_operation_inventory_v16.json"
ASSURANCE = ROOT / "reports/causal_projection_rust_assurance_v16.json"
VALIDATOR = ROOT / "scripts/validate_causal_projection_rust_assurance_v16.py"

class ReproductionError(RuntimeError): pass
def require(value, code):
    if not value: raise ReproductionError(code)
def sha(path): return hashlib.sha256(path.read_bytes()).hexdigest()

def validate(report):
    inventory = json.loads(INVENTORY.read_text())
    validator = VALIDATOR.read_text()
    rows = inventory["rows"]
    expected = {
        "schema":"nostr_automerge.causal_projection_inventory_reproduction.v17.v1",
        "status":"expected_defect","source_candidate":"0a0ce4d4ee8723bbec8473f8e6c984be6aa93df1",
        "inventory_sha256":sha(INVENTORY),"assurance_sha256":sha(ASSURANCE),
        "inventory_status":inventory["status"],"row_count":len(rows),
        "planned_proof_count":sum(row["proof"].startswith("planned:") for row in rows),
        "planned_mutation_count":sum(row["mutation"].startswith("planned:") for row in rows),
        "terminal_rejects_provisional":'inventory["status"] == "final"' in validator,
        "terminal_rejects_planned":"planned:" in validator,
        "closure_evidence":False,"result":"reproduced",
    }
    require(report == expected, "report:value")
    require(expected["inventory_status"] == "provisional_complete", "inventory:status")
    require(expected["planned_proof_count"] == expected["planned_mutation_count"] == expected["row_count"] == 68, "inventory:planned")
    require(not expected["terminal_rejects_provisional"] and not expected["terminal_rejects_planned"], "terminal:acceptance")

report = json.loads(REPORT.read_text())
validate(report)
changed = copy.deepcopy(report); changed["closure_evidence"] = True
try: validate(changed)
except ReproductionError: pass
else: raise ReproductionError("mutation:survived")
print("PASS: v17 inventory reproduction rows=68 planned_proofs=68 planned_mutations=68 closure=false")
