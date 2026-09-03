#!/usr/bin/env python3
"""Run property-specific v16 causal-projection mutations in an isolated tree."""

from __future__ import annotations

import argparse
import copy
import difflib
import hashlib
import json
import re
import shlex
import subprocess
import sys
import tempfile
from dataclasses import dataclass
from pathlib import Path
from typing import Callable

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parents[1]
ACTOR = "crates/nostr_automerge/src/graph/actor_state.rs"
CONSUMER = "crates/nostr_automerge/src/reference/epoch_engine.rs"
TARGETS = [ACTOR, CONSUMER]
INVENTORY = ROOT / "reports/causal_projection_operation_inventory_v16.json"
REPORT = ROOT / "reports/causal_projection_mutations_v16.json"
SCHEMA = ROOT / "tools/validation/causal_projection_mutations_v16.schema.json"
CANDIDATE = "a696e41dbc6eb966b3657a47331f1ed072308a0b"
COMPILE_COMMAND = "cargo check -p nostr_automerge --lib --locked"
STRUCTURAL_COMMAND = "python3 scripts/validate_causal_projection_structural_assurance_v16.py --mode structural"
BEHAVIOR_TEST = "graph::actor_state::tests::projection_causal_maximum_is_charged_once_per_accepted_change"
BEHAVIOR_COMMAND = f"cargo test -p nostr_automerge --lib {BEHAVIOR_TEST} --locked -- --exact"
TOP_FIELDS = [
    "schema", "status", "candidate", "targets", "mutation_count",
    "compile_failures", "identity_only_rejections", "survivors",
    "retained_v15_classes", "mutations", "mutation_identity_sha256", "result",
]
ROW_FIELDS = [
    "mutation_id", "mutation_class", "source_site", "row_id", "patch_sha256",
    "command", "compile_result", "expected_property_code", "actual_property_code",
    "transcript_sha256", "result",
]


class MutationError(RuntimeError):
    pass


def require(condition: bool, label: str) -> None:
    if not condition:
        raise MutationError(label)


def canonical(value: object) -> bytes:
    return json.dumps(value, ensure_ascii=True, separators=(",", ":"), sort_keys=True).encode()


def sha(value: bytes) -> str:
    return hashlib.sha256(value).hexdigest()


def replace_once(value: str, old: str, new: str, label: str) -> str:
    require(value.count(old) == 1, "anchor:" + label)
    return value.replace(old, new, 1)


def patch_sha(path: str, before: str, after: str) -> str:
    patch = "".join(
        difflib.unified_diff(
            before.splitlines(keepends=True),
            after.splitlines(keepends=True),
            fromfile="a/" + path,
            tofile="b/" + path,
        )
    ).encode()
    require(bool(patch), "patch:empty")
    return sha(patch)


@dataclass(frozen=True)
class Mutation:
    mutation_id: str
    mutation_class: str
    target: str
    source_site: str
    row_id: str
    expected_property_code: str
    command: str
    mode: str
    transform: Callable[[str], str]


