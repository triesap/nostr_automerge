#!/usr/bin/env python3
"""Execute and validate the strict T < E < A clean attestation for v19."""

from __future__ import annotations

import argparse
import copy
import hashlib
import json
import re
import subprocess
import sys
from pathlib import Path
from typing import Any, Callable

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parents[1]
REPORT_PATH = "reports/causal_projection_clean_candidate_v19.json"
REPORT = ROOT / REPORT_PATH
SCHEMA_PATH = "tools/validation/causal_projection_clean_candidate_v19.schema.json"
SCHEMA = ROOT / SCHEMA_PATH
RUNNER_PATH = "scripts/run_causal_projection_clean_attestation_v19.py"
EXECUTION_REPORT_PATH = "reports/causal_projection_attestation_execution_base_v19.json"
TERMINAL = "5fc3fb38ede64397684b699f364c29f07bc23559"
EMPTY_SHA256 = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
HOLDS = [
    "external_assurance", "event_kind_allocation", "nip_submission",
    "production_qualification", "publication", "release", "deployment",
    "remote_mutation",
]
FIELDS = [
    "schema", "status", "checkpoint", "terminal_artifact_commit",
    "attestation_execution_base", "observation", "verification",
    "terminal_artifacts", "lifecycle", "holds", "release_claimed",
    "publication_claimed", "remote_actions", "result", "result_identity_sha256",
]
RUN_FIELDS = [
    "ordinal", "gate", "argv", "cwd", "environment", "exit_status",
    "stdout_sha256", "stderr_sha256", "output_sha256", "result",
]


class AttestationError(RuntimeError):
    pass


def require(value: bool, code: str) -> None:
    if not value:
        raise AttestationError(code)


def git(*args: str) -> subprocess.CompletedProcess[bytes]:
    return subprocess.run(["git", *args], cwd=ROOT, capture_output=True, check=False)


