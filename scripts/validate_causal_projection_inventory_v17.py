#!/usr/bin/env python3
"""Derive and validate the provisional v17 Rust causal-projection inventory."""

from __future__ import annotations

import argparse
import copy
import hashlib
import json
import re
import subprocess
import sys
from pathlib import Path
from typing import Any

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parents[1]
REPORT = ROOT / "reports/causal_projection_inventory_v17.json"
SCHEMA = ROOT / "tools/validation/causal_projection_inventory_v17.schema.json"
SOURCE = ROOT / "crates/nostr_automerge/src/graph/actor_state.rs"
SOURCE_PATH = "crates/nostr_automerge/src/graph/actor_state.rs"
SOURCE_CANDIDATE = "789eae3c6e0994f71420f49fe51fe3ab7cb75ca9"
AUTHORITY = "spec/causal_projection_contracts_v17.json"
ROW_FIELDS = [
    "id", "phase", "site_id", "operation", "language", "applicability",
    "source_path", "source_symbol", "owner_mode", "counter",
    "abstract_owner_class", "reachability_artifact", "proof_test",
    "proof_command", "proof_status", "source_candidate", "result",
]
TOP_FIELDS = [
    "schema", "status", "authority", "source_candidate", "source_path",
    "source_production_sha256", "row_contract", "rows", "counts",
    "derivation", "result_identity_sha256", "result",
]
PHASES = [
    ("projection_build_sites", "ProjectionBuildSite", "projection_construction", "build_trusted_epoch_projection_observed", "source_operation", True),
    ("actor_decision_sites", "ActorDecisionSite", "actor_sequence", "actor_sequence_decision_metered_observed", "direct_operation", False),
    ("causal_next_sites", "CausalNextSite", "causal_counter", "causal_next_decision_metered_observed", "direct_operation", False),
    ("frontier_comparison_sites", "FrontierComparisonSite", "frontier_comparison", "empty_frontier_decision_metered_observed", "direct_operation", True),
]


class InventoryError(RuntimeError):
    pass


def require(condition: bool, code: str) -> None:
    if not condition:
        raise InventoryError(code)


def canonical(value: object) -> bytes:
    return json.dumps(value, ensure_ascii=True, separators=(",", ":"), sort_keys=True).encode()


def snake(value: str) -> str:
    return re.sub(r"(?<!^)(?=[A-Z])", "_", value).lower()


def production(source: str) -> str:
    marker = "\n#[cfg(test)]\npub(crate) mod tests {"
    require(source.count(marker) == 1, "SOURCE_TEST_BOUNDARY")
    return source.split(marker, 1)[0] + "\n"


def committed_source() -> str:
    completed = subprocess.run(
        ["git", "show", f"{SOURCE_CANDIDATE}:{SOURCE_PATH}"],
        cwd=ROOT,
        capture_output=True,
        text=True,
        check=False,
    )
    require(completed.returncode == 0, "SOURCE_CANDIDATE_MISSING")
    return completed.stdout


def registry(source: str, macro: str, with_counter: bool) -> tuple[list[tuple[str, str, str]], tuple[int, int]]:
    match = re.search(rf"(?m)^{macro}! \{{\n(?P<body>.*?)^\}}$", source, re.DOTALL)
    require(match is not None, f"REGISTRY_MISSING:{macro}")
    pattern = (
        r"^\s*(\w+) => \((\w+), (GraphNode|GraphEdge)\),$"
        if with_counter else r"^\s*(\w+) => (\w+),$"
    )
    rows: list[tuple[str, str, str]] = []
    for line in match.group("body").splitlines():
        if not line.strip():
            continue
        item = re.match(pattern, line)
        require(item is not None, f"REGISTRY_SHAPE:{macro}")
        site, operation = item.group(1), item.group(2)
        counter = item.group(3) if with_counter else "GraphNode"
        rows.append((site, operation, counter))
    require(rows and len(rows) == len({row[0] for row in rows}), f"REGISTRY_DUPLICATE:{macro}")
    return rows, match.span()


def exact_test(phase: str, site: str) -> str:
    test_phase = "frontier" if phase == "frontier_comparison" else phase
    return f"graph::actor_state::tests::causal_projection_v17_site_{test_phase}_{snake(site)}"


def derive_rows(source: str) -> list[dict[str, Any]]:
    code = source
    rows: list[dict[str, Any]] = []
    for macro, enum, phase, symbol, owner, with_counter in PHASES:
        entries, span = registry(code, macro, with_counter)
        reachable_code = code[:span[0]] + code[span[1]:]
        for site, operation, counter in entries:
            occurrences = len(re.findall(rf"\b{enum}::{site}\b", reachable_code))
            require(occurrences == 1, f"SITE_REACHABILITY:{enum}:{site}:{occurrences}")
            test = exact_test(phase, site)
            rows.append({
                "id": f"rust.{phase}.{snake(site)}",
                "phase": phase,
                "site_id": site,
                "operation": operation,
                "language": "rust",
                "applicability": "required",
                "source_path": SOURCE_PATH,
                "source_symbol": symbol,
                "owner_mode": "item_metered",
                "counter": snake(counter),
                "abstract_owner_class": owner,
                "reachability_artifact": hashlib.sha256(f"{SOURCE_CANDIDATE}:{enum}:{site}:{occurrences}".encode()).hexdigest(),
                "proof_test": test,
                "proof_command": f"cargo test -p nostr_automerge --lib {test} --locked -- --exact --nocapture",
                "proof_status": "pending_actual_execution",
                "source_candidate": SOURCE_CANDIDATE,
                "result": "pass",
            })
    require(len(rows) == len({row["id"] for row in rows}) == len({row["site_id"] for row in rows}), "SITE_ID_DUPLICATE")
    return rows


