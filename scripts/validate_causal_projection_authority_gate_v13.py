#!/usr/bin/env python3
"""Validate the closed RCLD-116 authority and proof-correction gate."""

from __future__ import annotations

import copy
import json
import subprocess
import sys
from pathlib import Path

sys.dont_write_bytecode = True

ROOT = Path(__file__).resolve().parents[1]
REPORT = ROOT / "reports/causal_projection_authority_gate_v13.json"
SCHEMA = ROOT / "tools/validation/causal_projection_authority_gate_v13.schema.json"
CONTRACT = ROOT / "spec/causal_projection_operation_contract_v13.json"
REPRODUCTIONS = ROOT / "spec/remediation_v13_reproductions.json"
FIELDS = ["schema","status","rcld","candidates","operation_families","expected_failure_reproductions","lexical_source_mutations","isolated_source_mutations","isolated_runner_mutations","behavior_findings_closed","next_step","holds","result"]
CANDIDATES = [
    {"step":"step_1420","candidate":"6bbc29b5fa1a0e88cf5f61d9b751181f913c928b"},
    {"step":"step_1421","candidate":"38c65ae597af0500af64b67c21fb12f7933125b0"},
    {"step":"step_1422","candidate":"2435f60145aba99cc5f96a49aadc86e162a82b06"},
    {"step":"step_1423","candidate":"285c39239ebef6dd1179d842ca29671b9cf92dfa"},
    {"step":"step_1424","candidate":"219d4844de68ee01eafb9a0bf6c55a8adac8f6db"},
]
HOLDS = ["external_assurance","event_kind_allocation","nip_submission","production_qualification","publication","release","remote_mutation"]


class GateError(RuntimeError):
    pass


def require(condition: bool, label: str) -> None:
    if not condition:
        raise GateError(label)


def validate(report: object, schema: object, contract: object, reproductions: object) -> None:
    require(type(report) is dict and list(report) == FIELDS, "report:shape")
    require(report["schema"] == "nostr_automerge.causal_projection_authority_gate.v13.v1" and report["status"] == "authority_proof_complete" and report["result"] == "pass", "report:state")
    require(report["rcld"] == 116 and report["candidates"] == CANDIDATES, "report:candidates")
    for index, row in enumerate(CANDIDATES):
        actual = subprocess.run(["git","rev-parse",row["candidate"]],cwd=ROOT,capture_output=True,text=True,check=False)
        require(actual.returncode == 0 and actual.stdout.strip() == row["candidate"], f"candidate:{index}")
    require(report["operation_families"] == 14 == contract["final_operation_count"] == len(contract["families"]), "report:operations")
    require(contract["status"] == "authority_frozen" and [row["proof_step"] for row in contract["families"]] == ["step_1427"] * 5 + ["step_1428"] * 5 + ["step_1429","step_1430","step_1431","step_1431"], "report:proof_owners")
    require(report["expected_failure_reproductions"] == len(reproductions["cases"]) == 2 and reproductions["status"] == "expected_failure", "report:reproductions")
    require(report["lexical_source_mutations"] == 10 and report["isolated_source_mutations"] == 1 and report["isolated_runner_mutations"] == 9, "report:mutations")
    require(report["behavior_findings_closed"] == 0 and report["next_step"] == "step_1426", "report:closure")
    require(report["holds"] == HOLDS, "report:holds")
    require(type(schema) is dict and schema.get("additionalProperties") is False and schema.get("required") == FIELDS, "schema")


def self_test(report: dict, schema: dict, contract: dict, reproductions: dict) -> int:
    mutations = []
    for target, mutate in [
        ("report",lambda value: value["candidates"].pop()),
        ("report",lambda value: value["candidates"].reverse()),
        ("report",lambda value: value["candidates"][0].update(candidate="0" * 40)),
        ("report",lambda value: value.update(operation_families=13)),
        ("report",lambda value: value.update(expected_failure_reproductions=1)),
        ("report",lambda value: value.update(lexical_source_mutations=9)),
        ("report",lambda value: value.update(isolated_source_mutations=0)),
        ("report",lambda value: value.update(isolated_runner_mutations=8)),
        ("report",lambda value: value.update(behavior_findings_closed=1)),
        ("report",lambda value: value["holds"].pop()),
        ("contract",lambda value: value["families"][0].update(proof_step="step_1430")),
        ("reproductions",lambda value: value["cases"].pop()),
        ("schema",lambda value: value.update(additionalProperties=True)),
    ]:
        values = {"report":copy.deepcopy(report),"schema":copy.deepcopy(schema),"contract":copy.deepcopy(contract),"reproductions":copy.deepcopy(reproductions)}
        mutate(values[target]); mutations.append(values)
    for values in mutations:
        try:
            validate(values["report"],values["schema"],values["contract"],values["reproductions"])
        except GateError:
            continue
        raise GateError("mutation_survived")
    return len(mutations)


def main() -> int:
    report = json.loads(REPORT.read_text())
    schema = json.loads(SCHEMA.read_text())
    contract = json.loads(CONTRACT.read_text())
    reproductions = json.loads(REPRODUCTIONS.read_text())
    validate(report,schema,contract,reproductions)
    mutations = self_test(report,schema,contract,reproductions)
    print(f"PASS: causal-projection authority gate rcld=116 families=14 mutations={mutations}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
