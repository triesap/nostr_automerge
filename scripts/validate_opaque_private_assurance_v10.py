#!/usr/bin/env python3
"""Validate the approved opaque private assurance import."""

from __future__ import annotations

import copy
import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
REPORT = ROOT / "reports/opaque_private_assurance_v10.json"
SCHEMA = ROOT / "tools/validation/opaque_private_assurance_v10.schema.json"
CLASSES = ("policy", "standard", "conformance", "coverage", "supply_chain", "robustness", "resource", "release_evidence")
IDENTITIES = (
    "03f0a295681d29b30d4fe7448cd00585318ee50ebe903a72caf0a89a51d19b89", "0db4d1e7f7b93134057ebe4ed8e370b60fe3dffd14de37c99197ff5971e90a4f",
    "3226b1ae0c6534c928e0bcf61e4b82f68d1447060f9e42aa85275d1178ff43c4", "4a663141aa5d122fd388e8c08e115d8ceb58efe8e36408abc0339f9aeba4a958",
    "61717b87fba7ffa7f1a5f0aa1f26cc6a98f3d77233a72fced3d54c677744d65e", "86ec32f34dd99ef0c1e5ea3531360a1f78bf07d62818375096e0bdf0f209b8e5",
    "b81d1be479c59b8b29be9b896a4bb6fa0af502b3faa9771ac4142049ee1433c2", "c9f28deb32dfedce674a6871b0eb949f38b5a5f977a67ca993f7ed639df1e112",
)

def digest(value: object) -> str:
    return hashlib.sha256(json.dumps(value, sort_keys=True, separators=(",", ":")).encode()).hexdigest()

def validate(value: dict[str, object]) -> None:
    expected_keys = ("schema", "checkpoint", "candidate", "status", "publication_status", "scenario_count", "delivery_order_count", "process_count", "runner_job_count", "results", "opaque_identities", "result_identity_sha256")
    assert tuple(value) == expected_keys
    assert value["schema"] == "nostr_automerge.opaque_private_assurance.v10.v1" and value["checkpoint"] == "step_1284"
    assert value["candidate"] == "fd8c436af0ae67aac996fba5ce6eb50e22a7914e"
    assert (value["status"], value["publication_status"]) == ("pass", "held")
    assert (value["scenario_count"], value["delivery_order_count"], value["process_count"], value["runner_job_count"]) == (192, 8, 2, 8)
    assert value["results"] == [{"class": item, "result": "pass"} for item in CLASSES]
    assert value["opaque_identities"] == list(IDENTITIES)
    projection = copy.deepcopy(value); identity = projection.pop("result_identity_sha256")
    assert identity == digest(projection)

def main() -> int:
    report = json.loads(REPORT.read_text()); schema = json.loads(SCHEMA.read_text())
    assert schema["additionalProperties"] is False and schema["required"] == list(report)
    validate(report)
    mutations = []
    for key in report:
        changed = copy.deepcopy(report); changed.pop(key); mutations.append(changed)
    for field in ("results", "opaque_identities"):
        changed = copy.deepcopy(report); changed[field].reverse(); mutations.append(changed)
    caught = 0
    for changed in mutations:
        try: validate(changed)
        except (AssertionError, KeyError): caught += 1
    assert caught == len(mutations)
    print("PASS: opaque private assurance v10")
    print(f"- results={len(CLASSES)}")
    print(f"- opaque_identities={len(IDENTITIES)}")
    print(f"- negative_mutations={caught}")
    return 0

if __name__ == "__main__":
    raise SystemExit(main())
