#!/usr/bin/env python3
"""Validate the final finding-by-finding closure ledger."""

from __future__ import annotations

import copy
import hashlib
import json
from pathlib import Path
from typing import Any


ROOT = Path(__file__).resolve().parents[1]
REPORT = ROOT / "reports/final_finding_closure_v10.json"
SCHEMA = ROOT / "tools/validation/final_finding_closure_v10.schema.json"
FINDING_IDS = tuple(
    [f"FINDING_{number:03d}" for number in range(73, 81)]
    + [f"FINDING_{number:03d}" for number in range(81, 94)]
)
REPORT_KEYS = (
    "schema",
    "checkpoint",
    "candidate",
    "status",
    "finding_count",
    "closed_count",
    "held_count",
    "findings",
    "finding_catalog_identity_sha256",
    "final_identity_sha256",
    "external_hold_count",
    "release_claimed",
    "remote_actions_performed",
    "result_identity_sha256",
)


def digest(value: Any) -> str:
    encoded = json.dumps(value, sort_keys=True, separators=(",", ":")).encode()
    return hashlib.sha256(encoded).hexdigest()


def validate(value: dict[str, Any]) -> None:
    assert tuple(value) == REPORT_KEYS
    assert (
        value["schema"],
        value["checkpoint"],
        value["candidate"],
        value["status"],
    ) == (
        "nostr_automerge.final_finding_closure.v10.v1",
        "step_1286",
        "402c928ad2dfa173e7a5876930fb9e771aba8598",
        "code_complete_publication_held",
    )
    expected_findings = [
        {
            "id": finding_id,
            "status": "held" if finding_id == "FINDING_080" else "closed",
        }
        for finding_id in FINDING_IDS
    ]
    assert value["findings"] == expected_findings
    assert (
        value["finding_count"],
        value["closed_count"],
        value["held_count"],
    ) == (21, 20, 1)
    assert value["finding_catalog_identity_sha256"] == (
        "0eb24b686f6ac30ff308981822d490525574a95e0f4cd7f9e752c191efe1a10d"
    )
    assert value["final_identity_sha256"] == (
        "b77f0d208ed0f1366211ed349c9e28f05df56b6e5b78d003a9f2998bacb8701c"
    )
    assert (
        value["external_hold_count"],
        value["release_claimed"],
        value["remote_actions_performed"],
    ) == (6, False, False)
    projection = copy.deepcopy(value)
    identity = projection.pop("result_identity_sha256")
    assert identity == digest(projection)


def main() -> int:
    value = json.loads(REPORT.read_text())
    schema = json.loads(SCHEMA.read_text())
    assert schema["additionalProperties"] is False
    assert schema["required"] == list(value)
    validate(value)

    mutations = []
    for key in value:
        changed = copy.deepcopy(value)
        changed.pop(key)
        mutations.append(changed)
    changed = copy.deepcopy(value)
    changed["findings"].reverse()
    mutations.append(changed)
    changed = copy.deepcopy(value)
    changed["findings"][7]["status"] = "closed"
    mutations.append(changed)

    caught = 0
    for changed in mutations:
        try:
            validate(changed)
        except (AssertionError, KeyError):
            caught += 1
    assert caught == len(mutations)
    print(
        f"PASS: final finding closure "
        f"({len(FINDING_IDS)} findings, {caught} mutations)"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
