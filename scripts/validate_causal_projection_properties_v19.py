#!/usr/bin/env python3
"""Run behavior-derived v19 causal-projection mutation properties."""

from __future__ import annotations

import argparse
import re
import subprocess
import sys
from pathlib import Path

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parents[1]
PROPERTY_TESTS = {
    "TYPED_BUDGET_EXHAUSTED_IDENTITY": "v19_runtime_helper_mutation_oracle",
    "TYPED_CANCELLED_IDENTITY": "v19_runtime_helper_mutation_oracle",
    "UNEXPECTED_WORK_ERROR_IDENTITY": "v19_runtime_helper_mutation_oracle",
    "CHARGE_AFTER_OPERATION": "v19_runtime_helper_mutation_oracle",
    "OPERATION_OBSERVATION_BEFORE_TARGET": "v19_runtime_helper_mutation_oracle",
    "TARGET_AFTER_STOP": "v19_runtime_helper_mutation_oracle",
    "OBSERVATION_AFTER_STOP": "v19_runtime_helper_mutation_oracle",
    "TARGET_EXECUTION_COUNT_MISMATCH": "v19_runtime_helper_mutation_oracle",
    "SITE_ID_MISMATCH": "v19_runtime_helper_mutation_oracle",
    "COUNTER_MISMATCH": "v19_runtime_helper_mutation_oracle",
    "SITE_TARGET_BEFORE_CHARGE": "v19_runtime_direct_target_oracle",
    "PUBLICATION_AFTER_STOP": "v19_runtime_publication_after_stop_oracle",
    "ALTERNATE_CONSUMER_BYPASS": "v19_runtime_alternate_consumer_oracle",
}
PREFIX = "v19-mutation-property="


class PropertyError(RuntimeError):
    pass


def require(condition: bool, code: str) -> None:
    if not condition:
        raise PropertyError(code)


def property_from_output(output: str) -> str:
    matches = re.findall(rf"^{PREFIX}([A-Z0-9_]+)$", output, re.MULTILINE)
    require(len(matches) == 1, "RUNTIME_PROPERTY_OUTPUT")
    require(matches[0] in PROPERTY_TESTS, "RUNTIME_PROPERTY_UNKNOWN")
    return matches[0]


def public_transcript(output: str) -> str:
    return re.sub(r"/(?:Users|Volumes)/[^\s)]+", "<local-path>", output)


def command(property_code: str) -> list[str]:
    test = PROPERTY_TESTS[property_code]
    return [
        "cargo", "extbuild", "run", "--", "cargo", "test", "-p",
        "nostr_automerge", "--lib",
        f"graph::actor_state::tests::{test}", "--locked", "--", "--exact",
        "--nocapture",
    ]


def execute(root: Path, property_code: str, expect_failure: bool) -> None:
    completed = subprocess.run(
        command(property_code), cwd=root, capture_output=True, text=True, check=False
    )
    output = completed.stdout + completed.stderr
    sys.stdout.write(public_transcript(completed.stdout))
    sys.stderr.write(public_transcript(completed.stderr))
    if expect_failure:
        require(completed.returncode != 0, "RUNTIME_MUTANT_SURVIVED")
        actual = property_from_output(output)
        require(actual == property_code, f"WRONG_RUNTIME_PROPERTY:{actual}")
        print(f"FAIL: {actual}")
        raise SystemExit(1)
    require(completed.returncode == 0, "RUNTIME_BASELINE_FAILED")
    require(PREFIX not in output, "RUNTIME_BASELINE_REPORTED_MUTATION")
    print(f"PASS: causal projection runtime property v19 property={property_code}")


def self_test() -> None:
    for code in PROPERTY_TESTS:
        require(property_from_output(f"noise\n{PREFIX}{code}\n") == code, "SELF_PARSE")
    attacks = [
        "",
        f"{PREFIX}UNKNOWN",
        f"{PREFIX}TARGET_AFTER_STOP\n{PREFIX}TARGET_AFTER_STOP",
        f"prefix-{PREFIX}TARGET_AFTER_STOP",
    ]
    caught = 0
    for output in attacks:
        try:
            property_from_output(output)
        except PropertyError:
            caught += 1
            continue
        raise PropertyError("SELF_ATTACK_SURVIVED")
    require(len(set(PROPERTY_TESTS.values())) == 4, "ORACLE_TEST_SET")
    print(f"PASS: causal projection runtime property v19 codes={len(PROPERTY_TESTS)} attacks={caught}")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--root", type=Path, default=ROOT)
    parser.add_argument("--property", choices=tuple(PROPERTY_TESTS))
    parser.add_argument("--expect-failure", action="store_true")
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args()
    if args.self_test:
        self_test()
        return 0
    require(args.property is not None, "PROPERTY_REQUIRED")
    execute(args.root.resolve(), args.property, args.expect_failure)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
