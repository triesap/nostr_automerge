#!/usr/bin/env python3
"""Validate the public import of the independent opaque projection assurance."""

from __future__ import annotations

import copy
import hashlib
import json
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[1]
REPORT = ROOT / "reports/opaque_causal_projection_v14.json"
SCHEMA = ROOT / "tools/validation/opaque_causal_projection_v14.schema.json"
FIELDS = (
    "schema",
    "status",
    "independent_candidate",
    "opaque_record_sha256",
    "assurance",
    "public_bindings",
    "result",
    "result_identity_sha256",
)
CANDIDATES = (
    "e94bfa4d96b0180037f7f6a81546ceb2ac1aa3ef",
    "08e9037037083099231d05051f75079af28e7071",
    "ddc39dbca35d0f735d5cee6cad84112cbf6ac37a",
    "a465425cd3b41a03260e741472f3ebf0aeb23d54",
    "06ab6bae2703215a29dbdf8f72f244c0732f16b3",
    "5968197f1342d65fabd5d0e4c8215ed4bbe8aa55",
)
COUNTS = {
    "operation_families": 14,
    "proofs": 14,
    "executed_mutations": 17,
    "mutation_survivors": 0,
    "scenarios": 204,
    "signed_events": 771,
    "delivery_orders": 8,
    "processes": 2,
    "budget_rebindings": 9,
}
HASHES = {
    "operation_contract_sha256": "0df7119713f5f59c5bcc1cb9149b734d394acc469f2839468ee32704b23f1d3f",
    "distribution_projection_sha256": "315f82e5795a0c8fba0e98d21891971d5180b338106e5791acb41bcc25c85f39",
    "canonical_output_sha256": "e69c721549966b1b88dcde3296674d675169840c6e8ebd0f236a5c07bcfc6415",
    "operation_inventory_identity_sha256": "c15201f700d5d0c883e148acc4976a6966b739b5f707d97735e2245aa67ef0cf",
    "mutation_identity_sha256": "392006debfe4d29cf56b9ee4a2a2817513b1f9401cccc9ba1e8f12e04babc820",
    "private_gate_identity_sha256": "4ca522e7ef5f2571ddc365f5eb0ef42092da9fb6c792e1d5b16c6d9639e14fa8",
}
CLASSES = [
    "logical_operation_parity",
    "typed_stop_preservation",
    "delivery_order_invariance",
    "mutation_qualification",
    "source_only_boundary",
]
BINDINGS = {
    "rust_candidate": "367ce3731d9bc2dd344ff77c48f2b63bb07b8bbe",
    "rust_assurance_sha256": "6f09d1fe2f0ca690838f82d463a2cc20ef18c5f382da1bb7c1b9f98287f7e44c",
    "operation_contract_sha256": "0df7119713f5f59c5bcc1cb9149b734d394acc469f2839468ee32704b23f1d3f",
    "distribution_manifest_sha256": "c76cd24bc91308b0e615bd837d69b72fe145b7713a544fb325f7f054275c485d",
    "distribution_lock_sha256": "0fc414a0e49b4e87bb0cf1f21bea3cf0cd70af904720b93a95fae00f079e7304",
    "canonical_output_sha256": "e69c721549966b1b88dcde3296674d675169840c6e8ebd0f236a5c07bcfc6415",
    "logical_operation_families": 14,
}
INDEPENDENT_CANDIDATE = "2ff0a9d4bbbd32cc07cecbda3fbb1abef8a1b95e"
OPAQUE_SOURCE_SHA256 = "8168bcf02c79b427b6f9c68cbaa6aeca421a65c36bddd47e65a269f1376fe57d"
OPAQUE_IDENTITY = "68de40abb71bd07eb9bafeb9a54f00187214ebe3a4fb8153a014f36d46b88a35"
RESULT_IDENTITY = "e5317db419535e10c494f5b61c46c01c00f4e256a7e8740d19692ae4f43367d7"


class OpaqueError(RuntimeError):
    pass


def require(condition: bool, label: str) -> None:
    if not condition:
        raise OpaqueError(label)


def canonical(value: Any) -> bytes:
    return json.dumps(value, ensure_ascii=False, sort_keys=True, separators=(",", ":")).encode()


