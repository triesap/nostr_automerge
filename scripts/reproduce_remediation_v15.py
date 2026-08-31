#!/usr/bin/env python3
"""Validate and execute the exact v15 expected-defect reproductions."""

from __future__ import annotations

import argparse
import copy
import json
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
REGISTRY = ROOT / "spec/remediation_v15_reproductions.json"
TEST_SOURCE = ROOT / "crates/nostr_automerge/tests/remediation_v15_reproductions.rs"
FIELDS = ["schema","status","cases","result"]
CASE_FIELDS = ["id","finding","test","expected"]
IDS = ["candidate_identity_comparison","dependency_count_read","candidate_readiness_comparison","candidate_kind_comparison","remaining_state_write","terminal_completion_comparison","initial_count_compound_operation","rust_shared_reference_clone_reachability","candidate_consumer_inventory","behavioral_source_mutations"]


class ReproductionError(RuntimeError):
    pass


def require(condition: bool, label: str) -> None:
    if not condition:
        raise ReproductionError(label)


def validate(registry: object) -> None:
    require(type(registry) is dict and list(registry) == FIELDS, "registry:shape")
    require(registry["schema"] == "nostr_automerge.remediation_v15_reproductions.v1" and registry["status"] == "mixed" and registry["result"] == "pass", "registry:state")
    cases = registry["cases"]
    require(type(cases) is list and [row["id"] for row in cases] == IDS, "cases:order")
    source = TEST_SOURCE.read_text()
    for index, row in enumerate(cases):
        require(type(row) is dict and list(row) == CASE_FIELDS, f"case:{index}:shape")
        require(row["finding"] in {"FINDING_113","FINDING_114","FINDING_115"} and row["expected"] in {"pass","fail"}, f"case:{index}:value")
        require(f"fn {row['test']}()" in source, f"case:{index}:binding")


def exact_failure(test: str, result: subprocess.CompletedProcess[str]) -> bool:
    output = result.stdout + result.stderr
    return result.returncode != 0 and f"test {test} ... FAILED" in output and "1 failed" in output and "0 ignored" in output


def exact_pass(test: str, result: subprocess.CompletedProcess[str]) -> bool:
    output = result.stdout + result.stderr
    return result.returncode == 0 and f"test {test} ... ok" in output and "1 passed" in output and "0 failed" in output and "0 ignored" in output


def run_cases(registry: dict) -> None:
    for row in registry["cases"]:
        command = ["cargo","test","-p","nostr_automerge","--test","remediation_v15_reproductions",row["test"],"--locked","--","--exact"]
        if row["expected"] == "fail": command.insert(-1,"--ignored")
        result = subprocess.run(command,cwd=ROOT,capture_output=True,text=True,check=False)
        accepted = exact_pass(row["test"],result) if row["expected"] == "pass" else exact_failure(row["test"],result)
        require(accepted, "case:not_exact_result:" + row["id"])


def self_test(registry: dict) -> int:
    cases = [
        ("missing",lambda value: value["cases"].pop()),
        ("extra",lambda value: value["cases"].append(copy.deepcopy(value["cases"][-1]))),
        ("duplicate",lambda value: value["cases"].__setitem__(1,copy.deepcopy(value["cases"][0]))),
        ("order",lambda value: value["cases"].reverse()),
        ("closed",lambda value: value.update(status="fixed")),
        ("wrong_test",lambda value: value["cases"][0].update(test="missing_test")),
        ("wrong_finding",lambda value: value["cases"][0].update(finding="FINDING_080")),
    ]
    caught = 0
    for label, mutate in cases:
        changed = copy.deepcopy(registry)
        mutate(changed)
        try:
            validate(changed)
        except ReproductionError:
            caught += 1
            continue
        raise ReproductionError("mutation_survived:" + label)
    return caught


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--run-open",action="store_true")
    args = parser.parse_args()
    registry = json.loads(REGISTRY.read_text())
    validate(registry)
    mutations = self_test(registry)
    if args.run_open:
        run_cases(registry)
    fixed = sum(row["expected"] == "pass" for row in registry["cases"])
    print(f"PASS: remediation-v15 reproductions cases=10 mutations={mutations} fixed={fixed} open={10-fixed}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
