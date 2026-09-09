#!/usr/bin/env python3
"""Validate the non-terminal public-side v19 assurance closure."""

from __future__ import annotations

import argparse
import copy
import hashlib
import json
import subprocess
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[1]
REPORT = ROOT / "reports/causal_projection_public_assurance_v19.json"
SCHEMA = ROOT / "tools/validation/causal_projection_public_assurance_v19.schema.json"
LEDGER = ROOT / "implementation/runtime_ledger_v19.json"
FINDINGS = ROOT / "spec/remediation_findings_v19.json"
HOLDS = [
    "external_assurance", "event_kind_allocation", "nip_submission",
    "production_qualification", "publication", "release", "deployment",
    "remote_mutation",
]
ARTIFACTS = [
    ("v18_execution_mapping", "reports/causal_projection_v18_execution_mapping_v19.json", "50457ab4efd482582291c77dd3c44d3e3ecc0adb", "41bb2d96e34ecefee89b86036b03a58306c19d9e4e0762915d34ecd489745a7d"),
    ("proofs", "reports/causal_projection_proofs_v19.json", "7c5a1603c4ee9f92d541a8ff6fd66071806518df", "875876bf9937fa581f413a8180325f7a55e3eb1230ed4383f7da16df2036dc48"),
    ("mutations", "reports/causal_projection_mutations_v19.json", "001bea9cb5edc8fb83aa972901f749f4707d59e1", "8a0e1a5c7d4f3c8e6c4b20a6928eda63a0f2d382b37db2ad5a992620ee1954fc"),
    ("catalogs", "reports/causal_projection_catalogs_v19.json", "266e80b67e13ea377ea874bbd4b75ac4ca5898fa", "132dc6a539cf3d435cb528080482c13313144d461e33002d9a457cde191a4a42"),
    ("final_inventory", "reports/causal_projection_final_inventory_v19.json", "b489ef551ace84791f3d513dd5add0a5729b53a7", "12d7d81d9370beb309c6f94846a884cb80c686fa18d3a5dd99b127649aad0308"),
    ("evidence_graph", "reports/causal_projection_evidence_graph_v19.json", "ac24566ae4b96bc1ac17b0c87b7ae70cbb9f939f", "0b0c5cd2e54805cd888bcb04306b921f7e31737b5524282f1d57b5dba7a6d4f3"),
    ("distribution_transition", "spec/distribution_v19_transition.json", "53f0bf69809cfdf2c59393dc9c3f696940ff7554", "e2cf7090b4d901deeeb77fe6f59d3a5ceaeee43ae397e92de001101ee5cd9aa4"),
    ("public_qualification", "reports/causal_projection_public_qualification_v19.json", "0735f7e23524764c889f4ec0cc9942c9e795740a", "a66fb0b6dbbeeee68dd5efcb9e8223957d9baaa380c2b3c332fc23acadc0c65f"),
]


class AssuranceError(RuntimeError):
    pass


def require(condition: bool, code: str) -> None:
    if not condition:
        raise AssuranceError(code)


def load(path: Path) -> dict[str, Any]:
    return json.loads(path.read_text())


def git(*args: str, binary: bool = False) -> str | bytes:
    result = subprocess.run(["git", *args], cwd=ROOT, capture_output=True, check=False)
    require(result.returncode == 0, "git:" + ":".join(args))
    return result.stdout if binary else result.stdout.decode().strip()


def artifact_rows() -> list[dict[str, str]]:
    return [
        {"role": role, "path": path, "artifact_commit": candidate, "sha256": digest, "result": "pass"}
        for role, path, candidate, digest in ARTIFACTS
    ]


