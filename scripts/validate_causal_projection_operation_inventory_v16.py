#!/usr/bin/env python3
"""Derive and validate the provisional Rust causal-projection site inventory."""

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
REPORT = ROOT / "reports/causal_projection_operation_inventory_v16.json"
SCHEMA = ROOT / "tools/validation/causal_projection_operation_inventory_v16.schema.json"
SOURCE = ROOT / "crates/nostr_automerge/src/graph/actor_state.rs"
SOURCE_PATH = "crates/nostr_automerge/src/graph/actor_state.rs"
SOURCE_CANDIDATE = "1d2dbb5e2358b430516ec876c0bb74e3ec1af68a"
AUTHORITY = "spec/causal_projection_contracts_v16.json"
TOP_FIELDS = [
    "schema", "status", "authority", "source_candidate", "source_path",
    "source_production_sha256", "row_contract", "rows", "counts",
    "counter_correction", "result_identity_sha256", "result",
]
ROW_FIELDS = [
    "id", "abstract_family", "phase", "language", "applicability",
    "source_path", "source_symbol", "source_site", "owner_mode", "counter",
    "abstract_owner_class", "reachability", "proof", "test", "command",
    "candidate", "artifact_sha256", "mutation",
]
PHASES = [
    ("projection_construction", "build_trusted_epoch_projection_observed", "perform_projection_build_operation", "ProjectionBuildOperation"),
    ("actor_sequence", "actor_sequence_decision_metered_observed", "direct_charge_observe", "ActorDecisionOperation"),
    ("causal_counter_consumer", "causal_next_decision_metered_observed", "direct_charge_observe", "CausalNextOperation"),
    ("frontier_comparison", "empty_frontier_decision_metered_observed", "metered_frontier_operation", "FrontierComparisonOperation"),
]

sys.path.insert(0, str(ROOT / "scripts"))
from validate_causal_projection_source_v13 import function_body  # noqa: E402
from validate_report_contract_v9 import ReportSuiteError, rust_code_view  # noqa: E402


class InventoryError(RuntimeError):
    pass


def require(condition: bool, label: str) -> None:
    if not condition:
        raise InventoryError(label)


def canonical(value: object) -> bytes:
    return json.dumps(value, ensure_ascii=True, separators=(",", ":"), sort_keys=True).encode()


def snake(value: str) -> str:
    return re.sub(r"(?<!^)(?=[A-Z])", "_", value).lower()


def production(source: str) -> str:
    marker = "\n#[cfg(test)]\npub(crate) mod tests {"
    require(source.count(marker) == 1, "source:test_boundary")
    return source.split(marker, 1)[0] + "\n"


def committed_source() -> str:
    completed = subprocess.run(
        ["git", "show", f"{SOURCE_CANDIDATE}:{SOURCE_PATH}"],
        cwd=ROOT,
        capture_output=True,
        text=True,
        check=False,
    )
    require(completed.returncode == 0, "source:candidate")
    return completed.stdout


def proof_fields(row_id: str, counter: str, site: str) -> tuple[str, str, str, str, str]:
    suffix = row_id.removeprefix("rust.").replace(".", "_")
    test = f"graph::actor_state::tests::causal_projection_v16_site_{suffix}"
    command = f"cargo test -p nostr_automerge --lib {test} --locked -- --exact"
    proof = f"planned:{test}"
    mutation = f"planned:mutation.{row_id}"
    artifact = hashlib.sha256(canonical({"command": command, "counter": counter, "site": site, "test": test})).hexdigest()
    return proof, test, command, artifact, mutation


def make_row(phase: str, symbol: str, wrapper: str, family: str, counter: str, occurrence: int) -> dict[str, Any]:
    family_id = snake(family)
    site = f"{wrapper}:{family}#{occurrence}"
    row_id = f"rust.{phase}.{family_id}.{occurrence:02d}"
    proof, test, command, artifact, mutation = proof_fields(row_id, counter, site)
    return {
        "id": row_id,
        "abstract_family": f"{phase}.{family_id}",
        "phase": phase,
        "language": "rust",
        "applicability": "required",
        "source_path": SOURCE_PATH,
        "source_symbol": symbol,
        "source_site": site,
        "owner_mode": "item_metered",
        "counter": counter,
        "abstract_owner_class": family_id,
        "reachability": 1,
        "proof": proof,
        "test": test,
        "command": command,
        "candidate": SOURCE_CANDIDATE,
        "artifact_sha256": artifact,
        "mutation": mutation,
    }


