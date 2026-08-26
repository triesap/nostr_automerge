#!/usr/bin/env python3
"""Validate the closed Rust persistent-state integration gate for remediation v11."""

from __future__ import annotations

import copy
import hashlib
import json
import pathlib
import subprocess
import sys

sys.dont_write_bytecode = True

ROOT = pathlib.Path(__file__).resolve().parents[1]
REPORT = ROOT / "reports/persistent_state_integration_v11.json"
SCHEMA = ROOT / "tools/validation/persistent_state_integration_v11.schema.json"
CORE_REPORT = ROOT / "reports/persistent_state_core_v11.json"
REPORT_SHA256 = "d5f7feb42dba21f079cbbcbf7b200cb84f2126dd851e51a0240de63b8eb0b55d"
SCHEMA_SHA256 = "38c40db2b084053d5517102bf367b65dae852f5fb8fe8d8fc34cb0714053774f"
CORE_GATE_SHA256 = "e540248bab985856d9aba407758ed1343c3c0e039f81347d29e4909abdecf695"
SOURCE_PROJECTION_SHA256 = "aae63c79619e4f42ad68f40a7b082d9f597388ff8479f4d02de9d7c1691639e1"
CANDIDATES = (
    ("step_1321", "e04e0e557755c5f7a460eb60231f6e123c86ebb1"),
    ("step_1322", "64604f274341df014634a9dcf4084b95b644a46d"),
    ("step_1323", "4191a65a7c6cf8e27184d8c5d61b42381f9cf250"),
    ("step_1324", "9ae36ba68525be4284fb96266c5e76c3c576fa13"),
    ("step_1325", "27618d0ed85f2d9bb38e2f4f6258262f801bb2df"),
)
PARENTS = (
    "8b508d9e9b8da34addd061b60465dc41ef62648d",
    CANDIDATES[0][1],
    CANDIDATES[1][1],
    CANDIDATES[2][1],
    CANDIDATES[3][1],
)
CALL_SITES = (
    ("parent_result_projection", "from_result_metered", "graph_node"),
    ("parent_additional_projection", "set_additional_prior_knowledge_metered", "graph_node"),
    ("parent_frontier_lookup", "frontier_knowledge_metered", "graph_node"),
    ("candidate_frontier", "evaluate_candidate_frontier_metered", "graph_node"),
    ("epoch_dependencies", "prior_dependencies_valid_metered", "graph_edge_and_graph_node"),
    ("referenced_branch_disposition", "referenced_branch_change_disposition_metered", "graph_node"),
    ("prior_extension", "extend_prior_knowledge_metered", "graph_node"),
    ("branch_disposition_extension", "extend_branch_dispositions_metered", "graph_node"),
)
SOURCE_INVENTORY = (
    ("crates/nostr_automerge/src/control/parent_view.rs", "52c546be373661f607c06503bba36c9f22486aa7843c9e3b3c4495f9ddcd6388"),
    ("crates/nostr_automerge/src/control/candidate.rs", "69a25ffb3c2cf8e8b61bdd3ff146ace90aa0f8e6097b6c11f065f66c278de3d5"),
    ("crates/nostr_automerge/src/reference/epoch_engine.rs", "819af3a1594ccdeb3c464ba5c98b9d470d15d6cbf38e7e3bae783bd6ad885e98"),
    ("crates/nostr_automerge/src/reference/evaluate.rs", "9a63fe6d420af47794fd8845ffbece1011c5784a89ee59ef1a4c02095aa717e4"),
    ("crates/nostr_automerge/src/engine/reference_evaluator.rs", "2668986962170911e4362f442c61493bbb487840276f8089b100057aa6c293db"),
    ("crates/nostr_automerge/tests/public_engine_api.rs", "07caa6cbaca73c6e940107aaaf301652383ce5af68d64772a8f42f7b9aa249f5"),
    ("scripts/validate_persistent_state_v11.py", "7c62023fb37955f8b3612f5b6dbdf7783eafeece1d7991123966212a1866411e"),
)
TESTS = (
    "parent_result_projection_charges_before_every_read_and_insert",
    "candidate_frontier_exposes_every_deep_knowledge_charge",
    "dependency_lookup_charges_before_outer_reads_and_persistent_nodes",
    "referenced_disposition_lookup_exposes_every_persistent_node",
    "branch_local_extension_owns_preparation_and_publication",
    "persistent_integration_is_exact_at_every_visible_boundary",
)
TEST_SOURCES = (
    SOURCE_INVENTORY[0][0],
    SOURCE_INVENTORY[1][0],
    SOURCE_INVENTORY[2][0],
    SOURCE_INVENTORY[3][0],
    SOURCE_INVENTORY[3][0],
    SOURCE_INVENTORY[5][0],
)
COUNTS = {
    "candidate_count": 5,
    "call_site_count": 8,
    "source_file_count": 7,
    "enabled_test_count": 6,
    "deep_chain_nodes": 64,
    "source_policy_mutations": 21,
    "integration_controls": 2,
    "integration_changes": 2,
}
PROPERTIES = {
    "outer_item_read_before_charge": False,
    "persistent_node_read_before_charge": False,
    "failed_extension_publishes_state": False,
    "typed_stop_preserved": True,
    "n_minus_one_no_progress": True,
    "n_and_n_plus_one_compatible": True,
    "every_observed_boundary_cancellable": True,
    "runtime_unmetered_bypass": False,
}


