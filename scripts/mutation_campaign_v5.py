#!/usr/bin/env python3
"""Execute the bounded remediation-v5 deterministic source mutation campaign."""

from __future__ import annotations

import json
import subprocess
from dataclasses import dataclass
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
OUTPUT = ROOT / "reports" / "mutation_campaign_v5.json"


@dataclass(frozen=True)
class Mutation:
    name: str
    path: str
    search: str
    replacement: str
    command: tuple[str, ...]


MUTATIONS = (
    Mutation(
        "other_control_dependency_becomes_unknown",
        "crates/nostr_automerge/src/reference/epoch_engine.rs",
        "                | Self::KnownOtherControl\n",
        "",
        ("cargo", "test", "-p", "nostr_automerge", "--lib", "only_known_impossible_dependency_states_invalidate", "--locked"),
    ),
    Mutation(
        "pending_claim_loses_precedence",
        "crates/nostr_automerge/src/engine/reference_evaluator.rs",
        "    } else if claims.contains(&ChangeClaimReason::Pending) {",
        "    } else if false && claims.contains(&ChangeClaimReason::Pending) {",
        ("cargo", "test", "-p", "nostr_automerge", "--lib", "reasoned_change_outcome_uses_final_precedence", "--locked"),
    ),
    Mutation(
        "missing_checkpoint_control_becomes_invalid",
        "crates/nostr_automerge/src/checkpoint/authorize.rs",
        "            ReferencedControlState::Pending(_) | ReferencedControlState::Missing => {",
        "            ReferencedControlState::Pending(_) => {",
        ("cargo", "test", "-p", "nostr_automerge", "--lib", "checkpoint_descriptor_authorization_is_causal_and_role_bound", "--locked"),
    ),
    Mutation(
        "reportable_coordinate_index_bypassed",
        "crates/nostr_automerge/src/evidence/document_view.rs",
        "            reportable_event_ids,\n",
        "            reportable_event_ids: BTreeSet::new(),\n",
        ("cargo", "test", "-p", "nostr_automerge", "--test", "public_engine_api", "signed_events_reach_materialized_state_through_public_engine", "--locked"),
    ),
    Mutation(
        "finalization_underflow_not_rejected",
        "crates/nostr_automerge/src/engine/reference_evaluator.rs",
        "            .checked_sub(amount)\n",
        "            .checked_add(amount)\n",
        ("cargo", "test", "-p", "nostr_automerge", "--lib", "finalization_dimensions_reject_underflow_and_double_finish", "--locked"),
    ),
    Mutation(
        "pre_view_cancellation_bypassed",
        "crates/nostr_automerge/src/engine/reference_evaluator.rs",
        "        if cancellation.is_cancelled() {\n            return compact_interrupted_report",
        "        if false && cancellation.is_cancelled() {\n            return compact_interrupted_report",
        ("cargo", "test", "-p", "nostr_automerge", "--test", "public_engine_api", "every_v3_work_counter_boundary", "--locked"),
    ),
)


def main() -> int:
    results: list[dict[str, str]] = []
    for mutation in MUTATIONS:
        path = ROOT / mutation.path
        original = path.read_text(encoding="utf-8")
        if original.count(mutation.search) != 1:
            raise AssertionError(f"stale mutation anchor: {mutation.name}")
        try:
            path.write_text(original.replace(mutation.search, mutation.replacement, 1), encoding="utf-8")
            completed = subprocess.run(mutation.command, cwd=ROOT, capture_output=True, check=False, text=True)
        finally:
            path.write_text(original, encoding="utf-8")
        if completed.returncode == 0:
            raise AssertionError(f"source mutation survived: {mutation.name}")
        results.append({"mutation": mutation.name, "result": "caught"})
    report = {
        "schema": "nostr_automerge.mutation_campaign.v5",
        "generated": len(results),
        "caught": len(results),
        "survived": 0,
        "status": "pass",
        "mutations": results,
    }
    OUTPUT.write_text(json.dumps(report, sort_keys=True, separators=(",", ":")) + "\n", encoding="utf-8")
    print(f"PASS: all {len(results)} remediation-v5 deterministic mutations were caught")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
