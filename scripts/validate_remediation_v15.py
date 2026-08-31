#!/usr/bin/env python3
"""Validate the active causal-projection operation-ownership authority."""

from __future__ import annotations

import copy
import hashlib
import json
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
AUTHORITY_PATH = ROOT / "spec/remediation_v15_authority.json"
FINDINGS_PATH = ROOT / "spec/remediation_findings_v15.json"
LEDGER_PATH = ROOT / "implementation/runtime_ledger_v15.json"
SCHEMA_PATH = ROOT / "tools/validation/runtime_ledger_v15.schema.json"
HOLDS = ["external_assurance","event_kind_allocation","nip_submission","production_qualification","publication","release","remote_mutation"]
STEPS = [f"step_{value}" for value in range(1453, 1469)]
STEP_SCOPES = {
    "step_1453": [
        "docs/execution/rcl/nostr_automerge_v1_multi_rcld_v15.md",
        "docs/execution/remediation_v15/baseline.md",
        "docs/execution/remediation_v15/ledger.md",
        "implementation/runtime_ledger_v15.json",
        "scripts/validate_remediation_v15.py",
        "spec/remediation_findings_v15.json",
        "spec/remediation_v15_authority.json",
        "tools/validation/runtime_ledger_v15.schema.json",
    ],
    "step_1454": [
        "crates/nostr_automerge/tests/remediation_v15_reproductions.rs",
        "docs/execution/remediation_v15/ledger.md",
        "implementation/runtime_ledger_v15.json",
        "scripts/reproduce_remediation_v15.py",
        "scripts/validate_remediation_v15.py",
        "spec/remediation_v15_reproductions.json",
    ],
    "step_1455": [
        "docs/execution/remediation_v15/ledger.md",
        "implementation/runtime_ledger_v15.json",
        "scripts/validate_causal_projection_operation_discovery_v15.py",
        "scripts/validate_remediation_v15.py",
        "spec/causal_projection_operation_discovery_v15.json",
        "tools/validation/causal_projection_operation_discovery_v15.schema.json",
    ],
    "step_1456": [
        "docs/execution/remediation_v15/ledger.md",
        "implementation/runtime_ledger_v15.json",
        "reports/causal_projection_discovery_v15.json",
        "scripts/validate_causal_projection_discovery_v15.py",
        "scripts/validate_remediation_v15.py",
        "tools/validation/causal_projection_discovery_v15.schema.json",
    ],
    "step_1457": [
        "crates/nostr_automerge/src/graph/actor_state.rs",
        "docs/execution/remediation_v15/ledger.md",
        "implementation/runtime_ledger_v15.json",
        "scripts/validate_causal_projection_discovery_v15.py",
        "scripts/validate_remediation_v15.py",
    ],
    "step_1458": [
        "crates/nostr_automerge/src/control/epoch_state.rs",
        "crates/nostr_automerge/src/engine/reference_evaluator.rs",
        "crates/nostr_automerge/src/graph/actor_state.rs",
        "crates/nostr_automerge/src/reference/epoch_engine.rs",
        "crates/nostr_automerge/src/reference/evaluate.rs",
        "crates/nostr_automerge/tests/remediation_v15_reproductions.rs",
        "docs/execution/remediation_v15/ledger.md",
        "implementation/runtime_ledger_v15.json",
        "scripts/reproduce_remediation_v15.py",
        "scripts/validate_remediation_v15.py",
        "spec/remediation_v15_reproductions.json",
    ],
    "step_1459": [
        "crates/nostr_automerge/src/control/epoch_state.rs",
        "crates/nostr_automerge/src/engine/reference_evaluator.rs",
        "crates/nostr_automerge/src/graph/actor_state.rs",
        "crates/nostr_automerge/src/reference/epoch_engine.rs",
        "crates/nostr_automerge/src/reference/evaluate.rs",
        "crates/nostr_automerge/tests/remediation_v15_reproductions.rs",
        "docs/execution/remediation_v15/ledger.md",
        "implementation/runtime_ledger_v15.json",
        "scripts/validate_remediation_v15.py",
        "spec/remediation_v15_reproductions.json",
    ],
    "step_1460": [
        "crates/nostr_automerge/src/control/epoch_state.rs",
        "crates/nostr_automerge/src/engine/reference_evaluator.rs",
        "crates/nostr_automerge/src/graph/actor_state.rs",
        "crates/nostr_automerge/src/reference/epoch_engine.rs",
        "crates/nostr_automerge/src/reference/evaluate.rs",
        "crates/nostr_automerge/tests/remediation_v15_reproductions.rs",
        "docs/execution/remediation_v15/ledger.md",
        "implementation/runtime_ledger_v15.json",
        "scripts/validate_remediation_v15.py",
        "spec/remediation_v15_reproductions.json",
    ],
    "step_1461": [
        "crates/nostr_automerge/src/graph/actor_state.rs",
        "crates/nostr_automerge/tests/remediation_v15_reproductions.rs",
        "docs/execution/remediation_v15/ledger.md",
        "implementation/runtime_ledger_v15.json",
        "reports/causal_projection_consumer_inventory_v15.json",
        "reports/spec_baseline.txt",
        "scripts/run_causal_projection_mutations_v13.py",
        "scripts/validate_causal_projection_consumer_v15.py",
        "scripts/validate_causal_projection_evidence_v14.py",
        "scripts/validate_causal_projection_source_v13.py",
        "scripts/validate_remediation_v15.py",
        "spec/remediation_v15_reproductions.json",
        "tools/validation/causal_projection_consumer_inventory_v15.schema.json",
    ],
}


