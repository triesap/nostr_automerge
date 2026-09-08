#!/usr/bin/env python3
"""Reproduce the four v19 assurance defects from immutable v18 evidence."""

from __future__ import annotations

import copy
import json
import re
import subprocess
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[1]
BASE = "4cc1bb5060b7477214eb22cf7b086735e4a70b7a"
PREDECESSOR = "8673ff8546b9e9d57218c15a4b81890d82137184"
REPORT = ROOT / "reports/causal_projection_reproductions_v19.json"
SCHEMA = ROOT / "tools/validation/causal_projection_reproductions_v19.schema.json"


class ReproductionError(RuntimeError):
    pass


def require(condition: bool, code: str) -> None:
    if not condition:
        raise ReproductionError(code)


def load(path: Path) -> Any:
    def closed(pairs: list[tuple[str, Any]]) -> dict[str, Any]:
        keys = [key for key, _ in pairs]
        require(len(keys) == len(set(keys)), "duplicate:" + path.name)
        return dict(pairs)

    return json.loads(path.read_text(), object_pairs_hook=closed)


def git_text(path: str) -> str:
    result = subprocess.run(["git", "show", f"{BASE}:{path}"], cwd=ROOT, capture_output=True, text=True, check=False)
    require(result.returncode == 0, "git_show:" + path)
    return result.stdout


def git_lines(*args: str) -> list[str]:
    result = subprocess.run(["git", *args], cwd=ROOT, capture_output=True, text=True, check=False)
    require(result.returncode == 0, "git:" + ":".join(args))
    return [line for line in result.stdout.splitlines() if line]


def expected() -> dict[str, Any]:
    actor = git_text("crates/nostr_automerge/src/graph/actor_state.rs")
    mutations = git_text("scripts/run_causal_projection_mutations_v18.py")
    properties = git_text("scripts/validate_causal_projection_properties_v18.py")
    require("let completions = targets;" in actor, "proof:copied_completion")
    require("TargetDispatched" not in actor and "TargetReturned" not in actor, "proof:independent_events")
    require('f"helper.{phase}.target_before_charge"' in mutations, "matrix:target_before")
    require("completion_before_target" not in mutations and "double_target" not in mutations, "matrix:missing_classes")
    require("PROVENANCE_MARKERS" in properties and "for marker, code in PROVENANCE_MARKERS.items()" in properties, "oracle:markers")
    require("let _v18_returned = Ok(result);" in mutations and "_v18_returned" in mutations, "oracle:inert_mutant")
    commits = git_lines("rev-list", f"{PREDECESSOR}..{BASE}")
    deviations = git_lines("ls-tree", "-r", "--name-only", BASE, "implementation/deviations")
    require(len(commits) == 21, "mapping:commit_count")
    require(not any("v18" in path or "1514" in path for path in deviations), "mapping:deviation_absent")
    return {
        "schema": "nostr_automerge.causal_projection_reproductions.v19.v1",
        "status": "expected_defects_confirmed",
        "baseline": BASE,
        "proof_event_alias": {"target_event": "TargetCompleted", "completion_count_copied_from_target": True, "independent_dispatch_present": False, "independent_return_present": False, "finding": "FINDING_130"},
        "helper_matrix_gap": {"helper_count": 4, "observed_classes_per_helper": 1, "observed_cells": 4, "required_classes_per_helper": 7, "required_cells": 28, "missing_cells": 24, "finding": "FINDING_131"},
        "property_oracle_gap": {"marker_selected_codes": True, "completion_after_return_mutant_is_inert": True, "finding": "FINDING_132"},
        "execution_mapping_gap": {"approved_step_count": 43, "actual_public_commit_count": len(commits), "v18_mapping_present": False, "v18_deviation_present": False, "finding": "FINDING_133"},
        "runtime_defect_observed": False,
        "result": "pass",
    }


def validate(report: dict[str, Any], schema: dict[str, Any]) -> None:
    require(schema["additionalProperties"] is False, "schema:closed")
    require(list(report) == schema["required"], "report:fields")
    require(report == expected(), "report:value")


def self_test(values: list[dict[str, Any]]) -> int:
    cases = [
        lambda r, _s: r["proof_event_alias"].update(completion_count_copied_from_target=False),
        lambda r, _s: r["helper_matrix_gap"].update(observed_cells=28),
        lambda r, _s: r["property_oracle_gap"].update(marker_selected_codes=False),
        lambda r, _s: r["execution_mapping_gap"].update(v18_mapping_present=True),
        lambda r, _s: r.update(runtime_defect_observed=True),
        lambda _r, s: s.update(additionalProperties=True),
    ]
    caught = 0
    for mutate in cases:
        changed = copy.deepcopy(values)
        mutate(*changed)
        try:
            validate(*changed)
        except ReproductionError:
            caught += 1
            continue
        raise ReproductionError("self_test:survivor")
    return caught


def main() -> int:
    values = [load(REPORT), load(SCHEMA)]
    validate(*values)
    print(f"PASS v19 assurance reproductions: findings=4 public_commits=21 attacks={self_test(values)} runtime_defect=false")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
