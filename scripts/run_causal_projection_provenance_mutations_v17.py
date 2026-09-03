#!/usr/bin/env python3
"""Execute the six property-specific v17 provenance mutations."""

from __future__ import annotations

import argparse
import hashlib
import json
import subprocess
import sys
import tempfile
from pathlib import Path
from typing import Any

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parents[1]
SOURCE_PATH = "crates/nostr_automerge/src/graph/actor_state.rs"
CONSUMER_PATH = "crates/nostr_automerge/src/reference/epoch_engine.rs"
BASE_CANDIDATE = "597f4e8b5762dddbb086cb08dbf8b5fd0278e02e"
REPORT = ROOT / "reports/causal_projection_provenance_mutations_v17.json"
SCHEMA = ROOT / "tools/validation/causal_projection_provenance_mutations_v17.schema.json"
OUT = ROOT / "reports/evidence/v17/mutations/provenance"
INVENTORY = json.loads((ROOT / "reports/causal_projection_inventory_v17.json").read_text())
PROPERTIES = json.loads((ROOT / "reports/causal_projection_properties_v17.json").read_text())
CASES = [
    ("typed_stop_collapse", "TYPED_BUDGET_EXHAUSTED_IDENTITY", "rust.actor_sequence.actor_state_read"),
    ("cancellation_collapse", "TYPED_CANCELLED_IDENTITY", "rust.actor_sequence.actor_state_read"),
    ("unexpected_error_replacement", "UNEXPECTED_WORK_ERROR_IDENTITY", "rust.actor_sequence.actor_state_read"),
    ("target_after_failed_charge", "TARGET_AFTER_STOP", "rust.actor_sequence.actor_state_read"),
    ("observation_after_failed_charge", "OBSERVATION_AFTER_STOP", "rust.actor_sequence.actor_state_read"),
    ("publication_after_failed_charge", "PUBLICATION_AFTER_STOP", "rust.projection_construction.projection_publish"),
]

sys.path.insert(0, str(ROOT / "scripts"))
from validate_causal_projection_structure_v17 import StructuralError, validate_structure  # noqa: E402


class CampaignError(RuntimeError):
    pass


def require(condition: bool, code: str) -> None:
    if not condition:
        raise CampaignError(code)


