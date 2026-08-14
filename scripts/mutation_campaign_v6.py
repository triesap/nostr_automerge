#!/usr/bin/env python3
"""Validate deterministic remediation-v6 mutation anchors without altering source."""

from __future__ import annotations

import json
from dataclasses import asdict, dataclass
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
OUTPUT = ROOT / "reports/mutation_campaign_v6_inventory.json"


@dataclass(frozen=True)
class MutationAnchor:
    name: str
    path: str
    search: str
    test_path: str
    test_filter: str


ANCHORS = (
    MutationAnchor("missing_parent_becomes_invalid", "crates/nostr_automerge/src/control/reference_state.rs", "Self::Pending(_) | Self::Missing => Some(ProtocolDisposition::Pending),", "crates/nostr_automerge/src/control/reference_state.rs", "every_parent_state_has_an_exhaustive_dependent_outcome"),
    MutationAnchor("known_unusable_parent_becomes_pending", "crates/nostr_automerge/src/control/reference_state.rs", "| Self::UnsupportedRevision => Some(ProtocolDisposition::Invalid),", "crates/nostr_automerge/src/control/reference_state.rs", "every_parent_state_has_an_exhaustive_dependent_outcome"),
    MutationAnchor("invalid_frontier_loses_precedence", "crates/nostr_automerge/src/control/frontier.rs", "return Some(crate::ProtocolDisposition::Invalid);", "crates/nostr_automerge/src/control/frontier.rs", "invalid_head_rejects_the_frontier"),
    MutationAnchor("pending_descendant_propagation_removed", "crates/nostr_automerge/src/reference/evaluate.rs", "Some(ProtocolDisposition::Pending) => ProtocolDisposition::Pending,", "crates/nostr_automerge/src/reference/evaluate.rs", "deep_pending_chain_reaches_a_fixed_point_independent_of_id_order"),
    MutationAnchor("invalid_descendant_propagation_removed", "crates/nostr_automerge/src/reference/evaluate.rs", "ProtocolDisposition::Invalid\n                }", "crates/nostr_automerge/src/reference/evaluate.rs", "deep_invalid_chain_reaches_a_fixed_point_independent_of_id_order"),
    MutationAnchor("noncanonical_branch_validation_removed", "crates/nostr_automerge/src/engine/reference_evaluator.rs", "(*disposition == ProtocolDisposition::Excluded).then_some(*event_id)", "crates/nostr_automerge/tests/public_engine_api.rs", "deep_noncanonical_branch_is_validated_before_exclusion"),
    MutationAnchor("missing_descriptor_becomes_invalid", "crates/nostr_automerge/src/checkpoint/reference_state.rs", "Self::Pending(_) | Self::Missing => Some(ProtocolDisposition::Pending),", "crates/nostr_automerge/src/checkpoint/reference_state.rs", "every_descriptor_reference_state_has_one_dependent_outcome"),
    MutationAnchor("known_unusable_descriptor_becomes_pending", "crates/nostr_automerge/src/checkpoint/reference_state.rs", "| Self::UnsupportedRevision => Some(ProtocolDisposition::Invalid),", "crates/nostr_automerge/src/checkpoint/reference_state.rs", "every_descriptor_reference_state_has_one_dependent_outcome"),
    MutationAnchor("verified_orphan_chunk_remains_excluded", "crates/nostr_automerge/src/engine/reference_evaluator.rs", ".unwrap_or((ProtocolDisposition::Pending, None));", "crates/nostr_automerge/tests/public_engine_api.rs", "orphan_checkpoint_chunk_promotes_after_descriptor_arrival"),
    MutationAnchor("unsupported_control_mapping", "crates/nostr_automerge/src/engine/reference_evaluator.rs", "ReferencedControlState::UnsupportedRevision => {\n                        ChangeClaimReason::InvalidReferencedControl", "crates/nostr_automerge/tests/public_engine_api.rs", "unsupported_control_reference_is_invalid"),
    MutationAnchor("prior_knowledge_charge", "crates/nostr_automerge/src/engine/reference_evaluator.rs", "fn additional_prior_knowledge(", "crates/nostr_automerge/tests/public_engine_api.rs", "prior_knowledge_exhaustion_is_deterministic_at_every_item_boundary"),
    MutationAnchor("finalization_remainder", "crates/nostr_automerge/src/engine/reference_evaluator.rs", "self.remaining != ReportFinalizationPlan::default()", "crates/nostr_automerge/src/engine/reference_evaluator.rs", "finalization_dimensions_reject_underflow_and_double_finish"),
    MutationAnchor("generic_critical_proof", "scripts/validate_requirement_matrix_v7.py", "generic-critical", "scripts/validate_requirement_matrix_v7.py", "generic_proof"),
)


def main() -> int:
    rows = []
    for anchor in ANCHORS:
        source = (ROOT / anchor.path).read_text(encoding="utf-8")
        if source.count(anchor.search) != 1:
            raise AssertionError(f"stale mutation anchor: {anchor.name}")
        test = (ROOT / anchor.test_path).read_text(encoding="utf-8")
        if anchor.test_filter not in test:
            raise AssertionError(f"missing mutation detector: {anchor.name}")
        rows.append({**asdict(anchor), "inventory_status": "validated"})
    report = {
        "schema": "nostr_automerge.mutation_inventory.v6",
        "status": "validated",
        "target_count": len(rows),
        "execution": "deferred_external_hold",
        "targets": rows,
    }
    OUTPUT.write_text(json.dumps(report, sort_keys=True, separators=(",", ":")) + "\n")
    print(f"PASS: {len(rows)} remediation-v6 mutation anchors")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
