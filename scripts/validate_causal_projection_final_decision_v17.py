#!/usr/bin/env python3
"""Validate the terminal v17 decision and its acyclic attestation requirement."""

from __future__ import annotations

import argparse
import copy
import hashlib
import json
import subprocess
import sys
from pathlib import Path
from typing import Any

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parents[1]
REPORT = ROOT / "reports/causal_projection_final_decision_v17.json"
SCHEMA = ROOT / "tools/validation/causal_projection_final_decision_v17.schema.json"
CANDIDATE = "07479bee4fc75ac809e75588ca2bb568b35b38e4"
IMPORTS = {
    "completion_sha256": "1dc1801cc6cfe0d6e8de0c1d30238eb87edcc9f3093a561f17531d45c4aac8a4",
    "authority_sha256": "bfe0e02ad8708b0869d76eb736dfde48a3894a63cfaac2a7fcbd054ce88807bd",
    "finding_registry_sha256": "c00de5c6cfbf4ac768f77b3351d50bf4f0c283ff8ee3d665d36bafc8c06f3704",
    "runtime_ledger_sha256": "9363f464367f5c930801dd3b168b436942de96eb0bb9498598fa6147ade8b16d",
    "combined_assurance_sha256": "271463576243408ea9d43fb9f1b5c4b904c2ae63b1342871aea72b50f913b508",
    "finding_closure_sha256": "2907785eaf8b7fd01a6c1b2f306b227d039740ca6d99df587c87d78fa6984997",
}
PATHS = {
    "completion_sha256": "reports/causal_projection_completion_v17.json",
    "authority_sha256": "spec/remediation_v17_authority.json",
    "finding_registry_sha256": "spec/remediation_findings_v17.json",
    "runtime_ledger_sha256": "implementation/runtime_ledger_v17.json",
    "combined_assurance_sha256": "reports/causal_projection_combined_assurance_v17.json",
    "finding_closure_sha256": "reports/causal_projection_finding_closure_v17.json",
}
DECISION = {
    "code_complete": True,
    "local_findings_closed": True,
    "public_checkpoint_count": 31,
    "independent_checkpoint_count": 7,
    "unfinished_rclds": [],
    "clean_candidate_attestation": "required_later",
    "candidate_lifecycle": "acyclic_later_attestation",
}
GATES = [
    {"name": name, "result": "pass"}
    for name in [
        "authority", "operation_contract", "source_inventory", "proof_catalog",
        "structural_identity", "behavioral_mutations", "final_inventory",
        "evidence_graph", "public_assurance", "distribution_transition",
        "rust_conformance", "opaque_assurance", "combined_assurance",
        "finding_closure", "complete_specification",
    ]
]
HOLDS = [
    "external_assurance", "event_kind_allocation", "nip_submission", "production_qualification",
    "publication", "release", "remote_mutation",
]
FIELDS = [
    "schema", "status", "checkpoint", "candidate", "imports", "decision", "gates",
    "holds", "release_claimed", "publication_claimed", "remote_actions", "result",
    "result_identity_sha256",
]


class DecisionError(RuntimeError):
    pass


def require(condition: bool, label: str) -> None:
    if not condition:
        raise DecisionError(label)


def load(path: Path) -> Any:
    def closed(pairs: list[tuple[str, Any]]) -> dict[str, Any]:
        keys = [key for key, _ in pairs]
        require(len(keys) == len(set(keys)), "duplicate:" + path.name)
        return dict(pairs)
    return json.loads(path.read_text(), object_pairs_hook=closed)


def sha(path: str) -> str:
    return hashlib.sha256((ROOT / path).read_bytes()).hexdigest()


def canonical(value: Any) -> bytes:
    return json.dumps(value, ensure_ascii=False, sort_keys=True, separators=(",", ":")).encode()


