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
EVIDENCE_POLICY = ROOT / "spec/remediation_v13_evidence_policy.json"
EVIDENCE_SCHEMA = ROOT / "tools/validation/remediation_v13_evidence_policy.schema.json"
HOLDS = ["external_assurance","event_kind_allocation","nip_submission","production_qualification","publication","release","remote_mutation"]
OPEN = [f"FINDING_{value:03d}" for value in range(104, 113)]
SCOPE = ["crates/nostr_automerge/src/control/epoch_state.rs","crates/nostr_automerge/src/engine/reference_evaluator.rs","crates/nostr_automerge/src/graph/actor_state.rs","crates/nostr_automerge/src/reference/epoch_engine.rs","crates/nostr_automerge/src/reference/evaluate.rs","docs/execution/remediation_v13/ledger.md","implementation/runtime_ledger_v13.json","reports/spec_baseline.txt","scripts/validate_causal_projection_source_v13.py","scripts/validate_remediation_v13.py","tools/nostr_automerge_conformance/src/fixture_generation.rs"]
ROW_FIELDS = ["id","family","source_path","source_symbol","owner_mode","requirements","test","command","candidate","artifact_sha256","mutation"]

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

def validate(authority: object, findings: object, ledger: object, schema: object, evidence: object, evidence_schema: object) -> None:
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
    require(sha(ROOT / "spec/REPORT_CONTRACT.md") == a["frozen_sha256"]["report_contract"] == "0135f6a484388e95ac4f6fe6f8ff4ea7690c58deadcee5818257e9483c9335cf", "authority:report_contract")
    f = keys(findings,["schema","status","findings","result"],"findings")
    require(f["schema"] == "nostr_automerge.remediation_findings.v13.v1" and f["status"] == "correction_active" and f["result"] == "pass", "findings:state")
    require(type(f["findings"]) is list and [row["id"] for row in f["findings"]] == OPEN + ["FINDING_080"], "findings:order")
    require(all(row["status"] == "open" for row in f["findings"][:-1]) and f["findings"][-1]["status"] == "held", "findings:status")
    l = keys(ledger,["schema","status","authority","cursor","findings","active_checkpoint_scope","predecessors"],"ledger")
    require(l["schema"] == "nostr_automerge.runtime_ledger.v13.v1" and l["status"] == "correction_active", "ledger:state")
    require(l["authority"] == "spec/remediation_v13_authority.json", "ledger:authority")
    require(l["cursor"] == {"active_rcld":117,"active_step":"step_1429","next_step":"step_1430","last_planned_step":"step_1452","remaining_checkpoint_count":24,"remaining_rcld_count":4}, "ledger:cursor")
    require(l["findings"] == {"open":OPEN,"held":["FINDING_080"]}, "ledger:findings")
    require(l["active_checkpoint_scope"] == SCOPE, "ledger:scope")
    require(l["predecessors"] == [{"step":"step_1419","candidate":"00ef954ff2dece37119ad235638046ffaa7305d4","owner_class":"public","result":"pass"},{"step":"step_1420","candidate":"6bbc29b5fa1a0e88cf5f61d9b751181f913c928b","owner_class":"public","result":"pass"},{"step":"step_1421","candidate":"38c65ae597af0500af64b67c21fb12f7933125b0","owner_class":"public","result":"pass"},{"step":"step_1422","candidate":"2435f60145aba99cc5f96a49aadc86e162a82b06","owner_class":"public","result":"pass"},{"step":"step_1423","candidate":"285c39239ebef6dd1179d842ca29671b9cf92dfa","owner_class":"public","result":"pass"},{"step":"step_1424","candidate":"219d4844de68ee01eafb9a0bf6c55a8adac8f6db","owner_class":"public","result":"pass"},{"step":"step_1425","candidate":"fbb3fd36cc91af41c7a91327688b921518958ba3","owner_class":"public","result":"pass"},{"step":"step_1426","candidate":"e3e8c0e650f7ce0ab95023b8240ffb96aeb627e9","owner_class":"public","result":"pass"},{"step":"step_1427","candidate":"5c65022c86f3931d2df16d71b334be17cd8483ad","owner_class":"public","result":"pass"},{"step":"step_1428","candidate":"f4efd6b4bfff04a0d2cce19d61c7487421113f06","owner_class":"public","result":"pass"}], "ledger:predecessor")
    require(type(schema) is dict and schema.get("additionalProperties") is False and schema.get("required") == ["schema","status","authority","cursor","findings","active_checkpoint_scope","predecessors"], "schema")
    require("nostr_automerge_v1_multi_rcld_v13.md" in (ROOT / "AGENTS.md").read_text(), "instructions:plan")
    e = keys(evidence,["schema","status","authority","policy","requirements","owner_modes","required_row_fields","approved_roots","opaque_allowed_fields","opaque_prohibited_fields","result"],"evidence")
    require(e["schema"] == "nostr_automerge.remediation_v13_evidence_policy.v1" and e["status"] == "approved_active" and e["result"] == "pass", "evidence:state")
    require(e["policy"] == {"path":"spec/EVIDENCE_POLICY.md","sha256":"e85d423580f1959a7bbe54f6222dd8dd552300f99223f2b138c600902385d545"} and sha(ROOT / e["policy"]["path"]) == e["policy"]["sha256"], "evidence:policy")
    require(e["requirements"] == ["NCRDT-RESOURCE-017","NCRDT-RESOURCE-018","NCRDT-RESOURCE-019","NCRDT-EVIDENCE-007"], "evidence:requirements")
    require(e["owner_modes"] == ["item_metered","exact_reserved","sealed_constant_time"] and e["required_row_fields"] == ROW_FIELDS, "evidence:rows")
    require(type(evidence_schema) is dict and evidence_schema.get("additionalProperties") is False and evidence_schema.get("required") == ["schema","status","authority","policy","requirements","owner_modes","required_row_fields","approved_roots","opaque_allowed_fields","opaque_prohibited_fields","result"], "evidence:schema")
    requirements = json.loads((ROOT / "spec/requirements.json").read_text())["requirements"]
    normative = (ROOT / "spec/NORMATIVE_REQUIREMENTS.md").read_text()
    report = (ROOT / "spec/REPORT_CONTRACT.md").read_text()
    applicability = json.loads((ROOT / "spec/requirements_applicability.json").read_text())["classifications"]
    for requirement in e["requirements"][:3]:
        row = next((item for item in requirements if item["id"] == requirement), None)
        require(row is not None and row["source"] == "spec/REPORT_CONTRACT.md", "provenance:registry:" + requirement)
        require(requirement in normative and requirement in report and applicability[requirement] == "rust-and-typescript", "provenance:surfaces:" + requirement)

