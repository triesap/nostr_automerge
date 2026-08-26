#!/usr/bin/env python3
"""Validate the closed Rust persistent-state core gate for remediation v11."""

from __future__ import annotations

import copy
import hashlib
import json
import pathlib
import subprocess
import sys

sys.dont_write_bytecode = True

ROOT = pathlib.Path(__file__).resolve().parents[1]
REPORT = ROOT / "reports/persistent_state_core_v11.json"
SCHEMA = ROOT / "tools/validation/persistent_state_core_v11.schema.json"
BRANCH_STATE = ROOT / "crates/nostr_automerge/src/reference/branch_state.rs"
REPORT_SHA256 = "e540248bab985856d9aba407758ed1343c3c0e039f81347d29e4909abdecf695"
SCHEMA_SHA256 = "79a46f48f9ca809beb5d7620fb6fb5d316021c3b46129f3a41ca6681c3ecb721"
SOURCE_PROJECTION_SHA256 = "d1d95e3411de4433b49959d42f1710e0e4233f19650db5d13eb3409bac9fe7d3"
CANDIDATES = (
    ("step_1315", "9f180d4141455522d647579fee049551a415aff9"),
    ("step_1316", "d6d7f4fb9984e72041cafe242aae5c14494ece8d"),
    ("step_1317", "3b41019d78a50b25d3d131065b5a94c307663f3b"),
    ("step_1318", "5ca15673198af818aa64a9413d775da7fe9240b8"),
    ("step_1319", "c0a3f89c913b3d1df7aadf07460db8d533d61a43"),
)
PARENTS = (
    "f2816e457876fb6f0a58f37dcd9ab54970360ef6",
    CANDIDATES[0][1],
    CANDIDATES[1][1],
    CANDIDATES[2][1],
    CANDIDATES[3][1],
)
OPERATIONS = (
    ("lookup", "get_metered", "persistent_nodes_actually_visited", "read_only"),
    ("membership", "contains_key_metered", "exact_lookup_projection", "read_only"),
    (
        "extension",
        "extend_prepared_metered",
        "prepared_item_lookup_node_accepted_insert",
        "after_all_work_succeeds",
    ),
    (
        "materialization",
        "materialize_metered",
        "persistent_nodes_and_emitted_items",
        "after_each_charged_item",
    ),
)
SOURCE_INVENTORY = (
    ("crates/nostr_automerge/src/reference/branch_state.rs", "c9b5489f27e5e085b2978b80a96786f081d94dc9911ae208f46b42973411f2cf"),
    ("crates/nostr_automerge/src/control/frontier.rs", "f2132e0ba8c49d2dcdb1633192cbdbc3c7da845efbdf0776e44c0bd969ab012e"),
    ("crates/nostr_automerge/src/control/parent_view.rs", "8ea34fb7284179de573111134ada57138182f910ea8a43bde32deafc20a09745"),
    ("crates/nostr_automerge/src/control/transition.rs", "f51eda0c0e51502b87e00c8d2e8d5844088df21657ca99ed55ae5d97dc2ce26c"),
    ("crates/nostr_automerge/src/control/candidate.rs", "fc30a0f85b9ebd25402d7040a89a827ed308ec53ecb789ddf9bebab208543c57"),
    ("crates/nostr_automerge/src/reference/epoch_engine.rs", "8a2f3b610fa9541774079fed27096f9dcf3ba45bc392f8173c8feda734d53ad1"),
    ("crates/nostr_automerge/src/reference/evaluate.rs", "a94e6ff7b885b70b32a00b0a7b798107f038efccb51d84fb8d6f17e79c9ddd0c"),
    ("crates/nostr_automerge/src/engine/reference_evaluator.rs", "2668986962170911e4362f442c61493bbb487840276f8089b100057aa6c293db"),
    ("scripts/validate_persistent_state_v11.py", "13e5b115f4ca9d2bf0d55d2446cd435ab59e4d0c37c73aaec82a8c1542681a21"),
)
TESTS = (
    "delta_chain_shares_parent_and_materializes_in_override_order",
    "metered_lookup_counts_only_nodes_actually_visited",
    "metered_membership_reuses_lookup_without_hidden_scans",
    "metered_extension_publishes_only_after_all_owned_work",
    "deep_persistent_boundaries_are_exact_and_cancellable",
)
COUNTS = {
    "deep_chain_nodes": 64,
    "enabled_tests": 5,
    "source_files": 9,
    "source_policy_mutations": 15,
}
PROPERTIES = {
    "every_prefix_cancellable": True,
    "n_minus_one_n_n_plus_one": True,
    "failed_work_publishes_state": False,
    "unexpected_error_identity_preserved": True,
    "runtime_unmetered_bypass": False,
    "ample_capacity_behavior_compatible": True,
}


