#!/usr/bin/env python3
"""Validate the source-derived closed v19 helper mutation matrix."""

from __future__ import annotations

import argparse
import copy
import json
import subprocess
import sys
from pathlib import Path
from typing import Any

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parents[1]
SOURCE_PATH = "crates/nostr_automerge/src/graph/actor_state.rs"
CONTRACT = ROOT / "spec/causal_projection_contracts_v19.json"
INVENTORY = ROOT / "reports/causal_projection_inventory_v19.json"
REPORT = ROOT / "reports/causal_projection_mutation_matrix_v19.json"
SCHEMA = ROOT / "tools/validation/causal_projection_mutation_matrix_v19.schema.json"
HELPERS = [
    ("projection_construction", "perform_projection_build_operation", "MemberCountRead"),
    ("actor_sequence", "perform_actor_decision_operation", "ActorStateRead"),
    ("causal_counter", "perform_causal_next_operation", "StoredCounterRead"),
    ("frontier_comparison", "metered_frontier_operation", "CandidateKindComparison"),
]
PROPERTIES = {
    "target_before_charge": "CHARGE_AFTER_OPERATION",
    "completion_before_target": "OPERATION_OBSERVATION_BEFORE_TARGET",
    "target_after_failed_charge": "TARGET_AFTER_STOP",
    "completion_after_failed_charge": "OBSERVATION_AFTER_STOP",
    "double_target": "TARGET_EXECUTION_COUNT_MISMATCH",
    "site_swap": "SITE_ID_MISMATCH",
    "counter_mismatch": "COUNTER_MISMATCH",
}
TOP_FIELDS = ["schema", "status", "authority", "source_candidate", "row_contract", "rows", "counts", "result"]
ROW_FIELDS = ["mutation_id", "phase", "helper", "representative_site", "kind", "expected_property_code"]


class MatrixError(RuntimeError):
    pass


def require(condition: bool, code: str) -> None:
    if not condition:
        raise MatrixError(code)


def committed_source(candidate: str) -> str:
    completed = subprocess.run(
        ["git", "show", f"{candidate}:{SOURCE_PATH}"], cwd=ROOT,
        capture_output=True, text=True, check=False,
    )
    require(completed.returncode == 0, "SOURCE_CANDIDATE")
    return completed.stdout


def expected_report(contract: dict[str, Any], inventory: dict[str, Any]) -> dict[str, Any]:
    source = committed_source(inventory["source_candidate"])
    require(contract["helpers"] == [helper for _, helper, _ in HELPERS], "HELPER_CONTRACT")
    require(contract["helper_mutations"] == list(PROPERTIES), "MUTATION_CONTRACT")
    for _, helper, _ in HELPERS:
        require(source.count(f"fn {helper}") == 1, "HELPER_SOURCE:" + helper)
    inventory_sites = {row["site_id"] for row in inventory["rows"]}
    rows = []
    for phase, helper, site in HELPERS:
        require(site in inventory_sites, "REPRESENTATIVE_SITE:" + site)
        for kind, property_code in PROPERTIES.items():
            rows.append({
                "mutation_id": f"helper.{phase}.{kind}",
                "phase": phase,
                "helper": helper,
                "representative_site": site,
                "kind": kind,
                "expected_property_code": property_code,
            })
    return {
        "schema": "nostr_automerge.causal_projection_mutation_matrix.v19.v1",
        "status": "source_derived_closed",
        "authority": "spec/causal_projection_contracts_v19.json",
        "source_candidate": inventory["source_candidate"],
        "row_contract": ROW_FIELDS,
        "rows": rows,
        "counts": {"helpers": len(HELPERS), "kinds_per_helper": len(PROPERTIES), "cells": len(rows)},
        "result": "pass",
    }


def validate(report: Any, schema: Any, contract: dict[str, Any], inventory: dict[str, Any]) -> None:
    require(type(report) is dict and list(report) == TOP_FIELDS, "REPORT_SHAPE")
    require(report == expected_report(contract, inventory), "REPORT_DERIVATION")
    require(len(report["rows"]) == len({row["mutation_id"] for row in report["rows"]}) == 28, "CELL_IDENTITY")
    require(all(list(row) == ROW_FIELDS for row in report["rows"]), "ROW_SHAPE")
    require(type(schema) is dict and schema.get("additionalProperties") is False, "SCHEMA_CLOSED")
    require(schema.get("required") == TOP_FIELDS, "SCHEMA_TOP")
    require(schema["properties"]["rows"]["items"].get("required") == ROW_FIELDS, "SCHEMA_ROWS")


def self_test(report: dict[str, Any], schema: dict[str, Any], contract: dict[str, Any], inventory: dict[str, Any]) -> int:
    attacks = [
        lambda value: value["rows"].pop(),
        lambda value: value["rows"].__setitem__(1, copy.deepcopy(value["rows"][0])),
        lambda value: value["rows"][0].update(helper="nearby_helper"),
        lambda value: value["rows"][0].update(kind="provenance_substitute"),
    ]
    caught = 0
    for mutate in attacks:
        changed = copy.deepcopy(report)
        mutate(changed)
        try:
            validate(changed, schema, contract, inventory)
        except MatrixError:
            caught += 1
            continue
        raise MatrixError("MATRIX_ATTACK_SURVIVED")
    return caught


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--write-report", action="store_true")
    args = parser.parse_args()
    contract = json.loads(CONTRACT.read_text())
    inventory = json.loads(INVENTORY.read_text())
    expected = expected_report(contract, inventory)
    if args.write_report:
        REPORT.write_text(json.dumps(expected, ensure_ascii=True, indent=2) + "\n")
    report = json.loads(REPORT.read_text())
    schema = json.loads(SCHEMA.read_text())
    validate(report, schema, contract, inventory)
    print(f"PASS v19 helper mutation matrix: cells={len(report['rows'])} attacks={self_test(report, schema, contract, inventory)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
