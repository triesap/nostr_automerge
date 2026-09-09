#!/usr/bin/env python3
"""Validate the v19 terminal completion before descendant attestation."""

from __future__ import annotations

import copy
import hashlib
import json
import subprocess
import sys
from pathlib import Path
from typing import Any, Callable

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parents[1]
REPORT = ROOT / "reports/causal_projection_completion_v19.json"
SCHEMA = ROOT / "tools/validation/causal_projection_completion_v19.schema.json"
BASE = "75182a7c16c88e4240bc55ca9772ffecd9a9197a"
FIELDS = [
    "schema", "status", "rcld", "base_candidate", "imports", "completion",
    "holds", "release_claimed", "publication_claimed", "remote_actions",
    "result", "result_identity_sha256",
]
PATHS = {
    "authority_sha256": "spec/remediation_v19_authority.json",
    "finding_registry_sha256": "spec/remediation_findings_v19.json",
    "runtime_ledger_sha256": "implementation/runtime_ledger_v19.json",
    "public_assurance_sha256": "reports/causal_projection_public_assurance_v19.json",
    "opaque_assurance_sha256": "reports/opaque_causal_projection_v19.json",
    "combined_assurance_sha256": "reports/causal_projection_combined_assurance_v19.json",
    "finding_closure_sha256": "reports/causal_projection_finding_closure_v19.json",
}
COMPLETION = {
    "code_complete": True,
    "local_findings_closed": True,
    "completed_rclds": [141, 142, 143, 144, 145, 146],
    "unfinished_rclds": [],
    "terminal_artifact_commit": None,
    "attestation_execution_base": None,
    "clean_candidate_attestation": "required_later",
    "candidate_lifecycle": "strict_T_E_A",
}
HOLDS = [
    "external_assurance", "event_kind_allocation", "nip_submission",
    "production_qualification", "publication", "release", "deployment",
    "remote_mutation",
]


class CompletionError(RuntimeError):
    pass


def require(value: bool, code: str) -> None:
    if not value:
        raise CompletionError(code)


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


def exact_object(schema: dict[str, Any], name: str, fields: list[str]) -> bool:
    value = schema["properties"][name]
    return (
        value.get("additionalProperties") is False
        and value.get("required") == fields
        and list(value.get("properties", {})) == fields
    )


def validate(record: dict[str, Any], schema: dict[str, Any]) -> None:
    require(list(record) == FIELDS, "shape")
    require(
        record["schema"] == "nostr_automerge.causal_projection_completion.v19.v1"
        and record["status"] == "code_complete_publication_held"
        and record["rcld"] == 146
        and record["base_candidate"] == BASE
        and record["result"] == "pass",
        "state",
    )
    require(
        subprocess.run(
            ["git", "merge-base", "--is-ancestor", BASE, "HEAD"],
            cwd=ROOT,
            capture_output=True,
            check=False,
        ).returncode
        == 0,
        "base:ancestor",
    )
    require(record["imports"] == {name: sha(path) for name, path in PATHS.items()}, "imports")
    require(record["completion"] == COMPLETION, "completion")
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
        and list(schema.get("properties", {})) == FIELDS
        and exact_object(schema, "imports", list(PATHS))
        and exact_object(schema, "completion", list(COMPLETION)),
        "schema",
    )
    require(
        record["result_identity_sha256"]
        == hashlib.sha256(canonical({key: record[key] for key in FIELDS[:-1]})).hexdigest(),
        "identity",
    )


def self_test(record: dict[str, Any], schema: dict[str, Any]) -> int:
    attacks: list[Callable[[dict[str, Any]], None]] = [
        lambda value: value.update(base_candidate="0" * 40),
        lambda value: value["imports"].update(authority_sha256="0" * 64),
        lambda value: value["completion"]["completed_rclds"].pop(),
        lambda value: value["completion"].update(unfinished_rclds=[146]),
        lambda value: value["completion"].update(terminal_artifact_commit=BASE),
        lambda value: value["completion"].update(attestation_execution_base=BASE),
        lambda value: value["completion"].update(clean_candidate_attestation="complete"),
        lambda value: value["holds"].pop(),
        lambda value: value.update(publication_claimed=True),
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
        except CompletionError:
            caught += 1
            continue
        raise CompletionError("attack:survived")
    return caught


def main() -> int:
    record, schema = load(REPORT), load(SCHEMA)
    validate(record, schema)
    attacks = self_test(record, schema)
    print(f"PASS: causal projection completion v19 rclds=6 holds=8 attacks={attacks}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
