#!/usr/bin/env python3
"""Validate the leak-free public import of independent v16 assurance."""

from __future__ import annotations

import copy
import hashlib
import json
import subprocess
import sys
from pathlib import Path
from typing import Any

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parents[1]
REPORT = ROOT / "reports/opaque_causal_projection_v16.json"
SCHEMA = ROOT / "tools/validation/opaque_causal_projection_v16.schema.json"

FIELDS = [
    "schema",
    "status",
    "independent_candidate",
    "opaque_record_sha256",
    "assurance",
    "public_bindings",
    "result",
    "result_identity_sha256",
]
ASSURANCE_FIELDS = [
    "schema",
    "status",
    "candidate_chain",
    "public_distribution_candidate",
    "evidence_sha256",
    "counts",
    "applicability_classes",
    "result_classes",
    "canonical_output_sha256",
    "clean_scope",
    "standalone_identity_assumed",
    "release_claimed",
    "publication_claimed",
    "remote_actions",
    "result",
    "opaque_identity_sha256",
    "identity_sha256",
]
COUNTS = {
    "operation_sites": 142,
    "operation_families": 40,
    "site_proofs": 142,
    "runtime_family_proofs": 40,
    "behavior_mutations": 10,
    "mutation_survivors": 0,
    "scenarios": 204,
    "signed_events": 771,
    "delivery_orders": 8,
    "processes": 2,
    "public_budget_rebindings": 8,
    "private_budget_changes": 8,
}
PRIVATE_CANDIDATES = [
    "83c5fbd90ecd9bc8d8f04f22106277a772b3fa42",
    "538421e3ffaf9ed97664f7b931ed9a5c8e24cf65",
    "3856822d4183fc8d452538c1f16f6b253deed2e9",
    "b15c703d5956024f9500647b4446d057227a0ebb",
]
EVIDENCE = [
    "450a910da69fe72d83436520fef452c8c6b55fc05d828af39797606e1400737f",
    "1f85c7771e1c139110def0f7a6feba7519fb4c723d6fec5dfcc38ad882252be7",
    "d1c61a40b8de011362873ca628507db186389e8911441ce72ed50dabe58a2520",
    "8e18c8e5f14727bfccbbbc0724399d7642f57afff7e78c195f265d66f39a8da1",
    "9594c85bb8fdd163ea1e58a8b4c06108ae0330ee48b27c7a4f80da24333fcc84",
]
APPLICABILITY = [
    "projection_construction",
    "actor_sequence",
    "causal_counter_consumer",
    "frontier_comparison",
]
RESULT_CLASSES = [
    "actor_stage_isolated",
    "counter_binding_exact",
    "structural_mutations_caught",
    "signed_distribution_byte_identical",
    "typed_stops_preserved",
    "independent_implementation_boundary",
]
INDEPENDENT = "f931df45c070b7617df61205963bbbd46d07618c"
OPAQUE_SHA = "61a47214c7cbf48da9d3773fb10c459a6d1fa38e1b85c0adb05a98054ac225fb"
OPAQUE_IDENTITY = "659adb0ec14959e649adb9212ef7e8d0b079a500432e9da7c9afe9b1b063c498"
DISTRIBUTION_CANDIDATE = "18cd91d8b69a57c1304ffc5d29490185401cc42d"
CANONICAL = "e69c721549966b1b88dcde3296674d675169840c6e8ebd0f236a5c07bcfc6415"
BINDINGS = {
    "distribution_candidate": DISTRIBUTION_CANDIDATE,
    "operation_contract_sha256": "bbd58073a7dab83d7a96541ba7d1a90e0ceb5c4876bb4533d7b196058b5e7b3b",
    "rust_inventory_sha256": "95562a0f032c6fcedf3e397f82f42072fa2179b30a48b7424e38c2bf39403de1",
    "rust_proof_catalog_sha256": "486dd1f70a108166a5380ef533f707f1aeebac6b4f5b2d1f20708a9a4e0f4ca0",
    "rust_structural_assurance_sha256": "fbd2b12e558f54d161dc778e189cccacd391c51db1ecbb89e0c58a535076c9d1",
    "rust_mutation_sha256": "d4d88b74b5de2f73a46f17436c62aa185519ecaed04bf5868dcf93ebd5e9e490",
    "rust_assurance_sha256": "a1dfc1f97adf35529b2a25ebb7b12f2d39df27ef0db42c522f6ed91b45b55b33",
    "rust_conformance_sha256": "f77dc5b45496fff16e726c9ec4705b45bee3515992fc89ed9563bb18eb4000d8",
    "distribution_manifest_sha256": "7890fe2532da48ca84e54f5b1b883a38fd1a60ff58bb2999a056025335a4b5d3",
    "distribution_lock_sha256": "9e09dfd2de706d320c3bcd7cfe45b2f9a7560d5e9354809d2a41e5f52a2fba90",
    "canonical_output_sha256": CANONICAL,
    "rust_operation_sites": 68,
    "rust_operation_families": 38,
    "independent_operation_sites": 142,
    "independent_operation_families": 40,
}
PATHS = {
    "operation_contract_sha256": "spec/causal_projection_contracts_v16.json",
    "rust_inventory_sha256": "reports/causal_projection_operation_inventory_v16.json",
    "rust_proof_catalog_sha256": "reports/causal_projection_proof_catalog_v16.json",
    "rust_structural_assurance_sha256": "reports/causal_projection_structural_assurance_v16.json",
    "rust_mutation_sha256": "reports/causal_projection_mutations_v16.json",
    "rust_assurance_sha256": "reports/causal_projection_rust_assurance_v16.json",
    "rust_conformance_sha256": "reports/rust_conformance_v16.json",
    "distribution_manifest_sha256": "fixtures/distribution/manifest_v16.json",
    "distribution_lock_sha256": "fixtures/distribution/manifest_v16.lock.json",
}
PROJECTION_IDENTITY = "9e3856536b461a5d8e1641ad701c6bf425e0fc6b0ee235f1a866f01798ca39b8"
IDENTITY = "3940df56fc353cf47611448556052061edf691d7773f123b7bd8448cb44e6c87"


