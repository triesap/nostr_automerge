#!/usr/bin/env python3
"""Validate the closed Rust causal-projection v16 assurance boundary."""

from __future__ import annotations

import argparse
import copy
import hashlib
import json
import subprocess
import sys
from pathlib import Path
from typing import Any, Callable

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parents[1]
REPORT = ROOT / "reports/causal_projection_rust_assurance_v16.json"
SCHEMA = ROOT / "tools/validation/causal_projection_rust_assurance_v16.schema.json"
CANDIDATE = "f52fdb9da47ccb6cb9dbc25c7b50954679d972b2"
PARENT = "a696e41dbc6eb966b3657a47331f1ed072308a0b"
SOURCE_BINDINGS = [
    {"role":"actor","path":"crates/nostr_automerge/src/graph/actor_state.rs","sha256":"101e9502101d7c08d11dadafc46c679a084bfe88b8ea8614c79682565c3bbc0e"},
    {"role":"control","path":"crates/nostr_automerge/src/control/epoch_state.rs","sha256":"734f70b9eed8f4281d719b0581153db1175bbe1401c11fcd0c0ef59b36343221"},
    {"role":"consumer","path":"crates/nostr_automerge/src/reference/epoch_engine.rs","sha256":"0f7e948b27b6cc0d7b921596bde5bba496ef72fca43c6f4e485a68a1919c4315"},
]
IMPORT_PATHS = {
    "operation_inventory_sha256":"reports/causal_projection_operation_inventory_v16.json",
    "proof_catalog_sha256":"reports/causal_projection_proof_catalog_v16.json",
    "structural_assurance_sha256":"reports/causal_projection_structural_assurance_v16.json",
    "mutation_qualification_sha256":"reports/causal_projection_mutations_v16.json",
    "contracts_sha256":"spec/causal_projection_contracts_v16.json",
    "distribution_manifest_v15_sha256":"fixtures/distribution/manifest_v15.json",
    "distribution_lock_v15_sha256":"fixtures/distribution/manifest_v15.lock.json",
    "rust_conformance_v15_sha256":"reports/rust_conformance_v15.json",
}
IMPORTS = {
    "operation_inventory_sha256":"95562a0f032c6fcedf3e397f82f42072fa2179b30a48b7424e38c2bf39403de1",
    "proof_catalog_sha256":"486dd1f70a108166a5380ef533f707f1aeebac6b4f5b2d1f20708a9a4e0f4ca0",
    "structural_assurance_sha256":"fbd2b12e558f54d161dc778e189cccacd391c51db1ecbb89e0c58a535076c9d1",
    "mutation_qualification_sha256":"d4d88b74b5de2f73a46f17436c62aa185519ecaed04bf5868dcf93ebd5e9e490",
    "contracts_sha256":"bbd58073a7dab83d7a96541ba7d1a90e0ceb5c4876bb4533d7b196058b5e7b3b",
    "distribution_manifest_v15_sha256":"862d0c1ad6ae14cd54b75f88742fa3b584c6c3981195bfeb988818403bee689c",
    "distribution_lock_v15_sha256":"a511c18a540aaa5de5a7ef23cf6b360108a74e0e178c1e1025907ae880d78da7",
    "rust_conformance_v15_sha256":"7ce224864f269e2818bc837d8252e6eeca8ee299a98604d4dd2d228d5c0ea6f5",
}
COUNTS = {"operation_sites":68,"operation_families":38,"proofs":68,"property_codes":10,"behavioral_mutations":13,"mutation_survivors":0,"consumer_bindings":3,"scenarios":204,"signed_events":771,"delivery_orders":8,"processes":2}
PHASES = {"projection_construction":50,"actor_sequence":4,"causal_counter_consumer":3,"frontier_comparison":11}
PROPERTY_CODES = ["UNWRAPPED_ACTOR_SEQUENCE_DECISION","CAUSAL_STAGE_BEFORE_ACTOR_SUCCESS","DUPLICATE_CAUSAL_START_COMPARISON","UNMETERED_FINAL_TRAVERSAL","STATE_WRITE_BEFORE_CHARGE","CHARGE_AFTER_OPERATION","POST_STOP_TARGET_WORK","PUBLICATION_BEFORE_CHARGE","ALTERNATE_CONSUMER_BYPASS","COUNTER_MISMATCH"]
CONSUMERS = [
    {"source":"control.new_metered","target":"initialize_actor_states_metered","count":1},
    {"source":"reference.evaluate_epoch","target":"initialize_actor_states_metered","count":1},
    {"source":"reference.evaluate_epoch","target":"candidate_semantics_decision_metered","count":1},
]
CONFORMANCE = {"manifest":"fixtures/distribution/manifest_v15.json","canonical_process_bytes":"identical","canonical_output_sha256":"e69c721549966b1b88dcde3296674d675169840c6e8ebd0f236a5c07bcfc6415","serialized_run_sha256":"000c52bde7102eaccec8cf65c875332e119fd25ccf4a2ac38973c456774a3344"}
ASSURANCE = {"inventory":"complete","proofs":"complete","structure":"complete","identity":"complete","mutations":"zero_survivors","consumers":"closed","dependency_count_counter":"GraphNode","applicability":"source_derived"}
FIELDS = ["schema","status","candidate","parent_candidate","source_bindings","imports","counts","phase_counts","property_codes","consumer_bindings","conformance","assurance","release_claimed","publication_claimed","remote_actions","result_identity_sha256","result"]


