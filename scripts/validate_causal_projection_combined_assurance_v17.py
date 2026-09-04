#!/usr/bin/env python3
"""Validate final public and opaque causal-projection v17 assurance."""

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
REPORT = ROOT / "reports/causal_projection_combined_assurance_v17.json"
SCHEMA = ROOT / "tools/validation/causal_projection_combined_assurance_v17.schema.json"
CANDIDATE = "75453b48e4e19851b1d7480f7e4c7af817bd300a"
CANONICAL = "e69c721549966b1b88dcde3296674d675169840c6e8ebd0f236a5c07bcfc6415"
SERIALIZED = "000c52bde7102eaccec8cf65c875332e119fd25ccf4a2ac38973c456774a3344"
FIELDS = [
    "schema", "status", "checkpoint", "candidate", "imports", "implementation_candidates",
    "counts", "applicability", "identities", "finding_closure", "result_classes",
    "canonical_process_bytes", "release_claimed", "publication_claimed", "remote_actions",
    "result", "result_identity_sha256",
]
IMPORTS = {
    "operation_contract_sha256": "4770199ae28c4b18e114c250f9f2010ffe2668ef63dea10274fd22eae01bdfde",
    "rust_final_inventory_sha256": "f2212c183d009d146663116322ec4640db8812f66b6d09570115a3535a7603a2",
    "rust_evidence_graph_sha256": "283224879f13a69840e7222523649cc9639d73ae3cbe99464127b78f0121c527",
    "rust_mutation_sha256": "3b725f73261d7e7934555115dd1ac2f23bcfc61fdd7e4831d1ee111627d761d1",
    "rust_public_assurance_sha256": "766f2626507ab272bffadde8ca724f0378f4165c06b8647c3934b8d6913c867a",
    "distribution_transition_sha256": "9c4e3758d67ac76fa7f270ce92db676271eb822e2ff75141a617836fd92dd6d4",
    "rust_conformance_sha256": "e9cdcc625745514d4c98968947379b554e99c9819049eceff2a20a724b356f2b",
    "opaque_import_sha256": "54907c6123cb719d8089976daa5e2c3c0440ba3e5d0d4a24116431a3974c8471",
    "distribution_manifest_sha256": "7890fe2532da48ca84e54f5b1b883a38fd1a60ff58bb2999a056025335a4b5d3",
    "distribution_lock_sha256": "9e09dfd2de706d320c3bcd7cfe45b2f9a7560d5e9354809d2a41e5f52a2fba90",
    "finding_registry_sha256": "017593a11a9e348958c9293976f52e7cd2d778198710c622a28cd94e1e44a3d1",
}
PATHS = {
    "operation_contract_sha256": "spec/causal_projection_contracts_v17.json",
    "rust_final_inventory_sha256": "reports/causal_projection_final_inventory_v17.json",
    "rust_evidence_graph_sha256": "reports/causal_projection_evidence_graph_v17.json",
    "rust_mutation_sha256": "reports/causal_projection_mutations_v17.json",
    "rust_public_assurance_sha256": "reports/causal_projection_public_assurance_v17.json",
    "distribution_transition_sha256": "spec/distribution_v17_transition.json",
    "rust_conformance_sha256": "reports/rust_conformance_v17.json",
    "opaque_import_sha256": "reports/opaque_causal_projection_v17.json",
    "distribution_manifest_sha256": "fixtures/distribution/manifest_v16.json",
    "distribution_lock_sha256": "fixtures/distribution/manifest_v16.lock.json",
    "finding_registry_sha256": "spec/remediation_findings_v17.json",
}
CANDIDATES = {
    "public_contract": "4d25f76277ad02547f36658d45a5ef1d28689f2d",
    "public_inventory": "ad02b6ee407d6f5958c480f7f1b1c447eecc6f26",
    "public_evidence_graph": "e74dcdb3fdaa30aeeb59bab53126bbee82a64557",
    "public_assurance": "54a983fc2608ea9ca869c8fb344139e3b2b718a4",
    "public_transition": "10be9bc3d9a5bf653338c3b30195d0c8299c2dac",
    "public_conformance": "844a904ada74f1d2bac90fa8c67290a7f05807af",
    "public_opaque_import": CANDIDATE,
    "independent_assurance": "b4c5474d16a9da877bb36ba2ea7e22f707bd0e9e",
    "independent_implementation": "0c0e92ba63ca07da0de2d991720ca4efb511db17",
}
COUNTS = {
    "rust_operation_sites": 68,
    "rust_operation_families": 38,
    "rust_site_proofs": 68,
    "independent_operation_sites": 142,
    "independent_operation_families": 40,
    "independent_site_proofs": 142,
    "rust_behavioral_mutations": 31,
    "independent_behavioral_mutations": 14,
    "combined_behavioral_mutations": 45,
    "mutation_survivors": 0,
    "scenarios": 204,
    "signed_events": 771,
    "delivery_orders": 8,
    "processes": 2,
    "transition_affected": 0,
}
APPLICABILITY = {
    "shared_abstract_classes": [
        "projection_construction", "actor_sequence", "causal_counter", "frontier_comparison"
    ],
    "rust_concrete_counters": ["graph_node", "graph_edge"],
    "independent_counter_result": "site_identity_exact",
    "binding_rule": "shared_abstract_owner_language_specific_sites_and_counters",
}
IDENTITIES = {
    "operation_contract_sha256": IMPORTS["operation_contract_sha256"],
    "public_evidence_graph_identity_sha256": "bf0ef7f4d82cce29ddf9aa43b8949664ed4a1e55858e50d446a282f1fa278a66",
    "opaque_source_identity_sha256": "251d4d1d57ddbcf2f770fcf27a4afba481f308916405d8a146edb1d88bed43cc",
    "opaque_import_identity_sha256": "44307f565c812372b8d8dc6513f923ffa93ae8a50abf1d39219a257e49939f8b",
    "rust_conformance_identity_sha256": "78f8ad43cd57eb7c2d93176cac0eee11c2a8dfe7583759d62511f9b6f1b5e55b",
    "canonical_output_sha256": CANONICAL,
    "serialized_run_sha256": SERIALIZED,
}
CLOSURE = [
    {"id": "FINDING_119", "status": "closed", "evidence": ["final_inventory", "bidirectional_graph", "no_planned_values", "no_dangling_edges"]},
    {"id": "FINDING_120", "status": "closed", "evidence": ["exact_site_keys", "per_site_proofs", "same_family_swap_rejected", "opaque_site_identity"]},
    {"id": "FINDING_121", "status": "closed", "evidence": ["sealed_site_boundary", "direct_operation_coverage", "target_order_mutations", "zero_survivors"]},
    {"id": "FINDING_122", "status": "closed", "evidence": ["distinct_property_codes", "typed_stop_mutations", "unexpected_identity_mutations", "post_stop_mutations"]},
]
CLASSES = [
    "exact_site_identity", "sealed_charge_target_observe", "bidirectional_evidence",
    "property_specific_mutation_qualification", "cross_implementation_parity",
    "distribution_byte_identity",
]


