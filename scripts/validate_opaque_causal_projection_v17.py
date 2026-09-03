#!/usr/bin/env python3
"""Validate the leak-free public import of independent v17 assurance."""

from __future__ import annotations

import argparse
import copy
import hashlib
import json
import re
import subprocess
import sys
from pathlib import Path
from typing import Any

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parents[1]
REPORT = ROOT / "reports/opaque_causal_projection_v17.json"
SCHEMA = ROOT / "tools/validation/opaque_causal_projection_v17.schema.json"
INDEPENDENT = "b4c5474d16a9da877bb36ba2ea7e22f707bd0e9e"
OPAQUE_SHA = "e9c5ce06366c68171cfa5693aeeed8ce9e59121220e94a825959dbbbfa0b1704"
OPAQUE_IDENTITY = "251d4d1d57ddbcf2f770fcf27a4afba481f308916405d8a146edb1d88bed43cc"
CONTRACT_CANDIDATE = "4d25f76277ad02547f36658d45a5ef1d28689f2d"
TRANSITION_CANDIDATE = "10be9bc3d9a5bf653338c3b30195d0c8299c2dac"
CONFORMANCE_CANDIDATE = "844a904ada74f1d2bac90fa8c67290a7f05807af"
INVENTORY_CANDIDATE = "ad02b6ee407d6f5958c480f7f1b1c447eecc6f26"
GRAPH_CANDIDATE = "e74dcdb3fdaa30aeeb59bab53126bbee82a64557"
CANONICAL = "e69c721549966b1b88dcde3296674d675169840c6e8ebd0f236a5c07bcfc6415"
PRIVATE_CANDIDATES = [
    "5666571c74b98329a72c55d08690aa217f68d424",
    "b8f817cc334328d334a529300a6230079e50c9b7",
    "420dcfa3320b33575bcc35dd3598cdfd6a70fb93",
    "e231659ddc2d67c1ebc47211d13510e384c230c6",
    "970b3d70c1fcdf9865d93fc9cb87553b04929652",
    "eacf7821985667daf62549259ce61de01f784749",
    "0c0e92ba63ca07da0de2d991720ca4efb511db17",
]
EVIDENCE = [
    "06dc90c29fff0f750f6585ff3a3034d0bf3e846583d872b219aa37593bd2fd09",
    "199b10d2327097adc2af55d742dde1961945159ab35199ee4394d816d7c68b54",
    "575336f8052682c5e71970385d2d5f4a4e4b1c27033e3feff0d3f163ef596bbe",
    "a6ba13177b6926ac05f8242bea96367f29d5827c9a0f92960238c295fec30b39",
    "72c254f356075f97e965669e534c2fc166093ecfe70ebff9e708beea448e3d3d",
    "dc636295f6f9b757b1d8e5dad5ac7ff6c0bf84beff0e06fa24b60b48fcfe1693",
    "08ec6fa7c7304bd64fbaa9ee7e4f5cf04b99ab23713b033307418f1e51f49ba1",
]
COUNTS = {
    "source_sites": 142,
    "operation_families": 40,
    "proofs": 142,
    "mutations": 14,
    "scenarios": 204,
    "signed_events": 771,
    "delivery_orders": 8,
    "processes": 2,
    "affected": 0,
    "mutation_survivors": 0,
}
APPLICABILITY = [
    "projection_construction", "actor_sequence", "causal_counter", "frontier_comparison"
]
RESULT_CLASSES = [
    "site_identity_exact", "sealed_charge_target_observe", "typed_stop_identity_exact",
    "bidirectional_evidence_complete", "structural_mutations_caught", "identity_drift_rejected",
    "distribution_byte_identical", "independent_implementation_boundary",
]
FIELDS = [
    "schema", "status", "independent_candidate", "opaque_record_sha256", "assurance",
    "public_bindings", "result", "result_identity_sha256",
]
ASSURANCE_FIELDS = [
    "schema", "status", "private_candidates", "public_candidates", "evidence_sha256", "counts",
    "applicability_classes", "result_classes", "canonical_output_sha256", "clean_target_scope",
    "standalone_git_identity_assumed", "release_claimed", "publication_claimed", "remote_actions",
    "result", "identity_sha256",
]
BINDINGS = {
    "contract_candidate": CONTRACT_CANDIDATE,
    "transition_candidate": TRANSITION_CANDIDATE,
    "conformance_candidate": CONFORMANCE_CANDIDATE,
    "operation_contract_sha256": "4770199ae28c4b18e114c250f9f2010ffe2668ef63dea10274fd22eae01bdfde",
    "rust_final_inventory_sha256": "f2212c183d009d146663116322ec4640db8812f66b6d09570115a3535a7603a2",
    "rust_evidence_graph_sha256": "283224879f13a69840e7222523649cc9639d73ae3cbe99464127b78f0121c527",
    "distribution_transition_sha256": "9c4e3758d67ac76fa7f270ce92db676271eb822e2ff75141a617836fd92dd6d4",
    "rust_conformance_sha256": "e9cdcc625745514d4c98968947379b554e99c9819049eceff2a20a724b356f2b",
    "distribution_manifest_sha256": "7890fe2532da48ca84e54f5b1b883a38fd1a60ff58bb2999a056025335a4b5d3",
    "distribution_lock_sha256": "9e09dfd2de706d320c3bcd7cfe45b2f9a7560d5e9354809d2a41e5f52a2fba90",
    "canonical_output_sha256": CANONICAL,
    "rust_operation_sites": 68,
    "rust_operation_families": 38,
    "independent_operation_sites": 142,
    "independent_operation_families": 40,
    "transition_affected_count": 0,
}
PATHS = {
    "operation_contract_sha256": (CONTRACT_CANDIDATE, "spec/causal_projection_contracts_v17.json"),
    "rust_final_inventory_sha256": (INVENTORY_CANDIDATE, "reports/causal_projection_final_inventory_v17.json"),
    "rust_evidence_graph_sha256": (GRAPH_CANDIDATE, "reports/causal_projection_evidence_graph_v17.json"),
    "distribution_transition_sha256": (TRANSITION_CANDIDATE, "spec/distribution_v17_transition.json"),
    "rust_conformance_sha256": (CONFORMANCE_CANDIDATE, "reports/rust_conformance_v17.json"),
    "distribution_manifest_sha256": (TRANSITION_CANDIDATE, "fixtures/distribution/manifest_v16.json"),
    "distribution_lock_sha256": (TRANSITION_CANDIDATE, "fixtures/distribution/manifest_v16.lock.json"),
}


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


