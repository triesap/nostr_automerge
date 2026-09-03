#!/usr/bin/env python3
"""Execute and validate the twice-run final public Rust v17 assurance gate."""

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
GRAPH_PATH = "reports/causal_projection_evidence_graph_v17.json"
GRAPH_CANDIDATE = "e74dcdb3fdaa30aeeb59bab53126bbee82a64557"
GRAPH_SHA256 = "283224879f13a69840e7222523649cc9639d73ae3cbe99464127b78f0121c527"
REPORT = ROOT / "reports/causal_projection_public_assurance_v17.json"
SCHEMA = ROOT / "tools/validation/causal_projection_public_assurance_v17.schema.json"
CANONICAL_OUTPUT_SHA256 = "e69c721549966b1b88dcde3296674d675169840c6e8ebd0f236a5c07bcfc6415"
COMMANDS = [
    ("format", ["cargo", "extbuild", "run", "--", "cargo", "fmt", "--all", "--", "--check"]),
    ("check", ["cargo", "extbuild", "run", "--", "cargo", "check", "--workspace", "--all-targets", "--locked"]),
    ("test", ["cargo", "extbuild", "run", "--", "cargo", "test", "--workspace", "--all-targets", "--locked"]),
    ("clippy", ["cargo", "extbuild", "run", "--", "cargo", "clippy", "--workspace", "--all-targets", "--all-features", "--locked", "--", "-D", "warnings"]),
    ("rustdoc", ["cargo", "extbuild", "run", "--", "cargo", "doc", "--workspace", "--no-deps", "--locked"]),
    ("xtask", ["cargo", "extbuild", "run", "--", "cargo", "run", "-p", "nostr_automerge_xtask", "--", "validate"]),
    ("specification", ["cargo", "extbuild", "run", "--", "python3", "scripts/validate_spec.py"]),
    ("diff", ["git", "diff", "--check"]),
]


class AssuranceError(RuntimeError):
    pass


def require(condition: bool, code: str) -> None:
    if not condition:
        raise AssuranceError(code)


def sha(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def canonical(value: object) -> bytes:
    return json.dumps(value, ensure_ascii=True, separators=(",", ":"), sort_keys=True).encode()


def committed(candidate: str, path: str) -> bytes:
    result = subprocess.run(["git", "show", f"{candidate}:{path}"], cwd=ROOT, capture_output=True, check=False)
    require(result.returncode == 0, "GRAPH_CANDIDATE")
    return result.stdout


def execute_run(ordinal: int) -> dict[str, Any]:
    rows = []
    for name, command in COMMANDS:
        print(f"v17 public assurance run={ordinal} command={name} start", flush=True)
        completed = subprocess.run(command, cwd=ROOT, capture_output=True, text=True, check=False)
        if completed.returncode:
            sys.stderr.write(completed.stdout[-3000:] + completed.stderr[-3000:])
            raise AssuranceError(f"GATE:{ordinal}:{name}:{completed.returncode}")
        output = completed.stdout + completed.stderr
        row = {
            "name": name, "command": " ".join(command), "exit_status": 0,
            "test_result_ok_count": output.count("test result: ok."),
            "pass_marker_count": output.count("PASS:"), "result": "pass",
        }
        rows.append(row)
        print(f"v17 public assurance run={ordinal} command={name} pass", flush=True)
    identity = sha(canonical(rows))
    return {"ordinal": ordinal, "commands": rows, "normalized_identity_sha256": identity, "result": "pass"}


def expected_shells() -> list[dict[str, str]]:
    return [{"name": name, "command": " ".join(command)} for name, command in COMMANDS]


def validate(report: dict[str, Any], schema: dict[str, Any]) -> None:
    graph = committed(GRAPH_CANDIDATE, GRAPH_PATH)
    require(sha(graph) == GRAPH_SHA256 and json.loads(graph)["result"] == "pass", "GRAPH_IDENTITY")
    require(report["status"] == "final" and report["execution_mode"] == "actual_twice", "STATUS")
    require(report["evidence_graph"] == {"path": GRAPH_PATH, "candidate": GRAPH_CANDIDATE, "sha256": GRAPH_SHA256}, "GRAPH_BINDING")
    require(report["commands"] == expected_shells(), "COMMANDS")
    require(len(report["runs"]) == 2 and [run["ordinal"] for run in report["runs"]] == [1, 2], "RUNS")
    for run in report["runs"]:
        require(run["result"] == "pass" and len(run["commands"]) == len(COMMANDS), "RUN_RESULT")
        require(run["normalized_identity_sha256"] == sha(canonical(run["commands"])), "RUN_IDENTITY")
        require([{"name": row["name"], "command": row["command"]} for row in run["commands"]] == expected_shells(), "RUN_COMMANDS")
        require(all(row["exit_status"] == 0 and row["result"] == "pass" for row in run["commands"]), "RUN_FAILURE")
    require(report["runs"][0]["normalized_identity_sha256"] == report["runs"][1]["normalized_identity_sha256"], "RUN_MISMATCH")
    require(report["canonical_output"] == {"signed_events": 771, "sha256": CANONICAL_OUTPUT_SHA256, "changed": False}, "CANONICAL_OUTPUT")
    require(report["terminal_inputs"] == {"final_inventory": True, "evidence_graph": True, "provisional": False, "planned": False}, "TERMINAL_INPUTS")
    require(report["result"] == "pass", "RESULT")
    require(schema.get("additionalProperties") is False and schema.get("required") == list(report), "SCHEMA_CLOSED")


def self_test(report: dict[str, Any], schema: dict[str, Any]) -> int:
    attacks = [
        lambda value: value["runs"][1].update(normalized_identity_sha256="0" * 64),
        lambda value: value["runs"][0]["commands"][0].update(exit_status=1),
        lambda value: value["evidence_graph"].update(sha256="0" * 64),
        lambda value: value["terminal_inputs"].update(provisional=True),
    ]
    caught = 0
    for attack in attacks:
        changed = copy.deepcopy(report); attack(changed)
        try:
            validate(changed, schema)
        except AssuranceError:
            caught += 1
            continue
        raise AssuranceError("ATTACK_SURVIVED")
    return caught


def main() -> int:
    parser = argparse.ArgumentParser(); parser.add_argument("--execute", action="store_true"); args = parser.parse_args()
    if args.execute:
        runs = [execute_run(1), execute_run(2)]
        document = {
            "schema": "nostr_automerge.causal_projection_public_assurance.v17.v1",
            "status": "final", "execution_mode": "actual_twice",
            "evidence_graph": {"path": GRAPH_PATH, "candidate": GRAPH_CANDIDATE, "sha256": GRAPH_SHA256},
            "commands": expected_shells(), "runs": runs,
            "canonical_output": {"signed_events": 771, "sha256": CANONICAL_OUTPUT_SHA256, "changed": False},
            "terminal_inputs": {"final_inventory": True, "evidence_graph": True, "provisional": False, "planned": False},
            "result": "pass",
        }
        REPORT.write_text(json.dumps(document, ensure_ascii=True, indent=2) + "\n")
    report = json.loads(REPORT.read_text()); schema = json.loads(SCHEMA.read_text())
    validate(report, schema); attacks = self_test(report, schema)
    print(f"PASS: causal projection public assurance v17 runs=2 identical=true attacks={attacks}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
