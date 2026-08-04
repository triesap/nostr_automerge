#!/usr/bin/env python3
"""Validate required repository-agent policy."""

from __future__ import annotations

import subprocess
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
AGENTS_PATH = ROOT / "AGENTS.md"
REQUIRED_HEADINGS = (
    "## Source Of Truth",
    "## Work Discipline",
    "## Naming",
    "## Architecture",
    "## Safety And Quality",
    "## Verification",
    "## Completion Report",
)
REQUIRED_PHRASES = (
    "triesap/nostr_automerge",
    "nostr_automerge_conformance",
    "nostr_automerge_xtask",
    "nostr-crdt/automerge/actor/v1",
    "one active checkpoint",
    "Record a deviation",
    "Do not push, publish, release, tag, deploy",
    "Next-step safety",
    "GitHub-hosted workflows are prohibited",
    ".act/workflows/**",
)


def workflow_violations(tracked_files: list[str], workflow_ignored: bool) -> list[str]:
    """Return stable violations of the local-only workflow policy."""

    violations = []
    for relative in tracked_files:
        if relative.startswith(".github/workflows/"):
            violations.append(f"tracked GitHub workflow: {relative}")
        if relative.startswith(".act/workflows/"):
            violations.append(f"tracked local Act workflow: {relative}")
    if not workflow_ignored:
        violations.append(".act/workflows is not ignored")
    return violations


def git_lines(*args: str) -> list[str]:
    """Run a read-only Git query and return non-empty output lines."""

    result = subprocess.run(
        ["git", *args], cwd=ROOT, check=True, capture_output=True, text=True
    )
    return [line for line in result.stdout.splitlines() if line]


def main() -> int:
    """Validate the policy file and print deterministic results."""

    text = AGENTS_PATH.read_text(encoding="utf-8")
    for heading in REQUIRED_HEADINGS:
        if heading not in text:
            raise AssertionError(f"AGENTS.md missing heading: {heading}")
    for phrase in REQUIRED_PHRASES:
        if phrase not in text:
            raise AssertionError(f"AGENTS.md missing policy: {phrase}")

    ignored = subprocess.run(
        ["git", "check-ignore", "-q", ".act/workflows/policy_probe.yml"],
        cwd=ROOT,
        check=False,
    ).returncode == 0
    actual_violations = workflow_violations(git_lines("ls-files"), ignored)
    if actual_violations:
        raise AssertionError(actual_violations[0])

    negative_cases = (
        ([".github/workflows/ci.yml"], True, "tracked GitHub workflow"),
        ([".act/workflows/ci.yml"], True, "tracked local Act workflow"),
        ([], False, ".act/workflows is not ignored"),
    )
    for tracked, is_ignored, expected in negative_cases:
        violations = workflow_violations(tracked, is_ignored)
        if not violations or expected not in violations[0]:
            raise AssertionError(f"workflow policy negative case failed: {expected}")

    print("PASS: repository agent policy")
    print(f"- required_headings={len(REQUIRED_HEADINGS)}")
    print(f"- required_policies={len(REQUIRED_PHRASES)}")
    print("- repository_identity=pass")
    print("- normative_actor_domain=pass")
    print(f"- workflow_negative_cases={len(negative_cases)}")
    print("- local_act_policy=pass")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
