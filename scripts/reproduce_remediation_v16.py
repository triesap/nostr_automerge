#!/usr/bin/env python3
"""Validate and execute the exact v16 expected-defect reproductions."""

from __future__ import annotations

import argparse
import copy
import hashlib
import json
import subprocess
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[1]
REPORT_PATH = ROOT / "reports/causal_projection_actor_reproductions_v16.json"
SCHEMA_PATH = ROOT / "tools/validation/causal_projection_actor_reproductions_v16.schema.json"
SOURCE_PATH = ROOT / "crates/nostr_automerge/src/graph/actor_state.rs"
SOURCE_CANDIDATE = "16a8ca3e3d4fe7f4ead60ba5c32ebd018c703856"
PRODUCTION_SHA256 = "3dc491772a2e052de0782b169445307633261dde6814b7c6a9cc823c1da4bb7e"
CASE_IDS = [
    "outer_actor_classification",
    "early_causal_work",
    "duplicate_start_comparison",
    "semantic_charge_boundary",
]
PROPERTIES = [
    "UNWRAPPED_ACTOR_SEQUENCE_DECISION",
    "CAUSAL_STAGE_BEFORE_ACTOR_SUCCESS",
    "DUPLICATE_CAUSAL_START_COMPARISON",
    "CHARGE_AFTER_OPERATION",
]


class ReproductionError(RuntimeError):
    pass


def require(condition: bool, label: str) -> None:
    if not condition:
        raise ReproductionError(label)


def exact(value: Any, fields: list[str], label: str) -> dict[str, Any]:
    require(type(value) is dict and list(value) == fields, f"{label}:shape")
    return value


def load(path: Path) -> Any:
    def closed(pairs: list[tuple[str, Any]]) -> dict[str, Any]:
        keys = [key for key, _ in pairs]
        require(len(keys) == len(set(keys)), f"duplicate:{path.name}")
        return dict(pairs)

    return json.loads(path.read_text(), object_pairs_hook=closed)


def production_prefix(contents: bytes) -> bytes:
    marker = b'#[cfg(test)]\npub(crate) mod tests'
    require(contents.count(marker) == 1, "source:test_marker")
    return contents.split(marker, 1)[0]


def validate(report: Any, schema: Any) -> None:
    row = exact(
        report,
        [
            "schema",
            "status",
            "source_candidate",
            "production_source_sha256",
            "finding",
            "requirements",
            "cases",
            "closure_evidence",
            "result",
        ],
        "report",
    )
    require(
        row["schema"] == "nostr_automerge.causal_projection_actor_reproductions.v16.v1"
        and row["status"] == "expected_defects_reproduced"
        and row["source_candidate"] == SOURCE_CANDIDATE
        and row["production_source_sha256"] == PRODUCTION_SHA256
        and row["finding"] == "FINDING_116"
        and row["requirements"]
        == ["NCRDT-RESOURCE-016", "NCRDT-RESOURCE-017", "NCRDT-RESOURCE-018"]
        and row["closure_evidence"] is False
        and row["result"] == "pass",
        "report:values",
    )
    cases = row["cases"]
    require(type(cases) is list and [case["id"] for case in cases] == CASE_IDS, "cases:order")
    require([case["property"] for case in cases] == PROPERTIES, "cases:properties")
    source = SOURCE_PATH.read_text()
    for index, case in enumerate(cases):
        exact(case, ["id", "test", "property", "expected"], f"case:{index}")
        require(case["expected"] == "fail", f"case:{index}:expected")
        short_name = case["test"].rsplit("::", 1)[-1]
        declaration = f'#[ignore = "expected FINDING_116 defect until step_1473"]\n    fn {short_name}()'
        require(source.count(declaration) == 1, f"case:{index}:ignored_test")

    current_prefix = production_prefix(SOURCE_PATH.read_bytes())
    require(hashlib.sha256(current_prefix).hexdigest() == PRODUCTION_SHA256, "source:production_hash")
    candidate = subprocess.run(
        ["git", "show", f"{SOURCE_CANDIDATE}:crates/nostr_automerge/src/graph/actor_state.rs"],
        cwd=ROOT,
        capture_output=True,
        check=False,
    )
    require(candidate.returncode == 0, "source:candidate")
    require(production_prefix(candidate.stdout) == current_prefix, "source:production_unchanged")

    schema_row = exact(schema, ["$schema", "type", "additionalProperties", "required", "properties"], "schema")
    require(schema_row["type"] == "object" and schema_row["additionalProperties"] is False, "schema:closed")
    require(schema_row["required"] == list(row), "schema:required")
    case_schema = schema_row["properties"]["cases"]["items"]
    require(case_schema["additionalProperties"] is False, "schema:case_closed")


def exact_failure(test: str, result: subprocess.CompletedProcess[str]) -> bool:
    output = result.stdout + result.stderr
    return (
        result.returncode != 0
        and f"test {test} ... FAILED" in output
        and "0 passed; 1 failed; 0 ignored" in output
    )


def run_cases(report: dict[str, Any]) -> None:
    for case in report["cases"]:
        command = [
            "cargo",
            "test",
            "-p",
            "nostr_automerge",
            "--lib",
            case["test"],
            "--locked",
            "--",
            "--exact",
            "--ignored",
        ]
        result = subprocess.run(command, cwd=ROOT, capture_output=True, text=True, check=False)
        require(exact_failure(case["test"], result), "run:not_exact:" + case["id"])


def self_test(report: Any, schema: Any) -> int:
    mutations = [
        ("missing", "report", lambda value: value["cases"].pop()),
        ("extra", "report", lambda value: value["cases"].append(copy.deepcopy(value["cases"][-1]))),
        ("duplicate", "report", lambda value: value["cases"].__setitem__(1, copy.deepcopy(value["cases"][0]))),
        ("order", "report", lambda value: value["cases"].reverse()),
        ("status", "report", lambda value: value.update(status="closed")),
        ("candidate", "report", lambda value: value.update(source_candidate="0" * 40)),
        ("source", "report", lambda value: value.update(production_source_sha256="0" * 64)),
        ("property", "report", lambda value: value["cases"][0].update(property="IDENTITY_MISMATCH")),
        ("closure", "report", lambda value: value.update(closure_evidence=True)),
        ("schema", "schema", lambda value: value.update(additionalProperties=True)),
    ]
    caught = 0
    for label, target, mutation in mutations:
        values = {"report": copy.deepcopy(report), "schema": copy.deepcopy(schema)}
        mutation(values[target])
        try:
            validate(values["report"], values["schema"])
        except ReproductionError:
            caught += 1
            continue
        raise ReproductionError("mutation_survived:" + label)
    return caught


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--run-open", action="store_true")
    args = parser.parse_args()
    report = load(REPORT_PATH)
    schema = load(SCHEMA_PATH)
    validate(report, schema)
    mutations = self_test(report, schema)
    if args.run_open:
        run_cases(report)
    print(
        "PASS: causal projection actor reproductions v16 "
        f"cases={len(report['cases'])} expected_failures={len(report['cases'])} mutations={mutations}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
