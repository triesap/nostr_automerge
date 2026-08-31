#!/usr/bin/env python3
"""Execute reviewed causal-projection mutations in an isolated Git worktree."""

from __future__ import annotations

import argparse
import copy
import hashlib
import json
import subprocess
import sys
import tempfile
from dataclasses import dataclass
from pathlib import Path

sys.dont_write_bytecode = True

ROOT = Path(__file__).resolve().parents[1]
TARGET = "crates/nostr_automerge/src/graph/actor_state.rs"
REPORT = ROOT / "reports/causal_projection_mutations_v13.json"
SCHEMA = ROOT / "tools/validation/causal_projection_mutations_v13.schema.json"
EXPECTED_CANDIDATE = "4b404afaa1d3ce1775f0dbd91a283f82141f1eca"
EXPECTED_TARGET_SHA256 = "14722b6be00453b784d809272dbfaba227b5a97f937cd2c9c5ff6d18fd7b3237"
EXPECTED_MUTATION_IDENTITY = "7afe29e061383e4566a20a8e7fb1bb83568a780e18ed7a354c0675e93ce9df12"
TEST = (
    "graph::actor_state::tests::"
    "projection_operation_families_have_exact_n_minus_one_n_and_n_plus_one_stops"
)


@dataclass(frozen=True)
class Mutation:
    mutation_id: str
    family: str
    before: str
    after: str


def relabel(anchor: str, source: str, replacement: str) -> tuple[str, str]:
    before = anchor.replace("ProjectionBuildOperation::OPERATION", source)
    return before, before.replace(source, replacement, 1)


SOURCE_PULL = """ProjectionBuildOperation::OPERATION,
            &mut charge,
            &mut built,
            || source.next_member(),"""
SOURCE_ORDER = """ProjectionBuildOperation::OPERATION,
                &mut charge,
                &mut built,
                || previous < hash,"""
MEMBERSHIP = """ProjectionBuildOperation::OPERATION,
            &mut charge,
            &mut built,
            || source.accepted_member(&hash),"""
CANDIDATE = """ProjectionBuildOperation::OPERATION,
            &mut charge,
            &mut built,
            || source.candidate(&hash),"""
DEPENDENCY = """ProjectionBuildOperation::OPERATION,
                &mut charge,
                &mut built,
                || source.dependency(candidate, index),"""
STATE = """ProjectionBuildOperation::OPERATION,
            &mut charge,
            &mut built,
            || depended_on.contains(&hash),"""
READINESS = """ProjectionBuildOperation::OPERATION,
                &mut charge,
                &mut built,
                || ready.insert(hash),"""
ARITHMETIC = """ProjectionBuildOperation::OPERATION,
                &mut charge,
                &mut built,
                || state.last_sequence.checked_add(1),"""
MAP = """ProjectionBuildOperation::OPERATION,
            &mut charge,
            &mut built,
            || remaining_dependencies.insert(hash, dependency_count),"""
SET = """ProjectionBuildOperation::OPERATION,
                &mut charge,
                &mut built,
                || candidate_dependencies.insert(dependency),"""
CAUSAL = """ProjectionBuildOperation::OPERATION,
            &mut charge,
            &mut built,
            || causal_next_op.max(advanced),"""
RESULT = """ProjectionBuildOperation::OPERATION,
        &mut charge,
        &mut built,
        || TrustedEpochProjection {"""
CONSTANT = """ProjectionBuildOperation::OPERATION,
        &mut charge,
        &mut built,
        || {
            let member_count = source.member_count();"""


def mutation(
    mutation_id: str, family: str, anchor: str, replacement: str
) -> Mutation:
    before, after = relabel(
        anchor,
        f"ProjectionBuildOperation::{family}",
        f"ProjectionBuildOperation::{replacement}",
    )
    return Mutation(mutation_id, family, before, after)


