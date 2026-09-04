#!/usr/bin/env python3
"""Validate the acyclic clean-candidate attestation for v18."""

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
REPORT = ROOT / "reports/causal_projection_clean_candidate_v18.json"
SCHEMA = ROOT / "tools/validation/causal_projection_clean_candidate_v18.schema.json"
CANDIDATE = "13083650829207c12e1fcd719251159816ab6833"
TREE = "94eba8825e33130473b9abc244416fa8c39ba3c4"
PARENT = "7150c33febcd0227484af4d95b2decf1c83ef6f8"
EMPTY_SHA256 = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
ARTIFACTS = [
    {
        "path": "docs/execution/rcl/nostr_automerge_v1_multi_rcld_v18.md",
        "sha256": "68be685d23c90641f58a3f3fa50c7b836b2febd2bf156fb9bb2c45958079b596",
    },
    {
        "path": "spec/remediation_v18_authority.json",
        "sha256": "c45b241c6a300ab3bb3a6120ba5fac3b53cb4a3589347ed4e1edda8b17923caa",
    },
    {
        "path": "spec/remediation_findings_v18.json",
        "sha256": "e7d39002597ae53c0cd1cd6a4247bd53903514682ccc709bdc11771faf51c76b",
    },
    {
        "path": "implementation/runtime_ledger_v18.json",
        "sha256": "1b8ad265fd9db38f809bec0b98869df44db6e1fc988a3bcbd65c6bb0e6b78e4f",
    },
    {
        "path": "reports/causal_projection_completion_v18.json",
        "sha256": "fabefed73f164db841dd587b239d374be20504c28d8c7da5aff9efa282a02e1a",
    },
    {
        "path": "reports/causal_projection_final_decision_v18.json",
        "sha256": "888f9161fc047997b186c7c1fce6cd7bb724d1b7cb8c1fc92010544da12a1fb6",
    },
]
OBSERVATION = {
    "command": "git status --porcelain=v1",
    "output_sha256": EMPTY_SHA256,
    "tracked_changes": 0,
    "staged_changes": 0,
    "untracked_paths": 0,
    "terminal_commit_path_count": 15,
    "result": "clean",
}
VERIFICATION = {
    "execution_mode": "actual_twice",
    "standard_command": "python3 scripts/local_gate.py standard",
    "conformance_command": "python3 scripts/local_gate.py conformance",
    "standard_runs": 2,
    "conformance_runs": 2,
    "conformance_processes_per_run": 2,
    "result": "pass",
}
LIFECYCLE = {
    "policy": "acyclic_later_attestation",
    "terminal_artifact_commit": CANDIDATE,
    "attestation_relationship": "strict_descendant",
    "self_reference": False,
    "terminal_artifacts_mutated": False,
}
HOLDS = [
    "external_assurance",
    "event_kind_allocation",
    "nip_submission",
    "production_qualification",
    "publication",
    "release",
    "remote_mutation",
]
FIELDS = [
    "schema",
    "status",
    "checkpoint",
    "candidate",
    "candidate_tree",
    "parent_candidate",
    "observation",
    "verification",
    "terminal_artifacts",
    "lifecycle",
    "holds",
    "release_claimed",
    "publication_claimed",
    "remote_actions",
    "result",
    "result_identity_sha256",
]


class AttestationError(RuntimeError):
    pass


def require(condition: bool, label: str) -> None:
    if not condition:
        raise AttestationError(label)


def load(path: Path) -> Any:
    def closed(pairs: list[tuple[str, Any]]) -> dict[str, Any]:
        keys = [key for key, _ in pairs]
        require(len(keys) == len(set(keys)), "duplicate:" + path.name)
        return dict(pairs)

    return json.loads(path.read_text(encoding="utf-8"), object_pairs_hook=closed)


def git(*args: str) -> subprocess.CompletedProcess[bytes]:
    return subprocess.run(["git", *args], cwd=ROOT, capture_output=True, check=False)


def canonical(value: Any) -> bytes:
    return json.dumps(
        value, ensure_ascii=False, sort_keys=True, separators=(",", ":")
    ).encode()


def expected() -> dict[str, Any]:
    value = {
        "schema": "nostr_automerge.causal_projection_clean_candidate.v18.v1",
        "status": "code_complete_publication_held",
        "checkpoint": "post_rcld_140_clean_candidate_attestation",
        "candidate": CANDIDATE,
        "candidate_tree": TREE,
        "parent_candidate": PARENT,
        "observation": OBSERVATION,
        "verification": VERIFICATION,
        "terminal_artifacts": ARTIFACTS,
        "lifecycle": LIFECYCLE,
        "holds": HOLDS,
        "release_claimed": False,
        "publication_claimed": False,
        "remote_actions": 0,
        "result": "pass",
        "result_identity_sha256": "",
    }
    value["result_identity_sha256"] = hashlib.sha256(
        canonical({key: value[key] for key in FIELDS[:-1]})
    ).hexdigest()
    return value


