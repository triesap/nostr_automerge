#!/usr/bin/env python3
"""Execute and validate the isolated behavior-derived v19 mutation campaign."""

from __future__ import annotations

import argparse
import copy
import hashlib
import json
import os
import re
import subprocess
import sys
import tempfile
from pathlib import Path
from typing import Any

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parents[1]
SOURCE_PATH = "crates/nostr_automerge/src/graph/actor_state.rs"
REPORT = ROOT / "reports/causal_projection_mutations_v19.json"
MATRIX = ROOT / "reports/causal_projection_mutation_matrix_v19.json"
INVENTORY = ROOT / "reports/causal_projection_inventory_v19.json"
SCHEMA = ROOT / "tools/validation/causal_projection_mutations_v19.schema.json"
OUT = ROOT / "reports/evidence/v19/mutations"
RUNNER_PATH = "scripts/run_causal_projection_mutations_v19.py"
AUTHORITY = "spec/causal_projection_contracts_v19.json"
COMPILE_ARGV = [
    "cargo", "extbuild", "run", "--", "cargo", "check", "-p",
    "nostr_automerge", "--lib", "--locked",
]
RESTORE_ARGV = ["git", "diff", "--quiet", "--", SOURCE_PATH]
ENVIRONMENT = {"inherit": True, "overrides": {"PYTHONDONTWRITEBYTECODE": "1"}}
HELPER_METADATA = {
    "projection_construction": ("ProjectionBuildSite", "AcceptedCountMatches", "ProjectionBuildDescriptor", "ProjectionBuildObservation", "ProjectionBuildObservationKind", "perform"),
    "actor_sequence": ("ActorDecisionSite", "ActorIdentityDecision", "ActorDecisionDescriptor", "ActorDecisionObservation", "ActorDecisionObservationKind", "perform"),
    "causal_counter": ("CausalNextSite", "ExpectedStartComparison", "CausalNextDescriptor", "CausalNextObservation", "CausalNextObservationKind", "perform"),
    "frontier_comparison": ("FrontierComparisonSite", "CandidateCountRead", "FrontierComparisonDescriptor", "FrontierComparisonObservation", "FrontierComparisonObservationKind", "target"),
}
HELPER_NAMES = [
    "perform_projection_build_operation",
    "perform_actor_decision_operation",
    "perform_causal_next_operation",
    "metered_frontier_operation",
]
DIRECT = {
    "ActorStateRead": ("actor_sequence", "perform_actor_decision_operation", "ActorDecisionSite", "causal_projection_v17_site_actor_sequence_actor_state_read"),
    "PredecessorCandidateRead": ("actor_sequence", "perform_actor_decision_operation", "ActorDecisionSite", "causal_projection_v17_site_actor_sequence_predecessor_candidate_read"),
    "ActorIdentityDecision": ("actor_sequence", "perform_actor_decision_operation", "ActorDecisionSite", "causal_projection_v17_site_actor_sequence_actor_identity_decision"),
    "SequenceRelationDecision": ("actor_sequence", "perform_actor_decision_operation", "ActorDecisionSite", "causal_projection_v17_site_actor_sequence_sequence_relation_decision"),
    "StoredCounterRead": ("causal_counter", "perform_causal_next_operation", "CausalNextSite", "causal_projection_v17_site_causal_counter_stored_counter_read"),
    "ExpectedStartComparison": ("causal_counter", "perform_causal_next_operation", "CausalNextSite", "causal_projection_v17_site_causal_counter_expected_start_comparison"),
    "CheckedAdvance": ("causal_counter", "perform_causal_next_operation", "CausalNextSite", "causal_projection_v17_site_causal_counter_checked_advance"),
}
PROVENANCE = [
    ("typed_budget_identity", "TYPED_BUDGET_EXHAUSTED_IDENTITY", "rust.projection_construction.member_count_read"),
    ("cancellation_identity", "TYPED_CANCELLED_IDENTITY", "rust.actor_sequence.actor_state_read"),
    ("unexpected_error_identity", "UNEXPECTED_WORK_ERROR_IDENTITY", "rust.causal_counter.stored_counter_read"),
    ("publication_after_stop", "PUBLICATION_AFTER_STOP", "rust.projection_construction.projection_publish"),
    ("alternate_consumer_bypass", "ALTERNATE_CONSUMER_BYPASS", "rust.actor_sequence.actor_state_read"),
]
ROW_FIELDS = [
    "mutation_id", "campaign", "inventory_row_id", "helper", "kind",
    "expected_property_code", "actual_property_code", "compile", "owner_test",
    "property", "restoration", "patch_artifact", "patch_sha256",
    "transcript_artifact", "transcript_sha256", "source_candidate",
    "execution_base_candidate", "isolated_worktree", "shared_helper_unchanged",
    "survivor", "result",
]
TOP_FIELDS = [
    "schema", "status", "authority", "source_candidate",
    "execution_base_candidate", "matrix_path", "matrix_sha256", "row_contract",
    "rows", "counts", "execution", "result_identity_sha256", "result",
]
TRANSCRIPT_FIELDS = [
    "schema", "authority", "mutation_id", "campaign", "inventory_row_id",
    "helper", "kind", "expected_property_code", "actual_property_code",
    "source_candidate", "execution_base_candidate", "patch_artifact",
    "patch_sha256", "compile", "owner_test", "property", "restoration",
    "isolated_worktree", "shared_helper_unchanged", "survivor", "result",
]


