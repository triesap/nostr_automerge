#!/usr/bin/env python3
"""Validate the acyclic clean-candidate attestation for v17."""

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
REPORT = ROOT / "reports/causal_projection_clean_candidate_v17.json"
SCHEMA = ROOT / "tools/validation/causal_projection_clean_candidate_v17.schema.json"
CANDIDATE = "5599e7dc8a7658a3d7edbdd189599e69b57136f1"
TREE = "6a798683347d9100345a9e0c0c323ec70e2f64eb"
PARENT = "07479bee4fc75ac809e75588ca2bb568b35b38e4"
EMPTY_SHA256 = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
ARTIFACTS = [
    {"path": "docs/execution/rcl/nostr_automerge_v1_multi_rcld_v17.md", "sha256": "a5be949823024fc454a697982f1b363a12560ff2c76abd6841d1587f9e83d5bb"},
    {"path": "spec/remediation_v17_authority.json", "sha256": "bfe0e02ad8708b0869d76eb736dfde48a3894a63cfaac2a7fcbd054ce88807bd"},
    {"path": "spec/remediation_findings_v17.json", "sha256": "c00de5c6cfbf4ac768f77b3351d50bf4f0c283ff8ee3d665d36bafc8c06f3704"},
    {"path": "implementation/runtime_ledger_v17.json", "sha256": "9363f464367f5c930801dd3b168b436942de96eb0bb9498598fa6147ade8b16d"},
    {"path": "reports/causal_projection_completion_v17.json", "sha256": "1dc1801cc6cfe0d6e8de0c1d30238eb87edcc9f3093a561f17531d45c4aac8a4"},
    {"path": "reports/causal_projection_final_decision_v17.json", "sha256": "8711108bdecb857cd11962f414c48d2b77a51a2a1204c54c9cc958f5405efdb8"},
    {"path": "reports/spec_baseline.txt", "sha256": "a5b9974f496cc3f8a34640e6d644c60f6a16c169c4326d514c7c32b64963cd33"},
]
OBSERVATION = {
    "command": "git status --porcelain=v1",
    "output_sha256": EMPTY_SHA256,
    "tracked_changes": 0,
    "staged_changes": 0,
    "untracked_paths": 0,
    "terminal_commit_path_count": 17,
    "result": "clean",
}
LIFECYCLE = {
    "policy": "acyclic_later_attestation",
    "terminal_artifact_commit": CANDIDATE,
    "attestation_relationship": "strict_descendant",
    "self_reference": False,
    "terminal_artifacts_mutated": False,
}
HOLDS = [
    "external_assurance", "event_kind_allocation", "nip_submission", "production_qualification",
    "publication", "release", "remote_mutation",
]
FIELDS = [
    "schema", "status", "checkpoint", "candidate", "candidate_tree", "parent_candidate",
    "observation", "terminal_artifacts", "lifecycle", "holds", "release_claimed",
    "publication_claimed", "remote_actions", "result", "result_identity_sha256",
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
    return json.loads(path.read_text(), object_pairs_hook=closed)


def git(*args: str) -> subprocess.CompletedProcess[bytes]:
    return subprocess.run(["git", *args], cwd=ROOT, capture_output=True, check=False)


def canonical(value: Any) -> bytes:
    return json.dumps(value, ensure_ascii=False, sort_keys=True, separators=(",", ":")).encode()


def expected() -> dict[str, Any]:
    value = {
        "schema": "nostr_automerge.causal_projection_clean_candidate.v17.v1",
        "status": "code_complete_publication_held",
        "checkpoint": "post_step_1513_clean_candidate_attestation",
        "candidate": CANDIDATE,
        "candidate_tree": TREE,
        "parent_candidate": PARENT,
        "observation": OBSERVATION,
        "terminal_artifacts": ARTIFACTS,
        "lifecycle": LIFECYCLE,
        "holds": HOLDS,
        "release_claimed": False,
        "publication_claimed": False,
        "remote_actions": 0,
        "result": "pass",
        "result_identity_sha256": "",
    }
    value["result_identity_sha256"] = hashlib.sha256(canonical({key: value[key] for key in FIELDS[:-1]})).hexdigest()
    return value


def exact_record(schema: dict[str, Any], name: str, fields: list[str]) -> bool:
    value = schema["$defs"][name]
    return value.get("additionalProperties") is False and value.get("required") == fields and list(value.get("properties", {})) == fields


def validate(record: Any, schema: Any) -> None:
    require(type(record) is dict and list(record) == FIELDS and record == expected(), "record:value")
    require(git("rev-parse", CANDIDATE + "^{commit}").stdout.decode().strip() == CANDIDATE, "candidate:commit")
    require(git("rev-parse", CANDIDATE + "^{tree}").stdout.decode().strip() == TREE, "candidate:tree")
    require(git("rev-parse", CANDIDATE + "^").stdout.decode().strip() == PARENT, "candidate:parent")
    require(git("merge-base", "--is-ancestor", CANDIDATE, "HEAD").returncode == 0, "candidate:ancestor")
    changed = [line for line in git("diff-tree", "--no-commit-id", "--name-only", "-r", CANDIDATE).stdout.decode().splitlines() if line]
    require(len(changed) == OBSERVATION["terminal_commit_path_count"], "candidate:path_count")
    for artifact in ARTIFACTS:
        content = git("show", f"{CANDIDATE}:{artifact['path']}")
        require(content.returncode == 0 and hashlib.sha256(content.stdout).hexdigest() == artifact["sha256"], "candidate:artifact")
        require(artifact["path"] in changed, "candidate:changed_artifact")
    require(type(schema) is dict and schema.get("additionalProperties") is False and schema.get("required") == FIELDS and list(schema.get("properties", {})) == FIELDS, "schema:root")
    require(exact_record(schema, "observation", list(OBSERVATION)) and exact_record(schema, "artifact", ["path", "sha256"]) and exact_record(schema, "lifecycle", list(LIFECYCLE)), "schema:nested")


def self_test(record: dict[str, Any], schema: dict[str, Any]) -> int:
    attacks = [
        ("record", lambda value: value.update(candidate=PARENT)),
        ("record", lambda value: value.update(candidate_tree="0" * 40)),
        ("record", lambda value: value["observation"].update(output_sha256="0" * 64)),
        ("record", lambda value: value["observation"].update(untracked_paths=1)),
        ("record", lambda value: value["terminal_artifacts"][0].update(sha256="0" * 64)),
        ("record", lambda value: value["terminal_artifacts"].pop()),
        ("record", lambda value: value["lifecycle"].update(self_reference=True)),
        ("record", lambda value: value["lifecycle"].update(terminal_artifacts_mutated=True)),
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
        REPORT.write_text(json.dumps(expected(), ensure_ascii=True, indent=2) + "\n")
    record, schema = load(REPORT), load(SCHEMA)
    validate(record, schema)
    attacks = self_test(record, schema)
    print(f"PASS: causal projection clean candidate v17 candidate={CANDIDATE} artifacts=7 attacks={attacks}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
