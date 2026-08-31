#!/usr/bin/env python3
"""Validate the public import of opaque independent v15 assurance."""
from __future__ import annotations
import copy, hashlib, json, sys
from pathlib import Path
from typing import Any
sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parents[1]
REPORT = ROOT / "reports/opaque_causal_projection_v15.json"
SCHEMA = ROOT / "tools/validation/opaque_causal_projection_v15.schema.json"
FIELDS = ["schema","status","independent_candidate","opaque_record_sha256","assurance","public_bindings","result","result_identity_sha256"]
ASSURANCE_FIELDS = ["schema","status","terminal_candidate","public_contract_identity_sha256","distribution_v15_identity_sha256","counts","canonical_output_sha256","result_classes","clean_tree","release_claimed","publication_claimed","remote_actions","result","identity_sha256"]
COUNTS = {"operation_families":43,"focused_proofs":43,"tests_passed":439,"tests_failed":0,"tests_skipped":15,"behavioral_mutations":9,"mutation_survivors":0,"scenarios":204,"signed_events":771,"delivery_orders":8,"processes":2,"budget_rebindings":9}
CLASSES = ["operation_ownership_complete","focused_stop_proofs_pass","behavioral_mutations_caught","distribution_v15_byte_identity","independent_implementation_boundary"]
INDEPENDENT = "2307800f980027bbe40ffc1312dde12f94ba2174"
TERMINAL = "1cbd985289ac35b9cf0f2fa3221b190ab1fb5c74"
OPAQUE_SHA = "1b5bd9c2f1d03049bcc4d3d4ae5286a6c7d6cc577e3e3ab1515b6752bb65b6e9"
OPAQUE_IDENTITY = "614cbbf92c2592e54337a170fc8c14792bb537994f211872cf312d1fec8d66de"
CONTRACT = "12dc6aca59ad0807757cc13c372b582e67bc70a7295fb741c15f5d91412ea078"
DISTRIBUTION = "be61110e2e1c3eb2dc7f30244e07a9efd6d0f4f1beae9693e77441506a35ac92"
CANONICAL = "e69c721549966b1b88dcde3296674d675169840c6e8ebd0f236a5c07bcfc6415"
BINDINGS = {"rust_candidate":"e4d418249585adcabaf1a94f4e6a31a1ce0ffb55","operation_contract_sha256":CONTRACT,"rust_ownership_sha256":"19813d525ee1ec037cfe3f789650789cae95a31dd1cf6db5b368a5fa0da625e6","proof_catalog_sha256":"9a4fa04c1c3be3934d3ef40d8573c16a955eca08092dd7e2f1ec8747580a7f96","mutation_qualification_sha256":"df292f4146d6c0d7e772af4ed457d9b79622dbed5fd76f32a5c3594ca0701edb","rust_conformance_sha256":"7ce224864f269e2818bc837d8252e6eeca8ee299a98604d4dd2d228d5c0ea6f5","distribution_manifest_sha256":"862d0c1ad6ae14cd54b75f88742fa3b584c6c3981195bfeb988818403bee689c","distribution_lock_sha256":"a511c18a540aaa5de5a7ef23cf6b360108a74e0e178c1e1025907ae880d78da7","distribution_identity_sha256":DISTRIBUTION,"canonical_output_sha256":CANONICAL,"logical_operation_families":43}
PATHS = {"operation_contract_sha256":"spec/causal_projection_operation_discovery_v15.json","rust_ownership_sha256":"reports/causal_projection_source_ownership_v15.json","proof_catalog_sha256":"reports/causal_projection_proof_catalog_v15.json","mutation_qualification_sha256":"reports/causal_projection_behavior_mutations_v15.json","rust_conformance_sha256":"reports/rust_conformance_v15.json","distribution_manifest_sha256":"fixtures/distribution/manifest_v15.json","distribution_lock_sha256":"fixtures/distribution/manifest_v15.lock.json"}
IDENTITY = "1c85a51f9f31d3541bc5a1762184c710e2fae71e1d199174dfb516ae19d22a1a"
class OpaqueError(RuntimeError): pass
def require(value: bool, label: str) -> None:
    if not value: raise OpaqueError(label)
