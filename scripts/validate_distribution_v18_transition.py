#!/usr/bin/env python3
"""Generate and validate the zero-budget-change v18 distribution transition."""

from __future__ import annotations

import argparse
import copy
import hashlib
import json
import subprocess
import sys
from pathlib import Path
from typing import Any

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parents[1]
TRANSITION = ROOT / "spec/distribution_v18_transition.json"
SCHEMA = ROOT / "tools/validation/distribution_v18_transition.schema.json"
GRAPH_PATH = "reports/causal_projection_evidence_graph_v18.json"
GRAPH_CANDIDATE = "6b73727be798e152aa3afbb98bf3683c7e52a393"
MANIFEST_PATH = "fixtures/distribution/manifest_v16.json"
LOCK_PATH = "fixtures/distribution/manifest_v16.lock.json"
PRIOR_PATH = "spec/distribution_v17_transition.json"
CANONICAL_SHA256 = "e69c721549966b1b88dcde3296674d675169840c6e8ebd0f236a5c07bcfc6415"
SERIALIZED_SHA256 = "000c52bde7102eaccec8cf65c875332e119fd25ccf4a2ac38973c456774a3344"


class TransitionError(RuntimeError):
    pass


def require(condition: bool, code: str) -> None:
    if not condition:
        raise TransitionError(code)


def sha(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def committed(candidate: str, path: str) -> bytes:
    completed = subprocess.run(
        ["git", "show", f"{candidate}:{path}"],
        cwd=ROOT,
        capture_output=True,
        check=False,
    )
    require(completed.returncode == 0, f"COMMITTED_PATH:{candidate}:{path}")
    return completed.stdout


def expected() -> dict[str, Any]:
    graph_raw = committed(GRAPH_CANDIDATE, GRAPH_PATH)
    graph = json.loads(graph_raw)
    manifest_raw = (ROOT / MANIFEST_PATH).read_bytes()
    lock_raw = (ROOT / LOCK_PATH).read_bytes()
    prior_raw = (ROOT / PRIOR_PATH).read_bytes()
    manifest, lock = json.loads(manifest_raw), json.loads(lock_raw)
    require(graph["status"] == "final_bidirectional" and graph["result"] == "pass", "GRAPH")
    return {
        "schema": "nostr_automerge.distribution_v18_transition.v1",
        "status": "immutable_reuse",
        "evidence_graph": {
            "path": GRAPH_PATH,
            "candidate": GRAPH_CANDIDATE,
            "sha256": sha(graph_raw),
        },
        "prior_transition": {"path": PRIOR_PATH, "sha256": sha(prior_raw)},
        "selected_manifest": {"path": MANIFEST_PATH, "sha256": sha(manifest_raw)},
        "selected_lock": {"path": LOCK_PATH, "sha256": sha(lock_raw)},
        "affected_fixture_ids": [],
        "counts": {
            "scenarios": len(manifest["fixtures"]),
            "signed_events": lock["signed_event_count"],
            "delivery_orders": 8,
            "processes_required": 2,
            "affected": 0,
        },
        "identity": {
            "signed_events_byte_identical": True,
            "ample_reports_byte_identical": True,
            "canonical_output_sha256": CANONICAL_SHA256,
            "serialized_run_sha256": SERIALIZED_SHA256,
        },
        "derivation": {
            "runtime_contract_changed": True,
            "runtime_budget_change": False,
            "observer_position_only": True,
            "synthetic_version_rebinding": False,
            "new_manifest_created": False,
        },
        "result": "pass",
    }


def validate(report: Any, schema: Any) -> None:
    value = expected()
    require(type(report) is dict and report == value, "TRANSITION_DERIVATION")
    require(report["affected_fixture_ids"] == [] and report["counts"]["affected"] == 0, "AFFECTED_SET")
    require(report["counts"]["scenarios"] == 204 and report["counts"]["signed_events"] == 771, "FROZEN_COUNTS")
    require(report["identity"]["canonical_output_sha256"] == CANONICAL_SHA256, "CANONICAL_OUTPUT")
    require(report["identity"]["serialized_run_sha256"] == SERIALIZED_SHA256, "SERIALIZED_OUTPUT")
    require(not (ROOT / "fixtures/distribution/manifest_v18.json").exists(), "SYNTHETIC_MANIFEST")
    require(type(schema) is dict and schema.get("additionalProperties") is False and schema.get("required") == list(value), "SCHEMA_CLOSED")


def self_test(report: dict[str, Any], schema: dict[str, Any]) -> int:
    attacks = [
        ("report", lambda value: value["affected_fixture_ids"].append("synthetic")),
        ("report", lambda value: value["selected_manifest"].update(path="fixtures/distribution/manifest_v18.json")),
        ("report", lambda value: value["selected_manifest"].update(sha256="0" * 64)),
        ("report", lambda value: value["identity"].update(signed_events_byte_identical=False)),
        ("report", lambda value: value["identity"].update(ample_reports_byte_identical=False)),
        ("report", lambda value: value["derivation"].update(runtime_budget_change=True)),
        ("report", lambda value: value["counts"].update(scenarios=203)),
        ("schema", lambda value: value.update(additionalProperties=True)),
    ]
    caught = 0
    for target, mutate in attacks:
        changed_report, changed_schema = copy.deepcopy(report), copy.deepcopy(schema)
        mutate(changed_report if target == "report" else changed_schema)
        try:
            validate(changed_report, changed_schema)
        except TransitionError:
            caught += 1
            continue
        raise TransitionError("ATTACK_SURVIVED")
    return caught


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--write-report", action="store_true")
    args = parser.parse_args()
    value = expected()
    if args.write_report:
        TRANSITION.write_text(json.dumps(value, ensure_ascii=True, indent=2) + "\n")
    report = json.loads(TRANSITION.read_text())
    schema = json.loads(SCHEMA.read_text())
    validate(report, schema)
    print(
        "PASS: distribution v18 transition selected=v16 scenarios=204 "
        f"affected=0 attacks={self_test(report, schema)}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
