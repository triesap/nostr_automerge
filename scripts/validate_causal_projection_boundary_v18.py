#!/usr/bin/env python3
"""Validate the v18 sealed operation boundary structurally or by identity."""

from __future__ import annotations

import argparse
import hashlib
import re
from pathlib import Path

DEFAULT_ROOT = Path(__file__).resolve().parents[1]
SOURCE_PATH = Path("crates/nostr_automerge/src/graph/actor_state.rs")
HELPERS = {
    "perform_actor_decision_operation": ("ActorDecisionDescriptor", "perform"),
    "perform_causal_next_operation": ("CausalNextDescriptor", "perform"),
    "perform_projection_build_operation": ("ProjectionBuildDescriptor", "perform"),
    "metered_frontier_operation": ("FrontierComparisonDescriptor", "target"),
}


class BoundaryError(RuntimeError):
    def __init__(self, code: str):
        super().__init__(code)
        self.code = code


def require(condition: bool, code: str) -> None:
    if not condition:
        raise BoundaryError(code)


def function_text(source: str, name: str) -> str:
    marker = f"fn {name}"
    start = source.find(marker)
    require(start >= 0, "HELPER_MISSING:" + name)
    opening = source.find("{", start)
    require(opening >= 0, "HELPER_BODY_MISSING:" + name)
    depth = 0
    for index in range(opening, len(source)):
        if source[index] == "{":
            depth += 1
        elif source[index] == "}":
            depth -= 1
            if depth == 0:
                return source[start:index + 1]
    raise BoundaryError("HELPER_BODY_UNCLOSED:" + name)


def structural(source: str) -> None:
    require("ChargeAttempt" not in source, "PRECHARGE_OPERATION_OBSERVER")
    require(source.count('applicability: "required"') == 4, "DESCRIPTOR_APPLICABILITY")
    require(source.count('phase: "projection_construction"') == 1, "DESCRIPTOR_PHASE")
    require('phase: "construction"' not in source and 'applicability: "public_rust"' not in source, "DESCRIPTOR_LEGACY_VOCABULARY")

    for name, (descriptor_type, target_name) in HELPERS.items():
        body = function_text(source, name)
        compact = re.sub(r"\s+", " ", body)
        require(f"FnMut({descriptor_type}) -> Result<(), E>" in compact, "CHARGE_DESCRIPTOR_MISMATCH:" + name)
        descriptor = body.find("let descriptor = site.descriptor();")
        charge = body.find("charge(descriptor).map_err")
        target = body.find(f"let result = {target_name}();")
        observed = body.find("observed(")
        returned = body.find("Ok(result)")
        require(min(descriptor, charge, target, observed, returned) >= 0, "SEALED_STEP_MISSING:" + name)
        require(descriptor < charge < target < observed < returned, "SEALED_ORDER:" + name)
        require(body.count("charge(descriptor).map_err") == 1, "CHARGE_COUNT:" + name)
        require(body.count(f"let result = {target_name}();") == 1, "TARGET_COUNT:" + name)
        require(body.count("observed(") == 1, "COMPLETION_OBSERVER_COUNT:" + name)


def self_test(source: str) -> int:
    actor = function_text(source, "perform_actor_decision_operation")
    cases = [
        actor.replace(
            "charge(descriptor).map_err(MeteredActorStateError::Work)?;\n    let result = perform();",
            "let result = perform();\n    charge(descriptor).map_err(MeteredActorStateError::Work)?;",
            1,
        ),
        actor.replace(
            "let result = perform();\n    observed(",
            "observed(ActorDecisionObservation { descriptor, kind: ActorDecisionObservationKind::TargetCompleted });\n    let result = perform();\n    observed(",
            1,
        ),
    ]
    caught = 0
    for replacement in cases:
        mutated = source.replace(actor, replacement, 1)
        try:
            structural(mutated)
        except BoundaryError:
            caught += 1
            continue
        raise BoundaryError("MUTATION_SURVIVED")
    neutral = source.replace(
        "fn perform_actor_decision_operation<T, E>(",
        "// neutral comment before sealed helper\nfn perform_actor_decision_operation<T, E>(",
        1,
    )
    structural(neutral)
    return caught


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--root", type=Path, default=DEFAULT_ROOT)
    parser.add_argument("--mode", choices=("structural", "identity"), default="structural")
    parser.add_argument("--expected-source-sha256")
    args = parser.parse_args()
    root = args.root.resolve()
    source_path = root / SOURCE_PATH
    require(source_path.is_file(), "SOURCE_MISSING")
    data = source_path.read_bytes()
    source = data.decode()
    if args.mode == "identity":
        require(args.expected_source_sha256 is not None, "IDENTITY_EXPECTED_REQUIRED")
        require(hashlib.sha256(data).hexdigest() == args.expected_source_sha256, "SOURCE_IDENTITY_MISMATCH")
        print("PASS: causal projection boundary v18 identity=exact")
        return 0
    structural(source)
    print(f"PASS: causal projection boundary v18 helpers={len(HELPERS)} mutations={self_test(source)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
