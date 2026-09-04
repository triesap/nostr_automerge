#!/usr/bin/env python3
"""Execute and validate one trace-derived exact proof per v18 Rust site."""

from __future__ import annotations

import argparse
import copy
import hashlib
import json
import shlex
import subprocess
import sys
from pathlib import Path
from typing import Any

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parents[1]
INVENTORY_PATH = "reports/causal_projection_inventory_v18.json"
INVENTORY = ROOT / INVENTORY_PATH
REPORT = ROOT / "reports/causal_projection_proofs_v18.json"
SCHEMA = ROOT / "tools/validation/causal_projection_proofs_v18.schema.json"
ARTIFACT_DIR = ROOT / "reports/evidence/v18/proofs"
AUTHORITY = "spec/causal_projection_contracts_v18.json"
SOURCE_CANDIDATE = "076221ad7f03e67d89ac4b2fcfc8f2586b97f182"
EXECUTION_BASE_CANDIDATE = "94348cf0f92c8eded557d67ec9c933e647d73f6b"
ROW_FIELDS = [
    "proof_row_id", "inventory_row_id", "site_id", "phase", "family",
    "counter", "command", "requested_site", "observed_completed_site",
    "n_minus_one_result", "n_result", "n_plus_one_result",
    "cancelled_result", "unexpected_error_identity",
    "target_count_at_n_minus_one", "completion_observation_count_at_n_minus_one",
    "publication_count_at_n_minus_one", "charge_attempt_count_at_n_minus_one",
    "trace_artifact", "trace_sha256", "source_candidate",
    "execution_base_candidate", "result",
]
TOP_FIELDS = [
    "schema", "status", "authority", "source_candidate",
    "execution_base_candidate", "inventory_path", "inventory_sha256",
    "row_contract", "rows", "counts", "execution",
    "result_identity_sha256", "result",
]
TRACE_FIELDS = [
    "site_id", "phase", "family", "counter", "requested_site",
    "observed_completed_site", "n_minus_one_result", "n_result",
    "n_plus_one_result", "cancelled_result", "unexpected_error_identity",
    "target_count_at_n_minus_one", "completion_observation_count_at_n_minus_one",
    "publication_count_at_n_minus_one", "charge_attempt_count_at_n_minus_one",
    "count_scope",
]
ARTIFACT_FIELDS = [
    "schema", "authority", "inventory_row_id", "source_candidate",
    "execution_base_candidate", "command", "cwd", "environment",
    "exit_status", "test_result", "stdout_sha256", "stderr_sha256",
    "trace", "result",
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


def git(*args: str, binary: bool = False) -> str | bytes:
    completed = subprocess.run(
        ["git", *args], cwd=ROOT, capture_output=True, text=not binary, check=False
    )
    require(completed.returncode == 0, "GIT:" + ":".join(args))
    return completed.stdout


def inventory() -> dict[str, Any]:
    return json.loads(INVENTORY.read_text())


def artifact_path(row: dict[str, Any]) -> str:
    return "reports/evidence/v18/proofs/" + row["id"].replace(".", "_") + ".json"


def parse_trace(row: dict[str, Any], output: str, returncode: int) -> dict[str, Any]:
    prefix = "v18-proof-json="
    lines = [line.strip().removeprefix(prefix) for line in output.splitlines() if line.strip().startswith(prefix)]
    require(returncode == 0, f"PROOF_EXIT:{row['id']}")
    require(len(lines) == 1, f"PROOF_TRACE_COUNT:{row['id']}")
    try:
        trace = json.loads(lines[0])
    except json.JSONDecodeError as error:
        raise ProofError(f"PROOF_TRACE_JSON:{row['id']}") from error
    require(type(trace) is dict and list(trace) == TRACE_FIELDS, f"PROOF_TRACE_SHAPE:{row['id']}")
    require(trace["site_id"] == row["site_id"], f"SITE_ID_MISMATCH:{row['id']}")
    require(trace["phase"] == row["phase"], f"PHASE_MISMATCH:{row['id']}")
    require(trace["family"] == row["family"], f"FAMILY_MISMATCH:{row['id']}")
    require(trace["counter"].casefold() == row["counter"].replace("_", ""), f"COUNTER_MISMATCH:{row['id']}")
    require(trace["requested_site"] == row["site_id"], f"REQUESTED_SITE_MISMATCH:{row['id']}")
    require(trace["observed_completed_site"] == row["site_id"], f"COMPLETED_SITE_MISMATCH:{row['id']}")
    require(trace["n_minus_one_result"] == "BudgetExhausted", f"BUDGET_IDENTITY:{row['id']}")
    require(trace["n_result"] == "requested_site_completed", f"N_RESULT:{row['id']}")
    require(trace["n_plus_one_result"] == "requested_site_completed", f"N_PLUS_ONE_RESULT:{row['id']}")
    require(trace["cancelled_result"] == "Cancelled", f"CANCELLED_IDENTITY:{row['id']}")
    require(trace["unexpected_error_identity"] == "exact", f"UNEXPECTED_IDENTITY:{row['id']}")
    require(trace["target_count_at_n_minus_one"] == 0, f"TARGET_AFTER_STOP:{row['id']}")
    require(trace["completion_observation_count_at_n_minus_one"] == 0, f"OBSERVATION_AFTER_STOP:{row['id']}")
    require(trace["publication_count_at_n_minus_one"] == 0, f"PUBLICATION_AFTER_STOP:{row['id']}")
    require(trace["charge_attempt_count_at_n_minus_one"] == 1, f"CHARGE_ATTEMPT_COUNT:{row['id']}")
    require(trace["count_scope"] == "post_failed_charge_suffix", f"COUNT_SCOPE:{row['id']}")
    require(output.count(f"test {row['proof_test']} ... ok") == 1, f"PROOF_TEST_RESULT:{row['id']}")
    require("running 1 test" in output and "1 passed; 0 failed; 0 ignored" in output, f"PROOF_NOT_EXACT:{row['id']}")
    return trace


def execute(rows: list[dict[str, Any]]) -> None:
    require(git("rev-parse", "HEAD").strip() == EXECUTION_BASE_CANDIDATE, "EXECUTION_BASE_NOT_HEAD")
    ARTIFACT_DIR.mkdir(parents=True, exist_ok=True)
    for row in rows:
        argv = shlex.split(row["proof_command"])
        completed = subprocess.run(argv, cwd=ROOT, capture_output=True, text=True, check=False)
        trace = parse_trace(row, completed.stdout + completed.stderr, completed.returncode)
        artifact = {
            "schema": "nostr_automerge.causal_projection_proof_trace.v18.v1",
            "authority": AUTHORITY,
            "inventory_row_id": row["id"],
            "source_candidate": SOURCE_CANDIDATE,
            "execution_base_candidate": EXECUTION_BASE_CANDIDATE,
            "command": argv,
            "cwd": ".",
            "environment": {"target_routing": "extbuild", "output_capture": "stdout_and_stderr"},
            "exit_status": completed.returncode,
            "test_result": "passed",
            "stdout_sha256": sha(completed.stdout.encode()),
            "stderr_sha256": sha(completed.stderr.encode()),
            "trace": trace,
            "result": "pass",
        }
        (ROOT / artifact_path(row)).write_text(json.dumps(artifact, ensure_ascii=True, indent=2) + "\n")


def load_artifact(row: dict[str, Any]) -> tuple[str, dict[str, Any]]:
    relative = artifact_path(row)
    raw = (ROOT / relative).read_text()
    artifact = json.loads(raw)
    require(type(artifact) is dict and list(artifact) == ARTIFACT_FIELDS, f"ARTIFACT_SHAPE:{row['id']}")
    require(artifact["schema"] == "nostr_automerge.causal_projection_proof_trace.v18.v1", f"ARTIFACT_SCHEMA:{row['id']}")
    require(artifact["authority"] == AUTHORITY, f"ARTIFACT_AUTHORITY:{row['id']}")
    require(artifact["inventory_row_id"] == row["id"], f"ARTIFACT_INVENTORY:{row['id']}")
    require(artifact["source_candidate"] == SOURCE_CANDIDATE, f"ARTIFACT_SOURCE:{row['id']}")
    require(artifact["execution_base_candidate"] == EXECUTION_BASE_CANDIDATE, f"ARTIFACT_BASE:{row['id']}")
    require(artifact["command"] == shlex.split(row["proof_command"]), f"ARTIFACT_COMMAND:{row['id']}")
    require(artifact["cwd"] == ".", f"ARTIFACT_CWD:{row['id']}")
    require(artifact["environment"] == {"target_routing": "extbuild", "output_capture": "stdout_and_stderr"}, f"ARTIFACT_ENVIRONMENT:{row['id']}")
    require(artifact["exit_status"] == 0 and artifact["test_result"] == "passed", f"ARTIFACT_RESULT:{row['id']}")
    require(artifact["result"] == "pass", f"ARTIFACT_RESULT_CLASS:{row['id']}")
    trace = artifact["trace"]
    parse_trace(row, "v18-proof-json=" + json.dumps(trace) + f"\nrunning 1 test\ntest {row['proof_test']} ... ok\ntest result: ok. 1 passed; 0 failed; 0 ignored", 0)
    return raw, artifact


def proof_row(row: dict[str, Any]) -> dict[str, Any]:
    raw, artifact = load_artifact(row)
    trace = artifact["trace"]
    return {
        "proof_row_id": "proof." + row["id"],
        "inventory_row_id": row["id"],
        "site_id": trace["site_id"],
        "phase": trace["phase"],
        "family": trace["family"],
        "counter": row["counter"],
        "command": row["proof_command"],
        "requested_site": trace["requested_site"],
        "observed_completed_site": trace["observed_completed_site"],
        "n_minus_one_result": trace["n_minus_one_result"],
        "n_result": trace["n_result"],
        "n_plus_one_result": trace["n_plus_one_result"],
        "cancelled_result": trace["cancelled_result"],
        "unexpected_error_identity": trace["unexpected_error_identity"],
        "target_count_at_n_minus_one": trace["target_count_at_n_minus_one"],
        "completion_observation_count_at_n_minus_one": trace["completion_observation_count_at_n_minus_one"],
        "publication_count_at_n_minus_one": trace["publication_count_at_n_minus_one"],
        "charge_attempt_count_at_n_minus_one": trace["charge_attempt_count_at_n_minus_one"],
        "trace_artifact": artifact_path(row),
        "trace_sha256": sha(raw.encode()),
        "source_candidate": SOURCE_CANDIDATE,
        "execution_base_candidate": EXECUTION_BASE_CANDIDATE,
        "result": "pass",
    }


def expected_report(inv: dict[str, Any]) -> dict[str, Any]:
    rows = [proof_row(row) for row in inv["rows"]]
    report: dict[str, Any] = {
        "schema": "nostr_automerge.causal_projection_proofs.v18.v1",
        "status": "actual_execution_raw_unbound",
        "authority": AUTHORITY,
        "source_candidate": SOURCE_CANDIDATE,
        "execution_base_candidate": EXECUTION_BASE_CANDIDATE,
        "inventory_path": INVENTORY_PATH,
        "inventory_sha256": sha(INVENTORY.read_bytes()),
        "row_contract": ROW_FIELDS,
        "rows": rows,
        "counts": {"requested": len(rows), "executed": len(rows), "passed": len(rows), "failed": 0},
        "execution": {"mode": "actual", "trace_facts": "structured_production_path", "count_scope": "post_failed_charge_suffix", "artifact_commit_binding": "later_catalog"},
        "result_identity_sha256": "",
        "result": "pass",
    }
    identity = {key: value for key, value in report.items() if key != "result_identity_sha256"}
    report["result_identity_sha256"] = sha(canonical(identity))
    return report


def validate(report: object, schema: object, inv: dict[str, Any]) -> None:
    require(type(report) is dict and list(report) == TOP_FIELDS, "REPORT_SHAPE")
    require(report == expected_report(inv), "REPORT_DERIVATION_MISMATCH")
    require(inv["status"] == "provisional_source_derived", "INVENTORY_STATUS")
    require(inv["source_candidate"] == SOURCE_CANDIDATE, "INVENTORY_SOURCE")
    committed_inventory = git("show", f"{EXECUTION_BASE_CANDIDATE}:{INVENTORY_PATH}", binary=True)
    require(sha(committed_inventory) == report["inventory_sha256"], "INVENTORY_BASE_DRIFT")
    require(git("rev-parse", f"{SOURCE_CANDIDATE}^{{commit}}").strip() == SOURCE_CANDIDATE, "SOURCE_CANDIDATE")
    require(git("rev-parse", f"{EXECUTION_BASE_CANDIDATE}^{{commit}}").strip() == EXECUTION_BASE_CANDIDATE, "EXECUTION_BASE_CANDIDATE")
    ancestry = subprocess.run(["git", "merge-base", "--is-ancestor", SOURCE_CANDIDATE, EXECUTION_BASE_CANDIDATE], cwd=ROOT, check=False)
    require(ancestry.returncode == 0, "CANDIDATE_ANCESTRY")
    require(len(report["rows"]) == len(inv["rows"]), "PROOF_ROW_COUNT")
    require(len(report["rows"]) == len({row["proof_row_id"] for row in report["rows"]}) == len({row["site_id"] for row in report["rows"]}), "PROOF_ROWS_UNIQUE")
    require(all(list(row) == ROW_FIELDS for row in report["rows"]), "PROOF_ROW_SHAPE")
    require(all(row["requested_site"] == row["observed_completed_site"] == row["site_id"] for row in report["rows"]), "SITE_IDENTITY")
    expected_artifacts = {artifact_path(row) for row in inv["rows"]}
    actual_artifacts = {path.relative_to(ROOT).as_posix() for path in ARTIFACT_DIR.glob("*.json")}
    require(actual_artifacts == expected_artifacts, "ARTIFACT_SET")
    require(type(schema) is dict and schema.get("additionalProperties") is False and schema.get("required") == TOP_FIELDS, "SCHEMA_CLOSED")
    rows_schema = schema["properties"]["rows"]
    require(rows_schema.get("minItems") == 1 and "maxItems" not in rows_schema, "SCHEMA_SOURCE_DERIVED_COUNT")
    require(rows_schema["items"].get("additionalProperties") is False and rows_schema["items"].get("required") == ROW_FIELDS, "SCHEMA_ROW_CLOSED")


def self_test(report: dict[str, Any], schema: dict[str, Any], inv: dict[str, Any]) -> int:
    attacks = [
        ("missing", "report", lambda value: value["rows"].pop()),
        ("duplicate", "report", lambda value: value["rows"].__setitem__(1, copy.deepcopy(value["rows"][0]))),
        ("requested", "report", lambda value: value["rows"][0].update(requested_site="Nearby")),
        ("completed", "report", lambda value: value["rows"][0].update(observed_completed_site="Nearby")),
        ("count", "report", lambda value: value["rows"][0].update(target_count_at_n_minus_one=1)),
        ("result", "report", lambda value: value["rows"][0].update(n_minus_one_result="label_only")),
        ("artifact", "report", lambda value: value["rows"][0].update(trace_sha256="0" * 64)),
        ("candidate", "report", lambda value: value.update(execution_base_candidate="0" * 40)),
        ("order", "report", lambda value: value["rows"].reverse()),
        ("schema", "schema", lambda value: value.update(additionalProperties=True)),
    ]
    caught = 0
    for label, target, mutate in attacks:
        changed_report, changed_schema = copy.deepcopy(report), copy.deepcopy(schema)
        mutate(changed_report if target == "report" else changed_schema)
        try:
            validate(changed_report, changed_schema, inv)
        except ProofError:
            caught += 1
            continue
        raise ProofError(f"MUTATION_SURVIVED:{label}")
    return caught


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--execute", action="store_true")
    parser.add_argument("--write-report", action="store_true")
    args = parser.parse_args()
    inv = inventory()
    if args.execute:
        execute(inv["rows"])
    expected = expected_report(inv)
    if args.write_report:
        require(args.execute, "WRITE_REQUIRES_EXECUTION")
        REPORT.write_text(json.dumps(expected, ensure_ascii=True, indent=2) + "\n")
    report = json.loads(REPORT.read_text())
    schema = json.loads(SCHEMA.read_text())
    validate(report, schema, inv)
    attacks = self_test(report, schema, inv)
    mode = "executed" if args.execute else "committed"
    print(f"PASS: causal projection proofs v18 mode={mode} exact={len(report['rows'])} attacks={attacks}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
