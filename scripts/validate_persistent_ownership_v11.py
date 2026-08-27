#!/usr/bin/env python3
"""Validate the closed bounded-persistent-ownership gate for remediation v11."""

from __future__ import annotations

import argparse
import copy
import hashlib
import json
import pathlib
import re
import subprocess
import sys

sys.dont_write_bytecode = True

ROOT = pathlib.Path(__file__).resolve().parents[1]
REPORT = ROOT / "reports/persistent_ownership_v11.json"
SCHEMA = ROOT / "tools/validation/persistent_ownership_v11.schema.json"
REPORT_SHA256 = "10235d1eac0b09a2b22ba70959a47a06478a08f595b31c9f843bb9fb41dcc67f"
SCHEMA_SHA256 = "0c8127f9a5cd45b159894ecf947a00ada4b7d9857e2909fbb028550004cd6548"
CANDIDATES = (
    ("step_1335", "ca9870f1ac0a4c70b7df5b9a4c437c06dde61fcd"),
    ("step_1336", "cf7150e820ec1625dd0027a160fe3cb362b5b629"),
    ("step_1337", "cce9236d7d791c8caafb91fb3f4e0b3f4c5b4b5a"),
    ("step_1338", "605d322415ba77ce3e671be28d52ee0f9db57d41"),
)
PARENTS = (
    "5d5a3ca0cb6133ce14dc55c501b4caefdab88a7c",
    CANDIDATES[0][1],
    CANDIDATES[1][1],
    CANDIDATES[2][1],
)
CANDIDATE_SCOPES = (
    (
        "crates/nostr_automerge/src/control/ancestry.rs",
        "crates/nostr_automerge/src/reference/branch_state.rs",
        "docs/execution/remediation_v11/ledger.md",
        "implementation/runtime_ledger_v11.json",
        "reports/spec_baseline.txt",
        "scripts/validate_remediation_v11.py",
        "scripts/validate_target_work_accounting_v11.py",
    ),
    (
        "crates/nostr_automerge/src/reference/branch_state.rs",
        "docs/execution/remediation_v11/ledger.md",
        "implementation/runtime_ledger_v11.json",
        "reports/spec_baseline.txt",
        "scripts/validate_remediation_v11.py",
        "spec/remediation_v11_reproductions.json",
    ),
    (
        "crates/nostr_automerge/src/control/ancestry.rs",
        "docs/execution/remediation_v11/ledger.md",
        "implementation/runtime_ledger_v11.json",
        "reports/spec_baseline.txt",
        "scripts/validate_remediation_v11.py",
        "spec/remediation_v11_reproductions.json",
    ),
    (
        "crates/nostr_automerge/src/control/ancestry.rs",
        "crates/nostr_automerge/src/reference/branch_state.rs",
        "docs/execution/remediation_v11/ledger.md",
        "implementation/runtime_ledger_v11.json",
        "reports/spec_baseline.txt",
        "scripts/validate_remediation_v11.py",
    ),
)
STRUCTURES = (
    ("persistent_delta_map", "iterative_arc_try_unwrap", "stop_at_first_shared_parent"),
    ("control_ancestry", "iterative_arc_try_unwrap", "stop_at_first_shared_parent"),
)
SOURCE_INVENTORY = (
    (
        "crates/nostr_automerge/src/reference/branch_state.rs",
        "54a26f01c1f732ac4199682c4b40696e833c5e3f0797160d8a8d3f621d3f8917",
    ),
    (
        "crates/nostr_automerge/src/control/ancestry.rs",
        "ef78b7bce8455ccfec0d2222b5491f35a8b28c786df6d3e7c69227ecc61ba910",
    ),
)
SOURCE_PROJECTION_SHA256 = "32bd97824e00ced98e549b9cd91f732f6fa8db86fd226cbe53d0413d573019b2"
TESTS = (
    "reference::branch_state::tests::deep_unique_delta_teardown_is_bounded_stack",
    "reference::branch_state::tests::constrained_stack_wide_delta_fork_preserves_shared_parent_teardown",
    "reference::branch_state::tests::constrained_stack_delta_drop_stops_at_a_retained_shared_prefix",
    "reference::branch_state::tests::clone_drop_permutations_release_each_delta_value_once",
    "reference::branch_state::tests::stopped_and_panicking_delta_construction_releases_unpublished_values_once",
    "control::ancestry::tests::deep_unique_control_ancestry_teardown_is_bounded_stack",
    "control::ancestry::tests::constrained_stack_wide_ancestry_fork_preserves_shared_parent_teardown",
    "control::ancestry::tests::constrained_stack_ancestry_drop_stops_at_a_retained_shared_prefix",
)
COUNTS = {
    "structure_count": 2,
    "source_file_count": 2,
    "enabled_test_count": 8,
    "constrained_stack_bytes": 65_536,
    "delta_unique_depth": 100_000,
    "ancestry_unique_depth": 20_000,
    "delta_wide_forks": 10_000,
    "ancestry_wide_forks": 4_096,
}
PROPERTIES = {
    "unique_chain_stack_bounded": True,
    "shared_parent_mutated": False,
    "shared_parent_recursive_drop": False,
    "value_double_drop": False,
    "unpublished_value_leak": False,
    "post_stop_semantic_traversal": False,
    "unexpected_panic_masked": False,
}