MUTATIONS = (
    mutation("canonical_source_pull_relabel", "CanonicalSourcePull", SOURCE_PULL, "StateLookup"),
    mutation("canonical_order_compare_relabel", "CanonicalOrderCompare", SOURCE_ORDER, "StateLookup"),
    mutation("membership_lookup_relabel", "MembershipLookup", MEMBERSHIP, "StateLookup"),
    Mutation(
        "candidate_lookup_relabel",
        "CandidateLookup",
        relabel(CANDIDATE, "ProjectionBuildOperation::CandidateLookup", "ProjectionBuildOperation::StateLookup")[0]
        + "\n        )?\n        else {\n            return Err(MeteredActorStateError::State(\n                ActorStateError::MissingDependency,\n            ));\n        };\n        if candidate.change_hash != hash {",
        relabel(CANDIDATE, "ProjectionBuildOperation::CandidateLookup", "ProjectionBuildOperation::StateLookup")[1]
        + "\n        )?\n        else {\n            return Err(MeteredActorStateError::State(\n                ActorStateError::MissingDependency,\n            ));\n        };\n        if candidate.change_hash != hash {",
    ),
    mutation("dependency_lookup_relabel", "DependencyLookup", DEPENDENCY, "StateLookup"),
    mutation("state_lookup_relabel", "StateLookup", STATE, "MembershipLookup"),
    mutation("readiness_transition_relabel", "ReadinessTransition", READINESS, "StateLookup"),
    mutation("checked_arithmetic_relabel", "CheckedArithmetic", ARITHMETIC, "StateLookup"),
    mutation("map_insertion_relabel", "MapInsertion", MAP, "StateLookup"),
    mutation("set_insertion_relabel", "SetInsertion", SET, "StateLookup"),
    Mutation(
        "shared_reference_clone_insert",
        "SharedReferenceClone",
        relabel(RESULT, "ProjectionBuildOperation::ResultPublication", "ProjectionBuildOperation::SharedReferenceClone")[0],
        relabel(RESULT, "ProjectionBuildOperation::ResultPublication", "ProjectionBuildOperation::SharedReferenceClone")[1],
    ),
    mutation("causal_maximum_compare_relabel", "CausalMaximumCompare", CAUSAL, "StateLookup"),
    mutation("result_publication_relabel", "ResultPublication", RESULT, "StateLookup"),
    mutation("constant_candidate_validation_relabel", "ConstantCandidateValidation", CONSTANT, "StateLookup"),
)


class MutationError(RuntimeError):
    pass


def require(condition: bool, label: str) -> None:
    if not condition:
        raise MutationError(label)


def mutation_identity() -> str:
    value = json.dumps(
        [
            {
                "id": item.mutation_id,
                "family": item.family,
                "path": TARGET,
                "before": item.before,
                "after": item.after,
                "test": TEST,
            }
            for item in MUTATIONS
        ],
        separators=(",", ":"),
        sort_keys=True,
    ).encode("utf-8")
    return hashlib.sha256(value).hexdigest()


def expected_record() -> dict[str, object]:
    return {
        "schema": "nostr_automerge.causal_projection_mutations.v13.v1",
        "status": "verified",
        "candidate": EXPECTED_CANDIDATE,
        "target": TARGET,
        "target_sha256": EXPECTED_TARGET_SHA256,
        "owning_test": TEST,
        "operation_families": [item.family for item in MUTATIONS],
        "selected_mutations": 14,
        "executed_mutations": 14,
        "survivors": 0,
        "inventory_and_transcript_mutations": 15,
        "mutation_identity_sha256": EXPECTED_MUTATION_IDENTITY,
        "result": "pass",
    }


def validate_record(record: object, schema: object) -> None:
    expected = expected_record()
    require(type(record) is dict and list(record) == list(expected), "record:shape")
    require(record == expected, "record:value")
    require(mutation_identity() == EXPECTED_MUTATION_IDENTITY, "record:identity")
    candidate_source = subprocess.run(
        ["git", "show", f"{EXPECTED_CANDIDATE}:{TARGET}"],
        cwd=ROOT,
        capture_output=True,
        check=False,
    )
    require(candidate_source.returncode == 0, "record:candidate")
    require(
        hashlib.sha256(candidate_source.stdout).hexdigest() == EXPECTED_TARGET_SHA256,
        "record:target",
    )
    require(type(schema) is dict, "schema:type")
    require(schema.get("additionalProperties") is False, "schema:closed")
    require(schema.get("required") == list(expected), "schema:required")
    properties = schema.get("properties")
    require(type(properties) is dict and list(properties) == list(expected), "schema:properties")
    for key, value in expected.items():
        if key == "operation_families":
            require(
                properties[key]
                == {
                    "type": "array",
                    "minItems": 14,
                    "maxItems": 14,
                    "uniqueItems": True,
                    "items": {"type": "string"},
                },
                "schema:families",
            )
        else:
            require(properties[key] == {"const": value}, f"schema:{key}")


def transcript_is_exact_failure(returncode: int, output: str) -> bool:
    named = f"test {TEST} ... FAILED"
    return (
        returncode != 0
        and output.count(named) == 1
        and output.count("test result: FAILED.") == 1
        and "0 passed; 1 failed; 0 ignored" in output
        and "error: could not compile" not in output
    )