class PersistentCoreGateError(RuntimeError):
    """The persistent-core evidence is stale, incomplete, or open shaped."""


def require(condition: bool, diagnostic: str) -> None:
    if not condition:
        raise PersistentCoreGateError(diagnostic)


def require_record(value: object, keys: tuple[str, ...], label: str) -> dict[str, object]:
    require(isinstance(value, dict) and tuple(value) == keys, f"{label}:keys")
    return value


def sha256(path: pathlib.Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def historical_source(path: str) -> bytes:
    return subprocess.run(
        ["git", "show", f"{CANDIDATES[-1][1]}:{path}"],
        cwd=ROOT,
        check=True,
        capture_output=True,
    ).stdout


def validate_report(value: object) -> None:
    record = require_record(
        value,
        (
            "schema", "status", "stage", "revision", "candidates", "operations",
            "source_inventory", "source_projection_sha256", "tests", "counts",
            "properties", "result",
        ),
        "report",
    )
    require(
        (record["schema"], record["status"], record["stage"], record["revision"], record["result"])
        == ("nostr_automerge.persistent_state_core.v11.v1", "pass", "rust_persistent_state_core", "draft_2026_08", "pass"),
        "report:identity",
    )
    candidates = record["candidates"]
    require(isinstance(candidates, list), "report:candidates:type")
    candidate_rows = []
    for index, candidate in enumerate(candidates):
        item = require_record(candidate, ("step", "candidate"), f"report:candidate:{index}")
        candidate_rows.append((item["step"], item["candidate"]))
    require(tuple(candidate_rows) == CANDIDATES, "report:candidates")

    operations = record["operations"]
    require(isinstance(operations, list), "report:operations:type")
    operation_rows = []
    for index, operation in enumerate(operations):
        item = require_record(operation, ("name", "method", "count", "publication"), f"report:operation:{index}")
        operation_rows.append((item["name"], item["method"], item["count"], item["publication"]))
    require(tuple(operation_rows) == OPERATIONS, "report:operations")

    inventory = record["source_inventory"]
    require(isinstance(inventory, list), "report:sources:type")
    source_rows = []
    for index, source in enumerate(inventory):
        item = require_record(source, ("path", "sha256"), f"report:source:{index}")
        source_rows.append((item["path"], item["sha256"]))
    require(tuple(source_rows) == SOURCE_INVENTORY, "report:sources")
    projection = hashlib.sha256(
        json.dumps(inventory, ensure_ascii=True, separators=(",", ":")).encode("utf-8")
    ).hexdigest()
    require(projection == SOURCE_PROJECTION_SHA256, "report:source_projection:derived")
    require(record["source_projection_sha256"] == SOURCE_PROJECTION_SHA256, "report:source_projection")
    require(tuple(record["tests"]) == TESTS, "report:tests")
    require(record["counts"] == COUNTS and tuple(record["counts"]) == tuple(COUNTS), "report:counts")
    require(record["properties"] == PROPERTIES and tuple(record["properties"]) == tuple(PROPERTIES), "report:properties")


def validate_schema(value: object) -> None:
    schema = require_record(
        value,
        ("$schema", "$id", "title", "type", "additionalProperties", "required", "properties"),
        "schema",
    )
    require(schema["type"] == "object" and schema["additionalProperties"] is False, "schema:closed")
    require(tuple(schema["required"]) == (
        "schema", "status", "stage", "revision", "candidates", "operations",
        "source_inventory", "source_projection_sha256", "tests", "counts", "properties", "result",
    ), "schema:required")
    properties = schema["properties"]
    require(isinstance(properties, dict) and tuple(properties) == tuple(schema["required"]), "schema:properties")
    for name, count in (("candidates", 5), ("operations", 4), ("source_inventory", 9), ("tests", 5)):
        item = properties[name]
        require(isinstance(item, dict) and item.get("minItems") == count and item.get("maxItems") == count, f"schema:{name}")
    for name in ("counts", "properties"):
        item = properties[name]
        require(isinstance(item, dict) and item.get("additionalProperties") is False, f"schema:{name}:closed")


def validate_repository() -> None:
    require(sha256(REPORT) == REPORT_SHA256, "repository:report_hash")
    require(sha256(SCHEMA) == SCHEMA_SHA256, "repository:schema_hash")
    report = json.loads(REPORT.read_text(encoding="utf-8"))
    validate_report(report)
    validate_schema(json.loads(SCHEMA.read_text(encoding="utf-8")))
    for (step, candidate), parent in zip(CANDIDATES, PARENTS, strict=True):
        resolved = subprocess.run(
            ["git", "rev-parse", f"{candidate}^"], cwd=ROOT, check=True,
            capture_output=True, text=True,
        ).stdout.strip()
        require(resolved == parent, f"repository:parent:{step}")
    historical_sources = {
        path: historical_source(path) for path, _digest in SOURCE_INVENTORY
    }
    for path, digest in SOURCE_INVENTORY:
        require(
            hashlib.sha256(historical_sources[path]).hexdigest() == digest,
            f"repository:source:{path}",
        )
    source = historical_source(BRANCH_STATE.relative_to(ROOT).as_posix()).decode("utf-8")
    require("const DEPTH: u8 = 64;" in source, "repository:depth")
    for test in TESTS:
        require(f"#[test]\n    fn {test}()" in source, f"repository:test:{test}")


def mutation_self_test() -> tuple[int, int]:
    report = json.loads(REPORT.read_text(encoding="utf-8"))
    schema = json.loads(SCHEMA.read_text(encoding="utf-8"))
    report_mutations = []
    for mutate in (
        lambda value: value["candidates"].pop(),
        lambda value: value["candidates"].reverse(),
        lambda value: value["candidates"][0].update(candidate="0" * 40),
        lambda value: value["operations"].pop(),
        lambda value: value["operations"][0].update(method="get"),
        lambda value: value["source_inventory"].reverse(),
        lambda value: value["source_inventory"][0].update(sha256="0" * 64),
        lambda value: value.update(source_projection_sha256="0" * 64),
        lambda value: value["tests"].pop(),
        lambda value: value["counts"].update(deep_chain_nodes=63),
        lambda value: value["properties"].update(runtime_unmetered_bypass=True),
        lambda value: value.update(extra=False),
    ):
        candidate = copy.deepcopy(report)
        mutate(candidate)
        report_mutations.append(candidate)
    for index, candidate in enumerate(report_mutations):
        try:
            validate_report(candidate)
        except PersistentCoreGateError:
            continue
        raise PersistentCoreGateError(f"mutation:report:{index}")

    schema_mutations = []
    for mutate in (
        lambda value: value.update(additionalProperties=True),
        lambda value: value["required"].pop(),
        lambda value: value["properties"].pop("tests"),
        lambda value: value["properties"]["candidates"].update(maxItems=6),
        lambda value: value["properties"]["counts"].update(additionalProperties=True),
    ):
        candidate = copy.deepcopy(schema)
        mutate(candidate)
        schema_mutations.append(candidate)
    for index, candidate in enumerate(schema_mutations):
        try:
            validate_schema(candidate)
        except PersistentCoreGateError:
            continue
        raise PersistentCoreGateError(f"mutation:schema:{index}")
    return len(report_mutations), len(schema_mutations)


def main() -> None:
    validate_repository()
    report_mutations, schema_mutations = mutation_self_test()
    print("PASS: persistent state core v11 gate")
    print(f"- candidates={len(CANDIDATES)}")
    print(f"- operations={len(OPERATIONS)}")
    print(f"- tests={len(TESTS)}")
    print(f"- mutations={report_mutations}+{schema_mutations}+15_policy")


if __name__ == "__main__":
    main()
