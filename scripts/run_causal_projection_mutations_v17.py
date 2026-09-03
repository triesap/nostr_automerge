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


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--campaign", choices=("construction",), default="construction")
    parser.add_argument("--execute", action="store_true")
    args = parser.parse_args()
    if args.execute:
        rows = execute()
        REPORT.write_text(json.dumps(expected_report(rows), ensure_ascii=True, indent=2) + "\n")
    report = json.loads(REPORT.read_text())
    schema = json.loads(SCHEMA.read_text())
    validate(report, schema)
    print("PASS: causal projection construction mutations v17 actual=12 sites=3 survivors=0 restored=pass")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
