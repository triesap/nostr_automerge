#!/usr/bin/env python3
"""Validate v17 causal-projection structure without candidate/hash checks."""

from __future__ import annotations

import argparse
import copy
import json
import re
import sys
from pathlib import Path
from typing import Any

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parents[1]
SOURCE = ROOT / "crates/nostr_automerge/src/graph/actor_state.rs"
CONSUMER = ROOT / "crates/nostr_automerge/src/reference/epoch_engine.rs"
INVENTORY = ROOT / "reports/causal_projection_inventory_v17.json"
PROPERTIES = ROOT / "reports/causal_projection_properties_v17.json"
REPORT = ROOT / "reports/causal_projection_structure_v17.json"
SCHEMA = ROOT / "tools/validation/causal_projection_structure_v17.schema.json"
PROPERTY_CODES = [
    "TYPED_BUDGET_EXHAUSTED_IDENTITY", "TYPED_CANCELLED_IDENTITY",
    "UNEXPECTED_WORK_ERROR_IDENTITY", "CHARGE_AFTER_OPERATION",
    "TARGET_AFTER_STOP", "OBSERVATION_AFTER_STOP", "PUBLICATION_AFTER_STOP",
    "SITE_ID_MISMATCH", "COUNTER_MISMATCH", "ALTERNATE_CONSUMER_BYPASS",
]
HELPERS = {
    "projection_construction": ("ProjectionBuildSite", "perform_projection_build_operation"),
    "actor_sequence": ("ActorDecisionSite", "perform_actor_decision_operation"),
    "causal_counter": ("CausalNextSite", "perform_causal_next_operation"),
    "frontier_comparison": ("FrontierComparisonSite", "metered_frontier_operation"),
}
MUTATIONS = [
    ("helper_bypass", "ALTERNATE_CONSUMER_BYPASS"),
    ("target_before_charge", "CHARGE_AFTER_OPERATION"),
    ("descriptor_mismatch", "SITE_ID_MISMATCH"),
    ("counter_mismatch", "COUNTER_MISMATCH"),
    ("alternate_consumer", "ALTERNATE_CONSUMER_BYPASS"),
    ("target_after_stop", "TARGET_AFTER_STOP"),
    ("observation_after_stop", "OBSERVATION_AFTER_STOP"),
    ("publication_after_stop", "PUBLICATION_AFTER_STOP"),
    ("typed_stop_substitution", "TYPED_BUDGET_EXHAUSTED_IDENTITY"),
    ("unexpected_error_substitution", "UNEXPECTED_WORK_ERROR_IDENTITY"),
]

sys.path.insert(0, str(ROOT / "scripts"))
from validate_causal_projection_inventory_v17 import derive_rows, production  # noqa: E402
from validate_causal_projection_source_v13 import function_body  # noqa: E402
from validate_report_contract_v9 import ReportSuiteError, rust_code_view  # noqa: E402


class StructuralError(RuntimeError):
    def __init__(self, code: str):
        super().__init__(code)
        self.code = code


def require(condition: bool, code: str) -> None:
    if not condition:
        raise StructuralError(code)


def code_view(source: str) -> str:
    try:
        return rust_code_view(source)
    except ReportSuiteError as error:
        raise StructuralError("SITE_ID_MISMATCH") from error