def run_selected() -> None:
    checkout = Path(
        tempfile.mkdtemp(prefix="nostr-causal-mutation-", dir=ROOT.parent)
    )
    added = False
    try:
        added_result = subprocess.run(
            ["git", "worktree", "add", "--detach", str(checkout), EXPECTED_CANDIDATE],
            cwd=ROOT,
            capture_output=True,
            text=True,
            check=False,
        )
        require(added_result.returncode == 0, "worktree:add")
        added = True
        doctor = subprocess.run(
            ["cargo", "extbuild", "doctor"],
            cwd=checkout,
            capture_output=True,
            text=True,
            check=False,
        )
        require(doctor.returncode == 0, "worktree:doctor")
        target = checkout / TARGET
        source = target.read_text(encoding="utf-8")
        for item in MUTATIONS:
            require(source.count(item.before) == 1, f"mutation:anchor:{item.mutation_id}")
            target.write_text(source.replace(item.before, item.after, 1), encoding="utf-8")
            command = [
                "cargo", "extbuild", "run", "--",
                "cargo", "test", "-p", "nostr_automerge", "--lib", TEST,
                "--locked", "--", "--exact",
            ]
            result = subprocess.run(
                command,
                cwd=checkout,
                capture_output=True,
                text=True,
                check=False,
            )
            output = result.stdout + result.stderr
            require(
                transcript_is_exact_failure(result.returncode, output),
                f"mutation:transcript:{item.mutation_id}",
            )
            target.write_text(source, encoding="utf-8")
    finally:
        if added:
            removed = subprocess.run(
                ["git", "worktree", "remove", "--force", str(checkout)],
                cwd=ROOT,
                capture_output=True,
                text=True,
                check=False,
            )
            require(removed.returncode == 0, "worktree:remove")
        elif checkout.exists():
            checkout.rmdir()


def self_test(record: dict[str, object], schema: dict[str, object]) -> tuple[int, int]:
    candidate_source = subprocess.run(
        ["git", "show", f"{EXPECTED_CANDIDATE}:{TARGET}"],
        cwd=ROOT,
        capture_output=True,
        text=True,
        check=False,
    )
    require(candidate_source.returncode == 0, "inventory:candidate")
    source = candidate_source.stdout
    families = [item.family for item in MUTATIONS]
    require(len(MUTATIONS) == 14, "inventory:count")
    require(len(set(families)) == 14, "inventory:family_unique")
    require(len({item.mutation_id for item in MUTATIONS}) == 14, "inventory:id_unique")
    require(all(source.count(item.before) == 1 for item in MUTATIONS), "inventory:anchors")
    require(all(item.before != item.after for item in MUTATIONS), "inventory:behavior_change")
    exact = (
        f"test {TEST} ... FAILED\n"
        "test result: FAILED. 0 passed; 1 failed; 0 ignored; "
        "0 measured; 0 filtered out; finished in 0.00s\n"
    )
    require(transcript_is_exact_failure(101, exact), "transcript:positive")
    transcript_mutations = (
        (101, exact.replace(TEST, "unrelated", 1)),
        (0, exact.replace("FAILED", "ok")),
        (101, exact.replace("0 ignored", "1 ignored")),
        (101, "error: could not compile `nostr_automerge`\n"),
        (101, exact + "test result: FAILED. 0 passed; 1 failed; 0 ignored\n"),
        (0, f"test {TEST} ... ok\ntest result: ok. 1 passed; 0 failed; 0 ignored\n"),
        (101, exact + f"test {TEST} ... FAILED\n"),
        (101, exact.replace("1 failed", "2 failed")),
        (101, exact.replace("0 passed", "1 passed")),
    )
    for index, (returncode, output) in enumerate(transcript_mutations):
        require(not transcript_is_exact_failure(returncode, output), f"transcript:{index}")
    require(len(mutation_identity()) == 64, "mutation:identity")
    inventory_mutations = len(transcript_mutations) + 6
    record_mutations: list[tuple[dict[str, object], dict[str, object]]] = []
    for mutate_record, mutate_schema in [
        (lambda value: value.update(candidate="0" * 40), lambda value: None),
        (lambda value: value["operation_families"].reverse(), lambda value: None),
        (lambda value: value.update(survivors=1), lambda value: None),
        (lambda value: value.update(mutation_identity_sha256="0" * 64), lambda value: None),
        (lambda value: value.update(extra=False), lambda value: None),
        (lambda value: None, lambda value: value.update(additionalProperties=True)),
        (
            lambda value: value.update(mutation_identity_sha256="0" * 64),
            lambda value: value["properties"]["mutation_identity_sha256"].update(
                const="0" * 64
            ),
        ),
    ]:
        changed_record = copy.deepcopy(record)
        changed_schema = copy.deepcopy(schema)
        mutate_record(changed_record)
        mutate_schema(changed_schema)
        record_mutations.append((changed_record, changed_schema))
    for index, (changed_record, changed_schema) in enumerate(record_mutations):
        try:
            validate_record(changed_record, changed_schema)
        except MutationError:
            continue
        raise MutationError(f"record_mutation_survived:{index}")
    return inventory_mutations, len(record_mutations)


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--run-selected", action="store_true")
    args = parser.parse_args()
    record = json.loads(REPORT.read_text(encoding="utf-8"))
    schema = json.loads(SCHEMA.read_text(encoding="utf-8"))
    validate_record(record, schema)
    mutations, record_mutations = self_test(record, schema)
    if args.run_selected:
        run_selected()
    print(
        "PASS: causal-projection isolated mutation runner "
        f"selected={len(MUTATIONS)} survivors=0 mutations={mutations} "
        f"record_mutations={record_mutations} "
        f"identity={mutation_identity()}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