def actor_mutations() -> list[Mutation]:
    def replacement(mutation_id: str, old: str, new: str) -> Callable[[str], str]:
        return lambda value: replace_once(value, old, new, mutation_id)

    return [
        Mutation(
            "actor_identity_charge_removed", "actor_sequence", ACTOR,
            "actor_sequence_decision_metered_observed:ActorIdentityDecision",
            "rust.actor_sequence.actor_identity_decision.01",
            "UNWRAPPED_ACTOR_SEQUENCE_DECISION", STRUCTURAL_COMMAND, "structural",
            replacement(
                "actor_identity_charge_removed",
                "        charge(WorkCounter::GraphNode).map_err(MeteredActorStateError::Work)?;\n        let actor_relation =",
                "        let actor_relation =",
            ),
        ),
        Mutation(
            "causal_stage_before_actor_success", "stage_order", ACTOR,
            "candidate_semantics_decision_metered_observed:ActorSequence->CausalCounter",
            "rust.causal_counter_consumer.stored_counter_read.01",
            "CAUSAL_STAGE_BEFORE_ACTOR_SUCCESS", STRUCTURAL_COMMAND, "structural",
            replacement(
                "causal_stage_before_actor_success",
                "        self.actor_sequence_decision_metered(candidate, &mut charge)?;\n        observed(CandidateSemanticStage::ActorSequence);\n        self.causal_next_decision_metered(candidate, &mut charge)?;",
                "        self.causal_next_decision_metered(candidate, &mut charge)?;\n        observed(CandidateSemanticStage::ActorSequence);\n        self.actor_sequence_decision_metered(candidate, &mut charge)?;",
            ),
        ),
        Mutation(
            "duplicate_causal_start_comparison", "causal_comparison", ACTOR,
            "causal_next_decision_metered_observed:ExpectedStartComparison",
            "rust.causal_counter_consumer.expected_start_comparison.01",
            "DUPLICATE_CAUSAL_START_COMPARISON", STRUCTURAL_COMMAND, "structural",
            replacement(
                "duplicate_causal_start_comparison",
                "candidate.start_op == causal_next_op",
                "candidate.start_op == causal_next_op && candidate.start_op == causal_next_op",
            ),
        ),
        Mutation(
            "final_state_scan_restored", "unmetered_final_traversal", ACTOR,
            "build_trusted_epoch_projection_observed:CompletionComparison",
            "rust.projection_construction.completion_comparison.01",
            "UNMETERED_FINAL_TRAVERSAL", STRUCTURAL_COMMAND, "structural",
            replacement(
                "final_state_scan_restored",
                "    let is_complete = perform_projection_build_operation(",
                "    for state in states.values() { causal_next_op = causal_next_op.max(state.next_op); }\n    let is_complete = perform_projection_build_operation(",
            ),
        ),
        Mutation(
            "remaining_state_write_before_charge", "state_write", ACTOR,
            "perform_projection_build_operation:RemainingStateWrite#1",
            "rust.projection_construction.remaining_state_write.01",
            "STATE_WRITE_BEFORE_CHARGE", STRUCTURAL_COMMAND, "structural",
            replacement(
                "remaining_state_write_before_charge",
                "                perform_projection_build_operation(\n                    WorkCounter::GraphEdge,\n                    ProjectionBuildOperation::RemainingStateWrite,",
                "                *remaining = updated_remaining;\n                perform_projection_build_operation(\n                    WorkCounter::GraphEdge,\n                    ProjectionBuildOperation::RemainingStateWrite,",
            ),
        ),
        Mutation(
            "charge_after_operation", "charge_order", ACTOR,
            "perform_projection_build_operation:SourceCountRead#1",
            "rust.projection_construction.source_count_read.01",
            "CHARGE_AFTER_OPERATION", STRUCTURAL_COMMAND, "structural",
            replacement(
                "charge_after_operation",
                "    charge(counter).map_err(MeteredActorStateError::Work)?;\n    let result = perform();",
                "    let result = perform();\n    charge(counter).map_err(MeteredActorStateError::Work)?;",
            ),
        ),
        Mutation(
            "post_stop_target_work", "post_stop", ACTOR,
            "perform_projection_build_operation:SourceCountRead#1",
            "rust.projection_construction.source_count_read.01",
            "POST_STOP_TARGET_WORK", STRUCTURAL_COMMAND, "structural",
            replacement(
                "post_stop_target_work",
                "    charge(counter).map_err(MeteredActorStateError::Work)?;\n    let result = perform();",
                "    let charged = charge(counter).map_err(MeteredActorStateError::Work);\n    let result = perform();\n    charged?;",
            ),
        ),
        Mutation(
            "typed_stop_collapsed", "typed_stop", ACTOR,
            "perform_projection_build_operation:SourceCountRead#1",
            "rust.projection_construction.source_count_read.01",
            "POST_STOP_TARGET_WORK", STRUCTURAL_COMMAND, "structural",
            replacement(
                "typed_stop_collapsed",
                "    charge(counter).map_err(MeteredActorStateError::Work)?;\n    let result = perform();",
                "    charge(counter).map_err(|_| MeteredActorStateError::State(ActorStateError::DependencyCycle))?;\n    let result = perform();",
            ),
        ),
        Mutation(
            "publication_before_charge", "publication", ACTOR,
            "perform_projection_build_operation:ResultPublication#1",
            "rust.projection_construction.result_publication.01",
            "PUBLICATION_BEFORE_CHARGE", STRUCTURAL_COMMAND, "structural",
            replacement(
                "publication_before_charge",
                "    let member_count = perform_projection_build_operation(",
                "    published(ProjectionPublicationOperation::Projection);\n    let member_count = perform_projection_build_operation(",
            ),
        ),
        Mutation(
            "dependency_counter_changed", "counter", ACTOR,
            "perform_projection_build_operation:DependencyCountRead#1",
            "rust.projection_construction.dependency_count_read.01",
            "COUNTER_MISMATCH", STRUCTURAL_COMMAND, "structural",
            replacement(
                "dependency_counter_changed",
                "WorkCounter::GraphNode,\n            ProjectionBuildOperation::DependencyCountRead",
                "WorkCounter::GraphEdge,\n            ProjectionBuildOperation::DependencyCountRead",
            ),
        ),
        Mutation(
            "double_source_pull_after_one_charge", "traversal", ACTOR,
            "perform_projection_build_operation:CanonicalSourcePull#1",
            "rust.projection_construction.canonical_source_pull.01",
            "UNMETERED_FINAL_TRAVERSAL", STRUCTURAL_COMMAND, "structural",
            replacement(
                "double_source_pull_after_one_charge",
                "|| source.next_member(),",
                "|| { let member = source.next_member(); let _ = source.next_member(); member },",
            ),
        ),
        Mutation(
            "causal_maximum_changed_to_minimum", "retained_v15_semantics", ACTOR,
            "perform_projection_build_operation:CausalMaximumCompare#1",
            "rust.projection_construction.causal_maximum_compare.01",
            "CAUSAL_MAXIMUM_SEMANTICS", BEHAVIOR_COMMAND, "behavior",
            replacement(
                "causal_maximum_changed_to_minimum",
                "|| causal_next_op.max(advanced),",
                "|| causal_next_op.min(advanced),",
            ),
        ),
    ]


