#!/usr/bin/env python3
"""Validate combined public and opaque independent v18 assurance."""

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
REPORT = ROOT / "reports/causal_projection_combined_assurance_v18.json"
SCHEMA = ROOT / "tools/validation/causal_projection_combined_assurance_v18.schema.json"
CANDIDATE = "f7f198eebf2b598f4f95b9edb30f4384fc993c49"
CANONICAL = "e69c721549966b1b88dcde3296674d675169840c6e8ebd0f236a5c07bcfc6415"
SERIALIZED = "000c52bde7102eaccec8cf65c875332e119fd25ccf4a2ac38973c456774a3344"
FIELDS = [
    "schema", "status", "rcld", "candidate", "imports", "candidate_roles", "counts",
    "applicability", "identities", "finding_closure", "result_classes",
    "canonical_process_bytes", "public_api_changed", "protocol_changed",
    "release_claimed", "publication_claimed", "remote_actions", "result",
    "result_identity_sha256",
]
IMPORTS = {
    "operation_contract_sha256": "5d3ec804f238e77e54b1db490207af7c7ec89a0a8511e740bfdd3611e52eaa4e",
    "public_proofs_sha256": "f01fa208d9d7fe9df522ec2d62d4c189861e713ff070fb8e6b973944d5582946",
    "public_mutations_sha256": "da9cb0416b9008daa504b6626c78387a494a6e154571cefbae2cf00de33baf67",
    "public_catalogs_sha256": "bbb263b45b7606637fe8a4f82775a6283d4fea3257a0087afc6b3a1b7572a9e1",
    "public_final_inventory_sha256": "0bd686a2c743414a898669f88115a3e42063321142ae8093b2e9c97606e6c9e0",
    "public_evidence_graph_sha256": "48a82ead9b1baf911638651191e2592df3f6ce259077ffc77642c39d8636a9e5",
    "public_qualification_sha256": "943622deb233fe3f597f71b159b58f23349bd5c4b3f156c7306c7cf8282f3bcc",
    "distribution_transition_sha256": "1408b71c6e7ee31a99e6e0436c4ed290467675a67f517bc0be082b10149a5153",
    "opaque_import_sha256": "2aaa6d3fb214b7076b78c82ca44c73097fdc35ba2ae9d8bbd33127c6697e134c",
    "finding_registry_sha256": "6cb9021b4fa827b7a1db50b0c2fb5d2951904c634c8ff1d678b374f8621e2725",
}
ROLES = {
    "source_candidate": "076221ad7f03e67d89ac4b2fcfc8f2586b97f182",
    "execution_base_candidate": "78a5b6e381a3a921fe18f9e7fcf0eb3084e01640",
    "proof_artifact_commit": "9dda56c11e7f2376a21b0ad8c7b02105e3c9a444",
    "mutation_artifact_commit": "3e101da1c0cabb6a2c5dd99279e8c3cf9f8eb0d7",
    "final_inventory_commit": "c90d61810bcf378eee9e6577428082a31aec1b5c",
    "evidence_graph_commit": "6b73727be798e152aa3afbb98bf3683c7e52a393",
    "public_qualification_commit": "67da17714ec97418950aff44e056badbe113b456",
    "opaque_import_commit": CANDIDATE,
    "independent_assurance_commit": "5ecb65588d7b03ebdc007294d70200600d3b832c",
}
COUNTS = {
    "public_operation_sites": 68, "public_site_proofs": 68, "public_mutations": 21,
    "independent_operation_sites": 142, "independent_site_proofs": 142,
    "independent_mutations": 19, "combined_mutations": 40, "mutation_survivors": 0,
    "scenarios": 204, "signed_events": 771, "delivery_orders": 8,
    "processes_per_implementation": 2, "transition_affected": 0,
}
APPLICABILITY = {
    "shared_abstract_classes": [
        "projection_construction", "actor_sequence", "causal_counter", "frontier_comparison",
    ],
    "binding_rule": "shared_abstract_owner_language_specific_sites_and_counters",
    "independence": "separate_implementation_and_evidence_histories",
}
IDENTITIES = {
    "public_evidence_graph_identity_sha256": "d9d3e919b3beef383451200760060d6f835dc11f90073fa22252207c16e1e6ca",
    "opaque_source_identity_sha256": "d6b754335cfd0aa95a001af72ec4b10c154879a57b7a5580bec4db2418bf9372",
    "opaque_import_identity_sha256": "ace76e6cceeb6bc16294f56f1df67c18653449d94f011e1011f79d097357a803",
    "canonical_output_sha256": CANONICAL,
    "serialized_run_sha256": SERIALIZED,
}
FINDINGS = [f"FINDING_{number}" for number in range(123, 130)]
RESULT_CLASSES = [
    "exact_site_identity", "sealed_charge_target_observe",
    "trace_derived_proof_qualification", "site_local_mutation_qualification",
    "bidirectional_evidence", "cross_implementation_parity", "distribution_byte_identity",
]
BINDINGS = {
    "operation_contract_sha256": (ROLES["source_candidate"], "spec/causal_projection_contracts_v18.json"),
    "public_proofs_sha256": (ROLES["proof_artifact_commit"], "reports/causal_projection_proofs_v18.json"),
    "public_mutations_sha256": (ROLES["mutation_artifact_commit"], "reports/causal_projection_mutations_v18.json"),
    "public_catalogs_sha256": ("2f44ca464d2b39f01617e17fe7fa7f8624478c0c", "reports/causal_projection_catalogs_v18.json"),
    "public_final_inventory_sha256": (ROLES["final_inventory_commit"], "reports/causal_projection_final_inventory_v18.json"),
    "public_evidence_graph_sha256": (ROLES["evidence_graph_commit"], "reports/causal_projection_evidence_graph_v18.json"),
    "public_qualification_sha256": (ROLES["public_qualification_commit"], "reports/causal_projection_public_qualification_v18.json"),
    "distribution_transition_sha256": ("b1a960ae32aa95c4a978b401af1b46e1cd9a29a0", "spec/distribution_v18_transition.json"),
    "opaque_import_sha256": (ROLES["opaque_import_commit"], "reports/opaque_causal_projection_v18.json"),
    "finding_registry_sha256": (ROLES["opaque_import_commit"], "spec/remediation_findings_v18.json"),
}


