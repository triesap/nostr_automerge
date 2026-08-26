#!/usr/bin/env python3
"""Fail closed over the v11 persistent-state production boundary."""

from __future__ import annotations

import copy
import pathlib
import re
import sys

sys.dont_write_bytecode = True

ROOT = pathlib.Path(__file__).resolve().parents[1]
SOURCES = (
    "crates/nostr_automerge/src/reference/branch_state.rs",
    "crates/nostr_automerge/src/control/frontier.rs",
    "crates/nostr_automerge/src/control/parent_view.rs",
    "crates/nostr_automerge/src/control/transition.rs",
    "crates/nostr_automerge/src/control/candidate.rs",
    "crates/nostr_automerge/src/reference/epoch_engine.rs",
    "crates/nostr_automerge/src/reference/evaluate.rs",
    "crates/nostr_automerge/src/engine/reference_evaluator.rs",
)


class PersistentStateError(RuntimeError):
    """The production persistent-state boundary is stale or bypassable."""


def require(condition: bool, diagnostic: str) -> None:
    if not condition:
        raise PersistentStateError(diagnostic)


def load_sources() -> dict[str, str]:
    return {relative: (ROOT / relative).read_text(encoding="utf-8") for relative in SOURCES}


def section(source: str, start: str, end: str, label: str) -> str:
    begin = source.find(start)
    require(begin >= 0, f"{label}:start")
    finish = source.find(end, begin + len(start))
    require(finish >= 0, f"{label}:end")
    return source[begin:finish]


def require_cfg_test(source: str, declaration: str, label: str) -> None:
    pattern = rf"#\[cfg\(test\)\]\s+{re.escape(declaration)}"
    require(re.search(pattern, source) is not None, f"{label}:cfg_test")


def require_order(source: str, anchors: tuple[str, ...], label: str) -> None:
    cursor = -1
    for anchor in anchors:
        position = source.find(anchor, cursor + 1)
        require(position >= 0, f"{label}:anchor:{anchor}")
        require(position > cursor, f"{label}:order:{anchor}")
        cursor = position