def all_mutations() -> list[Mutation]:
    values = actor_mutations()
    marker = "Ok(projection) => match projection.candidate_semantics_decision_metered(\n"
    replacement = (
        "Ok(projection) => match {\n"
        "                    let _ = projection.actor_sequence_decision_metered(\n"
        "                    &candidate,\n"
        "                    |counter| charge_epoch_item(counter, budget, cancellation),\n"
        "                    );\n"
        "                    projection\n"
        "                }.candidate_semantics_decision_metered(\n"
    )
    values.insert(
        9,
        Mutation(
            "alternate_consumer_bypass", "consumer_bypass", CONSUMER,
            "evaluate_epoch:candidate_semantics_decision_metered",
            "rust.causal_counter_consumer.stored_counter_read.01",
            "ALTERNATE_CONSUMER_BYPASS", STRUCTURAL_COMMAND, "structural",
            lambda source: replace_once(source, marker, replacement, "alternate_consumer_bypass"),
        ),
    )
    return values


def source_at_candidate(path: str) -> str:
    result = subprocess.run(
        ["git", "show", f"{CANDIDATE}:{path}"],
        cwd=ROOT,
        capture_output=True,
        text=True,
        check=False,
    )
    require(result.returncode == 0, "candidate_source:" + path)
    return result.stdout


def normalized_transcript(item: Mutation) -> dict[str, object]:
    return {
        "compile": {"command": COMPILE_COMMAND, "result": "pass"},
        "property": {"command": item.command, "returncode": "nonzero", "code": item.expected_property_code},
        "restoration": "clean",
    }


def expected_rows(sources: dict[str, str]) -> list[dict[str, object]]:
    rows: list[dict[str, object]] = []
    for item in all_mutations():
        changed = item.transform(sources[item.target])
        transcript = normalized_transcript(item)
        rows.append(
            {
                "mutation_id": item.mutation_id,
                "mutation_class": item.mutation_class,
                "source_site": item.source_site,
                "row_id": item.row_id,
                "patch_sha256": patch_sha(item.target, sources[item.target], changed),
                "command": item.command,
                "compile_result": "pass",
                "expected_property_code": item.expected_property_code,
                "actual_property_code": item.expected_property_code,
                "transcript_sha256": sha(canonical(transcript)),
                "result": "killed",
            }
        )
    return rows


def expected_report(sources: dict[str, str]) -> dict[str, object]:
    rows = expected_rows(sources)
    value: dict[str, object] = {
        "schema": "nostr_automerge.causal_projection_mutations.v16.v1",
        "status": "pass",
        "candidate": CANDIDATE,
        "targets": TARGETS,
        "mutation_count": len(rows),
        "compile_failures": 0,
        "identity_only_rejections": 1,
        "survivors": 0,
        "retained_v15_classes": [
            "charge_order", "unmetered_final_traversal", "state_write",
            "typed_stop", "post_stop", "publication", "traversal",
            "retained_v15_semantics",
        ],
        "mutations": rows,
        "mutation_identity_sha256": "",
        "result": "pass",
    }
    projection = {key: item for key, item in value.items() if key != "mutation_identity_sha256"}
    value["mutation_identity_sha256"] = sha(canonical(projection))
    return value