def helper_structure(source: str, name: str) -> None:
    body = code_view(function_body(source, name))
    descriptor = body.find("let descriptor = site.descriptor();")
    attempt = body.find("ChargeAttempt")
    charge = body.find("charge(descriptor.counter)")
    target = body.find("let result = target();" if name == "metered_frontier_operation" else "let result = perform();")
    completion = body.find("TargetCompleted")
    returned = body.find("Ok(result)")
    require(min(descriptor, attempt, charge, returned) >= 0, "ALTERNATE_CONSUMER_BYPASS")
    require(target >= 0, "TARGET_AFTER_STOP")
    target_call = "target();" if name == "metered_frontier_operation" else "perform();"
    require(body.count(target_call) == 1, "TARGET_AFTER_STOP")
    require(completion >= 0, "OBSERVATION_AFTER_STOP")
    require(descriptor < attempt < charge, "CHARGE_AFTER_OPERATION")
    require(charge < target, "CHARGE_AFTER_OPERATION")
    charge_tail = body[charge:target]
    require("?;" in charge_tail, "TARGET_AFTER_STOP")
    require(target < completion < returned, "OBSERVATION_AFTER_STOP")
    signature = code_view(source[source.find(f"fn {name}"):source.find(f"fn {name}") + 350])
    require("site:" in signature and "counter:" not in signature and "family:" not in signature, "SITE_ID_MISMATCH")


def validate_structure(source: str, consumer: str, inventory: dict[str, Any], properties: dict[str, Any]) -> None:
    source = production(source)
    require("_v17_typed_stop_collapsed" not in source, "TYPED_BUDGET_EXHAUSTED_IDENTITY")
    require("_v17_cancellation_collapsed" not in source, "TYPED_CANCELLED_IDENTITY")
    require("_v17_unexpected_error_replaced" not in source, "UNEXPECTED_WORK_ERROR_IDENTITY")
    require("_uncharged_second_result" not in source, "TARGET_AFTER_STOP")
    require("_v17_charge_removed" not in source, "CHARGE_AFTER_OPERATION")
    try:
        current = derive_rows(source)
    except Exception as error:
        raise StructuralError("SITE_ID_MISMATCH") from error
    expected_sites = [(row["phase"], row["site_id"], row["operation"]) for row in inventory["rows"]]
    current_sites = [(row["phase"], row["site_id"], row["operation"]) for row in current]
    require(current_sites == expected_sites, "SITE_ID_MISMATCH")
    expected_counters = [row["counter"] for row in inventory["rows"]]
    require([row["counter"] for row in current] == expected_counters, "COUNTER_MISMATCH")
    for phase, (enum, helper) in HELPERS.items():
        helper_structure(source, helper)
        for row in (row for row in inventory["rows"] if row["phase"] == phase):
            require(len(re.findall(rf"\b{helper}\s*\(\s*{enum}::{row['site_id']}\b", code_view(source))) == 1, "ALTERNATE_CONSUMER_BYPASS")
    publish = function_body(source, "build_trusted_epoch_projection_observed")
    site = publish.find("ProjectionBuildSite::ProjectionPublish")
    stop = publish.find(")?;", site)
    observation = publish.find("published(ProjectionPublicationOperation::Projection);", site)
    require(site >= 0 and stop > site and observation > stop, "PUBLICATION_AFTER_STOP")
    require(consumer.count(".candidate_semantics_decision_metered(") == 1, "ALTERNATE_CONSUMER_BYPASS")
    require(properties["property_codes"] == PROPERTY_CODES, "TYPED_BUDGET_EXHAUSTED_IDENTITY")
    require(properties["result_classes"]["TYPED_CANCELLED_IDENTITY"] == "typed_stop_provenance", "TYPED_CANCELLED_IDENTITY")
    require(properties["result_classes"]["UNEXPECTED_WORK_ERROR_IDENTITY"] == "unexpected_error_provenance", "UNEXPECTED_WORK_ERROR_IDENTITY")


