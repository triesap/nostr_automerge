#!/usr/bin/env python3
"""Validate the v17 zero-budget-change distribution transition."""

from __future__ import annotations

import argparse
import copy
import hashlib
import json
import subprocess
import sys
from pathlib import Path

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parents[1]
TRANSITION = ROOT / "spec/distribution_v17_transition.json"
SCHEMA = ROOT / "tools/validation/distribution_v17_transition.schema.json"
MANIFEST_PATH = "fixtures/distribution/manifest_v16.json"
LOCK_PATH = "fixtures/distribution/manifest_v16.lock.json"
V16_TRANSITION_PATH = "spec/distribution_v16_transition.json"
ASSURANCE_PATH = "reports/causal_projection_public_assurance_v17.json"
ASSURANCE_CANDIDATE = "54a983fc2608ea9ca869c8fb344139e3b2b718a4"


class TransitionError(RuntimeError):
    pass


def require(condition: bool, code: str) -> None:
    if not condition:
        raise TransitionError(code)


def sha(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def committed(candidate: str, path: str) -> bytes:
    result = subprocess.run(["git", "show", f"{candidate}:{path}"], cwd=ROOT, capture_output=True, check=False)
    require(result.returncode == 0, "ASSURANCE_CANDIDATE")
    return result.stdout


def expected() -> dict[str, object]:
    assurance_bytes = committed(ASSURANCE_CANDIDATE, ASSURANCE_PATH)
    assurance = json.loads(assurance_bytes)
    require(assurance["result"] == "pass" and assurance["canonical_output"]["changed"] is False, "PUBLIC_ASSURANCE")
    manifest = json.loads((ROOT / MANIFEST_PATH).read_text())
    lock = json.loads((ROOT / LOCK_PATH).read_text())
    return {
        "schema": "nostr_automerge.distribution_v17_transition.v1",
        "status": "immutable_reuse",
        "public_assurance": {"path": ASSURANCE_PATH, "candidate": ASSURANCE_CANDIDATE, "sha256": hashlib.sha256(assurance_bytes).hexdigest()},
        "prior_transition": {"path": V16_TRANSITION_PATH, "sha256": sha(ROOT / V16_TRANSITION_PATH)},
        "selected_manifest": {"path": MANIFEST_PATH, "sha256": sha(ROOT / MANIFEST_PATH)},
        "selected_lock": {"path": LOCK_PATH, "sha256": sha(ROOT / LOCK_PATH)},
        "affected_fixture_ids": [],
        "counts": {"scenarios": len(manifest["fixtures"]), "signed_events": lock["signed_event_count"], "delivery_orders": 8, "processes_required": 2, "affected": 0},
        "identity": {"signed_events_byte_identical": True, "ample_reports_byte_identical": True, "canonical_output_sha256": assurance["canonical_output"]["sha256"]},
        "derivation": {"runtime_budget_change": False, "synthetic_version_rebinding": False, "new_manifest_created": False},
        "result": "pass",
    }


def validate(report: object, schema: object) -> None:
    value = expected()
    require(type(report) is dict and report == value, "TRANSITION_DERIVATION")
    require(report["status"] == "immutable_reuse" and report["affected_fixture_ids"] == [], "AFFECTED_SET")
    require(report["counts"] == {"scenarios": 204, "signed_events": 771, "delivery_orders": 8, "processes_required": 2, "affected": 0}, "COUNTS")
    require(not (ROOT / "fixtures/distribution/manifest_v17.json").exists(), "SYNTHETIC_MANIFEST")
    require(report["identity"]["canonical_output_sha256"] == "e69c721549966b1b88dcde3296674d675169840c6e8ebd0f236a5c07bcfc6415", "CANONICAL_OUTPUT")
    require(schema.get("additionalProperties") is False and schema.get("required") == list(value), "SCHEMA_CLOSED")


def self_test(report: dict, schema: dict) -> int:
    attacks = [
        lambda value: value["affected_fixture_ids"].append("synthetic"),
        lambda value: value["selected_manifest"].update(path="fixtures/distribution/manifest_v17.json"),
        lambda value: value["selected_manifest"].update(sha256="0" * 64),
        lambda value: value["identity"].update(signed_events_byte_identical=False),
        lambda value: value["identity"].update(ample_reports_byte_identical=False),
        lambda value: value["derivation"].update(synthetic_version_rebinding=True),
        lambda value: value["counts"].update(scenarios=203),
    ]
    caught = 0
    for attack in attacks:
        changed = copy.deepcopy(report); attack(changed)
        try:
            validate(changed, schema)
        except TransitionError:
            caught += 1
            continue
        raise TransitionError("ATTACK_SURVIVED")
    return caught


def main() -> int:
    parser = argparse.ArgumentParser(); parser.add_argument("--write", action="store_true"); args = parser.parse_args()
    value = expected()
    if args.write:
        TRANSITION.write_text(json.dumps(value, ensure_ascii=True, indent=2) + "\n")
    report = json.loads(TRANSITION.read_text()); schema = json.loads(SCHEMA.read_text())
    validate(report, schema); attacks = self_test(report, schema)
    print(f"PASS: distribution v17 transition selected=v16 scenarios=204 affected=0 attacks={attacks}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
