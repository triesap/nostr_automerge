#!/usr/bin/env python3
"""Validate the public, opaque-safe v18 execution mapping."""

from __future__ import annotations

import copy
import json
import re
import subprocess
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[1]
REPORT = ROOT / "reports/causal_projection_v18_execution_mapping_v19.json"
SCHEMA = ROOT / "tools/validation/causal_projection_v18_execution_mapping_v19.schema.json"
SHA = re.compile(r"^[0-9a-f]{40}$")
SHA256 = re.compile(r"^[0-9a-f]{64}$")


class MappingError(RuntimeError):
    pass


def require(condition: bool, code: str) -> None:
    if not condition:
        raise MappingError(code)


def load(path: Path) -> Any:
    def closed(pairs: list[tuple[str, Any]]) -> dict[str, Any]:
        keys = [key for key, _ in pairs]
        require(len(keys) == len(set(keys)), "duplicate:" + path.name)
        return dict(pairs)

    return json.loads(path.read_text(), object_pairs_hook=closed)


def git(*args: str) -> list[str]:
    result = subprocess.run(["git", *args], cwd=ROOT, capture_output=True, text=True, check=False)
    require(result.returncode == 0, "git:" + ":".join(args))
    return [line for line in result.stdout.splitlines() if line]


def validate(report: dict[str, Any], schema: dict[str, Any]) -> None:
    require(schema["additionalProperties"] is False and list(report) == schema["required"], "schema:closed")
    require(report["schema"] == "nostr_automerge.causal_projection_v18_execution_mapping.v19.v1", "report:schema")
    require(report["status"] == "historical_execution_mapped" and report["result"] == "pass", "report:status")
    mapping = report["public_step_mapping"]
    independent = report["independent_steps"]
    expected_public_steps = [f"step_{number}" for number in range(1514, 1557) if number not in {1551, 1552, 1553}]
    require(list(mapping) == expected_public_steps, "mapping:steps")
    require(independent == ["step_1551", "step_1552", "step_1553"], "mapping:independent_steps")
    require(len(mapping) + len(independent) == report["approved_step_count"] == 43, "mapping:total")
    require(all(commits and all(SHA.fullmatch(commit) for commit in commits) for commits in mapping.values()), "mapping:commits")
    public = report["actual_public_commits"]
    require(len(public) == len(set(public)) == 21, "mapping:public_count")
    require({commit for commits in mapping.values() for commit in commits} == set(public), "mapping:reverse")
    observed = git("rev-list", "--reverse", f'{report["approved_baseline"]}..{report["public_terminal"]}')
    require(observed == public, "mapping:history")
    opaque = report["independent_mapping"]
    require(opaque["checkpoint_count"] == 17 and SHA.fullmatch(opaque["terminal_candidate"]) is not None, "mapping:opaque_count")
    require(SHA256.fullmatch(opaque["exact_mapping_sha256"]) is not None and opaque["detail"] == "opaque_only", "mapping:opaque_hash")
    require(report["deviations"] == ["sequence_refinement", "many_to_one", "one_to_many", "reordered", "terminal_split"], "mapping:deviations")


def self_test(values: list[dict[str, Any]]) -> int:
    cases = [
        lambda r, _s: r.update(approved_step_count=42),
        lambda r, _s: r["public_step_mapping"].pop("step_1514"),
        lambda r, _s: r["independent_steps"].pop(),
        lambda r, _s: r["actual_public_commits"].pop(),
        lambda r, _s: r["public_step_mapping"]["step_1514"].clear(),
        lambda r, _s: r["independent_mapping"].update(detail="expanded"),
        lambda r, _s: r["deviations"].pop(),
        lambda _r, s: s.update(additionalProperties=True),
    ]
    caught = 0
    for mutate in cases:
        changed = copy.deepcopy(values)
        mutate(*changed)
        try:
            validate(*changed)
        except MappingError:
            caught += 1
            continue
        raise MappingError("self_test:survivor")
    return caught


def main() -> int:
    values = [load(REPORT), load(SCHEMA)]
    validate(*values)
    print(f"PASS public v18 execution mapping: steps=43 public=21 independent=17 attacks={self_test(values)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
