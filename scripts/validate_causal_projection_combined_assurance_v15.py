#!/usr/bin/env python3
"""Validate combined public and opaque causal-projection v15 assurance."""

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
REPORT = ROOT / "reports/causal_projection_combined_assurance_v15.json"
SCHEMA = ROOT / "tools/validation/causal_projection_combined_assurance_v15.schema.json"
FIELDS = ["schema","status","candidate","imports","implementation_candidates","counts","identities","finding_closure","result_classes","canonical_process_bytes","release_claimed","publication_claimed","remote_actions","result","result_identity_sha256"]
IMPORTS = {
    "operation_contract_sha256":"12dc6aca59ad0807757cc13c372b582e67bc70a7295fb741c15f5d91412ea078",
    "rust_ownership_sha256":"19813d525ee1ec037cfe3f789650789cae95a31dd1cf6db5b368a5fa0da625e6",
    "rust_proof_catalog_sha256":"9a4fa04c1c3be3934d3ef40d8573c16a955eca08092dd7e2f1ec8747580a7f96",
    "rust_mutation_qualification_sha256":"df292f4146d6c0d7e772af4ed457d9b79622dbed5fd76f32a5c3594ca0701edb",
    "rust_conformance_sha256":"7ce224864f269e2818bc837d8252e6eeca8ee299a98604d4dd2d228d5c0ea6f5",
    "opaque_import_sha256":"c2885e24c1042a386eb20d27c3176715c83707f009d314a8c243e7d79b91af28",
    "distribution_manifest_sha256":"862d0c1ad6ae14cd54b75f88742fa3b584c6c3981195bfeb988818403bee689c",
    "distribution_lock_sha256":"a511c18a540aaa5de5a7ef23cf6b360108a74e0e178c1e1025907ae880d78da7",
}
PATHS = {
    "operation_contract_sha256":"spec/causal_projection_operation_discovery_v15.json",
    "rust_ownership_sha256":"reports/causal_projection_source_ownership_v15.json",
    "rust_proof_catalog_sha256":"reports/causal_projection_proof_catalog_v15.json",
    "rust_mutation_qualification_sha256":"reports/causal_projection_behavior_mutations_v15.json",
    "rust_conformance_sha256":"reports/rust_conformance_v15.json",
    "opaque_import_sha256":"reports/opaque_causal_projection_v15.json",
    "distribution_manifest_sha256":"fixtures/distribution/manifest_v15.json",
    "distribution_lock_sha256":"fixtures/distribution/manifest_v15.lock.json",
}
CANDIDATE = "f2cf08f1477de2620a3048bfba749588b047fea7"
CANDIDATES = {"public_assurance":CANDIDATE,"public_distribution":"e4d418249585adcabaf1a94f4e6a31a1ce0ffb55","independent_assurance":"2307800f980027bbe40ffc1312dde12f94ba2174","independent_implementation":"1cbd985289ac35b9cf0f2fa3221b190ab1fb5c74"}
COUNTS = {"operation_families":43,"rust_proofs":43,"independent_proofs":43,"rust_proof_executions":52,"rust_behavioral_mutations":13,"independent_behavioral_mutations":9,"combined_behavioral_mutations":22,"mutation_survivors":0,"scenarios":204,"signed_events":771,"delivery_orders":8,"processes":2,"budget_rebindings":9}
IDENTITIES = {"operation_contract_sha256":IMPORTS["operation_contract_sha256"],"distribution_identity_sha256":"be61110e2e1c3eb2dc7f30244e07a9efd6d0f4f1beae9693e77441506a35ac92","canonical_output_sha256":"e69c721549966b1b88dcde3296674d675169840c6e8ebd0f236a5c07bcfc6415","serialized_run_sha256":"000c52bde7102eaccec8cf65c875332e119fd25ccf4a2ac38973c456774a3344","opaque_assurance_identity_sha256":"614cbbf92c2592e54337a170fc8c14792bb537994f211872cf312d1fec8d66de","opaque_import_identity_sha256":"1c85a51f9f31d3541bc5a1762184c710e2fae71e1d199174dfb516ae19d22a1a"}
CLOSURE = [
    {"id":"FINDING_113","status":"closed","evidence":["operation_contract","rust_ownership","rust_proofs","independent_assurance"]},
    {"id":"FINDING_114","status":"closed","evidence":["rust_ownership","rust_proofs","independent_assurance","distribution_v15"]},
    {"id":"FINDING_115","status":"closed","evidence":["rust_mutations","independent_mutations","zero_survivors"]},
]
CLASSES = ["operation_ownership_complete","evidence_completeness","behavioral_mutation_qualification","cross_implementation_parity","distribution_byte_identity"]
IDENTITY = "2a726a8f7055f64b7c60b33bcef29a267c694309c11442d1da9e10674702c3a4"


