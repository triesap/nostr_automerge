#!/usr/bin/env python3
"""Validate the closed Rust target-work accounting gate for remediation v11."""

from __future__ import annotations

import argparse
import copy
import hashlib
import json
import pathlib
import subprocess
import sys

sys.dont_write_bytecode = True

ROOT = pathlib.Path(__file__).resolve().parents[1]
REPORT = ROOT / "reports/target_work_accounting_v11.json"
SCHEMA = ROOT / "tools/validation/target_work_accounting_v11.schema.json"
CORE = ROOT / "reports/persistent_state_core_v11.json"
INTEGRATION = ROOT / "reports/persistent_state_integration_v11.json"
REPRODUCTIONS = ROOT / "spec/remediation_v11_reproductions.json"
REPORT_SHA256 = "e15dca3958e9c9cf98da585c5a60135e4b3c9d8b59ddec9c0e3ef068615948ae"
SCHEMA_SHA256 = "d7d5b6ec7a4f9e3d4964f3f67224a33b34a5827b468b300311db254cf9a270e3"
CORE_SHA256 = "e540248bab985856d9aba407758ed1343c3c0e039f81347d29e4909abdecf695"
INTEGRATION_SHA256 = "d5f7feb42dba21f079cbbcbf7b200cb84f2126dd851e51a0240de63b8eb0b55d"
SOURCE_PROJECTION_SHA256 = "8c64daf8acf34937a6f28728f5e550285c502d8f27067ebd76f30144a244b4f8"
CLOSURE_CANDIDATE = "5d5a3ca0cb6133ce14dc55c501b4caefdab88a7c"
CLOSURE_SCOPE = (
    "crates/nostr_automerge/src/reference/branch_state.rs",
    "crates/nostr_automerge/tests/public_engine_api.rs",
    "docs/execution/remediation_v11/ledger.md",
    "implementation/runtime_ledger_v11.json",
    "reports/spec_baseline.txt",
    "reports/target_work_accounting_v11.json",
    "scripts/local_gate.py",
    "scripts/validate_private_reproduction_boundary_v9.py",
    "scripts/validate_remediation_v11.py",
    "scripts/validate_spec.py",
    "scripts/validate_target_work_accounting_v11.py",
    "spec/remediation_v11_reproductions.json",
    "tools/nostr_automerge_xtask/src/validate.rs",
    "tools/validation/target_work_accounting_v11.schema.json",
)
CANDIDATES = (
    ("step_1327", "e819055185480850d83330631744bb99b44c2c19"),
    ("step_1328", "9d0fb75bd5a617b48fc6927cd945bd3df60622b3"),
    ("step_1329", "9c342f89535b9e0c0a5d3552e82ad043a75195cd"),
    ("step_1330", "1e2215c2398db9d4e02ecbb969afde7686a0437f"),
    ("step_1331", "045a4317436915fed12fcfb8fa8552655bf14a5c"),
    ("step_1332", "61776aea62b838e467b23451b5db766079caf128"),
    ("step_1333", "cdd1218a7eb10e453f47bea980f0d7efbacf995e"),
)
PARENTS = (
    "31e9ec2358fe6dd956baf43b7581273deaf5240d",
    CANDIDATES[0][1], CANDIDATES[1][1], CANDIDATES[2][1],
    CANDIDATES[3][1], CANDIDATES[4][1], CANDIDATES[5][1],
)
OPERATIONS = (
    ("persistent_delta", "item_metered", "graph_node", "PersistentDeltaMap::*_metered", "finding_096_deep_persistent_lookup_is_internally_metered"),
    ("initial_control_parent_hash_maps", "item_metered", "control_and_graph_node", "prepare_initial_maps_metered", "initial_evaluator_maps_charge_before_every_item"),
    ("accepted_state_cache", "item_metered", "graph_node", "record_control_metered", "accepted_state_cache_is_shared_and_charged_per_key_and_insert"),
    ("parent_and_prior_projection", "item_metered", "graph_node", "ParentEpochView::*_metered", "parent_result_projection_charges_before_every_read_and_insert"),
    ("additional_prior_projection", "item_metered", "graph_node", "project_selected_prior_knowledge_metered", "additional_prior_projection_charges_before_every_read_and_insert"),
    ("accepted_base_projection", "item_metered", "graph_node", "clone_hash_set_metered", "accepted_base_projection_charges_before_every_read_and_insert"),
    ("branch_and_result_projection", "item_metered", "graph_node", "insert_branch_state_metered", "branch_table_publication_charges_before_each_insert"),
    ("accepted_candidate_and_raw_maps", "item_metered", "graph_node", "project_accepted_candidate_maps_metered", "accepted_candidate_and_raw_projection_charges_each_owned_operation"),
    ("document_application_and_materialization", "item_metered", "decode_byte_and_apply_change", "apply_exact_closure_metered", "automerge_application_and_materialization_are_charged"),
    ("head_derivation", "item_metered", "graph_node_and_graph_edge", "derive_heads_metered", "head_derivation_charges_deep_forked_and_duplicate_inputs_exactly"),
    ("member_and_dependency_scans", "item_metered", "control_and_graph_edge", "collect_change_dependencies_metered", "member_and_dependency_preparation_interleaves_every_owned_operation"),
    ("selected_manifest", "item_metered", "carrier_and_control", "selected_manifest_in_metered", "selected_manifest_metering_owns_every_read_clone_comparison_and_replacement"),
    ("equivocation_quarantine", "item_metered", "graph_node_and_graph_edge", "quarantine_equivocation_descendants", "quarantine_traversal_has_exact_prefix_and_cancellation_boundaries"),
    ("complete_report_finalization", "exact_reservation", "assertion_report_invariants", "ReportFinalizationPermit", "complete_finalization_passes_start_only_after_exact_consumption"),
    ("typed_stop_fallback", "constant_time", "sealed_fallback", "reserved_interrupted_report", "every_interrupted_prefix_uses_only_fallback_and_never_refunds"),
)
SOURCE_INVENTORY = (
    ("crates/nostr_automerge/src/reference/branch_state.rs", "33daec27868516b00d97fa4a9155c57ffe5ddf8f2ef7428bb36267435096e7b3"),
    ("crates/nostr_automerge/src/control/parent_view.rs", "99f0edd7e72f5e1b624691750151febc297da5a1dd8710fa839eecbad896d10f"),
    ("crates/nostr_automerge/src/control/candidate.rs", "a7d483d349fd33670da6f48df3122ba103853c4d834f85ae1513940724517db6"),
    ("crates/nostr_automerge/src/reference/epoch_engine.rs", "819af3a1594ccdeb3c464ba5c98b9d470d15d6cbf38e7e3bae783bd6ad885e98"),
    ("crates/nostr_automerge/src/reference/evaluate.rs", "72ce5642c1849992cabf5f6e46b2664a0d3cc33c4b92a8b3e175c35fe677a75f"),
    ("crates/nostr_automerge/src/engine/reference_evaluator.rs", "9a7dab9b8563f3fc8e4f7d8e228692d30e52e949349c620fc16e8559a9310f53"),
    ("crates/nostr_automerge/src/automerge_adapter/document.rs", "5c62ae05bddddedc93d296b57390b2c683a217c3e4017d4885c23c4fb443263c"),
    ("crates/nostr_automerge/src/carrier/manifest.rs", "4f82e1c60486fd930e06eb4580d87a070176b60f3a2ebe34f935a71b144841a6"),
    ("crates/nostr_automerge/src/evidence/corpus_builder.rs", "0bcb14c272d451f8548daf53327ac7d2acf930a77efec9d35b394d229473c948"),
    ("crates/nostr_automerge/src/evidence/document_view.rs", "927b6561b9cb69f2191b47dc1efff85127be760e40eb46bddbdd3e6adb47b10b"),
    ("crates/nostr_automerge/src/graph/equivocation.rs", "b22393f86b862bea2841d96674dd352b779766c143c539e48ceba16d30c8088a"),
    ("crates/nostr_automerge/src/integrity.rs", "3e55b011fdbab6005ea91eebbf9260ebcf424821a40cdb12ac1054bd493fc4e4"),
    ("crates/nostr_automerge/src/engine/evaluation_report.rs", "d54dff4dce0be14442784aa70c90fe07f2315d072e102275ceb44156050b8dcc"),
    ("crates/nostr_automerge/tests/public_engine_api.rs", "40cd466ac0a6920997f4446270e4697f02a2d1904dbb3b9c1451149c40fd358e"),
    ("scripts/validate_persistent_state_v11.py", "6501c795816151fa6f7ebd2b83a1a2b116d88eaec25c28fa5688b1ab64cbcdbf"),
)
TESTS = (
    ("lib", "reference::branch_state::tests::finding_096_deep_persistent_lookup_is_internally_metered"),
    ("lib", "reference::evaluate::tests::initial_evaluator_maps_charge_before_every_item"),
    ("lib", "reference::evaluate::tests::accepted_state_cache_is_shared_and_charged_per_key_and_insert"),
    ("lib", "control::parent_view::tests::parent_result_projection_charges_before_every_read_and_insert"),
    ("lib", "engine::reference_evaluator::tests::additional_prior_projection_charges_before_every_read_and_insert"),
    ("lib", "reference::evaluate::tests::accepted_base_projection_charges_before_every_read_and_insert"),
    ("lib", "reference::evaluate::tests::canonical_branch_projection_charges_before_every_owned_operation"),
    ("lib", "reference::evaluate::tests::branch_table_publication_charges_before_each_insert"),
    ("lib", "reference::evaluate::tests::accepted_candidate_and_raw_projection_charges_each_owned_operation"),
    ("public_engine_api", "automerge_application_and_materialization_are_charged"),
    ("lib", "reference::evaluate::tests::head_derivation_charges_deep_forked_and_duplicate_inputs_exactly"),
    ("lib", "engine::reference_evaluator::tests::member_and_dependency_preparation_interleaves_every_owned_operation"),
    ("lib", "evidence::corpus_builder::tests::selected_manifest_metering_owns_every_read_clone_comparison_and_replacement"),
    ("lib", "graph::equivocation::tests::quarantine_traversal_has_exact_prefix_and_cancellation_boundaries"),
    ("lib", "engine::reference_evaluator::tests::complete_finalization_passes_start_only_after_exact_consumption"),
    ("lib", "engine::reference_evaluator::tests::every_interrupted_prefix_uses_only_fallback_and_never_refunds"),
    ("public_engine_api", "cancellation_is_safe_at_every_evaluator_boundary"),
)
COUNTS = {
    "candidate_count": 7,
    "operation_family_count": 15,
    "source_file_count": 15,
    "enabled_test_count": 17,
    "source_policy_mutations": 63,
    "fixed_reproduction_count": 2,
    "open_reproduction_count": 2,
}
PROPERTIES = {
    "every_proportional_operation_classified": True,
    "every_live_target_operation_item_metered": True,
    "sealed_reservation_broadened": False,
    "post_stop_observations": 0,
    "typed_stop_preserved": True,
    "canonical_no_progress_only": True,
    "ample_capacity_reports_compatible": True,
    "bounded_teardown_qualified": False,
}