def sha(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def validate_schema(value: object) -> None:
    require(type(value) is dict, "schema:object")
    assert isinstance(value, dict)
    require(value.get("type") == "object" and value.get("additionalProperties") is False, "schema:closed")
    require(value.get("required") == list(FIELDS) and tuple(value.get("properties", {})) == FIELDS, "schema:shape")
    assurance = value["properties"]["assurance"]
    bindings = value["properties"]["public_bindings"]
    require(assurance.get("additionalProperties") is False and bindings.get("additionalProperties") is False, "schema:nested")
    require(value["$defs"]["counts"].get("additionalProperties") is False, "schema:counts")
    require(value["$defs"]["hashes"].get("additionalProperties") is False, "schema:hashes")


def validate_sources(value: dict[str, Any]) -> None:
    require(sha(ROOT / "reports/causal_projection_assurance_v13.json") == BINDINGS["rust_assurance_sha256"], "source:rust_assurance")
    require(sha(ROOT / "spec/causal_projection_operation_contract_v13.json") == BINDINGS["operation_contract_sha256"], "source:contract")
    require(sha(ROOT / "fixtures/distribution/manifest_v14.json") == BINDINGS["distribution_manifest_sha256"], "source:manifest")
    require(sha(ROOT / "fixtures/distribution/manifest_v14.lock.json") == BINDINGS["distribution_lock_sha256"], "source:lock")
    rust = json.loads((ROOT / "reports/rust_conformance_v14.json").read_text())
    require(rust["canonical_output_sha256"] == value["assurance"]["hashes"]["canonical_output_sha256"], "source:canonical")
    contract = json.loads((ROOT / "spec/causal_projection_operation_contract_v13.json").read_text())
    require(len(contract["families"]) == value["assurance"]["counts"]["operation_families"], "source:operations")


def validate(value: object, schema: object) -> None:
    require(type(value) is dict and tuple(value) == FIELDS, "record:shape")
    assert isinstance(value, dict)
    require(value["schema"] == "nostr_automerge.opaque_causal_projection_import.v14.v1", "record:schema")
    require(value["status"] == "code_complete_publication_held" and value["result"] == "pass", "record:state")
    require(value["independent_candidate"] == INDEPENDENT_CANDIDATE, "record:candidate")
    require(value["opaque_record_sha256"] == OPAQUE_SOURCE_SHA256, "record:opaque_hash")
    assurance = value["assurance"]
    require(type(assurance) is dict and tuple(assurance) == ("schema", "status", "candidates", "counts", "hashes", "result_classes", "release_claimed", "publication_claimed", "remote_actions", "result", "identity_sha256"), "assurance:shape")
    require(assurance["schema"] == "nostr_automerge.opaque_causal_projection.v14.v1" and assurance["status"] == "code_complete_publication_held", "assurance:state")
    require(tuple(assurance["candidates"]) == CANDIDATES, "assurance:candidates")
    require(assurance["counts"] == COUNTS and assurance["hashes"] == HASHES, "assurance:evidence")
    require(assurance["result_classes"] == CLASSES, "assurance:classes")
    require(assurance["release_claimed"] is False and assurance["publication_claimed"] is False and assurance["remote_actions"] == 0 and assurance["result"] == "pass", "assurance:holds")
    opaque_projection = {key: assurance[key] for key in tuple(assurance)[:-1]}
    require(assurance["identity_sha256"] == OPAQUE_IDENTITY == hashlib.sha256(canonical(opaque_projection)).hexdigest(), "assurance:identity")
    require(value["public_bindings"] == BINDINGS, "record:bindings")
    projection = {key: value[key] for key in FIELDS[:-1]}
    require(value["result_identity_sha256"] == RESULT_IDENTITY == hashlib.sha256(canonical(projection)).hexdigest(), "record:identity")
    serialized = json.dumps(value, separators=(",", ":"))
    for forbidden in ("src/", "test/", "scripts/", "package.json", "node_modules", "command", "credential", "workflow", chr(104) + "ttps://", chr(102) + "ile://", chr(47) + "Users/", "\\\\"):
        require(forbidden not in serialized, "record:leak:" + forbidden)
    validate_schema(schema)
    validate_sources(value)


def self_test(record: dict[str, Any], schema: dict[str, Any]) -> int:
    caught = 0
    mutations = (
        lambda value: value.update(independent_candidate="0" * 40),
        lambda value: value.update(opaque_record_sha256="0" * 64),
        lambda value: value["assurance"]["candidates"].reverse(),
        lambda value: value["assurance"]["counts"].update(proofs=13),
        lambda value: value["assurance"]["counts"].update(mutation_survivors=1),
        lambda value: value["assurance"]["hashes"].update(canonical_output_sha256="0" * 64),
        lambda value: value["assurance"]["result_classes"].reverse(),
        lambda value: value["assurance"].update(publication_claimed=True),
        lambda value: value["public_bindings"].update(rust_candidate="0" * 40),
        lambda value: value["public_bindings"].update(distribution_lock_sha256="0" * 64),
        lambda value: value.update(result_identity_sha256="0" * 64),
        lambda value: value.update(extra=False),
    )
    for mutate in mutations:
        changed = copy.deepcopy(record)
        mutate(changed)
        try:
            validate(changed, schema)
        except OpaqueError:
            caught += 1
            continue
        raise OpaqueError("mutation:record")
    coordinated = copy.deepcopy(record)
    coordinated["assurance"]["hashes"]["mutation_identity_sha256"] = "1" * 64
    coordinated["assurance"]["identity_sha256"] = hashlib.sha256(canonical({key: coordinated["assurance"][key] for key in tuple(coordinated["assurance"])[:-1]})).hexdigest()
    coordinated["result_identity_sha256"] = hashlib.sha256(canonical({key: coordinated[key] for key in FIELDS[:-1]})).hexdigest()
    try:
        validate(coordinated, schema)
    except OpaqueError:
        caught += 1
    else:
        raise OpaqueError("mutation:coordinated")
    for mutate in (
        lambda value: value.update(additionalProperties=True),
        lambda value: value["required"].pop(),
        lambda value: value["properties"]["assurance"].update(additionalProperties=True),
        lambda value: value["$defs"]["hashes"].update(additionalProperties=True),
    ):
        changed = copy.deepcopy(schema)
        mutate(changed)
        try:
            validate(record, changed)
        except OpaqueError:
            caught += 1
            continue
        raise OpaqueError("mutation:schema")
    require(caught == 17, "mutation:count")
    return caught


def main() -> int:
    record = json.loads(REPORT.read_text())
    schema = json.loads(SCHEMA.read_text())
    validate(record, schema)
    mutations = self_test(record, schema)
    print(f"PASS: opaque causal projection v14 candidate={INDEPENDENT_CANDIDATE[:8]} operations=14 scenarios=204x8x2 mutations={mutations}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
