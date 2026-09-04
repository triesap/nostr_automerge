#!/usr/bin/env python3
"""Validate v18 authority, frozen reproductions, findings, and runtime routing."""

from __future__ import annotations

import copy
import hashlib
import json
import re
import subprocess
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[1]
BASE = "8673ff8546b9e9d57218c15a4b81890d82137184"
TREE = "f6b6d044553d06e71d7c48f4a30d41922e99a2f0"
ACTOR_SHA = "4d825c9126b609bdb1c7ebc8580a901bc3e78bbd373086e5c0e0c2d945cbc3d6"
PLAN_SHA = "3c85951fa8af77f4faa5cfaae5b8dbecce959a8ecd1a695f9458a6744ae79d68"
HOLDS = [
    "external_assurance", "event_kind_allocation", "nip_submission",
    "production_qualification", "publication", "release", "remote_mutation",
]
FINDINGS = [f"FINDING_{number}" for number in range(123, 130)]
JOBS = [
    "remediation", "policy", "standard", "conformance", "coverage",
    "supply_chain", "robustness", "resource", "release_evidence",
]
PATHS = [
    ROOT / "spec/remediation_v18_authority.json",
    ROOT / "spec/remediation_findings_v18.json",
    ROOT / "implementation/runtime_ledger_v18.json",
    ROOT / "tools/validation/runtime_ledger_v18.schema.json",
]


class V18Error(RuntimeError):
    pass


def require(condition: bool, code: str) -> None:
    if not condition:
        raise V18Error(code)


def load(path: Path) -> Any:
    def closed(pairs: list[tuple[str, Any]]) -> dict[str, Any]:
        keys = [key for key, _ in pairs]
        require(len(keys) == len(set(keys)), "duplicate:" + path.name)
        return dict(pairs)
    return json.loads(path.read_text(), object_pairs_hook=closed)