class AuthorityError(RuntimeError):
    pass


def require(condition: bool, label: str) -> None:
    if not condition:
        raise AuthorityError(label)


def exact(value: object, fields: list[str], label: str) -> dict[str, object]:
    require(type(value) is dict and list(value) == fields, label + ":shape")
    return value


def sha(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def validate(authority: object, findings: object, ledger: object, schema: object) -> None:
    a = exact(authority,["schema","status","reviewed_public","governing_plan","historical_v14","active_sequence","frozen_sha256","holds","result"],"authority")
    require(a["schema"] == "nostr_automerge.remediation_v15_authority.v1" and a["status"] == "active" and a["result"] == "pass", "authority:state")
    require(a["reviewed_public"] == {"candidate":"0612e24ffa064b6ed362698a0ffcecad17b9b511","tree":"28ccf9d24ea0ae8883ee9e4e145024c8b8c20f72"}, "authority:reviewed")
    plan = ROOT / str(a["governing_plan"]["path"])
    require(a["governing_plan"] == {"path":"docs/execution/rcl/nostr_automerge_v1_multi_rcld_v15.md","sha256":"befb1c81f38a502a77dbc34e446b17c7836f7595d87102e302c579c46bc7bffe"} and sha(plan) == a["governing_plan"]["sha256"], "authority:plan")
    require(a["historical_v14"] == {"final_decision_sha256":"e344d3cbf5f4d10bc60d88a3d93da9c3f4f07c866232d7ed6a70a3103de5b3df","runtime_ledger_sha256":"01cfce72328c704d6d9a45b07fa9cd392a2a337514693d60f79678f62791ca60","combined_assurance_sha256":"d0557d5f3427b07e1edfa8b6cf2badda93b99203604995d3e058c2996b724ea3","opaque_assurance_sha256":"2afc2c53e1653f5db53309e7f506e7b08f585cb4d69ab51cfee872a30f47a881","status":"immutable_history"}, "authority:history")
    history = [("reports/causal_projection_final_decision_v14.json","final_decision_sha256"),("implementation/runtime_ledger_v13.json","runtime_ledger_sha256"),("reports/causal_projection_combined_assurance_v14.json","combined_assurance_sha256"),("reports/opaque_causal_projection_v14.json","opaque_assurance_sha256")]
    require(all(sha(ROOT / path) == a["historical_v14"][field] for path, field in history), "authority:history_hash")
    require(a["active_sequence"] == {"rcld_first":121,"rcld_last":124,"step_first":"step_1453","step_last":"step_1468","public_step_count":16,"private_step_count":6}, "authority:sequence")
    require(a["holds"] == HOLDS, "authority:holds")
    frozen = a["frozen_sha256"]
    require(sha(ROOT / "spec/NIP_DRAFT.md") == frozen["nip"] == "8262bf32cb70b7c0e46210441120652e52504fb73839641ac19dddfed840acf8", "authority:nip")
    require(sha(ROOT / "spec/requirements.json") == frozen["requirements"] == "a8926ae4610b4855294f769871e87a14dee73d05ed201419de35711a8a781974", "authority:requirements")
    require(sha(ROOT / "spec/REPORT_CONTRACT.md") == frozen["report_contract"] == "0135f6a484388e95ac4f6fe6f8ff4ea7690c58deadcee5818257e9483c9335cf", "authority:report_contract")

    f = exact(findings,["schema","status","findings","result"],"findings")
    require(f["schema"] == "nostr_automerge.remediation_findings.v15.v1" and f["status"] == "active" and f["result"] == "pass", "findings:state")
    require([row["id"] for row in f["findings"]] == ["FINDING_113","FINDING_114","FINDING_115","FINDING_080"], "findings:order")
    require([row["status"] for row in f["findings"]] == ["open","open","open","held"], "findings:status")

    l = exact(ledger,["schema","status","authority","cursor","findings","active_checkpoint_scope","predecessors"],"ledger")
    require(l["schema"] == "nostr_automerge.runtime_ledger.v15.v1" and l["status"] == "active" and l["authority"] == "spec/remediation_v15_authority.json", "ledger:state")
    cursor = l["cursor"]
    require(type(cursor) is dict and list(cursor) == ["active_rcld","active_step","next_step","last_planned_step","remaining_checkpoint_count","remaining_rcld_count"], "ledger:cursor_shape")
    active = cursor["active_step"]
    require(active in STEPS, "ledger:active")
    active_index = STEPS.index(active)
    expected_rcld = 121 if active_index < 4 else 122 if active_index < 9 else 123 if active_index < 13 else 124
    expected_next = STEPS[active_index + 1] if active_index + 1 < len(STEPS) else None
    require(cursor == {"active_rcld":expected_rcld,"active_step":active,"next_step":expected_next,"last_planned_step":"step_1468","remaining_checkpoint_count":15-active_index,"remaining_rcld_count":124-expected_rcld}, "ledger:cursor")
    require(l["findings"] == {"open":["FINDING_113","FINDING_114","FINDING_115"],"held":["FINDING_080"]}, "ledger:findings")
    scope = l["active_checkpoint_scope"]
    require(type(scope) is list and scope == STEP_SCOPES.get(active) and scope == sorted(scope) and len(scope) == len(set(scope)) and all(type(path) is str and (ROOT / path).exists() for path in scope), "ledger:scope")
    predecessors = l["predecessors"]
    require(type(predecessors) is list and len(predecessors) == active_index + 1, "ledger:predecessor_count")
    require([row["step"] for row in predecessors] == ["step_1452"] + STEPS[:active_index], "ledger:predecessor_steps")
    require(all(type(row) is dict and list(row) == ["step","candidate","owner_class","result"] and row["owner_class"] == "public" and row["result"] == "pass" for row in predecessors), "ledger:predecessor_rows")
    require(predecessors[0]["candidate"] == "0612e24ffa064b6ed362698a0ffcecad17b9b511", "ledger:base")
    for index, row in enumerate(predecessors):
        resolved = subprocess.run(["git","rev-parse","--verify",f"{row['candidate']}^{{commit}}"],cwd=ROOT,capture_output=True,text=True,check=False)
        require(resolved.returncode == 0 and resolved.stdout.strip() == row["candidate"], f"ledger:predecessor_commit:{index}")
        if index:
            parent = subprocess.run(["git","rev-parse",f"{row['candidate']}^"],cwd=ROOT,capture_output=True,text=True,check=True).stdout.strip()
            require(parent == predecessors[index-1]["candidate"], f"ledger:predecessor_parent:{index}")
    require(type(schema) is dict and schema.get("additionalProperties") is False and schema.get("required") == ["schema","status","authority","cursor","findings","active_checkpoint_scope","predecessors"], "schema")


def self_test(authority: dict, findings: dict, ledger: dict, schema: dict) -> int:
    cases = [
        ("candidate","authority",lambda value: value["reviewed_public"].update(candidate="0"*40)),
        ("tree","authority",lambda value: value["reviewed_public"].update(tree="0"*40)),
        ("plan","authority",lambda value: value["governing_plan"].update(sha256="0"*64)),
        ("history","authority",lambda value: value["historical_v14"].update(final_decision_sha256="0"*64)),
        ("hold","authority",lambda value: value["holds"].pop()),
        ("sequence","authority",lambda value: value["active_sequence"].update(public_step_count=15)),
        ("finding_order","findings",lambda value: value["findings"].reverse()),
        ("premature_close","findings",lambda value: value["findings"][0].update(status="closed")),
        ("cursor","ledger",lambda value: value["cursor"].update(next_step="step_1463")),
        ("scope","ledger",lambda value: value["active_checkpoint_scope"].pop()),
        ("predecessor","ledger",lambda value: value["predecessors"][0].update(candidate="0"*40)),
        ("schema","schema",lambda value: value.update(additionalProperties=True)),
    ]
    caught = 0
    for label, target, mutate in cases:
        values = {"authority":copy.deepcopy(authority),"findings":copy.deepcopy(findings),"ledger":copy.deepcopy(ledger),"schema":copy.deepcopy(schema)}
        mutate(values[target])
        try:
            validate(values["authority"],values["findings"],values["ledger"],values["schema"])
        except AuthorityError:
            caught += 1
            continue
        raise AuthorityError("mutation_survived:" + label)
    return caught


def main() -> int:
    authority = json.loads(AUTHORITY_PATH.read_text())
    findings = json.loads(FINDINGS_PATH.read_text())
    ledger = json.loads(LEDGER_PATH.read_text())
    schema = json.loads(SCHEMA_PATH.read_text())
    validate(authority,findings,ledger,schema)
    mutations = self_test(authority,findings,ledger,schema)
    print(f"PASS: remediation-v15 active={ledger['cursor']['active_step']} next={ledger['cursor']['next_step']} mutations={mutations}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
