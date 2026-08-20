#!/usr/bin/env python3
"""Reproduce the five remediation-v7 source defects without a red default suite."""

from __future__ import annotations

import subprocess
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
CASES = (
    (
        "finding_059_noncanonical_control_requires_branch_table",
        "FINDING_059 reproduced: preliminary exclusion still implies stateful validity",
    ),
    (
        "finding_060_checkpoint_index_is_coordinate_qualified",
        "FINDING_060 reproduced: checkpoint chunks lack a coordinate-plus-descriptor index",
    ),
    (
        "finding_061_change_indexes_are_coordinate_qualified",
        "FINDING_061 reproduced: change discovery lacks coordinate-qualified indexes",
    ),
    (
        "finding_062_parent_propagation_is_linear_and_metered",
        "FINDING_062 reproduced: propagation remains repeated, unmetered, or uncancellable",
    ),
    (
        "finding_063_interrupted_settlement_is_explicit",
        "FINDING_063 reproduced: interrupted finalization still erases remainder as consumption",
    ),
)


def main() -> int:
    for test_name, diagnostic in CASES:
        result = subprocess.run(
            (
                "cargo",
                "test",
                "-p",
                "nostr_automerge",
                "--test",
                "remediation_v7_baseline",
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
    print(f"PASS: reproduced {len(CASES)} remediation-v7 findings")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