def self_test(authority: dict, findings: dict, ledger: dict, schema: dict, evidence: dict, evidence_schema: dict) -> int:
    cases = []
    for label, target, mutate in [
        ("wrong_candidate","authority",lambda value: value["reviewed_public"].update(candidate="0"*40)),
        ("wrong_plan","authority",lambda value: value["governing_plan"].update(sha256="0"*64)),
        ("missing_hold","authority",lambda value: value["holds"].pop()),
        ("sequence","authority",lambda value: value["active_sequence"].update(step_count=32)),
        ("finding_order","findings",lambda value: value["findings"].reverse()),
        ("premature_close","findings",lambda value: value["findings"][0].update(status="closed")),
        ("cursor","ledger",lambda value: value["cursor"].update(active_step="step_1428")),
        ("scope","ledger",lambda value: value["active_checkpoint_scope"].pop()),
        ("predecessor","ledger",lambda value: value["predecessors"][0].update(candidate="0"*40)),
        ("open_schema","schema",lambda value: value.update(additionalProperties=True)),
        ("row_order","evidence",lambda value: value["required_row_fields"].reverse()),
        ("row_missing","evidence",lambda value: value["required_row_fields"].pop()),
        ("policy_hash","evidence",lambda value: value["policy"].update(sha256="0"*64)),
        ("evidence_open_schema","evidence_schema",lambda value: value.update(additionalProperties=True)),
    ]:
        copies = {"authority":copy.deepcopy(authority),"findings":copy.deepcopy(findings),"ledger":copy.deepcopy(ledger),"schema":copy.deepcopy(schema),"evidence":copy.deepcopy(evidence),"evidence_schema":copy.deepcopy(evidence_schema)}
        mutate(copies[target])
        cases.append((label,copies))
    for label, values in cases:
        try:
            validate(values["authority"],values["findings"],values["ledger"],values["schema"],values["evidence"],values["evidence_schema"])
        except EvidenceError:
            continue
        raise EvidenceError("mutation_survived:" + label)
    return len(cases)

def main() -> int:
    authority = json.loads(AUTHORITY.read_text())
    findings = json.loads(FINDINGS.read_text())
    ledger = json.loads(LEDGER.read_text())
    schema = json.loads(SCHEMA.read_text())
    evidence = json.loads(EVIDENCE_POLICY.read_text())
    evidence_schema = json.loads(EVIDENCE_SCHEMA.read_text())
    validate(authority, findings, ledger, schema, evidence, evidence_schema)
    mutations = self_test(authority, findings, ledger, schema, evidence, evidence_schema)
    print(f"PASS: remediation-v13 authority active=step_1429 findings=9 mutations={mutations}")
    return 0

if __name__ == "__main__":
    raise SystemExit(main())
