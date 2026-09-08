#!/usr/bin/env python3
"""Execute and validate one independent runtime proof per v19 Rust site."""

from __future__ import annotations

import argparse
import copy
import hashlib
import json
import re
import shlex
import subprocess
import sys
from pathlib import Path
from typing import Any

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parents[1]
INVENTORY_PATH = "reports/causal_projection_inventory_v19.json"
INVENTORY = ROOT / INVENTORY_PATH
REPORT = ROOT / "reports/causal_projection_proofs_v19.json"
SCHEMA = ROOT / "tools/validation/causal_projection_proofs_v19.schema.json"
ARTIFACT_DIR = ROOT / "reports/evidence/v19/proofs"
AUTHORITY = "spec/causal_projection_contracts_v19.json"
RUNNER_PATH = "scripts/validate_causal_projection_proof_events_v19.py"
FAILED_EVENTS = ["ChargeAttempt"]
SUCCESS_EVENTS = [
    "ChargeAttempt", "ChargeAccepted", "TargetDispatched",
    "TargetReturned", "CompletionObserved",
]
TRACE_FIELDS = [
    "site_id", "phase", "family", "counter", "requested_site",
    "n_minus_one_result", "n_minus_one_events", "n_result", "n_events",
    "n_plus_one_result", "n_plus_one_events", "cancelled_result",
    "cancelled_events", "unexpected_error_identity", "unexpected_events",
    "count_scope", "event_counts_derived",
]
ARTIFACT_FIELDS = [
    "schema", "authority", "inventory_row_id", "source_candidate",
    "execution_base_candidate", "command", "cwd", "environment",
    "exit_status", "test_result", "stdout", "stderr", "raw_stdout_sha256",
    "raw_stderr_sha256", "trace", "result",
]
ROW_FIELDS = [
    "proof_row_id", "inventory_row_id", "site_id", "phase", "family",
    "counter", "command", "trace_artifact", "trace_sha256", "stdout_sha256",
    "stderr_sha256", "raw_stdout_sha256", "raw_stderr_sha256",
    "source_candidate", "execution_base_candidate", "result",
]
TOP_FIELDS = [
    "schema", "status", "authority", "source_candidate",
    "execution_base_candidate", "inventory_path", "inventory_sha256",
    "row_contract", "rows", "counts", "execution", "result_identity_sha256",
    "result",
]


class ProofError(RuntimeError):
    pass


def require(condition: bool, code: str) -> None:
    if not condition:
        raise ProofError(code)


def canonical(value: object) -> bytes:
    return json.dumps(value, ensure_ascii=True, separators=(",", ":"), sort_keys=True).encode()