class MutationError(RuntimeError):
    pass


def require(condition: bool, code: str) -> None:
    if not condition:
        raise MutationError(code)


def canonical(value: object) -> bytes:
    return json.dumps(value, ensure_ascii=True, separators=(",", ":"), sort_keys=True).encode()


def sha(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def public_transcript(output: str) -> str:
    return re.sub(r"/(?:Users|Volumes)/[^\s)]+", "<local-path>", output)


def run(argv: list[str], cwd: Path) -> subprocess.CompletedProcess[str]:
    environment = dict(os.environ)
    environment["PYTHONDONTWRITEBYTECODE"] = "1"
    return subprocess.run(argv, cwd=cwd, capture_output=True, text=True, check=False, env=environment)


def git(*args: str) -> str:
    completed = run(["git", *args], ROOT)
    require(completed.returncode == 0, "GIT:" + ":".join(args))
    return completed.stdout.strip()


def command_record(argv: list[str], completed: subprocess.CompletedProcess[str]) -> dict[str, Any]:
    stdout = public_transcript(completed.stdout)
    stderr = public_transcript(completed.stderr)
    return {
        "argv": argv,
        "cwd": ".",
        "environment": ENVIRONMENT,
        "exit_status": completed.returncode,
        "stdout": stdout,
        "stderr": stderr,
        "stdout_sha256": sha(stdout.encode()),
        "stderr_sha256": sha(stderr.encode()),
        "raw_stdout_sha256": sha(completed.stdout.encode()),
        "raw_stderr_sha256": sha(completed.stderr.encode()),
    }


def snake(value: str) -> str:
    return re.sub(r"(?<!^)(?=[A-Z])", "_", value).lower()


def definitions(matrix: dict[str, Any], contract: dict[str, Any]) -> list[dict[str, Any]]:
    rows = [dict(row, campaign="helper", inventory_row_id=f"rust.{row['phase']}.{snake(row['representative_site'])}") for row in matrix["rows"]]
    rows.extend({
        "mutation_id": f"direct.{site}.target_hoist_and_cache",
        "campaign": "direct", "inventory_row_id": f"rust.{DIRECT[site][0]}.{snake(site)}",
        "helper": DIRECT[site][1], "kind": "target_hoist_and_cache",
        "expected_property_code": "SITE_TARGET_BEFORE_CHARGE", "site": site,
    } for site in contract["direct_sites"])
    rows.extend({
        "mutation_id": f"provenance.{kind}", "campaign": "provenance",
        "inventory_row_id": inventory_row_id, "helper": None, "kind": kind,
        "expected_property_code": expected,
    } for kind, expected, inventory_row_id in PROVENANCE)
    return rows


def matching_call_end(source: str, start: int) -> int:
    opening = source.find("(", start)
    require(opening >= 0, "CALL_OPEN")
    depth = 0
    for index in range(opening, len(source)):
        if source[index] == "(":
            depth += 1
        elif source[index] == ")":
            depth -= 1
            if depth == 0:
                return index + 1
    raise MutationError("CALL_CLOSE")


def function_bounds(source: str, name: str) -> tuple[int, int]:
    start = source.find(f"fn {name}")
    require(start >= 0, "FUNCTION:" + name)
    opening = source.find("{", start)
    require(opening >= 0, "FUNCTION_OPEN:" + name)
    depth = 0
    for index in range(opening, len(source)):
        if source[index] == "{":
            depth += 1
        elif source[index] == "}":
            depth -= 1
            if depth == 0:
                return start, index + 1
    raise MutationError("FUNCTION_CLOSE:" + name)


def replace_in_function(source: str, helper: str, old: str, new: str) -> str:
    start, end = function_bounds(source, helper)
    body = source[start:end]
    require(body.count(old) == 1, "HELPER_TARGET:" + helper)
    return source[:start] + body.replace(old, new, 1) + source[end:]


def helper_blocks(phase: str) -> tuple[str, str, str]:
    _enum, _swap, _descriptor, observation, observation_kind, target = HELPER_METADATA[phase]
    charge = "    charge(descriptor).map_err(MeteredActorStateError::Work)?;"
    target_block = (
        "    #[cfg(test)]\n"
        "    v19_record_proof_event(V19ProofEvent::TargetDispatched {\n"
        "        site_id: descriptor.site_id,\n        counter: descriptor.counter,\n    });\n"
        f"    let result = {target}();\n"
        "    #[cfg(test)]\n"
        "    v19_record_proof_event(V19ProofEvent::TargetReturned {\n"
        "        site_id: descriptor.site_id,\n        counter: descriptor.counter,\n    });"
    )
    observation_block = (
        f"    observed({observation} {{\n        descriptor,\n"
        f"        kind: {observation_kind}::TargetCompleted,\n    }});"
    )
    return charge, target_block, observation_block


def repeatable_projection_publish(source: str) -> str:
    old = (
        "        || TrustedEpochProjection {\n            branch_membership: changes,\n"
        "            accepted_closure,\n            dependencies,\n            frontier_heads,\n"
        "            actor_states: states,\n            writer_contributions,\n            causal_next_op,\n        },"
    )
    new = (
        "        || TrustedEpochProjection {\n            branch_membership: changes,\n"
        "            accepted_closure,\n            dependencies: dependencies.clone(),\n"
        "            frontier_heads: frontier_heads.clone(),\n            actor_states: states.clone(),\n"
        "            writer_contributions: writer_contributions.clone(),\n            causal_next_op,\n        },"
    )
    require(source.count(old) == 1, "PROJECTION_REPEATABLE")
    return source.replace(old, new, 1)


def mutate_helper(source: str, definition: dict[str, Any]) -> str:
    phase, helper, kind = definition["phase"], definition["helper"], definition["kind"]
    enum, swap, descriptor_type, _observation, _kind, target = HELPER_METADATA[phase]
    charge, target_block, observation_block = helper_blocks(phase)
    if kind == "target_before_charge":
        source = replace_in_function(source, helper, charge + "\n" + target_block, target_block + "\n" + charge)
    elif kind == "completion_before_target":
        source = replace_in_function(source, helper, target_block + "\n" + observation_block, observation_block + "\n" + target_block)
    elif kind == "target_after_failed_charge":
        source = replace_in_function(source, helper, charge, "    let _charge_result = charge(descriptor).map_err(MeteredActorStateError::Work);")
    elif kind == "completion_after_failed_charge":
        replacement = (
            "    if let Err(error) = charge(descriptor) {\n"
            + observation_block
            + "\n        return Err(MeteredActorStateError::Work(error));\n    }"
        )
        source = replace_in_function(source, helper, charge, replacement)
    elif kind == "double_target":
        source = replace_in_function(source, helper, f"{target}: impl FnOnce() -> T", f"mut {target}: impl FnMut() -> T")
        second = (
            target_block
            + "\n    #[cfg(test)]\n    v19_record_proof_event(V19ProofEvent::TargetDispatched {\n"
            "        site_id: descriptor.site_id,\n        counter: descriptor.counter,\n    });\n"
            f"    let _second_result = {target}();\n"
            "    #[cfg(test)]\n    v19_record_proof_event(V19ProofEvent::TargetReturned {\n"
            "        site_id: descriptor.site_id,\n        counter: descriptor.counter,\n    });"
        )
        source = replace_in_function(source, helper, target_block, second)
        if phase == "projection_construction":
            source = repeatable_projection_publish(source)
    elif kind == "site_swap":
        source = replace_in_function(source, helper, "    let descriptor = site.descriptor();", f"    let descriptor = {enum}::{swap}.descriptor();")
    elif kind == "counter_mismatch":
        replacement = f"    let descriptor = {descriptor_type} {{\n        counter: WorkCounter::GraphEdge,\n        ..site.descriptor()\n    }};"
        source = replace_in_function(source, helper, "    let descriptor = site.descriptor();", replacement)
    else:
        raise MutationError("UNKNOWN_HELPER_MUTATION:" + kind)
    return source


def hoist_direct_target(source: str, site: str) -> str:
    _phase, helper, enum, _test = DIRECT[site]
    match = re.search(rf"\b{helper}\s*\(\s*{enum}::{site}\b", source)
    require(match is not None, "DIRECT_CALL:" + site)
    end = matching_call_end(source, match.start())
    call = source[match.start():end]
    closure = call.find("||")
    comma = call.rfind(",")
    require(closure >= 0 and comma > closure, "DIRECT_CLOSURE:" + site)
    expression = call[closure + 2:comma].strip()
    cached = "cached_" + snake(site)
    changed_call = call[:closure + 2] + " " + cached + call[comma:]
    changed = source[:match.start()] + changed_call + source[end:]
    line = changed.rfind("\n", 0, match.start()) + 1
    indent = re.match(r"\s*", changed[line:match.start()]).group()
    return changed[:line] + f"{indent}let {cached} = {expression};\n" + changed[line:]


def mutate_provenance(source: str, kind: str) -> str:
    helper = {
        "typed_budget_identity": "perform_projection_build_operation",
        "cancellation_identity": "perform_actor_decision_operation",
        "unexpected_error_identity": "perform_causal_next_operation",
    }.get(kind)
    if helper is not None:
        return replace_in_function(
            source, helper,
            "    charge(descriptor).map_err(MeteredActorStateError::Work)?;",
            "    charge(descriptor)\n        .map_err(|_| MeteredActorStateError::State(ActorStateError::NoncanonicalInput))?;",
        )
    if kind == "publication_after_stop":
        publication = "    published(ProjectionPublicationOperation::Projection);\n"
        require(source.count(publication) == 1, "PUBLICATION_SOURCE")
        source = source.replace(publication, "", 1)
        call = "    let projection = perform_projection_build_operation(\n        ProjectionBuildSite::ProjectionPublish,"
        require(source.count(call) == 1, "PUBLICATION_CALL")
        return source.replace(call, publication + call, 1)
    if kind == "alternate_consumer_bypass":
        start = source.find("        let actor_state = perform_actor_decision_operation(\n            ActorDecisionSite::ActorStateRead,")
        require(start >= 0, "CONSUMER_SOURCE")
        call = source.find("perform_actor_decision_operation", start)
        end = matching_call_end(source, call)
        require(source[end:end + 2] == "?;", "CONSUMER_CALL_END")
        return source[:start] + "        let actor_state = self.actor_states.get(&candidate.actor).copied();" + source[end + 2:]
    raise MutationError("UNKNOWN_PROVENANCE:" + kind)


def mutate(source: str, definition: dict[str, Any]) -> tuple[str, bool]:
    helper_bodies = {
        helper: source[slice(*function_bounds(source, helper))]
        for helper in HELPER_NAMES
    }
    if definition["campaign"] == "helper":
        changed = mutate_helper(source, definition)
    elif definition["campaign"] == "direct":
        changed = hoist_direct_target(source, definition["site"])
    else:
        changed = mutate_provenance(source, definition["kind"])
    require(changed != source, "EMPTY_MUTATION:" + definition["mutation_id"])
    unchanged = all(changed[slice(*function_bounds(changed, helper))] == body for helper, body in helper_bodies.items())
    if definition["campaign"] == "direct":
        require(unchanged, "DIRECT_HELPER_CHANGED:" + definition["mutation_id"])
    return changed, unchanged


def property_argv(code: str) -> list[str]:
    return ["python3", "scripts/validate_causal_projection_properties_v19.py", "--root", ".", "--property", code, "--expect-failure"]


def owner_argv(definition: dict[str, Any]) -> list[str] | None:
    if definition["campaign"] != "direct":
        return None
    test = DIRECT[definition["site"]][3]
    return ["cargo", "extbuild", "run", "--", "cargo", "test", "-p", "nostr_automerge", "--lib", f"graph::actor_state::tests::{test}", "--locked", "--", "--exact", "--nocapture"]


def artifact_paths(definition: dict[str, Any]) -> tuple[Path, Path]:
    stem = definition["mutation_id"].replace(".", "_")
    return OUT / f"{stem}.patch.json", OUT / f"{stem}.transcript.json"


def relative(path: Path) -> str:
    return path.relative_to(ROOT).as_posix()


def property_code(completed: subprocess.CompletedProcess[str]) -> str:
    matches = re.findall(r"^FAIL: ([A-Z0-9_]+)$", completed.stdout + completed.stderr, re.MULTILINE)
    require(len(matches) == 1, "PROPERTY_OUTPUT")
    return matches[0]


def execute_one(definition: dict[str, Any], source_candidate: str, execution_base: str) -> dict[str, Any]:
    temp_root = Path(tempfile.mkdtemp(prefix="nostr-automerge-v19-mutant-"))
    worktree = temp_root / "worktree"
    try:
        require(run(["git", "worktree", "add", "--detach", str(worktree), execution_base], ROOT).returncode == 0, "WORKTREE_ADD")
        require(run(["cargo", "extbuild", "doctor"], worktree).returncode == 0, "EXTBUILD_DOCTOR")
        source_path = worktree / SOURCE_PATH
        baseline = source_path.read_text()
        changed, helper_unchanged = mutate(baseline, definition)
        source_path.write_text(changed)
        diff = run(["git", "diff", "--", SOURCE_PATH], worktree)
        require(diff.returncode == 0 and diff.stdout, "PATCH_EMPTY")
        patch_path, transcript_path = artifact_paths(definition)
        patch = {"schema": "nostr_automerge.causal_projection_mutation_patch.v19.v1", "mutation_id": definition["mutation_id"], "encoding": "unified_diff_utf8", "patch": diff.stdout, "result": "pass"}
        patch_raw = json.dumps(patch, ensure_ascii=True, indent=2) + "\n"
        patch_path.write_text(patch_raw)
        compiled = run(COMPILE_ARGV, worktree)
        owner_command = owner_argv(definition)
        owner = run(owner_command, worktree) if owner_command is not None else None
        property_command = property_argv(definition["expected_property_code"])
        checked = run(property_command, worktree)
        actual = property_code(checked)
        source_path.write_text(baseline)
        restored = run(RESTORE_ARGV, worktree)
        require(compiled.returncode == 0, "COMPILE:" + definition["mutation_id"])
        require(owner is None or owner.returncode == 0, "OWNER_TEST:" + definition["mutation_id"])
        require(checked.returncode == 1 and actual == definition["expected_property_code"], "PROPERTY:" + definition["mutation_id"] + ":" + actual)
        require(restored.returncode == 0, "RESTORATION:" + definition["mutation_id"])
        transcript = {
            "schema": "nostr_automerge.causal_projection_mutation_transcript.v19.v1",
            "authority": AUTHORITY, "mutation_id": definition["mutation_id"],
            "campaign": definition["campaign"], "inventory_row_id": definition["inventory_row_id"],
            "helper": definition.get("helper"), "kind": definition["kind"],
            "expected_property_code": definition["expected_property_code"], "actual_property_code": actual,
            "source_candidate": source_candidate, "execution_base_candidate": execution_base,
            "patch_artifact": relative(patch_path), "patch_sha256": sha(patch_raw.encode()),
            "compile": command_record(COMPILE_ARGV, compiled),
            "owner_test": None if owner is None else command_record(owner_command, owner),
            "property": command_record(property_command, checked),
            "restoration": command_record(RESTORE_ARGV, restored),
            "isolated_worktree": True, "shared_helper_unchanged": helper_unchanged,
            "survivor": False, "result": "killed",
        }
        transcript_raw = json.dumps(transcript, ensure_ascii=True, indent=2) + "\n"
        transcript_path.write_text(transcript_raw)
        return row_from_artifacts(definition, patch_raw, transcript_raw, transcript)
    finally:
        if worktree.exists():
            run(["git", "worktree", "remove", "--force", str(worktree)], ROOT)
        try:
            temp_root.rmdir()
        except OSError:
            pass


def row_from_artifacts(definition: dict[str, Any], patch_raw: str, transcript_raw: str, transcript: dict[str, Any]) -> dict[str, Any]:
    require(list(transcript) == TRANSCRIPT_FIELDS, "TRANSCRIPT_SHAPE")
    patch_path, transcript_path = artifact_paths(definition)
    return {
        "mutation_id": definition["mutation_id"], "campaign": definition["campaign"],
        "inventory_row_id": definition["inventory_row_id"], "helper": definition.get("helper"),
        "kind": definition["kind"], "expected_property_code": definition["expected_property_code"],
        "actual_property_code": transcript["actual_property_code"], "compile": transcript["compile"],
        "owner_test": transcript["owner_test"], "property": transcript["property"],
        "restoration": transcript["restoration"], "patch_artifact": relative(patch_path),
        "patch_sha256": sha(patch_raw.encode()), "transcript_artifact": relative(transcript_path),
        "transcript_sha256": sha(transcript_raw.encode()), "source_candidate": transcript["source_candidate"],
        "execution_base_candidate": transcript["execution_base_candidate"], "isolated_worktree": True,
        "shared_helper_unchanged": transcript["shared_helper_unchanged"], "survivor": False, "result": "killed",
    }


def load_row(definition: dict[str, Any], source_candidate: str, execution_base: str) -> dict[str, Any]:
    patch_path, transcript_path = artifact_paths(definition)
    patch_raw, transcript_raw = patch_path.read_text(), transcript_path.read_text()
    patch, transcript = json.loads(patch_raw), json.loads(transcript_raw)
    require(patch["mutation_id"] == definition["mutation_id"] and patch["result"] == "pass", "PATCH_ID")
    require("v19-mutation-property=" not in patch["patch"], "MARKER_SELECTED")
    require(transcript["patch_sha256"] == sha(patch_raw.encode()), "PATCH_HASH")
    require(transcript["source_candidate"] == source_candidate and transcript["execution_base_candidate"] == execution_base, "TRANSCRIPT_BASE")
    require(transcript["expected_property_code"] == transcript["actual_property_code"] == definition["expected_property_code"], "PROPERTY_CODE")
    require(transcript["compile"]["exit_status"] == 0 and transcript["property"]["exit_status"] == 1, "COMMAND_RESULTS")
    require(transcript["owner_test"] is None or transcript["owner_test"]["exit_status"] == 0, "OWNER_RESULT")
    require(transcript["restoration"]["exit_status"] == 0 and transcript["isolated_worktree"], "RESTORATION")
    for record in (transcript["compile"], transcript["property"], transcript["restoration"]):
        require(record["cwd"] == "." and record["environment"] == ENVIRONMENT, "COMMAND_ENVELOPE")
        require(not any(root in record["stdout"] + record["stderr"] for root in ("/" + "Users/", "/" + "Volumes/")), "PRIVATE_PATH")
    return row_from_artifacts(definition, patch_raw, transcript_raw, transcript)


def expected_report(source_candidate: str, execution_base: str, rows: list[dict[str, Any]]) -> dict[str, Any]:
    campaigns = {name: sum(row["campaign"] == name for row in rows) for name in ("helper", "direct", "provenance")}
    report: dict[str, Any] = {
        "schema": "nostr_automerge.causal_projection_mutations.v19.v1",
        "status": "actual_execution_raw_unbound", "authority": AUTHORITY,
        "source_candidate": source_candidate, "execution_base_candidate": execution_base,
        "matrix_path": "reports/causal_projection_mutation_matrix_v19.json",
        "matrix_sha256": sha(MATRIX.read_bytes()), "row_contract": ROW_FIELDS, "rows": rows,
        "counts": {"mutations": len(rows), "campaigns": campaigns, "compile_passed": len(rows), "properties_killed": len(rows), "survivors": 0},
        "execution": {"worktree": "isolated_detached_per_mutant", "property_process": "runtime_subprocess", "complete_redacted_transcripts": True, "raw_output_hashes": True, "marker_selected_properties": False, "identity_only_kills": False, "restored": True, "artifact_commit_binding": "later_catalog"},
        "result_identity_sha256": "", "result": "pass",
    }
    identity = {key: value for key, value in report.items() if key != "result_identity_sha256"}
    report["result_identity_sha256"] = sha(canonical(identity))
    return report


def execute(campaign: list[dict[str, Any]], source_candidate: str) -> tuple[str, list[dict[str, Any]]]:
    require(not git("status", "--porcelain=v1"), "EXECUTION_ROOT_DIRTY")
    execution_base = git("rev-parse", "HEAD")
    require(git("show", f"{execution_base}:{RUNNER_PATH}") == Path(RUNNER_PATH).read_text().strip(), "RUNNER_NOT_COMMITTED")
    require(not OUT.exists(), "ARTIFACT_DIRECTORY_EXISTS")
    OUT.mkdir(parents=True)
    return execution_base, [execute_one(definition, source_candidate, execution_base) for definition in campaign]


def validate(report: Any, schema: Any, matrix: dict[str, Any], contract: dict[str, Any]) -> None:
    require(type(report) is dict and list(report) == TOP_FIELDS, "REPORT_SHAPE")
    campaign = definitions(matrix, contract)
    rows = [load_row(definition, report["source_candidate"], report["execution_base_candidate"]) for definition in campaign]
    require(report == expected_report(report["source_candidate"], report["execution_base_candidate"], rows), "REPORT_DERIVATION")
    require(report["source_candidate"] == json.loads(INVENTORY.read_text())["source_candidate"], "SOURCE_CANDIDATE")
    require(len(rows) == len({row["mutation_id"] for row in rows}) == 40, "MUTATION_IDENTITY")
    require(report["counts"]["campaigns"] == {"helper": 28, "direct": 7, "provenance": 5}, "CAMPAIGN_COUNTS")
    require(len({row["patch_sha256"] for row in rows}) == 40, "PATCH_UNIQUENESS")
    require(all(row["shared_helper_unchanged"] for row in rows if row["campaign"] == "direct"), "DIRECT_HELPERS")
    require(report["counts"]["survivors"] == 0 and report["execution"]["restored"], "SURVIVORS")
    base = report["execution_base_candidate"]
    require(git("rev-parse", base + "^{commit}") == base, "EXECUTION_BASE")
    require(run(["git", "merge-base", "--is-ancestor", report["source_candidate"], base], ROOT).returncode == 0, "ANCESTRY")
    require(git("show", f"{base}:{RUNNER_PATH}") == Path(RUNNER_PATH).read_text().strip(), "RUNNER_DRIFT")
    expected_artifacts = {relative(path) for definition in campaign for path in artifact_paths(definition)}
    require({relative(path) for path in OUT.glob("*.json")} == expected_artifacts, "ARTIFACT_SET")
    require(schema.get("additionalProperties") is False and schema.get("required") == TOP_FIELDS, "SCHEMA_CLOSED")
    require(schema["properties"]["rows"]["items"].get("required") == ROW_FIELDS, "SCHEMA_ROWS")


def self_test(report: dict[str, Any], schema: dict[str, Any], matrix: dict[str, Any], contract: dict[str, Any]) -> int:
    attacks = [
        lambda value: value["rows"].pop(),
        lambda value: value["rows"].__setitem__(1, copy.deepcopy(value["rows"][0])),
        lambda value: value["rows"][0].update(actual_property_code="SURVIVED"),
        lambda value: value["rows"][0]["property"].update(exit_status=0),
        lambda value: value["rows"][0].update(patch_sha256="0" * 64),
        lambda value: value["rows"][0].update(transcript_sha256="0" * 64),
        lambda value: value["rows"][0].update(survivor=True),
        lambda value: value["counts"].update(survivors=1),
        lambda value: value["execution"].update(marker_selected_properties=True),
        lambda value: value.update(execution_base_candidate="0" * 40),
    ]
    caught = 0
    for mutate in attacks:
        changed = copy.deepcopy(report)
        mutate(changed)
        try:
            validate(changed, schema, matrix, contract)
        except MutationError:
            caught += 1
            continue
        raise MutationError("REPORT_ATTACK_SURVIVED")
    return caught


def definition_self_test(campaign: list[dict[str, Any]]) -> None:
    source = (ROOT / SOURCE_PATH).read_text()
    identities = []
    for definition in campaign:
        changed, helper_unchanged = mutate(source, definition)
        require(
            changed.count("v19-mutation-property=")
            == source.count("v19-mutation-property="),
            "MARKER_SELECTED_DEFINITION",
        )
        if definition["campaign"] == "direct":
            require(helper_unchanged, "DIRECT_HELPER_SELF_TEST")
        identities.append(sha(changed.encode()))
    require(len(campaign) == len(set(identities)) == 40, "DEFINITION_UNIQUENESS")
    print("PASS v19 mutation definitions: helper=28 direct=7 provenance=5 unique=40")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--execute", action="store_true")
    parser.add_argument("--write-report", action="store_true")
    parser.add_argument("--self-test-definitions", action="store_true")
    args = parser.parse_args()
    require(not args.write_report or args.execute, "WRITE_REQUIRES_EXECUTION")
    matrix = json.loads(MATRIX.read_text())
    contract = json.loads((ROOT / AUTHORITY).read_text())
    campaign = definitions(matrix, contract)
    if args.self_test_definitions:
        definition_self_test(campaign)
        return 0
    if args.execute:
        source_candidate = json.loads(INVENTORY.read_text())["source_candidate"]
        execution_base, rows = execute(campaign, source_candidate)
        if args.write_report:
            REPORT.write_text(json.dumps(expected_report(source_candidate, execution_base, rows), ensure_ascii=True, indent=2) + "\n")
    report = json.loads(REPORT.read_text())
    schema = json.loads(SCHEMA.read_text())
    validate(report, schema, matrix, contract)
    print(f"PASS v19 mutation campaign: actual={len(report['rows'])} helper=28 direct=7 provenance=5 survivors=0 attacks={self_test(report, schema, matrix, contract)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