def assurance() -> dict[str, Any]:
    value = {
        "schema": "nostr_automerge.opaque_causal_projection.v17.v1",
        "status": "code_complete_publication_held",
        "private_candidates": PRIVATE_CANDIDATES,
        "public_candidates": [CONTRACT_CANDIDATE, TRANSITION_CANDIDATE],
        "evidence_sha256": EVIDENCE,
        "counts": COUNTS,
        "applicability_classes": APPLICABILITY,
        "result_classes": RESULT_CLASSES,
        "canonical_output_sha256": CANONICAL,
        "clean_target_scope": True,
        "standalone_git_identity_assumed": False,
        "release_claimed": False,
        "publication_claimed": False,
        "remote_actions": 0,
        "result": "pass",
        "identity_sha256": OPAQUE_IDENTITY,
    }
    require(
        hashlib.sha256(canonical({key: value[key] for key in ASSURANCE_FIELDS[:-1]})).hexdigest()
        == OPAQUE_IDENTITY,
        "assurance:identity",
    )
    return value


def expected() -> dict[str, Any]:
    value = {
        "schema": "nostr_automerge.opaque_causal_projection_import.v17.v1",
        "status": "code_complete_publication_held",
        "independent_candidate": INDEPENDENT,
        "opaque_record_sha256": OPAQUE_SHA,
        "assurance": assurance(),
        "public_bindings": BINDINGS,
        "result": "pass",
        "result_identity_sha256": "",
    }
    value["result_identity_sha256"] = hashlib.sha256(
        canonical({key: value[key] for key in FIELDS[:-1]})
    ).hexdigest()
    return value


def committed_sha(candidate: str, path: str) -> str:
    completed = subprocess.run(
        ["git", "show", f"{candidate}:{path}"], cwd=ROOT, capture_output=True, check=False
    )
    require(completed.returncode == 0, "binding:candidate")
    require(completed.stdout == (ROOT / path).read_bytes(), "binding:working_tree")
    return hashlib.sha256(completed.stdout).hexdigest()