def expected_report() -> dict[str, Any]:
    return {
        "schema": "nostr_automerge.causal_projection_public_assurance.v19.v1",
        "status": "public_scope_complete_private_pending",
        "candidate_roles": {
            "source_candidate": "a8f8651c26a0ac7822e1e5d61f3aafa3fec53d1f",
            "proof_execution_base": "7050d3abf4104d5772e05b037a5799904de031f2",
            "proof_artifact_commit": "7c5a1603c4ee9f92d541a8ff6fd66071806518df",
            "mutation_execution_base": "8d14a3cbfddd80da4c621cdbb346997619b57b95",
            "mutation_artifact_commit": "001bea9cb5edc8fb83aa972901f749f4707d59e1",
            "final_inventory_commit": "b489ef551ace84791f3d513dd5add0a5729b53a7",
            "evidence_graph_commit": "ac24566ae4b96bc1ac17b0c87b7ae70cbb9f939f",
            "qualification_execution_base": "e3452781fe482418466335a61a280ebea39591c5",
            "public_qualification_commit": "0735f7e23524764c889f4ec0cc9942c9e795740a",
        },
        "artifacts": artifact_rows(),
        "counts": {
            "source_sites": 68, "proof_rows": 68, "mutation_rows": 40,
            "helper_matrix_cells": 28, "direct_mutations": 7,
            "provenance_mutations": 5, "mutation_edges": 488,
            "mapped_predecessor_steps": 43, "mutation_survivors": 0,
        },
        "qualification": {
            "repetitions": 2, "conformance_processes": 4,
            "gate_executions": 18, "scenarios": 204, "signed_events": 771,
            "delivery_orders": 8, "byte_identical": True,
            "changed": False,
        },
        "findings": {
            "public_scope_closed": ["FINDING_130", "FINDING_131", "FINDING_132", "FINDING_133"],
            "global_closed": ["FINDING_133"],
            "independent_pending": ["FINDING_130", "FINDING_131", "FINDING_132"],
            "held": ["FINDING_080"],
        },
        "completion": {
            "public_complete": True, "independent_complete": False,
            "combined_complete": False, "terminal_complete": False,
        },
        "holds": HOLDS,
        "remote_actions": 0,
        "result": "pass",
    }


def validate(report: dict[str, Any], schema: dict[str, Any], ledger: dict[str, Any], findings: dict[str, Any]) -> None:
    require(report == expected_report(), "report:contract")
    require(schema.get("additionalProperties") is False, "schema:closed")
    require(schema.get("required") == list(report), "schema:required")
    require(schema["properties"]["schema"]["const"] == report["schema"], "schema:identity")
    for row in report["artifacts"]:
        committed = git("show", f'{row["artifact_commit"]}:{row["path"]}', binary=True)
        require(hashlib.sha256(committed).hexdigest() == row["sha256"], "artifact:sha:" + row["role"])
        require(json.loads(committed)["result"] == "pass", "artifact:result:" + row["role"])
        require(subprocess.run(["git", "merge-base", "--is-ancestor", row["artifact_commit"], "HEAD"], cwd=ROOT).returncode == 0, "artifact:ancestry:" + row["role"])
    cursor = ledger["cursor"]
    completed = cursor["completed_rclds"]
    require(completed in [list(range(141, end + 1)) for end in range(144, 147)], "ledger:cursor:prefix")
    require(cursor["last_planned_rcld"] == 146 and cursor["remaining_rcld_count"] == 6 - len(completed), "ledger:cursor:remaining")
    roles = ledger["candidate_roles"]
    require(roles["public_qualification_commit"] == "0735f7e23524764c889f4ec0cc9942c9e795740a", "ledger:qualification")
    independent_complete = len(completed) >= 5
    require(ledger["independent"]["completed"] is independent_complete, "ledger:independent")
    require((roles["independent_assurance_commit"] is not None) is independent_complete, "ledger:independent:candidate")
    statuses = {row["id"]: row["status"] for row in findings["findings"]}
    expected_local = "closed" if independent_complete else "open"
    require(statuses == {"FINDING_130": expected_local, "FINDING_131": expected_local, "FINDING_132": expected_local, "FINDING_133": "closed", "FINDING_080": "held"}, "findings:status")
    expected_open = [] if independent_complete else ["FINDING_130", "FINDING_131", "FINDING_132"]
    require(ledger["findings"] == {"open": expected_open, "held": ["FINDING_080"]}, "ledger:findings")


def self_test(values: list[dict[str, Any]]) -> int:
    attacks = [
        lambda r, _s, _l, _f: r["counts"].update(mutation_survivors=1),
        lambda r, _s, _l, _f: r["qualification"].update(byte_identical=False),
        lambda r, _s, _l, _f: r["completion"].update(independent_complete=True),
        lambda r, _s, _l, _f: r.update(remote_actions=1),
        lambda r, _s, _l, _f: r["findings"]["held"].clear(),
        lambda _r, s, _l, _f: s.update(additionalProperties=True),
        lambda _r, _s, l, _f: l["cursor"].update(remaining_rcld_count=99),
        lambda _r, _s, _l, f: f["findings"][3].update(status="open"),
    ]
    caught = 0
    for attack in attacks:
        changed = copy.deepcopy(values)
        attack(*changed)
        try:
            validate(*changed)
        except AssuranceError:
            caught += 1
            continue
        raise AssuranceError("self_test:survivor")
    return caught


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--write", action="store_true")
    args = parser.parse_args()
    if args.write:
        REPORT.write_text(json.dumps(expected_report(), indent=2) + "\n")
    values = [load(path) for path in (REPORT, SCHEMA, LEDGER, FINDINGS)]
    validate(*values)
    print(f"PASS: causal projection public assurance v19 artifacts=8 attacks={self_test(values)} private=pending remote_actions=0")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
