#!/usr/bin/env python3
"""Validate complete assignment of the reviewed v19 step proposal."""

from __future__ import annotations

import copy
import json
from collections import Counter
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[1]
REPORT = ROOT / "implementation/v19_sequence_reconciliation.json"
SCHEMA = ROOT / "tools/validation/v19_sequence_reconciliation.schema.json"
CHECKPOINTS = ["141.1", "141.2", "141.3", "141.4", "142.1", "142.2", "142.3", "142.4", "143.1", "143.2", "143.3", "143.4", "144.1", "144.2", "144.3", "144.4", "145.1", "145.2", "145.3", "145.4", "145.5", "146.1", "146.2", "146.3", "146.4", "146.5", "146.6"]


class ReconciliationError(RuntimeError):
    pass


def require(condition: bool, code: str) -> None:
    if not condition:
        raise ReconciliationError(code)


def load(path: Path) -> Any:
    def closed(pairs: list[tuple[str, Any]]) -> dict[str, Any]:
        keys = [key for key, _ in pairs]
        require(len(keys) == len(set(keys)), "duplicate:" + path.name)
        return dict(pairs)

    return json.loads(path.read_text(), object_pairs_hook=closed)


def validate(report: dict[str, Any], schema: dict[str, Any]) -> None:
    require(schema["additionalProperties"] is False and list(report) == schema["required"], "schema:closed")
    require(report["schema"] == "nostr_automerge.v19_sequence_reconciliation.v1", "report:schema")
    require(report["status"] == "approved_consolidation" and report["result"] == "pass", "report:status")
    require((report["reviewed_step_first"], report["reviewed_step_last"], report["reviewed_step_count"]) == (1557, 1609, 53), "report:range")
    assignments = report["checkpoint_assignments"]
    require(list(assignments) == CHECKPOINTS, "report:checkpoints")
    counts = Counter(step for steps in assignments.values() for step in steps)
    require(set(counts) == set(range(1557, 1610)), "report:coverage")
    require(all(count == 1 for count in counts.values()), "report:duplicates")
    require(list(report["additional_checkpoints"]) == ["145.4", "146.4", "146.6"], "report:additional")
    require([row["id"] for row in report["deviations"]] == ["v19-coherent-checkpoints", "v19-private-ownership", "v19-native-helper-matrix", "v19-artifact-order", "v19-terminal-order"], "report:deviations")


def self_test(values: list[dict[str, Any]]) -> int:
    cases = [
        lambda r, _s: r.update(reviewed_step_count=52),
        lambda r, _s: r["checkpoint_assignments"]["141.2"].pop(),
        lambda r, _s: r["checkpoint_assignments"]["141.1"].append(1557),
        lambda r, _s: r["checkpoint_assignments"].pop("146.6"),
        lambda r, _s: r["additional_checkpoints"].pop("146.4"),
        lambda r, _s: r["deviations"].pop(),
        lambda _r, s: s.update(additionalProperties=True),
    ]
    caught = 0
    for mutate in cases:
        changed = copy.deepcopy(values)
        mutate(*changed)
        try:
            validate(*changed)
        except ReconciliationError:
            caught += 1
            continue
        raise ReconciliationError("self_test:survivor")
    return caught


def main() -> int:
    values = [load(REPORT), load(SCHEMA)]
    validate(*values)
    print(f"PASS v19 sequence reconciliation: steps=53 checkpoints=27 deviations=5 attacks={self_test(values)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
