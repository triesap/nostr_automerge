#!/usr/bin/env python3
"""Validate the leak-free v19 independent assurance import."""

from __future__ import annotations

import copy
import hashlib
import json
import re
import subprocess
import sys
from pathlib import Path
from typing import Any, Callable

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parents[1]
REPORT = ROOT / "reports/opaque_causal_projection_v19.json"
SCHEMA = ROOT / "tools/validation/opaque_causal_projection_v19.schema.json"
OPAQUE_SHA256 = "53bd0478aa66ad290cc787d865b77c58921e994bda5ec9c8c292227099ea71cb"
IDENTITY = "5f3dca11be0745a820a84e6ebddcd6f65e69bd0013b50321246c951051b0d56c"
CANONICAL = "e69c721549966b1b88dcde3296674d675169840c6e8ebd0f236a5c07bcfc6415"
SERIALIZED = "000c52bde7102eaccec8cf65c875332e119fd25ccf4a2ac38973c456774a3344"
FIELDS = [
    "schema", "status", "candidate_roles", "public_candidates", "evidence_sha256",
    "distribution_sha256", "counts", "applicability_classes", "result_classes",
    "canonical_output_sha256", "serialized_run_sha256", "clean_target_scope",
    "independent_implementation", "public_api_changed", "protocol_changed",
    "release_claimed", "publication_claimed", "remote_actions", "result",
    "identity_sha256",
]
COUNTS = {
    "source_sites": 142, "proofs": 142, "proof_runs": 710,
    "helper_mutations": 7, "abstract_helper_cells": 28, "direct_mutations": 7,
    "provenance_mutations": 5, "mutations": 19, "mutation_edges": 1711,
    "mutation_survivors": 0, "scenarios": 204, "signed_events": 771,
    "delivery_orders": 8, "repetitions": 2, "processes_per_repetition": 2,
    "processes": 4, "gates": 8, "gate_executions": 16, "affected": 0,
}
PUBLIC = {
    "contract_candidate": "a8f8651c26a0ac7822e1e5d61f3aafa3fec53d1f",
    "evidence_graph_candidate": "ac24566ae4b96bc1ac17b0c87b7ae70cbb9f939f",
    "transition_candidate": "53f0bf69809cfdf2c59393dc9c3f696940ff7554",
    "qualification_candidate": "0735f7e23524764c889f4ec0cc9942c9e795740a",
    "public_assurance_candidate": "156194f6453bd4f31da4eda512cf51361ec7c978",
}
PUBLIC_BINDINGS = {
    "contract_candidate": ("spec/causal_projection_contracts_v19.json", "6cce057c9e6002156628a93b6a5dd19bd6194a4143ff9bc9362cdfde7f262c6b"),
    "evidence_graph_candidate": ("reports/causal_projection_evidence_graph_v19.json", "0b0c5cd2e54805cd888bcb04306b921f7e31737b5524282f1d57b5dba7a6d4f3"),
    "transition_candidate": ("spec/distribution_v19_transition.json", "e2cf7090b4d901deeeb77fe6f59d3a5ceaeee43ae397e92de001101ee5cd9aa4"),
    "qualification_candidate": ("reports/causal_projection_public_qualification_v19.json", "a66fb0b6dbbeeee68dd5efcb9e8223957d9baaa380c2b3c332fc23acadc0c65f"),
    "public_assurance_candidate": ("reports/causal_projection_public_assurance_v19.json", "e7494864e1818032ccbcaedbb3829788c87a8ec3ced03abb75db64cabe427b0b"),
}
APPLICABILITY = [
    "projection_construction", "actor_sequence", "causal_counter", "frontier_comparison",
]
RESULT_CLASSES = [
    "independent_charge_target_return_completion", "typed_stop_identity_exact",
    "source_derived_site_proofs", "one_helper_seven_class_matrix",
    "seven_site_local_direct_mutations", "five_independent_provenance_mutations",
    "behavior_derived_properties", "bidirectional_evidence_complete",
    "double_compatibility_qualification", "distribution_byte_identical",
    "independent_implementation_boundary",
]
FORBIDDEN = (
    "src/", "test/", "scripts/", "fixtures/", "reports/", "package.json",
    "node_modules", "pnpm", "cargo", "command", "credential", "secret", "workflow",
    "http://", "https://", "file://", "/users/", "/volumes/", "\\",
)


class OpaqueError(RuntimeError):
    pass


def require(condition: bool, label: str) -> None:
    if not condition:
        raise OpaqueError(label)


