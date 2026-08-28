#!/usr/bin/env python3
"""Execute reviewed causal-projection mutations in isolated Git worktrees."""

from __future__ import annotations

import argparse
import hashlib
import json
import subprocess
import sys
import tempfile
from pathlib import Path

sys.dont_write_bytecode = True

ROOT = Path(__file__).resolve().parents[1]
TARGET = "crates/nostr_automerge/src/graph/actor_state.rs"
TEST = "graph::actor_state::tests::projection_semantic_matrix_is_complete_and_order_invariant"
BEFORE = """causal_next_op = perform_projection_build_operation(
            WorkCounter::GraphNode,
            ProjectionBuildOperation::CausalMaximumCompare,
            &mut charge,
            &mut built,
            || causal_next_op.max(advanced),
        )?;"""
AFTER = BEFORE.replace(".max(advanced)", ".min(advanced)")


class MutationError(RuntimeError):
    pass


def require(condition: bool, label: str) -> None:
    if not condition:
        raise MutationError(label)


def mutation_identity() -> str:
    value = json.dumps(
        {"id":"projection_max_to_min","path":TARGET,"before":BEFORE,"after":AFTER,"test":TEST},
        separators=(",", ":"),
        sort_keys=True,
    ).encode("utf-8")
    return hashlib.sha256(value).hexdigest()


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
    checkout = Path(tempfile.mkdtemp(prefix="nostr-causal-mutation-"))
    added = False
    try:
        added_result = subprocess.run(
            ["git", "worktree", "add", "--detach", str(checkout), "HEAD"],
            cwd=ROOT,
            capture_output=True,
            text=True,
            check=False,
        )
        require(added_result.returncode == 0, "worktree:add")
        added = True
        target = checkout / TARGET
        source = target.read_text(encoding="utf-8")
        require(source.count(BEFORE) == 1, "mutation:anchor")
        target.write_text(source.replace(BEFORE, AFTER, 1), encoding="utf-8")
        command = [
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
        require(transcript_is_exact_failure(result.returncode, output), "mutation:transcript")
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


def self_test() -> int:
    exact = f"test {TEST} ... FAILED\ntest result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s\n"
    require(transcript_is_exact_failure(101, exact), "transcript:positive")
    mutations = (
        (101, exact.replace(TEST, "unrelated", 1)),
        (0, exact.replace("FAILED", "ok")),
        (101, exact.replace("0 ignored", "1 ignored")),
        (101, "error: could not compile `nostr_automerge`\n"),
        (101, exact + "test result: FAILED. 0 passed; 1 failed; 0 ignored\n"),
        (0, f"test {TEST} ... ok\ntest result: ok. 1 passed; 0 failed; 0 ignored\n"),
        (101, exact + f"test {TEST} ... FAILED\n"),
    )
    for index, (returncode, output) in enumerate(mutations):
        require(not transcript_is_exact_failure(returncode, output), f"transcript:{index}")
    require(len(mutation_identity()) == 64, "mutation:identity")
    return len(mutations) + 2


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--run-selected", action="store_true")
    args = parser.parse_args()
    mutations = self_test()
    if args.run_selected:
        run_selected()
    print(
        "PASS: causal-projection isolated mutation runner "
        f"selected=1 mutations={mutations} identity={mutation_identity()}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