def wrapper_sites(source: str, symbol: str, wrapper: str, enum: str) -> list[tuple[str, str]]:
    body = function_body(source, symbol)
    try:
        code = rust_code_view(body)
    except ReportSuiteError as error:
        raise InventoryError("source:lexical") from error
    pattern = re.compile(
        rf"\b{re.escape(wrapper)}\s*\(\s*&mut\s+charge\s*,\s*&mut\s+(?:built|observed)\s*,\s*"
        rf"WorkCounter::(?P<counter>GraphNode|GraphEdge)\s*,\s*{re.escape(enum)}::(?P<family>[A-Za-z0-9_]+)"
        if wrapper == "metered_frontier_operation"
        else rf"\b{re.escape(wrapper)}\s*\(\s*WorkCounter::(?P<counter>GraphNode|GraphEdge)\s*,\s*"
             rf"{re.escape(enum)}::(?P<family>[A-Za-z0-9_]+)",
        re.MULTILINE,
    )
    return [(match.group("family"), snake(match.group("counter"))) for match in pattern.finditer(code)]


def direct_sites(source: str, symbol: str, enum: str) -> list[tuple[str, str]]:
    body = function_body(source, symbol)
    try:
        code = rust_code_view(body)
    except ReportSuiteError as error:
        raise InventoryError("source:lexical") from error
    observations = list(re.finditer(rf"\bobserved\s*\(\s*{re.escape(enum)}::(?P<family>[A-Za-z0-9_]+)\s*\)", code))
    rows: list[tuple[str, str]] = []
    start = 0
    for observation in observations:
        segment = code[start:observation.start()]
        charges = re.findall(r"\bcharge\s*\(\s*WorkCounter::(GraphNode|GraphEdge)\s*\)", segment)
        require(len(charges) == 1, f"source:direct_charge:{symbol}:{observation.group('family')}")
        rows.append((observation.group("family"), snake(charges[0])))
        start = observation.end()
    return rows


def derive_rows(source: str) -> list[dict[str, Any]]:
    rows: list[dict[str, Any]] = []
    for phase, symbol, wrapper, enum in PHASES:
        sites = direct_sites(source, symbol, enum) if wrapper == "direct_charge_observe" else wrapper_sites(source, symbol, wrapper, enum)
        occurrences: dict[str, int] = {}
        for family, counter in sites:
            occurrences[family] = occurrences.get(family, 0) + 1
            rows.append(make_row(phase, symbol, wrapper, family, counter, occurrences[family]))
    return rows


def expected_report(candidate_source: str) -> dict[str, Any]:
    source = production(candidate_source)
    rows = derive_rows(source)
    families = list(dict.fromkeys(row["abstract_family"] for row in rows))
    phases = {phase: sum(row["phase"] == phase for row in rows) for phase, *_ in PHASES}
    value: dict[str, Any] = {
        "schema": "nostr_automerge.causal_projection_operation_inventory.v16.v1",
        "status": "provisional_complete",
        "authority": AUTHORITY,
        "source_candidate": SOURCE_CANDIDATE,
        "source_path": SOURCE_PATH,
        "source_production_sha256": hashlib.sha256(source.encode()).hexdigest(),
        "row_contract": ROW_FIELDS,
        "rows": rows,
        "counts": {"rows": len(rows), "families": len(families), "phases": phases},
        "counter_correction": {
            "family": "projection_construction.dependency_count_read",
            "rust_counter": "graph_node",
            "historical_v15_evidence": "graph_edge",
            "status": "corrected_from_reachable_source",
        },
        "result_identity_sha256": "",
        "result": "pass",
    }
    identity = {key: item for key, item in value.items() if key != "result_identity_sha256"}
    value["result_identity_sha256"] = hashlib.sha256(canonical(identity)).hexdigest()
    return value


