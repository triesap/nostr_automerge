#!/usr/bin/env python3
"""Validate the causal-projection follow-up authority and active cursor."""

from __future__ import annotations

import copy
import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
AUTHORITY = ROOT / "spec/remediation_v13_authority.json"
FINDINGS = ROOT / "spec/remediation_findings_v13.json"
LEDGER = ROOT / "implementation/runtime_ledger_v13.json"
SCHEMA = ROOT / "tools/validation/runtime_ledger_v13.schema.json"
PLAN = ROOT / "docs/execution/rcl/nostr_automerge_v1_multi_rcld_v13.md"
HOLDS = ["external_assurance","event_kind_allocation","nip_submission","production_qualification","publication","release","remote_mutation"]
OPEN = [f"FINDING_{value:03d}" for value in range(104, 113)]
SCOPE = ["AGENTS.md","docs/execution/rcl/nostr_automerge_v1_multi_rcld_v13.md","docs/execution/remediation_v13/baseline.md","docs/execution/remediation_v13/ledger.md","implementation/runtime_ledger_v13.json","reports/spec_baseline.txt","scripts/validate_remediation_v13.py","scripts/validate_spec.py","spec/remediation_findings_v13.json","spec/remediation_v13_authority.json","tools/nostr_automerge_xtask/src/validate.rs","tools/validation/runtime_ledger_v13.schema.json"]

class EvidenceError(RuntimeError):
    pass

def require(condition: bool, label: str) -> None:
    if not condition:
        raise EvidenceError(label)

def keys(value: object, expected: list[str], label: str) -> dict[str, object]:
    require(type(value) is dict and list(value) == expected, label + ":shape")
    return value

def sha(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()

def validate(authority: object, findings: object, ledger: object, schema: object) -> None:
    a = keys(authority,["schema","status","reviewed_public","governing_plan","historical_v12","active_sequence","frozen_sha256","holds","result"],"authority")
    require(a["schema"] == "nostr_automerge.remediation_v13_authority.v1" and a["status"] == "approved_active" and a["result"] == "pass", "authority:state")
    require(a["reviewed_public"] == {"candidate":"00ef954ff2dece37119ad235638046ffaa7305d4","tree":"c248fd7415447900cf76f3d26977317b86816bca"}, "authority:reviewed")
    require(a["governing_plan"] == {"path":"docs/execution/rcl/nostr_automerge_v1_multi_rcld_v13.md","sha256":"b021eebe8ce18e196a0aaf42cead5f96321d8ed95742ce9ac07f129e67c10be9"}, "authority:plan")
    require(sha(PLAN) == a["governing_plan"]["sha256"], "authority:plan_hash")
    require(a["historical_v12"] == {"final_decision_sha256":"b7b11ebf3bbcea30e3dbacf5b8c01f9da18485a0f453257410d1ec08383f4349","runtime_ledger_sha256":"982019a68e984f6a2de7730b0ca816b5c9ff814f02684bfdb058f4c62958c16b","status":"immutable_history"}, "authority:history")
    require(a["active_sequence"] == {"rcld_first":116,"rcld_last":120,"step_first":"step_1420","step_last":"step_1452","step_count":33}, "authority:sequence")
    require(a["holds"] == HOLDS, "authority:holds")
    require(sha(ROOT / "spec/NIP_DRAFT.md") == a["frozen_sha256"]["nip"], "authority:nip")
    require(sha(ROOT / "spec/requirements.json") == a["frozen_sha256"]["requirements"], "authority:requirements")
    require(sha(ROOT / "spec/REPORT_CONTRACT.md") == a["frozen_sha256"]["report_contract"], "authority:report_contract")
    f = keys(findings,["schema","status","findings","result"],"findings")
    require(f["schema"] == "nostr_automerge.remediation_findings.v13.v1" and f["status"] == "correction_active" and f["result"] == "pass", "findings:state")
    require(type(f["findings"]) is list and [row["id"] for row in f["findings"]] == OPEN + ["FINDING_080"], "findings:order")
    require(all(row["status"] == "open" for row in f["findings"][:-1]) and f["findings"][-1]["status"] == "held", "findings:status")
    l = keys(ledger,["schema","status","authority","cursor","findings","active_checkpoint_scope","predecessors"],"ledger")
    require(l["schema"] == "nostr_automerge.runtime_ledger.v13.v1" and l["status"] == "correction_active", "ledger:state")
    require(l["authority"] == "spec/remediation_v13_authority.json", "ledger:authority")
    require(l["cursor"] == {"active_rcld":116,"active_step":"step_1420","next_step":"step_1421","last_planned_step":"step_1452","remaining_checkpoint_count":33,"remaining_rcld_count":5}, "ledger:cursor")
    require(l["findings"] == {"open":OPEN,"held":["FINDING_080"]}, "ledger:findings")
    require(l["active_checkpoint_scope"] == SCOPE, "ledger:scope")
    require(l["predecessors"] == [{"step":"step_1419","candidate":"00ef954ff2dece37119ad235638046ffaa7305d4","owner_class":"public","result":"pass"}], "ledger:predecessor")
    require(type(schema) is dict and schema.get("additionalProperties") is False and schema.get("required") == ["schema","status","authority","cursor","findings","active_checkpoint_scope","predecessors"], "schema")
    require("nostr_automerge_v1_multi_rcld_v13.md" in (ROOT / "AGENTS.md").read_text(), "instructions:plan")

def self_test(authority: dict, findings: dict, ledger: dict, schema: dict) -> int:
    cases = []
    for label, target, mutate in [
        ("wrong_candidate","authority",lambda value: value["reviewed_public"].update(candidate="0"*40)),
        ("wrong_plan","authority",lambda value: value["governing_plan"].update(sha256="0"*64)),
        ("missing_hold","authority",lambda value: value["holds"].pop()),
        ("sequence","authority",lambda value: value["active_sequence"].update(step_count=32)),
        ("finding_order","findings",lambda value: value["findings"].reverse()),
        ("premature_close","findings",lambda value: value["findings"][0].update(status="closed")),
        ("cursor","ledger",lambda value: value["cursor"].update(active_step="step_1421")),
        ("scope","ledger",lambda value: value["active_checkpoint_scope"].pop()),
        ("predecessor","ledger",lambda value: value["predecessors"][0].update(candidate="0"*40)),
        ("open_schema","schema",lambda value: value.update(additionalProperties=True)),
    ]:
        copies = {"authority":copy.deepcopy(authority),"findings":copy.deepcopy(findings),"ledger":copy.deepcopy(ledger),"schema":copy.deepcopy(schema)}
        mutate(copies[target])
        cases.append((label,copies))
    for label, values in cases:
        try:
            validate(values["authority"],values["findings"],values["ledger"],values["schema"])
        except EvidenceError:
            continue
        raise EvidenceError("mutation_survived:" + label)
    return len(cases)

def main() -> int:
    authority = json.loads(AUTHORITY.read_text())
    findings = json.loads(FINDINGS.read_text())
    ledger = json.loads(LEDGER.read_text())
    schema = json.loads(SCHEMA.read_text())
    validate(authority, findings, ledger, schema)
    mutations = self_test(authority, findings, ledger, schema)
    print(f"PASS: remediation-v13 authority active=step_1420 findings=9 mutations={mutations}")
    return 0

if __name__ == "__main__":
    raise SystemExit(main())