class PersistentIntegrationGateError(RuntimeError):
    """The integration evidence is stale, incomplete, or open shaped."""


def require(condition: bool, diagnostic: str) -> None:
    if not condition:
        raise PersistentIntegrationGateError(diagnostic)


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
            "schema", "status", "stage", "revision", "core_gate_sha256",
            "candidates", "call_sites", "source_inventory",
            "source_projection_sha256", "tests", "counts", "properties", "result",
        ),
        "report",
    )
    require(
        (record["schema"], record["status"], record["stage"], record["revision"], record["result"])
        == ("nostr_automerge.persistent_state_integration.v11.v1", "pass", "rust_persistent_state_integration", "draft_2026_08", "pass"),
        "report:identity",
    )
    require(record["core_gate_sha256"] == CORE_GATE_SHA256, "report:core")
    candidates = record["candidates"]
    require(isinstance(candidates, list), "report:candidates:type")
    candidate_rows = []
    for index, candidate in enumerate(candidates):
        item = require_record(candidate, ("step", "candidate"), f"report:candidate:{index}")
        candidate_rows.append((item["step"], item["candidate"]))
    require(tuple(candidate_rows) == CANDIDATES, "report:candidates")
    call_sites = record["call_sites"]
    require(isinstance(call_sites, list), "report:calls:type")
    call_rows = []
    for index, call in enumerate(call_sites):
        item = require_record(call, ("name", "boundary", "owner"), f"report:call:{index}")
        call_rows.append((item["name"], item["boundary"], item["owner"]))
    require(tuple(call_rows) == CALL_SITES, "report:calls")
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
    required = (
        "schema", "status", "stage", "revision", "core_gate_sha256", "candidates",
        "call_sites", "source_inventory", "source_projection_sha256", "tests",
        "counts", "properties", "result",
    )
    require(tuple(schema["required"]) == required, "schema:required")
    properties = schema["properties"]
    require(isinstance(properties, dict) and tuple(properties) == required, "schema:properties")
    for name, count in (("candidates", 5), ("call_sites", 8), ("source_inventory", 7), ("tests", 6)):
        item = properties[name]
        require(isinstance(item, dict) and item.get("minItems") == count and item.get("maxItems") == count, f"schema:{name}")
    for name in ("counts", "properties"):
        item = properties[name]
        require(isinstance(item, dict) and item.get("additionalProperties") is False, f"schema:{name}:closed")


def validate_sources(sources: dict[str, str]) -> None:
    anchors = (
        (SOURCE_INVENTORY[0][0], "pub(crate) fn from_result_metered<E>(", 1),
        (SOURCE_INVENTORY[0][0], "pub(crate) fn set_additional_prior_knowledge_metered<E>(", 1),
        (SOURCE_INVENTORY[0][0], "pub(crate) fn frontier_knowledge_metered<E>(", 1),
        (SOURCE_INVENTORY[1][0], "fn evaluate_candidate_frontier_metered(", 1),
        (SOURCE_INVENTORY[2][0], "fn prior_dependencies_valid_metered<E>(", 1),
        (SOURCE_INVENTORY[3][0], "pub(crate) fn referenced_branch_change_disposition_metered<E>(", 1),
        (SOURCE_INVENTORY[3][0], "fn extend_prior_knowledge_metered<E>(", 1),
        (SOURCE_INVENTORY[3][0], "fn extend_branch_dispositions_metered<E>(", 1),
        (SOURCE_INVENTORY[4][0], "batch.referenced_branch_change_disposition_metered(", 1),
    )
    for path, anchor, count in anchors:
        require(sources[path].count(anchor) == count, f"source:{path}:{anchor}")
    for path, test in zip(TEST_SOURCES, TESTS, strict=True):
        require(f"fn {test}()" in sources[path], f"source:test:{test}")


