#!/usr/bin/env python3
"""Validate active v17 causal-projection authority and runtime routing."""

from __future__ import annotations

import copy
import hashlib
import json
import subprocess
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[1]
PATHS = [
    ROOT / "spec/remediation_v17_authority.json",
    ROOT / "spec/remediation_findings_v17.json",
    ROOT / "implementation/runtime_ledger_v17.json",
    ROOT / "tools/validation/runtime_ledger_v17.schema.json",
]
BASE = "0a0ce4d4ee8723bbec8473f8e6c984be6aa93df1"
TREE = "01211cccdcbf91dc0764e28c08661c746e91f226"
ACTOR_SHA = "101e9502101d7c08d11dadafc46c679a084bfe88b8ea8614c79682565c3bbc0e"
PLAN_SHA = "7b8b705b517190bae5ef6e9a17c921e838598e68ef89d7d019445af1c539b871"
HOLDS = ["external_assurance","event_kind_allocation","nip_submission","production_qualification","publication","release","remote_mutation"]


class V17Error(RuntimeError):
    pass


def require(condition: bool, code: str) -> None:
    if not condition:
        raise V17Error(code)


def load(path: Path) -> Any:
    def closed(pairs: list[tuple[str, Any]]) -> dict[str, Any]:
        keys = [key for key, _ in pairs]
        require(len(keys) == len(set(keys)), "duplicate:" + path.name)
        return dict(pairs)
    return json.loads(path.read_text(), object_pairs_hook=closed)


def sha(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def git(*args: str) -> bytes:
    result = subprocess.run(["git", *args], cwd=ROOT, capture_output=True, check=False)
    require(result.returncode == 0, "git:" + ":".join(args))
    return result.stdout


def validate(authority: Any, findings: Any, ledger: Any, schema: Any) -> None:
    require(authority["schema"] == "nostr_automerge.remediation_v17_authority.v1", "authority:schema")
    require(authority["status"] == "active" and authority["result"] == "pass", "authority:state")
    require(authority["reviewed_public"] == {"candidate":BASE,"tree":TREE,"actor_source_sha256":ACTOR_SHA}, "authority:reviewed")
    require(git("rev-parse", f"{BASE}^{{tree}}").decode().strip() == TREE, "authority:tree")
    require(hashlib.sha256(git("show", f"{BASE}:crates/nostr_automerge/src/graph/actor_state.rs")).hexdigest() == ACTOR_SHA, "authority:source")
    plan = authority["governing_plan"]
    require(plan == {"path":"docs/execution/rcl/nostr_automerge_v1_multi_rcld_v17.md","sha256":PLAN_SHA}, "authority:plan")
    require(sha(ROOT / plan["path"]) == PLAN_SHA, "authority:plan_sha")
    history = authority["historical_v16"]
    history_paths = {"authority_sha256":"spec/remediation_v16_authority.json","findings_sha256":"spec/remediation_findings_v16.json","runtime_ledger_sha256":"implementation/runtime_ledger_v16.json","final_decision_sha256":"reports/causal_projection_final_decision_v16.json"}
    require(all(sha(ROOT / path) == history[field] for field, path in history_paths.items()), "authority:history")
    require(history["status"] == "immutable_history", "authority:history_status")
    require(authority["active_sequence"] == {"rcld_first":129,"rcld_last":133,"step_first":"step_1483","step_last":"step_1513","public_step_count":31,"independent_step_count":7}, "authority:sequence")
    decisions = authority["approved_decisions"]
    require(decisions["final_source_site_count"] is None, "authority:preset_count")
    require(decisions["candidate_lifecycle"] == "acyclic_later_attestation", "authority:candidate")
    require(authority["holds"] == HOLDS and authority["remote_actions"] == 0, "authority:holds")
    frozen = authority["frozen_sha256"]
    require(sha(ROOT / "spec/NIP_DRAFT.md") == frozen["nip"], "authority:nip")
    require(sha(ROOT / "spec/requirements.json") == frozen["requirements"], "authority:requirements")
    require(sha(ROOT / "spec/REPORT_CONTRACT.md") == frozen["report_contract"], "authority:report")

    require(findings["schema"] == "nostr_automerge.remediation_findings.v17.v1" and findings["status"] == "active" and findings["result"] == "pass", "findings:state")
    rows = findings["findings"]
    require([row["id"] for row in rows] == ["FINDING_119","FINDING_120","FINDING_121","FINDING_122","FINDING_080"], "findings:order")
    require([row["status"] for row in rows] == ["open","open","open","open","held"], "findings:status")
    require([row["requirements"] for row in rows[:4]] == list(authority["requirement_mapping"].values()), "findings:requirements")

    require(schema["additionalProperties"] is False and schema["properties"]["schema"]["const"] == "nostr_automerge.runtime_ledger.v17.v1", "schema:closed")
    require(ledger["schema"] == "nostr_automerge.runtime_ledger.v17.v1" and ledger["status"] == "active", "ledger:state")
    require(ledger["authority"] == "spec/remediation_v17_authority.json", "ledger:authority")
    require(ledger["cursor"] == {"active_rcld":129,"active_step":"step_1483","next_step":"step_1484","last_planned_step":"step_1513","remaining_checkpoint_count":30,"remaining_rcld_count":4}, "ledger:cursor")
    require(ledger["findings"] == {"open":["FINDING_119","FINDING_120","FINDING_121","FINDING_122"],"held":["FINDING_080"]}, "ledger:findings")
    require(ledger["independent"]["contract_barrier"] == "step_1487" and ledger["independent"]["distribution_barrier"] == "step_1509", "ledger:barriers")
    require(ledger["predecessors"] == [{"step":"step_1482","candidate":BASE,"owner_class":"public","result":"pass"}], "ledger:predecessor")
    require(all((ROOT / path).is_file() for path in ledger["active_checkpoint_scope"]), "ledger:scope")


def self_test(authority: Any, findings: Any, ledger: Any, schema: Any) -> int:
    cases = [
        lambda a, _f, _l, _s: a.update(remote_actions=1),
        lambda a, _f, _l, _s: a["approved_decisions"].update(final_source_site_count=68),
        lambda a, _f, _l, _s: a["approved_decisions"].update(candidate_lifecycle="self_candidate"),
        lambda _a, f, _l, _s: f["findings"][0].update(status="closed"),
        lambda _a, _f, l, _s: l["cursor"].update(next_step="step_1483"),
        lambda _a, _f, _l, s: s.update(additionalProperties=True),
    ]
    caught = 0
    for mutate in cases:
        values = [copy.deepcopy(value) for value in (authority, findings, ledger, schema)]
        mutate(*values)
        try:
            validate(*values)
        except V17Error:
            caught += 1
            continue
        raise V17Error("mutation:survived")
    return caught


def main() -> int:
    values = [load(path) for path in PATHS]
    validate(*values)
    mutations = self_test(*values)
    print(f"PASS: remediation-v17 active=step_1483 next=step_1484 findings=4 mutations={mutations} remote_actions=0")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
