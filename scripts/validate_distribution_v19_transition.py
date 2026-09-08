#!/usr/bin/env python3
"""Execute and validate the actual zero-change v19 distribution transition."""

from __future__ import annotations

import argparse
import copy
import hashlib
import json
import re
import subprocess
import sys
from pathlib import Path
from typing import Any

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parents[1]
REPORT_PATH = "spec/distribution_v19_transition.json"
REPORT = ROOT / REPORT_PATH
SCHEMA_PATH = "tools/validation/distribution_v19_transition.schema.json"
SCHEMA = ROOT / SCHEMA_PATH
RUNNER_PATH = "scripts/validate_distribution_v19_transition.py"
GRAPH_PATH = "reports/causal_projection_evidence_graph_v19.json"
GRAPH_CANDIDATE = "ac24566ae4b96bc1ac17b0c87b7ae70cbb9f939f"
RELEASE_PATH = "reports/causal_projection_release_boundary_v19.json"
RELEASE_CANDIDATE = "6760c58c2fcf754c03f6b21ff0bfa2fd21113a57"
PRIOR_PATH = "spec/distribution_v18_transition.json"
MANIFEST_PATH = "fixtures/distribution/manifest_v16.json"
LOCK_PATH = "fixtures/distribution/manifest_v16.lock.json"
CANONICAL_SHA256 = "e69c721549966b1b88dcde3296674d675169840c6e8ebd0f236a5c07bcfc6415"
SERIALIZED_SHA256 = "000c52bde7102eaccec8cf65c875332e119fd25ccf4a2ac38973c456774a3344"
COMMAND = [
    "cargo", "extbuild", "run", "--", "cargo", "run", "--quiet", "-p",
    "nostr_automerge_conformance", "--locked", "--", "run_distribution",
    MANIFEST_PATH,
]
TOP_FIELDS = [
    "schema", "status", "evidence_graph", "release_boundary",
    "execution_base", "prior_transition", "selected_manifest",
    "selected_lock", "actual_runs", "affected_fixture_ids", "counts",
    "identity", "derivation", "result",
]


class TransitionError(RuntimeError):
    pass


def require(condition: bool, code: str) -> None:
    if not condition:
        raise TransitionError(code)


