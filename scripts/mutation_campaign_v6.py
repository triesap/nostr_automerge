#!/usr/bin/env python3
"""Validate the remediation-v6 mutation inventory without altering source."""

from __future__ import annotations

import json
from dataclasses import asdict, dataclass
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
OUTPUT = ROOT / "reports/mutation_campaign_v6_inventory.json"


@dataclass(frozen=True)
class MutationTarget:
    name: str
    source_path: str
    source_anchor: str
    test_path: str
    test_id: str


TARGETS = (
    MutationTarget("unsupported_control_mapping", "crates/nostr_automerge/src/engine/reference_evaluator.rs", "ReferencedControlState::UnsupportedRevision", "crates/nostr_automerge/tests/public_engine_api.rs", "unsupported_control_reference_is_invalid"),
    MutationTarget("noncanonical_authorization_order", "crates/nostr_automerge/src/engine/reference_evaluator.rs", "ChangeClaimReason::Unauthorized", "crates/nostr_automerge/tests/public_engine_api.rs", "noncanonical_authorization_is_enforced_before_exclusion"),
    MutationTarget("terminal_control_mapping", "crates/nostr_automerge/src/control/validate.rs", "ControlValidationError::Terminal", "crates/nostr_automerge/tests/public_engine_api.rs", "terminal_control_change_is_invalid"),
    MutationTarget("missing_parent_mapping", "crates/nostr_automerge/src/control/reference_state.rs", "Self::Pending(_) | Self::Missing", "crates/nostr_automerge/src/control/reference_state.rs", "every_parent_state_has_an_exhaustive_dependent_outcome"),
    MutationTarget("invalid_frontier_mapping", "crates/nostr_automerge/src/control/frontier.rs", "Self::InvalidUnderParent", "crates/nostr_automerge/src/control/frontier.rs", "invalid_head_rejects_the_frontier"),
    MutationTarget("pending_descendant_propagation", "crates/nostr_automerge/src/reference/evaluate.rs", "ProtocolDisposition::Pending", "crates/nostr_automerge/src/reference/evaluate.rs", "pending_parent_state_propagates_through_descendants"),
    MutationTarget("invalid_descendant_propagation", "crates/nostr_automerge/src/reference/evaluate.rs", "ProtocolDisposition::Invalid", "crates/nostr_automerge/src/reference/evaluate.rs", "invalid_parent_state_propagates_through_descendants"),
    MutationTarget("wrong_kind_descriptor_mapping", "crates/nostr_automerge/src/checkpoint/reference_state.rs", "Self::WrongKind", "crates/nostr_automerge/src/checkpoint/reference_state.rs", "wrong_kind_descriptor_invalidates_dependent_chunk"),
    MutationTarget("prior_knowledge_charge", "crates/nostr_automerge/src/engine/reference_evaluator.rs", "fn additional_prior_knowledge", "crates/nostr_automerge/tests/public_engine_api.rs", "prior_knowledge_exhaustion_is_deterministic_at_every_item_boundary"),
    MutationTarget("pre_view_cancellation", "crates/nostr_automerge/src/engine/reference_evaluator.rs", "if cancellation.is_cancelled()", "crates/nostr_automerge/tests/public_engine_api.rs", "cancellation_before_control_evaluation_fabricates_no_state"),
    MutationTarget("finalization_remainder", "crates/nostr_automerge/src/engine/reference_evaluator.rs", "self.remaining != ReportFinalizationPlan::default()", "crates/nostr_automerge/src/engine/reference_evaluator.rs", "finalization_dimensions_reject_underflow_and_double_finish"),
    MutationTarget("report_validation_order", "crates/nostr_automerge/src/engine/reference_evaluator.rs", "report_validation_precedes_finalization_refund", "crates/nostr_automerge/src/engine/reference_evaluator.rs", "report_validation_precedes_finalization_refund"),
    MutationTarget("generic_critical_proof", "scripts/validate_requirement_matrix_v7.py", "generic-critical", "scripts/validate_requirement_matrix_v7.py", "generic_proof"),
)


def main() -> int:
    rows = []
    for target in TARGETS:
        source = (ROOT / target.source_path).read_text()
        test = (ROOT / target.test_path).read_text()
        if target.source_anchor not in source:
            raise AssertionError(f"stale mutation source anchor: {target.name}")
        if target.test_id not in test:
            raise AssertionError(f"stale mutation test anchor: {target.name}")
        rows.append({**asdict(target), "inventory_status": "validated"})
    report = {
        "schema": "nostr_automerge.mutation_inventory.v6",
        "status": "validated",
        "target_count": len(rows),
        "execution": "deferred_external_hold",
        "targets": rows,
    }
    OUTPUT.write_text(json.dumps(report, sort_keys=True, separators=(",", ":")) + "\n")
    print(f"PASS: validated {len(rows)} remediation-v6 mutation targets")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
