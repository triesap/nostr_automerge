#!/usr/bin/env python3
"""Validate the active resource follow-up runtime cursor and scope."""

from __future__ import annotations

import copy
import hashlib
import json
import pathlib
import subprocess

ROOT = pathlib.Path(__file__).resolve().parents[1]
LEDGER = ROOT / "implementation/runtime_ledger_v10.json"
SCHEMA = ROOT / "tools/validation/runtime_ledger_v10.schema.json"

LEDGER_SHA256 = "72353098c2b63fd832661214b4afe3d33e32614ce9e31e7cd352ca57c308e57b"
SCHEMA_SHA256 = "8ae43e4f3aa8d00ea27ebb64a2f0d741b54cd9b10c398900c16ba9cd94ed0814"
AUTHORITY_SHA256 = "0cac9bf4b90c55e428c335797a9d7195bc3ee08eed5bfb49fca4428e62702531"
EXPECTED_SCOPE = (
    "crates/nostr_automerge/src/control/epoch_state.rs",
    "crates/nostr_automerge/src/control/parent_view.rs",
    "crates/nostr_automerge/src/graph/actor_state.rs",
    "crates/nostr_automerge/src/graph/change_candidate.rs",
    "crates/nostr_automerge/src/graph/closure.rs",
    "crates/nostr_automerge/src/graph/dependency_graph.rs",
    "crates/nostr_automerge/src/graph/equivocation.rs",
    "crates/nostr_automerge/src/graph/scaling.rs",
    "crates/nostr_automerge/src/graph/schedule.rs",
    "crates/nostr_automerge/src/reference/epoch.rs",
    "crates/nostr_automerge/src/reference/epoch_engine.rs",
    "crates/nostr_automerge/src/reference/evaluate.rs",
    "docs/execution/remediation_v10/ledger.md",
    "implementation/runtime_ledger_v10.json",
    "reports/spec_baseline.txt",
    "scripts/validate_runtime_ledger_v10.py",
)
TOP_KEYS = (
    "schema", "status", "authority", "historical_predecessor", "cursor",
    "findings", "requirements", "active_checkpoint_scope", "predecessors",
    "holds", "result",
)


class LedgerError(RuntimeError):
    pass


def sha256(path: pathlib.Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def validate_record(record: object) -> None:
    if not isinstance(record, dict) or tuple(record) != TOP_KEYS:
        raise LedgerError("ledger:keys")
    if record["schema"] != "nostr_automerge.runtime_ledger.v10.v1":
        raise LedgerError("ledger:schema")
    if record["status"] != "resource_accounting_remediation_required" or record["result"] != "pass":
        raise LedgerError("ledger:status")
    authority = record["authority"]
    if authority != {"path": "spec/resource_followup_authority_v10.json", "sha256": AUTHORITY_SHA256}:
        raise LedgerError("ledger:authority")
    historical = record["historical_predecessor"]
    if historical != {
        "candidate": "bfad500706a834bd41ef4392613090d2381bd08b",
        "tree": "98630a87313f524b8efbe8182e19b9b897986e6e",
        "last_completed_rcld": 94,
        "last_completed_step": "step_1287",
        "final_decision_sha256": "a32fcbec532f8513c811a8b6b1eeac65d9ad64043210fe270e81f0387bc0302a",
    }:
        raise LedgerError("ledger:historical")
    cursor = record["cursor"]
    if cursor != {
        "active_rcld": 96,
        "active_step": "step_1291",
        "next_step": "step_1292",
        "last_planned_step": "step_1307",
        "remaining_checkpoint_count": 17,
        "remaining_rcld_count": 4,
    }:
        raise LedgerError("ledger:cursor")
    if record["findings"] != {"open": ["FINDING_094", "FINDING_095"], "closed": [], "held": ["FINDING_080"]}:
        raise LedgerError("ledger:findings")
    if tuple(record["requirements"]) != (
        "NCRDT-RESOURCE-001", "NCRDT-RESOURCE-013", "NCRDT-RESOURCE-014",
        "NCRDT-COMPLETION-001", "NCRDT-CONF-010", "NCRDT-EVIDENCE-006",
    ):
        raise LedgerError("ledger:requirements")
    if tuple(record["active_checkpoint_scope"]) != EXPECTED_SCOPE:
        raise LedgerError("ledger:scope")
    if record["predecessors"] != [
        {
            "step": "step_1288",
            "candidate": "53208563e7aa28bc00162ab3b5802824675df6d8",
            "owner_class": "public",
            "result": "pass",
        },
        {
            "step": "step_1289",
            "candidate": "3991ca4933318581cdde23680b9e03758f92b5df",
            "owner_class": "public",
            "result": "pass",
        },
        {
            "step": "step_1290",
            "candidate": "19420942f7814051ae458fb05f49050244394271",
            "owner_class": "opaque_private",
            "result": "pass",
        },
    ]:
        raise LedgerError("ledger:predecessors")
    if tuple(record["holds"]) != (
        "external_assurance", "nip_submission", "production_qualification",
        "publication", "release", "remote_mutation",
    ):
        raise LedgerError("ledger:holds")


def validate_worktree_scope() -> None:
    output = subprocess.run(
        ["git", "status", "--porcelain=v1", "-z", "--untracked-files=all"], cwd=ROOT,
        capture_output=True, check=True,
    ).stdout
    if not output:
        return
    entries = [entry for entry in output.decode().split("\0") if entry]
    paths = []
    for entry in entries:
        status, path = entry[:2], entry[3:]
        if status not in {" M", "M ", "MM", "A ", "AM", "??"} or " -> " in path:
            raise LedgerError(f"worktree:status:{status}:{path}")
        paths.append(path)
    if len(paths) != len(set(paths)) or not set(paths).issubset(EXPECTED_SCOPE):
        raise LedgerError("worktree:scope")


def mutation_self_test() -> int:
    original = json.loads(LEDGER.read_text())
    mutations = []
    for mutate in (
        lambda value: value.update(status="code_complete_publication_held"),
        lambda value: value["authority"].update(sha256="0" * 64),
        lambda value: value["historical_predecessor"].update(candidate="0" * 40),
        lambda value: value["cursor"].update(active_step="step_1292"),
        lambda value: value["cursor"].update(remaining_checkpoint_count=18),
        lambda value: value["findings"]["open"].reverse(),
        lambda value: value["active_checkpoint_scope"].pop(),
        lambda value: value["active_checkpoint_scope"].append("foreign"),
        lambda value: value["holds"].pop(),
        lambda value: value.update(extra=False),
    ):
        candidate = copy.deepcopy(original)
        mutate(candidate)
        mutations.append(candidate)
    for index, mutation in enumerate(mutations):
        try:
            validate_record(mutation)
        except LedgerError:
            continue
        raise LedgerError(f"mutation:{index}")
    return len(mutations)


def main() -> None:
    if sha256(LEDGER) != LEDGER_SHA256:
        raise LedgerError("ledger:sha256")
    if sha256(SCHEMA) != SCHEMA_SHA256:
        raise LedgerError("schema:sha256")
    if sha256(ROOT / "spec/resource_followup_authority_v10.json") != AUTHORITY_SHA256:
        raise LedgerError("authority:sha256")
    validate_record(json.loads(LEDGER.read_text()))
    validate_worktree_scope()
    mutations = mutation_self_test()
    print("PASS: runtime ledger v10")
    print("- active=step_1291")
    print(f"- mutations={mutations}")


if __name__ == "__main__":
    main()
