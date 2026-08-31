#!/usr/bin/env python3
"""Validate the distinct candidate-consumer operation inventory and proofs."""

from __future__ import annotations

import argparse
import copy
import hashlib
import json
import subprocess
import sys
from pathlib import Path

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parents[1]
REPORT = ROOT / "reports/causal_projection_consumer_inventory_v15.json"
SCHEMA = ROOT / "tools/validation/causal_projection_consumer_inventory_v15.schema.json"
SOURCE = ROOT / "crates/nostr_automerge/src/graph/actor_state.rs"
FIELDS = ["schema","status","implementation_predecessor","source_path","source_symbol","source_projection_sha256","language_applicability","rows","inactive_families_removed","result"]
ROW_FIELDS = ["id","family","phase","counter","owner_mode","reachability_count","proof_test","command"]
FAMILIES = ["StoredCounterRead","ExpectedStartComparison","CheckedAdvance"]
IDS = ["stored_counter_read","expected_start_comparison","checked_advance"]
TESTS = [f"graph::actor_state::tests::causal_consumer_{value}_is_owned" for value in IDS]
SOURCE_PROJECTION_SHA256 = "1e04143b6e6b7fc97c4d29074045b6c8ae26737f738f8e17f5d6ff87f3a8f7ca"

sys.path.insert(0, str(ROOT / "scripts"))
from validate_causal_projection_discovery_v15 import enum_variants  # noqa: E402
from validate_causal_projection_source_v13 import function_body  # noqa: E402


class ConsumerInventoryError(RuntimeError):
    pass


def require(condition: bool, label: str) -> None:
    if not condition:
        raise ConsumerInventoryError(label)


def source_projection(source: str) -> str:
    value = {
        "causal_next_operation": enum_variants(source, "CausalNextOperation"),
        "causal_next_decision_metered_observed": function_body(
            source, "causal_next_decision_metered_observed"
        ),
    }
    canonical = json.dumps(value, ensure_ascii=True, separators=(",", ":"), sort_keys=True)
    return hashlib.sha256(canonical.encode()).hexdigest()


def validate(report: object, schema: object, source: str) -> None:
    require(type(report) is dict and list(report) == FIELDS, "report:shape")
    require(report["schema"] == "nostr_automerge.causal_projection_consumer_inventory.v15.v1" and report["status"] == "pass" and report["result"] == "pass", "report:state")
    require(report["implementation_predecessor"] == "2e9ba998fa89a45a7cd617ccdd5ceb1f04e6dade", "report:predecessor")
    resolved = subprocess.run(["git","rev-parse","--verify",report["implementation_predecessor"] + "^{commit}"], cwd=ROOT, capture_output=True, text=True, check=False)
    require(resolved.returncode == 0 and resolved.stdout.strip() == report["implementation_predecessor"], "report:predecessor_commit")
    require(report["source_path"] == "crates/nostr_automerge/src/graph/actor_state.rs" and report["source_symbol"] == "causal_next_decision_metered_observed", "report:source")
    projection = source_projection(source)
    require(projection == SOURCE_PROJECTION_SHA256 and report["source_projection_sha256"] == projection, "report:source_projection")
    require(report["language_applicability"] == [{"language":"rust","status":"required_active"},{"language":"typescript","status":"required_private_sequence"}], "report:applicability")
    rows = report["rows"]
    require(type(rows) is list and len(rows) == 3, "rows:count")
    require([row["id"] for row in rows] == IDS and [row["family"] for row in rows] == FAMILIES, "rows:identity")
    require([row["proof_test"] for row in rows] == TESTS and len(set(TESTS)) == 3, "rows:proof")
    for index, row in enumerate(rows):
        require(type(row) is dict and list(row) == ROW_FIELDS, f"row:{index}:shape")
        require(row["phase"] == "candidate_consumer" and row["counter"] == "graph_node" and row["owner_mode"] == "item_metered" and row["reachability_count"] == 1, f"row:{index}:owner")
        command = f"cargo test -p nostr_automerge --lib {TESTS[index]} --locked -- --exact"
        require(row["command"] == command, f"row:{index}:command")
        short = TESTS[index].rsplit("::", 1)[1]
        require(source.count(f"fn {short}()") == 1, f"row:{index}:test")
    require(report["inactive_families_removed"] == ["ConstantCandidateValidation","SharedReferenceClone"], "report:inactive")
    require(enum_variants(source, "CausalNextOperation") == FAMILIES, "source:consumer_enum")
    require("SharedReferenceClone" not in source and "ConstantCandidateValidation" not in source, "source:inactive")
    body = function_body(source, "causal_next_decision_metered_observed")
    blocks = [
        "charge(WorkCounter::GraphNode).map_err(MeteredActorStateError::Work)?;\n        let causal_next_op = self.causal_next_op;\n        observed(CausalNextOperation::StoredCounterRead);",
        "charge(WorkCounter::GraphNode).map_err(MeteredActorStateError::Work)?;\n        let start_matches = candidate.start_op == causal_next_op;\n        observed(CausalNextOperation::ExpectedStartComparison);",
        "charge(WorkCounter::GraphNode).map_err(MeteredActorStateError::Work)?;\n        let advanced = causal_next_op.checked_add(candidate.operation_count);\n        observed(CausalNextOperation::CheckedAdvance);",
    ]
    require(all(body.count(block) == 1 for block in blocks), "source:immediate")
    require(type(schema) is dict and schema.get("additionalProperties") is False and schema.get("required") == FIELDS, "schema:closed")
    require(schema["properties"]["rows"]["minItems"] == schema["properties"]["rows"]["maxItems"] == 3, "schema:rows")