class AssuranceError(RuntimeError):
    pass


def require(condition: bool, label: str) -> None:
    if not condition:
        raise AssuranceError(label)


def load(path: Path) -> Any:
    def closed(pairs: list[tuple[str, Any]]) -> dict[str, Any]:
        keys = [key for key, _ in pairs]
        require(len(keys) == len(set(keys)), "duplicate:" + path.name)
        return dict(pairs)
    return json.loads(path.read_text(), object_pairs_hook=closed)


def sha(path: str) -> str:
    return hashlib.sha256((ROOT / path).read_bytes()).hexdigest()


def committed(candidate: str, path: str) -> bytes:
    result = subprocess.run(["git", "show", f"{candidate}:{path}"], cwd=ROOT, capture_output=True, check=False)
    require(result.returncode == 0, "evidence:candidate")
    return result.stdout


def canonical(value: Any) -> bytes:
    return json.dumps(value, ensure_ascii=False, sort_keys=True, separators=(",", ":")).encode()


def exact_schema_record(schema: dict[str, Any], name: str, fields: list[str]) -> bool:
    value = schema["$defs"][name]
    return value.get("additionalProperties") is False and value.get("required") == fields and list(value.get("properties", {})) == fields


def expected() -> dict[str, Any]:
    value = {
        "schema": "nostr_automerge.causal_projection_combined_assurance.v17.v1",
        "status": "verified",
        "checkpoint": "step_1512",
        "candidate": CANDIDATE,
        "imports": IMPORTS,
        "implementation_candidates": CANDIDATES,
        "counts": COUNTS,
        "applicability": APPLICABILITY,
        "identities": IDENTITIES,
        "finding_closure": CLOSURE,
        "result_classes": CLASSES,
        "canonical_process_bytes": "identical",
        "release_claimed": False,
        "publication_claimed": False,
        "remote_actions": 0,
        "result": "pass",
        "result_identity_sha256": "",
    }
    value["result_identity_sha256"] = hashlib.sha256(canonical({key: value[key] for key in FIELDS[:-1]})).hexdigest()
    return value


