#!/usr/bin/env python3
"""Validate the active remediation-v12 authority and runtime cursor."""

from __future__ import annotations

import copy
import hashlib
import json
import pathlib
import subprocess


ROOT = pathlib.Path(__file__).resolve().parents[1]
AUTHORITY_PATH = ROOT / "spec/remediation_v12_authority.json"
LEDGER_PATH = ROOT / "implementation/runtime_ledger_v12.json"
FINDINGS_PATH = ROOT / "spec/remediation_findings_v12.json"
REPRODUCTIONS_PATH = ROOT / "spec/remediation_v12_reproductions.json"
EVIDENCE_POLICY_PATH = ROOT / "spec/remediation_v12_evidence_policy.json"

REVIEWED_CANDIDATE = "9e99af892764ccb165a12b8bb186935bd599d561"
REVIEWED_TREE = "4b684dc123f371ded75c1469505b130c36359f93"
PLAN_CANDIDATE = "d1b9202be6bf9deb643ca7d81f89c5c3281eb523"
PLAN_TREE = "739068407a059b071655cc63bcf1b570285fbaf7"
PLAN_PATH = "docs/execution/rcl/nostr_automerge_v1_multi_rcld_v12.md"
PLAN_SHA256 = "aa8ea9bc6801175dd247dd283521a4ad8f0735eafcd280151a842c37418e5585"
HOLDS = [
    "external_assurance",
    "event_kind_allocation",
    "nip_submission",
    "production_qualification",
    "publication",
    "release",
    "remote_mutation",
]
ACTIVE_SCOPE = [
    "docs/adr/README.md",
    "docs/adr/adr_0076_authoritative_epoch_semantic_work.md",
    "docs/adr/adr_0077_complete_runtime_operation_inventory.md",
    "docs/execution/remediation_v12/ledger.md",
    "implementation/runtime_ledger_v12.json",
    "reports/spec_baseline.txt",
    "scripts/validate_adrs.py",
    "scripts/validate_remediation_v12.py",
    "scripts/validate_spec.py",
    "spec/EVIDENCE_POLICY.md",
    "spec/remediation_v12_evidence_policy.json",
    "tools/validation/remediation_v12_evidence_policy.schema.json",
]

EVIDENCE_REQUIREMENTS = [
    "NCRDT-RESOURCE-017",
    "NCRDT-RESOURCE-018",
    "NCRDT-RESOURCE-019",
    "NCRDT-EVIDENCE-007",
]
OWNER_MODES = ["item_metered", "exact_reserved", "sealed_constant_time"]
ROW_FIELDS = [
    "id", "family", "source_path", "source_symbol", "owner_mode",
    "requirements", "test", "command", "candidate", "artifact_sha256",
    "mutation",
]
APPROVED_ROOTS = [
    "docs/adr", "docs/execution/remediation_v12", "fixtures", "implementation",
    "reports", "scripts", "spec", "tests", "tools/validation",
]
OPAQUE_ALLOWED = [
    "artifact_sha256", "candidate", "counts", "identity_sha256", "result_classes",
]
OPAQUE_PROHIBITED = [
    "commands", "credentials", "logs", "package_layout", "paths", "source", "urls",
]


class EvidenceError(RuntimeError):
    pass


def require_keys(value: object, keys: list[str], label: str) -> dict[str, object]:
    if not isinstance(value, dict) or list(value) != keys:
        raise EvidenceError(f"{label}:shape")
    return value


def require_equal(actual: object, expected: object, label: str) -> None:
    if actual != expected:
        raise EvidenceError(label)