class OpaqueError(RuntimeError):
    pass


def require(condition: bool, label: str) -> None:
    if not condition:
        raise OpaqueError(label)


def load(path: Path) -> Any:
    def closed(pairs: list[tuple[str, Any]]) -> dict[str, Any]:
        keys = [key for key, _ in pairs]
        require(len(keys) == len(set(keys)), f"duplicate:{path.name}")
        return dict(pairs)

    return json.loads(path.read_text(), object_pairs_hook=closed)


def canonical(value: Any) -> bytes:
    return json.dumps(
        value, ensure_ascii=False, sort_keys=True, separators=(",", ":")
    ).encode()


def sha(path: str) -> str:
    return hashlib.sha256((ROOT / path).read_bytes()).hexdigest()


def validate(record: Any, schema: Any) -> None:
    require(type(record) is dict and list(record) == FIELDS, "record:shape")
    require(
        record["schema"] == "nostr_automerge.opaque_causal_projection_import.v16.v1"
        and record["status"] == "code_complete_publication_held"
        and record["result"] == "pass",
        "record:state",
    )
    require(
        record["independent_candidate"] == INDEPENDENT
        and record["opaque_record_sha256"] == OPAQUE_SHA,
        "record:source",
    )
    assurance = record["assurance"]
    require(
        type(assurance) is dict and list(assurance) == ASSURANCE_FIELDS,
        "assurance:shape",
    )
    expected_assurance = {
        "schema": "nostr_automerge.opaque_causal_projection.v16.public.v1",
        "status": "code_complete_publication_held",
        "candidate_chain": PRIVATE_CANDIDATES,
        "public_distribution_candidate": DISTRIBUTION_CANDIDATE,
        "evidence_sha256": EVIDENCE,
        "counts": COUNTS,
        "applicability_classes": APPLICABILITY,
        "result_classes": RESULT_CLASSES,
        "canonical_output_sha256": CANONICAL,
        "clean_scope": True,
        "standalone_identity_assumed": False,
        "release_claimed": False,
        "publication_claimed": False,
        "remote_actions": 0,
        "result": "pass",
        "opaque_identity_sha256": OPAQUE_IDENTITY,
        "identity_sha256": PROJECTION_IDENTITY,
    }
    require(assurance == expected_assurance, "assurance:value")
    require(
        hashlib.sha256(
            canonical({key: assurance[key] for key in ASSURANCE_FIELDS[:-1]})
        ).hexdigest()
        == PROJECTION_IDENTITY,
        "assurance:identity",
    )
    require(record["public_bindings"] == BINDINGS, "bindings:value")
    require(
        all(sha(path) == BINDINGS[key] for key, path in PATHS.items()),
        "bindings:hash",
    )
    require(
        subprocess.run(
            ["git", "rev-parse", f"{DISTRIBUTION_CANDIDATE}^{{commit}}"],
            cwd=ROOT,
            capture_output=True,
            text=True,
            check=True,
        ).stdout.strip()
        == DISTRIBUTION_CANDIDATE,
        "bindings:candidate",
    )
    lock = load(ROOT / "fixtures/distribution/manifest_v16.lock.json")
    require(
        lock["manifest_sha256"] == BINDINGS["distribution_manifest_sha256"]
        and lock["scenario_count"] == COUNTS["scenarios"]
        and lock["signed_event_count"] == COUNTS["signed_events"],
        "bindings:distribution",
    )
    require(
        record["result_identity_sha256"] == IDENTITY
        == hashlib.sha256(
            canonical({key: record[key] for key in FIELDS[:-1]})
        ).hexdigest(),
        "record:identity",
    )

    require(
        type(schema) is dict
        and list(schema) == [
            "title",
            "type",
            "additionalProperties",
            "required",
            "properties",
            "$defs",
        ]
        and schema["type"] == "object"
        and schema["additionalProperties"] is False
        and schema["required"] == FIELDS
        and list(schema["properties"]) == FIELDS,
        "schema:root",
    )
    require(
        schema["properties"]["assurance"]["additionalProperties"] is False
        and schema["properties"]["assurance"]["required"] == ASSURANCE_FIELDS
        and list(schema["properties"]["public_bindings"]["properties"])
        == list(BINDINGS)
        and schema["properties"]["public_bindings"]["required"] == list(BINDINGS)
        and schema["properties"]["public_bindings"]["additionalProperties"] is False
        and schema["$defs"]["counts"]["required"] == list(COUNTS)
        and list(schema["$defs"]["counts"]["properties"]) == list(COUNTS)
        and schema["$defs"]["counts"]["additionalProperties"] is False,
        "schema:nested",
    )

    serialized = json.dumps(record, separators=(",", ":"))
    forbidden = (
        "s" + "rc/",
        "te" + "st/",
        "pack" + "age.json",
        "node_" + "modules",
        "command",
        "credential",
        "workflow",
        chr(104) + "ttps" + chr(58) + chr(47) * 2,
        chr(102) + "ile" + chr(58) + chr(47) * 2,
        chr(47) + "Users/",
        "\\",
    )
    require(all(token not in serialized for token in forbidden), "record:leak")


