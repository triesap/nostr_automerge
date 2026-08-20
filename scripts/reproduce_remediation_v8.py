#!/usr/bin/env python3
"""Reproduce remediation-v8 defects without making the default suite red."""

from __future__ import annotations

import argparse
import subprocess
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
CASES = (
    (
        "finding_066_branch_results_reach_final_claim_reduction",
        "FINDING_066 reproduced: final reduction cannot query branch-local change outcomes",
    ),
    (
        "finding_067_control_work_is_coordinate_scoped",
        "FINDING_067 reproduced: target control work lacks coordinate-scoped parent edges",
    ),
    (
        "finding_068_interrupted_report_work_is_settled_by_pass",
        "FINDING_068 reproduced: interrupted report work occurs after coarse settlement",
    ),
    (
        "finding_069_change_carriers_have_event_dispositions",
        "FINDING_069 reproduced: change carriers have no dynamic Event dispositions",
    ),
    (
        "finding_070_local_nip_contains_reconciled_branch_rules",
        "FINDING_070 reproduced: the local NIP lacks reconciled branch-local outcomes",
    ),
    (
        "finding_071_distribution_contains_180_scenarios",
        "FINDING_071 reproduced: signed distribution does not contain 180 scenarios",
    ),
)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--expect-baseline-fail", action="store_true", required=True)
    return parser.parse_args()


def main() -> int:
    parse_args()
    for test_name, diagnostic in CASES:
        result = subprocess.run(
            (
                "cargo",
                "extbuild",
                "run",
                "--",
                "cargo",
                "test",
                "-p",
                "nostr_automerge",
                "--test",
                "remediation_v8_reproductions",
                "--locked",
                "--",
                "--ignored",
                "--exact",
                test_name,
            ),
            cwd=ROOT,
            check=False,
            capture_output=True,
            text=True,
        )
        output = result.stdout + result.stderr
        if result.returncode == 0:
            raise AssertionError(f"reviewed defect no longer reproduces: {test_name}")
        if diagnostic not in output:
            raise AssertionError(f"unexpected failure for {test_name}:\n{output}")
        print(f"PASS: reproduced {test_name}")
    print(f"PASS: reproduced {len(CASES)} remediation-v8 findings")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