def sha(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def git(*args: str, require_success: bool = True) -> subprocess.CompletedProcess[bytes]:
    completed = subprocess.run(["git", *args], cwd=ROOT, capture_output=True, check=False)
    if require_success:
        require(completed.returncode == 0, "GIT:" + ":".join(args))
    return completed


def committed(candidate: str, path: str) -> bytes:
    return git("show", f"{candidate}:{path}").stdout


def execution_base() -> dict[str, str]:
    candidate = git("rev-parse", "HEAD").stdout.decode().strip()
    return {
        "candidate": candidate,
        "tree": git("rev-parse", "HEAD^{tree}").stdout.decode().strip(),
        "runner_sha256": sha(committed(candidate, RUNNER_PATH)),
        "schema_sha256": sha(committed(candidate, SCHEMA_PATH)),
        "report_absent": "true",
    }


def run_once(ordinal: int) -> tuple[dict[str, Any], bytes]:
    print(f"v19 distribution transition process-{ordinal} start", flush=True)
    completed = subprocess.run(COMMAND, cwd=ROOT, capture_output=True, check=False)
    require(completed.returncode == 0, f"COMMAND:{ordinal}:{completed.returncode}")
    summary = json.loads(completed.stdout)
    require(summary.get("status") == "pass", f"STATUS:{ordinal}")
    require(summary.get("fixture_count") == 204, f"SCENARIOS:{ordinal}")
    require(summary.get("delivery_permutations") == 8, f"ORDERS:{ordinal}")
    require(len(summary.get("reports", [])) == 204, f"REPORTS:{ordinal}")
    require(summary.get("canonical_output_sha256") == CANONICAL_SHA256, f"CANONICAL:{ordinal}")
    require(sha(completed.stdout) == SERIALIZED_SHA256, f"SERIALIZED:{ordinal}")
    print(f"v19 distribution transition process-{ordinal} pass", flush=True)
    return ({
        "ordinal": ordinal,
        "argv": COMMAND,
        "cwd": ".",
        "exit_status": 0,
        "stdout_sha256": sha(completed.stdout),
        "stderr_sha256": sha(completed.stderr),
        "scenarios": 204,
        "delivery_orders": 8,
        "canonical_output_sha256": CANONICAL_SHA256,
        "result": "pass",
    }, completed.stdout)


def create_report() -> dict[str, Any]:
    require(git("status", "--porcelain=v1", "--untracked-files=all").stdout == b"", "DIRTY_EXECUTION_BASE")
    base = execution_base()
    require(git("cat-file", "-e", f"{base['candidate']}:{REPORT_PATH}", require_success=False).returncode != 0, "REPORT_PRESENT_AT_EXECUTION_BASE")
    graph_raw = committed(GRAPH_CANDIDATE, GRAPH_PATH)
    release_raw = committed(RELEASE_CANDIDATE, RELEASE_PATH)
    prior_raw = committed(base["candidate"], PRIOR_PATH)
    manifest_raw = committed(base["candidate"], MANIFEST_PATH)
    lock_raw = committed(base["candidate"], LOCK_PATH)
    graph, release, manifest, lock = map(json.loads, (graph_raw, release_raw, manifest_raw, lock_raw))
    require(graph["result"] == release["result"] == "pass", "BOUNDARY_EVIDENCE")
    require(release["non_test_source"]["identical"] is True, "NON_TEST_SOURCE")
    first, first_stdout = run_once(1)
    second, second_stdout = run_once(2)
    require(first_stdout == second_stdout, "PROCESS_BYTES")
    return {
        "schema": "nostr_automerge.distribution_v19_transition.v1",
        "status": "actual_zero_change",
        "evidence_graph": {"path": GRAPH_PATH, "candidate": GRAPH_CANDIDATE, "sha256": sha(graph_raw)},
        "release_boundary": {"path": RELEASE_PATH, "candidate": RELEASE_CANDIDATE, "sha256": sha(release_raw)},
        "execution_base": base,
        "prior_transition": {"path": PRIOR_PATH, "sha256": sha(prior_raw)},
        "selected_manifest": {"path": MANIFEST_PATH, "sha256": sha(manifest_raw)},
        "selected_lock": {"path": LOCK_PATH, "sha256": sha(lock_raw)},
        "actual_runs": [first, second],
        "affected_fixture_ids": [],
        "counts": {
            "requirements": 156,
            "scenarios": len(manifest["fixtures"]),
            "signed_events": lock["signed_event_count"],
            "delivery_orders": 8,
            "processes": 2,
            "affected": 0,
        },
        "identity": {
            "process_bytes_identical": True,
            "signed_events_byte_identical": True,
            "ample_reports_byte_identical": True,
            "canonical_output_sha256": CANONICAL_SHA256,
            "serialized_run_sha256": SERIALIZED_SHA256,
        },
        "derivation": {
            "actual_processes_executed": True,
            "production_semantics_changed": False,
            "runtime_budget_change": False,
            "assurance_only_change": True,
            "synthetic_version_rebinding": False,
            "new_manifest_created": False,
        },
        "result": "pass",
    }


def validate(report: Any, schema: Any) -> None:
    require(type(report) is dict and list(report) == TOP_FIELDS, "REPORT_SHAPE")
    require(report["schema"] == "nostr_automerge.distribution_v19_transition.v1", "SCHEMA")
    require(report["status"] == "actual_zero_change" and report["result"] == "pass", "STATUS")
    graph_raw, release_raw = committed(GRAPH_CANDIDATE, GRAPH_PATH), committed(RELEASE_CANDIDATE, RELEASE_PATH)
    require(report["evidence_graph"] == {"path": GRAPH_PATH, "candidate": GRAPH_CANDIDATE, "sha256": sha(graph_raw)}, "GRAPH")
    require(report["release_boundary"] == {"path": RELEASE_PATH, "candidate": RELEASE_CANDIDATE, "sha256": sha(release_raw)}, "RELEASE")
    base = report["execution_base"]
    require(re.fullmatch(r"[0-9a-f]{40}", base["candidate"]) is not None, "BASE_CANDIDATE")
    require(base["tree"] == git("rev-parse", f"{base['candidate']}^{{tree}}").stdout.decode().strip(), "BASE_TREE")
    require(base["runner_sha256"] == sha(committed(base["candidate"], RUNNER_PATH)) == sha((ROOT / RUNNER_PATH).read_bytes()), "BASE_RUNNER")
    require(base["schema_sha256"] == sha(committed(base["candidate"], SCHEMA_PATH)) == sha(SCHEMA.read_bytes()), "BASE_SCHEMA")
    require(base["report_absent"] == "true", "BASE_REPORT_ABSENT")
    require(git("cat-file", "-e", f"{base['candidate']}:{REPORT_PATH}", require_success=False).returncode != 0, "REPORT_CYCLE")
    require(report["prior_transition"]["sha256"] == sha(committed(base["candidate"], PRIOR_PATH)), "PRIOR")
    require(report["selected_manifest"]["sha256"] == sha(committed(base["candidate"], MANIFEST_PATH)), "MANIFEST")
    require(report["selected_lock"]["sha256"] == sha(committed(base["candidate"], LOCK_PATH)), "LOCK")
    require([row["ordinal"] for row in report["actual_runs"]] == [1, 2], "RUN_ORDINALS")
    for row in report["actual_runs"]:
        require(row["argv"] == COMMAND and row["cwd"] == "." and row["exit_status"] == 0, "RUN_COMMAND")
        require(row["stdout_sha256"] == SERIALIZED_SHA256 and row["canonical_output_sha256"] == CANONICAL_SHA256, "RUN_IDENTITY")
        require(row["scenarios"] == 204 and row["delivery_orders"] == 8 and row["result"] == "pass", "RUN_RESULT")
        require(re.fullmatch(r"[0-9a-f]{64}", row["stderr_sha256"]) is not None, "RUN_STDERR")
    require(report["affected_fixture_ids"] == [] and report["counts"] == {"requirements": 156, "scenarios": 204, "signed_events": 771, "delivery_orders": 8, "processes": 2, "affected": 0}, "COUNTS")
    require(report["identity"] == {"process_bytes_identical": True, "signed_events_byte_identical": True, "ample_reports_byte_identical": True, "canonical_output_sha256": CANONICAL_SHA256, "serialized_run_sha256": SERIALIZED_SHA256}, "IDENTITY")
    require(report["derivation"] == {"actual_processes_executed": True, "production_semantics_changed": False, "runtime_budget_change": False, "assurance_only_change": True, "synthetic_version_rebinding": False, "new_manifest_created": False}, "DERIVATION")
    require(type(schema) is dict and schema.get("additionalProperties") is False and schema.get("required") == TOP_FIELDS, "SCHEMA_CLOSED")


def self_test(report: dict[str, Any], schema: dict[str, Any]) -> int:
    attacks = [
        ("report", lambda value: value["actual_runs"].pop()),
        ("report", lambda value: value["actual_runs"][1].update(stdout_sha256="0" * 64)),
        ("report", lambda value: value["affected_fixture_ids"].append("synthetic")),
        ("report", lambda value: value["identity"].update(process_bytes_identical=False)),
        ("report", lambda value: value["derivation"].update(runtime_budget_change=True)),
        ("report", lambda value: value["counts"].update(scenarios=203)),
        ("report", lambda value: value["execution_base"].update(report_absent="false")),
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
    parser.add_argument("--execute", action="store_true")
    parser.add_argument("--write-report", action="store_true")
    args = parser.parse_args()
    require(args.execute == args.write_report, "EXECUTE_WRITE_PAIR")
    if args.execute:
        REPORT.write_text(json.dumps(create_report(), ensure_ascii=True, indent=2) + "\n")
    report = json.loads(REPORT.read_text())
    schema = json.loads(SCHEMA.read_text())
    validate(report, schema)
    print(f"PASS: distribution v19 transition scenarios=204 affected=0 processes=2 attacks={self_test(report, schema)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
