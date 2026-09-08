#!/usr/bin/env python3
"""Execute and validate two complete v19 public qualification repetitions."""

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
REPORT_PATH = "reports/causal_projection_public_qualification_v19.json"
REPORT = ROOT / REPORT_PATH
SCHEMA_PATH = "tools/validation/causal_projection_public_qualification_v19.schema.json"
SCHEMA = ROOT / SCHEMA_PATH
RUNNER_PATH = "scripts/run_causal_projection_public_qualification_v19.py"
GRAPH_PATH = "reports/causal_projection_evidence_graph_v19.json"
GRAPH_CANDIDATE = "ac24566ae4b96bc1ac17b0c87b7ae70cbb9f939f"
TRANSITION_PATH = "spec/distribution_v19_transition.json"
MANIFEST_PATH = "fixtures/distribution/manifest_v16.json"
CANONICAL_SHA256 = "e69c721549966b1b88dcde3296674d675169840c6e8ebd0f236a5c07bcfc6415"
SERIALIZED_SHA256 = "000c52bde7102eaccec8cf65c875332e119fd25ccf4a2ac38973c456774a3344"
HOLDS = [
    "external_assurance", "event_kind_allocation", "nip_submission",
    "production_qualification", "publication", "release", "deployment",
    "remote_mutation",
]
JOBS = [
    "remediation", "policy", "standard", "conformance", "coverage",
    "supply_chain", "robustness", "resource", "release_evidence",
]
CONFORMANCE_COMMAND = [
    "cargo", "extbuild", "run", "--", "cargo", "run", "--quiet", "-p",
    "nostr_automerge_conformance", "--locked", "--", "run_distribution",
    MANIFEST_PATH,
]
TOP_FIELDS = [
    "schema", "status", "execution_mode", "evidence_graph",
    "distribution_transition", "execution_base", "repetitions", "aggregate",
    "frozen", "holds", "remote_actions", "result",
]


class QualificationError(RuntimeError):
    pass


def require(condition: bool, code: str) -> None:
    if not condition:
        raise QualificationError(code)


