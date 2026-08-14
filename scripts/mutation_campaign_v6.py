#!/usr/bin/env python3
"""Validate deterministic remediation-v6 reference-resolution mutation anchors."""

from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]


@dataclass(frozen=True)
class MutationAnchor:
    name: str
    path: str
    search: str
    test_filter: str


ANCHORS = (
    MutationAnchor(
        "missing_parent_becomes_invalid",
        "crates/nostr_automerge/src/control/reference_state.rs",
        "Self::Pending(_) | Self::Missing => Some(ProtocolDisposition::Pending),",
        "every_parent_state_has_an_exhaustive_dependent_outcome",
    ),
    MutationAnchor(
        "known_unusable_parent_becomes_pending",
        "crates/nostr_automerge/src/control/reference_state.rs",
        "| Self::UnsupportedRevision => Some(ProtocolDisposition::Invalid),",
        "every_parent_state_has_an_exhaustive_dependent_outcome",
    ),
    MutationAnchor(
        "invalid_frontier_loses_precedence",
        "crates/nostr_automerge/src/control/frontier.rs",
        "return Some(crate::ProtocolDisposition::Invalid);",
        "invalid_head_rejects_the_frontier",
    ),
    MutationAnchor(
        "pending_descendant_propagation_removed",
        "crates/nostr_automerge/src/reference/evaluate.rs",
        "Some(ProtocolDisposition::Pending) => ProtocolDisposition::Pending,",
        "deep_pending_chain_reaches_a_fixed_point_independent_of_id_order",
    ),
    MutationAnchor(
        "invalid_descendant_propagation_removed",
        "crates/nostr_automerge/src/reference/evaluate.rs",
        "ProtocolDisposition::Invalid\n                }",
        "deep_invalid_chain_reaches_a_fixed_point_independent_of_id_order",
    ),
    MutationAnchor(
        "noncanonical_branch_validation_removed",
        "crates/nostr_automerge/src/engine/reference_evaluator.rs",
        "(*disposition == ProtocolDisposition::Excluded).then_some(*event_id)",
        "deep_noncanonical_branch_is_validated_before_exclusion",
    ),
    MutationAnchor(
        "missing_descriptor_becomes_invalid",
        "crates/nostr_automerge/src/checkpoint/reference_state.rs",
        "Self::Pending(_) | Self::Missing => Some(ProtocolDisposition::Pending),",
        "every_descriptor_reference_state_has_one_dependent_outcome",
    ),
    MutationAnchor(
        "known_unusable_descriptor_becomes_pending",
        "crates/nostr_automerge/src/checkpoint/reference_state.rs",
        "| Self::UnsupportedRevision => Some(ProtocolDisposition::Invalid),",
        "every_descriptor_reference_state_has_one_dependent_outcome",
    ),
    MutationAnchor(
        "verified_orphan_chunk_remains_excluded",
        "crates/nostr_automerge/src/engine/reference_evaluator.rs",
        ".unwrap_or((ProtocolDisposition::Pending, None));",
        "orphan_checkpoint_chunk_promotes_after_descriptor_arrival",
    ),
)


def main() -> int:
    for anchor in ANCHORS:
        source = (ROOT / anchor.path).read_text(encoding="utf-8")
        if source.count(anchor.search) != 1:
            raise AssertionError(f"stale mutation anchor: {anchor.name}")
        if not anchor.test_filter:
            raise AssertionError(f"missing mutation detector: {anchor.name}")
    print(f"PASS: {len(ANCHORS)} remediation-v6 reference mutation anchors")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
