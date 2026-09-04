#!/usr/bin/env python3
"""Execute and validate isolated replayable v18 mutation campaigns."""

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
CONSUMER_PATH = "crates/nostr_automerge/src/reference/epoch_engine.rs"
REPORT = ROOT / "reports/causal_projection_mutations_v18.json"
SCHEMA = ROOT / "tools/validation/causal_projection_mutations_v18.schema.json"
OUT = ROOT / "reports/evidence/v18/mutations"
AUTHORITY = "spec/causal_projection_contracts_v18.json"
CONTRACT_PATH = "spec/causal_projection_contracts_v18.json"
SOURCE_CANDIDATE = "076221ad7f03e67d89ac4b2fcfc8f2586b97f182"
COMPILE_ARGV = [
    "cargo", "extbuild", "run", "--", "cargo", "check", "-p",
    "nostr_automerge", "--lib", "--locked",
]
PROPERTY_ARGV = [
    "python3", "scripts/validate_causal_projection_properties_v18.py",
    "--root", ".", "--mode", "structural",
]
IDENTITY_ARGV = [
    "python3", "scripts/validate_causal_projection_properties_v18.py",
    "--root", ".", "--mode", "identity",
]
ENVIRONMENT = {"inherit": True, "overrides": {"PYTHONDONTWRITEBYTECODE": "1"}}
ROW_FIELDS = [
    "mutation_id", "campaign", "inventory_row_id", "kind",
    "expected_property_code", "actual_property_code", "compile_command",
    "compile_exit_status", "compile_output_sha256", "property_command",
    "property_exit_status", "property_output_sha256", "restoration_command",
    "restoration_result", "execution_envelope", "patch_artifact",
    "patch_sha256", "transcript_artifact", "transcript_sha256",
    "shared_helper_unchanged", "source_candidate", "execution_base_candidate",
    "survivor", "result",
]
TOP_FIELDS = [
    "schema", "status", "authority", "source_candidate",
    "execution_base_candidate", "row_contract", "rows", "counts",
    "execution", "preflight", "result_identity_sha256", "result",
]
TRANSCRIPT_FIELDS = [
    "schema", "authority", "mutation_id", "campaign", "inventory_row_id",
    "kind", "source_candidate", "execution_base_candidate", "patch_artifact",
    "patch_sha256", "compile", "property", "restoration",
    "expected_property_code", "actual_property_code", "shared_helper_unchanged",
    "survivor", "result",
]

sys.path.insert(0, str(ROOT / "scripts"))
from validate_causal_projection_properties_v18 import (  # noqa: E402
    DIRECT_SITES,
    HELPERS,
    function_body,
    hoist_direct_target,
)


class MutationError(RuntimeError):
    pass


def require(condition: bool, code: str) -> None:
    if not condition:
        raise MutationError(code)


def canonical(value: object) -> bytes:
    return json.dumps(value, ensure_ascii=True, separators=(",", ":"), sort_keys=True).encode()


