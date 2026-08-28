#!/usr/bin/env python3
"""Execute and validate remediation-v12 finding reproductions."""

from __future__ import annotations

import argparse
import copy
import json
import pathlib
import subprocess


ROOT = pathlib.Path(__file__).resolve().parents[1]
INVENTORY = ROOT / "spec/remediation_v12_reproductions.json"
EXPECTED_CASES = [
    {
        "finding": "FINDING_100",
        "family": "actor_predecessor",
        "kind": "rust_failure",
        "path": "crates/nostr_automerge/src/graph/actor_state.rs",
        "test": "graph::actor_state::tests::finding_100_actor_predecessor_scan_reproduction",
        "diagnostic": "unmetered actor predecessor collection remains",
        "expected": "fixed_pass",
    },
    {
        "finding": "FINDING_100",
        "family": "causal_next_op",
        "kind": "rust_failure",
        "path": "crates/nostr_automerge/src/graph/actor_state.rs",
        "test": "graph::actor_state::tests::finding_100_causal_next_op_scan_reproduction",
        "diagnostic": "unmetered causal next-op scan remains",
        "expected": "fixed_pass",
    },
    {
        "finding": "FINDING_100",
        "family": "empty_frontier",
        "kind": "rust_failure",
        "path": "crates/nostr_automerge/src/graph/actor_state.rs",
        "test": "graph::actor_state::tests::finding_100_empty_frontier_work_reproduction",
        "diagnostic": "unmetered empty-frontier allocation remains",
        "expected": "open_failure",
    },
    {
        "finding": "FINDING_100",
        "family": "epoch_ancestry",
        "kind": "rust_failure",
        "path": "crates/nostr_automerge/src/graph/epoch.rs",
        "test": "graph::epoch::tests::finding_100_epoch_ancestry_work_reproduction",
        "diagnostic": "unmetered epoch ancestry materialization remains",
        "expected": "open_failure",
    },
    {
        "finding": "FINDING_100",
        "family": "epoch_writer_authorization",
        "kind": "rust_failure",
        "path": "crates/nostr_automerge/src/reference/epoch_engine.rs",
        "test": "reference::epoch_engine::tests::finding_100_epoch_writer_authorization_work_reproduction",
        "diagnostic": "unmetered epoch writer authorization scan remains",
        "expected": "open_failure",
    },
    {
        "finding": "FINDING_100",
        "family": "dependency_closure",
        "kind": "rust_failure",
        "path": "crates/nostr_automerge/src/graph/closure.rs",
        "test": "graph::closure::tests::finding_100_dependency_closure_work_reproduction",
        "diagnostic": "unmetered dependency-closure preparation remains",
        "expected": "open_failure",
    },
    {
        "finding": "FINDING_100",
        "family": "schedule_readiness",
        "kind": "rust_failure",
        "path": "crates/nostr_automerge/src/graph/schedule.rs",
        "test": "graph::schedule::tests::finding_100_schedule_readiness_work_reproduction",
        "diagnostic": "unmetered schedule readiness and pop preparation remains",
        "expected": "open_failure",
    },
    {
        "finding": "FINDING_100",
        "family": "schedule_publication",
        "kind": "rust_failure",
        "path": "crates/nostr_automerge/src/graph/schedule.rs",
        "test": "graph::schedule::tests::finding_100_schedule_publication_work_reproduction",
        "diagnostic": "unmetered schedule insertion and result publication remains",
        "expected": "open_failure",
    },
    {
        "finding": "FINDING_100",
        "family": "quarantine_overlays",
        "kind": "rust_failure",
        "path": "crates/nostr_automerge/src/reference/epoch_engine.rs",
        "test": "reference::epoch_engine::tests::finding_100_quarantine_overlay_work_reproduction",
        "diagnostic": "unmetered selected and fallback quarantine overlays remain",
        "expected": "open_failure",
    },
    {
        "finding": "FINDING_100",
        "family": "zero_post_stop",
        "kind": "rust_failure",
        "path": "crates/nostr_automerge/src/reference/epoch_engine.rs",
        "test": "reference::epoch_engine::tests::finding_100_zero_post_stop_work_reproduction",
        "diagnostic": "unmetered target preparation remains before the first stop",
        "expected": "open_failure",
    },
]


class ReproductionError(AssertionError):
    pass


def require(condition: bool, diagnostic: str) -> None:
    if not condition:
        raise ReproductionError(diagnostic)


def validate_inventory(value: object) -> list[dict[str, str]]:
    require(isinstance(value, dict) and tuple(value) == ("schema", "cases", "result"), "inventory:keys")
    require(value["schema"] == "nostr_automerge.remediation_v12_reproductions.v1" and value["result"] == "pass", "inventory:identity")
    rows = value["cases"]
    require(isinstance(rows, list) and len(rows) == len(EXPECTED_CASES), "inventory:count")
    required = ("finding", "family", "kind", "path", "test", "diagnostic", "expected")
    for index, row in enumerate(rows):
        require(isinstance(row, dict) and tuple(row) == required, f"case:{index}:keys")
        require(row == EXPECTED_CASES[index], f"case:{index}:identity")
        require((ROOT / row["path"]).is_file(), f"case:{index}:path")
    return rows