def validate(report: object, schema: object, sources: dict[str, str], inventory: dict[str, object]) -> None:
    expected = expected_report(sources)
    require(type(report) is dict and list(report) == TOP_FIELDS and report == expected, "report:value")
    assert isinstance(report, dict)
    require(report["mutation_count"] == 13 and report["compile_failures"] == report["survivors"] == 0, "report:counts")
    rows = report["mutations"]
    require(type(rows) is list and len({row["mutation_id"] for row in rows}) == 13, "report:unique")
    inventory_ids = {row["id"] for row in inventory["rows"]}
    require(all(row["row_id"] in inventory_ids for row in rows), "report:row_binding")
    require(
        {row["expected_property_code"] for row in rows}
        >= {
            "UNWRAPPED_ACTOR_SEQUENCE_DECISION", "CAUSAL_STAGE_BEFORE_ACTOR_SUCCESS",
            "DUPLICATE_CAUSAL_START_COMPARISON", "UNMETERED_FINAL_TRAVERSAL",
            "STATE_WRITE_BEFORE_CHARGE", "CHARGE_AFTER_OPERATION",
            "POST_STOP_TARGET_WORK", "PUBLICATION_BEFORE_CHARGE",
            "ALTERNATE_CONSUMER_BYPASS", "COUNTER_MISMATCH",
        },
        "report:properties",
    )
    resolved = subprocess.run(["git", "rev-parse", f"{CANDIDATE}^{{commit}}"], cwd=ROOT, capture_output=True, text=True, check=False)
    require(resolved.returncode == 0 and resolved.stdout.strip() == CANDIDATE, "candidate:identity")
    require(type(schema) is dict and schema.get("additionalProperties") is False and schema.get("required") == TOP_FIELDS, "schema:closed")
    require(schema["$defs"]["mutation"]["required"] == ROW_FIELDS, "schema:row")


def property_code(item: Mutation, result: subprocess.CompletedProcess[str]) -> str | None:
    output = result.stdout + result.stderr
    if item.mode == "structural":
        match = re.findall(r"FAIL: causal projection structural assurance v16 code=([A-Z0-9_]+)", output)
        return match[0] if result.returncode != 0 and len(match) == 1 else None
    exact = f"test {BEHAVIOR_TEST} ... FAILED"
    if result.returncode != 0 and output.count(exact) == 1 and "0 passed; 1 failed; 0 ignored" in output and "could not compile" not in output:
        return "CAUSAL_MAXIMUM_SEMANTICS"
    return None


def run_extbuild(checkout: Path, command: str) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        ["cargo", "extbuild", "run", "--", *shlex.split(command)],
        cwd=checkout,
        capture_output=True,
        text=True,
        check=False,
    )


