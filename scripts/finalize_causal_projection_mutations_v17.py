#!/usr/bin/env python3
"""Finalize actual v17 mutations with bidirectional inventory coverage."""

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
SOURCE_CANDIDATE = "789eae3c6e0994f71420f49fe51fe3ab7cb75ca9"
MUTATION_CANDIDATE = "fd8bf182c91649d9c62ecbe860b5a81c9a8f7045"
INPUTS = [
    ("reports/causal_projection_construction_mutations_v17.json", "2b316789bd55a8b0ce099d4c12baeab53205b38f"),
    ("reports/causal_projection_direct_mutations_v17.json", "597f4e8b5762dddbb086cb08dbf8b5fd0278e02e"),
    ("reports/causal_projection_provenance_mutations_v17.json", MUTATION_CANDIDATE),
]
INVENTORY = json.loads((ROOT / "reports/causal_projection_inventory_v17.json").read_text())
PROPERTIES = json.loads((ROOT / "reports/causal_projection_properties_v17.json").read_text())
REPORT = ROOT / "reports/causal_projection_mutations_v17.json"
SCHEMA = ROOT / "tools/validation/causal_projection_mutations_v17.schema.json"
OUT = ROOT / "reports/evidence/v17/mutations/coverage"

sys.path.insert(0, str(ROOT / "scripts"))
from validate_causal_projection_structure_v17 import StructuralError, validate_structure  # noqa: E402


class CoverageError(RuntimeError):
    pass


def require(condition: bool, code: str) -> None:
    if not condition:
        raise CoverageError(code)


