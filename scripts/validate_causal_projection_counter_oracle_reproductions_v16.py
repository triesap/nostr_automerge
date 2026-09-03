#!/usr/bin/env python3
"""Validate the independent v16 counter and validation-oracle reproductions."""

from __future__ import annotations

import copy
import hashlib
import importlib.util
import json
import subprocess
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[1]
REPORT_PATH = ROOT / "reports/causal_projection_counter_oracle_reproductions_v16.json"
SCHEMA_PATH = ROOT / "tools/validation/causal_projection_counter_oracle_reproductions_v16.schema.json"
ACTOR_REPORT = ROOT / "reports/causal_projection_actor_reproductions_v16.json"
PROOF_REPORT = ROOT / "reports/causal_projection_proof_catalog_v15.json"
OWNERSHIP_VALIDATOR = ROOT / "scripts/validate_causal_projection_source_ownership_v15.py"
SOURCE_CANDIDATE = "dc5c93e94a1ee79cd9f10c5ae1c8cc74ebc331a9"
ACTOR_REPORT_SHA256 = "40b898367a3bdf376bca9f4863680b5bda0e1d409c1f5f895d80e6aacb165a45"
CASE_FIELDS = ["id","finding","abstract_operation","runtime_counter","evidence_counter","property","expected"]


class CounterOracleError(RuntimeError):
    pass


def require(condition: bool, label: str) -> None:
    if not condition:
        raise CounterOracleError(label)


def exact(value: Any, fields: list[str], label: str) -> dict[str, Any]:
    require(type(value) is dict and list(value) == fields, f"{label}:shape")
    return value


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def load(path: Path) -> Any:
    def closed(pairs: list[tuple[str, Any]]) -> dict[str, Any]:
        keys = [key for key, _ in pairs]
        require(len(keys) == len(set(keys)), f"duplicate:{path.name}")
        return dict(pairs)

    return json.loads(path.read_text(), object_pairs_hook=closed)


def committed_source(candidate: str) -> str:
    completed = subprocess.run(
        ["git", "show", f"{candidate}:crates/nostr_automerge/src/graph/actor_state.rs"],
        cwd=ROOT,
        capture_output=True,
        text=True,
        check=False,
    )
    require(completed.returncode == 0, "source:candidate")
    return completed.stdout


def load_ownership_module() -> Any:
    spec = importlib.util.spec_from_file_location("v15_source_ownership", OWNERSHIP_VALIDATOR)
    require(spec is not None and spec.loader is not None, "oracle:module_spec")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def reproduce_counter_mismatch(source: str) -> None:
    production = source.split("#[cfg(test)]\npub(crate) mod tests", 1)[0]
    runtime_binding = "WorkCounter::GraphNode,\n            ProjectionBuildOperation::DependencyCountRead"
    require(production.count(runtime_binding) == 1, "counter:runtime_graph_node")
    proof = load(PROOF_REPORT)
    rows = [row for row in proof["rows"] if row["id"] == "construction.dependency_count_read"]
    require(len(rows) == 1 and rows[0]["counter"] == "graph_edge", "counter:evidence_graph_edge")


def reproduce_identity_only_failure() -> None:
    module = load_ownership_module()
    source = committed_source(module.SOURCE_CANDIDATE)
    catalog = load(module.CATALOG)
    report = load(module.REPORT)
    schema = load(module.SCHEMA)
    neutral = "// neutral v16 identity-only reproduction\n" + source
    module.structural_report(neutral, catalog)
    try:
        module.validate(report, schema, neutral, catalog)
    except module.OwnershipError as error:
        require(str(error) == "source:sha256", "oracle:identity_code")
    else:
        raise CounterOracleError("oracle:identity_missing")


def validate(report: Any, schema: Any, *, exercise: bool = True) -> None:
    row = exact(
        report,
        ["schema","status","source_candidate","actor_reproduction_sha256","cases","closure_evidence","result"],
        "report",
    )
    require(
        row["schema"] == "nostr_automerge.causal_projection_counter_oracle_reproductions.v16.v1"
        and row["status"] == "expected_defects_reproduced"
        and row["source_candidate"] == SOURCE_CANDIDATE
        and row["actor_reproduction_sha256"] == ACTOR_REPORT_SHA256
        and row["closure_evidence"] is False
        and row["result"] == "pass",
        "report:values",
    )
    require(sha256(ACTOR_REPORT) == ACTOR_REPORT_SHA256, "report:actor_dependency")
    cases = row["cases"]
    require(type(cases) is list and [case["id"] for case in cases] == ["dependency_count_counter","structural_identity_oracle"], "cases:order")
    require([case["finding"] for case in cases] == ["FINDING_117","FINDING_118"], "cases:findings")
    require([case["property"] for case in cases] == ["COUNTER_MISMATCH","IDENTITY_ONLY_FAILURE"], "cases:properties")
    require(
        cases[0]["runtime_counter"] == "GraphNode"
        and cases[0]["evidence_counter"] == "graph_edge"
        and cases[0]["expected"] == "mismatch"
        and cases[1]["runtime_counter"] is None
        and cases[1]["evidence_counter"] is None
        and cases[1]["expected"] == "structural_pass_identity_fail",
        "cases:values",
    )
    for index, case in enumerate(cases):
        exact(case, CASE_FIELDS, f"case:{index}")
    if exercise:
        reproduce_counter_mismatch(committed_source(SOURCE_CANDIDATE))
        reproduce_identity_only_failure()

    schema_row = exact(schema, ["$schema","type","additionalProperties","required","properties"], "schema")
    require(schema_row["type"] == "object" and schema_row["additionalProperties"] is False, "schema:closed")
    require(schema_row["required"] == list(row), "schema:required")
    case_schema = schema_row["properties"]["cases"]["items"]
    require(case_schema["additionalProperties"] is False and case_schema["required"] == CASE_FIELDS, "schema:case")


def self_test(report: Any, schema: Any) -> int:
    mutations = [
        ("missing", "report", lambda value: value["cases"].pop()),
        ("extra", "report", lambda value: value["cases"].append(copy.deepcopy(value["cases"][-1]))),
        ("duplicate", "report", lambda value: value["cases"].__setitem__(1, copy.deepcopy(value["cases"][0]))),
        ("order", "report", lambda value: value["cases"].reverse()),
        ("candidate", "report", lambda value: value.update(source_candidate="0" * 40)),
        ("dependency", "report", lambda value: value.update(actor_reproduction_sha256="0" * 64)),
        ("counter", "report", lambda value: value["cases"][0].update(runtime_counter="GraphEdge")),
        ("property", "report", lambda value: value["cases"][1].update(property="SOURCE_HASH")),
        ("closure", "report", lambda value: value.update(closure_evidence=True)),
        ("schema", "schema", lambda value: value.update(additionalProperties=True)),
    ]
    caught = 0
    for label, target, mutation in mutations:
        values = {"report": copy.deepcopy(report), "schema": copy.deepcopy(schema)}
        mutation(values[target])
        try:
            validate(values["report"], values["schema"], exercise=False)
        except CounterOracleError:
            caught += 1
            continue
        raise CounterOracleError("mutation_survived:" + label)
    return caught


def main() -> int:
    report = load(REPORT_PATH)
    schema = load(SCHEMA_PATH)
    validate(report, schema)
    mutations = self_test(report, schema)
    print(
        "PASS: causal projection counter-oracle reproductions v16 "
        f"cases=2 expected_defects=2 mutations={mutations}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
