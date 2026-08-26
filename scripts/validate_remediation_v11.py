#!/usr/bin/env python3
"""Validate the closed remediation v11 baseline and initial runtime cursor."""

from __future__ import annotations

import copy
import json
import pathlib
import subprocess

ROOT = pathlib.Path(__file__).resolve().parents[1]
AUTHORITY = ROOT / "spec/remediation_v11_authority.json"
LEDGER = ROOT / "implementation/runtime_ledger_v11.json"

PUBLIC_CANDIDATE = "e1b4f461c0d2a1e8cc8e520bed2dfa64a62270f2"
PUBLIC_TREE = "5e62938bfe576d6c67b3bfe355d5b5dd47585e87"
PRIVATE_CANDIDATE = "2d708bb0a7a00523ab5c244fd0a15c96afcf0a4a"
PRIOR_HANDOFF = "e333873b2b2e42b42bc7d9e652012195ab70760b586eb184462e655a5682be44"
HOLDS = (
    "external_assurance",
    "event_kind_allocation",
    "nip_submission",
    "production_qualification",
    "publication",
    "release",
    "remote_mutation",
)
SCOPE = (
    "docs/execution/remediation_v11/baseline.md",
    "implementation/runtime_ledger_v11.json",
    "reports/spec_baseline.txt",
    "scripts/validate_remediation_v11.py",
    "scripts/validate_runtime_ledger_v10.py",
    "scripts/validate_spec.py",
    "spec/remediation_v11_authority.json",
)


class ValidationError(RuntimeError):
    pass


def require_record(value: object, keys: tuple[str, ...], label: str) -> dict[str, object]:
    if not isinstance(value, dict) or tuple(value) != keys:
        raise ValidationError(f"{label}:keys")
    return value


def validate_authority(value: object) -> None:
    record = require_record(
        value,
        (
            "schema",
            "status",
            "reviewed_public",
            "opaque_private",
            "prior_handoff_sha256",
            "historical_sequence",
            "active_sequence",
            "counts",
            "holds",
            "result",
        ),
        "authority",
    )
    if record["schema"] != "nostr_automerge.remediation_v11_authority.v1":
        raise ValidationError("authority:schema")
    if record["status"] != "authority_and_reproduction_correction_required":
        raise ValidationError("authority:status")
    if record["result"] != "pass":
        raise ValidationError("authority:result")
    if record["reviewed_public"] != {
        "candidate": PUBLIC_CANDIDATE,
        "tree": PUBLIC_TREE,
    }:
        raise ValidationError("authority:public")
    if record["opaque_private"] != {
        "candidate": PRIVATE_CANDIDATE,
        "source_disclosure": False,
    }:
        raise ValidationError("authority:private")
    if record["prior_handoff_sha256"] != PRIOR_HANDOFF:
        raise ValidationError("authority:handoff")
    if record["historical_sequence"] != {
        "rcld_first": 95,
        "rcld_last": 99,
        "step_first": "step_1288",
        "step_last": "step_1307",
        "status": "immutable_historical_superseded_for_v11_scope",
    }:
        raise ValidationError("authority:history")
    if record["active_sequence"] != {
        "rcld_first": 100,
        "rcld_last": 108,
        "step_first": "step_1308",
        "step_last": "step_1363",
        "step_count": 56,
    }:
        raise ValidationError("authority:sequence")
    if record["counts"] != {
        "requirements_current": 148,
        "requirements_target": 152,
        "scenarios_current": 193,
        "scenarios_target": 198,
    }:
        raise ValidationError("authority:counts")
    if tuple(record["holds"]) != HOLDS:
        raise ValidationError("authority:holds")


