#!/usr/bin/env python3
"""Validate the held-publication v19 terminal decision."""

from __future__ import annotations

import copy
import hashlib
import json
import sys
from pathlib import Path
from typing import Any, Callable

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parents[1]
REPORT = ROOT / "reports/causal_projection_final_decision_v19.json"
SCHEMA = ROOT / "tools/validation/causal_projection_final_decision_v19.schema.json"
COMPLETION = ROOT / "reports/causal_projection_completion_v19.json"
BASE = "75182a7c16c88e4240bc55ca9772ffecd9a9197a"
FIELDS = [
    "schema", "status", "rcld", "base_candidate", "imports", "decision",
    "gates", "holds", "release_claimed", "publication_claimed",
    "remote_actions", "result", "result_identity_sha256",
]
PATHS = {
    "completion_sha256": "reports/causal_projection_completion_v19.json",
    "authority_sha256": "spec/remediation_v19_authority.json",
    "finding_registry_sha256": "spec/remediation_findings_v19.json",
    "runtime_ledger_sha256": "implementation/runtime_ledger_v19.json",
    "combined_assurance_sha256": "reports/causal_projection_combined_assurance_v19.json",
    "finding_closure_sha256": "reports/causal_projection_finding_closure_v19.json",
}
DECISION = {
    "code_complete": True,
    "local_findings_closed": True,
    "rclds_complete": 6,
    "unfinished_rclds": [],
    "terminal_artifact_commit": None,
    "attestation_execution_base": None,
    "clean_candidate_attestation": "required_later",
    "candidate_lifecycle": "strict_T_E_A",
}
GATES = [
    "authority", "source_inventory", "independent_trace_proofs",
    "runtime_mutations", "final_inventory", "evidence_graph",
    "execution_mapping", "public_qualification", "private_qualification",
    "distribution_transition", "opaque_assurance", "combined_assurance",
    "finding_closure", "complete_specification",
]
HOLDS = [
    "external_assurance", "event_kind_allocation", "nip_submission",
    "production_qualification", "publication", "release", "deployment",
    "remote_mutation",
]


class DecisionError(RuntimeError):
    pass


def require(value: bool, code: str) -> None:
    if not value:
        raise DecisionError(code)


def load(path: Path) -> Any:
    def closed(pairs: list[tuple[str, Any]]) -> dict[str, Any]:
        keys = [key for key, _ in pairs]
        require(len(keys) == len(set(keys)), "duplicate:" + path.name)
        return dict(pairs)

    return json.loads(path.read_text(), object_pairs_hook=closed)


def canonical(value: Any) -> bytes:
    return json.dumps(
        value, ensure_ascii=False, sort_keys=True, separators=(",", ":")
    ).encode()


def sha(path: str) -> str:
    return hashlib.sha256((ROOT / path).read_bytes()).hexdigest()


def validate(record: dict[str, Any], schema: dict[str, Any]) -> None:
    require(list(record) == FIELDS, "shape")
    require(
        record["schema"] == "nostr_automerge.causal_projection_final_decision.v19.v1"
        and record["status"] == "code_complete_publication_held"
        and record["rcld"] == 146
        and record["base_candidate"] == BASE
        and record["result"] == "pass",
        "state",
    )
    require(record["imports"] == {name: sha(path) for name, path in PATHS.items()}, "imports")
    completion = load(COMPLETION)
    require(completion["status"] == record["status"] and completion["result"] == "pass", "completion")
    require(record["decision"] == DECISION, "decision")
    require(
        record["gates"] == [{"name": name, "result": "pass"} for name in GATES],
        "gates",
    )
    require(
        record["holds"] == HOLDS
        and record["release_claimed"] is False
        and record["publication_claimed"] is False
        and record["remote_actions"] == 0,
        "holds",
    )
    require(
        schema.get("additionalProperties") is False
        and schema.get("required") == FIELDS
        and list(schema.get("properties", {})) == FIELDS,
        "schema",
    )
    require(
        record["result_identity_sha256"]
        == hashlib.sha256(canonical({key: record[key] for key in FIELDS[:-1]})).hexdigest(),
        "identity",
    )


def self_test(record: dict[str, Any], schema: dict[str, Any]) -> int:
    attacks: list[Callable[[dict[str, Any]], None]] = [
        lambda value: value["imports"].update(completion_sha256="0" * 64),
        lambda value: value["decision"].update(code_complete=False),
        lambda value: value["decision"].update(unfinished_rclds=[146]),
        lambda value: value["decision"].update(terminal_artifact_commit=BASE),
        lambda value: value["decision"].update(attestation_execution_base=BASE),
        lambda value: value["decision"].update(clean_candidate_attestation="complete"),
        lambda value: value["gates"].pop(),
        lambda value: value["gates"][0].update(result="fail"),
        lambda value: value["holds"].pop(),
        lambda value: value.update(release_claimed=True),
        lambda value: value.update(remote_actions=1),
        lambda value: value.update(result_identity_sha256="0" * 64),
        lambda value: value.update(extra=False),
    ]
    caught = 0
    for attack in attacks:
        changed = copy.deepcopy(record)
        attack(changed)
        try:
            validate(changed, schema)
        except DecisionError:
            caught += 1
            continue
        raise DecisionError("attack:survived")
    return caught


def main() -> int:
    record, schema = load(REPORT), load(SCHEMA)
    validate(record, schema)
    attacks = self_test(record, schema)
    print(f"PASS: causal projection final decision v19 gates=14 holds=8 attacks={attacks}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
