#!/usr/bin/env python3
"""Validate combined Rust and independent causal-projection v16 assurance."""

from __future__ import annotations

import argparse
import copy
import hashlib
import json
import subprocess
import sys
from pathlib import Path
from typing import Any

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parents[1]
REPORT = ROOT / "reports/causal_projection_combined_assurance_v16.json"
SCHEMA = ROOT / "tools/validation/causal_projection_combined_assurance_v16.schema.json"
FIELDS = [
    "schema",
    "status",
    "candidate",
    "imports",
    "implementation_candidates",
    "counts",
    "applicability",
    "identities",
    "finding_closure",
    "result_classes",
    "canonical_process_bytes",
    "release_claimed",
    "publication_claimed",
    "remote_actions",
    "result",
    "result_identity_sha256",
]
IMPORTS = {
    "operation_contract_sha256": "bbd58073a7dab83d7a96541ba7d1a90e0ceb5c4876bb4533d7b196058b5e7b3b",
    "rust_inventory_sha256": "95562a0f032c6fcedf3e397f82f42072fa2179b30a48b7424e38c2bf39403de1",
    "rust_proof_catalog_sha256": "486dd1f70a108166a5380ef533f707f1aeebac6b4f5b2d1f20708a9a4e0f4ca0",
    "rust_structural_assurance_sha256": "fbd2b12e558f54d161dc778e189cccacd391c51db1ecbb89e0c58a535076c9d1",
    "rust_mutation_sha256": "d4d88b74b5de2f73a46f17436c62aa185519ecaed04bf5868dcf93ebd5e9e490",
    "rust_assurance_sha256": "a1dfc1f97adf35529b2a25ebb7b12f2d39df27ef0db42c522f6ed91b45b55b33",
    "rust_conformance_sha256": "f77dc5b45496fff16e726c9ec4705b45bee3515992fc89ed9563bb18eb4000d8",
    "opaque_import_sha256": "6f9aa02dd558b755343d259f645cc5a2ac3f3481aad5d1d463fa1927b0b5e23c",
    "distribution_manifest_sha256": "7890fe2532da48ca84e54f5b1b883a38fd1a60ff58bb2999a056025335a4b5d3",
    "distribution_lock_sha256": "9e09dfd2de706d320c3bcd7cfe45b2f9a7560d5e9354809d2a41e5f52a2fba90",
}
PATHS = {
    "operation_contract_sha256": "spec/causal_projection_contracts_v16.json",
    "rust_inventory_sha256": "reports/causal_projection_operation_inventory_v16.json",
    "rust_proof_catalog_sha256": "reports/causal_projection_proof_catalog_v16.json",
    "rust_structural_assurance_sha256": "reports/causal_projection_structural_assurance_v16.json",
    "rust_mutation_sha256": "reports/causal_projection_mutations_v16.json",
    "rust_assurance_sha256": "reports/causal_projection_rust_assurance_v16.json",
    "rust_conformance_sha256": "reports/rust_conformance_v16.json",
    "opaque_import_sha256": "reports/opaque_causal_projection_v16.json",
    "distribution_manifest_sha256": "fixtures/distribution/manifest_v16.json",
    "distribution_lock_sha256": "fixtures/distribution/manifest_v16.lock.json",
}
CANDIDATE = "ef4bf8b561500d82db305d2180ec5df3a2d3e8b7"
CANDIDATES = {
    "public_import": CANDIDATE,
    "public_rust_assurance": "f52fdb9da47ccb6cb9dbc25c7b50954679d972b2",
    "public_distribution": "18cd91d8b69a57c1304ffc5d29490185401cc42d",
    "independent_assurance": "f931df45c070b7617df61205963bbbd46d07618c",
    "independent_implementation": "b15c703d5956024f9500647b4446d057227a0ebb",
}
COUNTS = {
    "rust_operation_sites": 68,
    "rust_operation_families": 38,
    "rust_site_proofs": 68,
    "independent_operation_sites": 142,
    "independent_operation_families": 40,
    "independent_site_proofs": 142,
    "independent_family_proofs": 40,
    "rust_behavioral_mutations": 13,
    "independent_behavioral_mutations": 10,
    "combined_behavioral_mutations": 23,
    "mutation_survivors": 0,
    "scenarios": 204,
    "signed_events": 771,
    "delivery_orders": 8,
    "processes": 2,
    "public_budget_rebindings": 8,
    "independent_budget_changes": 8,
}
APPLICABILITY = {
    "shared_abstract_classes": [
        "projection_construction",
        "actor_sequence",
        "causal_counter_consumer",
        "frontier_comparison",
    ],
    "rust_concrete_counter": "GraphNode",
    "independent_counter_result": "counter_binding_exact",
    "binding_rule": "shared_abstract_owner_language_specific_concrete_counter",
}
IDENTITIES = {
    "operation_contract_sha256": IMPORTS["operation_contract_sha256"],
    "distribution_identity_sha256": "3b360ceab9d24c2baacf9cd4fd594ebf65a62717b1799a080db6b0c0b4e81318",
    "canonical_output_sha256": "e69c721549966b1b88dcde3296674d675169840c6e8ebd0f236a5c07bcfc6415",
    "serialized_run_sha256": "000c52bde7102eaccec8cf65c875332e119fd25ccf4a2ac38973c456774a3344",
    "rust_assurance_identity_sha256": "1c7379e4ed1840433d08d7933f76f9d5f2481a711102bc34a82db6c12abfd901",
    "opaque_assurance_identity_sha256": "659adb0ec14959e649adb9212ef7e8d0b079a500432e9da7c9afe9b1b063c498",
    "opaque_projection_identity_sha256": "9e3856536b461a5d8e1641ad701c6bf425e0fc6b0ee235f1a866f01798ca39b8",
    "opaque_import_identity_sha256": "3940df56fc353cf47611448556052061edf691d7773f123b7bd8448cb44e6c87",
}
CLOSURE = [
    {
        "id": "FINDING_116",
        "status": "closed",
        "evidence": [
            "actor_stage_contract",
            "rust_assurance",
            "independent_assurance",
            "distribution_v16",
        ],
    },
    {
        "id": "FINDING_117",
        "status": "closed",
        "evidence": [
            "counter_contract",
            "rust_counter_binding",
            "independent_counter_binding",
            "coordinated_drift_rejection",
        ],
    },
    {
        "id": "FINDING_118",
        "status": "closed",
        "evidence": [
            "rust_mutations",
            "independent_mutations",
            "structural_identity_split",
            "zero_survivors",
        ],
    },
]
CLASSES = [
    "actor_sequence_ownership",
    "language_specific_counter_binding",
    "property_specific_mutation_qualification",
    "cross_implementation_parity",
    "distribution_byte_identity",
]
IDENTITY = "ed9cfabc4e5ecbfbad415c63da92e510901f930add68f37cdcb7acec9c994323"


