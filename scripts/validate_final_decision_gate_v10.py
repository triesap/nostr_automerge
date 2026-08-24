#!/usr/bin/env python3
"""Validate the terminal local decision gate for the v10 remediation chain."""

from __future__ import annotations

import copy
import hashlib
import json
from pathlib import Path
from typing import Any


ROOT = Path(__file__).resolve().parents[1]
REPORT = ROOT / "reports/final_decision_gate_v10.json"
SCHEMA = ROOT / "tools/validation/final_decision_gate_v10.schema.json"
LEDGER = ROOT / "implementation/runtime_ledger_v9.json"
PUBLIC_ASSURANCE = ROOT / "reports/public_assurance_v10.json"
PRIVATE_ASSURANCE = ROOT / "reports/opaque_private_assurance_v10.json"
SEMANTIC_EVIDENCE = ROOT / "reports/semantic_evidence_gate_v10.json"
SIGNED_CONFORMANCE = ROOT / "reports/signed_conformance_gate_v10.json"
FINAL_IDENTITY = ROOT / "reports/final_identity_v10.json"
FINDING_CLOSURE = ROOT / "reports/final_finding_closure_v10.json"

REPORT_KEYS = (
    "schema",
    "checkpoint",
    "candidate",
    "status",
    "completed_rclds",
    "checkpoint_range",
    "requirement_count",
    "scenario_count",
    "delivery_order_count",
    "process_count",
    "finding_count",
    "closed_finding_count",
    "held_finding_count",
    "public_lane_count",
    "private_result_count",
    "public_held_campaign_count",
    "external_hold_count",
    "evidence_identities",
    "release_claimed",
    "remote_actions_performed",
    "result_identity_sha256",
)
EVIDENCE_FILES = (
    ("public_assurance", PUBLIC_ASSURANCE),
    ("private_assurance", PRIVATE_ASSURANCE),
    ("semantic_evidence", SEMANTIC_EVIDENCE),
    ("signed_conformance", SIGNED_CONFORMANCE),
    ("final_identity", FINAL_IDENTITY),
    ("final_finding_closure", FINDING_CLOSURE),
)


def file_digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def projection_digest(value: Any) -> str:
    encoded = json.dumps(value, sort_keys=True, separators=(",", ":")).encode()
    return hashlib.sha256(encoded).hexdigest()


def load(path: Path) -> dict[str, Any]:
    value = json.loads(path.read_text())
    assert isinstance(value, dict)
    return value


def validate(value: dict[str, Any], ledger: dict[str, Any]) -> None:
    assert tuple(value) == REPORT_KEYS
    assert (
        value["schema"],
        value["checkpoint"],
        value["candidate"],
        value["status"],
    ) == (
        "nostr_automerge.final_decision_gate.v10.v1",
        "step_1287",
        "ed21e29a7179f9bcd5c2827f2bdd0dfddf36e417",
        "code_complete_publication_held",
    )
    assert value["completed_rclds"] == list(range(81, 95))
    assert value["checkpoint_range"] == {
        "first": "step_1158",
        "last": "step_1287",
        "count": 130,
        "contiguous": True,
    }
    assert (
        value["requirement_count"],
        value["scenario_count"],
        value["delivery_order_count"],
        value["process_count"],
    ) == (148, 192, 8, 2)
    assert (
        value["finding_count"],
        value["closed_finding_count"],
        value["held_finding_count"],
    ) == (21, 20, 1)
    assert (
        value["public_lane_count"],
        value["private_result_count"],
        value["public_held_campaign_count"],
        value["external_hold_count"],
    ) == (12, 8, 7, 6)
    assert value["evidence_identities"] == [
        {"class": evidence_class, "sha256": file_digest(path)}
        for evidence_class, path in EVIDENCE_FILES
    ]
    assert (value["release_claimed"], value["remote_actions_performed"]) == (
        False,
        False,
    )

    predecessors = ledger["predecessors"]
    assert len(predecessors) == 129
    assert [row["step"] for row in predecessors] == [
        f"step_{number}" for number in range(1158, 1287)
    ]
    assert all(row["result"] == "pass" for row in predecessors)
    assert predecessors[-1]["candidate"] == value["candidate"]
    assert ledger["status"] == "code_complete_publication_held"
    assert ledger["rcld"] == 94
    assert ledger["cursor"] == {
        "active_step": "step_1287",
        "next_step": "step_1288",
        "last_step": "step_1287",
        "remaining_checkpoint_count": 0,
        "first_rcld": 94,
        "last_rcld": 94,
        "remaining_rcld_count": 0,
    }
    assert ledger["requirements"]["current_count"] == 148
    assert ledger["authority_projection"]["signed_fixture_count"] == 192
    assert ledger["findings"]["status"] == "code_complete_publication_held"

    public = load(PUBLIC_ASSURANCE)
    private = load(PRIVATE_ASSURANCE)
    closure = load(FINDING_CLOSURE)
    assert len(public["lanes"]) == 12
    assert all(row["result"] == "pass" for row in public["lanes"])
    assert len(public["held_campaigns"]) == 7
    assert len(private["results"]) == 8
    assert all(row["result"] == "pass" for row in private["results"])
    assert (private["scenario_count"], private["delivery_order_count"], private["process_count"]) == (192, 8, 2)
    assert (closure["finding_count"], closure["closed_count"], closure["held_count"]) == (21, 20, 1)

    projection = copy.deepcopy(value)
    identity = projection.pop("result_identity_sha256")
    assert identity == projection_digest(projection)


def main() -> int:
    value = load(REPORT)
    ledger = load(LEDGER)
    schema = load(SCHEMA)
    assert schema["additionalProperties"] is False
    assert schema["required"] == list(value)
    validate(value, ledger)

    mutations = []
    for key in value:
        changed = copy.deepcopy(value)
        changed.pop(key)
        mutations.append((changed, ledger))
    changed = copy.deepcopy(value)
    changed["completed_rclds"].reverse()
    mutations.append((changed, ledger))
    changed = copy.deepcopy(value)
    changed["evidence_identities"].reverse()
    mutations.append((changed, ledger))
    changed_ledger = copy.deepcopy(ledger)
    changed_ledger["predecessors"].pop()
    mutations.append((value, changed_ledger))
    changed_ledger = copy.deepcopy(ledger)
    changed_ledger["predecessors"][-1]["candidate"] = "0" * 40
    mutations.append((value, changed_ledger))

    caught = 0
    for changed, changed_ledger in mutations:
        try:
            validate(changed, changed_ledger)
        except (AssertionError, KeyError):
            caught += 1
    assert caught == len(mutations)
    print(
        "PASS: final decision gate "
        f"(130 checkpoints, 14 RCLDs, {caught} mutations)"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
