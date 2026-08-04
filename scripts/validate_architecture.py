#!/usr/bin/env python3
"""Enforce dependency containment and forbidden architecture edges."""

import pathlib

ROOT = pathlib.Path(__file__).resolve().parents[1]
SOURCE = ROOT / "crates/nostr_automerge/src"
ADAPTER = SOURCE / "automerge_adapter"
FORBIDDEN_DEPENDENCIES = ("tokio", "async-std", "reqwest", "sqlx", "rusqlite", "radroots_", "farm", "marmot", "tangle")


def main() -> None:
    violations = []
    for path in sorted(SOURCE.rglob("*.rs")):
        text = path.read_text(encoding="utf-8")
        if not path.is_relative_to(ADAPTER) and "automerge::" in text:
            violations.append(f"direct Automerge use: {path.relative_to(ROOT)}")
    manifest = (ROOT / "crates/nostr_automerge/Cargo.toml").read_text(encoding="utf-8").lower()
    for dependency in FORBIDDEN_DEPENDENCIES:
        if dependency in manifest:
            violations.append(f"forbidden dependency marker: {dependency}")
    if violations:
        raise SystemExit("FAIL: architecture boundary\n" + "\n".join(violations))
    print("PASS: architecture boundary")
    print("- automerge_adapter=exclusive")
    print("- forbidden_dependencies=absent")


if __name__ == "__main__":
    main()