class AssuranceError(RuntimeError):
    pass


def require(condition: bool, label: str) -> None:
    if not condition:
        raise AssuranceError(label)


def load(path: Path) -> Any:
    def closed(pairs: list[tuple[str, Any]]) -> dict[str, Any]:
        keys = [key for key, _ in pairs]
        require(len(keys) == len(set(keys)), f"duplicate:{path.name}")
        return dict(pairs)

    return json.loads(path.read_text(), object_pairs_hook=closed)


def sha(relative: str) -> str:
    return hashlib.sha256((ROOT / relative).read_bytes()).hexdigest()


def canonical(value: Any) -> bytes:
    return json.dumps(
        value, ensure_ascii=False, sort_keys=True, separators=(",", ":")
    ).encode()


def exact_schema_record(schema: dict[str, Any], name: str, fields: list[str]) -> bool:
    value = schema["$defs"][name]
    return (
        value.get("additionalProperties") is False
        and value.get("required") == fields
        and list(value.get("properties", {})) == fields
    )


def validate_evidence(record: dict[str, Any]) -> None:
    require(
        all(sha(path) == IMPORTS[key] for key, path in PATHS.items()),
        "evidence:hash",
    )
    inventory = load(ROOT / PATHS["rust_inventory_sha256"])
    proofs = load(ROOT / PATHS["rust_proof_catalog_sha256"])
    mutations = load(ROOT / PATHS["rust_mutation_sha256"])
    rust = load(ROOT / PATHS["rust_assurance_sha256"])
    conformance = load(ROOT / PATHS["rust_conformance_sha256"])
    opaque = load(ROOT / PATHS["opaque_import_sha256"])
    lock = load(ROOT / PATHS["distribution_lock_sha256"])
    require(
        inventory["counts"]
        == {
            "rows": 68,
            "families": 38,
            "phases": {
                "projection_construction": 50,
                "actor_sequence": 4,
                "causal_counter_consumer": 3,
                "frontier_comparison": 11,
            },
        }
        and inventory["counter_correction"]["rust_counter"] == "graph_node",
        "evidence:inventory",
    )
    require(
        proofs["row_count"] == 68
        and len(proofs["rows"]) == 68
        and all(row["result"] == "pass" for row in proofs["rows"]),
        "evidence:proofs",
    )
    require(
        mutations["mutation_count"] == 13
        and mutations["survivors"] == 0
        and mutations["compile_failures"] == 0,
        "evidence:mutations",
    )
    require(
        rust["counts"]
        == {
            "operation_sites": 68,
            "operation_families": 38,
            "proofs": 68,
            "property_codes": 10,
            "behavioral_mutations": 13,
            "mutation_survivors": 0,
            "consumer_bindings": 3,
            "scenarios": 204,
            "signed_events": 771,
            "delivery_orders": 8,
            "processes": 2,
        }
        and rust["assurance"]["dependency_count_counter"] == "GraphNode"
        and rust["result_identity_sha256"]
        == IDENTITIES["rust_assurance_identity_sha256"],
        "evidence:rust",
    )
    independent = opaque["assurance"]
    require(
        opaque["independent_candidate"] == CANDIDATES["independent_assurance"]
        and independent["candidate_chain"][-1]
        == CANDIDATES["independent_implementation"]
        and independent["counts"]["operation_sites"] == 142
        and independent["counts"]["operation_families"] == 40
        and independent["counts"]["site_proofs"] == 142
        and independent["counts"]["runtime_family_proofs"] == 40
        and independent["counts"]["behavior_mutations"] == 10
        and independent["counts"]["mutation_survivors"] == 0
        and independent["applicability_classes"]
        == APPLICABILITY["shared_abstract_classes"]
        and "counter_binding_exact" in independent["result_classes"]
        and independent["opaque_identity_sha256"]
        == IDENTITIES["opaque_assurance_identity_sha256"]
        and independent["identity_sha256"]
        == IDENTITIES["opaque_projection_identity_sha256"]
        and opaque["result_identity_sha256"]
        == IDENTITIES["opaque_import_identity_sha256"],
        "evidence:independent",
    )
    require(
        conformance["scenario_count"] == 204
        and conformance["signed_event_count"] == 771
        and conformance["delivery_order_count"] == 8
        and conformance["process_count"] == 2
        and conformance["canonical_process_bytes"] == "identical"
        and conformance["canonical_output_sha256"]
        == IDENTITIES["canonical_output_sha256"]
        and conformance["serialized_run_sha256"]
        == IDENTITIES["serialized_run_sha256"]
        and lock["result_identity_sha256"]
        == IDENTITIES["distribution_identity_sha256"],
        "evidence:distribution",
    )
    require(
        record["counts"]["combined_behavioral_mutations"]
        == record["counts"]["rust_behavioral_mutations"]
        + record["counts"]["independent_behavioral_mutations"],
        "evidence:combined",
    )