def mutated_inputs(label: str, source: str, consumer: str, properties: dict[str, Any]) -> tuple[str, str, dict[str, Any]]:
    changed_source, changed_consumer, changed_properties = source, consumer, copy.deepcopy(properties)
    if label == "helper_bypass":
        changed_source = source.replace("perform_projection_build_operation(\n        ProjectionBuildSite::MemberCountRead", "bypass_projection_build_operation(\n        ProjectionBuildSite::MemberCountRead", 1)
    elif label == "target_before_charge":
        changed_source = source.replace("charge(descriptor.counter).map_err(MeteredActorStateError::Work)?;\n    let result = perform();", "let result = perform();\n    charge(descriptor.counter).map_err(MeteredActorStateError::Work)?;", 1)
    elif label == "descriptor_mismatch":
        changed_source = source.replace("MemberCountRead => (SourceCountRead, GraphNode)", "MemberCountRead => (CandidateLookup, GraphNode)", 1)
    elif label == "counter_mismatch":
        changed_source = source.replace("MemberCountRead => (SourceCountRead, GraphNode)", "MemberCountRead => (SourceCountRead, GraphEdge)", 1)
    elif label == "alternate_consumer":
        changed_consumer = consumer.replace(".candidate_semantics_decision_metered(", ".candidate_semantics_decision_unmetered(", 1)
    elif label == "target_after_stop":
        changed_source = source.replace("charge(descriptor.counter).map_err(MeteredActorStateError::Work)?;", "charge(descriptor.counter).map_err(MeteredActorStateError::Work);", 1)
    elif label == "observation_after_stop":
        changed_source = source.replace("kind: ActorDecisionObservationKind::TargetCompleted", "kind: ActorDecisionObservationKind::ChargeAttempt", 1)
    elif label == "publication_after_stop":
        changed_source = source.replace(")?;\n    published(ProjectionPublicationOperation::Projection);", ");\n    published(ProjectionPublicationOperation::Projection);", 1)
    elif label == "typed_stop_substitution":
        changed_properties["property_codes"][0] = "TYPED_CANCELLED_IDENTITY"
    elif label == "unexpected_error_substitution":
        changed_properties["result_classes"]["UNEXPECTED_WORK_ERROR_IDENTITY"] = "typed_stop_provenance"
    return changed_source, changed_consumer, changed_properties


def exercise(source: str, consumer: str, inventory: dict[str, Any], properties: dict[str, Any]) -> list[dict[str, str]]:
    validate_structure(source, consumer, inventory, properties)
    validate_structure(source + "\n// neutral structural comment\n", consumer, inventory, properties)
    results = []
    for label, expected in MUTATIONS:
        changed = mutated_inputs(label, source, consumer, properties)
        try:
            validate_structure(changed[0], changed[1], inventory, changed[2])
        except StructuralError as error:
            require(error.code == expected, f"WRONG_PROPERTY:{label}:{error.code}")
            results.append({"mutation": label, "expected_code": expected, "actual_code": error.code, "result": "killed"})
            continue
        raise StructuralError(f"MUTATION_SURVIVED:{label}")
    return results


def expected_report(results: list[dict[str, str]]) -> dict[str, Any]:
    return {
        "schema": "nostr_automerge.causal_projection_structure.v17.v1",
        "status": "structural_only",
        "mode": "structural",
        "source_path": "crates/nostr_automerge/src/graph/actor_state.rs",
        "site_count": 68,
        "helper_checks": ["site_only", "descriptor", "charge", "target_once", "completion", "return"],
        "consumer_checks": ["publication_after_charge", "canonical_epoch_consumer"],
        "mutation_matrix": results,
        "neutral_comment": "pass",
        "result": "pass",
    }


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--mode", choices=("structural",), default="structural")
    parser.add_argument("--write-report", action="store_true")
    args = parser.parse_args()
    inventory = json.loads(INVENTORY.read_text())
    properties = json.loads(PROPERTIES.read_text())
    results = exercise(SOURCE.read_text(), CONSUMER.read_text(), inventory, properties)
    expected = expected_report(results)
    if args.write_report:
        REPORT.write_text(json.dumps(expected, ensure_ascii=True, indent=2) + "\n")
    report = json.loads(REPORT.read_text())
    schema = json.loads(SCHEMA.read_text())
    require(report == expected, "REPORT_MISMATCH")
    require(schema.get("additionalProperties") is False and schema.get("required") == list(expected), "SCHEMA_CLOSED")
    print(f"PASS: causal projection structure v17 sites=68 mutations={len(results)} neutral_comment=pass")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