def validate(report: object, schema: object, candidate_source: str, current_source: str) -> None:
    expected = expected_report(candidate_source)
    require(type(report) is dict and list(report) == TOP_FIELDS and report == expected, "report:value")
    authority = json.loads((ROOT / AUTHORITY).read_text())
    require(report["row_contract"] == authority["operation_discovery"]["row_fields"], "authority:row_contract")
    require(production(current_source) == production(candidate_source), "source:current_production")
    rows = report["rows"]
    require(len(rows) == len({row["id"] for row in rows}) == len({row["source_site"] for row in rows}) == 68, "rows:unique")
    require(report["counts"] == {"rows": 68, "families": 38, "phases": {"projection_construction": 50, "actor_sequence": 4, "causal_counter_consumer": 3, "frontier_comparison": 11}}, "counts:value")
    dependency = [row for row in rows if row["abstract_family"] == "projection_construction.dependency_count_read"]
    require(
        len(dependency) == 1
        and dependency[0]["counter"] == "graph_node"
        and dependency[0]["abstract_owner_class"] == "dependency_count_read"
        and authority["counter_binding"] == {
            "abstract_operation": "DependencyCountRead",
            "abstract_owner_class": "dependency_count_read",
            "rust": "GraphNode",
            "typescript": "source_derived_after_private_refactor",
            "cross_language_rule": "shared_abstract_owner_language_specific_concrete_counter",
            "drift_failures": ["source_only", "evidence_only", "coordinated"],
        },
        "counter:dependency_count",
    )
    require(all(type(row) is dict and list(row) == ROW_FIELDS for row in rows), "rows:shape")
    require(type(schema) is dict and schema.get("additionalProperties") is False and schema.get("required") == TOP_FIELDS, "schema:closed")
    require(schema["properties"]["rows"].get("minItems") == schema["properties"]["rows"].get("maxItems") == 68, "schema:rows")


def self_test(report: dict, schema: dict, candidate_source: str, current_source: str) -> int:
    cases = [
        ("missing", "report", lambda value: value["rows"].pop()),
        ("extra", "report", lambda value: value["rows"].append(copy.deepcopy(value["rows"][-1]))),
        ("duplicate", "report", lambda value: value["rows"].__setitem__(1, copy.deepcopy(value["rows"][0]))),
        ("order", "report", lambda value: value["rows"].reverse()),
        ("counter", "report", lambda value: next(row for row in value["rows"] if row["abstract_family"].endswith("dependency_count_read")).update(counter="graph_edge")),
        ("site", "report", lambda value: value["rows"][0].update(source_site="nearby")),
        ("proof", "report", lambda value: value["rows"][0].update(proof="umbrella")),
        ("candidate", "report", lambda value: value["rows"][0].update(candidate="0" * 40)),
        ("identity", "report", lambda value: value.update(result_identity_sha256="0" * 64)),
        ("schema", "schema", lambda value: value.update(additionalProperties=True)),
        ("source_only", "source", lambda value: value.replace("ProjectionBuildOperation::DependencyCountRead", "ProjectionBuildOperation::StateLookup", 1)),
        ("coordinated_counter", "coordinated", lambda value: next(row for row in value["rows"] if row["abstract_family"].endswith("dependency_count_read")).update(counter="graph_edge")),
        ("lexical_shadow", "source", lambda value: value.replace("perform_projection_build_operation(\n        WorkCounter::GraphNode,", "/* perform_projection_build_operation( WorkCounter::GraphEdge, ProjectionBuildOperation::DependencyCountRead */\n    perform_projection_build_operation(\n        WorkCounter::GraphNode,", 1)),
    ]
    caught = 0
    for label, target, mutation in cases:
        changed_report = copy.deepcopy(report)
        changed_schema = copy.deepcopy(schema)
        changed_source = current_source
        if target == "report":
            mutation(changed_report)
        elif target == "schema":
            mutation(changed_schema)
        elif target == "source":
            changed_source = mutation(changed_source)
        else:
            mutation(changed_report)
            changed_source = changed_source.replace(
                "WorkCounter::GraphNode,\n            ProjectionBuildOperation::DependencyCountRead",
                "WorkCounter::GraphEdge,\n            ProjectionBuildOperation::DependencyCountRead",
                1,
            )
        try:
            validate(changed_report, changed_schema, candidate_source, changed_source)
        except InventoryError:
            caught += 1
            continue
        raise InventoryError("mutation_survived:" + label)
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
    mutations = self_test(report, schema, candidate_source, SOURCE.read_text())
    print(
        "PASS: causal projection operation inventory v16 "
        f"rows={report['counts']['rows']} families={report['counts']['families']} "
        f"dependency_count={report['counter_correction']['rust_counter']} mutations={mutations}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