def validate_ledger(value: object) -> None:
    record = require_record(
        value,
        (
            "schema",
            "status",
            "authority",
            "cursor",
            "findings",
            "requirements",
            "active_checkpoint_scope",
            "predecessors",
            "holds",
            "result",
        ),
        "ledger",
    )
    if record["schema"] != "nostr_automerge.runtime_ledger.v11.v1":
        raise ValidationError("ledger:schema")
    if record["status"] != "authority_and_reproduction_correction_required":
        raise ValidationError("ledger:status")
    if record["authority"] != "spec/remediation_v11_authority.json":
        raise ValidationError("ledger:authority")
    if record["cursor"] != {
        "active_rcld": 100,
        "active_step": "step_1308",
        "next_step": "step_1309",
        "last_planned_step": "step_1363",
        "remaining_checkpoint_count": 56,
        "remaining_rcld_count": 9,
    }:
        raise ValidationError("ledger:cursor")
    if record["findings"] != {
        "open": ["FINDING_096", "FINDING_097", "FINDING_098", "FINDING_099"],
        "held": ["FINDING_080"],
    }:
        raise ValidationError("ledger:findings")
    if tuple(record["requirements"]) != (
        "NCRDT-RESOURCE-015",
        "NCRDT-RESOURCE-016",
        "NCRDT-VERSION-003",
        "NCRDT-OWNERSHIP-001",
    ):
        raise ValidationError("ledger:requirements")
    if tuple(record["active_checkpoint_scope"]) != SCOPE:
        raise ValidationError("ledger:scope")
    if record["predecessors"] != []:
        raise ValidationError("ledger:predecessors")
    if tuple(record["holds"]) != HOLDS or record["result"] != "pass":
        raise ValidationError("ledger:result")


def validate_repository() -> None:
    validate_authority(json.loads(AUTHORITY.read_text()))
    validate_ledger(json.loads(LEDGER.read_text()))
    tree = subprocess.run(
        ["git", "rev-parse", f"{PUBLIC_CANDIDATE}^{{tree}}"],
        cwd=ROOT,
        check=True,
        capture_output=True,
        text=True,
    ).stdout.strip()
    if tree != PUBLIC_TREE:
        raise ValidationError("repository:tree")
    status = subprocess.run(
        ["git", "status", "--porcelain=v1", "-z", "--untracked-files=all"],
        cwd=ROOT,
        check=True,
        capture_output=True,
    ).stdout.decode().split("\0")
    paths = tuple(sorted(entry[3:] for entry in status if entry))
    if len(paths) != len(set(paths)) or not set(paths).issubset(SCOPE):
        raise ValidationError(f"repository:scope:{paths}")


def mutation_self_test() -> int:
    authority = json.loads(AUTHORITY.read_text())
    ledger = json.loads(LEDGER.read_text())
    mutations: list[tuple[str, object]] = []
    for mutate in (
        lambda value: value["reviewed_public"].update(candidate="0" * 40),
        lambda value: value["reviewed_public"].update(tree="0" * 40),
        lambda value: value["opaque_private"].update(source_disclosure=True),
        lambda value: value.update(prior_handoff_sha256="0" * 64),
        lambda value: value["historical_sequence"].update(status="current"),
        lambda value: value["active_sequence"].update(step_count=55),
        lambda value: value["counts"].update(scenarios_target=199),
        lambda value: value["holds"].pop(),
        lambda value: value.update(extra=False),
    ):
        candidate = copy.deepcopy(authority)
        mutate(candidate)
        mutations.append(("authority", candidate))
    for mutate in (
        lambda value: value["cursor"].update(active_step="step_1309"),
        lambda value: value["cursor"].update(remaining_checkpoint_count=55),
        lambda value: value["findings"]["open"].reverse(),
        lambda value: value["requirements"].pop(),
        lambda value: value["active_checkpoint_scope"].reverse(),
        lambda value: value["predecessors"].append({"step": "step_1308"}),
        lambda value: value["holds"].pop(),
        lambda value: value.update(extra=False),
    ):
        candidate = copy.deepcopy(ledger)
        mutate(candidate)
        mutations.append(("ledger", candidate))
    for kind, candidate in mutations:
        try:
            if kind == "authority":
                validate_authority(candidate)
            else:
                validate_ledger(candidate)
        except ValidationError:
            continue
        raise ValidationError(f"mutation:{kind}")
    return len(mutations)


def main() -> None:
    validate_repository()
    mutations = mutation_self_test()
    print("PASS: remediation v11 baseline")
    print(f"- mutations={mutations}")
    print("- steps=56")
    print("- rclds=9")


if __name__ == "__main__":
    main()
