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
    parent_metered = section(parent, "pub(crate) fn frontier_knowledge_metered<E>(", "    #[cfg(test)]\n    pub(crate) fn frontier_knowledge", "parent:metered")
    require(".get_metered(hash, visit)?" in parent_metered, "parent:metered_lookup")

    transition = sources[SOURCES[3]]
    require_cfg_test(transition, "pub(crate) fn validate_base_frontier_antichain(", "transition:antichain")
    require_cfg_test(transition, "pub(crate) fn validate_retained_writer_frontier(", "transition:retained_writer")

    candidate = sources[SOURCES[4]]
    require_cfg_test(candidate, "pub(crate) fn evaluate_child(", "candidate:child")
    require_cfg_test(candidate, "pub(crate) fn evaluate_retained_writer_continuity(", "candidate:writer")
    require(candidate.count("reasoned_frontier_disposition_metered(") == 1, "candidate:metered_frontier")
    require(candidate.count("frontier_knowledge_metered(hash") == 1, "candidate:metered_knowledge")

    epoch = sources[SOURCES[5]]
    require(epoch.count("prior_change_knowledge().get_metered(dependency") == 1, "epoch:metered_prior")
    require("prior_change_knowledge().get(dependency" not in epoch, "epoch:unmetered_prior")

    evaluate = sources[SOURCES[6]]
    require(evaluate.count(".extend_prepared_metered(") == 2, "evaluate:extensions")
    require(evaluate.count(".get_metered(") >= 4, "evaluate:lookups")
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
        replaced(sources, transition, "#[cfg(test)]\npub(crate) fn validate_base_frontier_antichain(", "pub(crate) fn validate_base_frontier_antichain("),
        replaced(sources, candidate, "frontier_knowledge_metered(hash", "frontier_knowledge(hash"),
        replaced(sources, epoch, "prior_change_knowledge().get_metered(dependency", "prior_change_knowledge().get(dependency"),
        replaced(sources, evaluate, "dispositions.get_metered(&hash, visit)?", "dispositions.get(&hash)"),
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
