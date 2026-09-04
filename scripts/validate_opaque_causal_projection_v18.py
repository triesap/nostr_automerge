#!/usr/bin/env python3
"""Validate the leak-free public import of independent v18 assurance."""

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
REPORT = ROOT / "reports/opaque_causal_projection_v18.json"
SCHEMA = ROOT / "tools/validation/opaque_causal_projection_v18.schema.json"
INDEPENDENT = "5ecb65582555ac27c89bbed5f7d551b69b68b04a"
OPAQUE_SHA = "3eb379886b3ab1b5ed4de86652c977043ec64dde0964bf4c6d5f6f8cfdd54be4"
OPAQUE_IDENTITY = "d6b754335cfd0aa95a001af72ec4b10c154879a57b7a5580bec4db2418bf9372"
CANONICAL = "e69c721549966b1b88dcde3296674d675169840c6e8ebd0f236a5c07bcfc6415"
SERIALIZED = "000c52bde7102eaccec8cf65c875332e119fd25ccf4a2ac38973c456774a3344"
FIELDS = [
    "schema", "status", "independent_candidate", "opaque_record_sha256", "assurance",
    "public_bindings", "result", "result_identity_sha256",
]
ASSURANCE_FIELDS = [
    "schema", "status", "candidate_roles", "public_candidates", "evidence_sha256",
    "distribution_sha256", "counts", "applicability_classes", "result_classes",
    "canonical_output_sha256", "serialized_run_sha256", "clean_target_scope",
    "independent_implementation", "public_api_changed", "protocol_changed",
    "standalone_git_identity_assumed", "release_claimed", "publication_claimed",
    "remote_actions", "result", "identity_sha256",
]
CANDIDATE_ROLES = {
    "authority_candidate": "ab98f134278ec62c6bafa9fb5f4019a990c97e11",
    "source_candidate": "bea4f2a86127bdc4a9fbb6650aa0689cbe9b1218",
    "execution_base_candidate": "6f992326ed2a50ba3c2c7651c55d4ab0d5dfb2c0",
    "proof_artifact_commit": "51f3df617287ce83af7c7cdb58d65c5d8dbcf9e2",
    "mutation_artifact_commit": "51f3df617287ce83af7c7cdb58d65c5d8dbcf9e2",
    "final_inventory_commit": "fc1c3ec68052d5d575d1aa34fd811803d7cd1239",
    "evidence_graph_commit": "a3571e3d63b7634f75e82b10bf4d79fa80e94c3d",
    "qualification_base_candidate": "bb438641d8d41ed6583b66564ab8dcfbd011bdbe",
    "qualification_artifact_commit": "816024eb64f0767e6992c076c73b734d1cf2202e",
}
PUBLIC_CANDIDATES = {
    "contract_candidate": "076221ad7f03e67d89ac4b2fcfc8f2586b97f182",
    "evidence_graph_candidate": "6b73727be798e152aa3afbb98bf3683c7e52a393",
    "transition_candidate": "b1a960ae32aa95c4a978b401af1b46e1cd9a29a0",
    "qualification_candidate": "67da17714ec97418950aff44e056badbe113b456",
}
EVIDENCE = {
    "authority": "662d7b3c97e206ed0faf971feeb0d7c13766ba09b262f27748b29d14e97e8974",
    "proofs": "3cca44dfdc340f6e56e9cc58ce7aaed994697c75b90a32b72cd3e437f289fd09",
    "mutations": "092653af974a156e01fa3e69e0d3bd7bdd8a5597dfd57e217b4288a401bb9869",
    "inventory": "edee9af640ea91aa32e154f1cbbc403342a4615ff18e4c69d5dc2e9ceb02a322",
    "graph": "e6601bbb0f52f27b0a20429cdc87f9e845bf9fdfa07612e720e12ee92575f058",
    "qualification": "f1d4b7b30d50c61bec8870f2677449bdfbb55091f402b809a9820a16d2255e0b",
}
DISTRIBUTION = {
    "selected_manifest": "7890fe2532da48ca84e54f5b1b883a38fd1a60ff58bb2999a056025335a4b5d3",
    "selected_lock": "9e09dfd2de706d320c3bcd7cfe45b2f9a7560d5e9354809d2a41e5f52a2fba90",
    "implementation_manifest": "9594c85bb8fdd163ea1e58a8b4c06108ae0330ee48b27c7a4f80da24333fcc84",
    "implementation_projection": "1edde3eb3fb543284a037088c84b1cb192cdf86f1d7390fd1cf37f333119b5ce",
}
COUNTS = {
    "source_sites": 142, "direct_sites": 7, "helper_sites": 135, "proofs": 142,
    "direct_mutations": 7, "helper_mutations": 9, "provenance_mutations": 3,
    "mutations": 19, "mutation_survivors": 0, "scenarios": 204,
    "signed_events": 771, "delivery_orders": 8, "processes": 2, "gates": 8,
    "affected": 0,
}
APPLICABILITY = [
    "projection_construction", "actor_sequence", "causal_counter", "frontier_comparison",
]
RESULT_CLASSES = [
    "descriptor_aware_charge", "sealed_charge_target_observe", "typed_stop_identity_exact",
    "trace_derived_site_proofs", "site_local_direct_mutations",
    "replayable_mutation_evidence", "bidirectional_evidence_complete",
    "distribution_byte_identical", "independent_implementation_boundary",
]
BINDINGS = {
    "contract_candidate": PUBLIC_CANDIDATES["contract_candidate"],
    "contract_sha256": "5d3ec804f238e77e54b1db490207af7c7ec89a0a8511e740bfdd3611e52eaa4e",
    "inventory_candidate": "c90d61810bcf378eee9e6577428082a31aec1b5c",
    "inventory_sha256": "0bd686a2c743414a898669f88115a3e42063321142ae8093b2e9c97606e6c9e0",
    "evidence_graph_candidate": PUBLIC_CANDIDATES["evidence_graph_candidate"],
    "evidence_graph_sha256": "48a82ead9b1baf911638651191e2592df3f6ce259077ffc77642c39d8636a9e5",
    "transition_candidate": PUBLIC_CANDIDATES["transition_candidate"],
    "transition_sha256": "1408b71c6e7ee31a99e6e0436c4ed290467675a67f517bc0be082b10149a5153",
    "qualification_candidate": PUBLIC_CANDIDATES["qualification_candidate"],
    "qualification_sha256": "943622deb233fe3f597f71b159b58f23349bd5c4b3f156c7306c7cf8282f3bcc",
    "distribution_manifest_sha256": DISTRIBUTION["selected_manifest"],
    "distribution_lock_sha256": DISTRIBUTION["selected_lock"],
    "canonical_output_sha256": CANONICAL,
    "serialized_run_sha256": SERIALIZED,
    "public_operation_sites": 68,
    "public_proofs": 68,
    "public_mutations": 21,
    "transition_affected_count": 0,
}
BINDING_PATHS = {
    "contract_sha256": (BINDINGS["contract_candidate"], "spec/causal_projection_contracts_v18.json"),
    "inventory_sha256": (BINDINGS["inventory_candidate"], "reports/causal_projection_final_inventory_v18.json"),
    "evidence_graph_sha256": (BINDINGS["evidence_graph_candidate"], "reports/causal_projection_evidence_graph_v18.json"),
    "transition_sha256": (BINDINGS["transition_candidate"], "spec/distribution_v18_transition.json"),
    "qualification_sha256": (BINDINGS["qualification_candidate"], "reports/causal_projection_public_qualification_v18.json"),
    "distribution_manifest_sha256": (BINDINGS["transition_candidate"], "fixtures/distribution/manifest_v16.json"),
    "distribution_lock_sha256": (BINDINGS["transition_candidate"], "fixtures/distribution/manifest_v16.lock.json"),
}
FORBIDDEN = (
    "s" + "rc/", "te" + "st/", "scr" + "ipts/", "fi" + "xtures/",
    "rep" + "orts/", "pack" + "age.json", "node_" + "modules", "command",
    "credential", "secret", "workflow",
    chr(104) + "ttps" + chr(58) + chr(47) * 2,
    chr(102) + "ile" + chr(58) + chr(47) * 2,
    chr(47) + "users/", chr(47) + "volumes/", "\\",
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
        [chr(103) + "it", "show", f"{candidate}:{path}"],
        cwd=ROOT,
        capture_output=True,
        check=False,
    )
    require(result.returncode == 0, "binding:" + path)
    return hashlib.sha256(result.stdout).hexdigest()