def sha(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def canonical(value: Any) -> bytes:
    return json.dumps(
        value, ensure_ascii=False, sort_keys=True, separators=(",", ":")
    ).encode()


def load(path: Path) -> Any:
    def closed(pairs: list[tuple[str, Any]]) -> dict[str, Any]:
        keys = [key for key, _ in pairs]
        require(len(keys) == len(set(keys)), "duplicate:" + path.name)
        return dict(pairs)

    return json.loads(path.read_text(), object_pairs_hook=closed)


def committed(candidate: str, path: str) -> bytes:
    result = git("show", f"{candidate}:{path}")
    require(result.returncode == 0, "binding:" + path)
    return result.stdout


def environment_record() -> dict[str, Any]:
    return {"mode": "inherited", "overrides": {}}


def gate_command(gate: str) -> list[str]:
    return ["cargo", "extbuild", "run", "--", "python3", "scripts/local_gate.py", gate]


def execute(ordinal: int, gate: str) -> dict[str, Any]:
    command = gate_command(gate)
    print(f"v19 clean attestation repetition-{ordinal}-{gate} start", flush=True)
    completed = subprocess.run(command, cwd=ROOT, capture_output=True, check=False)
    if completed.returncode:
        sys.stdout.buffer.write(completed.stdout[-4000:])
        sys.stderr.buffer.write(completed.stderr[-4000:])
        raise AttestationError(f"command:{ordinal}:{gate}:{completed.returncode}")
    print(f"v19 clean attestation repetition-{ordinal}-{gate} pass", flush=True)
    return {
        "ordinal": ordinal,
        "gate": gate,
        "argv": command,
        "cwd": ".",
        "environment": environment_record(),
        "exit_status": 0,
        "stdout_sha256": sha(completed.stdout),
        "stderr_sha256": sha(completed.stderr),
        "output_sha256": sha(completed.stdout + b"\x00stderr\x00" + completed.stderr),
        "result": "pass",
    }


def create_report() -> dict[str, Any]:
    pre_status = git("status", "--porcelain=v1", "--untracked-files=all").stdout
    require(pre_status == b"", "dirty:pre_gate")
    execution_base = git("rev-parse", "HEAD").stdout.decode().strip()
    require(execution_base != TERMINAL, "ancestry:not_strict")
    require(git("merge-base", "--is-ancestor", TERMINAL, execution_base).returncode == 0, "ancestry:T_E")
    require(git("cat-file", "-e", f"{execution_base}:{REPORT_PATH}").returncode != 0, "cycle")
    execution_record = json.loads(committed(execution_base, EXECUTION_REPORT_PATH))
    require(execution_record["terminal_artifact_commit"] == TERMINAL, "execution_report:terminal")
    runs = [execute(ordinal, gate) for ordinal in (1, 2) for gate in ("standard", "conformance")]
    post_status = git("status", "--porcelain=v1", "--untracked-files=all").stdout
    require(post_status == b"", "dirty:post_gate")
    value = {
        "schema": "nostr_automerge.causal_projection_clean_candidate.v19.v1",
        "status": "code_complete_publication_held",
        "checkpoint": "post_rcld_146_clean_candidate_attestation",
        "terminal_artifact_commit": TERMINAL,
        "attestation_execution_base": {
            "candidate": execution_base,
            "tree": git("rev-parse", "HEAD^{tree}").stdout.decode().strip(),
            "parent_candidate": git("rev-parse", "HEAD^").stdout.decode().strip(),
            "runner_sha256": sha(committed(execution_base, RUNNER_PATH)),
            "schema_sha256": sha(committed(execution_base, SCHEMA_PATH)),
            "execution_base_report_sha256": sha(committed(execution_base, EXECUTION_REPORT_PATH)),
            "report_absent": True,
        },
        "observation": {
            "command": "git status --porcelain=v1 --untracked-files=all",
            "pre_gate_output_sha256": sha(pre_status),
            "post_gate_output_sha256": sha(post_status),
            "tracked_changes": 0,
            "staged_changes": 0,
            "untracked_paths": 0,
            "result": "clean",
        },
        "verification": {
            "execution_mode": "actual_twice_from_clean_E",
            "runs": runs,
            "aggregate": {
                "standard_runs": 2,
                "conformance_runs": 2,
                "conformance_processes": 4,
                "result": "pass",
            },
        },
        "terminal_artifacts": execution_record["terminal_artifacts"],
        "lifecycle": {
            "policy": "strict_T_E_A",
            "terminal_artifact_commit": TERMINAL,
            "attestation_execution_base": execution_base,
            "terminal_to_execution_base": "strict_descendant",
            "execution_base_to_attestation": "strict_descendant_required",
            "self_reference": False,
            "terminal_artifacts_mutated": False,
        },
        "holds": HOLDS,
        "release_claimed": False,
        "publication_claimed": False,
        "remote_actions": 0,
        "result": "pass",
        "result_identity_sha256": "",
    }
    value["result_identity_sha256"] = sha(canonical({key: value[key] for key in FIELDS[:-1]}))
    return value


def validate(record: dict[str, Any], schema: dict[str, Any]) -> None:
    require(list(record) == FIELDS, "shape")
    require(
        record["schema"] == "nostr_automerge.causal_projection_clean_candidate.v19.v1"
        and record["status"] == "code_complete_publication_held"
        and record["checkpoint"] == "post_rcld_146_clean_candidate_attestation"
        and record["terminal_artifact_commit"] == TERMINAL
        and record["result"] == "pass",
        "state",
    )
    base = record["attestation_execution_base"]
    require(re.fullmatch(r"[0-9a-f]{40}", base["candidate"]) is not None, "base:candidate")
    require(base["candidate"] != TERMINAL, "base:strict")
    require(git("merge-base", "--is-ancestor", TERMINAL, base["candidate"]).returncode == 0, "ancestry:T_E")
    require(base["tree"] == git("rev-parse", f"{base['candidate']}^{{tree}}").stdout.decode().strip(), "base:tree")
    require(base["parent_candidate"] == TERMINAL, "base:parent")
    require(base["runner_sha256"] == sha(committed(base["candidate"], RUNNER_PATH)) == sha((ROOT / RUNNER_PATH).read_bytes()), "base:runner")
    require(base["schema_sha256"] == sha(committed(base["candidate"], SCHEMA_PATH)) == sha(SCHEMA.read_bytes()), "base:schema")
    require(base["execution_base_report_sha256"] == sha(committed(base["candidate"], EXECUTION_REPORT_PATH)), "base:report")
    require(base["report_absent"] is True and git("cat-file", "-e", f"{base['candidate']}:{REPORT_PATH}").returncode != 0, "cycle")
    require(
        record["observation"] == {
            "command": "git status --porcelain=v1 --untracked-files=all",
            "pre_gate_output_sha256": EMPTY_SHA256,
            "post_gate_output_sha256": EMPTY_SHA256,
            "tracked_changes": 0,
            "staged_changes": 0,
            "untracked_paths": 0,
            "result": "clean",
        },
        "observation",
    )
    runs = record["verification"]["runs"]
    require([(row["ordinal"], row["gate"]) for row in runs] == [(1, "standard"), (1, "conformance"), (2, "standard"), (2, "conformance")], "runs:order")
    for row in runs:
        require(list(row) == RUN_FIELDS, "run:shape")
        require(row["argv"] == gate_command(row["gate"]), "run:argv")
        require(row["cwd"] == "." and row["environment"] == environment_record(), "run:context")
        require(row["exit_status"] == 0 and row["result"] == "pass", "run:result")
        require(all(re.fullmatch(r"[0-9a-f]{64}", row[name]) for name in ("stdout_sha256", "stderr_sha256", "output_sha256")), "run:hash")
    require(
        record["verification"]["execution_mode"] == "actual_twice_from_clean_E"
        and record["verification"]["aggregate"] == {
            "standard_runs": 2,
            "conformance_runs": 2,
            "conformance_processes": 4,
            "result": "pass",
        },
        "verification",
    )
    execution_record = json.loads(committed(base["candidate"], EXECUTION_REPORT_PATH))
    require(record["terminal_artifacts"] == execution_record["terminal_artifacts"], "artifacts:record")
    for artifact in record["terminal_artifacts"]:
        content = committed(TERMINAL, artifact["path"])
        require(sha(content) == artifact["sha256"], "artifact:terminal")
        require(git("diff", "--quiet", TERMINAL, "--", artifact["path"]).returncode == 0, "artifact:mutated")
    require(
        record["lifecycle"] == {
            "policy": "strict_T_E_A",
            "terminal_artifact_commit": TERMINAL,
            "attestation_execution_base": base["candidate"],
            "terminal_to_execution_base": "strict_descendant",
            "execution_base_to_attestation": "strict_descendant_required",
            "self_reference": False,
            "terminal_artifacts_mutated": False,
        },
        "lifecycle",
    )
    require(record["holds"] == HOLDS and record["release_claimed"] is False and record["publication_claimed"] is False and record["remote_actions"] == 0, "holds")
    require(schema.get("additionalProperties") is False and schema.get("required") == FIELDS and list(schema.get("properties", {})) == FIELDS, "schema")
    require(record["result_identity_sha256"] == sha(canonical({key: record[key] for key in FIELDS[:-1]})), "identity")
    head = git("rev-parse", "HEAD").stdout.decode().strip()
    require(git("merge-base", "--is-ancestor", base["candidate"], head).returncode == 0, "ancestry:E_A")
    if head == base["candidate"]:
        require(REPORT.is_file(), "attestation:bootstrap")
    else:
        require(git("status", "--porcelain=v1", "--untracked-files=all").stdout == b"", "attestation:current_clean")


def self_test(record: dict[str, Any], schema: dict[str, Any]) -> int:
    attacks: list[Callable[[dict[str, Any]], None]] = [
        lambda value: value.update(terminal_artifact_commit="0" * 40),
        lambda value: value["attestation_execution_base"].update(candidate=TERMINAL),
        lambda value: value["observation"].update(pre_gate_output_sha256="0" * 64),
        lambda value: value["verification"]["runs"].pop(),
        lambda value: value["verification"]["runs"][0].update(exit_status=1),
        lambda value: value["verification"]["aggregate"].update(standard_runs=1),
        lambda value: value["terminal_artifacts"].pop(),
        lambda value: value["terminal_artifacts"][0].update(sha256="0" * 64),
        lambda value: value["lifecycle"].update(self_reference=True),
        lambda value: value["lifecycle"].update(terminal_artifacts_mutated=True),
        lambda value: value["holds"].pop(),
        lambda value: value.update(publication_claimed=True),
        lambda value: value.update(remote_actions=1),
        lambda value: value.update(result_identity_sha256="0" * 64),
        lambda value: value.update(extra=False),
    ]
    caught = 0
    for attack in attacks:
        changed = copy.deepcopy(record)
        attack(changed)
        try:
            validate(changed, schema)
        except AttestationError:
            caught += 1
            continue
        raise AttestationError("attack:survived")
    return caught


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--write", action="store_true")
    args = parser.parse_args()
    if args.write:
        REPORT.write_text(json.dumps(create_report(), ensure_ascii=True, indent=2) + "\n")
    record, schema = load(REPORT), load(SCHEMA)
    validate(record, schema)
    attacks = self_test(record, schema)
    print(f"PASS: causal projection clean candidate v19 T={TERMINAL[:8]} E={record['attestation_execution_base']['candidate'][:8]} standard=2 conformance=2 attacks={attacks}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