class OwnershipGateError(RuntimeError):
    """The bounded persistent ownership evidence is stale or open shaped."""


def require(condition: bool, diagnostic: str) -> None:
    if not condition:
        raise OwnershipGateError(diagnostic)


def require_record(value: object, keys: tuple[str, ...], label: str) -> dict[str, object]:
    require(isinstance(value, dict) and tuple(value) == keys, f"{label}:keys")
    return value


def sha256(path: pathlib.Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def source_projection(rows: tuple[tuple[str, str], ...]) -> str:
    digest = hashlib.sha256()
    for path, source_hash in rows:
        digest.update(path.encode())
        digest.update(b"\0")
        digest.update(source_hash.encode())
        digest.update(b"\n")
    return digest.hexdigest()


def validate_report(value: object) -> None:
    record = require_record(
        value,
        (
            "schema", "status", "stage", "revision", "requirement", "candidates",
            "structures", "source_inventory", "source_projection_sha256", "tests",
            "counts", "properties", "result",
        ),
        "report",
    )
    require(
        (
            record["schema"], record["status"], record["stage"], record["revision"],
            record["requirement"], record["result"],
        )
        == (
            "nostr_automerge.persistent_ownership.v11.v1", "pass",
            "rust_bounded_persistent_teardown", "draft_2026_08",
            "NCRDT-OWNERSHIP-001", "pass",
        ),
        "report:identity",
    )
    candidates = []
    require(isinstance(record["candidates"], list), "report:candidates:type")
    for index, value in enumerate(record["candidates"]):
        item = require_record(value, ("step", "candidate"), f"report:candidate:{index}")
        candidates.append((item["step"], item["candidate"]))
    require(tuple(candidates) == CANDIDATES, "report:candidates")
    structures = []
    require(isinstance(record["structures"], list), "report:structures:type")
    for index, value in enumerate(record["structures"]):
        item = require_record(
            value,
            ("name", "unique_teardown", "shared_teardown"),
            f"report:structure:{index}",
        )
        structures.append((item["name"], item["unique_teardown"], item["shared_teardown"]))
    require(tuple(structures) == STRUCTURES, "report:structures")
    sources = []
    require(isinstance(record["source_inventory"], list), "report:sources:type")
    for index, value in enumerate(record["source_inventory"]):
        item = require_record(value, ("path", "sha256"), f"report:source:{index}")
        sources.append((item["path"], item["sha256"]))
    require(tuple(sources) == SOURCE_INVENTORY, "report:sources")
    require(record["source_projection_sha256"] == SOURCE_PROJECTION_SHA256, "report:projection")
    require(tuple(record["tests"]) == TESTS, "report:tests")
    require(record["counts"] == COUNTS and tuple(record["counts"]) == tuple(COUNTS), "report:counts")
    require(
        record["properties"] == PROPERTIES and tuple(record["properties"]) == tuple(PROPERTIES),
        "report:properties",
    )


def validate_schema(value: object) -> None:
    schema = require_record(
        value,
        ("$schema", "$id", "title", "type", "additionalProperties", "required", "properties"),
        "schema",
    )
    require(schema["type"] == "object" and schema["additionalProperties"] is False, "schema:closed")
    required = (
        "schema", "status", "stage", "revision", "requirement", "candidates",
        "structures", "source_inventory", "source_projection_sha256", "tests",
        "counts", "properties", "result",
    )
    require(tuple(schema["required"]) == required, "schema:required")
    require(tuple(schema["properties"]) == required, "schema:properties")
    require(schema["properties"]["candidates"]["minItems"] == 4, "schema:candidates:min")
    require(schema["properties"]["candidates"]["maxItems"] == 4, "schema:candidates:max")
    require(schema["properties"]["tests"]["minItems"] == 8, "schema:tests:min")
    require(schema["properties"]["tests"]["maxItems"] == 8, "schema:tests:max")
    require(schema["properties"]["counts"]["additionalProperties"] is False, "schema:counts:closed")
    require(schema["properties"]["properties"]["additionalProperties"] is False, "schema:properties:closed")


def drop_body(source: str, declaration: str, following: str) -> str:
    start = source.find(declaration)
    require(start >= 0, f"source:{declaration}:drop")
    end = source.find(following, start)
    require(end > start, f"source:{declaration}:drop_end")
    return source[start:end]


def validate_sources(sources: dict[str, str]) -> None:
    require(tuple(sources) == tuple(path for path, _digest in SOURCE_INVENTORY), "source:inventory")
    for path, declaration, type_name, following in (
        (
            SOURCE_INVENTORY[0][0],
            "impl<K, V> Drop for PersistentDeltaMap<K, V> {",
            "PersistentDeltaMap<K, V>",
            "#[derive(Debug, PartialEq, Eq)]",
        ),
        (
            SOURCE_INVENTORY[1][0],
            "impl Drop for ControlAncestry {",
            "ControlAncestry",
            "impl ControlAncestry {",
        ),
    ):
        body = drop_body(sources[path], declaration, following)
        anchors = (
            "let mut cursor = self.tail.take();",
            "while let Some(node) = cursor",
            "Arc::try_unwrap(node)",
            "Ok(mut owned) => cursor = owned.parent.take()",
            "Err(shared)",
            "drop(shared);",
            "break;",
        )
        cursor = -1
        for anchor in anchors:
            position = body.find(anchor, cursor + 1)
            require(position > cursor, f"source:{type_name}:anchor:{anchor}")
            cursor = position
        require(body.count("Arc::try_unwrap(node)") == 1, f"source:{type_name}:unwrap_count")
        require("self.drop(" not in body and "drop(self" not in body, f"source:{type_name}:recursive")
    combined = "\n".join(sources.values())
    for test in TESTS:
        name = test.rsplit("::", 1)[1]
        require(
            re.search(rf"#\[test\]\s+fn {re.escape(name)}\s*\(", combined) is not None,
            f"source:test:{name}",
        )
    require("stack_size(64 * 1024)" in sources[SOURCE_INVENTORY[0][0]], "source:delta:stack")
    require("1_usize..=100_000" in sources[SOURCE_INVENTORY[0][0]], "source:delta:depth")
    require("1_u32..=10_000" in sources[SOURCE_INVENTORY[0][0]], "source:delta:width")
    require("stack_size(64 * 1024)" in sources[SOURCE_INVENTORY[1][0]], "source:ancestry:stack")
    require("1_u64..=20_000" in sources[SOURCE_INVENTORY[1][0]], "source:ancestry:depth")
    require("1_u64..=4_096" in sources[SOURCE_INVENTORY[1][0]], "source:ancestry:width")


def validate_repository() -> None:
    require(sha256(REPORT) == REPORT_SHA256, "repository:report_hash")
    require(sha256(SCHEMA) == SCHEMA_SHA256, "repository:schema_hash")
    report = json.loads(REPORT.read_text())
    schema = json.loads(SCHEMA.read_text())
    validate_report(report)
    validate_schema(schema)
    source_rows = tuple((path, sha256(ROOT / path)) for path, _digest in SOURCE_INVENTORY)
    require(source_rows == SOURCE_INVENTORY, "repository:source_hashes")
    require(source_projection(source_rows) == SOURCE_PROJECTION_SHA256, "repository:source_projection")
    validate_sources({path: (ROOT / path).read_text() for path, _digest in SOURCE_INVENTORY})
    for index, (_step, candidate) in enumerate(CANDIDATES):
        parent = subprocess.run(
            ["git", "rev-parse", f"{candidate}^"], cwd=ROOT, check=True, capture_output=True, text=True,
        ).stdout.strip()
        require(parent == PARENTS[index], f"repository:candidate:{index}:parent")
        scope = tuple(
            subprocess.run(
                ["git", "diff-tree", "--no-commit-id", "--name-only", "-r", candidate],
                cwd=ROOT, check=True, capture_output=True, text=True,
            ).stdout.splitlines()
        )
        require(scope == CANDIDATE_SCOPES[index], f"repository:candidate:{index}:scope")


def mutation_self_test() -> tuple[int, int, int]:
    report = json.loads(REPORT.read_text())
    schema = json.loads(SCHEMA.read_text())
    report_mutations = []
    for mutate in (
        lambda value: value["candidates"].pop(),
        lambda value: value["candidates"].reverse(),
        lambda value: value["candidates"][0].update(candidate="0" * 40),
        lambda value: value["structures"].reverse(),
        lambda value: value["structures"][0].update(unique_teardown="recursive_drop"),
        lambda value: value["source_inventory"].reverse(),
        lambda value: value["source_inventory"][0].update(sha256="0" * 64),
        lambda value: value.update(source_projection_sha256="0" * 64),
        lambda value: value["tests"].pop(),
        lambda value: value["tests"].reverse(),
        lambda value: value["counts"].update(delta_unique_depth=99_999),
        lambda value: value["counts"].update(constrained_stack_bytes=131_072),
        lambda value: value["properties"].update(unique_chain_stack_bounded=False),
        lambda value: value["properties"].update(shared_parent_mutated=True),
        lambda value: value["properties"].update(value_double_drop=True),
        lambda value: value["properties"].update(unexpected_panic_masked=True),
        lambda value: value.update(extra=False),
    ):
        candidate = copy.deepcopy(report)
        mutate(candidate)
        report_mutations.append(candidate)
    for index, candidate in enumerate(report_mutations):
        try:
            validate_report(candidate)
        except OwnershipGateError:
            continue
        raise OwnershipGateError(f"mutation:report:{index}")
    schema_mutations = []
    for mutate in (
        lambda value: value.update(additionalProperties=True),
        lambda value: value["required"].pop(),
        lambda value: value["properties"].pop("tests"),
        lambda value: value["properties"]["tests"].update(maxItems=9),
        lambda value: value["properties"]["counts"].update(additionalProperties=True),
    ):
        candidate = copy.deepcopy(schema)
        mutate(candidate)
        schema_mutations.append(candidate)
    for index, candidate in enumerate(schema_mutations):
        try:
            validate_schema(candidate)
        except OwnershipGateError:
            continue
        raise OwnershipGateError(f"mutation:schema:{index}")
    sources = {path: (ROOT / path).read_text() for path, _digest in SOURCE_INVENTORY}
    source_mutations = []
    for path, old, new in (
        (SOURCE_INVENTORY[0][0], "self.tail.take()", "self.tail.clone()"),
        (SOURCE_INVENTORY[0][0], "Arc::try_unwrap(node)", "Arc::clone(&node)"),
        (SOURCE_INVENTORY[0][0], "owned.parent.take()", "owned.parent.clone()"),
        (SOURCE_INVENTORY[0][0], "fn deep_unique_delta_teardown", "#[ignore]\n    fn deep_unique_delta_teardown"),
        (SOURCE_INVENTORY[1][0], "self.tail.take()", "self.tail.clone()"),
        (SOURCE_INVENTORY[1][0], "drop(shared);", "drop(self);"),
        (SOURCE_INVENTORY[1][0], "fn deep_unique_control_ancestry_teardown", "#[ignore]\n    fn deep_unique_control_ancestry_teardown"),
        (SOURCE_INVENTORY[1][0], "1_u64..=20_000", "1_u64..=20"),
    ):
        candidate = dict(sources)
        require(old in candidate[path], f"mutation:source:anchor:{path}:{old}")
        candidate[path] = candidate[path].replace(old, new, 1)
        source_mutations.append(candidate)
    for index, candidate in enumerate(source_mutations):
        try:
            validate_sources(candidate)
        except OwnershipGateError:
            continue
        raise OwnershipGateError(f"mutation:source:{index}")
    return len(report_mutations), len(schema_mutations), len(source_mutations)


def run_proofs() -> int:
    command = ["cargo", "test", "-p", "nostr_automerge", "--lib", "--locked", "--", *TESTS]
    result = subprocess.run(command, cwd=ROOT, check=True, capture_output=True, text=True)
    for test in TESTS:
        require(f"test {test} ... ok" in result.stdout, f"proof:{test}")
    require("test result: ok. 8 passed; 0 failed; 0 ignored;" in result.stdout, "proof:summary")
    return len(TESTS)


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--run-proofs", action="store_true")
    args = parser.parse_args()
    validate_repository()
    report_mutations, schema_mutations, source_mutations = mutation_self_test()
    executed = run_proofs() if args.run_proofs else 0
    print("PASS: bounded persistent ownership v11 gate")
    print(f"- candidates={len(CANDIDATES)}")
    print(f"- structures={len(STRUCTURES)}")
    print(f"- tests={len(TESTS)}")
    print(f"- executed={executed}")
    print(f"- mutations={report_mutations + schema_mutations + source_mutations}")


if __name__ == "__main__":
    main()