def validate_failure_transcript(row: dict[str, str], result: subprocess.CompletedProcess[str]) -> None:
    require(result.returncode == 101, "transcript:returncode")
    require(f"test {row['test']} ... FAILED" in result.stdout, "transcript:test")
    require(row["diagnostic"] in result.stdout, "transcript:diagnostic")
    require("test result: FAILED. 0 passed; 1 failed; 0 ignored;" in result.stdout, "transcript:summary")
    require(f"failures:\n    {row['test']}\n" in result.stdout, "transcript:failure_list")


def validate_pass_transcript(row: dict[str, str], result: subprocess.CompletedProcess[str]) -> None:
    require(result.returncode == 0, "transcript:pass_returncode")
    require(
        result.stdout.count(f"test {row['test']} ... ok") == 1,
        "transcript:pass_test",
    )
    require("test result: ok. 1 passed; 0 failed; 0 ignored;" in result.stdout, "transcript:pass_summary")


def rust_case(row: dict[str, str]) -> bool:
    command = [
        "cargo", "extbuild", "run", "--", "cargo", "test", "-p",
        "nostr_automerge", "--lib", "--locked", "--",
    ]
    if row["expected"] == "open_failure":
        command.append("--ignored")
    command.extend(("--exact", row["test"]))
    result = subprocess.run(command, cwd=ROOT, capture_output=True, text=True, check=False)
    if row["expected"] == "open_failure":
        validate_failure_transcript(row, result)
        return True
    validate_pass_transcript(row, result)
    return False


def verify_state(rows: list[dict[str, str]]) -> tuple[int, int]:
    fixed = 0
    opened = 0
    for row in rows:
        reproduced = rust_case(row)
        if row["expected"] == "open_failure":
            require(reproduced, "case:not_reproduced")
            opened += 1
        else:
            require(not reproduced, "case:still_reproduced")
            fixed += 1
    return fixed, opened


def mutation_self_test(value: object) -> int:
    mutators = (
        lambda item: item["cases"].clear(),
        lambda item: item["cases"].append(copy.deepcopy(item["cases"][0])),
        lambda item: item["cases"][0].update(finding="FINDING_101"),
        lambda item: item["cases"][0].update(family="other"),
        lambda item: item["cases"][0].update(kind="source_failure"),
        lambda item: item["cases"][0].update(path="missing"),
        lambda item: item["cases"][0].update(test="other"),
        lambda item: item["cases"][0].update(diagnostic=""),
        lambda item: item["cases"][0].update(expected="closed"),
        lambda item: item.update(extra=False),
    )
    caught = 0
    for mutate in mutators:
        candidate = copy.deepcopy(value)
        mutate(candidate)
        try:
            validate_inventory(candidate)
        except ReproductionError:
            caught += 1
            continue
        raise ReproductionError("inventory mutation survived")
    rows = validate_inventory(value)
    row = rows[2]
    stdout = (
        f"running 1 test\ntest {row['test']} ... FAILED\n\n"
        f"{row['diagnostic']}\n\nfailures:\n    {row['test']}\n\n"
        "test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out;\n"
    )
    base = subprocess.CompletedProcess([], 101, stdout, "")
    validate_failure_transcript(row, base)
    for changed in (
        subprocess.CompletedProcess([], 0, stdout, ""),
        subprocess.CompletedProcess([], 101, stdout.replace(row["test"], "other", 1), ""),
        subprocess.CompletedProcess([], 101, stdout.replace(row["diagnostic"], "other"), ""),
        subprocess.CompletedProcess([], 101, stdout.replace("0 passed; 1 failed", "1 passed; 0 failed"), ""),
        subprocess.CompletedProcess([], 101, stdout.replace(f"failures:\n    {row['test']}\n", "failures:\n"), ""),
    ):
        try:
            validate_failure_transcript(row, changed)
        except ReproductionError:
            caught += 1
            continue
        raise ReproductionError("transcript mutation survived")
    fixed_row = rows[0]
    pass_stdout = (
        f"running 1 test\ntest {fixed_row['test']} ... ok\n\n"
        "test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out;\n"
    )
    pass_base = subprocess.CompletedProcess([], 0, pass_stdout, "")
    validate_pass_transcript(fixed_row, pass_base)
    for changed in (
        subprocess.CompletedProcess([], 101, pass_stdout, ""),
        subprocess.CompletedProcess([], 0, pass_stdout.replace(fixed_row["test"], "other"), ""),
        subprocess.CompletedProcess([], 0, pass_stdout.replace(" ... ok", " ... ignored"), ""),
        subprocess.CompletedProcess([], 0, pass_stdout.replace("1 passed", "0 passed"), ""),
        subprocess.CompletedProcess([], 0, pass_stdout + f"test {fixed_row['test']} ... ok\n", ""),
    ):
        try:
            validate_pass_transcript(fixed_row, changed)
        except ReproductionError:
            caught += 1
            continue
        raise ReproductionError("pass transcript mutation survived")
    return caught


def main() -> None:
    parser = argparse.ArgumentParser()
    modes = parser.add_mutually_exclusive_group(required=True)
    modes.add_argument("--verify-state", action="store_true")
    modes.add_argument("--self-test-only", action="store_true")
    args = parser.parse_args()
    value = json.loads(INVENTORY.read_text())
    rows = validate_inventory(value)
    mutations = mutation_self_test(value)
    fixed, opened = verify_state(rows) if args.verify_state else (0, 0)
    print("PASS: remediation v12 reproduction harness")
    print(f"- fixed={fixed}")
    print(f"- open={opened}")
    print(f"- mutations={mutations}")


if __name__ == "__main__":
    main()