def self_test(record: dict[str, Any], schema: dict[str, Any]) -> int:
    attacks = [
        lambda value: value.update(independent_candidate="0" * 40),
        lambda value: value.update(opaque_record_sha256="0" * 64),
        lambda value: value["assurance"]["candidate_chain"].reverse(),
        lambda value: value["assurance"]["candidate_chain"].pop(),
        lambda value: value["assurance"]["candidate_chain"].append("0" * 40),
        lambda value: value["assurance"]["evidence_sha256"].reverse(),
        lambda value: value["assurance"]["evidence_sha256"].pop(),
        lambda value: value["assurance"]["counts"].update(operation_families=41),
        lambda value: value["assurance"]["counts"].update(mutation_survivors=1),
        lambda value: value["assurance"]["applicability_classes"].reverse(),
        lambda value: value["assurance"]["result_classes"].reverse(),
        lambda value: value["assurance"].update(clean_scope=False),
        lambda value: value["assurance"].update(standalone_identity_assumed=True),
        lambda value: value["assurance"].update(remote_actions=1),
        lambda value: value["public_bindings"].update(rust_inventory_sha256="0" * 64),
        lambda value: value["public_bindings"].update(rust_operation_sites=69),
        lambda value: value.update(result_identity_sha256="0" * 64),
        lambda value: value.update(extra=False),
        lambda value: value.update(schema=value.pop("schema")),
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
        raise OpaqueError("mutation:record")

    coordinated = copy.deepcopy(record)
    coordinated["assurance"]["counts"]["operation_families"] = 41
    coordinated["assurance"]["identity_sha256"] = hashlib.sha256(
        canonical(
            {key: coordinated["assurance"][key] for key in ASSURANCE_FIELDS[:-1]}
        )
    ).hexdigest()
    coordinated["result_identity_sha256"] = hashlib.sha256(
        canonical({key: coordinated[key] for key in FIELDS[:-1]})
    ).hexdigest()
    try:
        validate(coordinated, schema)
    except OpaqueError:
        caught += 1
    else:
        raise OpaqueError("mutation:coordinated")

    schema_attacks = [
        lambda value: value.update(additionalProperties=True),
        lambda value: value["required"].pop(),
        lambda value: value["properties"]["assurance"].update(
            additionalProperties=True
        ),
        lambda value: value["properties"]["public_bindings"]["required"].pop(),
        lambda value: value["$defs"]["counts"]["properties"].pop(
            "private_budget_changes"
        ),
    ]
    for mutate in schema_attacks:
        changed = copy.deepcopy(schema)
        mutate(changed)
        try:
            validate(record, changed)
        except OpaqueError:
            caught += 1
            continue
        raise OpaqueError("mutation:schema")
    require(caught == 25, "mutation:count")
    return caught


def main() -> int:
    record = load(REPORT)
    schema = load(SCHEMA)
    validate(record, schema)
    mutations = self_test(record, schema)
    print(
        "PASS: opaque causal projection v16 "
        f"candidate={INDEPENDENT[:8]} rust=68,38 independent=142,40 "
        f"scenarios=204x8x2 mutations={mutations}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