def sha(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def run(command: list[str]) -> subprocess.CompletedProcess[str]:
    return subprocess.run(command, cwd=ROOT, capture_output=True, text=True, check=False)


def public_transcript(output: str) -> str:
    """Redact workstation paths while preserving complete diagnostic text."""

    return re.sub(r"/(?:Users|Volumes)/[^\s)]+", "<local-path>", output)


def git(*args: str) -> str:
    completed = run(["git", *args])
    require(completed.returncode == 0, "GIT:" + ":".join(args))
    return completed.stdout.strip()


def load(path: Path) -> Any:
    def closed(pairs: list[tuple[str, Any]]) -> dict[str, Any]:
        keys = [key for key, _ in pairs]
        require(len(keys) == len(set(keys)), "DUPLICATE:" + path.name)
        return dict(pairs)

    return json.loads(path.read_text(), object_pairs_hook=closed)


def validate_success_events(events: Any, code: str) -> None:
    require(type(events) is list, code + ":TYPE")
    require(events[:5] == SUCCESS_EVENTS, code + ":ORDER")
    require(all(event == "PublicationCompleted" for event in events[5:]), code + ":TAIL")
    require(all(events.count(event) == 1 for event in SUCCESS_EVENTS), code + ":COUNT")


def validate_trace(trace: Any, row: dict[str, Any], code: str) -> None:
    require(type(trace) is dict and list(trace) == TRACE_FIELDS, code + ":SHAPE")
    require(trace["site_id"] == trace["requested_site"] == row["site_id"], code + ":SITE")
    require(trace["phase"] == row["phase"] and trace["family"] == row["family"], code + ":OWNER")
    require(trace["counter"].casefold() == row["counter"].replace("_", ""), code + ":COUNTER")
    require(trace["n_minus_one_result"] == "BudgetExhausted", code + ":BUDGET")
    require(trace["cancelled_result"] == "Cancelled", code + ":CANCELLED")
    require(trace["unexpected_error_identity"] == "exact", code + ":UNEXPECTED")
    require(trace["n_result"] == trace["n_plus_one_result"] == "requested_site_completed", code + ":SUCCESS")
    require(trace["n_minus_one_events"] == FAILED_EVENTS, code + ":BUDGET_EVENTS")
    require(trace["cancelled_events"] == FAILED_EVENTS, code + ":CANCELLED_EVENTS")
    require(trace["unexpected_events"] == FAILED_EVENTS, code + ":UNEXPECTED_EVENTS")
    validate_success_events(trace["n_events"], code + ":N")
    validate_success_events(trace["n_plus_one_events"], code + ":N_PLUS_ONE")
    require(trace["count_scope"] == "requested_operation_window", code + ":SCOPE")
    require(trace["event_counts_derived"] is True, code + ":DERIVATION")


def parse_trace(row: dict[str, Any], output: str, returncode: int) -> dict[str, Any]:
    prefix = "v19-proof-json="
    lines = [line.strip().removeprefix(prefix) for line in output.splitlines() if line.strip().startswith(prefix)]
    require(returncode == 0, "PROOF_EXIT:" + row["id"])
    require(len(lines) == 1, "PROOF_TRACE_COUNT:" + row["id"])
    try:
        trace = json.loads(lines[0])
    except json.JSONDecodeError as error:
        raise ProofError("PROOF_TRACE_JSON:" + row["id"]) from error
    validate_trace(trace, row, "TRACE:" + row["id"])
    require(output.count(f"test {row['proof_test']} ... ok") == 1, "PROOF_TEST_RESULT:" + row["id"])
    require("running 1 test" in output and "1 passed; 0 failed; 0 ignored" in output, "PROOF_NOT_EXACT:" + row["id"])
    return trace


def artifact_path(row: dict[str, Any]) -> str:
    return "reports/evidence/v19/proofs/" + row["id"].replace(".", "_") + ".json"


def execute(rows: list[dict[str, Any]], source_candidate: str) -> str:
    execution_base = git("rev-parse", "HEAD")
    require(not git("status", "--porcelain=v1"), "EXECUTION_BASE_DIRTY")
    require(git("show", f"{execution_base}:{RUNNER_PATH}"), "RUNNER_NOT_COMMITTED")
    ARTIFACT_DIR.mkdir(parents=True, exist_ok=False)
    for row in rows:
        command = shlex.split(row["proof_command"])
        completed = run(command)
        output = completed.stdout + completed.stderr
        trace = parse_trace(row, output, completed.returncode)
        artifact = {
            "schema": "nostr_automerge.causal_projection_proof_trace.v19.v1",
            "authority": AUTHORITY,
            "inventory_row_id": row["id"],
            "source_candidate": source_candidate,
            "execution_base_candidate": execution_base,
            "command": command,
            "cwd": ".",
            "environment": {
                "target_routing": "extbuild",
                "output_capture": "complete_redacted_stdout_and_stderr_with_raw_hashes",
                "source_state": "clean_committed_base",
            },
            "exit_status": completed.returncode,
            "test_result": "passed",
            "stdout": public_transcript(completed.stdout),
            "stderr": public_transcript(completed.stderr),
            "raw_stdout_sha256": sha(completed.stdout.encode()),
            "raw_stderr_sha256": sha(completed.stderr.encode()),
            "trace": trace,
            "result": "pass",
        }
        (ROOT / artifact_path(row)).write_text(json.dumps(artifact, ensure_ascii=True, indent=2) + "\n")
    return execution_base


def load_artifact(row: dict[str, Any], source_candidate: str, execution_base: str) -> tuple[str, dict[str, Any]]:
    raw = (ROOT / artifact_path(row)).read_text()
    artifact = json.loads(raw)
    require(type(artifact) is dict and list(artifact) == ARTIFACT_FIELDS, "ARTIFACT_SHAPE:" + row["id"])
    require(artifact["schema"] == "nostr_automerge.causal_projection_proof_trace.v19.v1", "ARTIFACT_SCHEMA:" + row["id"])
    require(artifact["authority"] == AUTHORITY and artifact["inventory_row_id"] == row["id"], "ARTIFACT_OWNER:" + row["id"])
    require(artifact["source_candidate"] == source_candidate and artifact["execution_base_candidate"] == execution_base, "ARTIFACT_CANDIDATE:" + row["id"])
    require(artifact["command"] == shlex.split(row["proof_command"]), "ARTIFACT_COMMAND:" + row["id"])
    require(artifact["cwd"] == "." and artifact["environment"] == {
        "target_routing": "extbuild", "output_capture": "complete_redacted_stdout_and_stderr_with_raw_hashes",
        "source_state": "clean_committed_base",
    }, "ARTIFACT_ENVIRONMENT:" + row["id"])
    transcript = artifact["stdout"] + artifact["stderr"]
    private_roots = ("/" + "Users/", "/" + "Volumes/")
    require(not any(root in transcript for root in private_roots), "ARTIFACT_PRIVATE_PATH:" + row["id"])
    require(len(artifact["raw_stdout_sha256"]) == len(artifact["raw_stderr_sha256"]) == 64, "ARTIFACT_RAW_HASH:" + row["id"])
    require(artifact["exit_status"] == 0 and artifact["test_result"] == "passed" and artifact["result"] == "pass", "ARTIFACT_RESULT:" + row["id"])
    parsed = parse_trace(row, artifact["stdout"] + artifact["stderr"], artifact["exit_status"])
    require(parsed == artifact["trace"], "ARTIFACT_TRACE:" + row["id"])
    return raw, artifact


def expected_report(inventory: dict[str, Any], execution_base: str) -> dict[str, Any]:
    source_candidate = inventory["source_candidate"]
    rows = []
    for row in inventory["rows"]:
        raw, artifact = load_artifact(row, source_candidate, execution_base)
        rows.append({
            "proof_row_id": "proof." + row["id"],
            "inventory_row_id": row["id"],
            "site_id": row["site_id"],
            "phase": row["phase"],
            "family": row["family"],
            "counter": row["counter"],
            "command": row["proof_command"],
            "trace_artifact": artifact_path(row),
            "trace_sha256": sha(raw.encode()),
            "stdout_sha256": sha(artifact["stdout"].encode()),
            "stderr_sha256": sha(artifact["stderr"].encode()),
            "raw_stdout_sha256": artifact["raw_stdout_sha256"],
            "raw_stderr_sha256": artifact["raw_stderr_sha256"],
            "source_candidate": source_candidate,
            "execution_base_candidate": execution_base,
            "result": "pass",
        })
    report: dict[str, Any] = {
        "schema": "nostr_automerge.causal_projection_proofs.v19.v1",
        "status": "actual_execution_raw_unbound",
        "authority": AUTHORITY,
        "source_candidate": source_candidate,
        "execution_base_candidate": execution_base,
        "inventory_path": INVENTORY_PATH,
        "inventory_sha256": sha(INVENTORY.read_bytes()),
        "row_contract": ROW_FIELDS,
        "rows": rows,
        "counts": {"requested": len(rows), "executed": len(rows), "passed": len(rows), "failed": 0},
        "execution": {
            "mode": "actual", "trace_facts": "independent_runtime_events",
            "count_scope": "requested_operation_window", "raw_transcript": "embedded_redacted_with_raw_hashes",
            "artifact_commit_binding": "later_catalog",
        },
        "result_identity_sha256": "",
        "result": "pass",
    }
    identity = {key: value for key, value in report.items() if key != "result_identity_sha256"}
    report["result_identity_sha256"] = sha(canonical(identity))
    return report


def validate(report: dict[str, Any], schema: dict[str, Any], inventory: dict[str, Any]) -> None:
    require(list(report) == TOP_FIELDS, "REPORT_SHAPE")
    require(schema["additionalProperties"] is False and schema["required"] == TOP_FIELDS, "SCHEMA_CLOSED")
    require(report == expected_report(inventory, report["execution_base_candidate"]), "REPORT_DERIVATION")
    require(report["counts"]["requested"] == len(inventory["rows"]), "REPORT_COUNT")
    require(len(report["rows"]) == len({row["proof_row_id"] for row in report["rows"]}) == len({row["site_id"] for row in report["rows"]}), "REPORT_UNIQUE")
    require(all(list(row) == ROW_FIELDS for row in report["rows"]), "REPORT_ROW_SHAPE")
    base = report["execution_base_candidate"]
    require(git("rev-parse", base + "^{commit}") == base, "EXECUTION_BASE")
    require(run(["git", "merge-base", "--is-ancestor", report["source_candidate"], base]).returncode == 0, "CANDIDATE_ANCESTRY")
    require(git("show", f"{base}:{RUNNER_PATH}") == Path(RUNNER_PATH).read_text().strip(), "RUNNER_BASE_DRIFT")
    expected_artifacts = {artifact_path(row) for row in inventory["rows"]}
    actual_artifacts = {path.relative_to(ROOT).as_posix() for path in ARTIFACT_DIR.glob("*.json")}
    require(actual_artifacts == expected_artifacts, "ARTIFACT_SET")


def self_test(report: dict[str, Any], schema: dict[str, Any], inventory: dict[str, Any]) -> int:
    cases = [
        lambda r, _s: r["rows"].pop(),
        lambda r, _s: r["rows"][0].update(site_id="Nearby"),
        lambda r, _s: r["rows"][0].update(trace_sha256="0" * 64),
        lambda r, _s: r["counts"].update(executed=0),
        lambda r, _s: r["execution"].update(trace_facts="copied_counts"),
        lambda r, _s: r.update(execution_base_candidate="0" * 40),
        lambda _r, s: s.update(additionalProperties=True),
    ]
    caught = 0
    for mutate in cases:
        changed_report, changed_schema = copy.deepcopy(report), copy.deepcopy(schema)
        mutate(changed_report, changed_schema)
        try:
            validate(changed_report, changed_schema, inventory)
        except ProofError:
            caught += 1
            continue
        raise ProofError("MUTATION_SURVIVED")
    return caught


def parser_self_test(inventory: dict[str, Any]) -> int:
    row = inventory["rows"][0]
    valid = {
        "site_id": row["site_id"], "phase": row["phase"], "family": row["family"],
        "counter": row["counter"].replace("_", "").title(), "requested_site": row["site_id"],
        "n_minus_one_result": "BudgetExhausted", "n_minus_one_events": FAILED_EVENTS,
        "n_result": "requested_site_completed", "n_events": SUCCESS_EVENTS,
        "n_plus_one_result": "requested_site_completed", "n_plus_one_events": SUCCESS_EVENTS,
        "cancelled_result": "Cancelled", "cancelled_events": FAILED_EVENTS,
        "unexpected_error_identity": "exact", "unexpected_events": FAILED_EVENTS,
        "count_scope": "requested_operation_window", "event_counts_derived": True,
    }
    cases = [
        lambda value: value["n_events"].__setitem__(3, "CompletionObserved"),
        lambda value: value["n_events"].pop(),
        lambda value: value["n_events"].insert(3, "TargetDispatched"),
        lambda value: value["n_events"].reverse(),
        lambda value: value.update(n_minus_one_events=["ChargeAttempt", "TargetDispatched"]),
        lambda value: value.update(event_counts_derived=False),
    ]
    validate_trace(valid, row, "SELF_VALID")
    redacted = public_transcript("from /" + "Users/example/project and /" + "Volumes/build/target)\n")
    require(redacted == "from <local-path> and <local-path>)\n", "TRANSCRIPT_REDACTION")
    caught = 0
    for mutate in cases:
        changed = copy.deepcopy(valid)
        mutate(changed)
        try:
            validate_trace(changed, row, "SELF_ATTACK")
        except ProofError:
            caught += 1
            continue
        raise ProofError("PARSER_MUTATION_SURVIVED")
    return caught


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--execute", action="store_true")
    parser.add_argument("--write-report", action="store_true")
    parser.add_argument("--self-test-parser", action="store_true")
    args = parser.parse_args()
    inventory = load(INVENTORY)
    if args.self_test_parser:
        print(f"PASS v19 proof parser attacks={parser_self_test(inventory)}")
        return 0
    if args.execute:
        base = execute(inventory["rows"], inventory["source_candidate"])
        generated = expected_report(inventory, base)
        if args.write_report:
            REPORT.write_text(json.dumps(generated, ensure_ascii=True, indent=2) + "\n")
    require(REPORT.is_file(), "REPORT_MISSING")
    report = load(REPORT)
    schema = load(SCHEMA)
    validate(report, schema, inventory)
    mode = "executed" if args.execute else "committed"
    print(f"PASS v19 causal projection proofs: mode={mode} exact={len(report['rows'])} attacks={self_test(report, schema, inventory)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