class TargetWorkGateError(RuntimeError):
    """The target-work accounting evidence is stale, incomplete, or open shaped."""


def require(condition: bool, diagnostic: str) -> None:
    if not condition:
        raise TargetWorkGateError(diagnostic)


def require_record(value: object, keys: tuple[str, ...], label: str) -> dict[str, object]:
    require(isinstance(value, dict) and tuple(value) == keys, f"{label}:keys")
    return value


def sha256(path: pathlib.Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def historical_bytes(path: str) -> bytes:
    return subprocess.run(
        ["git", "show", f"{CLOSURE_CANDIDATE}:{path}"],
        cwd=ROOT,
        check=True,
        capture_output=True,
    ).stdout


def validate_report(value: object) -> None:
    record = require_record(value, (
        "schema", "status", "stage", "revision", "imported_gates", "candidates",
        "operations", "source_inventory", "source_projection_sha256", "tests",
        "counts", "properties", "result",
    ), "report")
    require(
        (record["schema"], record["status"], record["stage"], record["revision"], record["result"])
        == ("nostr_automerge.target_work_accounting.v11.v1", "pass", "rust_complete_target_work_accounting", "draft_2026_08", "pass"),
        "report:identity",
    )
    require(record["imported_gates"] == {
        "persistent_core_sha256": CORE_SHA256,
        "persistent_integration_sha256": INTEGRATION_SHA256,
    } and tuple(record["imported_gates"]) == ("persistent_core_sha256", "persistent_integration_sha256"), "report:imports")
    candidate_rows = []
    require(isinstance(record["candidates"], list), "report:candidates:type")
    for index, value in enumerate(record["candidates"]):
        item = require_record(value, ("step", "candidate"), f"report:candidate:{index}")
        candidate_rows.append((item["step"], item["candidate"]))
    require(tuple(candidate_rows) == CANDIDATES, "report:candidates")
    operation_rows = []
    require(isinstance(record["operations"], list), "report:operations:type")
    for index, value in enumerate(record["operations"]):
        item = require_record(value, ("family", "mode", "owner", "boundary", "proof"), f"report:operation:{index}")
        operation_rows.append(tuple(item[key] for key in ("family", "mode", "owner", "boundary", "proof")))
    require(tuple(operation_rows) == OPERATIONS, "report:operations")
    source_rows = []
    require(isinstance(record["source_inventory"], list), "report:sources:type")
    for index, value in enumerate(record["source_inventory"]):
        item = require_record(value, ("path", "sha256"), f"report:source:{index}")
        source_rows.append((item["path"], item["sha256"]))
    require(tuple(source_rows) == SOURCE_INVENTORY, "report:sources")
    projection = hashlib.sha256(json.dumps(record["source_inventory"], ensure_ascii=True, separators=(",", ":")).encode()).hexdigest()
    require(projection == SOURCE_PROJECTION_SHA256 == record["source_projection_sha256"], "report:source_projection")
    require(tuple(record["tests"]) == tuple(name.rsplit("::", 1)[-1] for _target, name in TESTS), "report:tests")
    require(record["counts"] == COUNTS and tuple(record["counts"]) == tuple(COUNTS), "report:counts")
    require(record["properties"] == PROPERTIES and tuple(record["properties"]) == tuple(PROPERTIES), "report:properties")


def validate_schema(value: object) -> None:
    record = require_record(value, ("$schema", "$id", "title", "type", "additionalProperties", "required", "properties"), "schema")
    required = ("schema", "status", "stage", "revision", "imported_gates", "candidates", "operations", "source_inventory", "source_projection_sha256", "tests", "counts", "properties", "result")
    require(record["type"] == "object" and record["additionalProperties"] is False, "schema:closed")
    require(tuple(record["required"]) == required, "schema:required")
    properties = record["properties"]
    require(isinstance(properties, dict) and tuple(properties) == required, "schema:properties")
    for name, count in (("candidates", 7), ("operations", 15), ("source_inventory", 15), ("tests", 17)):
        item = properties[name]
        require(isinstance(item, dict) and item.get("minItems") == count and item.get("maxItems") == count, f"schema:{name}")
    for name in ("imported_gates", "counts", "properties"):
        item = properties[name]
        require(isinstance(item, dict) and item.get("additionalProperties") is False, f"schema:{name}:closed")


def validate_sources(sources: dict[str, str]) -> None:
    joined = "\n".join(sources.values())
    boundary_sources = {
        "prepare_initial_maps_metered": (SOURCE_INVENTORY[4][0], "fn prepare_initial_maps_metered"),
        "record_control_metered": (SOURCE_INVENTORY[4][0], "fn record_control_metered"),
        "project_selected_prior_knowledge_metered": (SOURCE_INVENTORY[5][0], "fn project_selected_prior_knowledge_metered"),
        "clone_hash_set_metered": (SOURCE_INVENTORY[4][0], "fn clone_hash_set_metered"),
        "insert_branch_state_metered": (SOURCE_INVENTORY[4][0], "fn insert_branch_state_metered"),
        "project_accepted_candidate_maps_metered": (SOURCE_INVENTORY[4][0], "fn project_accepted_candidate_maps_metered"),
        "apply_exact_closure_metered": (SOURCE_INVENTORY[6][0], "fn apply_exact_closure_metered"),
        "derive_heads_metered": (SOURCE_INVENTORY[4][0], "fn derive_heads_metered"),
        "collect_change_dependencies_metered": (SOURCE_INVENTORY[5][0], "fn collect_change_dependencies_metered"),
        "selected_manifest_in_metered": (SOURCE_INVENTORY[8][0], "fn selected_manifest_in_metered"),
        "quarantine_equivocation_descendants": (SOURCE_INVENTORY[10][0], "fn quarantine_equivocation_descendants"),
        "ReportFinalizationPermit": (SOURCE_INVENTORY[5][0], "struct ReportFinalizationPermit"),
        "reserved_interrupted_report": (SOURCE_INVENTORY[5][0], "fn reserved_interrupted_report"),
    }
    for _family, _mode, _owner, boundary, proof in OPERATIONS:
        if boundary == "PersistentDeltaMap::*_metered":
            for anchor in ("fn get_metered", "fn contains_key_metered", "fn extend_prepared_metered", "fn materialize_metered"):
                require(anchor in sources[SOURCE_INVENTORY[0][0]], f"source:boundary:{anchor}")
        elif boundary == "ParentEpochView::*_metered":
            for anchor in ("fn from_result_metered", "fn set_additional_prior_knowledge_metered", "fn frontier_knowledge_metered"):
                require(anchor in sources[SOURCE_INVENTORY[1][0]], f"source:boundary:{anchor}")
        else:
            path, anchor = boundary_sources[boundary]
            require(anchor in sources[path], f"source:boundary:{boundary}")
        require(f"fn {proof}()" in joined, f"source:proof:{proof}")
    branch = sources[SOURCE_INVENTORY[0][0]]
    require("#[ignore = \"open remediation v11 finding\"]\n    fn finding_096" not in branch, "source:finding096:ignored")
    require("#[ignore]\n    fn finding_096" not in branch, "source:finding096:ignored")
    public = sources[SOURCE_INVENTORY[13][0]]
    require("calls.get(),\n            cancel_at + 1" in public, "source:post_stop:observation")
    require("no work boundary may be observed after the first positive cancellation" in public, "source:post_stop:diagnostic")


def validate_repository() -> None:
    require(sha256(REPORT) == REPORT_SHA256, "repository:report_hash")
    require(sha256(SCHEMA) == SCHEMA_SHA256, "repository:schema_hash")
    require(sha256(CORE) == CORE_SHA256, "repository:core_hash")
    require(sha256(INTEGRATION) == INTEGRATION_SHA256, "repository:integration_hash")
    validate_report(json.loads(REPORT.read_text(encoding="utf-8")))
    validate_schema(json.loads(SCHEMA.read_text(encoding="utf-8")))
    for (step, candidate), parent in zip(CANDIDATES, PARENTS, strict=True):
        resolved = subprocess.run(["git", "rev-parse", f"{candidate}^"], cwd=ROOT, check=True, capture_output=True, text=True).stdout.strip()
        require(resolved == parent, f"repository:parent:{step}")
    closure_parent = subprocess.run(["git", "rev-parse", f"{CLOSURE_CANDIDATE}^"], cwd=ROOT, check=True, capture_output=True, text=True).stdout.strip()
    require(closure_parent == CANDIDATES[-1][1], "repository:closure_parent")
    closure_scope = tuple(subprocess.run(
        ["git", "diff-tree", "--no-commit-id", "--name-only", "-r", CLOSURE_CANDIDATE],
        cwd=ROOT,
        check=True,
        capture_output=True,
        text=True,
    ).stdout.splitlines())
    require(closure_scope == CLOSURE_SCOPE, "repository:closure_scope")
    sources = {path: historical_bytes(path).decode("utf-8") for path, _digest in SOURCE_INVENTORY}
    for path, digest in SOURCE_INVENTORY:
        require(hashlib.sha256(sources[path].encode()).hexdigest() == digest, f"repository:source:{path}")
    validate_sources(sources)
    reproduction = json.loads(historical_bytes(REPRODUCTIONS.relative_to(ROOT).as_posix()).decode("utf-8"))
    require([row["expected"] for row in reproduction["cases"]] == ["fixed_pass", "fixed_pass", "open_failure", "open_failure"], "repository:reproductions")


def mutation_self_test() -> tuple[int, int, int]:
    report = json.loads(REPORT.read_text(encoding="utf-8"))
    schema = json.loads(SCHEMA.read_text(encoding="utf-8"))
    report_mutations = []
    for mutate in (
        lambda value: value["imported_gates"].update(persistent_core_sha256="0" * 64),
        lambda value: value["candidates"].pop(),
        lambda value: value["candidates"].reverse(),
        lambda value: value["candidates"][0].update(candidate="0" * 40),
        lambda value: value["operations"].pop(),
        lambda value: value["operations"].reverse(),
        lambda value: value["operations"][0].update(mode="constant_time"),
        lambda value: value["operations"][1].update(owner="none"),
        lambda value: value["operations"][2].update(boundary="unmetered"),
        lambda value: value["operations"][3].update(proof="missing"),
        lambda value: value["source_inventory"].pop(),
        lambda value: value["source_inventory"].reverse(),
        lambda value: value["source_inventory"][0].update(sha256="0" * 64),
        lambda value: value.update(source_projection_sha256="0" * 64),
        lambda value: value["tests"].pop(),
        lambda value: value["tests"].reverse(),
        lambda value: value["counts"].update(source_policy_mutations=62),
        lambda value: value["properties"].update(post_stop_observations=1),
        lambda value: value["properties"].update(bounded_teardown_qualified=True),
        lambda value: value.update(extra=False),
    ):
        candidate = copy.deepcopy(report)
        mutate(candidate)
        report_mutations.append(candidate)
    for index, candidate in enumerate(report_mutations):
        try:
            validate_report(candidate)
        except TargetWorkGateError:
            continue
        raise TargetWorkGateError(f"mutation:report:{index}")
    schema_mutations = []
    for mutate in (
        lambda value: value.update(additionalProperties=True),
        lambda value: value["required"].pop(),
        lambda value: value["properties"].pop("tests"),
        lambda value: value["properties"]["operations"].update(maxItems=16),
        lambda value: value["properties"]["counts"].update(additionalProperties=True),
    ):
        candidate = copy.deepcopy(schema)
        mutate(candidate)
        schema_mutations.append(candidate)
    for index, candidate in enumerate(schema_mutations):
        try:
            validate_schema(candidate)
        except TargetWorkGateError:
            continue
        raise TargetWorkGateError(f"mutation:schema:{index}")
    sources = {path: historical_bytes(path).decode("utf-8") for path, _digest in SOURCE_INVENTORY}
    source_mutations = []
    for path, old, new in (
        (SOURCE_INVENTORY[0][0], "fn get_metered", "fn get_unmetered"),
        (SOURCE_INVENTORY[0][0], "    fn finding_096", "    #[ignore]\n    fn finding_096"),
        (SOURCE_INVENTORY[4][0], "fn derive_heads_metered", "fn derive_heads"),
        (SOURCE_INVENTORY[5][0], "fn collect_change_dependencies_metered", "fn collect_change_dependencies"),
        (SOURCE_INVENTORY[8][0], "fn selected_manifest_in_metered", "fn selected_manifest_in"),
        (SOURCE_INVENTORY[10][0], "fn quarantine_traversal_has_exact_prefix", "fn quarantine_traversal_has_prefix"),
        (SOURCE_INVENTORY[13][0], "cancel_at + 1", "cancel_at + 2"),
    ):
        candidate = dict(sources)
        require(old in candidate[path], f"mutation:source:anchor:{path}")
        candidate[path] = candidate[path].replace(old, new, 1)
        source_mutations.append(candidate)
    for index, candidate in enumerate(source_mutations):
        try:
            validate_sources(candidate)
        except TargetWorkGateError:
            continue
        raise TargetWorkGateError(f"mutation:source:{index}")
    return len(report_mutations), len(schema_mutations), len(source_mutations)


def run_proofs() -> int:
    outputs: dict[str, str] = {}
    for target, command in (
        ("lib", ["cargo", "test", "-p", "nostr_automerge", "--lib", "--locked"]),
        ("public_engine_api", ["cargo", "test", "-p", "nostr_automerge", "--test", "public_engine_api", "--locked"]),
    ):
        outputs[target] = subprocess.run(command, cwd=ROOT, check=True, capture_output=True, text=True).stdout
    for target, test in TESTS:
        require(f"test {test} ... ok" in outputs[target], f"proof:{target}:{test}")
    return len(TESTS)


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--run-proofs", action="store_true")
    args = parser.parse_args()
    validate_repository()
    report_mutations, schema_mutations, source_mutations = mutation_self_test()
    executed = run_proofs() if args.run_proofs else 0
    print("PASS: target work accounting v11 gate")
    print(f"- candidates={len(CANDIDATES)}")
    print(f"- operations={len(OPERATIONS)}")
    print(f"- sources={len(SOURCE_INVENTORY)}")
    print(f"- tests={len(TESTS)}")
    print(f"- executed={executed}")
    print(f"- mutations={report_mutations + schema_mutations + source_mutations}")


if __name__ == "__main__":
    main()