def expected_report(source: str) -> dict[str, Any]:
    source = production(source)
    rows = derive_rows(source)
    phases = {phase: sum(row["phase"] == phase for row in rows) for _, _, phase, *_ in PHASES}
    report: dict[str, Any] = {
        "schema": "nostr_automerge.causal_projection_inventory.v17.v1",
        "status": "provisional",
        "authority": AUTHORITY,
        "source_candidate": SOURCE_CANDIDATE,
        "source_path": SOURCE_PATH,
        "source_production_sha256": hashlib.sha256(source.encode()).hexdigest(),
        "row_contract": ROW_FIELDS,
        "rows": rows,
        "counts": {"rows": len(rows), "phases": phases},
        "derivation": {
            "method": "committed_reachable_descriptor_registry",
            "planned_counts": False,
            "proof_evidence_terminal": False,
        },
        "result_identity_sha256": "",
        "result": "pass",
    }
    identity = {key: value for key, value in report.items() if key != "result_identity_sha256"}
    report["result_identity_sha256"] = hashlib.sha256(canonical(identity)).hexdigest()
    return report


def validate(report: object, schema: object, candidate_source: str, current_source: str) -> None:
    expected = expected_report(candidate_source)
    require(type(report) is dict and list(report) == TOP_FIELDS, "REPORT_SHAPE")
    require(report == expected, "REPORT_DERIVATION_MISMATCH")
    require(production(current_source) == production(candidate_source), "SOURCE_CANDIDATE_DRIFT")
    require(report["status"] == "provisional", "STATUS_NOT_PROVISIONAL")
    require(report["counts"] == {"rows": 68, "phases": {"projection_construction": 50, "actor_sequence": 4, "causal_counter": 3, "frontier_comparison": 11}}, "COUNT_MISMATCH")
    require(all(list(row) == ROW_FIELDS for row in report["rows"]), "ROW_SHAPE")
    require(type(schema) is dict and schema.get("additionalProperties") is False, "SCHEMA_OPEN")
    require(schema.get("required") == TOP_FIELDS, "SCHEMA_REQUIRED_MISMATCH")
    require(schema["properties"]["rows"].get("minItems") == schema["properties"]["rows"].get("maxItems") == 68, "SCHEMA_ROW_COUNT")


def self_test(report: dict[str, Any], schema: dict[str, Any], candidate_source: str, current_source: str) -> int:
    attacks = [
        ("missing", "report", lambda value: value["rows"].pop()),
        ("duplicate", "report", lambda value: value["rows"].__setitem__(1, copy.deepcopy(value["rows"][0]))),
        ("dead", "report", lambda value: value["rows"][0].update(reachability_artifact="0" * 64)),
        ("mismatch", "report", lambda value: value["rows"][0].update(counter="graph_edge")),
        ("order", "report", lambda value: value["rows"].reverse()),
        ("schema", "schema", lambda value: value.update(additionalProperties=True)),
        ("source_missing", "source", lambda value: value.replace("ProjectionBuildSite::MemberCountRead", "ProjectionBuildSite::MemberCountGone", 1)),
        ("shadowed", "source", lambda value: value.replace("ProjectionBuildSite::MemberCountRead", "/* ProjectionBuildSite::MemberCountRead */ ProjectionBuildSite::MemberCountRead", 1)),
        ("coordinated", "coordinated", lambda value: value["rows"][0].update(site_id="MemberCountGone")),
    ]
    caught = 0
    for label, target, mutate in attacks:
        changed_report, changed_schema, changed_source = copy.deepcopy(report), copy.deepcopy(schema), current_source
        if target == "report":
            mutate(changed_report)
        elif target == "schema":
            mutate(changed_schema)
        elif target == "source":
            changed_source = mutate(changed_source)
        else:
            mutate(changed_report)
            changed_source = changed_source.replace("ProjectionBuildSite::MemberCountRead", "ProjectionBuildSite::MemberCountGone", 1)
        try:
            validate(changed_report, changed_schema, candidate_source, changed_source)
        except InventoryError:
            caught += 1
            continue
        raise InventoryError(f"MUTATION_SURVIVED:{label}")
    return caught


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--write-report", action="store_true")
    args = parser.parse_args()
    candidate_source = committed_source()
    expected = expected_report(candidate_source)
    if args.write_report:
        REPORT.write_text(json.dumps(expected, ensure_ascii=True, indent=2) + "\n")
    report = json.loads(REPORT.read_text())
    schema = json.loads(SCHEMA.read_text())
    validate(report, schema, candidate_source, SOURCE.read_text())
    attacks = self_test(report, schema, candidate_source, SOURCE.read_text())
    print(f"PASS: causal projection inventory v17 status=provisional rows={report['counts']['rows']} attacks={attacks}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
