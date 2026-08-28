#!/usr/bin/env python3
"""Validate and optionally execute the causal-projection reproductions."""

from __future__ import annotations

import argparse
import copy
import json
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
REGISTRY = ROOT / "spec/remediation_v13_reproductions.json"
FIELDS = ["schema","status","cases","result"]
CASE_FIELDS = ["id","finding","test","expected"]
IDS = ["f104_causal_projection_final_scan","f108_projection_operation_boundary"]
FINDINGS = ["FINDING_104","FINDING_108"]

class ReproductionError(RuntimeError):
    pass

def require(condition: bool, label: str) -> None:
    if not condition:
        raise ReproductionError(label)

def validate(registry: object) -> None:
    require(type(registry) is dict and list(registry) == FIELDS, "registry:shape")
    require(registry["schema"] == "nostr_automerge.remediation_v13_reproductions.v1" and registry["status"] == "mixed" and registry["result"] == "pass", "registry:state")
    cases = registry["cases"]
    require(type(cases) is list and len(cases) == 2, "cases:count")
    require([row["id"] for row in cases] == IDS and [row["finding"] for row in cases] == FINDINGS, "cases:order")
    source = (ROOT / "crates/nostr_automerge/tests/remediation_v13_reproductions.rs").read_text()
    for index, row in enumerate(cases):
        require(type(row) is dict and list(row) == CASE_FIELDS, f"case:{index}:shape")
        require(row["expected"] == ["pass", "failed_assertion"][index] and row["test"] in source, f"case:{index}:binding")

def transcript_is_exact_pass(test: str, returncode: int, output: str) -> bool:
    return returncode == 0 and f"test {test} ... ok" in output and "1 passed; 0 failed; 0 ignored" in output

def transcript_is_expected_failure(test: str, returncode: int, output: str) -> bool:
    return returncode != 0 and f"test {test} ... FAILED" in output and "test result: FAILED" in output

def run_cases(registry: dict) -> None:
    for row in registry["cases"]:
        command = ["cargo","test","-p","nostr_automerge","--test","remediation_v13_reproductions",row["test"],"--","--ignored","--exact"]
        if row["expected"] == "pass":
            command.remove("--ignored")
        result = subprocess.run(command,cwd=ROOT,capture_output=True,text=True,check=False)
        output = result.stdout + result.stderr
        if row["expected"] == "pass":
            require(transcript_is_exact_pass(row["test"],result.returncode,output), "case:not_exact_pass:" + row["id"])
        else:
            require(transcript_is_expected_failure(row["test"],result.returncode,output), "case:not_expected_failure:" + row["id"])

def self_test(registry: dict) -> int:
    mutations = []
    for label, mutate in [
        ("missing",lambda value: value["cases"].pop()),
        ("extra",lambda value: value["cases"].append(copy.deepcopy(value["cases"][-1]))),
        ("duplicate",lambda value: value["cases"].__setitem__(1,copy.deepcopy(value["cases"][0]))),
        ("order",lambda value: value["cases"].reverse()),
        ("closed",lambda value: value.update(status="pass")),
        ("wrong_test",lambda value: value["cases"][0].update(test="missing_test")),
    ]:
        changed = copy.deepcopy(registry); mutate(changed); mutations.append((label,changed))
    for label, changed in mutations:
        try: validate(changed)
        except ReproductionError: continue
        raise ReproductionError("mutation_survived:" + label)
    require(not transcript_is_expected_failure("proof","1","test unrelated ... FAILED\ntest result: FAILED"), "transcript:wrong_test")
    require(not transcript_is_expected_failure("proof",0,"test proof ... ok\ntest result: ok"), "transcript:false_pass")
    require(not transcript_is_exact_pass("proof", 0, "test unrelated ... ok\ntest result: ok. 1 passed; 0 failed; 0 ignored"), "transcript:wrong_pass")
    require(not transcript_is_exact_pass("proof", 1, "test proof ... ok\ntest result: ok. 1 passed; 0 failed; 0 ignored"), "transcript:failed_pass")
    return len(mutations) + 4

def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--run-open",action="store_true")
    args = parser.parse_args()
    registry = json.loads(REGISTRY.read_text())
    validate(registry)
    mutations = self_test(registry)
    if args.run_open: run_cases(registry)
    print(f"PASS: remediation-v13 reproductions cases=2 mutations={mutations} fixed=1 open=1")
    return 0

if __name__ == "__main__":
    raise SystemExit(main())