def validate_sources(sources: dict[str, str]) -> None:
    require(tuple(sources) == SOURCES, "sources:inventory")
    branch = sources[SOURCES[0]]
    for declaration, label in (
        ("pub(crate) fn get(&self, key: &K) -> Option<&V>", "branch:get"),
        ("pub(crate) fn contains_key(&self, key: &K) -> bool", "branch:contains"),
        ("pub(crate) fn extend_local(&self, mut local: BTreeMap<K, V>) -> Self", "branch:extend"),
        ("impl<K: Ord, V> From<BTreeMap<K, V>> for PersistentDeltaMap<K, V>", "branch:from"),
    ):
        require_cfg_test(branch, declaration, label)

    lookup = section(
        branch,
        "pub(crate) fn get_metered<E>(",
        "    #[cfg(test)]\n    pub(crate) fn contains_key",
        "branch:get_metered",
    )
    require("Option<impl FnMut" not in lookup, "branch:get_metered:optional")
    require_order(lookup, ("while let Some(node)", "visit()?;", "node.local.get(key)"), "branch:get_metered")

    membership = section(
        branch,
        "pub(crate) fn contains_key_metered<E>(",
        "    #[cfg(test)]\n    pub(crate) fn extend_local",
        "branch:contains_metered",
    )
    require(membership.count("self.get_metered(key, visit)") == 1, "branch:contains_metered:delegation")
    require("Option<impl FnMut" not in membership, "branch:contains_metered:optional")

    extension = section(
        branch,
        "pub(crate) fn extend_prepared_metered<E>(",
        "    pub(crate) fn materialize_metered<E>(",
        "branch:extend_metered",
    )
    require("Option<impl FnMut" not in extension, "branch:extend_metered:optional")
    require_order(
        extension,
        (
            "work(PersistentDeltaWork::PreparedItem)?;",
            "prepared.pop_first()",
            "self.get_metered(&key, || work(PersistentDeltaWork::LookupNode))?",
            "work(PersistentDeltaWork::AcceptedInsert)?;",
            "accepted.insert(key, value);",
            "tail: Some(Arc::new(DeltaNode",
        ),
        "branch:extend_metered",
    )

    materialize = section(
        branch,
        "pub(crate) fn materialize_metered<E>(",
        "    #[cfg(test)]\n    fn shares_parent_with",
        "branch:materialize_metered",
    )
    require_order(materialize, ("while let Some(node)", "visit()?;", "nodes.push(node)"), "branch:materialize_nodes")
    require_order(materialize, ("for (key, value) in &node.local", "visit()?;", "result.insert(*key, *value)"), "branch:materialize_items")

    frontier = sources[SOURCES[1]]
    require_cfg_test(frontier, "pub(crate) fn reasoned_frontier_disposition(", "frontier:reasoned")
    require_cfg_test(frontier, "pub(crate) fn accepted_frontier_closure(", "frontier:closure")
    require("pub(crate) fn reasoned_frontier_disposition_metered<E>(" in frontier, "frontier:metered")

    parent = sources[SOURCES[2]]
    require_cfg_test(parent, "pub(crate) fn frontier_knowledge(&self, hash: &ChangeHash)", "parent:knowledge")
    result_projection = section(
        parent,
        "pub(crate) fn from_result_metered<E>(",
        "    pub(crate) fn extend_prior_knowledge",
        "parent:result_projection",
    )
    require_order(
        result_projection,
        (
            "visit().map_err(ParentEpochViewBuildError::Work)?;",
            "dispositions.next()",
            "visit().map_err(ParentEpochViewBuildError::Work)?;",
            "view.frontier_knowledge.insert(*hash, knowledge);",
        ),
        "parent:result_projection",
    )
    additional_projection = section(
        parent,
        "pub(crate) fn set_additional_prior_knowledge_metered<E>(",
        "    #[cfg(test)]\n    pub(crate) fn from_parts_for_test",
        "parent:additional_projection",
    )
    require_order(
        additional_projection,
        (
            "visit().map_err(ParentEpochViewBuildError::Work)?;",
            "items.next()",
            "visit().map_err(ParentEpochViewBuildError::Work)?;",
            "projected.insert(*hash, *item);",
            "self.additional_prior = projected;",
        ),
        "parent:additional_projection",
    )
    parent_metered = section(parent, "pub(crate) fn frontier_knowledge_metered<E>(", "    #[cfg(test)]\n    pub(crate) fn frontier_knowledge", "parent:metered")
    require_order(
        parent_metered,
        (
            "visit()?;",
            "self.frontier_knowledge.get(hash)",
            ".get_metered(hash, &mut visit)?",
            "visit()?;",
            ".additional_prior",
            "visit()?;",
            "self.contains(hash)",
        ),
        "parent:metered_lookup",
    )

    transition = sources[SOURCES[3]]
    require_cfg_test(transition, "pub(crate) fn validate_base_frontier_antichain(", "transition:antichain")
    require_cfg_test(transition, "pub(crate) fn validate_retained_writer_frontier(", "transition:retained_writer")

    candidate = sources[SOURCES[4]]
    require_cfg_test(candidate, "pub(crate) fn evaluate_child(", "candidate:child")
    require_cfg_test(candidate, "pub(crate) fn evaluate_retained_writer_continuity(", "candidate:writer")
    require(candidate.count("reasoned_frontier_disposition_metered(") == 1, "candidate:metered_frontier")
    require(candidate.count("frontier_knowledge_metered(hash") == 1, "candidate:metered_knowledge")
    require(
        candidate.count("evaluate_candidate_frontier_metered(child, view, visit)?") == 1,
        "candidate:frontier_boundary",
    )

    epoch = sources[SOURCES[5]]
    dependency_lookup = section(
        epoch,
        "fn prior_dependencies_valid_metered<E>(",
        "\n}\n\nfn metered_hash_sets_equal",
        "epoch:dependency_lookup",
    )
    require_order(
        dependency_lookup,
        (
            "visit(WorkCounter::GraphEdge)?;",
            "declared_items.next()",
            ".get_metered(dependency, || visit(WorkCounter::GraphNode))?",
        ),
        "epoch:dependency_lookup",
    )
    require(
        dependency_lookup.count(".get_metered(dependency, || visit(WorkCounter::GraphNode))?")
        == 2,
        "epoch:dependency_persistent_reads",
    )
    require(".get(dependency)" not in dependency_lookup, "epoch:dependency_bypass")
    require(epoch.count("prior_dependencies_valid_metered(") == 2, "epoch:dependency_boundary")

    evaluate = sources[SOURCES[6]]
    initial_maps = section(
        evaluate,
        "fn prepare_initial_maps_metered<E>(",
        "\n}\n\nstruct ValidBranchEvaluation",
        "evaluate:initial_maps",
    )
    require_order(
        initial_maps,
        (
            "let mut collected_items = collected.into_iter();",
            "visit(WorkCounter::Control).map_err(InitialMapBuildError::Work)?;",
            "collected_items.next()",
            "controls.insert(control.event_id, control);",
            "let mut parent_items = controls.values();",
            "parent_items.next()",
            ".entry(control.parent)",
            "let mut control_ids = controls.keys();",
            "control_ids.next()",
            "control_dispositions.insert(*event_id, ProtocolDisposition::Excluded);",
            "let mut control_items = controls.values();",
            "control_items.next()",
            "changes.next()",
            "change_dispositions.insert(",
        ),
        "evaluate:initial_maps",
    )
    accepted_state = section(
        evaluate,
        "fn accepted_state_for_closure(",
        "\n}\n\nfn metered_hash_sets_equal",
        "evaluate:accepted_state",
    )
    require_order(
        accepted_state,
        (
            "let shared = parent.accepted_state_handle();",
            "charge_prior_knowledge_item(WorkCounter::GraphNode, budget, cancellation)?;",
            "cache.insert(Arc::clone(&cache_key), Arc::clone(&shared));",
            "let state = Arc::new(state);",
            "charge_prior_knowledge_item(WorkCounter::GraphNode, budget, cancellation)?;",
            "cache.insert(cache_key, Arc::clone(&state));",
        ),
        "evaluate:accepted_state_cache_inserts",
    )
    require(evaluate.count(".extend_prepared_metered(") == 2, "evaluate:extensions")
    require(evaluate.count(".get_metered(") >= 4, "evaluate:lookups")
    prior_extension = section(
        evaluate,
        "fn extend_prior_knowledge_metered<E>(",
        "\n}\n\nfn extend_branch_dispositions_metered",
        "evaluate:prior_extension",
    )
    require_order(
        prior_extension,
        (
            "visit(WorkCounter::GraphNode).map_err(BranchDeltaError::Work)?;",
            "items.next()",
            "visit(WorkCounter::GraphNode).map_err(BranchDeltaError::Work)?;",
            "local.entry(*hash).or_insert(*item);",
            ".extend_prepared_metered(local",
        ),
        "evaluate:prior_extension",
    )
    branch_extension = section(
        evaluate,
        "fn extend_branch_dispositions_metered<E>(",
        "\n}\n\nfn evaluate_branch_table",
        "evaluate:branch_extension",
    )
    require(branch_extension.count(".extend_prepared_metered(local") == 1, "evaluate:branch_publication")
    require(branch_extension.count(".get_metered(") == 3, "evaluate:branch_duplicate_checks")
    require(evaluate.count("extend_prior_knowledge_metered(") == 2, "evaluate:prior_route")
    require(evaluate.count("extend_branch_dispositions_metered(") == 2, "evaluate:branch_route")
    referenced = section(
        evaluate,
        "pub(crate) fn referenced_branch_change_disposition_metered<E>(",
        "}\n\nfn no_progress_batch_report",
        "evaluate:referenced",
    )
    require_order(referenced, ("visit()?;", "branch_change_dispositions.get(&control)", "dispositions.get_metered(&hash, visit)?"), "evaluate:referenced")

    evaluator = sources[SOURCES[7]]
    require("batch.referenced_branch_change_disposition(" not in evaluator, "evaluator:referenced_bypass")
    require(evaluator.count("referenced_branch_change_disposition_metered(") == 1, "evaluator:referenced_metered")
    require("dispositions.contains_key(&carrier.change_hash)" not in evaluator, "evaluator:membership_bypass")
    require(evaluator.count("dispositions.contains_key_metered(&carrier.change_hash") == 1, "evaluator:membership_metered")