def validate_repository() -> None:
    require(sha256(REPORT) == REPORT_SHA256, "repository:report_hash")
    require(sha256(SCHEMA) == SCHEMA_SHA256, "repository:schema_hash")
    require(sha256(CORE_REPORT) == CORE_GATE_SHA256, "repository:core_hash")
    report = json.loads(REPORT.read_text(encoding="utf-8"))
    validate_report(report)
    validate_schema(json.loads(SCHEMA.read_text(encoding="utf-8")))
    for (step, candidate), parent in zip(CANDIDATES, PARENTS, strict=True):
        resolved = subprocess.run(
            ["git", "rev-parse", f"{candidate}^"], cwd=ROOT, check=True,
            capture_output=True, text=True,
        ).stdout.strip()
        require(resolved == parent, f"repository:parent:{step}")
    sources = {
        path: historical_source(path).decode("utf-8") for path, _digest in SOURCE_INVENTORY
    }
    for path, digest in SOURCE_INVENTORY:
        require(hashlib.sha256(sources[path].encode("utf-8")).hexdigest() == digest, f"repository:source:{path}")
    validate_sources(sources)


def mutation_self_test() -> tuple[int, int, int]:
    report = json.loads(REPORT.read_text(encoding="utf-8"))
    schema = json.loads(SCHEMA.read_text(encoding="utf-8"))
    report_mutations = []
    for mutate in (
        lambda value: value["candidates"].pop(),
        lambda value: value["candidates"].reverse(),
        lambda value: value["candidates"][0].update(candidate="0" * 40),
        lambda value: value.update(core_gate_sha256="0" * 64),
        lambda value: value["call_sites"].pop(),
        lambda value: value["call_sites"].reverse(),
        lambda value: value["call_sites"][0].update(boundary="from_result"),
        lambda value: value["source_inventory"].pop(),
        lambda value: value["source_inventory"].reverse(),
        lambda value: value["source_inventory"][0].update(sha256="0" * 64),
        lambda value: value.update(source_projection_sha256="0" * 64),
        lambda value: value["tests"].pop(),
        lambda value: value["counts"].update(source_policy_mutations=20),
        lambda value: value["properties"].update(runtime_unmetered_bypass=True),
        lambda value: value.update(extra=False),
    ):
        candidate = copy.deepcopy(report)
        mutate(candidate)
        report_mutations.append(candidate)
    for index, candidate in enumerate(report_mutations):
        try:
            validate_report(candidate)
        except PersistentIntegrationGateError:
            continue
        raise PersistentIntegrationGateError(f"mutation:report:{index}")
    schema_mutations = []
    for mutate in (
        lambda value: value.update(additionalProperties=True),
        lambda value: value["required"].pop(),
        lambda value: value["properties"].pop("tests"),
        lambda value: value["properties"]["call_sites"].update(maxItems=9),
        lambda value: value["properties"]["counts"].update(additionalProperties=True),
    ):
        candidate = copy.deepcopy(schema)
        mutate(candidate)
        schema_mutations.append(candidate)
    for index, candidate in enumerate(schema_mutations):
        try:
            validate_schema(candidate)
        except PersistentIntegrationGateError:
            continue
        raise PersistentIntegrationGateError(f"mutation:schema:{index}")
    sources = {
        path: historical_source(path).decode("utf-8") for path, _digest in SOURCE_INVENTORY
    }
    source_mutations = []
    for path, anchor in (
        (SOURCE_INVENTORY[0][0], "pub(crate) fn from_result_metered<E>("),
        (SOURCE_INVENTORY[0][0], "pub(crate) fn frontier_knowledge_metered<E>("),
        (SOURCE_INVENTORY[1][0], "fn evaluate_candidate_frontier_metered("),
        (SOURCE_INVENTORY[2][0], "fn prior_dependencies_valid_metered<E>("),
        (SOURCE_INVENTORY[3][0], "fn extend_prior_knowledge_metered<E>("),
        (SOURCE_INVENTORY[3][0], "fn extend_branch_dispositions_metered<E>("),
        (SOURCE_INVENTORY[4][0], "batch.referenced_branch_change_disposition_metered("),
        (SOURCE_INVENTORY[5][0], f"fn {TESTS[-1]}()"),
    ):
        candidate = copy.deepcopy(sources)
        require(anchor in candidate[path], f"mutation:source_anchor:{path}")
        candidate[path] = candidate[path].replace(anchor, "removed_boundary(", 1)
        source_mutations.append(candidate)
    for index, candidate in enumerate(source_mutations):
        try:
            validate_sources(candidate)
        except PersistentIntegrationGateError:
            continue
        raise PersistentIntegrationGateError(f"mutation:source:{index}")
    return len(report_mutations), len(schema_mutations), len(source_mutations)


def main() -> None:
    validate_repository()
    report_mutations, schema_mutations, source_mutations = mutation_self_test()
    print("PASS: persistent state integration v11 gate")
    print(f"- candidates={len(CANDIDATES)}")
    print(f"- call_sites={len(CALL_SITES)}")
    print(f"- tests={len(TESTS)}")
    print(f"- mutations={report_mutations}+{schema_mutations}+{source_mutations}")


if __name__ == "__main__":
    main()
