#!/usr/bin/env python3
"""Run deterministic material mutations and publish evidence only when none survive."""

from __future__ import annotations

import argparse
import json
import os
import subprocess
from dataclasses import dataclass
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
OUTPUT = Path(os.environ.get("NOSTR_AUTOMERGE_OUTPUT_ROOT", ROOT / ".local/evidence"))
COMMAND_TIMEOUT_SECONDS = int(os.environ.get("NOSTR_AUTOMERGE_MUTATION_TIMEOUT", "180"))


@dataclass(frozen=True)
class Mutation:
    category: str
    path: str
    search: str
    replacement: str
    test_filter: str
    test_target: str | None = None


MUTATIONS = (
    Mutation("limit", "crates/nostr_automerge/src/wire/base64.rs", "if input.len() > encoded_maximum {", "if input.len() >= encoded_maximum {", "wire::base64::tests"),
    Mutation("canonicalization", "crates/nostr_automerge/src/wire/base64.rs", "if STANDARD.encode(&decoded) != input {", "if STANDARD.encode(&decoded) == input {", "wire::base64::tests"),
    Mutation("checkpoint", "crates/nostr_automerge/src/checkpoint/merkle.rs", "hash.update([0]);\n    hash.update(super::MERKLE_DOMAIN);\n    hash.update([0]);\n    hash.update(index.to_be_bytes());", "hash.update([1]);\n    hash.update(super::MERKLE_DOMAIN);\n    hash.update([0]);\n    hash.update(index.to_be_bytes());", "checkpoint::merkle::tests"),
    Mutation("checkpoint", "crates/nostr_automerge/src/checkpoint/merkle.rs", "fn node(left: [u8; 32], right: [u8; 32]) -> [u8; 32] {\n    let mut hash = Sha256::new();\n    hash.update([1]);", "fn node(left: [u8; 32], right: [u8; 32]) -> [u8; 32] {\n    let mut hash = Sha256::new();\n    hash.update([0]);", "checkpoint::merkle::tests"),
    Mutation("limit", "crates/nostr_automerge/src/checkpoint/merkle.rs", "if leaves.is_empty() || leaves.len() > super::MAX_CHUNK_COUNT as usize {", "if leaves.is_empty() && leaves.len() > super::MAX_CHUNK_COUNT as usize {", "checkpoint::merkle::tests"),
    Mutation("checkpoint", "crates/nostr_automerge/src/checkpoint/merkle.rs", "if expected.len() != proof.len() {", "if expected.len() == proof.len() {", "checkpoint::merkle::tests"),
    Mutation("consensus", "crates/nostr_automerge/src/control/select.rs", ".filter(|candidate| Some(*candidate) != selected)", ".filter(|candidate| Some(*candidate) == selected)", "control::select::tests"),
    Mutation("equivocation", "crates/nostr_automerge/src/reference/epoch_engine.rs", "|| dispositions.get(&candidate.change_hash) == Some(&ProtocolDisposition::Accepted)", "|| dispositions.get(&candidate.change_hash) != Some(&ProtocolDisposition::Excluded)", "cannot_poison", "public_engine_api"),
    Mutation("graph", "crates/nostr_automerge/src/graph/topology.rs", "if !missing.is_empty() {", "if missing.is_empty() {", "graph::topology::tests"),
    Mutation("graph", "crates/nostr_automerge/src/graph/topology.rs", "if order.len() != graph.nodes.len() {", "if order.len() == graph.nodes.len() {", "graph::topology::tests"),
    Mutation("projection", "crates/nostr_automerge/src/automerge_adapter/materialized_view.rs", "find(|entry| entry.path == path)?", "find(|entry| entry.path != path)?", "automerge_adapter::materialized_view::tests"),
    Mutation("projection", "crates/nostr_automerge/src/automerge_adapter/materialized_view.rs", "pub const fn start(&self) -> u64 {\n        self.start\n    }", "pub const fn start(&self) -> u64 {\n        self.end\n    }", "automerge_adapter::materialized_view::tests"),
    Mutation("projection", "crates/nostr_automerge/src/automerge_adapter/materialized_view.rs", "for key in keys {", "for key in Vec::<String>::new() {", "automerge_adapter::materialized_view::tests"),
    Mutation("projection", "crates/nostr_automerge/src/automerge_adapter/materialized_view.rs", "for index in 0..document.length(&current.object) {", "for index in 0..0 {", "automerge_adapter::materialized_view::tests"),
)


