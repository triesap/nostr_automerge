#!/usr/bin/env python3
"""Validate the terminal v15 decision while preserving every external hold."""

from __future__ import annotations

import copy
import hashlib
import json
import subprocess
import sys
from pathlib import Path
from typing import Any

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parents[1]
REPORT = ROOT / "reports/causal_projection_final_decision_v15.json"
SCHEMA = ROOT / "tools/validation/causal_projection_final_decision_v15.schema.json"
CANDIDATE = "fe816b5ce3ff5e8d7b3d319aa0bc1a3823127371"
COMBINED_CANDIDATE = "f2cf08f1477de2620a3048bfba749588b047fea7"
IMPORTS = {"authority_sha256":"063e70835b18cfda959b8153b3d5e9ade3b28fa5fb5b3311ce49c9474a157c46","finding_registry_sha256":"a43379224ec0811cbafd27fb69a82ec47bfb1914b22167157c22d944b771d202","combined_assurance_sha256":"503618a729bb9f17c858746ba110c36079cab5d4c8059e4c28542cf0d4e9cc81","opaque_assurance_sha256":"c2885e24c1042a386eb20d27c3176715c83707f009d314a8c243e7d79b91af28","rust_conformance_sha256":"7ce224864f269e2818bc837d8252e6eeca8ee299a98604d4dd2d228d5c0ea6f5","distribution_lock_sha256":"a511c18a540aaa5de5a7ef23cf6b360108a74e0e178c1e1025907ae880d78da7","operation_contract_sha256":"12dc6aca59ad0807757cc13c372b582e67bc70a7295fb741c15f5d91412ea078"}
PATHS = {"authority_sha256":"spec/remediation_v15_authority.json","finding_registry_sha256":"spec/remediation_findings_v15.json","combined_assurance_sha256":"reports/causal_projection_combined_assurance_v15.json","opaque_assurance_sha256":"reports/opaque_causal_projection_v15.json","rust_conformance_sha256":"reports/rust_conformance_v15.json","distribution_lock_sha256":"fixtures/distribution/manifest_v15.lock.json","operation_contract_sha256":"spec/causal_projection_operation_discovery_v15.json"}
COMPLETION = {"rclds":[121,122,123,124],"public_checkpoints":16,"independent_checkpoints":6,"unfinished_rclds":[],"public_candidate":CANDIDATE,"independent_assurance_candidate":"2307800f980027bbe40ffc1312dde12f94ba2174","independent_implementation_candidate":"1cbd985289ac35b9cf0f2fa3221b190ab1fb5c74","operation_families":43,"focused_proofs":86,"behavioral_mutations":22,"mutation_survivors":0,"scenarios":204,"signed_events":771,"delivery_orders":8,"processes":2,"canonical_output_sha256":"e69c721549966b1b88dcde3296674d675169840c6e8ebd0f236a5c07bcfc6415","serialized_run_sha256":"000c52bde7102eaccec8cf65c875332e119fd25ccf4a2ac38973c456774a3344"}
FINDINGS = {"closed":["FINDING_113","FINDING_114","FINDING_115"],"held":["FINDING_080"],"open":[]}
GATES = [{"name":name,"result":"pass"} for name in ["authority","source_ownership","proof_catalog","behavioral_mutations","distribution_v15","rust_conformance","opaque_assurance","combined_assurance","private_boundary","complete_specification"]]
HOLDS = ["external_assurance","event_kind_allocation","nip_submission","production_qualification","publication","release","remote_mutation"]
FIELDS = ["schema","status","checkpoint","candidate","imports","completion","findings","gates","holds","release_claimed","publication_claimed","remote_actions","result","result_identity_sha256"]
IDENTITY = "b9ddae35f1a53cf3bb85be8d0a95e0615f5e5f0472c005d3d6fd56d067260fad"


class DecisionError(RuntimeError):
    pass


def require(value: bool, label: str) -> None:
    if not value:
        raise DecisionError(label)


def sha(relative: str) -> str:
    return hashlib.sha256((ROOT / relative).read_bytes()).hexdigest()


def canonical(value: Any) -> bytes:
    return json.dumps(value, ensure_ascii=False, separators=(",", ":")).encode()


def exact_schema_record(schema: dict[str, Any], definition: str, fields: list[str]) -> bool:
    value = schema["$defs"][definition]
    return value.get("additionalProperties") is False and value.get("required") == fields and list(value.get("properties", {})) == fields


