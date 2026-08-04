#!/usr/bin/env python3
"""Validate the sealed draft protocol revision."""

from __future__ import annotations

import copy
import json
from pathlib import Path
from typing import Any


ROOT = Path(__file__).resolve().parents[1]
EXPECTED_KINDS = {
    "change": 1624,
    "control": 1625,
    "checkpoint_descriptor": 1626,
    "checkpoint_chunk": 1627,
    "manifest": 31624,
}
EXPECTED_TOP_LEVEL = {
    "schema", "project", "revision", "status", "sealed", "nip", "repository",
    "cargo_package", "rust_crate", "format", "text_encoding", "actor_domain",
    "kinds", "checkpoints", "limits_status", "conformance",
}


class RevisionError(Exception):
    """A stable protocol-revision validation error."""


def load_json(path: Path) -> dict[str, Any]:
    """Load a JSON object."""

    value = json.loads(path.read_text(encoding="utf-8"))
    if not isinstance(value, dict):
        raise RevisionError("revision_not_object")
    return value


def validate(value: dict[str, Any]) -> None:
    """Validate the complete sealed revision."""

    if set(value) != EXPECTED_TOP_LEVEL:
        raise RevisionError("invalid_revision_fields")
    constants = {
        "schema": "nostr_automerge.protocol_revision.v1",
        "project": "nostr_automerge_v1_spec",
        "revision": "draft_2026_08",
        "status": "approved_implementation_baseline",
        "sealed": True,
        "repository": "triesap/nostr_automerge",
        "cargo_package": "nostr_automerge",
        "rust_crate": "nostr_automerge",
        "format": "automerge-change-v1",
        "text_encoding": "utf16",
        "actor_domain": "nostr-crdt/automerge/actor/v1",
        "limits_status": "normative_for_draft_provisional_for_production",
    }
    for field, expected in constants.items():
        if value.get(field) != expected:
            raise RevisionError(f"invalid_{field}")
    if value.get("kinds") != EXPECTED_KINDS:
        raise RevisionError("invalid_kinds")
    if value.get("checkpoints") != {
        "status": "later_milestone",
        "v1_provenance": "verified_history_only",
        "missing_history_recovery": "deferred",
    }:
        raise RevisionError("invalid_checkpoints")
    if value.get("conformance") != {
        "normative_save_bytes_digest": False,
        "history_digest": True,
        "dispositions_digest": True,
        "typed_state_assertions": True,
    }:
        raise RevisionError("invalid_conformance")


def expect_failure(value: dict[str, Any], code: str) -> None:
    """Require *value* to fail with *code*."""

    try:
        validate(value)
    except RevisionError as error:
        if str(error) != code:
            raise AssertionError(f"expected {code}, received {error}") from error
    else:
        raise AssertionError(f"invalid revision unexpectedly passed: {code}")


def main() -> int:
    """Validate the canonical revision and required negative cases."""

    schema = load_json(ROOT / "spec/protocol_revision.schema.json")
    if schema.get("$schema") != "https://json-schema.org/draft/2020-12/schema":
        raise AssertionError("protocol revision schema must use JSON Schema 2020-12")

    revision = load_json(ROOT / "spec/protocol_revision.json")
    validate(revision)

    custom_kind = copy.deepcopy(revision)
    custom_kind["kinds"]["change"] = 9999
    expect_failure(custom_kind, "invalid_kinds")
    missing_kind = copy.deepcopy(revision)
    del missing_kind["kinds"]["control"]
    expect_failure(missing_kind, "invalid_kinds")
    changed_domain = copy.deepcopy(revision)
    changed_domain["actor_domain"] = "nostr_crdt/automerge/actor/v1"
    expect_failure(changed_domain, "invalid_actor_domain")
    final_status = copy.deepcopy(revision)
    final_status["status"] = "final"
    expect_failure(final_status, "invalid_status")

    print("PASS: sealed protocol revision")
    print(f"- kinds={len(EXPECTED_KINDS)}")
    print("- negative_cases=4")
    print("- repository_identity=pass")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