def sha(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def git(*args: str, require_success: bool = True) -> subprocess.CompletedProcess[bytes]:
    completed = subprocess.run(["git", *args], cwd=ROOT, capture_output=True, check=False)
    if require_success:
        require(completed.returncode == 0, "GIT:" + ":".join(args))
    return completed


def committed(candidate: str, path: str) -> bytes:
    return git("show", f"{candidate}:{path}").stdout


def environment_record() -> dict[str, Any]:
    return {"mode": "inherited", "overrides": {}}


def execute(command: list[str], label: str) -> tuple[bytes, bytes]:
    print(f"v19 public qualification {label} start", flush=True)
    completed = subprocess.run(command, cwd=ROOT, capture_output=True, check=False)
    if completed.returncode:
        sys.stdout.buffer.write(completed.stdout[-4000:])
        sys.stderr.buffer.write(completed.stderr[-4000:])
        raise QualificationError(f"COMMAND:{label}:{completed.returncode}")
    print(f"v19 public qualification {label} pass", flush=True)
    return completed.stdout, completed.stderr


def conformance_process(repetition: int, ordinal: int) -> tuple[dict[str, Any], bytes]:
    stdout, stderr = execute(CONFORMANCE_COMMAND, f"repetition-{repetition}-conformance-process-{ordinal}")
    try:
        summary = json.loads(stdout)
    except json.JSONDecodeError as error:
        raise QualificationError(f"CONFORMANCE_JSON:{repetition}:{ordinal}") from error
    require(summary.get("status") == "pass", f"CONFORMANCE_STATUS:{repetition}:{ordinal}")
    require(summary.get("fixture_count") == 204, f"CONFORMANCE_FIXTURES:{repetition}:{ordinal}")
    require(summary.get("delivery_permutations") == 8, f"CONFORMANCE_ORDERS:{repetition}:{ordinal}")
    require(len(summary.get("reports", [])) == 204, f"CONFORMANCE_REPORTS:{repetition}:{ordinal}")
    require(summary.get("canonical_output_sha256") == CANONICAL_SHA256, f"CONFORMANCE_CANONICAL:{repetition}:{ordinal}")
    require(sha(stdout) == SERIALIZED_SHA256, f"CONFORMANCE_SERIALIZED:{repetition}:{ordinal}")
    return ({
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
    }, stdout)


def gate_record(repetition: int, job: str) -> dict[str, Any]:
    command = ["cargo", "extbuild", "run", "--", "python3", "scripts/local_gate.py", job]
    stdout, stderr = execute(command, f"repetition-{repetition}-gate-{job}")
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
    return {
        "candidate": candidate,
        "tree": git("rev-parse", "HEAD^{tree}").stdout.decode().strip(),
        "runner_sha256": sha(committed(candidate, RUNNER_PATH)),
        "schema_sha256": sha(committed(candidate, SCHEMA_PATH)),
        "report_absent": "true",
    }


def qualification_repetition(ordinal: int) -> tuple[dict[str, Any], list[bytes]]:
    first, first_stdout = conformance_process(ordinal, 1)
    second, second_stdout = conformance_process(ordinal, 2)
    require(first_stdout == second_stdout, f"CONFORMANCE_PROCESS_MISMATCH:{ordinal}")
    gates = [gate_record(ordinal, job) for job in JOBS]
    return ({
        "ordinal": ordinal,
        "conformance": {
            "process_count": 2,
            "byte_identical": True,
            "serialized_run_sha256": SERIALIZED_SHA256,
            "processes": [first, second],
        },
        "gates": gates,
        "result": "pass",
    }, [first_stdout, second_stdout])


def create_report() -> dict[str, Any]:
    require(git("status", "--porcelain=v1", "--untracked-files=all").stdout == b"", "DIRTY_EXECUTION_BASE")
    base = execution_base()
    require(git("cat-file", "-e", f"{base['candidate']}:{REPORT_PATH}", require_success=False).returncode != 0, "REPORT_PRESENT_AT_EXECUTION_BASE")
    graph_raw = committed(GRAPH_CANDIDATE, GRAPH_PATH)
    transition_raw = committed(base["candidate"], TRANSITION_PATH)
    transition = json.loads(transition_raw)
    require(transition["status"] == "actual_zero_change" and transition["result"] == "pass", "TRANSITION")
    first, first_outputs = qualification_repetition(1)
    second, second_outputs = qualification_repetition(2)
    require(len(set(first_outputs + second_outputs)) == 1, "CROSS_REPETITION_CONFORMANCE")
    return {
        "schema": "nostr_automerge.causal_projection_public_qualification.v19.v1",
        "status": "final",
        "execution_mode": "two_clean_repetitions",
        "evidence_graph": {"path": GRAPH_PATH, "candidate": GRAPH_CANDIDATE, "sha256": sha(graph_raw)},
        "distribution_transition": {"path": TRANSITION_PATH, "candidate": base["candidate"], "sha256": sha(transition_raw)},
        "execution_base": base,
        "repetitions": [first, second],
        "aggregate": {
            "repetition_count": 2,
            "conformance_processes": 4,
            "gate_executions": 18,
            "cross_repetition_conformance_byte_identical": True,
        },
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
    require(type(report) is dict and list(report) == TOP_FIELDS, "REPORT_SHAPE")
    require(report["schema"] == "nostr_automerge.causal_projection_public_qualification.v19.v1", "SCHEMA")
    require(report["status"] == "final" and report["execution_mode"] == "two_clean_repetitions", "STATUS")
    graph_raw = committed(GRAPH_CANDIDATE, GRAPH_PATH)
    require(report["evidence_graph"] == {"path": GRAPH_PATH, "candidate": GRAPH_CANDIDATE, "sha256": sha(graph_raw)}, "GRAPH_BINDING")
    base = report["execution_base"]
    require(re.fullmatch(r"[0-9a-f]{40}", base["candidate"]) is not None, "EXECUTION_BASE_CANDIDATE")
    require(git("merge-base", "--is-ancestor", base["candidate"], "HEAD", require_success=False).returncode == 0, "EXECUTION_BASE_ANCESTRY")
    require(base["tree"] == git("rev-parse", f"{base['candidate']}^{{tree}}").stdout.decode().strip(), "EXECUTION_BASE_TREE")
    require(base["runner_sha256"] == sha(committed(base["candidate"], RUNNER_PATH)) == sha((ROOT / RUNNER_PATH).read_bytes()), "EXECUTION_BASE_RUNNER")
    require(base["schema_sha256"] == sha(committed(base["candidate"], SCHEMA_PATH)) == sha(SCHEMA.read_bytes()), "EXECUTION_BASE_SCHEMA")
    require(base["report_absent"] == "true", "EXECUTION_BASE_REPORT_ABSENT")
    require(git("cat-file", "-e", f"{base['candidate']}:{REPORT_PATH}", require_success=False).returncode != 0, "REPORT_CYCLE")
    transition_raw = committed(base["candidate"], TRANSITION_PATH)
    require(report["distribution_transition"] == {"path": TRANSITION_PATH, "candidate": base["candidate"], "sha256": sha(transition_raw)}, "TRANSITION_BINDING")
    require([row["ordinal"] for row in report["repetitions"]] == [1, 2], "REPETITION_ORDINALS")
    for repetition in report["repetitions"]:
        conformance = repetition["conformance"]
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
        require([row["job"] for row in repetition["gates"]] == JOBS, "GATE_ORDER")
        for job, row in zip(JOBS, repetition["gates"], strict=True):
            require(row["argv"] == ["cargo", "extbuild", "run", "--", "python3", "scripts/local_gate.py", job], "GATE_COMMAND:" + job)
            require(row["cwd"] == "." and row["environment"] == environment_record(), "GATE_CONTEXT:" + job)
            require(row["exit_status"] == 0 and row["result"] == "pass", "GATE_RESULT:" + job)
            require(all(re.fullmatch(r"[0-9a-f]{64}", row[field]) for field in ("stdout_sha256", "stderr_sha256", "output_sha256")), "GATE_HASH:" + job)
        require(repetition["result"] == "pass", "REPETITION_RESULT")
    require(report["aggregate"] == {"repetition_count": 2, "conformance_processes": 4, "gate_executions": 18, "cross_repetition_conformance_byte_identical": True}, "AGGREGATE")
    require(report["frozen"] == {"requirements": 156, "scenarios": 204, "signed_events": 771, "delivery_orders": 8, "canonical_output_sha256": CANONICAL_SHA256, "serialized_run_sha256": SERIALIZED_SHA256, "changed": False}, "FROZEN")
    require(report["holds"] == HOLDS and report["remote_actions"] == 0, "AUTHORITY")
    require(report["result"] == "pass", "RESULT")
    require(type(schema) is dict and schema.get("additionalProperties") is False and schema.get("required") == TOP_FIELDS, "SCHEMA_CLOSED")


def self_test(report: dict[str, Any], schema: dict[str, Any]) -> int:
    attacks = [
        ("report", lambda value: value["repetitions"].pop()),
        ("report", lambda value: value["repetitions"][0]["conformance"].update(byte_identical=False)),
        ("report", lambda value: value["repetitions"][1]["conformance"]["processes"][1].update(stdout_sha256="0" * 64)),
        ("report", lambda value: value["repetitions"][0]["gates"].pop()),
        ("report", lambda value: value["repetitions"][1]["gates"][0].update(exit_status=1)),
        ("report", lambda value: value["aggregate"].update(gate_executions=17)),
        ("report", lambda value: value["frozen"].update(changed=True)),
        ("report", lambda value: value.update(remote_actions=1)),
        ("report", lambda value: value["holds"].remove("publication")),
        ("schema", lambda value: value.update(additionalProperties=True)),
    ]
    caught = 0
    for target, attack in attacks:
        changed_report, changed_schema = copy.deepcopy(report), copy.deepcopy(schema)
        attack(changed_report if target == "report" else changed_schema)
        try:
            validate(changed_report, changed_schema)
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
    print("PASS: causal projection public qualification v19 repetitions=2 processes=4 gates=18 " f"attacks={self_test(report, schema)} remote_actions=0")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
