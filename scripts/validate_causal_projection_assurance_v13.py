#!/usr/bin/env python3
"""Validate the closed RCLD-118 causal projection assurance."""

from __future__ import annotations

import copy
import hashlib
import json
import subprocess
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[1]
REPORT = ROOT / "reports/causal_projection_assurance_v13.json"
SCHEMA = ROOT / "tools/validation/causal_projection_assurance_v13.schema.json"
FINDINGS = ROOT / "spec/remediation_findings_v13.json"
SCHEMA_SHA256 = "4d04605983db2ca7ec569d76284f2ffaa187327f91a60077fdde1535c062d52d"
KEYS = ("schema","status","rcld","checkpoint","candidate_chain","imports","finding_registry_sha256","findings","counts","holds","release_claimed","remote_actions","result","result_identity_sha256")
CANDIDATES = (
    ("step_1433","2bf59a8a22aff9acad87c0d5e09f37e2ebc443a6"),
    ("step_1434","4b404afaa1d3ce1775f0dbd91a283f82141f1eca"),
    ("step_1435","19e2ee7de07d02a92e9702540c80963a665d6611"),
    ("step_1436","54537099a48f79150e46a7d6ebbdab55044a4e42"),
    ("step_1437","6d6c507d86f84b25d4fb2a0c46fd48ab0cc14e4b"),
)
IMPORTS = (
    ("operation_contract","spec/causal_projection_operation_contract_v13.json","0df7119713f5f59c5bcc1cb9149b734d394acc469f2839468ee32704b23f1d3f"),
    ("authority_gate","reports/causal_projection_authority_gate_v13.json","8d4581eaf266876c3c0dafc5f2cd0c8a662931b4b176f8c4308f1ddd17d25cf0"),
    ("implementation_gate","reports/causal_projection_implementation_gate_v13.json","591df469c965427d98e0d3f56d46d968fef8c23b69c175f9696c196e168f4f44"),
    ("mutation_execution","reports/causal_projection_mutations_v13.json","4769b40515bc9f66e76aeade3ff70cf00a1fa8f070fd0b9705b3812d51793e17"),
    ("distribution_lock","fixtures/distribution/manifest_v14.lock.json","0fc414a0e49b4e87bb0cf1f21bea3cf0cd70af904720b93a95fae00f079e7304"),
    ("rust_conformance","reports/rust_conformance_v14.json","1a3788359da325ddecfa7d9d9f9c0031503b6530ed21f7998854f9c39911f7d3"),
)
EVIDENCE = {
    "FINDING_104":["operation_contract","implementation_gate","mutation_execution"],
    "FINDING_105":["operation_contract","authority_gate","implementation_gate"],
    "FINDING_106":["operation_contract","authority_gate"],
    "FINDING_107":["authority_gate"],
    "FINDING_108":["implementation_gate","mutation_execution"],
    "FINDING_109":["implementation_gate","mutation_execution"],
    "FINDING_110":["mutation_execution"],
    "FINDING_112":["distribution_lock","rust_conformance"],
}
HOLDS = ["external_assurance","event_kind_allocation","nip_submission","production_qualification","publication","release","remote_mutation"]


class AssuranceError(RuntimeError):
    pass


def require(condition: bool, label: str) -> None:
    if not condition:
        raise AssuranceError(label)


def canonical(value: Any) -> bytes:
    return json.dumps(value,ensure_ascii=False,sort_keys=True,separators=(",",":")).encode()


