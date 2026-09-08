#!/usr/bin/env python3
"""Validate append-only v19 authority, findings, frozen state, and cursor."""

from __future__ import annotations

import copy
import hashlib
import json
import subprocess
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[1]
BASE = "4cc1bb5060b7477214eb22cf7b086735e4a70b7a"
TREE = "7f26d0307e5b59adfbd6ba0afeb07644f6907792"
ACTOR_SHA = "d4c152536383ea95bb10f26f002d65bb270f1415cf9fe25efe098b31d2a05573"
FINDINGS = ["FINDING_130", "FINDING_131", "FINDING_132", "FINDING_133"]
HOLDS = ["external_assurance", "event_kind_allocation", "nip_submission", "production_qualification", "publication", "release", "deployment", "remote_mutation"]
PATHS = [
    ROOT / "spec/remediation_v19_authority.json",
    ROOT / "spec/remediation_findings_v19.json",
    ROOT / "implementation/runtime_ledger_v19.json",
    ROOT / "tools/validation/runtime_ledger_v19.schema.json",
]


class V19Error(RuntimeError):
    pass


def require(condition: bool, code: str) -> None:
    if not condition:
        raise V19Error(code)


def load(path: Path) -> Any:
    def closed(pairs: list[tuple[str, Any]]) -> dict[str, Any]:
        keys = [key for key, _ in pairs]
        require(len(keys) == len(set(keys)), "duplicate:" + path.name)
        return dict(pairs)

    return json.loads(path.read_text(), object_pairs_hook=closed)