def run_proofs(report: dict) -> None:
    for row in report["rows"]:
        completed = subprocess.run(row["command"].split(), cwd=ROOT, capture_output=True, text=True, check=False)
        output = completed.stdout + completed.stderr
        require(completed.returncode == 0, "proof:exit:" + row["id"])
        require(f"test {row['proof_test']} ... ok" in output and "1 passed; 0 failed; 0 ignored" in output, "proof:transcript:" + row["id"])


def self_test(report: dict, schema: dict, source: str) -> int:
    cases = [
        ("missing_row","report",lambda value: value["rows"].pop()),
        ("row_order","report",lambda value: value["rows"].reverse()),
        ("duplicate_row","report",lambda value: value["rows"].__setitem__(1,copy.deepcopy(value["rows"][0]))),
        ("zero_reachability","report",lambda value: value["rows"][0].update(reachability_count=0)),
        ("shared_proof","report",lambda value: value["rows"][1].update(proof_test=value["rows"][0]["proof_test"])),
        ("applicability","report",lambda value: value["language_applicability"][1].update(status="not_applicable")),
        ("inactive","report",lambda value: value["inactive_families_removed"].pop()),
        ("projection","report",lambda value: value.update(source_projection_sha256="0"*64)),
        ("schema","schema",lambda value: value.update(additionalProperties=True)),
        ("consumer_removed","source",lambda value: value.replace("CausalNextOperation::StoredCounterRead", "CausalNextOperation::CheckedAdvance", 1)),
        ("charge_moved","source",lambda value: value.replace("let causal_next_op = self.causal_next_op;\n        observed(CausalNextOperation::StoredCounterRead);", "observed(CausalNextOperation::StoredCounterRead);\n        let causal_next_op = self.causal_next_op;", 1)),
        ("inactive_restored","source",lambda value: value.replace("enum ProjectionBuildOperation {", "enum ProjectionBuildOperation {\n    SharedReferenceClone,", 1)),
    ]
    caught = 0
    for label, target, mutate in cases:
        changed_report = copy.deepcopy(report)
        changed_schema = copy.deepcopy(schema)
        changed_source = source
        if target == "report": mutate(changed_report)
        elif target == "schema": mutate(changed_schema)
        else: changed_source = mutate(changed_source)
        try:
            validate(changed_report, changed_schema, changed_source)
        except ConsumerInventoryError:
            caught += 1
            continue
        raise ConsumerInventoryError("mutation_survived:" + label)
    return caught


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--run-proofs", action="store_true")
    args = parser.parse_args()
    report = json.loads(REPORT.read_text())
    schema = json.loads(SCHEMA.read_text())
    source = SOURCE.read_text()
    if SOURCE_PROJECTION_SHA256 == "SOURCE_PROJECTION_PLACEHOLDER":
        print(source_projection(source))
        return 2
    validate(report, schema, source)
    mutations = self_test(report, schema, source)
    if args.run_proofs:
        run_proofs(report)
    print(f"PASS: causal projection consumer rows=3 mutations={mutations} proofs={3 if args.run_proofs else 0}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
