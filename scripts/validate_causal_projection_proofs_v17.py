#!/usr/bin/env python3
"""Execute and validate one actual exact proof for every provisional v17 site."""

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
INVENTORY_PATH = "reports/causal_projection_inventory_v17.json"
INVENTORY = ROOT / INVENTORY_PATH
REPORT = ROOT / "reports/causal_projection_proofs_v17.json"
SCHEMA = ROOT / "tools/validation/causal_projection_proofs_v17.schema.json"
ARTIFACT_DIR = ROOT / "reports/evidence/v17/proofs"
SOURCE = ROOT / "crates/nostr_automerge/src/graph/actor_state.rs"
SOURCE_CANDIDATE = "789eae3c6e0994f71420f49fe51fe3ab7cb75ca9"
INVENTORY_CANDIDATE = "6f8ee840b7be41a32ad6b46392b75aae921df3cb"
PROOF_CANDIDATE = INVENTORY_CANDIDATE
INVENTORY_SHA256 = "802fb3c3b75bb915a0f765b70f0eb9fc4d9eb72d97bc4f5f6ed4f8f400208551"
AUTHORITY = "spec/causal_projection_contracts_v17.json"
ROW_FIELDS = [
    "proof_row_id", "inventory_row_id", "site_id", "command",
    "requested_site", "observed_site", "counter", "n_minus_one", "n",
    "n_plus_one", "cancellation_identity", "unexpected_error_identity",
    "target_after_stop", "observation_after_stop", "transcript_artifact",
    "transcript_sha256", "source_candidate", "proof_candidate", "result",
]
TOP_FIELDS = [
    "schema", "status", "authority", "source_candidate",
    "inventory_candidate", "proof_candidate", "inventory_path",
    "inventory_sha256", "row_contract", "rows", "counts", "execution",
    "result_identity_sha256", "result",
]
OBSERVED = re.compile(
    r"^v17-proof site=(?P<site>\w+) family=(?P<family>\w+) counter=(?P<counter>GraphNode|GraphEdge) "
    r"n_minus_one=blocked n=observed n_plus_one=observed cancellation=exact "
    r"unexpected_error=exact target_after_stop=0 observation_after_stop=0$",
    re.MULTILINE,
)


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
    completed = subprocess.run(["git", *args], cwd=ROOT, capture_output=True, text=not binary, check=False)
    require(completed.returncode == 0, "GIT:" + ":".join(args))
    return completed.stdout


def inventory() -> dict[str, Any]:
    return json.loads(INVENTORY.read_text())


def artifact_path(row: dict[str, Any]) -> str:
    return "reports/evidence/v17/proofs/" + row["id"].replace(".", "_") + ".txt"


def normalize(row: dict[str, Any], output: str, returncode: int) -> str:
    matches = list(OBSERVED.finditer(output))
    require(returncode == 0, f"PROOF_EXIT:{row['id']}")
    require(len(matches) == 1, f"PROOF_OBSERVATION_COUNT:{row['id']}")
    observed = matches[0]
    require(observed.group("site") == row["site_id"], f"SITE_ID_MISMATCH:{row['id']}")
    require(observed.group("family") == row["operation"], f"OPERATION_MISMATCH:{row['id']}")
    require(observed.group("counter").lower() == row["counter"].replace("_", ""), f"COUNTER_MISMATCH:{row['id']}")
    require(output.count(f"test {row['proof_test']} ... ok") == 1, f"PROOF_TEST_RESULT:{row['id']}")
    require("running 1 test" in output and "1 passed; 0 failed; 0 ignored" in output, f"PROOF_NOT_EXACT:{row['id']}")
    return "\n".join((
        f"command={row['proof_command']}",
        f"requested_site={row['site_id']}",
        f"observed={observed.group(0)}",
        "exit_status=0",
        "test_result=passed",
        "",
    ))


def execute(rows: list[dict[str, Any]]) -> None:
    ARTIFACT_DIR.mkdir(parents=True, exist_ok=True)
    for row in rows:
        completed = subprocess.run(row["proof_command"].split(), cwd=ROOT, capture_output=True, text=True, check=False)
        transcript = normalize(row, completed.stdout + completed.stderr, completed.returncode)
        (ROOT / artifact_path(row)).write_text(transcript)