def replaced(sources: dict[str, str], relative: str, old: str, new: str) -> dict[str, str]:
    candidate = copy.deepcopy(sources)
    require(old in candidate[relative], f"mutation:anchor:{relative}")
    candidate[relative] = candidate[relative].replace(old, new, 1)
    return candidate


def mutation_self_test(sources: dict[str, str]) -> int:
    branch, frontier, parent, transition, candidate, epoch, evaluate, evaluator = SOURCES
    mutations = [
        replaced(sources, branch, "#[cfg(test)]\n    pub(crate) fn get(", "    pub(crate) fn get("),
        replaced(sources, branch, "#[cfg(test)]\n    pub(crate) fn contains_key(", "    pub(crate) fn contains_key("),
        replaced(sources, branch, "#[cfg(test)]\n    pub(crate) fn extend_local(", "    pub(crate) fn extend_local("),
        replaced(sources, branch, "#[cfg(test)]\nimpl<K: Ord, V> From", "impl<K: Ord, V> From"),
        replaced(sources, branch, "visit()?;\n            if let Some(value)", "if let Some(value)"),
        replaced(sources, branch, "mut visit: impl FnMut()", "mut visit: Option<impl FnMut()>"),
        replaced(sources, branch, "work(PersistentDeltaWork::AcceptedInsert)?;", ""),
        replaced(sources, frontier, "#[cfg(test)]\npub(crate) fn reasoned_frontier_disposition(", "pub(crate) fn reasoned_frontier_disposition("),
        replaced(sources, parent, "#[cfg(test)]\n    pub(crate) fn frontier_knowledge(", "    pub(crate) fn frontier_knowledge("),
        replaced(sources, parent, "visit().map_err(ParentEpochViewBuildError::Work)?;\n            let Some((hash, disposition))", "let Some((hash, disposition))"),
        replaced(sources, parent, "visit().map_err(ParentEpochViewBuildError::Work)?;\n            projected.insert(*hash, *item);", "projected.insert(*hash, *item);"),
        replaced(sources, parent, "visit()?;\n        if let Some(knowledge) = self.frontier_knowledge", "if let Some(knowledge) = self.frontier_knowledge"),
        replaced(sources, transition, "#[cfg(test)]\npub(crate) fn validate_base_frontier_antichain(", "pub(crate) fn validate_base_frontier_antichain("),
        replaced(sources, candidate, "frontier_knowledge_metered(hash", "frontier_knowledge(hash"),
        replaced(sources, epoch, "visit(WorkCounter::GraphEdge)?;\n        let Some(dependency) = declared_items.next()", "let Some(dependency) = declared_items.next()"),
        replaced(sources, epoch, ".get_metered(dependency, || visit(WorkCounter::GraphNode))?", ".get(dependency)"),
        replaced(sources, evaluate, "dispositions.get_metered(&hash, visit)?", "dispositions.get(&hash)"),
        replaced(sources, evaluate, "visit(WorkCounter::GraphNode).map_err(BranchDeltaError::Work)?;\n            local.entry(*hash).or_insert(*item);", "local.entry(*hash).or_insert(*item);"),
        replaced(sources, evaluate, "visit(WorkCounter::Control).map_err(InitialMapBuildError::Work)?;\n        let Some(control) = collected_items.next()", "let Some(control) = collected_items.next()"),
        replaced(sources, evaluate, "charge_prior_knowledge_item(WorkCounter::GraphNode, budget, cancellation)?;\n    cache.insert(cache_key", "cache.insert(cache_key"),
        replaced(sources, evaluate, "let branch_change_dispositions = extend_branch_dispositions_metered(", "let branch_change_dispositions = parent_change_dispositions.extend_prepared_metered("),
        replaced(sources, evaluator, "dispositions.contains_key_metered(&carrier.change_hash", "dispositions.contains_key(&carrier.change_hash"),
    ]
    missing = copy.deepcopy(sources)
    missing.pop(evaluator)
    mutations.append(missing)
    for index, mutation in enumerate(mutations):
        try:
            validate_sources(mutation)
        except PersistentStateError:
            continue
        raise PersistentStateError(f"mutation:{index}")
    return len(mutations)


def main() -> None:
    sources = load_sources()
    validate_sources(sources)
    mutations = mutation_self_test(sources)
    print("PASS: persistent state v11 source policy")
    print(f"- sources={len(SOURCES)}")
    print(f"- mutations={mutations}")


if __name__ == "__main__":
    main()