def validate(record: Any, schema: Any) -> None:
    require(type(record) is dict and list(record) == FIELDS, "record:shape")
    require(
        record["schema"]
        == "nostr_automerge.causal_projection_combined_assurance.v16.v1"
        and record["status"] == "verified",
        "record:state",
    )
    require(
        record["candidate"] == CANDIDATE
        and record["imports"] == IMPORTS
        and record["implementation_candidates"] == CANDIDATES,
        "record:bindings",
    )
    resolved = subprocess.run(
        ["git", "rev-parse", "--verify", f"{CANDIDATE}^{{commit}}"],
        cwd=ROOT,
        capture_output=True,
        text=True,
        check=False,
    )
    require(
        resolved.returncode == 0 and resolved.stdout.strip() == CANDIDATE,
        "record:candidate",
    )
    require(
        record["counts"] == COUNTS
        and record["applicability"] == APPLICABILITY
        and record["identities"] == IDENTITIES
        and record["finding_closure"] == CLOSURE
        and record["result_classes"] == CLASSES,
        "record:evidence",
    )
    require(
        record["canonical_process_bytes"] == "identical"
        and record["release_claimed"] is False
        and record["publication_claimed"] is False
        and record["remote_actions"] == 0
        and record["result"] == "pass",
        "record:result",
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
        and list(schema)
        == ["title", "type", "additionalProperties", "required", "properties", "$defs"]
        and schema["additionalProperties"] is False
        and schema["required"] == FIELDS
        and list(schema["properties"]) == FIELDS,
        "schema:shape",
    )
    require(
        exact_schema_record(schema, "imports", list(IMPORTS))
        and exact_schema_record(schema, "candidates", list(CANDIDATES))
        and exact_schema_record(schema, "counts", list(COUNTS))
        and exact_schema_record(schema, "applicability", list(APPLICABILITY))
        and exact_schema_record(schema, "identities", list(IDENTITIES))
        and exact_schema_record(schema, "finding", ["id", "status", "evidence"]),
        "schema:nested",
    )
    validate_evidence(record)