def validate_record(value: object) -> None:
    require(type(value) is dict and tuple(value) == KEYS,"record:shape")
    assert isinstance(value,dict)
    require(value["schema"] == "nostr_automerge.causal_projection_assurance.v13.v1" and value["status"] == "rcld_118_complete" and value["rcld"] == 118 and value["checkpoint"] == "step_1438","record:state")
    require(tuple((row.get("step"),row.get("candidate")) for row in value["candidate_chain"]) == CANDIDATES,"record:candidates")
    require(tuple((row.get("category"),row.get("path"),row.get("sha256")) for row in value["imports"]) == IMPORTS,"record:imports")
    expected_findings = [
        {"id":identifier,"status":"closed","evidence":evidence}
        for identifier,evidence in EVIDENCE.items()
    ] + [
        {"id":"FINDING_111","status":"open","evidence":[]},
        {"id":"FINDING_080","status":"held","evidence":[]},
    ]
    require(value["findings"] == expected_findings,"record:findings")
    require(value["counts"] == {"findings":10,"closed":8,"open":1,"held":1},"record:counts")
    require(value["holds"] == HOLDS and value["release_claimed"] is False and value["remote_actions"] == 0 and value["result"] == "pass","record:result")
    require(value["finding_registry_sha256"] == hashlib.sha256(FINDINGS.read_bytes()).hexdigest() == "a04492a584f87d140f15789095b79fe2b2991a987c22a334c99093c24295ad3c","record:registry")
    require(value["result_identity_sha256"] == hashlib.sha256(canonical({key:value[key] for key in KEYS[:-1]})).hexdigest(),"record:identity")


def validate_sources(value: dict[str, Any]) -> None:
    prior = "898545fddf1c40b77b7557d49ae1030a009059db"
    for step,candidate in CANDIDATES:
        result = subprocess.run(("git","rev-parse",candidate + "^"),cwd=ROOT,capture_output=True,text=True,check=False)
        require(result.returncode == 0 and result.stdout.strip() == prior,"candidate:parent:" + step)
        prior = candidate
    for category,path,expected in IMPORTS:
        require(hashlib.sha256((ROOT / path).read_bytes()).hexdigest() == expected,"import:" + category)
    registry = json.loads(FINDINGS.read_text())
    statuses = {row["id"]:row["status"] for row in registry["findings"]}
    require(all(statuses[identifier] == "closed" for identifier in EVIDENCE),"registry:closed")
    require(statuses["FINDING_111"] == "open" and statuses["FINDING_080"] == "held","registry:remaining")


def validate_schema(value: object) -> None:
    require(type(value) is dict,"schema:object")
    assert isinstance(value,dict)
    require(hashlib.sha256(SCHEMA.read_bytes()).hexdigest() == SCHEMA_SHA256,"schema:sha256")
    require(value.get("type") == "object" and value.get("additionalProperties") is False,"schema:closed")
    require(value.get("required") == list(KEYS) and tuple(value.get("properties",{})) == KEYS,"schema:shape")
    require(value["properties"]["rcld"] == {"const":118} and value["properties"]["candidate_chain"]["minItems"] == 5,"schema:counts")


def self_test(record: dict[str, Any],schema: dict[str, Any]) -> int:
    count = 0
    mutations = (
        lambda value:value.update(status="open"),
        lambda value:value["candidate_chain"].reverse(),
        lambda value:value["candidate_chain"].pop(),
        lambda value:value["imports"].reverse(),
        lambda value:value["imports"][0].update(sha256="0"*64),
        lambda value:value["findings"][0].update(status="open"),
        lambda value:value["findings"][-2].update(status="closed"),
        lambda value:value["counts"].update(closed=7),
        lambda value:value.update(release_claimed=True),
        lambda value:value.update(result_identity_sha256="0"*64),
        lambda value:value.update(extra=False),
    )
    for mutate in mutations:
        changed = copy.deepcopy(record); mutate(changed)
        try: validate_record(changed)
        except AssuranceError: count += 1; continue
        raise AssuranceError("mutation:record")
    for mutate in (lambda value:value.update(additionalProperties=True),lambda value:value["required"].pop(),lambda value:value["properties"].pop("counts")):
        changed = copy.deepcopy(schema); mutate(changed)
        try: validate_schema(changed)
        except AssuranceError: count += 1; continue
        raise AssuranceError("mutation:schema")
    return count


def main() -> int:
    record = json.loads(REPORT.read_text()); schema = json.loads(SCHEMA.read_text())
    validate_record(record); validate_sources(record); validate_schema(schema)
    mutations = self_test(record,schema)
    print(f"PASS: causal projection assurance v13 closed=8 open=1 held=1 mutations={mutations}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
