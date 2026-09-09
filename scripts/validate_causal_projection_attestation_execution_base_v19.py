#!/usr/bin/env python3
"""Validate the non-self-referential v19 attestation execution base."""

from __future__ import annotations

import copy
import hashlib
import json
import subprocess
import sys
from pathlib import Path
from typing import Any, Callable

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parents[1]
REPORT_PATH = "reports/causal_projection_attestation_execution_base_v19.json"
REPORT = ROOT / REPORT_PATH
SCHEMA = ROOT / "tools/validation/causal_projection_attestation_execution_base_v19.schema.json"
TERMINAL = "5fc3fb38ede64397684b699f364c29f07bc23559"
ARTIFACTS = [
    {"path": "spec/remediation_v19_authority.json", "sha256": "f92c5a9c92fc5967d6060ad420009dd5c690f0d61f0264c7cc8f4cf3220066dd"},
    {"path": "spec/remediation_findings_v19.json", "sha256": "523e5e9e97bae869e8978e4023ca4f5b9804e49eedd114c7d9a88972e0645f4f"},
    {"path": "implementation/runtime_ledger_v19.json", "sha256": "3015c23ea5c74f4c804562d6b81a4cf721950b6d8b2d3912c1eee93274917130"},
    {"path": "reports/causal_projection_combined_assurance_v19.json", "sha256": "091df28a6ae35ccce16ec2b17436d7009b6e26b5df7f47064fab324090102f7f"},
    {"path": "reports/causal_projection_finding_closure_v19.json", "sha256": "0384a03b16ca09d79a34f033adfc7a803d8b0eb2ede64522e029d6fd17878d62"},
    {"path": "reports/causal_projection_completion_v19.json", "sha256": "17f7d6de58e26249be74513e1b14b9b2717aeee4435ca4f90394a9a423dfd0eb"},
    {"path": "reports/causal_projection_final_decision_v19.json", "sha256": "dd583d80992af03d50f8caae472662fb6affeb3a03bccb8aae91568630e23f92"},
]
LIFECYCLE = {
    "policy": "strict_T_E_A",
    "required_relationship": "strict_descendant",
    "terminal_self_reference": False,
    "execution_base_self_reference": False,
    "terminal_artifacts_mutated": False,
}
HOLDS = [
    "external_assurance", "event_kind_allocation", "nip_submission",
    "production_qualification", "publication", "release", "deployment",
    "remote_mutation",
]
FIELDS = [
    "schema", "status", "checkpoint", "terminal_artifact_commit",
    "terminal_artifacts", "attestation_execution_base", "lifecycle", "holds",
    "release_claimed", "publication_claimed", "remote_actions", "result",
    "result_identity_sha256",
]


class ExecutionBaseError(RuntimeError):
    pass


def require(value: bool, code: str) -> None:
    if not value:
        raise ExecutionBaseError(code)


def git(*args: str) -> subprocess.CompletedProcess[bytes]:
    return subprocess.run(["git", *args], cwd=ROOT, capture_output=True, check=False)


def load(path: Path) -> Any:
    def closed(pairs: list[tuple[str, Any]]) -> dict[str, Any]:
        keys = [key for key, _ in pairs]
        require(len(keys) == len(set(keys)), "duplicate:" + path.name)
        return dict(pairs)

    return json.loads(path.read_text(), object_pairs_hook=closed)


def canonical(value: Any) -> bytes:
    return json.dumps(
        value, ensure_ascii=False, sort_keys=True, separators=(",", ":")
    ).encode()


def validate(record: dict[str, Any], schema: dict[str, Any]) -> None:
    require(list(record) == FIELDS, "shape")
    require(
        record["schema"]
        == "nostr_automerge.causal_projection_attestation_execution_base.v19.v1"
        and record["status"] == "prepared"
        and record["checkpoint"] == "rcld_146_execution_base"
        and record["terminal_artifact_commit"] == TERMINAL
        and record["result"] == "pass",
        "state",
    )
    require(record["terminal_artifacts"] == ARTIFACTS, "artifacts")
    require(record["attestation_execution_base"] is None, "self_reference")
    require(record["lifecycle"] == LIFECYCLE, "lifecycle")
    require(
        record["holds"] == HOLDS
        and record["release_claimed"] is False
        and record["publication_claimed"] is False
        and record["remote_actions"] == 0,
        "holds",
    )
    require(git("rev-parse", TERMINAL + "^{commit}").stdout.decode().strip() == TERMINAL, "terminal")
    head = git("rev-parse", "HEAD").stdout.decode().strip()
    require(git("merge-base", "--is-ancestor", TERMINAL, head).returncode == 0, "ancestry")
    require(git("cat-file", "-e", f"{TERMINAL}:{REPORT_PATH}").returncode != 0, "cycle")
    if head != TERMINAL:
        require(git("cat-file", "-e", f"{head}:{REPORT_PATH}").returncode == 0, "execution_base:committed")
    else:
        require(REPORT.is_file(), "execution_base:bootstrap")
    for artifact in ARTIFACTS:
        content = git("show", f"{TERMINAL}:{artifact['path']}")
        require(
            content.returncode == 0
            and hashlib.sha256(content.stdout).hexdigest() == artifact["sha256"],
            "artifact:" + artifact["path"],
        )
        require(git("diff", "--quiet", TERMINAL, "--", artifact["path"]).returncode == 0, "mutated:" + artifact["path"])
    require(
        schema.get("additionalProperties") is False
        and schema.get("required") == FIELDS
        and list(schema.get("properties", {})) == FIELDS,
        "schema",
    )
    require(
        record["result_identity_sha256"]
        == hashlib.sha256(canonical({key: record[key] for key in FIELDS[:-1]})).hexdigest(),
        "identity",
    )


def self_test(record: dict[str, Any], schema: dict[str, Any]) -> int:
    attacks: list[Callable[[dict[str, Any]], None]] = [
        lambda value: value.update(terminal_artifact_commit="0" * 40),
        lambda value: value["terminal_artifacts"].pop(),
        lambda value: value["terminal_artifacts"][0].update(sha256="0" * 64),
        lambda value: value.update(attestation_execution_base=TERMINAL),
        lambda value: value["lifecycle"].update(terminal_self_reference=True),
        lambda value: value["lifecycle"].update(execution_base_self_reference=True),
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
        except ExecutionBaseError:
            caught += 1
            continue
        raise ExecutionBaseError("attack:survived")
    return caught


def main() -> int:
    record, schema = load(REPORT), load(SCHEMA)
    validate(record, schema)
    attacks = self_test(record, schema)
    print(f"PASS: causal projection v19 execution base terminal={TERMINAL[:8]} artifacts=7 attacks={attacks}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