def sha256(path: pathlib.Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def git(*args: str) -> str:
    result = subprocess.run(
        ["git", *args], cwd=ROOT, capture_output=True, text=True, check=False
    )
    if result.returncode:
        raise EvidenceError("git:" + ":".join(args))
    return result.stdout.strip()


def validate_authority(authority: object) -> None:
    record = require_keys(
        authority,
        [
            "schema",
            "status",
            "reviewed_public",
            "governing_plan",
            "historical_v11",
            "active_sequence",
            "counts",
            "frozen_sha256",
            "holds",
            "result",
        ],
        "authority",
    )
    require_equal(record["schema"], "nostr_automerge.remediation_v12_authority.v1", "authority:schema")
    require_equal(record["status"], "authority_and_reproduction_correction_required", "authority:status")
    reviewed = require_keys(record["reviewed_public"], ["candidate", "tree"], "authority:reviewed")
    require_equal(reviewed, {"candidate": REVIEWED_CANDIDATE, "tree": REVIEWED_TREE}, "authority:reviewed")
    plan = require_keys(record["governing_plan"], ["candidate", "tree", "path", "sha256"], "authority:plan")
    require_equal(plan, {"candidate": PLAN_CANDIDATE, "tree": PLAN_TREE, "path": PLAN_PATH, "sha256": PLAN_SHA256}, "authority:plan")
    historical = require_keys(record["historical_v11"], ["final_decision_sha256", "runtime_ledger_sha256", "authority_sha256", "status"], "authority:historical")
    require_equal(historical["status"], "immutable_history", "authority:historical_status")
    sequence = require_keys(record["active_sequence"], ["rcld_first", "rcld_last", "step_first", "step_last", "step_count"], "authority:sequence")
    require_equal(sequence, {"rcld_first": 109, "rcld_last": 115, "step_first": "step_1364", "step_last": "step_1419", "step_count": 56}, "authority:sequence")
    counts = require_keys(record["counts"], ["requirements_current", "requirements_target", "scenarios_current", "scenarios_target"], "authority:counts")
    require_equal(counts, {"requirements_current": 152, "requirements_target": 156, "scenarios_current": 198, "scenarios_target": 204}, "authority:counts")
    frozen = require_keys(record["frozen_sha256"], ["nip", "requirements", "report_contract"], "authority:frozen")
    require_equal(frozen, {
        "nip": "8262bf32cb70b7c0e46210441120652e52504fb73839641ac19dddfed840acf8",
        "requirements": "840822a1acf171c887b9a9aba79ddf159ffcd9c5d7a74bd74d7e0bac5c6161f4",
        "report_contract": "636bd1ff32673a00dc0f41440bde61f2b0f8d86f853a7feaaf119de1ff2ce189",
    }, "authority:frozen")
    require_equal(record["holds"], HOLDS, "authority:holds")
    require_equal(record["result"], "pass", "authority:result")


def validate_ledger(ledger: object) -> None:
    record = require_keys(ledger, ["schema", "status", "authority", "cursor", "findings", "requirements", "active_checkpoint_scope", "predecessors"], "ledger")
    require_equal(record["schema"], "nostr_automerge.runtime_ledger.v12.v1", "ledger:schema")
    require_equal(record["status"], "implementation_in_progress", "ledger:status")
    require_equal(record["authority"], "spec/remediation_v12_authority.json", "ledger:authority")
    cursor = require_keys(record["cursor"], ["active_rcld", "active_step", "next_step", "last_planned_step", "remaining_checkpoint_count", "remaining_rcld_count"], "ledger:cursor")
    require_equal(cursor, {"active_rcld": 109, "active_step": "step_1370", "next_step": "step_1371", "last_planned_step": "step_1419", "remaining_checkpoint_count": 49, "remaining_rcld_count": 7}, "ledger:cursor")
    findings = require_keys(record["findings"], ["open", "held"], "ledger:findings")
    require_equal(findings, {"open": ["FINDING_100", "FINDING_101", "FINDING_102", "FINDING_103"], "held": ["FINDING_080"]}, "ledger:findings")
    require_equal(record["requirements"], [], "ledger:requirements")
    require_equal(record["active_checkpoint_scope"], ACTIVE_SCOPE, "ledger:scope")
    predecessors = record["predecessors"]
    if not isinstance(predecessors, list) or len(predecessors) != 8:
        raise EvidenceError("ledger:predecessors")
    require_equal(predecessors[0], {"step": "step_1363", "candidate": REVIEWED_CANDIDATE, "owner_class": "public", "result": "pass"}, "ledger:predecessor_v11")
    require_equal(predecessors[1], {"step": "plan_v12", "candidate": PLAN_CANDIDATE, "owner_class": "public", "result": "pass"}, "ledger:predecessor_plan")
    require_equal(predecessors[2], {"step": "step_1364", "candidate": "22cb8f0c77637647ce485e4d6f206316113e429a", "owner_class": "public", "result": "pass"}, "ledger:predecessor_1364")
    require_equal(predecessors[3], {"step": "step_1365", "candidate": "4e6b9e2c189d407b29a478c5445405b922789aa0", "owner_class": "public", "result": "pass"}, "ledger:predecessor_1365")
    require_equal(predecessors[4], {"step": "step_1366", "candidate": "00fca7681ba079e98ebf8d116bc7fa12926d1a87", "owner_class": "public", "result": "pass"}, "ledger:predecessor_1366")
    require_equal(predecessors[5], {"step": "step_1367", "candidate": "1de9769b36b5fa610483c3f0ffcd0e7e6ee2768c", "owner_class": "public", "result": "pass"}, "ledger:predecessor_1367")
    require_equal(predecessors[6], {"step": "step_1368", "candidate": "bb8b8fd4560eaf141ff599ed440edeb68c30a33f", "owner_class": "public", "result": "pass"}, "ledger:predecessor_1368")
    require_equal(predecessors[7], {"step": "step_1369", "candidate": "4819b9ae58650f8b5decfb19e0f8d895dc47c7d2", "owner_class": "public", "result": "pass"}, "ledger:predecessor_1369")


def validate_reproductions(reproductions: object) -> None:
    record = require_keys(reproductions, ["schema", "cases", "result"], "reproductions")
    require_equal(record["schema"], "nostr_automerge.remediation_v12_reproductions.v1", "reproductions:schema")
    rows = record["cases"]
    if not isinstance(rows, list) or len(rows) != 10:
        raise EvidenceError("reproductions:rows")
    expected = [
        ("actor_predecessor", "crates/nostr_automerge/src/graph/actor_state.rs", "graph::actor_state::tests::finding_100_actor_predecessor_scan_reproduction", "unmetered actor predecessor collection remains"),
        ("causal_next_op", "crates/nostr_automerge/src/graph/actor_state.rs", "graph::actor_state::tests::finding_100_causal_next_op_scan_reproduction", "unmetered causal next-op scan remains"),
        ("empty_frontier", "crates/nostr_automerge/src/graph/actor_state.rs", "graph::actor_state::tests::finding_100_empty_frontier_work_reproduction", "unmetered empty-frontier allocation remains"),
        ("epoch_ancestry", "crates/nostr_automerge/src/graph/epoch.rs", "graph::epoch::tests::finding_100_epoch_ancestry_work_reproduction", "unmetered epoch ancestry materialization remains"),
        ("epoch_writer_authorization", "crates/nostr_automerge/src/reference/epoch_engine.rs", "reference::epoch_engine::tests::finding_100_epoch_writer_authorization_work_reproduction", "unmetered epoch writer authorization scan remains"),
        ("dependency_closure", "crates/nostr_automerge/src/graph/closure.rs", "graph::closure::tests::finding_100_dependency_closure_work_reproduction", "unmetered dependency-closure preparation remains"),
        ("schedule_readiness", "crates/nostr_automerge/src/graph/schedule.rs", "graph::schedule::tests::finding_100_schedule_readiness_work_reproduction", "unmetered schedule readiness and pop preparation remains"),
        ("schedule_publication", "crates/nostr_automerge/src/graph/schedule.rs", "graph::schedule::tests::finding_100_schedule_publication_work_reproduction", "unmetered schedule insertion and result publication remains"),
        ("quarantine_overlays", "crates/nostr_automerge/src/reference/epoch_engine.rs", "reference::epoch_engine::tests::finding_100_quarantine_overlay_work_reproduction", "unmetered selected and fallback quarantine overlays remain"),
        ("zero_post_stop", "crates/nostr_automerge/src/reference/epoch_engine.rs", "reference::epoch_engine::tests::finding_100_zero_post_stop_work_reproduction", "unmetered target preparation remains before the first stop"),
    ]
    for index, (family, path, test, diagnostic) in enumerate(expected):
        row = require_keys(rows[index], ["finding", "family", "kind", "path", "test", "diagnostic", "expected"], f"reproductions:row:{index}")
        require_equal(row, {
            "finding": "FINDING_100",
            "family": family,
            "kind": "rust_failure",
            "path": path,
            "test": test,
            "diagnostic": diagnostic,
            "expected": "open_failure",
        }, f"reproductions:{family}")
    require_equal(record["result"], "pass", "reproductions:result")


def validate_findings(findings: object) -> None:
    record = require_keys(findings, ["schema", "status", "findings", "result"], "findings")
    require_equal(record["schema"], "nostr_automerge.remediation_findings.v12.v1", "findings:schema")
    require_equal(record["status"], "implementation_in_progress", "findings:status")
    rows = record["findings"]
    if not isinstance(rows, list) or len(rows) != 5:
        raise EvidenceError("findings:rows")
    expected_ids = ["FINDING_100", "FINDING_101", "FINDING_102", "FINDING_103", "FINDING_080"]
    require_equal([row.get("id") if isinstance(row, dict) else None for row in rows], expected_ids, "findings:ids")
    for row in rows:
        require_keys(row, ["id", "severity", "class", "title", "requirements", "source_paths", "closure", "status"], "findings:row")
        if not isinstance(row["title"], str) or not row["title"]:
            raise EvidenceError("findings:title")
        if not isinstance(row["closure"], str) or not row["closure"]:
            raise EvidenceError("findings:closure")
        if not isinstance(row["requirements"], list) or not isinstance(row["source_paths"], list):
            raise EvidenceError("findings:vectors")
    require_equal([row["status"] for row in rows], ["open", "open", "open", "open", "held"], "findings:statuses")
    require_equal(rows[-1]["severity"], "hold", "findings:held_severity")
    require_equal(record["result"], "pass", "findings:result")


def validate_evidence_policy(policy: object) -> None:
    record = require_keys(policy, [
        "schema", "status", "authority", "policy", "decisions", "requirements",
        "owner_modes", "required_row_fields", "approved_roots",
        "opaque_allowed_fields", "opaque_prohibited_fields", "holds", "result",
    ], "evidence_policy")
    require_equal(record["schema"], "nostr_automerge.remediation_v12_evidence_policy.v1", "evidence_policy:schema")
    require_equal(record["status"], "approved_staged", "evidence_policy:status")
    require_equal(record["authority"], "spec/remediation_v12_authority.json", "evidence_policy:authority")
    require_equal(require_keys(record["policy"], ["path", "sha256"], "evidence_policy:policy"), {
        "path": "spec/EVIDENCE_POLICY.md",
        "sha256": "43f99e4151b037682f2135d1f80e4e254fcc59d4097fc2032b7a8be519bd51fc",
    }, "evidence_policy:policy")
    decisions = record["decisions"]
    require_equal(decisions, [
        {
            "id": "ADR-0076",
            "path": "docs/adr/adr_0076_authoritative_epoch_semantic_work.md",
            "sha256": "35876ef2f7d8c189d535c104bbd4baa57bd2e94d432f7b04147373f976f3463a",
        },
        {
            "id": "ADR-0077",
            "path": "docs/adr/adr_0077_complete_runtime_operation_inventory.md",
            "sha256": "8f4f4d51c763272e84f1a16d93fe2428461d3658c8377e975f318875881bb6db",
        },
    ], "evidence_policy:decisions")
    for index, decision in enumerate(decisions):
        require_keys(decision, ["id", "path", "sha256"], f"evidence_policy:decision:{index}")
    require_equal(record["requirements"], EVIDENCE_REQUIREMENTS, "evidence_policy:requirements")
    require_equal(record["owner_modes"], OWNER_MODES, "evidence_policy:owner_modes")
    require_equal(record["required_row_fields"], ROW_FIELDS, "evidence_policy:row_fields")
    require_equal(record["approved_roots"], APPROVED_ROOTS, "evidence_policy:roots")
    require_equal(record["opaque_allowed_fields"], OPAQUE_ALLOWED, "evidence_policy:opaque_allowed")
    require_equal(record["opaque_prohibited_fields"], OPAQUE_PROHIBITED, "evidence_policy:opaque_prohibited")
    require_equal(record["holds"], HOLDS, "evidence_policy:holds")
    require_equal(record["result"], "pass", "evidence_policy:result")


def validate_files() -> None:
    require_equal(git("rev-parse", f"{REVIEWED_CANDIDATE}^{{tree}}"), REVIEWED_TREE, "git:reviewed_tree")
    require_equal(git("rev-parse", f"{PLAN_CANDIDATE}^{{tree}}"), PLAN_TREE, "git:plan_tree")
    require_equal(sha256(ROOT / PLAN_PATH), PLAN_SHA256, "file:plan")
    require_equal(sha256(ROOT / "spec/NIP_DRAFT.md"), "8262bf32cb70b7c0e46210441120652e52504fb73839641ac19dddfed840acf8", "file:nip")
    require_equal(sha256(ROOT / "spec/requirements.json"), "840822a1acf171c887b9a9aba79ddf159ffcd9c5d7a74bd74d7e0bac5c6161f4", "file:requirements")
    require_equal(sha256(ROOT / "spec/REPORT_CONTRACT.md"), "636bd1ff32673a00dc0f41440bde61f2b0f8d86f853a7feaaf119de1ff2ce189", "file:report_contract")
    instructions = (ROOT / "AGENTS.md").read_text()
    if "nostr_automerge_v1_multi_rcld_v12.md" not in instructions or "RCLDs 109 through 115" not in instructions:
        raise EvidenceError("file:instructions")
    require_equal(sha256(ROOT / "spec/EVIDENCE_POLICY.md"), "43f99e4151b037682f2135d1f80e4e254fcc59d4097fc2032b7a8be519bd51fc", "file:evidence_policy")
    require_equal(sha256(ROOT / "docs/adr/adr_0076_authoritative_epoch_semantic_work.md"), "35876ef2f7d8c189d535c104bbd4baa57bd2e94d432f7b04147373f976f3463a", "file:adr_0076")
    require_equal(sha256(ROOT / "docs/adr/adr_0077_complete_runtime_operation_inventory.md"), "8f4f4d51c763272e84f1a16d93fe2428461d3658c8377e975f318875881bb6db", "file:adr_0077")
    require_equal(sha256(ROOT / "tools/validation/remediation_v12_evidence_policy.schema.json"), "f4be03d9d38af88277182d951a8e67ff2a34f7090b9c413e66a7a373b50ba669", "file:evidence_policy_schema")


def mutation_self_test(authority: object, ledger: object, findings: object, reproductions: object, evidence_policy: object) -> int:
    mutations: list[tuple[str, object, object]] = []
    for label, path, value in (
        ("reviewed", ("reviewed_public", "candidate"), "0" * 40),
        ("plan", ("governing_plan", "sha256"), "0" * 64),
        ("count", ("counts", "scenarios_target"), 205),
        ("hold", ("holds",), HOLDS[:-1]),
    ):
        changed = copy.deepcopy(authority)
        target = changed
        for key in path[:-1]:
            target = target[key]
        target[path[-1]] = value
        mutations.append((label, changed, ledger))
    extra = copy.deepcopy(authority)
    extra["unapproved"] = False
    mutations.append(("authority_extra", extra, ledger))
    reordered = copy.deepcopy(authority)
    reordered["schema"] = reordered.pop("schema")
    mutations.append(("authority_order", reordered, ledger))
    for label, field, value in (
        ("cursor", "next_step", "step_1372"),
        ("scope", "active_checkpoint_scope", ACTIVE_SCOPE[:-1]),
        ("finding", "findings", {"open": ["FINDING_100"], "held": ["FINDING_080"]}),
    ):
        changed = copy.deepcopy(ledger)
        if field == "next_step":
            changed["cursor"][field] = value
        else:
            changed[field] = value
        mutations.append((label, authority, changed))
    for label, changed_authority, changed_ledger in mutations:
        try:
            validate_authority(changed_authority)
            validate_ledger(changed_ledger)
        except EvidenceError:
            continue
        raise EvidenceError("mutation_survived:" + label)
    finding_mutations = []
    missing = copy.deepcopy(findings)
    missing["findings"].pop(1)
    finding_mutations.append(("finding_missing", missing))
    closed = copy.deepcopy(findings)
    closed["findings"][0]["status"] = "closed"
    finding_mutations.append(("finding_closed", closed))
    unheld = copy.deepcopy(findings)
    unheld["findings"][-1]["status"] = "open"
    finding_mutations.append(("finding_unheld", unheld))
    extra_finding_key = copy.deepcopy(findings)
    extra_finding_key["findings"][0]["unapproved"] = False
    finding_mutations.append(("finding_extra", extra_finding_key))
    for label, changed in finding_mutations:
        try:
            validate_findings(changed)
        except EvidenceError:
            continue
        raise EvidenceError("mutation_survived:" + label)
    reproduction_mutations = []
    missing_reproduction = copy.deepcopy(reproductions)
    missing_reproduction["cases"].clear()
    reproduction_mutations.append(("reproduction_missing", missing_reproduction))
    fixed_early = copy.deepcopy(reproductions)
    fixed_early["cases"][0]["expected"] = "fixed_pass"
    reproduction_mutations.append(("reproduction_premature", fixed_early))
    for label, changed in reproduction_mutations:
        try:
            validate_reproductions(changed)
        except EvidenceError:
            continue
        raise EvidenceError("mutation_survived:" + label)
    policy_mutations = []
    for label, path, value in (
        ("policy_root", ("approved_roots",), APPROVED_ROOTS[:-1]),
        ("policy_mode", ("owner_modes",), OWNER_MODES[::-1]),
        ("policy_row", ("required_row_fields",), ROW_FIELDS[:-1]),
        ("policy_hold", ("holds",), HOLDS[:-1]),
        ("policy_hash", ("policy", "sha256"), "0" * 64),
    ):
        changed = copy.deepcopy(evidence_policy)
        target = changed
        for key in path[:-1]:
            target = target[key]
        target[path[-1]] = value
        policy_mutations.append((label, changed))
    extra_policy = copy.deepcopy(evidence_policy)
    extra_policy["unapproved"] = False
    policy_mutations.append(("policy_extra", extra_policy))
    reordered_policy = copy.deepcopy(evidence_policy)
    reordered_policy["schema"] = reordered_policy.pop("schema")
    policy_mutations.append(("policy_order", reordered_policy))
    for label, changed in policy_mutations:
        try:
            validate_evidence_policy(changed)
        except EvidenceError:
            continue
        raise EvidenceError("mutation_survived:" + label)
    return len(mutations) + len(finding_mutations) + len(reproduction_mutations) + len(policy_mutations)


def main() -> None:
    authority = json.loads(AUTHORITY_PATH.read_text())
    ledger = json.loads(LEDGER_PATH.read_text())
    findings = json.loads(FINDINGS_PATH.read_text())
    reproductions = json.loads(REPRODUCTIONS_PATH.read_text())
    evidence_policy = json.loads(EVIDENCE_POLICY_PATH.read_text())
    validate_authority(authority)
    validate_ledger(ledger)
    validate_findings(findings)
    validate_reproductions(reproductions)
    validate_evidence_policy(evidence_policy)
    validate_files()
    mutation_count = mutation_self_test(authority, ledger, findings, reproductions, evidence_policy)
    print("PASS: remediation v12 authority")
    print(f"- mutations={mutation_count}")
    print("- active=RCLD109/step_1370")


if __name__ == "__main__":
    main()
