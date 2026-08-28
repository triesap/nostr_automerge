#!/usr/bin/env python3
"""Validate the closed resource-operation and reproduction inventory."""

from __future__ import annotations

import argparse
import copy
import hashlib
import json
import pathlib
import re
import subprocess

ROOT = pathlib.Path(__file__).resolve().parents[1]
INVENTORY = ROOT / "spec/resource_operation_inventory_v10.json"
SCHEMA = ROOT / "tools/validation/resource_operation_inventory_v10.schema.json"

INVENTORY_SHA256 = "cae0e490046cd70f1798573bcf80e0e9f4d520e37afb19225a84845b11b63525"
SCHEMA_SHA256 = "6144d398c8f839c0a1442de04b4c9de1c34339f0b374bd11a1af8ecc859c15c0"
HARNESS_SHA256 = "86a09907bbd61f4a324af88f82d536a711877c16e14e3b80471aa53d83f6c303"
TOP_KEYS = ("schema", "status", "findings", "operations", "reproductions", "result")
OPERATION_IDS = (
    "parent_epoch_view_copy",
    "branch_prior_knowledge_copy",
    "branch_disposition_copy",
    "control_closure_precharge",
    "device_ancestry_materialization",
    "accepted_state_reconstruction",
    "authoritative_epoch_preparation",
    "epoch_actor_reconstruction",
    "control_ancestry_index",
    "final_change_lineage",
    "carrier_contribution_vectors",
    "checkpoint_historical_control",
)
TESTS = (
    "control::parent_view::tests::finding_094_parent_epoch_view_shares_accepted_payload",
    "engine::reference_evaluator::tests::finding_095_lower_sequence_sibling_is_not_historical",
)
PROOF_TESTS = (
    "control::parent_view::tests::finding_094_parent_epoch_view_shares_accepted_payload",
    "reference::evaluate::tests::prior_knowledge_is_charged_per_item_before_access",
    "reference::branch_state::tests::delta_chain_shares_parent_and_materializes_in_override_order",
    "control::frontier::tests::metered_closure_charges_before_every_node_and_edge_operation",
    "control::ancestry::tests::ancestry_member_traversal_stops_at_every_prefix",
    "control::epoch_state::tests::metered_builder_has_exact_deep_wide_and_dense_boundaries",
    "reference::epoch_engine::tests::candidate_projections_charge_before_each_owned_entry",
    "reference::epoch_engine::tests::actor_reconstruction_is_item_metered_before_each_operation",
    "control::ancestry::tests::persistent_chain_retains_only_the_checked_parent_handle",
    "engine::reference_evaluator::tests::canonical_lineage_is_one_charged_control_and_hash_traversal",
    "engine::reference_evaluator::tests::aggregate_reduction_cannot_rewrite_an_invalid_carrier",
    "engine::reference_evaluator::tests::finding_095_lower_sequence_sibling_is_not_historical",
)


class InventoryError(RuntimeError):
    pass