def sha(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def git(*args: str) -> bytes:
    completed = subprocess.run(["git", *args], cwd=ROOT, capture_output=True, check=False)
    require(completed.returncode == 0, "git:" + ":".join(args))
    return completed.stdout


def baseline_text(path: str) -> str:
    return git("show", f"{BASE}:{path}").decode()


def validate_reproductions() -> None:
    actor = baseline_text("crates/nostr_automerge/src/graph/actor_state.rs")
    for prefix in ("ActorDecision", "CausalNext", "ProjectionBuild", "FrontierComparison"):
        pattern = re.compile(
            rf"observed\({prefix}Observation \{{\s*descriptor,\s*kind: {prefix}ObservationKind::ChargeAttempt,\s*\}}\);\s*charge\(",
            re.MULTILINE,
        )
        require(len(pattern.findall(actor)) == 1, "reproduction:precharge:" + prefix)
    require('phase: "construction"' in actor and 'applicability: "public_rust"' in actor, "reproduction:descriptor_vocabulary")

    proof = baseline_text("scripts/validate_causal_projection_proofs_v17.py")
    require('"n_minus_one": "typed_budget_exhausted"' in proof, "reproduction:fixed_n_minus_one")
    require('"n": "observed"' in proof and '"n_plus_one": "observed"' in proof, "reproduction:fixed_success")
    require('"target_after_stop": 0' in proof and '"observation_after_stop": 0' in proof, "reproduction:fixed_counts")

    mutations = baseline_text("scripts/run_causal_projection_mutations_v17.py")
    require("def mutate_direct(source: str, helper: str, site: str, kind: str)" in mutations, "reproduction:direct_entry")
    require('head, tail = source.split(f"fn {helper}", 1)' in mutations, "reproduction:helper_wide")
    require("validate_structure(changed, consumer, INVENTORY, PROPERTIES)" in mutations, "reproduction:in_process_property")
    require("--root" not in mutations, "reproduction:no_isolated_root_cli")


def validate(authority: Any, findings: Any, ledger: Any, schema: Any) -> None:
    require(authority["schema"] == "nostr_automerge.remediation_v18_authority.v1", "authority:schema")
    require(authority["status"] in {"active", "code_complete_publication_held"} and authority["result"] == "pass", "authority:state")
    require(authority["reviewed_public"] == {"candidate": BASE, "tree": TREE, "actor_source_sha256": ACTOR_SHA}, "authority:reviewed")
    require(git("rev-parse", BASE + "^{tree}").decode().strip() == TREE, "authority:tree")
    require(hashlib.sha256(baseline_text("crates/nostr_automerge/src/graph/actor_state.rs").encode()).hexdigest() == ACTOR_SHA, "authority:source")
    plan = authority["governing_plan"]
    require(plan == {"path": "docs/execution/rcl/nostr_automerge_v1_multi_rcld_v18.md", "sha256": PLAN_SHA}, "authority:plan")
    require(sha(ROOT / plan["path"]) == PLAN_SHA, "authority:plan_sha")
    require(authority["historical_v17"]["status"] == "immutable_history", "authority:history")
    require(all((ROOT / authority["historical_v17"][field]).is_file() for field in ("authority", "runtime_ledger", "terminal_decision")), "authority:history_paths")
    require(authority["active_sequence"] == {"rcld_first": 134, "rcld_last": 140, "rcld_count": 7, "independent_rcld": 139, "opaque_join_rcld": 140}, "authority:sequence")
    require(list(authority["requirement_mapping"]) == FINDINGS, "authority:requirements")
    decisions = authority["approved_decisions"]
    require(decisions["sealed_order"] == ["descriptor", "charge", "target", "completion_observation", "return"], "authority:order")
    require(decisions["attempt_telemetry_owner"] == "charge_invocation", "authority:attempt")
    require(decisions["proof_facts"] == "structured_trace_derived" and decisions["proof_count_scope"] == "requested_site_or_post_failed_charge_suffix", "authority:proof")
    require(decisions["direct_mutation"] == "site_local_target_hoist_and_cache" and decisions["site_local_property"] == "SITE_TARGET_BEFORE_CHARGE", "authority:mutation")
    require(decisions["property_execution"] == "isolated_root_subprocess" and decisions["candidate_lifecycle"] == "acyclic_later_catalogs", "authority:evidence")
    require(decisions["counts"] == "source_derived" and decisions["descriptor_phase"] == "projection_construction" and decisions["descriptor_applicability"] == "required", "authority:vocabulary")

    frozen = authority["frozen"]
    require(frozen["requirements_count"] == len(load(ROOT / "spec/requirements.json")["requirements"]) == 156, "frozen:requirements_count")
    manifest = load(ROOT / "fixtures/distribution/manifest_v16.json")
    require(frozen["scenario_count"] == manifest["fixture_count"] == 204 and frozen["signed_event_count"] == 771, "frozen:distribution_counts")
    require(frozen["ample_work_canonical_sha256"] == "e69c721549966b1b88dcde3296674d675169840c6e8ebd0f236a5c07bcfc6415", "frozen:canonical")
    for field, path in (("nip_sha256", "spec/NIP_DRAFT.md"), ("requirements_sha256", "spec/requirements.json"), ("report_contract_sha256", "spec/REPORT_CONTRACT.md"), ("distribution_manifest_sha256", "fixtures/distribution/manifest_v16.json"), ("distribution_lock_sha256", "fixtures/distribution/manifest_v16.lock.json")):
        require(frozen[field] == sha(ROOT / path), "frozen:" + field)
    require(authority["standard_gate_jobs"] == JOBS, "authority:gates")
    gate = (ROOT / "scripts/local_gate.py").read_text()
    require(all(f'"{job}": {job}' in gate for job in JOBS), "authority:gate_discovery")
    require(authority["holds"] == HOLDS and authority["remote_actions"] == 0, "authority:holds")

    require(findings["schema"] == "nostr_automerge.remediation_findings.v18.v1" and findings["result"] == "pass", "findings:state")
    rows = findings["findings"]
    require([row["id"] for row in rows] == FINDINGS + ["FINDING_080"], "findings:order")
    require(all(row["requirements"] == authority["requirement_mapping"][row["id"]] for row in rows[:-1]), "findings:requirements")
    require(all(row["status"] in {"open", "closed"} for row in rows[:-1]) and rows[-1]["status"] == "held", "findings:status")

    require(schema["additionalProperties"] is False and schema["properties"]["schema"]["const"] == "nostr_automerge.runtime_ledger.v18.v1", "ledger_schema:closed")
    require(ledger["schema"] == "nostr_automerge.runtime_ledger.v18.v1" and ledger["authority"] == "spec/remediation_v18_authority.json", "ledger:state")
    completed = ledger["cursor"]["completed_rclds"]
    require(completed == list(range(134, 134 + len(completed))), "ledger:completed_prefix")
    require(ledger["cursor"]["last_planned_rcld"] == 140 and ledger["cursor"]["remaining_rcld_count"] == 7 - len(completed), "ledger:remaining")
    if len(completed) < 7:
        expected = 134 + len(completed)
        require(ledger["cursor"]["active_rcld"] == ledger["cursor"]["next_rcld"] == expected, "ledger:cursor")
    else:
        require(ledger["cursor"]["active_rcld"] == 140 and ledger["cursor"]["next_rcld"] is None, "ledger:terminal_cursor")
    statuses = {row["id"]: row["status"] for row in rows}
    require(ledger["findings"]["open"] == [finding for finding in FINDINGS if statuses[finding] == "open"], "ledger:findings")
    require(ledger["findings"]["held"] == ["FINDING_080"], "ledger:held")
    roles = ledger["candidate_roles"]
    require(list(roles) == ["source_candidate", "execution_base_candidate", "proof_artifact_commit", "mutation_artifact_commit", "final_inventory_commit", "evidence_graph_commit", "terminal_artifact_commit", "clean_attestation_commit"], "ledger:roles")
    require(roles["execution_base_candidate"] == BASE, "ledger:execution_base")
    require(ledger["independent"]["rcld"] == 139 and ledger["independent"]["public_detail"] == "opaque_only", "ledger:independent")
    require(all((ROOT / path).is_file() for path in ledger["active_checkpoint_scope"]), "ledger:scope")
    require(ledger["predecessors"][0] == {"rcld": 133, "candidate": BASE, "owner_class": "public", "result": "pass"}, "ledger:predecessor")
    validate_reproductions()


def self_test(values: list[Any]) -> int:
    cases = [
        lambda a, _f, _l, _s: a.update(remote_actions=1),
        lambda a, _f, _l, _s: a["approved_decisions"].update(attempt_telemetry_owner="completion_observer"),
        lambda a, _f, _l, _s: a["approved_decisions"].update(proof_count_scope="whole_trace"),
        lambda a, _f, _l, _s: a["approved_decisions"].update(site_local_property="CHARGE_AFTER_OPERATION"),
        lambda _a, f, _l, _s: f["findings"][-1].update(status="closed"),
        lambda _a, _f, l, _s: l["cursor"].update(remaining_rcld_count=0),
        lambda _a, _f, l, _s: l["candidate_roles"].pop("clean_attestation_commit"),
        lambda _a, _f, _l, s: s.update(additionalProperties=True),
    ]
    caught = 0
    for mutate in cases:
        mutated = copy.deepcopy(values)
        mutate(*mutated)
        try:
            validate(*mutated)
        except V18Error:
            caught += 1
            continue
        raise V18Error("mutation:survived")
    return caught


def main() -> int:
    values = [load(path) for path in PATHS]
    validate(*values)
    print(f"PASS: remediation-v18 rclds=7 findings=7 reproductions=4 mutations={self_test(values)} remote_actions=0")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