def cargo(*arguments: str) -> int | None:
    try:
        return subprocess.run(
            ["cargo", "extbuild", "run", "--", "cargo", *arguments],
            cwd=ROOT,
            stdout=subprocess.DEVNULL,
            stderr=subprocess.STDOUT,
            check=False,
            timeout=COMMAND_TIMEOUT_SECONDS,
        ).returncode
    except subprocess.TimeoutExpired:
        return None


def write_report(name: str, report: dict[str, object]) -> None:
    OUTPUT.mkdir(parents=True, exist_ok=True)
    (OUTPUT / name).write_text(
        json.dumps(report, sort_keys=True, separators=(",", ":")) + "\n",
        encoding="utf-8",
    )


def merge() -> int:
    reports = [
        json.loads((OUTPUT / f"rust_mutation_{index:02d}.json").read_text())
        for index in range(1, len(MUTATIONS) + 1)
    ]
    if any(report.get("status") != "pass" or report.get("generated") != 1 for report in reports):
        raise AssertionError("mutation shard is missing or did not pass")
    merged = {
        "caught": [item for report in reports for item in report["caught"]],
        "generated": len(MUTATIONS),
        "schema": "nostr_automerge.rust_mutation.v1",
        "status": "pass",
        "survived": [],
        "timeouts": [],
        "tool": "repository deterministic source mutator v1",
        "unviable": [item for report in reports for item in report["unviable"]],
        "unviable_policy": "excluded only when the mutated library does not compile",
    }
    if len(merged["caught"]) + len(merged["unviable"]) != len(MUTATIONS):
        raise AssertionError("mutation shards do not cover the closed campaign")
    write_report("rust_mutation_summary.json", merged)
    print(f"PASS: merged {len(MUTATIONS)} deterministic Rust mutation shards")
    return 0


def main() -> int:
    parser = argparse.ArgumentParser()
    group = parser.add_mutually_exclusive_group()
    group.add_argument("--index", type=int)
    group.add_argument("--merge", action="store_true")
    args = parser.parse_args()
    if args.merge:
        return merge()
    if args.index is not None and not 1 <= args.index <= len(MUTATIONS):
        parser.error(f"--index must be between 1 and {len(MUTATIONS)}")

    caught: list[str] = []
    unviable: list[str] = []
    survived: list[str] = []
    timeouts: list[str] = []
    selected = enumerate(MUTATIONS, start=1)
    if args.index is not None:
        selected = ((args.index, MUTATIONS[args.index - 1]),)
    for index, mutation in selected:
        path = ROOT / mutation.path
        original = path.read_text(encoding="utf-8")
        if original.count(mutation.search) != 1:
            raise AssertionError(f"stale mutation anchor: {mutation.path}:{index}")
        mutated = original.replace(mutation.search, mutation.replacement, 1)
        identity = f"{index:02d}:{mutation.category}:{mutation.path}"
        try:
            path.write_text(mutated, encoding="utf-8")
            build = cargo(
                "test", "--no-run", "-p", "nostr_automerge", "--lib", "--locked"
            )
            if build is None:
                timeouts.append(identity)
                continue
            if build != 0:
                unviable.append(identity)
                continue
            test_arguments = ["test", "-p", "nostr_automerge"]
            if mutation.test_target is None:
                test_arguments.append("--lib")
            else:
                test_arguments.extend(["--test", mutation.test_target])
            test_arguments.extend([mutation.test_filter, "--locked"])
            result = cargo(*test_arguments)
            if result is None:
                timeouts.append(identity)
            else:
                (caught if result != 0 else survived).append(identity)
        finally:
            path.write_text(original, encoding="utf-8")
    report = {
        "caught": caught,
        "generated": 1 if args.index is not None else len(MUTATIONS),
        "schema": "nostr_automerge.rust_mutation.v1",
        "status": "pass" if not survived and not timeouts else "fail",
        "survived": survived,
        "timeouts": timeouts,
        "tool": "repository deterministic source mutator v1",
        "unviable": unviable,
        "unviable_policy": "excluded only when the mutated library does not compile",
    }
    name = (
        f"rust_mutation_{args.index:02d}.json"
        if args.index is not None
        else "rust_mutation_summary.json"
    )
    write_report(name, report)
    if survived or timeouts:
        raise AssertionError(f"material mutation did not fail cleanly: {survived + timeouts}")
    print(f"PASS: caught {len(caught)} material Rust mutations; {len(unviable)} unviable")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
