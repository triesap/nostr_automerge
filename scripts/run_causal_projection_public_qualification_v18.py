#!/usr/bin/env python3
"""Execute and validate the sealed-boundary v18 public qualification."""

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
REPORT_PATH = "reports/causal_projection_public_qualification_v18.json"
REPORT = ROOT / REPORT_PATH
SCHEMA_PATH = "tools/validation/causal_projection_public_qualification_v18.schema.json"
SCHEMA = ROOT / SCHEMA_PATH
RUNNER_PATH = "scripts/run_causal_projection_public_qualification_v18.py"
GRAPH_PATH = "reports/causal_projection_evidence_graph_v18.json"
GRAPH_CANDIDATE = "6b73727be798e152aa3afbb98bf3683c7e52a393"
GRAPH_SHA256 = "48a82ead9b1baf911638651191e2592df3f6ce259077ffc77642c39d8636a9e5"
TRANSITION_PATH = "spec/distribution_v18_transition.json"
TRANSITION_CANDIDATE = "b1a960ae32aa95c4a978b401af1b46e1cd9a29a0"
TRANSITION_SHA256 = "1408b71c6e7ee31a99e6e0436c4ed290467675a67f517bc0be082b10149a5153"
MANIFEST_PATH = "fixtures/distribution/manifest_v16.json"
CANONICAL_SHA256 = "e69c721549966b1b88dcde3296674d675169840c6e8ebd0f236a5c07bcfc6415"
SERIALIZED_SHA256 = "000c52bde7102eaccec8cf65c875332e119fd25ccf4a2ac38973c456774a3344"
HOLDS = [
    "external_assurance",
    "event_kind_allocation",
    "nip_submission",
    "production_qualification",
    "publication",
    "release",
    "remote_mutation",
]
JOBS = [
    "remediation",
    "policy",
    "standard",
    "conformance",
    "coverage",
    "supply_chain",
    "robustness",
    "resource",
    "release_evidence",
]
CONFORMANCE_COMMAND = [
    "cargo",
    "extbuild",
    "run",
    "--",
    "cargo",
    "run",
    "--quiet",
    "-p",
    "nostr_automerge_conformance",
    "--locked",
    "--",
    "run_distribution",
    MANIFEST_PATH,
]


class QualificationError(RuntimeError):
    pass


def require(condition: bool, code: str) -> None:
    if not condition:
        raise QualificationError(code)