def validate_sources() -> None:
    require(all(sha(path) == IMPORTS[key] for key, path in PATHS.items()), "source:hash")
    authority = json.loads((ROOT / PATHS["authority_sha256"]).read_text())
    findings = json.loads((ROOT / PATHS["finding_registry_sha256"]).read_text())
    combined = json.loads((ROOT / PATHS["combined_assurance_sha256"]).read_text())
    opaque = json.loads((ROOT / PATHS["opaque_assurance_sha256"]).read_text())
    rust = json.loads((ROOT / PATHS["rust_conformance_sha256"]).read_text())
    lock = json.loads((ROOT / PATHS["distribution_lock_sha256"]).read_text())
    require(authority["status"] == "code_complete_publication_held" and authority["active_sequence"] == {"rcld_first":121,"rcld_last":124,"step_first":"step_1453","step_last":"step_1468","public_step_count":16,"private_step_count":6} and authority["holds"] == HOLDS, "source:authority")
    require(findings["status"] == "code_complete_publication_held" and [row["status"] for row in findings["findings"]] == ["closed","closed","closed","held"], "source:findings")
    require(combined["candidate"] == COMBINED_CANDIDATE and combined["counts"]["operation_families"] == 43 and combined["counts"]["rust_proofs"] + combined["counts"]["independent_proofs"] == 86 and combined["counts"]["combined_behavioral_mutations"] == 22 and combined["counts"]["mutation_survivors"] == 0 and combined["identities"]["canonical_output_sha256"] == COMPLETION["canonical_output_sha256"], "source:combined")
    require(opaque["independent_candidate"] == COMPLETION["independent_assurance_candidate"] and opaque["assurance"]["terminal_candidate"] == COMPLETION["independent_implementation_candidate"] and opaque["assurance"]["clean_tree"] is True, "source:opaque")
    require(rust["scenario_count"] == 204 and rust["process_count"] == 2 and rust["delivery_order_count"] == 8 and rust["canonical_process_bytes"] == "identical" and rust["canonical_output_sha256"] == COMPLETION["canonical_output_sha256"] and rust["serialized_run_sha256"] == COMPLETION["serialized_run_sha256"], "source:rust")
    require(lock["scenario_count"] == 204 and lock["result_identity_sha256"] == combined["identities"]["distribution_identity_sha256"], "source:lock")
    plan = (ROOT / authority["governing_plan"]["path"]).read_text()
    require(hashlib.sha256(plan.encode()).hexdigest() == authority["governing_plan"]["sha256"] and "Status: complete — `code_complete_publication_held`" in plan and "No RCLD in this sequence remains unfinished." in plan, "source:plan")


def validate(record: object, schema: object) -> None:
    require(type(record) is dict and list(record) == FIELDS, "record:shape")
    assert isinstance(record, dict)
    require(record["schema"] == "nostr_automerge.causal_projection_final_decision.v15.v1" and record["status"] == "code_complete_publication_held" and record["checkpoint"] == "step_1468" and record["candidate"] == CANDIDATE, "record:state")
    resolved = subprocess.run(["git","rev-parse","--verify",f"{CANDIDATE}^{{commit}}"],cwd=ROOT,capture_output=True,text=True,check=False)
    require(resolved.returncode == 0 and resolved.stdout.strip() == CANDIDATE, "record:candidate")
    require(record["imports"] == IMPORTS and record["completion"] == COMPLETION and record["findings"] == FINDINGS and record["gates"] == GATES and record["holds"] == HOLDS, "record:evidence")
    require(record["release_claimed"] is False and record["publication_claimed"] is False and record["remote_actions"] == 0 and record["result"] == "pass", "record:holds")
    projection = {key: record[key] for key in FIELDS[:-1]}
    require(record["result_identity_sha256"] == IDENTITY == hashlib.sha256(canonical(projection)).hexdigest(), "record:identity")
    require(type(schema) is dict and list(schema) == ["title","type","additionalProperties","required","properties","$defs"] and schema["additionalProperties"] is False and schema["required"] == FIELDS and list(schema["properties"]) == FIELDS, "schema:shape")
    require(exact_schema_record(schema,"imports",list(IMPORTS)) and exact_schema_record(schema,"completion",list(COMPLETION)) and exact_schema_record(schema,"findings",list(FINDINGS)) and exact_schema_record(schema,"gate",["name","result"]), "schema:nested")
    validate_sources()


def self_test(record: dict[str, Any], schema: dict[str, Any]) -> int:
    attacks = [
        lambda value: value.update(candidate="0"*40), lambda value: value["imports"].update(authority_sha256="0"*64),
        lambda value: value["completion"]["rclds"].pop(), lambda value: value["completion"].update(public_checkpoints=15),
        lambda value: value["completion"].update(independent_checkpoints=5), lambda value: value["completion"]["unfinished_rclds"].append(124),
        lambda value: value["completion"].update(operation_families=42), lambda value: value["completion"].update(mutation_survivors=1),
        lambda value: value["findings"]["closed"].pop(), lambda value: value["findings"]["held"].clear(),
        lambda value: value["gates"].reverse(), lambda value: value["gates"][0].update(result="fail"),
        lambda value: value["holds"].pop(), lambda value: value.update(release_claimed=True),
        lambda value: value.update(publication_claimed=True), lambda value: value.update(remote_actions=1),
        lambda value: value.update(result_identity_sha256="0"*64), lambda value: value.update(extra=False),
    ]
    caught = 0
    for mutate in attacks:
        changed = copy.deepcopy(record); mutate(changed)
        try: validate(changed, schema)
        except DecisionError: caught += 1; continue
        raise DecisionError("mutation:record")
    for mutate in (lambda value: value.update(additionalProperties=True), lambda value: value["required"].pop(), lambda value: value["$defs"]["completion"]["required"].pop(), lambda value: value["$defs"]["findings"].update(additionalProperties=True)):
        changed = copy.deepcopy(schema); mutate(changed)
        try: validate(record, changed)
        except DecisionError: caught += 1; continue
        raise DecisionError("mutation:schema")
    require(caught == 22, "mutation:count")
    return caught


def main() -> int:
    record = json.loads(REPORT.read_text()); schema = json.loads(SCHEMA.read_text()); validate(record, schema); mutations = self_test(record, schema)
    print(f"PASS: causal projection final decision v15 rclds=4 public=16 independent=6 unfinished=0 mutations={mutations}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