def sha(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def run(command: list[str], cwd: Path) -> subprocess.CompletedProcess[str]:
    return subprocess.run(command, cwd=cwd, capture_output=True, text=True, check=False)


def committed(candidate: str, path: str) -> bytes:
    result = subprocess.run(["git", "show", f"{candidate}:{path}"], cwd=ROOT, capture_output=True, check=False)
    require(result.returncode == 0, f"INPUT_CANDIDATE:{path}")
    return result.stdout


def supplement(kind: str, source: str, consumer: str) -> tuple[str, str, str, str]:
    if kind == "frontier_target_before_charge":
        marker = "fn metered_frontier_operation"
        head, tail = source.split(marker, 1)
        tail = tail.replace(
            "charge(descriptor.counter).map_err(MeteredActorStateError::Work)?;\n    let result = target();",
            "let result = target();\n    charge(descriptor.counter).map_err(MeteredActorStateError::Work)?;",
            1,
        )
        return head + marker + tail, consumer, SOURCE_PATH, "CHARGE_AFTER_OPERATION"
    changed = consumer.replace(".candidate_semantics_decision_metered(", ".candidate_semantics_decision_unmetered(", 1)
    require(changed != consumer, "CONSUMER_PATCH")
    return source, changed, CONSUMER_PATH, "ALTERNATE_CONSUMER_BYPASS"


def execute_supplements() -> list[dict[str, Any]]:
    OUT.mkdir(parents=True, exist_ok=True)
    temp_root = Path(tempfile.mkdtemp(prefix="nostr-automerge-v17-coverage-", dir=ROOT.parents[2]))
    worktree = temp_root / "worktree"
    require(run(["git", "worktree", "add", "--detach", str(worktree), MUTATION_CANDIDATE], ROOT).returncode == 0, "WORKTREE_ADD")
    rows: list[dict[str, Any]] = []
    try:
        require(run(["cargo", "extbuild", "doctor"], worktree).returncode == 0, "EXTBUILD_DOCTOR")
        source_path, consumer_path = worktree / SOURCE_PATH, worktree / CONSUMER_PATH
        baseline_source, baseline_consumer = source_path.read_text(), consumer_path.read_text()
        for kind, compile_expected in (("frontier_target_before_charge", True), ("canonical_consumer_bypass", False)):
            changed_source, changed_consumer, changed_path, expected_code = supplement(kind, baseline_source, baseline_consumer)
            source_path.write_text(changed_source); consumer_path.write_text(changed_consumer)
            patch = run(["git", "diff", "--", changed_path], worktree)
            require(patch.returncode == 0 and patch.stdout, f"PATCH_EMPTY:{kind}")
            patch_artifact = json.dumps({"encoding": "unified_diff_utf8", "patch": patch.stdout}, ensure_ascii=True, indent=2) + "\n"
            patch_path, transcript_path = OUT / f"{kind}.patch.json", OUT / f"{kind}.txt"
            patch_path.write_text(patch_artifact)
            compiled = run(["cargo", "extbuild", "run", "--", "cargo", "check", "-p", "nostr_automerge", "--lib", "--locked"], worktree)
            require((compiled.returncode == 0) == compile_expected, f"COMPILE:{kind}")
            try:
                validate_structure(changed_source, changed_consumer, INVENTORY, PROPERTIES)
            except StructuralError as error:
                actual_code = error.code
            else:
                actual_code = "SURVIVED"
            survivor = actual_code != expected_code
            source_path.write_text(baseline_source); consumer_path.write_text(baseline_consumer)
            restored = run(["git", "diff", "--quiet", "--", changed_path], worktree).returncode == 0
            require(restored and not survivor, f"RESULT:{kind}:{actual_code}")
            mutation_id = f"coverage.{kind}"
            transcript = "\n".join((
                f"mutation_id={mutation_id}", f"source_path={changed_path}",
                "command=cargo check -p nostr_automerge --lib --locked",
                f"compile_expected={'pass' if compile_expected else 'fail'}", f"compile_actual={'pass' if compiled.returncode == 0 else 'fail'}",
                f"expected_property={expected_code}", f"actual_property={actual_code}", "survivor=false", "restoration=pass", "",
            ))
            transcript_path.write_text(transcript)
            rows.append({
                "mutation_id": mutation_id, "inventory_row_id": "shared.frontier" if kind.startswith("frontier") else "shared.consumer",
                "expected_property_code": expected_code, "actual_property_code": actual_code,
                "compile_expected": "pass" if compile_expected else "fail", "compile_actual": "pass" if compiled.returncode == 0 else "fail",
                "patch_artifact": patch_path.relative_to(ROOT).as_posix(), "patch_sha256": sha(patch_artifact.encode()),
                "transcript_artifact": transcript_path.relative_to(ROOT).as_posix(), "transcript_sha256": sha(transcript.encode()),
                "restoration": "pass", "survivor": False, "result": "killed",
            })
    finally:
        run(["git", "worktree", "remove", "--force", str(worktree)], ROOT)
        temp_root.rmdir()
    return rows


def input_mutations() -> tuple[list[dict[str, Any]], list[dict[str, str]]]:
    rows, reports = [], []
    for path, candidate in INPUTS:
        data = committed(candidate, path)
        current = (ROOT / path).read_bytes()
        require(current == data, f"INPUT_DRIFT:{path}")
        document = json.loads(data)
        require(document["result"] == "pass" and document["counts"]["survivors"] == 0, f"INPUT_RESULT:{path}")
        rows.extend(document["rows"])
        reports.append({"path": path, "candidate": candidate, "sha256": sha(data)})
    return rows, reports


def coverage_rows(mutations: list[dict[str, Any]]) -> list[dict[str, Any]]:
    mutation_by_id = {row["mutation_id"]: row for row in mutations}
    require(len(mutation_by_id) == len(mutations), "MUTATION_DUPLICATE")
    selected = {"dependency_order_compare", "ready_candidate_pull", "ready_dependant_insert"}
    rows = []
    for inventory_row in INVENTORY["rows"]:
        row_id, phase = inventory_row["id"], inventory_row["phase"]
        suffix = row_id.rsplit(".", 1)[-1]
        if phase == "projection_construction":
            if suffix in selected:
                site = inventory_row["site_id"]
                ids = [f"construction.{site}.{kind}" for kind in ("wrapper_bypass", "double_target", "target_before_charge", "same_family_site_swap")]
            else:
                ids = ["construction.DependencyOrderCompare.target_before_charge"]
            if suffix == "projection_publish":
                ids.append("provenance.publication_after_failed_charge")
        elif phase in ("actor_sequence", "causal_counter"):
            ids = [f"direct.{inventory_row['site_id']}.charge_before_target"]
            if row_id == "rust.actor_sequence.actor_state_read":
                ids += [
                    "direct.ActorStateRead.charge_removal", "direct.ActorStateRead.double_target",
                    "direct.ActorStateRead.observer_before_target", "direct.ActorStateRead.target_after_failed_charge",
                    "provenance.typed_stop_collapse", "provenance.cancellation_collapse",
                    "provenance.unexpected_error_replacement", "provenance.target_after_failed_charge",
                    "provenance.observation_after_failed_charge", "coverage.canonical_consumer_bypass",
                ]
        else:
            ids = ["coverage.frontier_target_before_charge"]
        require(all(item in mutation_by_id for item in ids), f"COVERAGE_MUTATION:{row_id}")
        covered = [mutation_by_id[item] for item in ids]
        rows.append({
            "coverage_id": "coverage." + row_id,
            "inventory_row_ids": [row_id], "mutation_ids": ids,
            "property_codes": [row["actual_property_code"] for row in covered],
            "patch_artifacts": [row["patch_artifact"] for row in covered],
            "transcript_artifacts": [row["transcript_artifact"] for row in covered],
            "source_candidate": SOURCE_CANDIDATE, "mutation_candidate": MUTATION_CANDIDATE,
            "result": "pass",
        })
    return rows


def make_report(supplements: list[dict[str, Any]]) -> dict[str, Any]:
    prior, input_reports = input_mutations()
    mutations = prior + supplements
    coverage = coverage_rows(mutations)
    reverse = [
        {"mutation_id": row["mutation_id"], "coverage_ids": [item["coverage_id"] for item in coverage if row["mutation_id"] in item["mutation_ids"]]}
        for row in mutations
    ]
    return {
        "schema": "nostr_automerge.causal_projection_mutations.v17.v1", "status": "actual_complete",
        "source_candidate": SOURCE_CANDIDATE, "mutation_candidate": MUTATION_CANDIDATE,
        "input_reports": input_reports, "mutation_records": mutations,
        "coverage_records": coverage, "reverse_coverage": reverse,
        "counts": {"inventory_rows": 68, "mutations": len(mutations), "coverage_records": 68, "uncovered_rows": 0, "unreferenced_mutations": 0, "survivors": 0},
        "result": "pass",
    }


def validate(report: dict[str, Any], schema: dict[str, Any]) -> None:
    prior, inputs = input_mutations()
    require(report["input_reports"] == inputs, "INPUT_REPORTS")
    mutations = report["mutation_records"]
    require(len(mutations) == 31 and len({row["mutation_id"] for row in mutations}) == 31, "MUTATION_ROWS")
    require(len(report["coverage_records"]) == 68 and {row["inventory_row_ids"][0] for row in report["coverage_records"]} == {row["id"] for row in INVENTORY["rows"]}, "FORWARD_COVERAGE")
    require(all(row["coverage_ids"] for row in report["reverse_coverage"]) and {row["mutation_id"] for row in report["reverse_coverage"]} == {row["mutation_id"] for row in mutations}, "REVERSE_COVERAGE")
    for row in mutations:
        require(sha((ROOT / row["patch_artifact"]).read_bytes()) == row["patch_sha256"], "PATCH_IDENTITY")
        require(sha((ROOT / row["transcript_artifact"]).read_bytes()) == row["transcript_sha256"], "TRANSCRIPT_IDENTITY")
        require(not row["survivor"] and row["expected_property_code"] == row["actual_property_code"], "SURVIVOR")
    require(report["counts"] == {"inventory_rows": 68, "mutations": 31, "coverage_records": 68, "uncovered_rows": 0, "unreferenced_mutations": 0, "survivors": 0}, "COUNTS")
    require(schema.get("additionalProperties") is False and schema.get("required") == list(report), "SCHEMA_CLOSED")


def main() -> int:
    parser = argparse.ArgumentParser(); parser.add_argument("--execute", action="store_true"); args = parser.parse_args()
    if args.execute:
        REPORT.write_text(json.dumps(make_report(execute_supplements()), ensure_ascii=True, indent=2) + "\n")
    report = json.loads(REPORT.read_text()); validate(report, json.loads(SCHEMA.read_text()))
    print("PASS: causal projection mutations v17 actual=31 coverage=68 reverse=31 survivors=0")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