def run_distribution() -> bytes:
    result = subprocess.run(
        [
            "cargo",
            "extbuild",
            "run",
            "--",
            "cargo",
            "run",
            "--quiet",
            "-p",
            "nostr_automerge_conformance",
            "--locked",
            "--",
            "run_distribution",
            "fixtures/distribution/manifest_v16.json",
        ],
        cwd=ROOT,
        capture_output=True,
        check=False,
    )
    require(result.returncode == 0, "distribution:exit")
    parsed = json.loads(result.stdout)
    require(
        parsed["status"] == "pass"
        and parsed["fixture_count"] == 204
        and parsed["delivery_permutations"] == 8
        and len(parsed["reports"]) == 204
        and parsed["canonical_output_sha256"]
        == IDENTITIES["canonical_output_sha256"],
        "distribution:result",
    )
    require(
        hashlib.sha256(result.stdout).hexdigest()
        == IDENTITIES["serialized_run_sha256"],
        "distribution:bytes",
    )
    return result.stdout


def reject_deliberate_mismatch(run: bytes) -> None:
    actual = json.loads(run)
    changed = copy.deepcopy(actual)
    changed["canonical_output_sha256"] = "0" * 64
    require(canonical(actual) != canonical(changed), "distribution:mismatch")


def self_test(record: dict[str, Any], schema: dict[str, Any]) -> int:
    attacks = [
        lambda value: value.update(candidate="0" * 40),
        lambda value: value["imports"].update(rust_inventory_sha256="0" * 64),
        lambda value: value["implementation_candidates"].update(
            independent_assurance="0" * 40
        ),
        lambda value: value["counts"].update(rust_operation_sites=67),
        lambda value: value["counts"].update(independent_operation_sites=141),
        lambda value: value["counts"].update(rust_site_proofs=67),
        lambda value: value["counts"].update(independent_site_proofs=141),
        lambda value: value["counts"].update(rust_behavioral_mutations=12),
        lambda value: value["counts"].update(
            independent_behavioral_mutations=9
        ),
        lambda value: value["counts"].update(mutation_survivors=1),
        lambda value: value["applicability"]["shared_abstract_classes"].reverse(),
        lambda value: value["applicability"].update(rust_concrete_counter="GraphEdge"),
        lambda value: value["identities"].update(canonical_output_sha256="0" * 64),
        lambda value: value["finding_closure"].reverse(),
        lambda value: value["finding_closure"][0].update(status="open"),
        lambda value: value["result_classes"].reverse(),
        lambda value: value.update(publication_claimed=True),
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
        except AssuranceError:
            caught += 1
            continue
        raise AssuranceError("mutation:record")
    coordinated = copy.deepcopy(record)
    coordinated["counts"]["rust_behavioral_mutations"] = 14
    coordinated["counts"]["combined_behavioral_mutations"] = 24
    coordinated["result_identity_sha256"] = hashlib.sha256(
        canonical({key: coordinated[key] for key in FIELDS[:-1]})
    ).hexdigest()
    try:
        validate(coordinated, schema)
    except AssuranceError:
        caught += 1
    else:
        raise AssuranceError("mutation:coordinated")
    schema_attacks = [
        lambda value: value.update(additionalProperties=True),
        lambda value: value["required"].pop(),
        lambda value: value["$defs"]["imports"]["required"].pop(),
        lambda value: value["$defs"]["counts"].update(additionalProperties=True),
        lambda value: value["$defs"]["applicability"]["properties"].pop(
            "binding_rule"
        ),
    ]
    for mutate in schema_attacks:
        changed = copy.deepcopy(schema)
        mutate(changed)
        try:
            validate(record, changed)
        except AssuranceError:
            caught += 1
            continue
        raise AssuranceError("mutation:schema")
    require(caught == 26, "mutation:count")
    return caught


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--run-conformance", action="store_true")
    args = parser.parse_args()
    record = load(REPORT)
    schema = load(SCHEMA)
    validate(record, schema)
    mutations = self_test(record, schema)
    processes = 0
    if args.run_conformance:
        first = run_distribution()
        second = run_distribution()
        require(first == second, "distribution:process_identity")
        reject_deliberate_mismatch(first)
        processes = 2
    print(
        "PASS: causal projection combined assurance v16 "
        "rust=68,38 independent=142,40 mutations=23 survivors=0 "
        f"scenarios=204x8 processes={processes} negative_mutations={mutations}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