def validate_evidence(record: dict[str, Any]) -> None:
    require(
        all(sha(path) == IMPORTS[key] for key, path in PATHS.items() if key != "finding_registry_sha256")
        and hashlib.sha256(committed(CANDIDATE, PATHS["finding_registry_sha256"])).hexdigest()
        == IMPORTS["finding_registry_sha256"],
        "evidence:hash",
    )
    inventory = load(ROOT / PATHS["rust_final_inventory_sha256"])
    graph = load(ROOT / PATHS["rust_evidence_graph_sha256"])
    mutations = load(ROOT / PATHS["rust_mutation_sha256"])
    conformance = load(ROOT / PATHS["rust_conformance_sha256"])
    opaque = load(ROOT / PATHS["opaque_import_sha256"])
    registry = json.loads(committed(CANDIDATE, PATHS["finding_registry_sha256"]))
    require(inventory["counts"] == {"rows": 68, "proofs": 68, "coverage": 68, "planned_values": 0}, "evidence:inventory")
    require(graph["counts"] == {"inventory_rows": 68, "proof_edges": 68, "coverage_edges": 68, "dangling": 0, "extra": 0}, "evidence:graph")
    require(mutations["counts"] == {"inventory_rows": 68, "mutations": 31, "coverage_records": 68, "uncovered_rows": 0, "unreferenced_mutations": 0, "survivors": 0}, "evidence:mutations")
    assurance = opaque["assurance"]
    require(
        assurance["counts"] == {"source_sites": 142, "operation_families": 40, "proofs": 142, "mutations": 14, "scenarios": 204, "signed_events": 771, "delivery_orders": 8, "processes": 2, "affected": 0, "mutation_survivors": 0}
        and assurance["applicability_classes"] == APPLICABILITY["shared_abstract_classes"]
        and all(value in assurance["result_classes"] for value in ("site_identity_exact", "sealed_charge_target_observe", "typed_stop_identity_exact", "bidirectional_evidence_complete")),
        "evidence:independent",
    )
    require(
        conformance["scenario_count"] == 204 and conformance["signed_event_count"] == 771
        and conformance["delivery_order_count"] == 8 and conformance["process_count"] == 2
        and conformance["transition_affected_count"] == 0 and conformance["canonical_process_bytes"] == "identical"
        and conformance["canonical_output_sha256"] == CANONICAL and conformance["serialized_run_sha256"] == SERIALIZED,
        "evidence:conformance",
    )
    require(
        [row["id"] for row in registry["findings"]] == ["FINDING_119", "FINDING_120", "FINDING_121", "FINDING_122", "FINDING_080"]
        and [row["status"] for row in registry["findings"]] == ["open", "open", "open", "open", "held"],
        "evidence:registry",
    )
    require(record["counts"]["combined_behavioral_mutations"] == record["counts"]["rust_behavioral_mutations"] + record["counts"]["independent_behavioral_mutations"], "evidence:combined")


def validate(record: Any, schema: Any) -> None:
    require(type(record) is dict and list(record) == FIELDS and record == expected(), "record:value")
    resolved = subprocess.run(["git", "rev-parse", "--verify", CANDIDATE + "^{commit}"], cwd=ROOT, capture_output=True, text=True, check=False)
    require(resolved.returncode == 0 and resolved.stdout.strip() == CANDIDATE, "record:candidate")
    require(
        type(schema) is dict and list(schema) == ["title", "type", "additionalProperties", "required", "properties", "$defs"]
        and schema["additionalProperties"] is False and schema["required"] == FIELDS and list(schema["properties"]) == FIELDS,
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


def self_test(record: dict[str, Any], schema: dict[str, Any]) -> int:
    attacks = [
        ("record", lambda value: value.update(candidate="0" * 40)),
        ("record", lambda value: value["imports"].update(rust_evidence_graph_sha256="0" * 64)),
        ("record", lambda value: value["implementation_candidates"].update(public_opaque_import="0" * 40)),
        ("record", lambda value: value["counts"].update(rust_operation_sites=67)),
        ("record", lambda value: value["counts"].update(independent_operation_sites=141)),
        ("record", lambda value: value["counts"].update(combined_behavioral_mutations=44)),
        ("record", lambda value: value["counts"].update(mutation_survivors=1)),
        ("record", lambda value: value["applicability"]["shared_abstract_classes"].reverse()),
        ("record", lambda value: value["applicability"]["rust_concrete_counters"].reverse()),
        ("record", lambda value: value["identities"].update(opaque_source_identity_sha256="0" * 64)),
        ("record", lambda value: value["finding_closure"][0].update(status="open")),
        ("record", lambda value: value["finding_closure"].reverse()),
        ("record", lambda value: value["result_classes"].reverse()),
        ("record", lambda value: value.update(release_claimed=True)),
        ("record", lambda value: value.update(publication_claimed=True)),
        ("record", lambda value: value.update(remote_actions=1)),
        ("record", lambda value: value.update(result_identity_sha256="0" * 64)),
        ("record", lambda value: value.update(extra=False)),
        ("schema", lambda value: value.update(additionalProperties=True)),
    ]
    caught = 0
    for target, mutate in attacks:
        changed_record, changed_schema = copy.deepcopy(record), copy.deepcopy(schema)
        mutate(changed_record if target == "record" else changed_schema)
        try:
            validate(changed_record, changed_schema)
        except AssuranceError:
            caught += 1
            continue
        raise AssuranceError("mutation:survived")
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
    print(f"PASS: causal projection combined assurance v17 sites=68+142 mutations=45 survivors=0 findings=4 attacks={attacks}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