def sha(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def run(command: list[str], cwd: Path) -> subprocess.CompletedProcess[str]:
    return subprocess.run(command, cwd=cwd, capture_output=True, text=True, check=False)


def actor_helper_replace(source: str, old: str, new: str) -> str:
    marker = "fn perform_actor_decision_operation"
    head, tail = source.split(marker, 1)
    changed = head + marker + tail.replace(old, new, 1)
    require(changed != source, "ACTOR_HELPER_PATCH")
    return changed


def mutate(source: str, kind: str) -> str:
    charge = "charge(descriptor.counter).map_err(MeteredActorStateError::Work)?;"
    if kind in ("typed_stop_collapse", "cancellation_collapse", "unexpected_error_replacement"):
        marker = {
            "typed_stop_collapse": "_v17_typed_stop_collapsed",
            "cancellation_collapse": "_v17_cancellation_collapsed",
            "unexpected_error_replacement": "_v17_unexpected_error_replaced",
        }[kind]
        replacement = (
            f"let {marker} = ();\n    "
            "charge(descriptor.counter)\n        .map_err(|_| MeteredActorStateError::State(ActorStateError::NoncanonicalInput))?;"
        )
        changed = actor_helper_replace(source, charge, replacement)
    elif kind == "target_after_failed_charge":
        changed = actor_helper_replace(source, charge, "let _v17_ignored_charge = charge(descriptor.counter).map_err(MeteredActorStateError::Work);")
    elif kind == "observation_after_failed_charge":
        old = (
            f"{charge}\n    let result = perform();\n"
            "    observed(ActorDecisionObservation {\n        descriptor,\n        kind: ActorDecisionObservationKind::TargetCompleted,\n    });"
        )
        new = (
            "observed(ActorDecisionObservation {\n        descriptor,\n        kind: ActorDecisionObservationKind::TargetCompleted,\n    });\n"
            f"    {charge}\n    let result = perform();"
        )
        changed = actor_helper_replace(source, old, new)
    else:
        publication = "    published(ProjectionPublicationOperation::Projection);\n"
        changed = source.replace(publication, "", 1)
        changed = changed.replace(
            "    let projection = perform_projection_build_operation(\n        ProjectionBuildSite::ProjectionPublish,",
            publication + "    let projection = perform_projection_build_operation(\n        ProjectionBuildSite::ProjectionPublish,",
            1,
        )
        require(changed != source, "PUBLICATION_PATCH")
    boundary = "\n#[cfg(test)]\npub(crate) mod tests {"
    return changed.replace(boundary, f"\n// v17 provenance mutation {kind}\n#[cfg(test)]\npub(crate) mod tests {{", 1)


def execute() -> list[dict[str, Any]]:
    OUT.mkdir(parents=True, exist_ok=True)
    temp_root = Path(tempfile.mkdtemp(prefix="nostr-automerge-v17-provenance-", dir=ROOT.parents[2]))
    worktree = temp_root / "worktree"
    require(run(["git", "worktree", "add", "--detach", str(worktree), BASE_CANDIDATE], ROOT).returncode == 0, "WORKTREE_ADD")
    rows: list[dict[str, Any]] = []
    try:
        require(run(["cargo", "extbuild", "doctor"], worktree).returncode == 0, "EXTBUILD_DOCTOR")
        source_path = worktree / SOURCE_PATH
        baseline = source_path.read_text()
        consumer = (worktree / CONSUMER_PATH).read_text()
        for kind, expected_code, inventory_row_id in CASES:
            changed = mutate(baseline, kind)
            source_path.write_text(changed)
            patch = run(["git", "diff", "--", SOURCE_PATH], worktree)
            require(patch.returncode == 0 and patch.stdout, f"PATCH_EMPTY:{kind}")
            patch_artifact = json.dumps({"encoding": "unified_diff_utf8", "patch": patch.stdout}, ensure_ascii=True, indent=2) + "\n"
            patch_path = OUT / f"{kind}.patch.json"
            transcript_path = OUT / f"{kind}.txt"
            patch_path.write_text(patch_artifact)
            compiled = run(["cargo", "extbuild", "run", "--", "cargo", "check", "-p", "nostr_automerge", "--lib", "--locked"], worktree)
            require(compiled.returncode == 0, f"COMPILE:{kind}:{compiled.stderr[-500:]}")
            try:
                validate_structure(changed, consumer, INVENTORY, PROPERTIES)
            except StructuralError as error:
                actual_code = error.code
            else:
                actual_code = "SURVIVED"
            survivor = actual_code != expected_code
            source_path.write_text(baseline)
            restored = run(["git", "diff", "--quiet", "--", SOURCE_PATH], worktree).returncode == 0
            require(restored and not survivor, f"RESULT:{kind}:{actual_code}")
            transcript = "\n".join((
                f"mutation_id=provenance.{kind}", f"inventory_row_id={inventory_row_id}",
                "command=cargo check -p nostr_automerge --lib --locked", "compile_actual=pass",
                f"expected_property={expected_code}", f"actual_property={actual_code}",
                "wrong_property_is_survivor=true", "survivor=false", "restoration=pass", "",
            ))
            transcript_path.write_text(transcript)
            rows.append({
                "mutation_id": f"provenance.{kind}", "inventory_row_id": inventory_row_id,
                "expected_property_code": expected_code, "actual_property_code": actual_code,
                "compile_actual": "pass", "patch_artifact": patch_path.relative_to(ROOT).as_posix(),
                "patch_sha256": sha(patch_artifact.encode()), "transcript_artifact": transcript_path.relative_to(ROOT).as_posix(),
                "transcript_sha256": sha(transcript.encode()), "wrong_property_is_survivor": True,
                "restoration": "pass", "survivor": False, "result": "killed",
            })
    finally:
        run(["git", "worktree", "remove", "--force", str(worktree)], ROOT)
        temp_root.rmdir()
    return rows


def document(rows: list[dict[str, Any]]) -> dict[str, Any]:
    return {
        "schema": "nostr_automerge.causal_projection_provenance_mutations.v17.v1",
        "status": "actual_execution", "base_candidate": BASE_CANDIDATE,
        "rows": rows, "counts": {"mutations": 6, "compile_pass": 6, "survivors": 0},
        "worktree": {"isolated": True, "extbuild": True, "restored": True}, "result": "pass",
    }


def validate(report: dict[str, Any], schema: dict[str, Any]) -> None:
    require(report["status"] == "actual_execution" and len(report["rows"]) == 6, "ROWS")
    require([row["expected_property_code"] for row in report["rows"]] == [case[1] for case in CASES], "PROPERTY_ORDER")
    for row in report["rows"]:
        require(row["expected_property_code"] == row["actual_property_code"] and not row["survivor"], "SURVIVOR")
        require(sha((ROOT / row["patch_artifact"]).read_bytes()) == row["patch_sha256"], "PATCH_IDENTITY")
        require(sha((ROOT / row["transcript_artifact"]).read_bytes()) == row["transcript_sha256"], "TRANSCRIPT_IDENTITY")
    require(schema.get("additionalProperties") is False and schema.get("required") == list(report), "SCHEMA_CLOSED")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--execute", action="store_true")
    args = parser.parse_args()
    if args.execute:
        REPORT.write_text(json.dumps(document(execute()), ensure_ascii=True, indent=2) + "\n")
    report = json.loads(REPORT.read_text())
    validate(report, json.loads(SCHEMA.read_text()))
    print("PASS: causal projection provenance mutations v17 actual=6 compile=6 survivors=0 restored=pass")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