def proof_row(row: dict[str, Any]) -> dict[str, Any]:
    path = artifact_path(row)
    transcript = (ROOT / path).read_text()
    observed_lines = [line.removeprefix("observed=") for line in transcript.splitlines() if line.startswith("observed=")]
    observed = OBSERVED.fullmatch(observed_lines[0]) if len(observed_lines) == 1 else None
    require(observed is not None, f"TRANSCRIPT_OBSERVATION:{row['id']}")
    return {
        "proof_row_id": "proof." + row["id"],
        "inventory_row_id": row["id"],
        "site_id": row["site_id"],
        "command": row["proof_command"],
        "requested_site": row["site_id"],
        "observed_site": observed.group("site"),
        "counter": row["counter"],
        "n_minus_one": "typed_budget_exhausted",
        "n": "observed",
        "n_plus_one": "observed",
        "cancellation_identity": "exact",
        "unexpected_error_identity": "exact",
        "target_after_stop": 0,
        "observation_after_stop": 0,
        "transcript_artifact": path,
        "transcript_sha256": sha(transcript.encode()),
        "source_candidate": SOURCE_CANDIDATE,
        "proof_candidate": PROOF_CANDIDATE,
        "result": "pass",
    }


def expected_report(inv: dict[str, Any]) -> dict[str, Any]:
    rows = [proof_row(row) for row in inv["rows"]]
    report: dict[str, Any] = {
        "schema": "nostr_automerge.causal_projection_proofs.v17.v1",
        "status": "actual_execution",
        "authority": AUTHORITY,
        "source_candidate": SOURCE_CANDIDATE,
        "inventory_candidate": INVENTORY_CANDIDATE,
        "proof_candidate": PROOF_CANDIDATE,
        "inventory_path": INVENTORY_PATH,
        "inventory_sha256": INVENTORY_SHA256,
        "row_contract": ROW_FIELDS,
        "rows": rows,
        "counts": {"requested": len(rows), "executed": len(rows), "passed": len(rows), "failed": 0},
        "execution": {"mode": "actual", "normalization": "runner_noise_removed_identity_preserved"},
        "result_identity_sha256": "",
        "result": "pass",
    }
    identity = {key: value for key, value in report.items() if key != "result_identity_sha256"}
    report["result_identity_sha256"] = sha(canonical(identity))
    return report


def validate(report: object, schema: object, inv: dict[str, Any]) -> None:
    require(type(report) is dict and list(report) == TOP_FIELDS, "REPORT_SHAPE")
    require(report == expected_report(inv), "REPORT_DERIVATION_MISMATCH")
    require(inv["status"] == "provisional" and len(inv["rows"]) == 68, "INVENTORY_INPUT")
    committed_inventory = git("show", f"{INVENTORY_CANDIDATE}:{INVENTORY_PATH}", binary=True)
    require(sha(committed_inventory) == INVENTORY_SHA256, "INVENTORY_CANDIDATE_DRIFT")
    require(git("rev-parse", f"{SOURCE_CANDIDATE}^{{commit}}").strip() == SOURCE_CANDIDATE, "SOURCE_CANDIDATE")
    require(git("rev-parse", f"{PROOF_CANDIDATE}^{{commit}}").strip() == PROOF_CANDIDATE, "PROOF_CANDIDATE")
    require(len(report["rows"]) == len({row["proof_row_id"] for row in report["rows"]}) == len({row["site_id"] for row in report["rows"]}) == 68, "PROOF_ROWS_UNIQUE")
    require(all(list(row) == ROW_FIELDS and row["requested_site"] == row["observed_site"] for row in report["rows"]), "PROOF_ROW_SHAPE")
    require(all(row["proof_test"].rsplit("::", 1)[-1] in SOURCE.read_text() for row in inv["rows"]), "PROOF_TEST_MISSING")
    require(type(schema) is dict and schema.get("additionalProperties") is False and schema.get("required") == TOP_FIELDS, "SCHEMA_CLOSED")
    require(schema["properties"]["rows"].get("minItems") == schema["properties"]["rows"].get("maxItems") == 68, "SCHEMA_ROW_COUNT")


def self_test(report: dict[str, Any], schema: dict[str, Any], inv: dict[str, Any]) -> int:
    attacks = [
        ("missing", "report", lambda value: value["rows"].pop()),
        ("duplicate", "report", lambda value: value["rows"].__setitem__(1, copy.deepcopy(value["rows"][0]))),
        ("requested", "report", lambda value: value["rows"][0].update(requested_site="Nearby")),
        ("observed", "report", lambda value: value["rows"][0].update(observed_site="Nearby")),
        ("command", "report", lambda value: value["rows"][0].update(command="cargo test umbrella")),
        ("artifact", "report", lambda value: value["rows"][0].update(transcript_sha256="0" * 64)),
        ("candidate", "report", lambda value: value.update(proof_candidate="0" * 40)),
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
    print(f"PASS: causal projection proofs v17 mode={mode} exact={len(report['rows'])} attacks={attacks}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