def run_campaign(report: dict[str, object], sources: dict[str, str]) -> None:
    checkout = Path(tempfile.mkdtemp(prefix="nostr-causal-v16-mutation-", dir=ROOT.parent))
    added = False
    try:
        add = subprocess.run(["git", "worktree", "add", "--detach", str(checkout), CANDIDATE], cwd=ROOT, capture_output=True, text=True, check=False)
        require(add.returncode == 0, "worktree:add")
        added = True
        doctor = subprocess.run(["cargo", "extbuild", "doctor"], cwd=checkout, capture_output=True, text=True, check=False)
        require(doctor.returncode == 0, "worktree:doctor")
        items = all_mutations()
        rows = report["mutations"]
        assert isinstance(rows, list)
        require([item.mutation_id for item in items] == [row["mutation_id"] for row in rows], "campaign:order")
        for item, row in zip(items, rows, strict=True):
            target = checkout / item.target
            changed = item.transform(sources[item.target])
            target.write_text(changed)
            compile_result = run_extbuild(checkout, COMPILE_COMMAND)
            require(compile_result.returncode == 0, "compile:" + item.mutation_id)
            if item.mode == "structural":
                command = subprocess.run(
                    [
                        "cargo", "extbuild", "run", "--", "python3",
                        str(ROOT / "scripts/validate_causal_projection_structural_assurance_v16.py"),
                        "--mode", "structural", "--source-root", str(checkout),
                    ],
                    cwd=checkout,
                    capture_output=True,
                    text=True,
                    check=False,
                )
            else:
                command = run_extbuild(checkout, item.command)
            actual = property_code(item, command)
            require(actual == item.expected_property_code, "property:" + item.mutation_id + ":" + str(actual))
            transcript = {
                "compile": {"command": COMPILE_COMMAND, "result": "pass"},
                "property": {"command": item.command, "returncode": "nonzero", "code": actual},
                "restoration": "clean",
            }
            require(sha(canonical(transcript)) == row["transcript_sha256"], "transcript:" + item.mutation_id)
            target.write_text(sources[item.target])
            status = subprocess.run(["git", "status", "--short"], cwd=checkout, capture_output=True, text=True, check=False)
            require(status.returncode == 0 and not status.stdout, "restoration:" + item.mutation_id)

        neutral_target = checkout / ACTOR
        neutral_target.write_text("// identity-only neutral mutation\n" + sources[ACTOR])
        structural = subprocess.run(
            [
                "cargo", "extbuild", "run", "--", "python3",
                str(ROOT / "scripts/validate_causal_projection_structural_assurance_v16.py"),
                "--mode", "structural", "--source-root", str(checkout),
            ],
            cwd=checkout,
            capture_output=True,
            text=True,
            check=False,
        )
        identity = subprocess.run(
            [
                "cargo", "extbuild", "run", "--", "python3",
                str(ROOT / "scripts/validate_causal_projection_structural_assurance_v16.py"),
                "--mode", "identity", "--source-root", str(checkout),
            ],
            cwd=checkout,
            capture_output=True,
            text=True,
            check=False,
        )
        require(structural.returncode == 0, "identity_only:structural")
        require(identity.returncode != 0 and "code=SOURCE_IDENTITY" in identity.stderr, "identity_only:identity")
        neutral_target.write_text(sources[ACTOR])
        final_status = subprocess.run(["git", "status", "--short"], cwd=checkout, capture_output=True, text=True, check=False)
        require(final_status.returncode == 0 and not final_status.stdout, "restoration:final")
    finally:
        if added:
            remove = subprocess.run(["git", "worktree", "remove", "--force", str(checkout)], cwd=ROOT, capture_output=True, text=True, check=False)
            require(remove.returncode == 0, "worktree:remove")
        elif checkout.exists():
            checkout.rmdir()


def self_test(report: dict[str, object], schema: dict[str, object], sources: dict[str, str], inventory: dict[str, object]) -> int:
    attacks = [
        lambda value: value["mutations"].pop(),
        lambda value: value["mutations"].reverse(),
        lambda value: value["mutations"].append(copy.deepcopy(value["mutations"][0])),
        lambda value: value["mutations"][0].update(row_id="missing"),
        lambda value: value["mutations"][0].update(patch_sha256="0" * 64),
        lambda value: value["mutations"][0].update(command="cargo test nearby"),
        lambda value: value["mutations"][0].update(compile_result="fail"),
        lambda value: value["mutations"][0].update(actual_property_code="SOURCE_IDENTITY"),
        lambda value: value["mutations"][0].update(transcript_sha256="0" * 64),
        lambda value: value.update(compile_failures=1),
        lambda value: value.update(identity_only_rejections=0),
        lambda value: value.update(survivors=1),
        lambda value: value.update(mutation_identity_sha256="0" * 64),
        lambda value: value.update(extra=False),
    ]
    caught = 0
    for mutate in attacks:
        changed = copy.deepcopy(report)
        mutate(changed)
        try:
            validate(changed, schema, sources, inventory)
        except MutationError:
            caught += 1
            continue
        raise MutationError("self_test:report")
    changed_schema = copy.deepcopy(schema)
    changed_schema["additionalProperties"] = True
    try:
        validate(report, changed_schema, sources, inventory)
    except MutationError:
        caught += 1
    else:
        raise MutationError("self_test:schema")
    require(caught == 15, "self_test:count")
    return caught


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--write-report", action="store_true")
    parser.add_argument("--run-campaign", action="store_true")
    args = parser.parse_args()
    sources = {path: source_at_candidate(path) for path in TARGETS}
    expected = expected_report(sources)
    if args.write_report:
        REPORT.write_text(json.dumps(expected, ensure_ascii=True, indent=2) + "\n")
    report = json.loads(REPORT.read_text())
    schema = json.loads(SCHEMA.read_text())
    inventory = json.loads(INVENTORY.read_text())
    validate(report, schema, sources, inventory)
    negative = self_test(report, schema, sources, inventory)
    if args.run_campaign:
        run_campaign(report, sources)
    print(
        "PASS: causal projection mutations v16 "
        f"mutations={report['mutation_count']} compile_failures=0 survivors=0 negative={negative} "
        f"executed={report['mutation_count'] if args.run_campaign else 0}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