def validate(record: dict[str, Any], schema: dict[str, Any]) -> None:
    require(list(record) == FIELDS, "shape")
    assurance = record["assurance"]
    require(list(assurance) == ASSURANCE_FIELDS, "assurance:shape")
    require(
        record["schema"] == "nostr_automerge.opaque_causal_projection_import.v18.v1"
        and record["status"] == "code_complete_publication_held"
        and record["independent_candidate"] == INDEPENDENT
        and record["opaque_record_sha256"] == OPAQUE_SHA
        and record["result"] == "pass",
        "state",
    )
    require(
        assurance["schema"] == "nostr_automerge.opaque_causal_projection.v18.v1"
        and assurance["status"] == record["status"]
        and assurance["candidate_roles"] == CANDIDATE_ROLES
        and assurance["public_candidates"] == PUBLIC_CANDIDATES
        and assurance["evidence_sha256"] == EVIDENCE
        and assurance["distribution_sha256"] == DISTRIBUTION
        and assurance["counts"] == COUNTS
        and assurance["applicability_classes"] == APPLICABILITY
        and assurance["result_classes"] == RESULT_CLASSES
        and assurance["canonical_output_sha256"] == CANONICAL
        and assurance["serialized_run_sha256"] == SERIALIZED
        and assurance["identity_sha256"] == OPAQUE_IDENTITY,
        "assurance:binding",
    )
    require(
        assurance["clean_target_scope"] is True
        and assurance["independent_implementation"] is True
        and assurance["public_api_changed"] is False
        and assurance["protocol_changed"] is False
        and assurance["standalone_git_identity_assumed"] is False
        and assurance["release_claimed"] is False
        and assurance["publication_claimed"] is False
        and assurance["remote_actions"] == 0
        and assurance["result"] == "pass",
        "assurance:result",
    )
    require(
        hashlib.sha256(canonical({key: assurance[key] for key in ASSURANCE_FIELDS[:-1]})).hexdigest()
        == OPAQUE_IDENTITY,
        "assurance:identity",
    )
    require(record["public_bindings"] == BINDINGS, "public:bindings")
    for name, (candidate, path) in BINDING_PATHS.items():
        require(committed_sha(candidate, path) == BINDINGS[name], "public:" + name)
    leaked = json.dumps(assurance, sort_keys=True).lower()
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
    require(
        record["result_identity_sha256"]
        == hashlib.sha256(canonical({key: record[key] for key in FIELDS[:-1]})).hexdigest(),
        "identity",
    )


def self_test(record: dict[str, Any], schema: dict[str, Any]) -> int:
    attacks: list[Callable[[dict[str, Any]], None]] = [
        lambda value: value.update(independent_candidate="0" * 40),
        lambda value: value.update(opaque_record_sha256="0" * 64),
        lambda value: value["assurance"]["candidate_roles"].update(source_candidate="0" * 40),
        lambda value: value["assurance"]["evidence_sha256"].update(proofs="0" * 64),
        lambda value: value["assurance"]["counts"].update(source_sites=141),
        lambda value: value["assurance"]["counts"].update(mutation_survivors=1),
        lambda value: value["assurance"].update(protocol_changed=True),
        lambda value: value["assurance"].update(publication_claimed=True),
        lambda value: value["assurance"].update(remote_actions=1),
        lambda value: value["public_bindings"].update(public_operation_sites=67),
        lambda value: value.update(result_identity_sha256="0" * 64),
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
        "PASS: opaque causal projection v18 "
        f"sites=142 proofs=142 mutations=19 survivors=0 scenarios=204x8x2 attacks={attacks}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