def exact_record(schema: dict[str, Any], name: str, fields: list[str]) -> bool:
    value = schema["$defs"][name]
    return (
        value.get("additionalProperties") is False
        and value.get("required") == fields
        and list(value.get("properties", {})) == fields
    )


def validate(record: Any, schema: Any) -> None:
    require(
        type(record) is dict and list(record) == FIELDS and record == expected(),
        "record:value",
    )
    require(
        git("rev-parse", CANDIDATE + "^{commit}").stdout.decode().strip()
        == CANDIDATE,
        "candidate:commit",
    )
    require(
        git("rev-parse", CANDIDATE + "^{tree}").stdout.decode().strip() == TREE,
        "candidate:tree",
    )
    require(
        git("rev-parse", CANDIDATE + "^").stdout.decode().strip() == PARENT,
        "candidate:parent",
    )
    head = git("rev-parse", "HEAD").stdout.decode().strip()
    require(
        git("merge-base", "--is-ancestor", CANDIDATE, head).returncode == 0,
        "candidate:ancestor",
    )
    if head == CANDIDATE:
        report_at_candidate = git(
            "cat-file", "-e", f"{CANDIDATE}:{REPORT.relative_to(ROOT).as_posix()}"
        )
        require(
            report_at_candidate.returncode != 0 and REPORT.is_file(),
            "candidate:bootstrap_only",
        )
    else:
        require(head != CANDIDATE, "candidate:strict_descendant")
    changed = [
        line
        for line in git(
            "diff-tree", "--no-commit-id", "--name-only", "-r", CANDIDATE
        )
        .stdout.decode()
        .splitlines()
        if line
    ]
    require(
        len(changed) == OBSERVATION["terminal_commit_path_count"],
        "candidate:path_count",
    )
    for artifact in ARTIFACTS:
        content = git("show", f"{CANDIDATE}:{artifact['path']}")
        require(
            content.returncode == 0
            and hashlib.sha256(content.stdout).hexdigest() == artifact["sha256"],
            "candidate:artifact",
        )
        require(artifact["path"] in changed, "candidate:changed_artifact")
        require(
            git("diff", "--quiet", CANDIDATE, "--", artifact["path"]).returncode
            == 0,
            "candidate:artifact_mutated",
        )
    require(
        type(schema) is dict
        and schema.get("additionalProperties") is False
        and schema.get("required") == FIELDS
        and list(schema.get("properties", {})) == FIELDS,
        "schema:root",
    )
    require(
        exact_record(schema, "observation", list(OBSERVATION))
        and exact_record(schema, "verification", list(VERIFICATION))
        and exact_record(schema, "artifact", ["path", "sha256"])
        and exact_record(schema, "lifecycle", list(LIFECYCLE)),
        "schema:nested",
    )


def self_test(record: dict[str, Any], schema: dict[str, Any]) -> int:
    attacks = [
        ("record", lambda value: value.update(candidate=PARENT)),
        ("record", lambda value: value.update(candidate_tree="0" * 40)),
        (
            "record",
            lambda value: value["observation"].update(output_sha256="0" * 64),
        ),
        ("record", lambda value: value["observation"].update(untracked_paths=1)),
        ("record", lambda value: value["verification"].update(standard_runs=1)),
        ("record", lambda value: value["verification"].update(result="fail")),
        (
            "record",
            lambda value: value["terminal_artifacts"][0].update(sha256="0" * 64),
        ),
        ("record", lambda value: value["terminal_artifacts"].pop()),
        ("record", lambda value: value["lifecycle"].update(self_reference=True)),
        (
            "record",
            lambda value: value["lifecycle"].update(terminal_artifacts_mutated=True),
        ),
        ("record", lambda value: value["holds"].pop()),
        ("record", lambda value: value.update(publication_claimed=True)),
        ("record", lambda value: value.update(remote_actions=1)),
        ("record", lambda value: value.update(result_identity_sha256="0" * 64)),
        ("schema", lambda value: value.update(additionalProperties=True)),
    ]
    caught = 0
    for target, mutate in attacks:
        changed_record, changed_schema = copy.deepcopy(record), copy.deepcopy(schema)
        mutate(changed_record if target == "record" else changed_schema)
        try:
            validate(changed_record, changed_schema)
        except AttestationError:
            caught += 1
            continue
        raise AttestationError("mutation:survived")
    return caught


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--write", action="store_true")
    args = parser.parse_args()
    if args.write:
        REPORT.write_text(
            json.dumps(expected(), ensure_ascii=True, indent=2) + "\n",
            encoding="utf-8",
        )
    record, schema = load(REPORT), load(SCHEMA)
    validate(record, schema)
    attacks = self_test(record, schema)
    print(
        "PASS: causal projection clean candidate v18 "
        f"candidate={CANDIDATE} artifacts=6 standard=2 conformance=2 attacks={attacks}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
