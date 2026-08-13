#!/usr/bin/env python3
"""Validate repository governance, ownership, and license policy."""

from __future__ import annotations

from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]


def require(path: str, phrases: tuple[str, ...]) -> None:
    """Require *path* and each policy phrase."""

    target = ROOT / path
    if not target.is_file():
        raise AssertionError(f"missing governance file: {path}")
    text = target.read_text(encoding="utf-8")
    for phrase in phrases:
        if phrase not in text:
            raise AssertionError(f"{path} missing policy: {phrase}")


def main() -> int:
    """Run deterministic governance validation."""

    require(
        "README.md",
        (
            "## Status",
            "remediation-v5 implementation",
            "publication is not authorized",
            "does not currently claim",
            "## Architecture Boundary",
            "spec/NIP_DRAFT.md",
            "SECURITY.md",
            "LICENSE-MIT",
            "LICENSE-APACHE",
        ),
    )
    require(
        "CONTRIBUTING.md",
        (
            "## Development setup",
            "## Pull request checklist",
            "## Protocol Changes",
            "## Security",
            "Record deviations",
        ),
    )
    require(
        "SECURITY.md",
        (
            "## Supported Versions",
            "## Reporting A Vulnerability",
            "## Security Scope",
            "## Disclosure And Fixes",
        ),
    )
    require("CODEOWNERS", ("* @triesap",))
    require("LICENSE-MIT", ("MIT License", "Copyright (c) 2026 Tyson Lupul"))
    require(
        "LICENSE-APACHE",
        ("Apache License", "Version 2.0, January 2004", "Copyright 2026 Tyson Lupul"),
    )

    contributing = (ROOT / "CONTRIBUTING.md").read_text(encoding="utf-8")
    for stale in ("WAI-ARIA", "keyboard coverage", "All components"):
        if stale in contributing:
            raise AssertionError(f"CONTRIBUTING.md retains unrelated policy: {stale}")

    print("PASS: repository governance")
    print("- governance_files=6")
    print("- ownership=pass")
    print("- dual_license=pass")
    print("- draft_claims=pass")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