class AssuranceError(RuntimeError):
    pass


def require(value: bool, label: str) -> None:
    if not value:
        raise AssuranceError(label)


def sha(relative: str) -> str:
    return hashlib.sha256((ROOT / relative).read_bytes()).hexdigest()


def canonical(value: Any) -> bytes:
    return json.dumps(value, ensure_ascii=False, separators=(",", ":")).encode()


def exact_schema_record(schema: dict[str, Any], definition: str, fields: list[str]) -> bool:
    value = schema["$defs"][definition]
    return value.get("additionalProperties") is False and value.get("required") == fields and list(value.get("properties", {})) == fields


def validate_sources(record: dict[str, Any]) -> None:
    require(all(sha(path) == IMPORTS[key] for key, path in PATHS.items()), "source:hash")
    ownership = json.loads((ROOT / PATHS["rust_ownership_sha256"]).read_text())
    proofs = json.loads((ROOT / PATHS["rust_proof_catalog_sha256"]).read_text())
    mutations = json.loads((ROOT / PATHS["rust_mutation_qualification_sha256"]).read_text())
    rust = json.loads((ROOT / PATHS["rust_conformance_sha256"]).read_text())
    opaque = json.loads((ROOT / PATHS["opaque_import_sha256"]).read_text())
    lock = json.loads((ROOT / PATHS["distribution_lock_sha256"]).read_text())
    require(ownership["operation_count"] == 43 and ownership["status"] == "pass", "source:ownership")
    require(proofs["row_count"] == 43 and len(proofs["rows"]) == 43 and all(row["reachability_count"] > 0 and row["result"] == "pass" for row in proofs["rows"]), "source:proofs")
    require(mutations["mutation_count"] == 13 and mutations["covered_operation_count"] == 43 and mutations["proof_execution_count"] == 52 and mutations["survivors"] == 0, "source:mutations")
    require(rust["scenario_count"] == 204 and rust["process_count"] == 2 and rust["delivery_order_count"] == 8 and rust["canonical_process_bytes"] == "identical" and rust["canonical_output_sha256"] == IDENTITIES["canonical_output_sha256"] and rust["serialized_run_sha256"] == IDENTITIES["serialized_run_sha256"], "source:rust")
    require(opaque["independent_candidate"] == CANDIDATES["independent_assurance"] and opaque["assurance"]["terminal_candidate"] == CANDIDATES["independent_implementation"] and opaque["assurance"]["counts"]["operation_families"] == 43 and opaque["assurance"]["counts"]["focused_proofs"] == 43 and opaque["assurance"]["counts"]["behavioral_mutations"] == 9 and opaque["assurance"]["counts"]["mutation_survivors"] == 0 and opaque["assurance"]["identity_sha256"] == IDENTITIES["opaque_assurance_identity_sha256"] and opaque["result_identity_sha256"] == IDENTITIES["opaque_import_identity_sha256"], "source:opaque")
    require(lock["scenario_count"] == 204 and lock["result_identity_sha256"] == IDENTITIES["distribution_identity_sha256"], "source:distribution")
    require(record["counts"]["combined_behavioral_mutations"] == record["counts"]["rust_behavioral_mutations"] + record["counts"]["independent_behavioral_mutations"], "source:combined")


def validate(record: object, schema: object) -> None:
    require(type(record) is dict and list(record) == FIELDS, "record:shape")
    assert isinstance(record, dict)
    require(record["schema"] == "nostr_automerge.causal_projection_combined_assurance.v15.v1" and record["status"] == "verified", "record:state")
    require(record["candidate"] == CANDIDATE and record["imports"] == IMPORTS and record["implementation_candidates"] == CANDIDATES, "record:bindings")
    resolved = subprocess.run(["git","rev-parse","--verify",f"{CANDIDATE}^{{commit}}"],cwd=ROOT,capture_output=True,text=True,check=False)
    require(resolved.returncode == 0 and resolved.stdout.strip() == CANDIDATE, "record:candidate")
    require(record["counts"] == COUNTS and record["identities"] == IDENTITIES and record["finding_closure"] == CLOSURE and record["result_classes"] == CLASSES, "record:evidence")
    require(record["canonical_process_bytes"] == "identical" and record["release_claimed"] is False and record["publication_claimed"] is False and record["remote_actions"] == 0 and record["result"] == "pass", "record:result")
    projection = {key: record[key] for key in FIELDS[:-1]}
    require(record["result_identity_sha256"] == IDENTITY == hashlib.sha256(canonical(projection)).hexdigest(), "record:identity")
    require(type(schema) is dict and list(schema) == ["title","type","additionalProperties","required","properties","$defs"] and schema["additionalProperties"] is False and schema["required"] == FIELDS and list(schema["properties"]) == FIELDS, "schema:shape")
    require(exact_schema_record(schema,"imports",list(IMPORTS)) and exact_schema_record(schema,"candidates",list(CANDIDATES)) and exact_schema_record(schema,"counts",list(COUNTS)) and exact_schema_record(schema,"identities",list(IDENTITIES)) and exact_schema_record(schema,"finding",["id","status","evidence"]), "schema:nested")
    validate_sources(record)