def sha(path: str) -> str: return hashlib.sha256((ROOT / path).read_bytes()).hexdigest()
def canonical(value: Any) -> bytes: return json.dumps(value, ensure_ascii=False, sort_keys=True, separators=(",", ":")).encode()
def validate(record: object, schema: object) -> None:
    require(type(record) is dict and list(record) == FIELDS, "record:shape")
    assert isinstance(record, dict)
    require(record["schema"] == "nostr_automerge.opaque_causal_projection_import.v15.v1" and record["status"] == "code_complete_publication_held" and record["result"] == "pass", "record:state")
    require(record["independent_candidate"] == INDEPENDENT and record["opaque_record_sha256"] == OPAQUE_SHA, "record:source")
    assurance = record["assurance"]
    require(type(assurance) is dict and list(assurance) == ASSURANCE_FIELDS, "assurance:shape")
    require(assurance == {"schema":"nostr_automerge.opaque_causal_projection.v15.v1","status":"code_complete_publication_held","terminal_candidate":TERMINAL,"public_contract_identity_sha256":CONTRACT,"distribution_v15_identity_sha256":DISTRIBUTION,"counts":COUNTS,"canonical_output_sha256":CANONICAL,"result_classes":CLASSES,"clean_tree":True,"release_claimed":False,"publication_claimed":False,"remote_actions":0,"result":"pass","identity_sha256":OPAQUE_IDENTITY}, "assurance:value")
    require(hashlib.sha256(canonical({key: assurance[key] for key in ASSURANCE_FIELDS[:-1]})).hexdigest() == OPAQUE_IDENTITY, "assurance:identity")
    require(record["public_bindings"] == BINDINGS and all(sha(path) == BINDINGS[key] for key, path in PATHS.items()), "record:bindings")
    lock = json.loads((ROOT / "fixtures/distribution/manifest_v15.lock.json").read_text())
    require(lock["result_identity_sha256"] == DISTRIBUTION, "record:distribution")
    require(record["result_identity_sha256"] == IDENTITY == hashlib.sha256(canonical({key: record[key] for key in FIELDS[:-1]})).hexdigest(), "record:identity")
    require(type(schema) is dict and schema.get("type") == "object" and schema.get("additionalProperties") is False and schema.get("required") == FIELDS and list(schema.get("properties", {})) == FIELDS, "schema:closed")
    require(schema["properties"]["assurance"]["additionalProperties"] is False and schema["$defs"]["counts"]["additionalProperties"] is False, "schema:nested")
    bindings_schema = schema["properties"]["public_bindings"]
    require(bindings_schema.get("additionalProperties") is False and bindings_schema.get("required") == list(BINDINGS) and list(bindings_schema.get("properties", {})) == list(BINDINGS), "schema:bindings")
    serialized = json.dumps(record, separators=(",", ":"))
    for forbidden in ("src/","test/","scripts/","package.json","node_modules","command","credential","workflow",chr(104)+"ttps://",chr(102)+"ile://",chr(47)+"Users/","\\\\"):
        require(forbidden not in serialized, "record:leak:" + forbidden)
def self_test(record: dict[str, Any], schema: dict[str, Any]) -> int:
    attacks = [
        lambda value: value.update(independent_candidate="0"*40), lambda value: value.update(opaque_record_sha256="0"*64),
        lambda value: value["assurance"].update(terminal_candidate="0"*40), lambda value: value["assurance"]["counts"].update(operation_families=42),
        lambda value: value["assurance"]["counts"].update(mutation_survivors=1), lambda value: value["assurance"].update(identity_sha256="0"*64),
        lambda value: value["public_bindings"].update(rust_candidate="0"*40), lambda value: value["public_bindings"].update(distribution_identity_sha256="0"*64),
        lambda value: value.update(result_identity_sha256="0"*64), lambda value: value.update(extra=False),
    ]
    caught = 0
    for mutate in attacks:
        changed = copy.deepcopy(record); mutate(changed)
        try: validate(changed, schema)
        except OpaqueError: caught += 1; continue
        raise OpaqueError("mutation:record")
    coordinated = copy.deepcopy(record); coordinated["assurance"]["counts"]["tests_passed"] = 440; coordinated["assurance"]["identity_sha256"] = hashlib.sha256(canonical({key: coordinated["assurance"][key] for key in ASSURANCE_FIELDS[:-1]})).hexdigest(); coordinated["result_identity_sha256"] = hashlib.sha256(canonical({key: coordinated[key] for key in FIELDS[:-1]})).hexdigest()
    try: validate(coordinated, schema)
    except OpaqueError: caught += 1
    else: raise OpaqueError("mutation:coordinated")
    for mutate in (lambda value: value.update(additionalProperties=True), lambda value: value["required"].pop(), lambda value: value["properties"]["assurance"].update(additionalProperties=True), lambda value: value["properties"]["public_bindings"]["required"].pop()):
        changed = copy.deepcopy(schema); mutate(changed)
        try: validate(record, changed)
        except OpaqueError: caught += 1; continue
        raise OpaqueError("mutation:schema")
    require(caught == 15, "mutation:count"); return caught
def main() -> int:
    record = json.loads(REPORT.read_text()); schema = json.loads(SCHEMA.read_text()); validate(record, schema); mutations = self_test(record, schema)
    print(f"PASS: opaque causal projection v15 candidate={INDEPENDENT[:8]} operations=43 scenarios=204x8x2 mutations={mutations}"); return 0
if __name__ == "__main__": raise SystemExit(main())