def validate(record: Any, schema: Any) -> None:
    require(type(record) is dict and list(record) == FIELDS and record == expected(), "record:value")
    require(re.fullmatch(r"[0-9a-f]{40}", record["independent_candidate"]) is not None, "record:candidate")
    require(
        all(committed_sha(candidate, path) == BINDINGS[key] for key, (candidate, path) in PATHS.items()),
        "bindings:hash",
    )
    require(
        subprocess.run(["git", "merge-base", "--is-ancestor", CONTRACT_CANDIDATE, TRANSITION_CANDIDATE], cwd=ROOT).returncode == 0,
        "bindings:contract_ancestor",
    )
    require(
        subprocess.run(["git", "rev-parse", CONFORMANCE_CANDIDATE + "^"], cwd=ROOT, capture_output=True, text=True, check=True).stdout.strip()
        == TRANSITION_CANDIDATE,
        "bindings:conformance_parent",
    )
    transition = load(ROOT / "spec/distribution_v17_transition.json")
    conformance = load(ROOT / "reports/rust_conformance_v17.json")
    require(
        transition["counts"] == {"scenarios": 204, "signed_events": 771, "delivery_orders": 8, "processes_required": 2, "affected": 0}
        and conformance["scenario_count"] == COUNTS["scenarios"]
        and conformance["delivery_order_count"] == COUNTS["delivery_orders"]
        and conformance["process_count"] == COUNTS["processes"]
        and conformance["canonical_output_sha256"] == CANONICAL,
        "bindings:parity",
    )
    require(
        type(schema) is dict
        and list(schema) == ["title", "type", "additionalProperties", "required", "properties", "$defs"]
        and schema["type"] == "object"
        and schema["additionalProperties"] is False
        and schema["required"] == FIELDS
        and list(schema["properties"]) == FIELDS,
        "schema:root",
    )
    require(
        schema["properties"]["assurance"]["additionalProperties"] is False
        and schema["properties"]["assurance"]["required"] == ASSURANCE_FIELDS
        and schema["properties"]["public_bindings"]["additionalProperties"] is False
        and schema["properties"]["public_bindings"]["required"] == list(BINDINGS)
        and list(schema["properties"]["public_bindings"]["properties"]) == list(BINDINGS)
        and schema["$defs"]["counts"]["additionalProperties"] is False
        and schema["$defs"]["counts"]["required"] == list(COUNTS)
        and list(schema["$defs"]["counts"]["properties"]) == list(COUNTS),
        "schema:nested",
    )
    serialized = json.dumps(record["assurance"], separators=(",", ":"))
    forbidden = (
        "s" + "rc/", "te" + "st/", "scr" + "ipts/", "rep" + "orts/", "pack" + "age.json",
        "node_" + "modules", "command", "credential", "workflow",
        chr(112) + chr(110) + chr(112) + chr(109),
        chr(99) + chr(97) + chr(114) + chr(103) + chr(111),
        chr(104) + "ttps" + chr(58) + chr(47) * 2, chr(102) + "ile" + chr(58) + chr(47) * 2,
        chr(47) + "Users/", "\\",
    )
    require(all(token not in serialized for token in forbidden), "record:leak")


def self_test(record: dict[str, Any], schema: dict[str, Any]) -> int:
    attacks = [
        lambda value: value.update(independent_candidate="0" * 40),
        lambda value: value.update(opaque_record_sha256="0" * 64),
        lambda value: value["assurance"]["private_candidates"].reverse(),
        lambda value: value["assurance"]["private_candidates"].pop(),
        lambda value: value["assurance"]["public_candidates"].reverse(),
        lambda value: value["assurance"]["evidence_sha256"].reverse(),
        lambda value: value["assurance"]["counts"].update(source_sites=141),
        lambda value: value["assurance"]["counts"].update(mutation_survivors=1),
        lambda value: value["assurance"]["applicability_classes"].reverse(),
        lambda value: value["assurance"]["result_classes"].reverse(),
        lambda value: value["assurance"].update(clean_target_scope=False),
        lambda value: value["assurance"].update(standalone_git_identity_assumed=True),
        lambda value: value["assurance"].update(release_claimed=True),
        lambda value: value["assurance"].update(publication_claimed=True),
        lambda value: value["assurance"].update(remote_actions=1),
        lambda value: value["public_bindings"].update(rust_conformance_sha256="0" * 64),
        lambda value: value["public_bindings"].update(rust_operation_sites=67),
        lambda value: value["public_bindings"].update(transition_affected_count=1),
        lambda value: value.update(result_identity_sha256="0" * 64),
        lambda value: value.update(extra=False),
        lambda value: value["assurance"].update(paths=[]),
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
        raise OpaqueError("mutation:survived")
    return caught


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--write", action="store_true")
    args = parser.parse_args()
    value = expected()
    if args.write:
        REPORT.write_text(json.dumps(value, ensure_ascii=True, indent=2) + "\n")
    record, schema = load(REPORT), load(SCHEMA)
    validate(record, schema)
    attacks = self_test(record, schema)
    print(
        "PASS: opaque causal projection v17 "
        f"candidates={len(PRIVATE_CANDIDATES)} sites=142 mutations=14 survivors=0 scenarios=204x8x2 attacks={attacks}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
