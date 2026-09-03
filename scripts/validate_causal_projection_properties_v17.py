#!/usr/bin/env python3
"""Validate the v17 property-result vocabulary and its independent classifiers."""

from __future__ import annotations

import argparse
import copy
import json
from dataclasses import dataclass, replace
from pathlib import Path
from typing import Final

ROOT = Path(__file__).resolve().parents[1]
REPORT = ROOT / "reports/causal_projection_properties_v17.json"
SCHEMA = ROOT / "tools/validation/causal_projection_properties_v17.schema.json"

PROPERTY_CODES: Final = (
    "TYPED_BUDGET_EXHAUSTED_IDENTITY",
    "TYPED_CANCELLED_IDENTITY",
    "UNEXPECTED_WORK_ERROR_IDENTITY",
    "CHARGE_AFTER_OPERATION",
    "TARGET_AFTER_STOP",
    "OBSERVATION_AFTER_STOP",
    "PUBLICATION_AFTER_STOP",
    "SITE_ID_MISMATCH",
    "COUNTER_MISMATCH",
    "ALTERNATE_CONSUMER_BYPASS",
)


class PropertyFailure(RuntimeError):
    """A structural or runtime evidence failure with one stable property code."""

    def __init__(self, code: str):
        super().__init__(code)
        self.code = code


@dataclass(frozen=True)
class Observation:
    budget_identity: bool = True
    cancelled_identity: bool = True
    unexpected_identity: bool = True
    charge_before_target: bool = True
    target_after_stop: int = 0
    observation_after_stop: int = 0
    publication_after_stop: int = 0
    requested_site: str = "site"
    observed_site: str = "site"
    expected_counter: str = "graph_node"
    observed_counter: str = "graph_node"
    canonical_consumer: bool = True


CHECKS: Final = (
    ("TYPED_BUDGET_EXHAUSTED_IDENTITY", lambda row: row.budget_identity),
    ("TYPED_CANCELLED_IDENTITY", lambda row: row.cancelled_identity),
    ("UNEXPECTED_WORK_ERROR_IDENTITY", lambda row: row.unexpected_identity),
    ("CHARGE_AFTER_OPERATION", lambda row: row.charge_before_target),
    ("TARGET_AFTER_STOP", lambda row: row.target_after_stop == 0),
    ("OBSERVATION_AFTER_STOP", lambda row: row.observation_after_stop == 0),
    ("PUBLICATION_AFTER_STOP", lambda row: row.publication_after_stop == 0),
    ("SITE_ID_MISMATCH", lambda row: row.requested_site == row.observed_site),
    ("COUNTER_MISMATCH", lambda row: row.expected_counter == row.observed_counter),
    ("ALTERNATE_CONSUMER_BYPASS", lambda row: row.canonical_consumer),
)

NEGATIVE: Final = {
    "TYPED_BUDGET_EXHAUSTED_IDENTITY": {"budget_identity": False},
    "TYPED_CANCELLED_IDENTITY": {"cancelled_identity": False},
    "UNEXPECTED_WORK_ERROR_IDENTITY": {"unexpected_identity": False},
    "CHARGE_AFTER_OPERATION": {"charge_before_target": False},
    "TARGET_AFTER_STOP": {"target_after_stop": 1},
    "OBSERVATION_AFTER_STOP": {"observation_after_stop": 1},
    "PUBLICATION_AFTER_STOP": {"publication_after_stop": 1},
    "SITE_ID_MISMATCH": {"observed_site": "other_site"},
    "COUNTER_MISMATCH": {"observed_counter": "graph_edge"},
    "ALTERNATE_CONSUMER_BYPASS": {"canonical_consumer": False},
}


def classify_runtime_observation(row: Observation) -> None:
    for code, predicate in CHECKS[:3]:
        if not predicate(row):
            raise PropertyFailure(code)


def validate_structural_observation(row: Observation) -> None:
    for code, predicate in CHECKS[3:]:
        if not predicate(row):
            raise PropertyFailure(code)


def validate_full_observation(row: Observation) -> None:
    classify_runtime_observation(row)
    validate_structural_observation(row)


def require(condition: bool, code: str) -> None:
    if not condition:
        raise PropertyFailure(code)


def validate_document(report: dict, schema: dict) -> None:
    require(schema.get("$id", "").endswith("causal_projection_properties_v17.schema.json"), "schema:id")
    expected_keys = {"schema", "status", "property_codes", "result_classes", "interfaces", "positive_case", "negative_cases", "result"}
    require(set(report) == expected_keys, "report:fields")
    require(report["schema"] == "nostr_automerge.causal_projection_properties.v17.v1", "report:schema")
    require(report["status"] == "implemented" and report["result"] == "pass", "report:status")
    require(tuple(report["property_codes"]) == PROPERTY_CODES, "report:codes")
    require(set(report["result_classes"]) == set(PROPERTY_CODES), "report:classes")
    require(report["positive_case"] == {"failures": [], "result": "pass"}, "report:positive")
    negative = report["negative_cases"]
    require(len(negative) == len(PROPERTY_CODES), "report:negative-count")
    require([row["requested_code"] for row in negative] == list(PROPERTY_CODES), "report:negative-order")
    require(all(row == {"requested_code": row["requested_code"], "actual_code": row["requested_code"], "result": "detected"} for row in negative), "report:negative-result")


def exercise_classifiers() -> None:
    baseline = Observation()
    validate_full_observation(baseline)
    for expected in PROPERTY_CODES:
        mutated = replace(baseline, **NEGATIVE[expected])
        try:
            validate_full_observation(mutated)
        except PropertyFailure as error:
            require(error.code == expected, f"classifier:{expected}:{error.code}")
        else:
            raise PropertyFailure(f"classifier:{expected}:survived")


def exercise_document_attacks(report: dict, schema: dict) -> None:
    attacks = (
        lambda row: row["property_codes"].reverse(),
        lambda row: row["result_classes"].pop("TYPED_CANCELLED_IDENTITY"),
        lambda row: row["negative_cases"][2].update(actual_code="TARGET_AFTER_STOP"),
        lambda row: row.update(result="fail"),
    )
    for index, attack in enumerate(attacks):
        changed = copy.deepcopy(report)
        attack(changed)
        try:
            validate_document(changed, schema)
        except PropertyFailure:
            continue
        raise PropertyFailure(f"attack:{index}:survived")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--probe", choices=PROPERTY_CODES)
    args = parser.parse_args()
    report = json.loads(REPORT.read_text(encoding="utf-8"))
    schema = json.loads(SCHEMA.read_text(encoding="utf-8"))
    validate_document(report, schema)
    exercise_classifiers()
    exercise_document_attacks(report, schema)
    if args.probe:
        try:
            validate_full_observation(replace(Observation(), **NEGATIVE[args.probe]))
        except PropertyFailure as error:
            print(error.code)
            return 0 if error.code == args.probe else 1
        return 1
    print("PASS: causal projection properties v17 codes=10 positive=1 negative=10 attacks=4")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
