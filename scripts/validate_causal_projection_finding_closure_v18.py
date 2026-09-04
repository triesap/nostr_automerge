#!/usr/bin/env python3
"""Validate local v18 finding closure while preserving external holds."""

from __future__ import annotations

import copy
import hashlib
import json
import sys
from pathlib import Path
from typing import Any, Callable

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parents[1]
REPORT = ROOT / "reports/causal_projection_finding_closure_v18.json"
SCHEMA = ROOT / "tools/validation/causal_projection_finding_closure_v18.schema.json"
COMBINED = ROOT / "reports/causal_projection_combined_assurance_v18.json"
CANDIDATE = "272c254868efd2e936938e22cdbd764b7f8f527b"
FIELDS = [
    "schema", "status", "rcld", "candidate", "imports", "history", "findings",
    "counts", "holds", "release_claimed", "publication_claimed", "remote_actions",
    "result", "result_identity_sha256",
]
IMPORTS = {
    "combined_assurance_sha256": "44a6bdb0add32ff92dfd43d89339f5c6860ef4bc7e7e3497eb568aa36eed422a",
    "opaque_import_sha256": "7c65ac14d8fa4a1ef72c5a26ddac50c0b6629d0839be2ca6e459348143f6d8b6",
    "evidence_graph_sha256": "48a82ead9b1baf911638651191e2592df3f6ce259077ffc77642c39d8636a9e5",
    "finding_registry_sha256": "6cb9021b4fa827b7a1db50b0c2fb5d2951904c634c8ff1d678b374f8621e2725",
    "authority_sha256": "00e15eff444c2c407ce6d4fb632ecf7939f3c18ecfa95ba2cd0dad671f01a58a",
}
HISTORY = {
    "v17_terminal_candidate": "5599e7dc8a7658a3d7edbdd189599e69b57136f1",
    "v17_clean_attestation_candidate": "8673ff8546b9e9d57218c15a4b81890d82137184",
    "relationship": "supersedes_without_rewriting_history",
}
FINDINGS = [f"FINDING_{number}" for number in range(123, 130)] + ["FINDING_080"]
HOLDS = [
    "external_assurance", "event_kind_allocation", "nip_submission",
    "production_qualification", "publication", "release", "remote_mutation",
]


class ClosureError(RuntimeError):
    pass


def require(condition: bool, label: str) -> None:
    if not condition:
        raise ClosureError(label)


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
        record["schema"] == "nostr_automerge.causal_projection_finding_closure.v18.v1"
        and record["status"] == "code_complete_publication_held"
        and record["rcld"] == 140 and record["candidate"] == CANDIDATE
        and record["result"] == "pass",
        "state",
    )
    require(record["imports"] == IMPORTS, "imports")
    require(hashlib.sha256(COMBINED.read_bytes()).hexdigest() == IMPORTS["combined_assurance_sha256"], "combined:sha")
    combined = load(COMBINED)
    require(combined["result"] == "pass" and combined["remote_actions"] == 0, "combined:state")
    require(record["history"] == HISTORY, "history")
    require([row["id"] for row in record["findings"]] == FINDINGS, "findings")
    require(all(row["status"] == "closed" for row in record["findings"][:-1]), "findings:closed")
    require(record["findings"][-1]["status"] == "held", "finding:held")
    require(record["counts"] == {"findings": 8, "closed": 7, "held": 1, "open": 0}, "counts")
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
        lambda value: value["imports"].update(combined_assurance_sha256="0" * 64),
        lambda value: value["findings"].pop(),
        lambda value: value["findings"][0].update(status="open"),
        lambda value: value["findings"][-1].update(status="closed"),
        lambda value: value["counts"].update(open=1),
        lambda value: value["holds"].pop(),
        lambda value: value.update(publication_claimed=True),
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
        except ClosureError:
            caught += 1
            continue
        raise ClosureError("attack:survived")
    return caught


def main() -> int:
    record, schema = load(REPORT), load(SCHEMA)
    validate(record, schema)
    attacks = self_test(record, schema)
    print(f"PASS: causal projection finding closure v18 findings=7 closed=7 held=1 attacks={attacks}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