def sha(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def git(*args: str) -> str:
    result = subprocess.run(["git", *args], cwd=ROOT, capture_output=True, text=True, check=False)
    require(result.returncode == 0, "git:" + ":".join(args))
    return result.stdout.strip()


def validate(authority: dict[str, Any], findings: dict[str, Any], ledger: dict[str, Any], schema: dict[str, Any]) -> None:
    require(authority["schema"] == "nostr_automerge.remediation_v19_authority.v1", "authority:schema")
    require(authority["status"] in {"active", "code_complete_publication_held"} and authority["result"] == "pass", "authority:status")
    reviewed = authority["reviewed_public"]
    require(reviewed == {"candidate": BASE, "tree": TREE, "actor_source_sha256": ACTOR_SHA}, "authority:baseline")
    require(git("rev-parse", BASE + "^{tree}") == TREE, "authority:tree")
    actor = subprocess.run(["git", "show", f"{BASE}:crates/nostr_automerge/src/graph/actor_state.rs"], cwd=ROOT, capture_output=True, check=True).stdout
    require(hashlib.sha256(actor).hexdigest() == ACTOR_SHA, "authority:actor")
    require(subprocess.run(["git", "merge-base", "--is-ancestor", BASE, "HEAD"], cwd=ROOT).returncode == 0, "authority:ancestry")
    plan = authority["governing_plan"]
    require(plan["path"] == "docs/execution/rcl/nostr_automerge_v1_multi_rcld_v19.md" and sha(ROOT / plan["path"]) == plan["sha256"], "authority:plan")
    historical = authority["historical_v18"]
    require(historical["status"] == "immutable_history" and all((ROOT / historical[key]).is_file() for key in ("authority", "runtime_ledger", "terminal_decision", "clean_attestation")), "authority:history")
    require(authority["active_sequence"] == {"rcld_first": 141, "rcld_last": 146, "rcld_count": 6, "independent_rcld": 145, "opaque_join_rcld": 146}, "authority:sequence")
    require(list(authority["requirement_mapping"]) == FINDINGS, "authority:mapping")
    require(all(value == ["NCRDT-EVIDENCE-007"] for value in authority["requirement_mapping"].values()), "authority:requirements")
    decisions = authority["approved_decisions"]
    require(decisions["production_order"] == ["descriptor", "charge", "target", "completion_observation", "return"], "authority:order")
    require(decisions["proof_events"] == ["ChargeAttempt", "ChargeAccepted", "TargetDispatched", "TargetReturned", "CompletionObserved", "PublicationCompleted"], "authority:events")
    require(decisions["proof_facts"] == "independent_trace_derived", "authority:proof")
    require(decisions["public_minimum_mutations"] == 40 and decisions["independent_minimum_mutations"] == 19, "authority:mutations")
    require(decisions["property_execution"] == "runtime_derived_isolated_worktree", "authority:properties")
    require(decisions["candidate_lifecycle"] == "acyclic_explicit_roles" and decisions["counts"] == "source_derived", "authority:lifecycle")
    require(decisions["public_symbols_changed"] is False, "authority:api")
    frozen = authority["frozen"]
    require(frozen["requirements_count"] == len(load(ROOT / "spec/requirements.json")["requirements"]) == 156, "frozen:requirements")
    require(frozen["scenario_count"] == load(ROOT / "fixtures/distribution/manifest_v16.json")["fixture_count"] == 204, "frozen:scenarios")
    require(frozen["signed_event_count"] == 771 and frozen["delivery_order_count"] == 8 and frozen["processes_per_implementation"] == 2, "frozen:counts")
    require(frozen["ample_work_canonical_sha256"] == "e69c721549966b1b88dcde3296674d675169840c6e8ebd0f236a5c07bcfc6415", "frozen:canonical")
    require(frozen["serialized_run_sha256"] == "000c52bde7102eaccec8cf65c875332e119fd25ccf4a2ac38973c456774a3344", "frozen:serialized")
    for key, path in (("nip_sha256", "spec/NIP_DRAFT.md"), ("requirements_sha256", "spec/requirements.json"), ("report_contract_sha256", "spec/REPORT_CONTRACT.md"), ("distribution_manifest_sha256", "fixtures/distribution/manifest_v16.json"), ("distribution_lock_sha256", "fixtures/distribution/manifest_v16.lock.json")):
        require(frozen[key] == sha(ROOT / path), "frozen:" + key)
    require(authority["holds"] == HOLDS and authority["remote_actions"] == 0, "authority:holds")
    require(findings["schema"] == "nostr_automerge.remediation_findings.v19.v1" and findings["result"] == "pass", "findings:schema")
    rows = findings["findings"]
    require([row["id"] for row in rows] == FINDINGS + ["FINDING_080"], "findings:order")
    require(all(row["status"] in {"open", "closed"} for row in rows[:-1]) and rows[-1]["status"] == "held", "findings:state")
    require(schema["additionalProperties"] is False and schema["properties"]["schema"]["const"] == "nostr_automerge.runtime_ledger.v19.v1", "ledger:schema")
    require(ledger["schema"] == "nostr_automerge.runtime_ledger.v19.v1" and ledger["authority"] == "spec/remediation_v19_authority.json", "ledger:identity")
    cursor = ledger["cursor"]
    completed = cursor["completed_rclds"]
    require(completed == list(range(141, 141 + len(completed))), "ledger:prefix")
    require(cursor["last_planned_rcld"] == 146 and cursor["remaining_rcld_count"] == 6 - len(completed), "ledger:remaining")
    if len(completed) < 6:
        require(cursor["active_rcld"] == cursor["next_rcld"] == 141 + len(completed), "ledger:cursor")
    else:
        require(cursor["active_rcld"] == 146 and cursor["next_rcld"] is None, "ledger:terminal")
    statuses = {row["id"]: row["status"] for row in rows}
    require(ledger["findings"]["open"] == [item for item in FINDINGS if statuses[item] == "open"], "ledger:findings")
    require(ledger["findings"]["held"] == ["FINDING_080"], "ledger:held")
    require(list(ledger["candidate_roles"]) == ["source_candidate", "execution_base_candidate", "proof_artifact_commit", "mutation_artifact_commit", "final_inventory_commit", "evidence_graph_commit", "public_qualification_commit", "independent_assurance_commit", "terminal_artifact_commit", "clean_attestation_commit"], "ledger:roles")
    require(ledger["independent"]["rcld"] == 145 and ledger["independent"]["public_detail"] == "opaque_only", "ledger:independent")
    require(ledger["predecessors"] == [{"rcld": 140, "candidate": BASE, "owner_class": "public", "result": "pass"}], "ledger:predecessor")


def self_test(values: list[dict[str, Any]]) -> int:
    cases = [
        lambda a, _f, _l, _s: a.update(remote_actions=1),
        lambda a, _f, _l, _s: a["approved_decisions"].update(proof_facts="copied"),
        lambda a, _f, _l, _s: a["approved_decisions"].update(public_minimum_mutations=21),
        lambda a, _f, _l, _s: a["frozen"].update(scenario_count=203),
        lambda _a, f, _l, _s: f["findings"][-1].update(status="closed"),
        lambda _a, _f, l, _s: l["cursor"].update(remaining_rcld_count=99),
        lambda _a, _f, l, _s: l["candidate_roles"].pop("clean_attestation_commit"),
        lambda _a, _f, _l, s: s.update(additionalProperties=True),
    ]
    caught = 0
    for mutate in cases:
        changed = copy.deepcopy(values)
        mutate(*changed)
        try:
            validate(*changed)
        except V19Error:
            caught += 1
            continue
        raise V19Error("self_test:survivor")
    return caught


def main() -> int:
    values = [load(path) for path in PATHS]
    validate(*values)
    print(f"PASS remediation v19: rclds=6 findings=4 attacks={self_test(values)} remote_actions=0")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