def load(path: Path) -> Any:
    def closed(pairs: list[tuple[str, Any]]) -> dict[str, Any]:
        keys = [key for key, _ in pairs]
        require(len(keys) == len(set(keys)), "duplicate:" + path.name)
        return dict(pairs)
    return json.loads(path.read_text(), object_pairs_hook=closed)


def canonical(value: Any) -> bytes:
    return json.dumps(value, ensure_ascii=False, sort_keys=True, separators=(",", ":")).encode()


def committed_sha(candidate: str, path: str) -> str:
    result = subprocess.run(
        ["git", "show", f"{candidate}:{path}"], cwd=ROOT, capture_output=True, check=False,
    )
    require(result.returncode == 0, "binding:" + path)
    return hashlib.sha256(result.stdout).hexdigest()


def validate(record: dict[str, Any], schema: dict[str, Any]) -> None:
    require(list(record) == FIELDS, "shape")
    require(
        record["schema"] == "nostr_automerge.opaque_causal_projection.v19.v1"
        and record["status"] == "code_complete_publication_held"
        and record["public_candidates"] == PUBLIC
        and record["counts"] == COUNTS
        and record["applicability_classes"] == APPLICABILITY
        and record["result_classes"] == RESULT_CLASSES
        and record["canonical_output_sha256"] == CANONICAL
        and record["serialized_run_sha256"] == SERIALIZED
        and record["identity_sha256"] == IDENTITY,
        "binding",
    )
    require(
        record["clean_target_scope"] is True
        and record["independent_implementation"] is True
        and record["public_api_changed"] is False
        and record["protocol_changed"] is False
        and record["release_claimed"] is False
        and record["publication_claimed"] is False
        and record["remote_actions"] == 0
        and record["result"] == "pass",
        "result",
    )
    body = {key: record[key] for key in FIELDS[:-1]}
    require(hashlib.sha256(canonical(body)).hexdigest() == IDENTITY, "identity")
    require(hashlib.sha256(REPORT.read_bytes()).hexdigest() == OPAQUE_SHA256, "opaque:bytes")
    for role, (path, expected) in PUBLIC_BINDINGS.items():
        require(committed_sha(PUBLIC[role], path) == expected, "public:" + role)
    chain = [PUBLIC[role] for role in (
        "contract_candidate", "evidence_graph_candidate", "qualification_candidate",
        "public_assurance_candidate",
    )]
    for parent, child in zip(chain, chain[1:]):
        result = subprocess.run(
            ["git", "merge-base", "--is-ancestor", parent, child],
            cwd=ROOT, capture_output=True, check=False,
        )
        require(result.returncode == 0, "ancestry:" + child)
    leaked = json.dumps(record, sort_keys=True).lower()
    for token in FORBIDDEN:
        require(token not in leaked, "leak:" + token)
    require(re.search(r"(?:[a-z]:)?/[a-z0-9_.-]+/", leaked) is None, "leak:path")
    require(
        schema.get("type") == "object"
        and schema.get("additionalProperties") is False
        and schema.get("required") == FIELDS
        and list(schema.get("properties", {})) == FIELDS,
        "schema",
    )


def self_test(record: dict[str, Any], schema: dict[str, Any]) -> int:
    attacks: list[Callable[[dict[str, Any]], None]] = [
        lambda value: value["candidate_roles"].update(source_candidate="0" * 40),
        lambda value: value["public_candidates"].update(contract_candidate="0" * 40),
        lambda value: value["evidence_sha256"].update(proofs="0" * 64),
        lambda value: value["counts"].update(source_sites=141),
        lambda value: value["counts"].update(mutation_survivors=1),
        lambda value: value["counts"].update(repetitions=1),
        lambda value: value["result_classes"].reverse(),
        lambda value: value.update(clean_target_scope=False),
        lambda value: value.update(protocol_changed=True),
        lambda value: value.update(publication_claimed=True),
        lambda value: value.update(remote_actions=1),
        lambda value: value.update(identity_sha256="0" * 64),
        lambda value: value.update(extra=False),
    ]
    caught = 0
    for mutate in attacks:
        changed = copy.deepcopy(record)
        mutate(changed)
        try:
            validate(changed, schema)
        except OpaqueError:
            caught += 1
            continue
        raise OpaqueError("attack:survived")
    return caught


def main() -> int:
    record, schema = load(REPORT), load(SCHEMA)
    validate(record, schema)
    attacks = self_test(record, schema)
    print(
        "PASS: opaque causal projection v19 "
        f"sites=142 proofs=142 mutations=19 survivors=0 qualification=2x204x8x2 attacks={attacks}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