def sha(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def git(*args: str, require_success: bool = True) -> subprocess.CompletedProcess[bytes]:
    completed = subprocess.run(
        ["git", *args], cwd=ROOT, capture_output=True, check=False
    )
    if require_success:
        require(completed.returncode == 0, "GIT:" + ":".join(args))
    return completed


def committed(candidate: str, path: str) -> bytes:
    return git("show", f"{candidate}:{path}").stdout


def environment_record() -> dict[str, Any]:
    return {"mode": "inherited", "overrides": {}}


def execute(command: list[str], label: str) -> tuple[bytes, bytes]:
    print(f"v18 public qualification {label} start", flush=True)
    completed = subprocess.run(command, cwd=ROOT, capture_output=True, check=False)
    if completed.returncode:
        sys.stdout.buffer.write(completed.stdout[-4000:])
        sys.stderr.buffer.write(completed.stderr[-4000:])
        raise QualificationError(f"COMMAND:{label}:{completed.returncode}")
    print(f"v18 public qualification {label} pass", flush=True)
    return completed.stdout, completed.stderr


def conformance_process(ordinal: int) -> tuple[dict[str, Any], bytes]:
    stdout, stderr = execute(CONFORMANCE_COMMAND, f"conformance-process-{ordinal}")
    try:
        summary = json.loads(stdout)
    except json.JSONDecodeError as error:
        raise QualificationError(f"CONFORMANCE_JSON:{ordinal}") from error
    require(summary.get("status") == "pass", f"CONFORMANCE_STATUS:{ordinal}")
    require(summary.get("fixture_count") == 204, f"CONFORMANCE_FIXTURES:{ordinal}")
    require(summary.get("delivery_permutations") == 8, f"CONFORMANCE_ORDERS:{ordinal}")
    require(len(summary.get("reports", [])) == 204, f"CONFORMANCE_REPORTS:{ordinal}")
    require(summary.get("canonical_output_sha256") == CANONICAL_SHA256, f"CONFORMANCE_CANONICAL:{ordinal}")
    require(sha(stdout) == SERIALIZED_SHA256, f"CONFORMANCE_SERIALIZED:{ordinal}")
    return (
        {
            "ordinal": ordinal,
            "argv": CONFORMANCE_COMMAND,
            "cwd": ".",
            "environment": environment_record(),
            "exit_status": 0,
            "stdout_sha256": sha(stdout),
            "stderr_sha256": sha(stderr),
            "fixture_count": 204,
            "delivery_permutations": 8,
            "report_count": 204,
            "canonical_output_sha256": CANONICAL_SHA256,
            "result": "pass",
        },
        stdout,
    )


def gate_record(job: str) -> dict[str, Any]:
    command = ["cargo", "extbuild", "run", "--", "python3", "scripts/local_gate.py", job]
    stdout, stderr = execute(command, f"gate-{job}")
    return {
        "job": job,
        "argv": command,
        "cwd": ".",
        "environment": environment_record(),
        "exit_status": 0,
        "stdout_sha256": sha(stdout),
        "stderr_sha256": sha(stderr),
        "output_sha256": sha(stdout + b"\x00stderr\x00" + stderr),
        "result": "pass",
    }


def execution_base() -> dict[str, str]:
    candidate = git("rev-parse", "HEAD").stdout.decode().strip()
    tree = git("rev-parse", "HEAD^{tree}").stdout.decode().strip()
    return {
        "candidate": candidate,
        "tree": tree,
        "runner_sha256": sha(committed(candidate, RUNNER_PATH)),
        "schema_sha256": sha(committed(candidate, SCHEMA_PATH)),
        "report_absent": "true",
    }


def create_report() -> dict[str, Any]:
    require(git("status", "--porcelain=v1", "--untracked-files=all").stdout == b"", "DIRTY_EXECUTION_BASE")
    base = execution_base()
    require(
        git("cat-file", "-e", f"{base['candidate']}:{REPORT_PATH}", require_success=False).returncode != 0,
        "REPORT_PRESENT_AT_EXECUTION_BASE",
    )
    first, first_stdout = conformance_process(1)
    second, second_stdout = conformance_process(2)
    require(first_stdout == second_stdout, "CONFORMANCE_PROCESS_MISMATCH")
    gates = [gate_record(job) for job in JOBS]
    return {
        "schema": "nostr_automerge.causal_projection_public_qualification.v18.v1",
        "status": "final",
        "execution_mode": "clean_committed_base",
        "evidence_graph": {
            "path": GRAPH_PATH,
            "candidate": GRAPH_CANDIDATE,
            "sha256": GRAPH_SHA256,
        },
        "distribution_transition": {
            "path": TRANSITION_PATH,
            "candidate": TRANSITION_CANDIDATE,
            "sha256": TRANSITION_SHA256,
        },
        "execution_base": base,
        "conformance": {
            "process_count": 2,
            "byte_identical": True,
            "serialized_run_sha256": SERIALIZED_SHA256,
            "processes": [first, second],
        },
        "gates": gates,
        "frozen": {
            "requirements": 156,
            "scenarios": 204,
            "signed_events": 771,
            "delivery_orders": 8,
            "canonical_output_sha256": CANONICAL_SHA256,
            "serialized_run_sha256": SERIALIZED_SHA256,
            "changed": False,
        },
        "holds": HOLDS,
        "remote_actions": 0,
        "result": "pass",
    }


def validate(report: Any, schema: Any) -> None:
    require(type(report) is dict, "REPORT_TYPE")
    require(report["schema"] == "nostr_automerge.causal_projection_public_qualification.v18.v1", "SCHEMA")
    require(report["status"] == "final" and report["execution_mode"] == "clean_committed_base", "STATUS")
    require(report["evidence_graph"] == {"path": GRAPH_PATH, "candidate": GRAPH_CANDIDATE, "sha256": GRAPH_SHA256}, "GRAPH_BINDING")
    require(sha(committed(GRAPH_CANDIDATE, GRAPH_PATH)) == GRAPH_SHA256, "GRAPH_COMMITTED")
    require(report["distribution_transition"] == {"path": TRANSITION_PATH, "candidate": TRANSITION_CANDIDATE, "sha256": TRANSITION_SHA256}, "TRANSITION_BINDING")
    require(sha(committed(TRANSITION_CANDIDATE, TRANSITION_PATH)) == TRANSITION_SHA256, "TRANSITION_COMMITTED")
    base = report["execution_base"]
    require(re.fullmatch(r"[0-9a-f]{40}", base["candidate"]) is not None, "EXECUTION_BASE_CANDIDATE")
    require(git("merge-base", "--is-ancestor", base["candidate"], "HEAD", require_success=False).returncode == 0, "EXECUTION_BASE_ANCESTRY")
    require(base["tree"] == git("rev-parse", f"{base['candidate']}^{{tree}}").stdout.decode().strip(), "EXECUTION_BASE_TREE")
    require(base["runner_sha256"] == sha(committed(base["candidate"], RUNNER_PATH)) == sha((ROOT / RUNNER_PATH).read_bytes()), "EXECUTION_BASE_RUNNER")
    require(base["schema_sha256"] == sha(committed(base["candidate"], SCHEMA_PATH)) == sha(SCHEMA.read_bytes()), "EXECUTION_BASE_SCHEMA")
    require(base["report_absent"] == "true", "EXECUTION_BASE_REPORT_ABSENT")
    require(git("cat-file", "-e", f"{base['candidate']}:{REPORT_PATH}", require_success=False).returncode != 0, "REPORT_CYCLE")
    conformance = report["conformance"]
    require(conformance["process_count"] == 2 and conformance["byte_identical"] is True, "CONFORMANCE_COUNT")
    require(conformance["serialized_run_sha256"] == SERIALIZED_SHA256, "CONFORMANCE_IDENTITY")
    require([row["ordinal"] for row in conformance["processes"]] == [1, 2], "CONFORMANCE_ORDINALS")
    for row in conformance["processes"]:
        require(row["argv"] == CONFORMANCE_COMMAND and row["cwd"] == ".", "CONFORMANCE_COMMAND")
        require(row["environment"] == environment_record(), "CONFORMANCE_ENVIRONMENT")
        require(row["exit_status"] == 0 and row["result"] == "pass", "CONFORMANCE_RESULT")
        require(row["stdout_sha256"] == SERIALIZED_SHA256, "CONFORMANCE_STDOUT")
        require(row["fixture_count"] == row["report_count"] == 204, "CONFORMANCE_SCENARIOS")
        require(row["delivery_permutations"] == 8 and row["canonical_output_sha256"] == CANONICAL_SHA256, "CONFORMANCE_OUTPUT")
    require([row["job"] for row in report["gates"]] == JOBS, "GATE_ORDER")
    for job, row in zip(JOBS, report["gates"], strict=True):
        require(row["argv"] == ["cargo", "extbuild", "run", "--", "python3", "scripts/local_gate.py", job], "GATE_COMMAND:" + job)
        require(row["cwd"] == "." and row["environment"] == environment_record(), "GATE_CONTEXT:" + job)
        require(row["exit_status"] == 0 and row["result"] == "pass", "GATE_RESULT:" + job)
        require(all(re.fullmatch(r"[0-9a-f]{64}", row[field]) for field in ("stdout_sha256", "stderr_sha256", "output_sha256")), "GATE_HASH:" + job)
    require(report["frozen"] == {"requirements": 156, "scenarios": 204, "signed_events": 771, "delivery_orders": 8, "canonical_output_sha256": CANONICAL_SHA256, "serialized_run_sha256": SERIALIZED_SHA256, "changed": False}, "FROZEN")
    require(report["holds"] == HOLDS and report["remote_actions"] == 0, "AUTHORITY")
    require(report["result"] == "pass", "RESULT")
    require(type(schema) is dict and schema.get("additionalProperties") is False and schema.get("required") == list(report), "SCHEMA_CLOSED")


def self_test(report: dict[str, Any], schema: dict[str, Any]) -> int:
    attacks = [
        lambda value: value["execution_base"].update(report_absent="false"),
        lambda value: value["conformance"].update(byte_identical=False),
        lambda value: value["conformance"]["processes"][1].update(stdout_sha256="0" * 64),
        lambda value: value["gates"].pop(),
        lambda value: value["gates"][0].update(exit_status=1),
        lambda value: value["frozen"].update(changed=True),
        lambda value: value.update(remote_actions=1),
        lambda value: value["holds"].remove("publication"),
    ]
    caught = 0
    for attack in attacks:
        changed = copy.deepcopy(report)
        attack(changed)
        try:
            validate(changed, schema)
        except QualificationError:
            caught += 1
            continue
        raise QualificationError("ATTACK_SURVIVED")
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
    print(
        "PASS: causal projection public qualification v18 "
        f"processes=2 gates=9 attacks={self_test(report, schema)} remote_actions=0"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
