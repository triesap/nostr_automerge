#!/usr/bin/env python3
"""Run isolated actual v17 causal-projection mutation campaigns."""

from __future__ import annotations

import argparse
import hashlib
import json
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
INVENTORY = json.loads((ROOT / "reports/causal_projection_inventory_v17.json").read_text())
PROPERTIES = json.loads((ROOT / "reports/causal_projection_properties_v17.json").read_text())
BASE_CANDIDATE = "89bd44daa54749fe40ac8eb963a27e9b11a91da4"
OUT = ROOT / "reports/evidence/v17/mutations"
REPORT = ROOT / "reports/causal_projection_construction_mutations_v17.json"
SCHEMA = ROOT / "tools/validation/causal_projection_construction_mutations_v17.schema.json"
DIRECT_REPORT = ROOT / "reports/causal_projection_direct_mutations_v17.json"
DIRECT_SCHEMA = ROOT / "tools/validation/causal_projection_direct_mutations_v17.schema.json"
DIRECT_BASE_CANDIDATE = "2b316789bd55a8b0ce099d4c12baeab53205b38f"
SITES = [
    ("DependencyOrderCompare", "MemberOrderCompare"),
    ("ReadyCandidatePull", "NextMemberPull"),
    ("ReadyDependantInsert", "InitialReadyInsert"),
]
KINDS = [
    ("wrapper_bypass", "ALTERNATE_CONSUMER_BYPASS", False),
    ("double_target", "TARGET_AFTER_STOP", True),
    ("target_before_charge", "CHARGE_AFTER_OPERATION", True),
    ("same_family_site_swap", "SITE_ID_MISMATCH", True),
]
DIRECT_SITES = [
    ("actor_sequence", "ActorStateRead", "perform_actor_decision_operation"),
    ("actor_sequence", "PredecessorCandidateRead", "perform_actor_decision_operation"),
    ("actor_sequence", "ActorIdentityDecision", "perform_actor_decision_operation"),
    ("actor_sequence", "SequenceRelationDecision", "perform_actor_decision_operation"),
    ("causal_counter", "StoredCounterRead", "perform_causal_next_operation"),
    ("causal_counter", "ExpectedStartComparison", "perform_causal_next_operation"),
    ("causal_counter", "CheckedAdvance", "perform_causal_next_operation"),
]

sys.path.insert(0, str(ROOT / "scripts"))
from validate_causal_projection_structure_v17 import StructuralError, validate_structure  # noqa: E402


class MutationError(RuntimeError):
    pass


def require(condition: bool, code: str) -> None:
    if not condition:
        raise MutationError(code)


