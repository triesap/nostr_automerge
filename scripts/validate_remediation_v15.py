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
SCOPE = [
    "docs/execution/rcl/nostr_automerge_v1_multi_rcld_v15.md",
    "docs/execution/remediation_v15/baseline.md",
    "docs/execution/remediation_v15/ledger.md",
    "implementation/runtime_ledger_v15.json",
    "scripts/validate_remediation_v15.py",
    "spec/remediation_findings_v15.json",
    "spec/remediation_v15_authority.json",
    "tools/validation/runtime_ledger_v15.schema.json",
]


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
    require(l["cursor"] == {"active_rcld":121,"active_step":"step_1453","next_step":"step_1454","last_planned_step":"step_1468","remaining_checkpoint_count":15,"remaining_rcld_count":3}, "ledger:cursor")
    require(l["findings"] == {"open":["FINDING_113","FINDING_114","FINDING_115"],"held":["FINDING_080"]}, "ledger:findings")
    require(l["active_checkpoint_scope"] == SCOPE, "ledger:scope")
    require(l["predecessors"] == [{"step":"step_1452","candidate":"0612e24ffa064b6ed362698a0ffcecad17b9b511","owner_class":"public","result":"pass"}], "ledger:predecessor")
    resolved = subprocess.run(["git","rev-parse","--verify","0612e24ffa064b6ed362698a0ffcecad17b9b511^{commit}"],cwd=ROOT,capture_output=True,text=True,check=False)
    require(resolved.returncode == 0 and resolved.stdout.strip() == "0612e24ffa064b6ed362698a0ffcecad17b9b511", "ledger:predecessor_commit")
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
        ("cursor","ledger",lambda value: value["cursor"].update(next_step="step_1455")),
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
    print(f"PASS: remediation-v15 active=step_1453 next=step_1454 mutations={mutations}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