def run_distribution() -> bytes:
    result = subprocess.run(["cargo","extbuild","run","--","cargo","run","--quiet","-p","nostr_automerge_conformance","--locked","--","run_distribution","fixtures/distribution/manifest_v15.json"],cwd=ROOT,capture_output=True,check=False)
    require(result.returncode == 0, "distribution:exit")
    parsed = json.loads(result.stdout)
    require(parsed["status"] == "pass" and parsed["fixture_count"] == 204 and parsed["delivery_permutations"] == 8 and parsed["canonical_output_sha256"] == IDENTITIES["canonical_output_sha256"] and len(parsed["reports"]) == 204, "distribution:result")
    require(hashlib.sha256(result.stdout).hexdigest() == IDENTITIES["serialized_run_sha256"], "distribution:bytes")
    return result.stdout


def self_test(record: dict[str, Any], schema: dict[str, Any]) -> int:
    attacks = [
        lambda value: value.update(candidate="0"*40), lambda value: value["imports"].update(rust_ownership_sha256="0"*64),
        lambda value: value["implementation_candidates"].update(independent_implementation="0"*40), lambda value: value["counts"].update(operation_families=42),
        lambda value: value["counts"].update(rust_proofs=42), lambda value: value["counts"].update(rust_behavioral_mutations=12),
        lambda value: value["counts"].update(independent_behavioral_mutations=8), lambda value: value["counts"].update(mutation_survivors=1),
        lambda value: value["identities"].update(canonical_output_sha256="0"*64), lambda value: value["finding_closure"].reverse(),
        lambda value: value["finding_closure"][0].update(status="open"), lambda value: value["result_classes"].reverse(),
        lambda value: value.update(publication_claimed=True), lambda value: value.update(result_identity_sha256="0"*64), lambda value: value.update(extra=False),
    ]
    caught = 0
    for mutate in attacks:
        changed = copy.deepcopy(record); mutate(changed)
        try: validate(changed, schema)
        except AssuranceError: caught += 1; continue
        raise AssuranceError("mutation:record")
    coordinated = copy.deepcopy(record)
    coordinated["counts"]["rust_behavioral_mutations"] = 14
    coordinated["counts"]["combined_behavioral_mutations"] = 23
    coordinated["result_identity_sha256"] = hashlib.sha256(canonical({key: coordinated[key] for key in FIELDS[:-1]})).hexdigest()
    try: validate(coordinated, schema)
    except AssuranceError: caught += 1
    else: raise AssuranceError("mutation:coordinated")
    schema_attacks = [
        lambda value: value.update(additionalProperties=True), lambda value: value["required"].pop(),
        lambda value: value["$defs"]["imports"]["required"].pop(), lambda value: value["$defs"]["candidates"].update(additionalProperties=True),
        lambda value: value["$defs"]["counts"]["properties"].pop("mutation_survivors"),
    ]
    for mutate in schema_attacks:
        changed = copy.deepcopy(schema); mutate(changed)
        try: validate(record, changed)
        except AssuranceError: caught += 1; continue
        raise AssuranceError("mutation:schema")
    require(caught == 21, "mutation:count")
    return caught


def main() -> int:
    parser = argparse.ArgumentParser(); parser.add_argument("--run-conformance", action="store_true"); args = parser.parse_args()
    record = json.loads(REPORT.read_text()); schema = json.loads(SCHEMA.read_text()); validate(record, schema); mutations = self_test(record, schema); processes = 0
    if args.run_conformance:
        first = run_distribution(); second = run_distribution(); require(first == second, "distribution:process_identity"); processes = 2
    print(f"PASS: causal projection combined assurance v15 operations=43 proofs=43+43 mutations=22 survivors=0 scenarios=204x8 processes={processes} negative_mutations={mutations}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
