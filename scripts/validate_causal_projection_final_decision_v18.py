#!/usr/bin/env python3
"""Validate the held-publication v18 terminal decision."""

from __future__ import annotations

import copy
import hashlib
import json
import sys
from pathlib import Path
from typing import Any, Callable

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parents[1]
REPORT = ROOT / "reports/causal_projection_final_decision_v18.json"
SCHEMA = ROOT / "tools/validation/causal_projection_final_decision_v18.schema.json"
COMPLETION = ROOT / "reports/causal_projection_completion_v18.json"
CANDIDATE = "7150c33febcd0227484af4d95b2decf1c83ef6f8"
FIELDS = [
    "schema", "status", "rcld", "candidate", "imports", "decision", "gates",
    "holds", "release_claimed", "publication_claimed", "remote_actions", "result",
    "result_identity_sha256",
]
IMPORTS = {
    "completion_sha256": "fabefed73f164db841dd587b239d374be20504c28d8c7da5aff9efa282a02e1a",
    "authority_sha256": "c45b241c6a300ab3bb3a6120ba5fac3b53cb4a3589347ed4e1edda8b17923caa",
    "finding_registry_sha256": "e7d39002597ae53c0cd1cd6a4247bd53903514682ccc709bdc11771faf51c76b",
    "runtime_ledger_sha256": "1b8ad265fd9db38f809bec0b98869df44db6e1fc988a3bcbd65c6bb0e6b78e4f",
    "combined_assurance_sha256": "44a6bdb0add32ff92dfd43d89339f5c6860ef4bc7e7e3497eb568aa36eed422a",
    "finding_closure_sha256": "deb203c98556a3cc139652becd6d8d93291db47824625fc93f32fccccdc9d4b9",
}
GATES = [
    "authority", "operation_contract", "source_inventory", "trace_proofs",
    "structural_identity", "site_local_mutations", "final_inventory", "evidence_graph",
    "public_qualification", "distribution_transition", "opaque_assurance",
    "combined_assurance", "finding_closure", "complete_specification",
]
HOLDS = [
    "external_assurance", "event_kind_allocation", "nip_submission",
    "production_qualification", "publication", "release", "remote_mutation",
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


def canonical(value: Any) -> bytes:
    return json.dumps(value, ensure_ascii=False, sort_keys=True, separators=(",", ":")).encode()


def validate(record: dict[str, Any], schema: dict[str, Any]) -> None:
    require(list(record) == FIELDS, "shape")
    require(
        record["schema"] == "nostr_automerge.causal_projection_final_decision.v18.v1"
        and record["status"] == "code_complete_publication_held"
        and record["rcld"] == 140 and record["candidate"] == CANDIDATE
        and record["result"] == "pass",
        "state",
    )
    require(record["imports"] == IMPORTS, "imports")
    require(hashlib.sha256(COMPLETION.read_bytes()).hexdigest() == IMPORTS["completion_sha256"], "completion:sha")
    completion = load(COMPLETION)
    require(completion["status"] == record["status"] and completion["result"] == "pass", "completion:state")
    require(
        record["decision"] == {
            "code_complete": True, "local_findings_closed": True, "rclds_complete": 7,
            "public_checkpoint_count": 19, "independent_checkpoint_count": 17,
            "unfinished_rclds": [], "clean_candidate_attestation": "required_later",
            "candidate_lifecycle": "acyclic_later_attestation",
        },
        "decision",
    )
    require([gate["name"] for gate in record["gates"]] == GATES, "gates:names")
    require(all(gate == {"name": name, "result": "pass"} for gate, name in zip(record["gates"], GATES)), "gates:state")
    require(
        record["holds"] == HOLDS and record["release_claimed"] is False
        and record["publication_claimed"] is False and record["remote_actions"] == 0,
        "holds",
    )
    require(
        schema.get("additionalProperties") is False and schema.get("required") == FIELDS
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
        lambda value: value["decision"].update(unfinished_rclds=[140]),
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
    for mutate in attacks:
        changed = copy.deepcopy(record)
        mutate(changed)
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
    print(f"PASS: causal projection final decision v18 gates=14 holds=7 attacks={attacks}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
