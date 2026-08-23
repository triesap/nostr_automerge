#!/usr/bin/env python3
"""Validate the exact four authorized checkpoint expectation corrections."""

from __future__ import annotations

import validate_authority_transition_v10 as authority


def main() -> int:
    state = authority.load_object(authority.STATE_PATH)
    stage = state.get("current_stage")
    authority.require(
        isinstance(stage, str)
        and authority.STAGES.index(stage)
        >= authority.STAGES.index("distribution_locked"),
        "correction_stage",
    )
    authority.validate_state(state)
    authority.require(
        tuple(
            row[0]
            for row in authority.CORRECTION_BINDINGS
        )
        == authority.CORRECTED_REPORTS,
        "correction_order",
    )
    fixture_mutations = authority.fixture_correction_mutation_self_test(stage)
    authority_mutations = authority.correction_authority_mutation_self_test(state)
    print("PASS: exact four corrected checkpoint expectations")
    print(f"- corrected_fixture_count={len(authority.CORRECTION_BINDINGS)}")
    print(f"- fixture_negative_mutations={fixture_mutations}")
    print(f"- authority_negative_mutations={authority_mutations}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