def sha256(path: pathlib.Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def validate_record(record: object, *, inspect_source: bool) -> None:
    if not isinstance(record, dict) or tuple(record) != TOP_KEYS:
        raise InventoryError("inventory:keys")
    if record["schema"] != "nostr_automerge.resource_operation_inventory.v10.v1":
        raise InventoryError("inventory:schema")
    if record["status"] != "closed" or record["result"] != "pass":
        raise InventoryError("inventory:status")
    if record["findings"] != ["FINDING_094", "FINDING_095"]:
        raise InventoryError("inventory:findings")
    operations = record["operations"]
    if not isinstance(operations, list) or tuple(item.get("id") for item in operations if isinstance(item, dict)) != OPERATION_IDS:
        raise InventoryError("inventory:operations")
    if tuple(item.get("proof_test") for item in operations if isinstance(item, dict)) != PROOF_TESTS:
        raise InventoryError("inventory:proof_tests")
    if len(set(PROOF_TESTS)) != len(PROOF_TESTS):
        raise InventoryError("inventory:proof_duplicates")
    reproductions = record["reproductions"]
    if not isinstance(reproductions, list) or tuple(item.get("test") for item in reproductions if isinstance(item, dict)) != TESTS:
        raise InventoryError("inventory:reproductions")
    if [item.get("finding") for item in reproductions] != ["FINDING_094", "FINDING_095"]:
        raise InventoryError("inventory:reproduction_findings")
    if [item.get("expected") for item in reproductions] != ["fixed_pass", "fixed_pass"]:
        raise InventoryError("inventory:reproduction_status")
    if not inspect_source:
        return
    for operation in operations:
        path = ROOT / operation["path"]
        if not path.is_file():
            raise InventoryError(f"operation:path:{operation['id']}")
        source = path.read_text()
        function = re.compile(
            rf"(?:pub\(crate\)\s+)?(?:const\s+)?fn\s+{re.escape(operation['function'])}"
            r"(?:<[^>]+>)?\s*\("
        )
        if not function.search(source):
            raise InventoryError(f"operation:function:{operation['id']}")
        proof_source = (ROOT / operation["proof_source"]).read_text()
        short_name = operation["proof_test"].rsplit("::", 1)[1]
        declaration = re.search(rf"fn\s+{re.escape(short_name)}\s*\(\)", proof_source)
        if declaration is None:
            raise InventoryError(f"operation:proof:{operation['id']}")
        attributes = proof_source[max(0, declaration.start() - 220):declaration.start()]
        if "#[test]" not in attributes or "#[ignore" in attributes:
            raise InventoryError(f"operation:proof_attributes:{operation['id']}")
    for reproduction in reproductions:
        source = (ROOT / reproduction["source"]).read_text()
        short_name = reproduction["test"].rsplit("::", 1)[1]
        declaration = re.search(rf"fn\s+{re.escape(short_name)}\s*\(\)", source)
        if declaration is None:
            raise InventoryError(f"reproduction:test:{short_name}")
        attributes = source[max(0, declaration.start() - 220):declaration.start()]
        ignored = "#[ignore = \"open FINDING_" in attributes
        if "#[test]" not in attributes or ignored != (reproduction["expected"] == "open_failure"):
            raise InventoryError(f"reproduction:attributes:{short_name}")
        if reproduction["diagnostic"] not in source[declaration.start():]:
            raise InventoryError(f"reproduction:diagnostic:{short_name}")


METERED_SOURCE_ANCHORS = (
    (
        "crates/nostr_automerge/src/control/frontier.rs",
        "visit(crate::WorkCounter::GraphNode)?;\n        let hash = frontier[index];",
    ),
    (
        "crates/nostr_automerge/src/control/frontier.rs",
        "visit(crate::WorkCounter::GraphNode)?;\n        stack.push(frontier[frontier_index]);",
    ),
    (
        "crates/nostr_automerge/src/control/frontier.rs",
        "visit(crate::WorkCounter::GraphNode)?;\n        let Some(hash) = stack.pop()",
    ),
    (
        "crates/nostr_automerge/src/control/frontier.rs",
        "visit(crate::WorkCounter::GraphEdge)?;\n            let Some(dependency) = dependencies.next()",
    ),
    (
        "crates/nostr_automerge/src/reference/evaluate.rs",
        "crate::control::frontier::accepted_frontier_closure_metered(",
    ),
    (
        "crates/nostr_automerge/src/graph/actor_state.rs",
        "charge(WorkCounter::GraphNode).map_err(MeteredActorStateError::Work)?;\n        let Some(hash) = source.next_member()",
    ),
    (
        "crates/nostr_automerge/src/graph/actor_state.rs",
        "charge(WorkCounter::GraphEdge).map_err(MeteredActorStateError::Work)?;\n            let Some(dependency) = source.dependency(candidate, index)",
    ),
    (
        "crates/nostr_automerge/src/graph/dependency_graph.rs",
        "charge(crate::WorkCounter::GraphNode).map_err(MeteredGraphBuildError::Work)?;\n        let Some(candidate) = candidate_iter.next()",
    ),
    (
        "crates/nostr_automerge/src/graph/dependency_graph.rs",
        "charge(crate::WorkCounter::GraphEdge).map_err(MeteredGraphBuildError::Work)?;\n            let Some(dependency) = dependency_iter.next().copied()",
    ),
    (
        "crates/nostr_automerge/src/reference/epoch_engine.rs",
        "charge(counter)?;\n    let value = target();\n    observed(operation);",
    ),
    (
        "crates/nostr_automerge/src/reference/evaluate.rs",
        "charge_prior_knowledge_item(WorkCounter::GraphNode, budget, cancellation)?;\n        let Some(change) = changes.next()",
    ),
    (
        "crates/nostr_automerge/src/reference/epoch.rs",
        "budget\n                    .charge(WorkCounter::GraphEdge, 1)\n                    .map_err(|_| ScheduleError::BudgetExhausted)?;\n                let Some(dependency) = dependency_iter.next()",
    ),
    (
        "crates/nostr_automerge/src/engine/reference_evaluator.rs",
        "charge_evaluation_work(budget, cancellation, WorkCounter::Control, 1)\n            .map_err(ChangeReductionError::Stopped)?;\n        let Some(control_id) = controls.next()",
    ),
    (
        "crates/nostr_automerge/src/engine/reference_evaluator.rs",
        "charge_evaluation_work(budget, cancellation, WorkCounter::GraphNode, 1)\n                .map_err(ChangeReductionError::Stopped)?;\n            let Some(hash) = closure_hashes.next()",
    ),
    (
        "crates/nostr_automerge/src/engine/reference_evaluator.rs",
        "charge_evaluation_work(budget, cancellation, WorkCounter::Carrier, 1)\n                .map_err(ChangeReductionError::Stopped)?;\n            let Some(event_id) = event_iter.next().copied()",
    ),
    (
        "crates/nostr_automerge/src/engine/reference_evaluator.rs",
        "let final_accepted = &batch.accepted_changes;",
    ),
    (
        "crates/nostr_automerge/src/engine/reference_evaluator.rs",
        ".insert(outcome.event_id, outcome)\n                .is_some()",
    ),
    (
        "crates/nostr_automerge/src/engine/reference_evaluator.rs",
        "charge_checkpoint_work(budget, cancellation, 1)?;\n        if !historical.insert(control_id)",
    ),
    (
        "crates/nostr_automerge/src/engine/reference_evaluator.rs",
        "charge_checkpoint_work(budget, cancellation, 1)?;\n        let Some(EventEvidence::VerifiedCarrier",
    ),
    (
        "crates/nostr_automerge/src/engine/reference_evaluator.rs",
        "let Some(next_remaining) = remaining.checked_sub(1)",
    ),
    (
        "crates/nostr_automerge/src/engine/reference_evaluator.rs",
        "let historical_controls = checkpoint_historical_control_ancestry(",
    ),
    (
        "tools/nostr_automerge_conformance/src/fixture_generation.rs",
        "historical_exact_budget.checked_add(current_delta),\n                Some(exact),\n                \"{fixture_id}\"\n            );\n            assert!(signed.budget.max_items < exact, \"{fixture_id}\");",
    ),
)

PROJECTION_WORK_CONTRACT_PATH = "crates/nostr_automerge/src/graph/actor_state.rs"
PROJECTION_WORK_CONTRACT_TEST = (
    "projection_work_contract_preserves_first_stop_and_predecessor_output"
)
PROJECTION_WORK_CONTRACT_ANCHORS = (
    "const TOTAL_CHARGES: usize = 41;",
    "const GRAPH_NODES: usize = 32;",
    "const GRAPH_EDGES: usize = 9;",
    "for successful_limit in 0..TOTAL_CHARGES",
    "for stopped in [Completion::BudgetExhausted, Completion::Cancelled]",
    "successful_limit + 1",
    "Some(ProjectionWorkTrace::Charge(_))",
    "for successful_limit in [TOTAL_CHARGES, TOTAL_CHARGES + 1]",
    "actor_state_bytes(&metered_states)",
    "actor_state_bytes(&predecessor_states)",
    "core::ptr::eq(error, &injected)",
    "std::panic::panic_any(PANIC_IDENTITY)",
)


def validate_projection_work_contract(source: str) -> None:
    declaration = f"fn {PROJECTION_WORK_CONTRACT_TEST}()"
    if source.count(declaration) != 1:
        raise InventoryError("projection_contract:test")
    body = source.split(declaration, 1)[1].split("\n    #[test]", 1)[0]
    for anchor in PROJECTION_WORK_CONTRACT_ANCHORS:
        if anchor not in body:
            raise InventoryError(f"projection_contract:anchor:{anchor[:24]}")


def validate_metered_sources(sources: dict[str, str]) -> None:
    if "fn charge_control_closures(" in sources[
        "crates/nostr_automerge/src/reference/evaluate.rs"
    ]:
        raise InventoryError("metered_source:coarse_precharge")
    if "fn charge_actor_reconstruction(" in sources[
        "crates/nostr_automerge/src/reference/epoch_engine.rs"
    ]:
        raise InventoryError("metered_source:actor_precharge")
    if "let before = dispositions.clone();" in sources[
        "crates/nostr_automerge/src/reference/epoch.rs"
    ]:
        raise InventoryError("metered_source:disposition_clone")
    reduction = sources["crates/nostr_automerge/src/engine/reference_evaluator.rs"]
    for forbidden in (
        "let final_accepted = batch.accepted_changes.clone();",
        "let mut aggregate_contributions = Vec::new();",
        "let mut outcomes = Vec::new();",
        "batch.canonical_controls.iter().any(",
        "candidate_sequence < through_sequence",
    ):
        if forbidden in reduction:
            raise InventoryError(f"metered_source:reduction:{forbidden}")
    for path in (
        "crates/nostr_automerge/src/reference/epoch_engine.rs",
        "crates/nostr_automerge/src/reference/evaluate.rs",
    ):
        if "build_graph_metered(" not in sources[path]:
            raise InventoryError(f"metered_source:graph_bypass:{path}")
    for path, anchor in METERED_SOURCE_ANCHORS:
        if anchor not in sources[path]:
            raise InventoryError(f"metered_source:{path}:{anchor[:24]}")


def mutation_self_test() -> int:
    original = json.loads(INVENTORY.read_text())
    mutations = []
    for mutate in (
        lambda value: value.update(status="implementation_in_progress"),
        lambda value: value["findings"].reverse(),
        lambda value: value["operations"].pop(),
        lambda value: value["operations"].reverse(),
        lambda value: value["operations"][0].update(id="other"),
        lambda value: value["operations"][0].update(proof_test="module::other"),
        lambda value: value["operations"][1].update(proof_test=PROOF_TESTS[0]),
        lambda value: value["reproductions"].pop(),
        lambda value: value["reproductions"].reverse(),
        lambda value: value["reproductions"][0].update(expected="open_failure"),
        lambda value: value.update(extra=False),
    ):
        candidate = copy.deepcopy(original)
        mutate(candidate)
        mutations.append(candidate)
    for index, mutation in enumerate(mutations):
        try:
            validate_record(mutation, inspect_source=False)
        except InventoryError:
            continue
        raise InventoryError(f"mutation:{index}")
    return len(mutations)


def source_mutation_self_test() -> int:
    paths = {path for path, _ in METERED_SOURCE_ANCHORS}
    sources = {path: (ROOT / path).read_text() for path in paths}
    validate_metered_sources(sources)
    validate_projection_work_contract(sources[PROJECTION_WORK_CONTRACT_PATH])
    mutations = []
    for path, anchor in METERED_SOURCE_ANCHORS:
        candidate = dict(sources)
        replacement = anchor.split("\n")[-1] if "\n" in anchor else ""
        candidate[path] = candidate[path].replace(anchor, replacement)
        mutations.append(candidate)
    candidate = dict(sources)
    candidate["crates/nostr_automerge/src/reference/evaluate.rs"] += (
        "\nfn charge_control_closures() {}\n"
    )
    mutations.append(candidate)
    candidate = dict(sources)
    candidate["crates/nostr_automerge/src/reference/epoch_engine.rs"] += (
        "\nfn charge_actor_reconstruction() {}\n"
    )
    mutations.append(candidate)
    candidate = dict(sources)
    candidate["crates/nostr_automerge/src/reference/epoch.rs"] += (
        "\nfn stale_clone() { let before = dispositions.clone(); }\n"
    )
    mutations.append(candidate)
    for forbidden in (
        "let final_accepted = batch.accepted_changes.clone();",
        "let mut aggregate_contributions = Vec::new();",
        "let mut outcomes = Vec::new();",
        "batch.canonical_controls.iter().any(",
        "candidate_sequence < through_sequence",
    ):
        candidate = dict(sources)
        candidate["crates/nostr_automerge/src/engine/reference_evaluator.rs"] += forbidden
        mutations.append(candidate)
    for anchor in PROJECTION_WORK_CONTRACT_ANCHORS:
        candidate = dict(sources)
        before, separator, after = candidate[PROJECTION_WORK_CONTRACT_PATH].rpartition(
            anchor
        )
        if not separator:
            raise InventoryError(f"source_mutation_anchor:{anchor[:24]}")
        candidate[PROJECTION_WORK_CONTRACT_PATH] = before + after
        mutations.append(candidate)
    for index, mutation in enumerate(mutations):
        try:
            validate_metered_sources(mutation)
            validate_projection_work_contract(
                mutation[PROJECTION_WORK_CONTRACT_PATH]
            )
        except InventoryError:
            continue
        raise InventoryError(f"source_mutation:{index}")
    return len(mutations)


def run_proofs() -> int:
    for proof in PROOF_TESTS:
        result = subprocess.run(
            [
                "cargo", "extbuild", "run", "--", "cargo", "test",
                "-p", "nostr_automerge", "--lib", "--locked", "--",
                "--exact", proof,
            ],
            cwd=ROOT,
            check=False,
        )
        if result.returncode != 0:
            raise InventoryError(f"proof:failed:{proof}")
    return len(PROOF_TESTS)


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--run-proofs", action="store_true")
    args = parser.parse_args()
    if sha256(INVENTORY) != INVENTORY_SHA256:
        raise InventoryError("inventory:sha256")
    if sha256(SCHEMA) != SCHEMA_SHA256:
        raise InventoryError("schema:sha256")
    if sha256(ROOT / "scripts/reproduce_resource_followup_v10.py") != HARNESS_SHA256:
        raise InventoryError("harness:sha256")
    validate_record(json.loads(INVENTORY.read_text()), inspect_source=True)
    mutations = mutation_self_test()
    sources = {path: (ROOT / path).read_text() for path, _ in METERED_SOURCE_ANCHORS}
    validate_metered_sources(sources)
    validate_projection_work_contract(sources[PROJECTION_WORK_CONTRACT_PATH])
    source_mutations = source_mutation_self_test()
    executed = run_proofs() if args.run_proofs else 0
    print("PASS: resource operation inventory v10")
    print(f"- operations={len(OPERATION_IDS)}")
    print(f"- reproductions={len(TESTS)}")
    print(f"- mutations={mutations}")
    print(f"- source_mutations={source_mutations}")
    print(f"- proofs={len(PROOF_TESTS)}")
    print("- projection_contract=1")
    print(f"- executed={executed}")


if __name__ == "__main__":
    main()