class AssuranceError(RuntimeError):
    pass


def require(condition: bool, label: str) -> None:
    if not condition:
        raise AssuranceError(label)


def canonical(value: object) -> bytes:
    return json.dumps(value, ensure_ascii=True, separators=(",", ":"), sort_keys=True).encode()


def sha(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def git(*args: str) -> str:
    completed = subprocess.run(["git", *args], cwd=ROOT, capture_output=True, text=True, check=False)
    require(completed.returncode == 0, "git:" + ":".join(args))
    return completed.stdout.strip()


def exact_schema_record(schema: dict[str, Any], name: str, fields: list[str]) -> None:
    value = schema["$defs"][name]
    require(value.get("additionalProperties") is False and value.get("required") == fields and list(value.get("properties", {})) == fields, "schema:" + name)


def validate_sources() -> None:
    require({key: sha(ROOT / path) for key, path in IMPORT_PATHS.items()} == IMPORTS, "imports:hash")
    inventory = json.loads((ROOT / IMPORT_PATHS["operation_inventory_sha256"]).read_text())
    proofs = json.loads((ROOT / IMPORT_PATHS["proof_catalog_sha256"]).read_text())
    structural = json.loads((ROOT / IMPORT_PATHS["structural_assurance_sha256"]).read_text())
    mutations = json.loads((ROOT / IMPORT_PATHS["mutation_qualification_sha256"]).read_text())
    conformance = json.loads((ROOT / IMPORT_PATHS["rust_conformance_v15_sha256"]).read_text())
    lock = json.loads((ROOT / IMPORT_PATHS["distribution_lock_v15_sha256"]).read_text())
    rows = inventory["rows"]
    require(len(rows) == len({row["id"] for row in rows}) == 68, "inventory:rows")
    require(len({row["abstract_family"] for row in rows}) == 38 and inventory["counts"] == {"rows":68,"families":38,"phases":PHASES}, "inventory:counts")
    dependency = [row for row in rows if row["abstract_family"] == "projection_construction.dependency_count_read"]
    require(len(dependency) == 1 and dependency[0]["counter"] == "graph_node", "inventory:dependency_counter")
    require(proofs["row_count"] == 68 and len(proofs["rows"]) == 68 and len({row["test"] for row in proofs["rows"]}) == 68 and all(row["result"] == "pass" for row in proofs["rows"]), "proofs:coverage")
    require([row["id"] for row in proofs["rows"]] == [row["id"] for row in rows], "proofs:order")
    require(structural["property_codes"] == PROPERTY_CODES and structural["structural_summary"]["phases"] == PHASES and structural["structural_summary"]["consumer_bindings"] == 3 and structural["result"] == "pass", "structural:coverage")
    require(mutations["mutation_count"] == 13 and mutations["compile_failures"] == 0 and mutations["survivors"] == 0 and len(mutations["mutations"]) == 13 and all(row["result"] == "killed" for row in mutations["mutations"]), "mutations:coverage")
    require(lock["scenario_count"] == 204 and conformance["scenario_count"] == 204 and conformance["process_count"] == 2 and conformance["delivery_order_count"] == 8 and conformance["canonical_process_bytes"] == "identical" and conformance["canonical_output_sha256"] == CONFORMANCE["canonical_output_sha256"] and conformance["serialized_run_sha256"] == CONFORMANCE["serialized_run_sha256"], "conformance:coverage")


def validate(record: object, schema: object) -> None:
    require(type(record) is dict and list(record) == FIELDS, "record:shape")
    assert isinstance(record, dict)
    require(record["schema"] == "nostr_automerge.causal_projection_rust_assurance.v16.v1" and record["status"] == "pass" and record["result"] == "pass", "record:state")
    require(record["candidate"] == CANDIDATE and record["parent_candidate"] == PARENT and git("rev-parse", CANDIDATE + "^{commit}") == CANDIDATE and git("rev-parse", CANDIDATE + "^") == PARENT, "record:candidate")
    require(record["source_bindings"] == SOURCE_BINDINGS and record["imports"] == IMPORTS and record["counts"] == COUNTS and record["phase_counts"] == PHASES, "record:bindings")
    require(record["property_codes"] == PROPERTY_CODES and record["consumer_bindings"] == CONSUMERS and record["conformance"] == CONFORMANCE and record["assurance"] == ASSURANCE, "record:assurance")
    require(record["release_claimed"] is False and record["publication_claimed"] is False and record["remote_actions"] == 0, "record:holds")
    for binding in SOURCE_BINDINGS:
        committed = subprocess.run(["git","show",f"{CANDIDATE}:{binding['path']}"], cwd=ROOT, capture_output=True, check=False)
        require(committed.returncode == 0 and hashlib.sha256(committed.stdout).hexdigest() == binding["sha256"] and sha(ROOT / binding["path"]) == binding["sha256"], "source:" + binding["role"])
    projection = {key: record[key] for key in FIELDS if key != "result_identity_sha256"}
    require(record["result_identity_sha256"] == hashlib.sha256(canonical(projection)).hexdigest(), "record:identity")
    require(type(schema) is dict and list(schema) == ["$schema","title","type","additionalProperties","required","properties","$defs"] and schema["additionalProperties"] is False and schema["required"] == FIELDS and list(schema["properties"]) == FIELDS, "schema:shape")
    exact_schema_record(schema, "source_binding", ["role","path","sha256"])
    exact_schema_record(schema, "imports", list(IMPORTS))
    exact_schema_record(schema, "counts", list(COUNTS))
    exact_schema_record(schema, "phases", list(PHASES))
    exact_schema_record(schema, "consumer", ["source","target","count"])
    exact_schema_record(schema, "conformance", list(CONFORMANCE))
    exact_schema_record(schema, "assurance", list(ASSURANCE))
    validate_sources()


def self_test(record: dict[str, Any], schema: dict[str, Any]) -> int:
    attacks: list[tuple[str, Callable[[dict[str, Any]], None]]] = [
        ("record", lambda value: value.update(candidate="0" * 40)),
        ("record", lambda value: value["source_bindings"].reverse()),
        ("record", lambda value: value["imports"].update(operation_inventory_sha256="0" * 64)),
        ("record", lambda value: value["counts"].update(operation_sites=67)),
        ("record", lambda value: value["phase_counts"].update(actor_sequence=3)),
        ("record", lambda value: value["property_codes"].reverse()),
        ("record", lambda value: value["consumer_bindings"].pop()),
        ("record", lambda value: value["conformance"].update(canonical_output_sha256="0" * 64)),
        ("record", lambda value: value["assurance"].update(dependency_count_counter="GraphEdge")),
        ("record", lambda value: value.update(remote_actions=1)),
        ("record", lambda value: value.update(result_identity_sha256="0" * 64)),
        ("record", lambda value: value.update(extra=False)),
        ("schema", lambda value: value.update(additionalProperties=True)),
        ("schema", lambda value: value["required"].pop()),
        ("schema", lambda value: value["$defs"]["counts"].update(additionalProperties=True)),
    ]
    caught = 0
    for target, mutate in attacks:
        changed_record = copy.deepcopy(record)
        changed_schema = copy.deepcopy(schema)
        mutate(changed_record if target == "record" else changed_schema)
        try:
            validate(changed_record, changed_schema)
        except AssuranceError:
            caught += 1
            continue
        raise AssuranceError("mutation_survived:" + target)
    coordinated = copy.deepcopy(record)
    coordinated["counts"]["operation_sites"] = 67
    coordinated["result_identity_sha256"] = hashlib.sha256(canonical({key: coordinated[key] for key in FIELDS if key != "result_identity_sha256"})).hexdigest()
    try:
        validate(coordinated, schema)
    except AssuranceError:
        caught += 1
    else:
        raise AssuranceError("mutation_survived:coordinated")
    require(caught == 16, "mutation:count")
    return caught


def run_conformance() -> bytes:
    completed = subprocess.run(["cargo","run","--quiet","-p","nostr_automerge_conformance","--locked","--","run_distribution",CONFORMANCE["manifest"]], cwd=ROOT, capture_output=True, check=False)
    require(completed.returncode == 0 and hashlib.sha256(completed.stdout).hexdigest() == CONFORMANCE["serialized_run_sha256"], "conformance:bytes")
    value = json.loads(completed.stdout)
    require(value["status"] == "pass" and value["fixture_count"] == 204 and value["delivery_permutations"] == 8 and value["canonical_output_sha256"] == CONFORMANCE["canonical_output_sha256"] and len(value["reports"]) == 204, "conformance:result")
    return completed.stdout


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--run-conformance", action="store_true")
    args = parser.parse_args()
    record = json.loads(REPORT.read_text())
    schema = json.loads(SCHEMA.read_text())
    validate(record, schema)
    mutations = self_test(record, schema)
    processes = 0
    if args.run_conformance:
        first = run_conformance()
        second = run_conformance()
        require(first == second, "conformance:process_identity")
        processes = 2
    print(f"PASS: causal projection Rust assurance v16 sites=68 proofs=68 mutations=13 survivors=0 scenarios=204x8 processes={processes} negative_mutations={mutations}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