def expected() -> dict[str, Any]:
    value = {
        "schema": "nostr_automerge.causal_projection_final_decision.v17.v1",
        "status": "code_complete_publication_held",
        "checkpoint": "step_1513",
        "candidate": CANDIDATE,
        "imports": IMPORTS,
        "decision": DECISION,
        "gates": GATES,
        "holds": HOLDS,
        "release_claimed": False,
        "publication_claimed": False,
        "remote_actions": 0,
        "result": "pass",
        "result_identity_sha256": "",
    }
    value["result_identity_sha256"] = hashlib.sha256(canonical({key: value[key] for key in FIELDS[:-1]})).hexdigest()
    return value


def exact_record(schema: dict[str, Any], name: str, fields: list[str]) -> bool:
    value = schema["$defs"][name]
    return value.get("additionalProperties") is False and value.get("required") == fields and list(value.get("properties", {})) == fields


def validate(record: Any, schema: Any) -> None:
    require(type(record) is dict and list(record) == FIELDS and record == expected(), "record:value")
    resolved = subprocess.run(["git", "rev-parse", "--verify", CANDIDATE + "^{commit}"], cwd=ROOT, capture_output=True, text=True, check=False)
    require(resolved.returncode == 0 and resolved.stdout.strip() == CANDIDATE, "record:candidate")
    require(all(sha(path) == IMPORTS[key] for key, path in PATHS.items()), "record:imports")
    completion = load(ROOT / PATHS["completion_sha256"])
    authority = load(ROOT / PATHS["authority_sha256"])
    findings = load(ROOT / PATHS["finding_registry_sha256"])
    ledger = load(ROOT / PATHS["runtime_ledger_sha256"])
    require(completion["sequence"]["unfinished_rclds"] == [] and completion["findings"]["open"] == [] and completion["next_checkpoint"] == "clean_candidate_attestation", "source:completion")
    require(authority["approved_decisions"]["candidate_lifecycle"] == "acyclic_later_attestation" and authority["holds"] == HOLDS, "source:authority")
    require([row["status"] for row in findings["findings"]] == ["closed", "closed", "closed", "closed", "held"], "source:findings")
    require(ledger["cursor"]["next_step"] is None and ledger["cursor"]["remaining_rcld_count"] == 0, "source:ledger")
    require(type(schema) is dict and schema.get("additionalProperties") is False and schema.get("required") == FIELDS and list(schema.get("properties", {})) == FIELDS, "schema:root")
    require(exact_record(schema, "imports", list(IMPORTS)) and exact_record(schema, "decision", list(DECISION)) and exact_record(schema, "gate", ["name", "result"]), "schema:nested")


def self_test(record: dict[str, Any], schema: dict[str, Any]) -> int:
    attacks = [
        ("record", lambda value: value["decision"].update(code_complete=False)),
        ("record", lambda value: value["decision"].update(public_checkpoint_count=30)),
        ("record", lambda value: value["decision"]["unfinished_rclds"].append(133)),
        ("record", lambda value: value["decision"].update(clean_candidate_attestation="self_attested")),
        ("record", lambda value: value["gates"][0].update(result="fail")),
        ("record", lambda value: value["holds"].pop()),
        ("record", lambda value: value.update(publication_claimed=True)),
        ("record", lambda value: value.update(remote_actions=1)),
        ("record", lambda value: value.update(result_identity_sha256="0" * 64)),
        ("schema", lambda value: value.update(additionalProperties=True)),
    ]
    caught = 0
    for target, mutate in attacks:
        changed_record, changed_schema = copy.deepcopy(record), copy.deepcopy(schema)
        mutate(changed_record if target == "record" else changed_schema)
        try:
            validate(changed_record, changed_schema)
        except DecisionError:
            caught += 1
            continue
        raise DecisionError("mutation:survived")
    return caught


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--write", action="store_true")
    args = parser.parse_args()
    if args.write:
        REPORT.write_text(json.dumps(expected(), ensure_ascii=True, indent=2) + "\n")
    record, schema = load(REPORT), load(SCHEMA)
    validate(record, schema)
    attacks = self_test(record, schema)
    print(f"PASS: causal projection final decision v17 public=31 independent=7 findings=0 attacks={attacks}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