class CombinedError(RuntimeError):
    pass


def require(condition: bool, label: str) -> None:
    if not condition:
        raise CombinedError(label)


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
        [chr(103) + "it", "show", f"{candidate}:{path}"], cwd=ROOT,
        capture_output=True, check=False,
    )
    require(result.returncode == 0, "binding:" + path)
    return hashlib.sha256(result.stdout).hexdigest()


def validate(record: dict[str, Any], schema: dict[str, Any]) -> None:
    require(list(record) == FIELDS, "shape")
    require(
        record["schema"] == "nostr_automerge.causal_projection_combined_assurance.v18.v1"
        and record["status"] == "verified" and record["rcld"] == 140
        and record["candidate"] == CANDIDATE and record["result"] == "pass",
        "state",
    )
    require(record["imports"] == IMPORTS, "imports")
    for name, (candidate, path) in BINDINGS.items():
        require(committed_sha(candidate, path) == IMPORTS[name], "binding:" + name)
    require(record["candidate_roles"] == ROLES, "roles")
    public_chain = [
        ROLES["source_candidate"], ROLES["proof_artifact_commit"],
        ROLES["mutation_artifact_commit"], ROLES["final_inventory_commit"],
        ROLES["evidence_graph_commit"], ROLES["public_qualification_commit"],
        ROLES["opaque_import_commit"],
    ]
    for parent, child in zip(public_chain, public_chain[1:]):
        result = subprocess.run(
            [chr(103) + "it", "merge-base", "--is-ancestor", parent, child], cwd=ROOT,
            capture_output=True, check=False,
        )
        require(result.returncode == 0, "ancestry:" + child)
    require(record["counts"] == COUNTS, "counts")
    require(record["applicability"] == APPLICABILITY, "applicability")
    require(record["identities"] == IDENTITIES, "identities")
    require([row["id"] for row in record["finding_closure"]] == FINDINGS, "findings")
    require(all(row["status"] == "closed" and len(row["evidence"]) == 3 for row in record["finding_closure"]), "finding:state")
    require(record["result_classes"] == RESULT_CLASSES, "classes")
    require(
        record["canonical_process_bytes"] == "identical"
        and record["public_api_changed"] is False and record["protocol_changed"] is False
        and record["release_claimed"] is False and record["publication_claimed"] is False
        and record["remote_actions"] == 0,
        "result",
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
        lambda value: value["imports"].update(opaque_import_sha256="0" * 64),
        lambda value: value["candidate_roles"].update(source_candidate="0" * 40),
        lambda value: value["counts"].update(public_operation_sites=67),
        lambda value: value["counts"].update(mutation_survivors=1),
        lambda value: value["finding_closure"].pop(),
        lambda value: value["finding_closure"][0].update(status="open"),
        lambda value: value["result_classes"].reverse(),
        lambda value: value.update(protocol_changed=True),
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
        except CombinedError:
            caught += 1
            continue
        raise CombinedError("attack:survived")
    return caught


def main() -> int:
    record, schema = load(REPORT), load(SCHEMA)
    validate(record, schema)
    attacks = self_test(record, schema)
    print(
        "PASS: causal projection combined assurance v18 "
        f"sites=68+142 proofs=68+142 mutations=40 survivors=0 scenarios=204x8x2 attacks={attacks}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