def sha(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def run(command: list[str], cwd: Path) -> subprocess.CompletedProcess[str]:
    return subprocess.run(command, cwd=cwd, capture_output=True, text=True, check=False)


def mutate_helper(source: str, replacement: str) -> str:
    marker = "fn perform_projection_build_operation<T, E>("
    head, tail = source.split(marker, 1)
    return head + marker + tail.replace(
        "charge(descriptor.counter).map_err(MeteredActorStateError::Work)?;\n    let result = perform();",
        replacement,
        1,
    )


def mutation(source: str, site: str, swap: str, kind: str) -> str:
    if kind == "wrapper_bypass":
        changed, count = re.subn(
            rf"perform_projection_build_operation(?=\(\s*ProjectionBuildSite::{site}\b)",
            "bypass_projection_build_operation",
            source,
            count=1,
        )
    elif kind == "double_target":
        targets = {
            "DependencyOrderCompare": ("|| previous < dependency,", "|| { let result = previous < dependency; let _uncharged_second_result = previous < dependency; result },"),
            "ReadyCandidatePull": ("|| ready.pop_first(),", "|| { let result = ready.pop_first(); let _uncharged_second_result = ready.pop_first(); result },"),
            "ReadyDependantInsert": ("|| ready.insert(child),", "|| { let result = ready.insert(child); let _uncharged_second_result = ready.insert(child); result },"),
        }
        old, new = targets[site]
        changed = source.replace(old, new, 1)
        count = int(changed != source)
    elif kind == "target_before_charge":
        changed = mutate_helper(source, "let result = perform();\n    charge(descriptor.counter).map_err(MeteredActorStateError::Work)?;")
        count = int(changed != source)
    else:
        changed, count = re.subn(
            rf"(perform_projection_build_operation\(\s*)ProjectionBuildSite::{site}\b",
            rf"\g<1>ProjectionBuildSite::{swap}",
            source,
            count=1,
        )
    require(count == 1, f"PATCH_TARGET:{site}:{kind}")
    boundary = "\n#[cfg(test)]\npub(crate) mod tests {"
    marker = f"\n// v17 mutation {site}:{kind}\n#[cfg(test)]\npub(crate) mod tests {{"
    return changed.replace(boundary, marker, 1)


def normalized_compile(completed: subprocess.CompletedProcess[str], expected: bool) -> str:
    classification = "pass" if completed.returncode == 0 else "fail"
    if (completed.returncode == 0) != expected:
        raise MutationError(f"COMPILE_CLASSIFICATION:{classification}:{expected}\n{completed.stdout[-1200:]}\n{completed.stderr[-1200:]}")
    return classification


def execute() -> list[dict[str, Any]]:
    rows: list[dict[str, Any]] = []
    construction_dir = OUT / "construction"
    construction_dir.mkdir(parents=True, exist_ok=True)
    for obsolete in construction_dir.glob("*.patch"):
        obsolete.unlink()
    temp_root = Path(tempfile.mkdtemp(prefix="nostr-automerge-v17-mutations-", dir=ROOT.parents[2]))
    worktree = temp_root / "worktree"
    added = run(["git", "worktree", "add", "--detach", str(worktree), BASE_CANDIDATE], ROOT)
    require(added.returncode == 0, "WORKTREE_ADD")
    try:
        doctor = run(["cargo", "extbuild", "doctor"], worktree)
        require(doctor.returncode == 0, "EXTBUILD_DOCTOR")
        source_path = worktree / SOURCE_PATH
        baseline = source_path.read_text()
        consumer = (worktree / CONSUMER_PATH).read_text()
        for site, swap in SITES:
            for kind, expected_code, expected_compile in KINDS:
                mutation_id = f"construction.{site}.{kind}"
                changed = mutation(baseline, site, swap, kind)
                source_path.write_text(changed)
                patch = run(["git", "diff", "--", SOURCE_PATH], worktree)
                require(patch.returncode == 0 and patch.stdout, f"PATCH_EMPTY:{mutation_id}")
                artifact_base = OUT / "construction" / mutation_id.replace(".", "_")
                artifact_base.parent.mkdir(parents=True, exist_ok=True)
                patch_path = artifact_base.parent / f"{artifact_base.name}.patch.json"
                transcript_path = artifact_base.with_suffix(".txt")
                patch_artifact = json.dumps({"encoding": "unified_diff_utf8", "patch": patch.stdout}, ensure_ascii=True, indent=2) + "\n"
                patch_path.write_text(patch_artifact)
                compiled = run(["cargo", "extbuild", "run", "--", "cargo", "check", "-p", "nostr_automerge", "--lib", "--locked"], worktree)
                compile_result = normalized_compile(compiled, expected_compile)
                try:
                    validate_structure(changed, consumer, INVENTORY, PROPERTIES)
                except StructuralError as error:
                    actual_code = error.code
                else:
                    actual_code = "SURVIVED"
                survivor = actual_code != expected_code
                transcript = "\n".join((
                    f"mutation_id={mutation_id}", f"inventory_row_id=rust.projection_construction.{re.sub(r'(?<!^)(?=[A-Z])', '_', site).lower()}",
                    "command=cargo check -p nostr_automerge --lib --locked", f"compile_expected={'pass' if expected_compile else 'fail'}",
                    f"compile_actual={compile_result}", f"expected_property={expected_code}", f"actual_property={actual_code}",
                    f"survivor={str(survivor).lower()}", "restoration=pending", "",
                ))
                transcript_path.write_text(transcript)
                source_path.write_text(baseline)
                restored = run(["git", "diff", "--quiet", "--", SOURCE_PATH], worktree).returncode == 0
                require(restored and not survivor, f"MUTATION_RESULT:{mutation_id}")
                transcript = transcript.replace("restoration=pending", "restoration=pass")
                transcript_path.write_text(transcript)
                rows.append({
                    "mutation_id": mutation_id,
                    "inventory_row_id": f"rust.projection_construction.{re.sub(r'(?<!^)(?=[A-Z])', '_', site).lower()}",
                    "kind": kind, "expected_property_code": expected_code, "actual_property_code": actual_code,
                    "compile_expected": "pass" if expected_compile else "fail", "compile_actual": compile_result,
                    "patch_artifact": patch_path.relative_to(ROOT).as_posix(), "patch_sha256": sha(patch_artifact.encode()),
                    "transcript_artifact": transcript_path.relative_to(ROOT).as_posix(), "transcript_sha256": sha(transcript.encode()),
                    "restoration": "pass", "survivor": False, "result": "killed",
                })
    finally:
        run(["git", "worktree", "remove", "--force", str(worktree)], ROOT)
        temp_root.rmdir()
    return rows


def expected_report(rows: list[dict[str, Any]]) -> dict[str, Any]:
    return {
        "schema": "nostr_automerge.causal_projection_construction_mutations.v17.v1",
        "status": "actual_execution", "base_candidate": BASE_CANDIDATE,
        "campaign": "three_non_first_repeated_construction_sites",
        "rows": rows,
        "counts": {"sites": 3, "mutations": 12, "compile_pass": 9, "compile_expected_fail": 3, "survivors": 0},
        "worktree": {"isolated": True, "extbuild": True, "restored": True}, "result": "pass",
    }


def mutate_direct(source: str, helper: str, site: str, kind: str) -> str:
    if kind == "charge_before_target":
        head, tail = source.split(f"fn {helper}", 1)
        tail = tail.replace(
            "charge(descriptor.counter).map_err(MeteredActorStateError::Work)?;\n    let result = perform();",
            "let result = perform();\n    charge(descriptor.counter).map_err(MeteredActorStateError::Work)?;",
            1,
        )
        changed = head + f"fn {helper}" + tail
    elif kind == "charge_removal":
        head, tail = source.split("fn perform_actor_decision_operation", 1)
        tail = tail.replace(
            "charge(descriptor.counter).map_err(MeteredActorStateError::Work)?;",
            "let _v17_charge_removed = descriptor.counter;",
            1,
        )
        changed = head + "fn perform_actor_decision_operation" + tail
    elif kind == "double_target":
        changed = source.replace(
            "|| self.actor_states.get(&candidate.actor).copied(),",
            "|| { let result = self.actor_states.get(&candidate.actor).copied(); let _uncharged_second_result = self.actor_states.get(&candidate.actor).copied(); result },",
            1,
        )
    elif kind == "observer_before_target":
        old = "let result = perform();\n    observed(ActorDecisionObservation {\n        descriptor,\n        kind: ActorDecisionObservationKind::TargetCompleted,\n    });"
        new = "observed(ActorDecisionObservation {\n        descriptor,\n        kind: ActorDecisionObservationKind::TargetCompleted,\n    });\n    let result = perform();"
        changed = source.replace(old, new, 1)
    else:
        head, tail = source.split("fn perform_actor_decision_operation", 1)
        tail = tail.replace(
            "charge(descriptor.counter).map_err(MeteredActorStateError::Work)?;",
            "let _v17_ignored_charge = charge(descriptor.counter).map_err(MeteredActorStateError::Work);",
            1,
        )
        changed = head + "fn perform_actor_decision_operation" + tail
    require(changed != source, f"DIRECT_PATCH_TARGET:{site}:{kind}")
    boundary = "\n#[cfg(test)]\npub(crate) mod tests {"
    return changed.replace(boundary, f"\n// v17 direct mutation {site}:{kind}\n#[cfg(test)]\npub(crate) mod tests {{", 1)


def execute_direct() -> list[dict[str, Any]]:
    definitions = [
        *((phase, site, helper, "charge_before_target", "CHARGE_AFTER_OPERATION") for phase, site, helper in DIRECT_SITES),
        ("actor_sequence", "ActorStateRead", "perform_actor_decision_operation", "charge_removal", "CHARGE_AFTER_OPERATION"),
        ("actor_sequence", "ActorStateRead", "perform_actor_decision_operation", "double_target", "TARGET_AFTER_STOP"),
        ("actor_sequence", "ActorStateRead", "perform_actor_decision_operation", "observer_before_target", "OBSERVATION_AFTER_STOP"),
        ("actor_sequence", "ActorStateRead", "perform_actor_decision_operation", "target_after_failed_charge", "TARGET_AFTER_STOP"),
    ]
    direct_dir = OUT / "direct"
    direct_dir.mkdir(parents=True, exist_ok=True)
    temp_root = Path(tempfile.mkdtemp(prefix="nostr-automerge-v17-direct-", dir=ROOT.parents[2]))
    worktree = temp_root / "worktree"
    require(run(["git", "worktree", "add", "--detach", str(worktree), DIRECT_BASE_CANDIDATE], ROOT).returncode == 0, "DIRECT_WORKTREE_ADD")
    rows: list[dict[str, Any]] = []
    try:
        require(run(["cargo", "extbuild", "doctor"], worktree).returncode == 0, "DIRECT_EXTBUILD_DOCTOR")
        source_path = worktree / SOURCE_PATH
        baseline = source_path.read_text()
        consumer = (worktree / CONSUMER_PATH).read_text()
        for phase, site, helper, kind, expected_code in definitions:
            mutation_id = f"direct.{site}.{kind}"
            changed = mutate_direct(baseline, helper, site, kind)
            source_path.write_text(changed)
            patch = run(["git", "diff", "--", SOURCE_PATH], worktree)
            require(patch.returncode == 0 and patch.stdout, f"DIRECT_PATCH_EMPTY:{mutation_id}")
            artifact_name = mutation_id.replace(".", "_")
            patch_path = direct_dir / f"{artifact_name}.patch.json"
            transcript_path = direct_dir / f"{artifact_name}.txt"
            patch_artifact = json.dumps({"encoding": "unified_diff_utf8", "patch": patch.stdout}, ensure_ascii=True, indent=2) + "\n"
            patch_path.write_text(patch_artifact)
            compiled = run(["cargo", "extbuild", "run", "--", "cargo", "check", "-p", "nostr_automerge", "--lib", "--locked"], worktree)
            compile_result = normalized_compile(compiled, True)
            try:
                validate_structure(changed, consumer, INVENTORY, PROPERTIES)
            except StructuralError as error:
                actual_code = error.code
            else:
                actual_code = "SURVIVED"
            survivor = actual_code != expected_code
            source_path.write_text(baseline)
            restored = run(["git", "diff", "--quiet", "--", SOURCE_PATH], worktree).returncode == 0
            require(restored and not survivor, f"DIRECT_MUTATION_RESULT:{mutation_id}:{actual_code}")
            row_id = f"rust.{phase}.{re.sub(r'(?<!^)(?=[A-Z])', '_', site).lower()}"
            transcript = "\n".join((
                f"mutation_id={mutation_id}", f"inventory_row_id={row_id}",
                "command=cargo check -p nostr_automerge --lib --locked", "compile_expected=pass",
                f"compile_actual={compile_result}", f"expected_property={expected_code}", f"actual_property={actual_code}",
                f"survivor={str(survivor).lower()}", "restoration=pass", "",
            ))
            transcript_path.write_text(transcript)
            rows.append({
                "mutation_id": mutation_id, "inventory_row_id": row_id, "kind": kind,
                "expected_property_code": expected_code, "actual_property_code": actual_code,
                "compile_expected": "pass", "compile_actual": compile_result,
                "patch_artifact": patch_path.relative_to(ROOT).as_posix(), "patch_sha256": sha(patch_artifact.encode()),
                "transcript_artifact": transcript_path.relative_to(ROOT).as_posix(), "transcript_sha256": sha(transcript.encode()),
                "restoration": "pass", "survivor": False, "result": "killed",
            })
    finally:
        run(["git", "worktree", "remove", "--force", str(worktree)], ROOT)
        temp_root.rmdir()
    return rows


def expected_direct_report(rows: list[dict[str, Any]]) -> dict[str, Any]:
    return {
        "schema": "nostr_automerge.causal_projection_direct_mutations.v17.v1",
        "status": "actual_execution", "base_candidate": DIRECT_BASE_CANDIDATE,
        "campaign": "every_actor_and_causal_direct_site", "rows": rows,
        "counts": {"direct_sites": 7, "per_site_charge_before_target": 7, "representative": 4, "mutations": 11, "compile_pass": 11, "survivors": 0},
        "worktree": {"isolated": True, "extbuild": True, "restored": True}, "result": "pass",
    }


def validate(report: dict[str, Any], schema: dict[str, Any]) -> None:
    require(report["status"] == "actual_execution" and len(report["rows"]) == 12, "REPORT_ROWS")
    require(len({row["mutation_id"] for row in report["rows"]}) == 12, "REPORT_UNIQUE")
    require({row["inventory_row_id"] for row in report["rows"]} == {f"rust.projection_construction.{re.sub(r'(?<!^)(?=[A-Z])', '_', site).lower()}" for site, _ in SITES}, "REPORT_SITES")
    for row in report["rows"]:
        require(sha((ROOT / row["patch_artifact"]).read_bytes()) == row["patch_sha256"], "PATCH_IDENTITY")
        require(sha((ROOT / row["transcript_artifact"]).read_bytes()) == row["transcript_sha256"], "TRANSCRIPT_IDENTITY")
        require(row["expected_property_code"] == row["actual_property_code"] and not row["survivor"], "MUTATION_SURVIVOR")
    require(report["counts"]["survivors"] == 0 and report["worktree"]["restored"], "CAMPAIGN_RESULT")
    require(schema.get("additionalProperties") is False and schema.get("required") == list(report), "SCHEMA_CLOSED")


def validate_direct(report: dict[str, Any], schema: dict[str, Any]) -> None:
    require(report["status"] == "actual_execution" and len(report["rows"]) == 11, "DIRECT_REPORT_ROWS")
    require(len({row["mutation_id"] for row in report["rows"]}) == 11, "DIRECT_REPORT_UNIQUE")
    per_site = [row for row in report["rows"] if row["kind"] == "charge_before_target"]
    require(len(per_site) == 7 and len({row["inventory_row_id"] for row in per_site}) == 7, "DIRECT_SITE_COVERAGE")
    for row in report["rows"]:
        require(sha((ROOT / row["patch_artifact"]).read_bytes()) == row["patch_sha256"], "DIRECT_PATCH_IDENTITY")
        require(sha((ROOT / row["transcript_artifact"]).read_bytes()) == row["transcript_sha256"], "DIRECT_TRANSCRIPT_IDENTITY")
        require(row["expected_property_code"] == row["actual_property_code"] and not row["survivor"], "DIRECT_SURVIVOR")
    require(report["counts"]["survivors"] == 0 and report["worktree"]["restored"], "DIRECT_CAMPAIGN_RESULT")
    require(schema.get("additionalProperties") is False and schema.get("required") == list(report), "DIRECT_SCHEMA_CLOSED")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--campaign", choices=("construction", "direct", "all"), default="all")
    parser.add_argument("--execute", action="store_true")
    args = parser.parse_args()
    if args.execute and args.campaign in ("construction", "all"):
        rows = execute()
        REPORT.write_text(json.dumps(expected_report(rows), ensure_ascii=True, indent=2) + "\n")
    if args.execute and args.campaign in ("direct", "all"):
        rows = execute_direct()
        DIRECT_REPORT.write_text(json.dumps(expected_direct_report(rows), ensure_ascii=True, indent=2) + "\n")
    report = json.loads(REPORT.read_text())
    schema = json.loads(SCHEMA.read_text())
    validate(report, schema)
    direct_report = json.loads(DIRECT_REPORT.read_text())
    direct_schema = json.loads(DIRECT_SCHEMA.read_text())
    validate_direct(direct_report, direct_schema)
    print("PASS: causal projection mutations v17 construction=12 direct=11 survivors=0 restored=pass")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