def sha(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def run(argv: list[str], cwd: Path) -> subprocess.CompletedProcess[str]:
    environment = dict(os.environ)
    environment["PYTHONDONTWRITEBYTECODE"] = "1"
    return subprocess.run(argv, cwd=cwd, capture_output=True, text=True, check=False, env=environment)


def output_identity(completed: subprocess.CompletedProcess[str]) -> str:
    return sha(canonical({
        "stdout_sha256": sha(completed.stdout.encode()),
        "stderr_sha256": sha(completed.stderr.encode()),
    }))


def command_record(argv: list[str], completed: subprocess.CompletedProcess[str]) -> dict[str, Any]:
    return {
        "argv": argv,
        "cwd": ".",
        "environment": ENVIRONMENT,
        "exit_status": completed.returncode,
        "stdout_sha256": sha(completed.stdout.encode()),
        "stderr_sha256": sha(completed.stderr.encode()),
        "output_sha256": output_identity(completed),
    }


def snake(value: str) -> str:
    return re.sub(r"(?<!^)(?=[A-Z])", "_", value).lower()


def definitions(contract: dict[str, Any]) -> list[dict[str, str]]:
    direct_phase = {
        site: "actor_sequence" if owner.startswith("actor_") else "causal_counter"
        for owner, _helper, _enum, site, _target in DIRECT_SITES
    }
    rows = [
        {
            "mutation_id": f"direct.{site}.target_hoist_and_cache",
            "campaign": "direct",
            "inventory_row_id": f"rust.{direct_phase[site]}.{snake(site)}",
            "kind": "site_local_target_hoist_and_cache",
            "site": site,
            "expected": "SITE_TARGET_BEFORE_CHARGE",
        }
        for site in contract["mutation"]["mandatory_direct_sites"]
    ]
    representatives = {
        "projection_construction": "rust.projection_construction.member_count_read",
        "actor_sequence": "rust.actor_sequence.actor_state_read",
        "causal_counter": "rust.causal_counter.stored_counter_read",
        "frontier_comparison": "rust.frontier_comparison.candidate_kind_comparison",
    }
    rows.extend({
        "mutation_id": f"helper.{phase}.target_before_charge",
        "campaign": "helper",
        "inventory_row_id": representatives[phase],
        "kind": "helper_target_before_charge",
        "phase": phase,
        "expected": "CHARGE_AFTER_OPERATION",
    } for phase in HELPERS)
    provenance = [
        ("typed_stop_collapse", "TYPED_BUDGET_EXHAUSTED_IDENTITY", "rust.actor_sequence.actor_state_read"),
        ("cancellation_collapse", "TYPED_CANCELLED_IDENTITY", "rust.actor_sequence.actor_state_read"),
        ("unexpected_error_replacement", "UNEXPECTED_WORK_ERROR_IDENTITY", "rust.actor_sequence.actor_state_read"),
        ("target_after_failed_charge", "TARGET_AFTER_STOP", "rust.actor_sequence.actor_state_read"),
        ("observation_before_target", "OPERATION_OBSERVATION_BEFORE_TARGET", "rust.actor_sequence.actor_state_read"),
        ("completion_after_return", "OBSERVATION_AFTER_STOP", "rust.actor_sequence.actor_state_read"),
        ("publication_before_charge", "PUBLICATION_AFTER_STOP", "rust.projection_construction.projection_publish"),
        ("site_identity_mismatch", "SITE_ID_MISMATCH", "rust.projection_construction.member_count_read"),
        ("counter_identity_mismatch", "COUNTER_MISMATCH", "rust.projection_construction.member_count_read"),
        ("alternate_consumer_bypass", "ALTERNATE_CONSUMER_BYPASS", "rust.projection_construction.member_count_read"),
    ]
    rows.extend({
        "mutation_id": f"provenance.{kind}",
        "campaign": "provenance",
        "inventory_row_id": inventory_row_id,
        "kind": kind,
        "expected": expected,
    } for kind, expected, inventory_row_id in provenance)
    return rows


def helper_replace(source: str, helper: str, old: str, new: str) -> str:
    marker = f"fn {helper}"
    head, tail = source.split(marker, 1)
    changed_tail = tail.replace(old, new, 1)
    require(changed_tail != tail, f"HELPER_PATCH:{helper}")
    return head + marker + changed_tail


def mutate_helper(source: str, phase: str) -> str:
    helper = HELPERS[phase][1]
    target = "target" if helper == "metered_frontier_operation" else "perform"
    old = f"charge(descriptor).map_err(MeteredActorStateError::Work)?;\n    let result = {target}();"
    new = f"let result = {target}();\n    charge(descriptor).map_err(MeteredActorStateError::Work)?;"
    return helper_replace(source, helper, old, new)


def mutate_provenance(source: str, consumer: str, kind: str) -> tuple[str, str]:
    charge = "charge(descriptor).map_err(MeteredActorStateError::Work)?;"
    if kind in {"typed_stop_collapse", "cancellation_collapse", "unexpected_error_replacement"}:
        marker = {
            "typed_stop_collapse": "_v18_typed_stop_collapsed",
            "cancellation_collapse": "_v18_cancellation_collapsed",
            "unexpected_error_replacement": "_v18_unexpected_error_replaced",
        }[kind]
        replacement = (
            f"let {marker} = ();\n    charge(descriptor)\n"
            "        .map_err(|_| MeteredActorStateError::State(ActorStateError::NoncanonicalInput))?;"
        )
        source = helper_replace(source, "perform_actor_decision_operation", charge, replacement)
    elif kind == "target_after_failed_charge":
        source = helper_replace(
            source,
            "perform_actor_decision_operation",
            charge,
            "let _v18_ignored_charge = charge(descriptor).map_err(MeteredActorStateError::Work);",
        )
    elif kind == "observation_before_target":
        old = (
            f"{charge}\n    let result = perform();\n"
            "    observed(ActorDecisionObservation {\n        descriptor,\n"
            "        kind: ActorDecisionObservationKind::TargetCompleted,\n    });"
        )
        new = (
            f"{charge}\n    observed(ActorDecisionObservation {{\n        descriptor,\n"
            "        kind: ActorDecisionObservationKind::TargetCompleted,\n    });\n"
            "    let result = perform();"
        )
        source = helper_replace(source, "perform_actor_decision_operation", old, new)
    elif kind == "completion_after_return":
        old = (
            "observed(ActorDecisionObservation {\n        descriptor,\n"
            "        kind: ActorDecisionObservationKind::TargetCompleted,\n    });\n    Ok(result)"
        )
        new = (
            "let _v18_returned = Ok(result);\n    observed(ActorDecisionObservation {\n"
            "        descriptor,\n        kind: ActorDecisionObservationKind::TargetCompleted,\n"
            "    });\n    _v18_returned"
        )
        source = helper_replace(source, "perform_actor_decision_operation", old, new)
    elif kind == "publication_before_charge":
        publication = "    published(ProjectionPublicationOperation::Projection);\n"
        require(source.count(publication) == 1, "PUBLICATION_PATCH")
        source = source.replace(publication, "", 1)
        call = "    let projection = perform_projection_build_operation(\n        ProjectionBuildSite::ProjectionPublish,"
        require(source.count(call) == 1, "PUBLICATION_CALL")
        source = source.replace(call, publication + call, 1)
    elif kind == "site_identity_mismatch":
        source = source.replace(
            "MemberCountRead => (SourceCountRead, GraphNode)",
            "MemberCountRead => (CandidateLookup, GraphNode)",
            1,
        )
    elif kind == "counter_identity_mismatch":
        source = source.replace(
            "MemberCountRead => (SourceCountRead, GraphNode)",
            "MemberCountRead => (SourceCountRead, GraphEdge)",
            1,
        )
    elif kind == "alternate_consumer_bypass":
        consumer = consumer.replace(
            ".candidate_semantics_decision_metered(",
            ".candidate_semantics_decision_unmetered(",
            1,
        )
    else:
        raise MutationError(f"UNKNOWN_MUTATION:{kind}")
    return source, consumer


def mutate(
    definition: dict[str, str], source: str, consumer: str
) -> tuple[str, str, bool]:
    helper_bodies = {
        helper: function_body(source, helper) for _, helper in HELPERS.values()
    }
    if definition["campaign"] == "direct":
        changed_source = hoist_direct_target(source, definition["site"])
        changed_consumer = consumer
    elif definition["campaign"] == "helper":
        changed_source = mutate_helper(source, definition["phase"])
        changed_consumer = consumer
    else:
        changed_source, changed_consumer = mutate_provenance(
            source, consumer, definition["kind"]
        )
    require(
        changed_source != source or changed_consumer != consumer,
        "MUTATION_PATCH_EMPTY:" + definition["mutation_id"],
    )
    unchanged = all(
        function_body(changed_source, helper) == body
        for helper, body in helper_bodies.items()
    )
    if definition["campaign"] == "direct":
        require(unchanged, "DIRECT_SHARED_HELPER_CHANGED:" + definition["mutation_id"])
    return changed_source, changed_consumer, unchanged


def artifact_paths(definition: dict[str, str]) -> tuple[Path, Path]:
    stem = definition["mutation_id"].replace(".", "_")
    return OUT / f"{stem}.patch.json", OUT / f"{stem}.transcript.json"


def relative(path: Path) -> str:
    return path.relative_to(ROOT).as_posix()


def property_code(completed: subprocess.CompletedProcess[str]) -> str:
    matches = re.findall(r"^FAIL: ([A-Z0-9_]+)$", completed.stdout + completed.stderr, re.MULTILINE)
    require(len(matches) == 1, "PROPERTY_OUTPUT")
    return matches[0]


def execute() -> tuple[str, list[dict[str, Any]], dict[str, Any]]:
    require(not subprocess.run(["git", "status", "--porcelain"], cwd=ROOT, capture_output=True, text=True).stdout, "EXECUTION_ROOT_DIRTY")
    execution_base = run(["git", "rev-parse", "HEAD"], ROOT).stdout.strip()
    contract = json.loads((ROOT / CONTRACT_PATH).read_text())
    campaign = definitions(contract)
    require(not OUT.exists() or not any(OUT.iterdir()), "ARTIFACT_DIRECTORY_NOT_EMPTY")
    OUT.mkdir(parents=True, exist_ok=True)
    temp_root = Path(tempfile.mkdtemp(prefix="nostr-automerge-v18-mutations-"))
    worktree = temp_root / "worktree"
    added = run(["git", "worktree", "add", "--detach", str(worktree), execution_base], ROOT)
    require(added.returncode == 0, "WORKTREE_ADD")
    rows: list[dict[str, Any]] = []
    try:
        doctor = run(["cargo", "extbuild", "doctor"], worktree)
        require(doctor.returncode == 0, "EXTBUILD_DOCTOR")
        identity_preflight = run(IDENTITY_ARGV, worktree)
        structural_preflight = run(PROPERTY_ARGV, worktree)
        require(identity_preflight.returncode == structural_preflight.returncode == 0, "PREFLIGHT")
        preflight = {
            "identity": command_record(IDENTITY_ARGV, identity_preflight),
            "structural": command_record(PROPERTY_ARGV, structural_preflight),
            "extbuild_doctor": "pass",
        }
        source_path = worktree / SOURCE_PATH
        consumer_path = worktree / CONSUMER_PATH
        baseline_source = source_path.read_text()
        baseline_consumer = consumer_path.read_text()
        for definition in campaign:
            changed_source, changed_consumer, shared_helper_unchanged = mutate(
                definition, baseline_source, baseline_consumer
            )
            source_path.write_text(changed_source)
            consumer_path.write_text(changed_consumer)
            diff = run(["git", "diff", "--", SOURCE_PATH, CONSUMER_PATH], worktree)
            require(diff.returncode == 0 and diff.stdout, "PATCH_EMPTY:" + definition["mutation_id"])
            patch_path, transcript_path = artifact_paths(definition)
            patch = {
                "schema": "nostr_automerge.causal_projection_mutation_patch.v18.v1",
                "mutation_id": definition["mutation_id"],
                "encoding": "unified_diff_utf8",
                "patch": diff.stdout,
                "result": "pass",
            }
            patch_raw = json.dumps(patch, ensure_ascii=True, indent=2) + "\n"
            patch_path.write_text(patch_raw)
            compiled = run(COMPILE_ARGV, worktree)
            checked = run(PROPERTY_ARGV, worktree)
            actual_code = property_code(checked)
            source_path.write_text(baseline_source)
            consumer_path.write_text(baseline_consumer)
            restoration_argv = ["git", "diff", "--quiet", "--", SOURCE_PATH, CONSUMER_PATH]
            restored = run(restoration_argv, worktree)
            survivor = checked.returncode != 1 or actual_code != definition["expected"]
            require(compiled.returncode == 0, "COMPILE:" + definition["mutation_id"])
            require(restored.returncode == 0, "RESTORATION:" + definition["mutation_id"])
            require(not survivor, f"SURVIVOR:{definition['mutation_id']}:{actual_code}")
            transcript = {
                "schema": "nostr_automerge.causal_projection_mutation_transcript.v18.v1",
                "authority": AUTHORITY,
                "mutation_id": definition["mutation_id"],
                "campaign": definition["campaign"],
                "inventory_row_id": definition["inventory_row_id"],
                "kind": definition["kind"],
                "source_candidate": SOURCE_CANDIDATE,
                "execution_base_candidate": execution_base,
                "patch_artifact": relative(patch_path),
                "patch_sha256": sha(patch_raw.encode()),
                "compile": command_record(COMPILE_ARGV, compiled),
                "property": command_record(PROPERTY_ARGV, checked),
                "restoration": command_record(restoration_argv, restored),
                "expected_property_code": definition["expected"],
                "actual_property_code": actual_code,
                "shared_helper_unchanged": shared_helper_unchanged,
                "survivor": False,
                "result": "killed",
            }
            transcript_raw = json.dumps(transcript, ensure_ascii=True, indent=2) + "\n"
            transcript_path.write_text(transcript_raw)
            rows.append(row_from_transcript(definition, patch_raw, transcript_raw, transcript))
    finally:
        run(["git", "worktree", "remove", "--force", str(worktree)], ROOT)
        try:
            temp_root.rmdir()
        except OSError:
            pass
    return execution_base, rows, preflight


def row_from_transcript(
    definition: dict[str, str],
    patch_raw: str,
    transcript_raw: str,
    transcript: dict[str, Any],
) -> dict[str, Any]:
    require(list(transcript) == TRANSCRIPT_FIELDS, "TRANSCRIPT_SHAPE:" + definition["mutation_id"])
    compile_record = transcript["compile"]
    property_record = transcript["property"]
    restoration_record = transcript["restoration"]
    return {
        "mutation_id": definition["mutation_id"],
        "campaign": definition["campaign"],
        "inventory_row_id": definition["inventory_row_id"],
        "kind": definition["kind"],
        "expected_property_code": definition["expected"],
        "actual_property_code": transcript["actual_property_code"],
        "compile_command": compile_record["argv"],
        "compile_exit_status": compile_record["exit_status"],
        "compile_output_sha256": compile_record["output_sha256"],
        "property_command": property_record["argv"],
        "property_exit_status": property_record["exit_status"],
        "property_output_sha256": property_record["output_sha256"],
        "restoration_command": restoration_record["argv"],
        "restoration_result": "pass" if restoration_record["exit_status"] == 0 else "fail",
        "execution_envelope": {"cwd": ".", "environment": ENVIRONMENT},
        "patch_artifact": transcript["patch_artifact"],
        "patch_sha256": sha(patch_raw.encode()),
        "transcript_artifact": relative(artifact_paths(definition)[1]),
        "transcript_sha256": sha(transcript_raw.encode()),
        "shared_helper_unchanged": transcript["shared_helper_unchanged"],
        "source_candidate": transcript["source_candidate"],
        "execution_base_candidate": transcript["execution_base_candidate"],
        "survivor": transcript["survivor"],
        "result": transcript["result"],
    }


def load_row(definition: dict[str, str]) -> dict[str, Any]:
    patch_path, transcript_path = artifact_paths(definition)
    patch_raw, transcript_raw = patch_path.read_text(), transcript_path.read_text()
    patch, transcript = json.loads(patch_raw), json.loads(transcript_raw)
    require(patch == {
        "schema": "nostr_automerge.causal_projection_mutation_patch.v18.v1",
        "mutation_id": definition["mutation_id"],
        "encoding": "unified_diff_utf8",
        "patch": patch["patch"],
        "result": "pass",
    }, "PATCH_SHAPE:" + definition["mutation_id"])
    require(transcript["patch_sha256"] == sha(patch_raw.encode()), "PATCH_IDENTITY:" + definition["mutation_id"])
    require(list(transcript) == TRANSCRIPT_FIELDS, "TRANSCRIPT_SHAPE:" + definition["mutation_id"])
    require(transcript["mutation_id"] == definition["mutation_id"], "TRANSCRIPT_ID:" + definition["mutation_id"])
    require(transcript["campaign"] == definition["campaign"] and transcript["kind"] == definition["kind"], "TRANSCRIPT_CLASS:" + definition["mutation_id"])
    require(transcript["inventory_row_id"] == definition["inventory_row_id"], "TRANSCRIPT_INVENTORY:" + definition["mutation_id"])
    require(transcript["source_candidate"] == SOURCE_CANDIDATE, "TRANSCRIPT_SOURCE:" + definition["mutation_id"])
    require(transcript["compile"]["argv"] == COMPILE_ARGV, "COMPILE_COMMAND:" + definition["mutation_id"])
    require(transcript["property"]["argv"] == PROPERTY_ARGV, "PROPERTY_COMMAND:" + definition["mutation_id"])
    for name in ("compile", "property", "restoration"):
        record = transcript[name]
        require(record["cwd"] == "." and record["environment"] == ENVIRONMENT, "COMMAND_ENVELOPE:" + definition["mutation_id"])
        require(record["output_sha256"] == sha(canonical({"stdout_sha256": record["stdout_sha256"], "stderr_sha256": record["stderr_sha256"]})), "COMMAND_OUTPUT_IDENTITY:" + definition["mutation_id"])
    require(transcript["expected_property_code"] == transcript["actual_property_code"] == definition["expected"], "PROPERTY_CODE:" + definition["mutation_id"])
    require(transcript["compile"]["exit_status"] == 0, "COMPILE_RESULT:" + definition["mutation_id"])
    require(transcript["property"]["exit_status"] == 1, "PROPERTY_RESULT:" + definition["mutation_id"])
    require(transcript["restoration"]["exit_status"] == 0, "RESTORATION_RESULT:" + definition["mutation_id"])
    require(transcript["survivor"] is False and transcript["result"] == "killed", "MUTATION_RESULT:" + definition["mutation_id"])
    if definition["campaign"] == "direct":
        require(transcript["shared_helper_unchanged"] is True, "DIRECT_HELPER_CHANGED:" + definition["mutation_id"])
    return row_from_transcript(definition, patch_raw, transcript_raw, transcript)


def expected_report(
    execution_base: str, rows: list[dict[str, Any]], preflight: dict[str, Any]
) -> dict[str, Any]:
    campaigns = {
        name: sum(row["campaign"] == name for row in rows)
        for name in ("direct", "helper", "provenance")
    }
    report: dict[str, Any] = {
        "schema": "nostr_automerge.causal_projection_mutations.v18.v1",
        "status": "actual_execution_raw_unbound",
        "authority": AUTHORITY,
        "source_candidate": SOURCE_CANDIDATE,
        "execution_base_candidate": execution_base,
        "row_contract": ROW_FIELDS,
        "rows": rows,
        "counts": {
            "mutations": len(rows),
            "campaigns": campaigns,
            "compile_passed": sum(row["compile_exit_status"] == 0 for row in rows),
            "properties_killed": sum(not row["survivor"] for row in rows),
            "survivors": sum(row["survivor"] for row in rows),
        },
        "execution": {
            "worktree": "isolated_detached",
            "property_process": "isolated_root_subprocess",
            "structural_identity_split": True,
            "compile_property_records": "separate",
            "artifact_commit_binding": "later_catalog",
            "restored": True,
        },
        "preflight": preflight,
        "result_identity_sha256": "",
        "result": "pass",
    }
    identity = {key: value for key, value in report.items() if key != "result_identity_sha256"}
    report["result_identity_sha256"] = sha(canonical(identity))
    return report


def validate(report: Any, schema: Any, contract: dict[str, Any]) -> None:
    require(type(report) is dict and list(report) == TOP_FIELDS, "REPORT_SHAPE")
    campaign = definitions(contract)
    rows = [load_row(definition) for definition in campaign]
    require(report == expected_report(report["execution_base_candidate"], rows, report["preflight"]), "REPORT_DERIVATION_MISMATCH")
    require(report["source_candidate"] == SOURCE_CANDIDATE, "SOURCE_CANDIDATE")
    execution_base = report["execution_base_candidate"]
    require(run(["git", "rev-parse", f"{execution_base}^{{commit}}"], ROOT).stdout.strip() == execution_base, "EXECUTION_BASE")
    require(run(["git", "merge-base", "--is-ancestor", SOURCE_CANDIDATE, execution_base], ROOT).returncode == 0, "CANDIDATE_ANCESTRY")
    require(run(["git", "cat-file", "-e", f"{execution_base}:scripts/validate_causal_projection_properties_v18.py"], ROOT).returncode == 0, "ORACLE_NOT_COMMITTED")
    require(run(["git", "cat-file", "-e", f"{execution_base}:scripts/run_causal_projection_mutations_v18.py"], ROOT).returncode == 0, "RUNNER_NOT_COMMITTED")
    direct = [row for row in report["rows"] if row["campaign"] == "direct"]
    require([row["mutation_id"].split(".")[1] for row in direct] == contract["mutation"]["mandatory_direct_sites"], "DIRECT_SITE_COVERAGE")
    require(all(row["shared_helper_unchanged"] for row in direct), "DIRECT_HELPER_UNCHANGED")
    require(report["counts"]["survivors"] == 0 and report["execution"]["restored"], "CAMPAIGN_RESULT")
    expected_artifacts = {relative(path) for definition in campaign for path in artifact_paths(definition)}
    actual_artifacts = {relative(path) for path in OUT.glob("*.json")}
    require(actual_artifacts == expected_artifacts, "ARTIFACT_SET")
    require(report["preflight"]["identity"]["exit_status"] == 0, "IDENTITY_PREFLIGHT")
    require(report["preflight"]["structural"]["exit_status"] == 0, "STRUCTURAL_PREFLIGHT")
    require(report["preflight"]["identity"]["argv"] == IDENTITY_ARGV, "IDENTITY_PREFLIGHT_COMMAND")
    require(report["preflight"]["structural"]["argv"] == PROPERTY_ARGV, "STRUCTURAL_PREFLIGHT_COMMAND")
    require(type(schema) is dict and schema.get("additionalProperties") is False and schema.get("required") == TOP_FIELDS, "SCHEMA_CLOSED")
    row_schema = schema["properties"]["rows"]
    require(row_schema.get("minItems") == 1 and "maxItems" not in row_schema, "SCHEMA_DERIVED_COUNT")
    require(row_schema["items"].get("additionalProperties") is False and row_schema["items"].get("required") == ROW_FIELDS, "SCHEMA_ROW_CLOSED")


def self_test(report: dict[str, Any], schema: dict[str, Any], contract: dict[str, Any]) -> int:
    attacks = [
        ("missing", "report", lambda value: value["rows"].pop()),
        ("duplicate", "report", lambda value: value["rows"].__setitem__(1, copy.deepcopy(value["rows"][0]))),
        ("code", "report", lambda value: value["rows"][0].update(actual_property_code="SURVIVED")),
        ("compile", "report", lambda value: value["rows"][0].update(compile_exit_status=1)),
        ("property", "report", lambda value: value["rows"][0].update(property_exit_status=0)),
        ("patch", "report", lambda value: value["rows"][0].update(patch_sha256="0" * 64)),
        ("transcript", "report", lambda value: value["rows"][0].update(transcript_sha256="0" * 64)),
        ("base", "report", lambda value: value.update(execution_base_candidate="0" * 40)),
        ("survivor", "report", lambda value: value["rows"][0].update(survivor=True)),
        ("schema", "schema", lambda value: value.update(additionalProperties=True)),
    ]
    caught = 0
    for label, target, mutate in attacks:
        changed_report, changed_schema = copy.deepcopy(report), copy.deepcopy(schema)
        mutate(changed_report if target == "report" else changed_schema)
        try:
            validate(changed_report, changed_schema, contract)
        except MutationError:
            caught += 1
            continue
        raise MutationError(f"MUTATION_SURVIVED:{label}")
    return caught


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--execute", action="store_true")
    parser.add_argument("--write-report", action="store_true")
    args = parser.parse_args()
    contract = json.loads((ROOT / CONTRACT_PATH).read_text())
    if args.execute:
        execution_base, rows, preflight = execute()
        generated = expected_report(execution_base, rows, preflight)
        if args.write_report:
            REPORT.write_text(json.dumps(generated, ensure_ascii=True, indent=2) + "\n")
    require(not args.write_report or args.execute, "WRITE_REQUIRES_EXECUTION")
    report = json.loads(REPORT.read_text())
    schema = json.loads(SCHEMA.read_text())
    validate(report, schema, contract)
    attacks = self_test(report, schema, contract)
    print(
        "PASS: causal projection mutations v18 "
        f"actual={len(report['rows'])} direct={report['counts']['campaigns']['direct']} "
        f"survivors={report['counts']['survivors']} attacks={attacks} restored=pass"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
