#!/usr/bin/env python3
"""Validate the closed remediation v11 authority, findings, and runtime cursor."""

from __future__ import annotations

import copy
import hashlib
import json
import pathlib
import subprocess

ROOT = pathlib.Path(__file__).resolve().parents[1]
AUTHORITY = ROOT / "spec/remediation_v11_authority.json"
FINDINGS = ROOT / "spec/remediation_findings_v11.json"
EVIDENCE = ROOT / "reports/evidence_transition_v11.json"
LEDGER = ROOT / "implementation/runtime_ledger_v11.json"
PLAN = ROOT / "docs/execution/rcl/nostr_automerge_v1_multi_rcld_v11.md"
REPRODUCTIONS = ROOT / "spec/remediation_v11_reproductions.json"

PUBLIC_CANDIDATE = "e1b4f461c0d2a1e8cc8e520bed2dfa64a62270f2"
PUBLIC_TREE = "5e62938bfe576d6c67b3bfe355d5b5dd47585e87"
PRIVATE_CANDIDATE = "2d708bb0a7a00523ab5c244fd0a15c96afcf0a4a"
PRIOR_HANDOFF = "e333873b2b2e42b42bc7d9e652012195ab70760b586eb184462e655a5682be44"
STEP_1308 = "e3c88c6803dcd862b225ff169cf6d65ac5655a6f"
STEP_1309 = "a582868b753400bd74b77090292aa3621f61d910"
STEP_1310 = "4f9599a6cfcf0efe214fe68b0a858295bd616519"
STEP_1311 = "0884fadfa7e9c4227acec351f2385a14a00d2ab2"
STEP_1312 = "ca36b2efac805f67173f9ba384e6c00243578a12"
STEP_1313 = "8f5a17d195be7bcb68af78aa20fc99e14934e2e4"
STEP_1314 = "f2816e457876fb6f0a58f37dcd9ab54970360ef6"
STEP_1315 = "9f180d4141455522d647579fee049551a415aff9"
STEP_1316 = "d6d7f4fb9984e72041cafe242aae5c14494ece8d"
STEP_1317 = "3b41019d78a50b25d3d131065b5a94c307663f3b"
STEP_1318 = "5ca15673198af818aa64a9413d775da7fe9240b8"
STEP_1319 = "c0a3f89c913b3d1df7aadf07460db8d533d61a43"
STEP_1320 = "8b508d9e9b8da34addd061b60465dc41ef62648d"
STEP_1321 = "e04e0e557755c5f7a460eb60231f6e123c86ebb1"
STEP_1322 = "64604f274341df014634a9dcf4084b95b644a46d"
STEP_1323 = "4191a65a7c6cf8e27184d8c5d61b42381f9cf250"
STEP_1324 = "9ae36ba68525be4284fb96266c5e76c3c576fa13"
STEP_1325 = "27618d0ed85f2d9bb38e2f4f6258262f801bb2df"
STEP_1326 = "31e9ec2358fe6dd956baf43b7581273deaf5240d"
STEP_1327 = "e819055185480850d83330631744bb99b44c2c19"
PLAN_SHA256 = "02348e20f719c0ffceda9a2d8afb9cfbeaafc579a4b9b23ba36cf719b948dc42"
HARNESS_SHA256 = "639526c213c2727513ab82fc282344a2f3b524ba8da4ede0cbe170881fef0705"
REPRODUCTIONS_SHA256 = "48501be122679e5cd0846bd2d002bea3e3356355e086c9148a184b2384a266b6"
HOLDS = (
    "external_assurance", "event_kind_allocation", "nip_submission",
    "production_qualification", "publication", "release", "remote_mutation",
)
FINDING_IDS = ("FINDING_096", "FINDING_097", "FINDING_098", "FINDING_099", "FINDING_080")
FINDING_CLASSES = (
    "resource_accounting", "resource_accounting", "specification_authority",
    "ownership_hardening", "external_assurance",
)
FINDING_SEVERITIES = ("high", "high", "high", "medium", "hold")
FINDING_REQUIREMENTS = (
    ("NCRDT-RESOURCE-001", "NCRDT-RESOURCE-014", "NCRDT-RESOURCE-015", "NCRDT-COMPLETION-001"),
    ("NCRDT-RESOURCE-001", "NCRDT-RESOURCE-014", "NCRDT-RESOURCE-016", "NCRDT-COMPLETION-001"),
    ("NCRDT-VERSION-002", "NCRDT-VERSION-003", "NCRDT-EVIDENCE-006"),
    ("NCRDT-OWNERSHIP-001", "NCRDT-RESOURCE-001"),
    (),
)
HISTORICAL_EVIDENCE = (
    ("spec/resource_followup_authority_v10.json", "0cac9bf4b90c55e428c335797a9d7195bc3ee08eed5bfb49fca4428e62702531"),
    ("spec/resource_operation_inventory_v10.json", "cae0e490046cd70f1798573bcf80e0e9f4d520e37afb19225a84845b11b63525"),
    ("spec/resource_ancestry_proof_catalog_v10.json", "a6158951a9e67b7dfcf16765bccb752a6fd20e6e6feb2fde3468c1c66ca1d238"),
    ("implementation/runtime_ledger_v10.json", "9f1227ccb391d8ca120463b5be25ce14b14647c28546721afc685d985f8915d0"),
    ("reports/resource_followup_finding_closure_v10.json", "a544cdf1d2be10a855891e0681df2d236dcaf7a1f7230eb35c4d719ec738dd83"),
    ("reports/resource_followup_final_decision_v10.json", "43d28679234b7c11878f615faf57fc65f298fa99505cdcc70f2d86022b40dd9c"),
)
SCOPE = (
    "crates/nostr_automerge/src/control/candidate.rs",
    "crates/nostr_automerge/src/control/parent_view.rs",
    "crates/nostr_automerge/src/engine/reference_evaluator.rs",
    "crates/nostr_automerge/src/reference/evaluate.rs",
    "docs/execution/remediation_v11/ledger.md",
    "implementation/runtime_ledger_v11.json",
    "reports/spec_baseline.txt",
    "scripts/validate_persistent_state_v11.py",
    "scripts/validate_remediation_v11.py",
)


class ValidationError(RuntimeError):
    pass


def require_record(value: object, keys: tuple[str, ...], label: str) -> dict[str, object]:
    if not isinstance(value, dict) or tuple(value) != keys:
        raise ValidationError(f"{label}:keys")
    return value


def sha256(path: pathlib.Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def validate_authority(value: object) -> None:
    record = require_record(
        value,
        ("schema", "status", "reviewed_public", "opaque_private", "prior_handoff_sha256", "historical_sequence", "active_sequence", "counts", "holds", "result"),
        "authority",
    )
    if record["schema"] != "nostr_automerge.remediation_v11_authority.v1" or record["result"] != "pass":
        raise ValidationError("authority:identity")
    if record["status"] != "authority_and_reproduction_correction_required":
        raise ValidationError("authority:status")
    if record["reviewed_public"] != {"candidate": PUBLIC_CANDIDATE, "tree": PUBLIC_TREE}:
        raise ValidationError("authority:public")
    if record["opaque_private"] != {"candidate": PRIVATE_CANDIDATE, "source_disclosure": False}:
        raise ValidationError("authority:private")
    if record["prior_handoff_sha256"] != PRIOR_HANDOFF:
        raise ValidationError("authority:handoff")
    if record["historical_sequence"] != {"rcld_first": 95, "rcld_last": 99, "step_first": "step_1288", "step_last": "step_1307", "status": "immutable_historical_superseded_for_v11_scope"}:
        raise ValidationError("authority:history")
    if record["active_sequence"] != {"rcld_first": 100, "rcld_last": 108, "step_first": "step_1308", "step_last": "step_1363", "step_count": 56}:
        raise ValidationError("authority:sequence")
    if record["counts"] != {"requirements_current": 148, "requirements_target": 152, "scenarios_current": 193, "scenarios_target": 198}:
        raise ValidationError("authority:counts")
    if tuple(record["holds"]) != HOLDS:
        raise ValidationError("authority:holds")


def validate_findings(value: object) -> None:
    record = require_record(value, ("schema", "status", "findings", "result"), "findings")
    if record["schema"] != "nostr_automerge.remediation_findings.v11.v1" or record["status"] != "implementation_remediation_required" or record["result"] != "pass":
        raise ValidationError("findings:identity")
    rows = record["findings"]
    if not isinstance(rows, list) or len(rows) != 5:
        raise ValidationError("findings:count")
    for index, row in enumerate(rows):
        item = require_record(row, ("id", "severity", "class", "title", "requirements", "source_paths", "closure", "status"), f"finding:{index}")
        if item["id"] != FINDING_IDS[index] or item["severity"] != FINDING_SEVERITIES[index] or item["class"] != FINDING_CLASSES[index]:
            raise ValidationError(f"finding:{index}:identity")
        if tuple(item["requirements"]) != FINDING_REQUIREMENTS[index]:
            raise ValidationError(f"finding:{index}:requirements")
        expected_status = "held" if item["id"] == "FINDING_080" else "open"
        if item["status"] != expected_status:
            raise ValidationError(f"finding:{index}:status")
        if not isinstance(item["title"], str) or not item["title"] or not isinstance(item["closure"], str) or not item["closure"]:
            raise ValidationError(f"finding:{index}:text")
        paths = item["source_paths"]
        if not isinstance(paths, list) or len(paths) != len(set(paths)) or any(not isinstance(path, str) or not (ROOT / path).is_file() for path in paths):
            raise ValidationError(f"finding:{index}:paths")


def validate_evidence(value: object, check_files: bool = False) -> None:
    record = require_record(value, ("schema", "status", "historical_findings", "new_findings", "retained_hold", "historical_evidence", "supersession", "result"), "evidence")
    if record["schema"] != "nostr_automerge.evidence_transition.v11.v1" or record["status"] != "historical_v10_superseded_for_v11_resource_scope" or record["result"] != "pass":
        raise ValidationError("evidence:identity")
    if tuple(record["historical_findings"]) != ("FINDING_094", "FINDING_095") or tuple(record["new_findings"]) != FINDING_IDS[:4] or record["retained_hold"] != "FINDING_080":
        raise ValidationError("evidence:findings")
    rows = record["historical_evidence"]
    if not isinstance(rows, list) or tuple((row.get("path"), row.get("sha256")) for row in rows if isinstance(row, dict)) != HISTORICAL_EVIDENCE:
        raise ValidationError("evidence:artifacts")
    for row in rows:
        require_record(row, ("path", "sha256"), "evidence:artifact")
    if check_files:
        for path, digest in HISTORICAL_EVIDENCE:
            if sha256(ROOT / path) != digest:
                raise ValidationError(f"evidence:hash:{path}")
    if record["supersession"] != {
        "preserved_claims": "FINDING_094 and FINDING_095 remain closed for their exact recorded operations and checkpoint ancestry behavior.",
        "superseded_claims": "The v10 inventory does not prove every retained-depth persistent lookup or every remaining target-sized preparation operation required by Findings 096 and 097.",
        "historical_bytes_mutable": False,
        "v11_closure_required": True,
    }:
        raise ValidationError("evidence:supersession")


def validate_plan(value: object) -> None:
    if not isinstance(value, str):
        raise ValidationError("plan:type")
    ranges = (
        (100, "step_1308", "step_1314"),
        (101, "step_1315", "step_1320"),
        (102, "step_1321", "step_1326"),
        (103, "step_1327", "step_1334"),
        (104, "step_1335", "step_1339"),
        (105, "step_1340", "step_1345"),
        (106, "step_1346", "step_1351"),
        (107, "step_1352", "step_1358"),
        (108, "step_1359", "step_1363"),
    )
    if value.count("| RCLD | Checkpoints | Lane | Exit condition |") != 1:
        raise ValidationError("plan:table")
    for rcld, first, last in ranges:
        marker = f"| {rcld} | `{first}`–`{last}` |"
        if value.count(marker) != 1:
            raise ValidationError(f"plan:rcld:{rcld}")
    if "Steps `step_1308` through `step_1363` are 56 contiguous checkpoints." not in value:
        raise ValidationError("plan:count")
    if "No remote\naction is authorized." not in value:
        raise ValidationError("plan:remote")
    if "A red checkpoint is repaired, split, or blocked and is never committed." not in value:
        raise ValidationError("plan:red")


def validate_reproductions(value: object) -> None:
    record = require_record(value, ("schema", "cases", "result"), "reproductions")
    if record["schema"] != "nostr_automerge.remediation_v11_reproductions.v1" or record["result"] != "pass":
        raise ValidationError("reproductions:identity")
    cases = record["cases"]
    if not isinstance(cases, list) or len(cases) != 4:
        raise ValidationError("reproductions:count")
    kinds = ("rust_failure", "source_failure", "source_failure", "rust_failure")
    for index, case in enumerate(cases):
        item = require_record(case, ("finding", "kind", "path", "test", "diagnostic", "expected"), f"reproduction:{index}")
        if item["finding"] != FINDING_IDS[index] or item["kind"] != kinds[index] or item["expected"] != "open_failure":
            raise ValidationError(f"reproduction:{index}:identity")
        if not all(isinstance(item[key], str) and item[key] for key in ("path", "test", "diagnostic")):
            raise ValidationError(f"reproduction:{index}:value")
        if not (ROOT / item["path"]).is_file():
            raise ValidationError(f"reproduction:{index}:path")
    if len({case["test"] for case in cases}) != 4:
        raise ValidationError("reproductions:duplicate")


def validate_ledger(value: object) -> None:
    record = require_record(value, ("schema", "status", "authority", "cursor", "findings", "requirements", "active_checkpoint_scope", "predecessors", "holds", "result"), "ledger")
    if record["schema"] != "nostr_automerge.runtime_ledger.v11.v1" or record["status"] != "authority_and_reproduction_correction_required" or record["result"] != "pass":
        raise ValidationError("ledger:identity")
    if record["authority"] != "spec/remediation_v11_authority.json":
        raise ValidationError("ledger:authority")
    if record["cursor"] != {"active_rcld": 103, "active_step": "step_1328", "next_step": "step_1329", "last_planned_step": "step_1363", "remaining_checkpoint_count": 36, "remaining_rcld_count": 7}:
        raise ValidationError("ledger:cursor")
    if record["findings"] != {"open": list(FINDING_IDS[:4]), "held": ["FINDING_080"]}:
        raise ValidationError("ledger:findings")
    if tuple(record["requirements"]) != ("NCRDT-RESOURCE-015", "NCRDT-RESOURCE-016", "NCRDT-VERSION-003", "NCRDT-OWNERSHIP-001"):
        raise ValidationError("ledger:requirements")
    if tuple(record["active_checkpoint_scope"]) != SCOPE:
        raise ValidationError("ledger:scope")
    if record["predecessors"] != [
        {"step": "step_1308", "candidate": STEP_1308, "owner_class": "public", "result": "pass"},
        {"step": "step_1309", "candidate": STEP_1309, "owner_class": "public", "result": "pass"},
        {"step": "step_1310", "candidate": STEP_1310, "owner_class": "public", "result": "pass"},
        {"step": "step_1311", "candidate": STEP_1311, "owner_class": "public", "result": "pass"},
        {"step": "step_1312", "candidate": STEP_1312, "owner_class": "public", "result": "pass"},
        {"step": "step_1313", "candidate": STEP_1313, "owner_class": "public", "result": "pass"},
        {"step": "step_1314", "candidate": STEP_1314, "owner_class": "public", "result": "pass"},
        {"step": "step_1315", "candidate": STEP_1315, "owner_class": "public", "result": "pass"},
        {"step": "step_1316", "candidate": STEP_1316, "owner_class": "public", "result": "pass"},
        {"step": "step_1317", "candidate": STEP_1317, "owner_class": "public", "result": "pass"},
        {"step": "step_1318", "candidate": STEP_1318, "owner_class": "public", "result": "pass"},
        {"step": "step_1319", "candidate": STEP_1319, "owner_class": "public", "result": "pass"},
        {"step": "step_1320", "candidate": STEP_1320, "owner_class": "public", "result": "pass"},
        {"step": "step_1321", "candidate": STEP_1321, "owner_class": "public", "result": "pass"},
        {"step": "step_1322", "candidate": STEP_1322, "owner_class": "public", "result": "pass"},
        {"step": "step_1323", "candidate": STEP_1323, "owner_class": "public", "result": "pass"},
        {"step": "step_1324", "candidate": STEP_1324, "owner_class": "public", "result": "pass"},
        {"step": "step_1325", "candidate": STEP_1325, "owner_class": "public", "result": "pass"},
        {"step": "step_1326", "candidate": STEP_1326, "owner_class": "public", "result": "pass"},
        {"step": "step_1327", "candidate": STEP_1327, "owner_class": "public", "result": "pass"},
    ]:
        raise ValidationError("ledger:predecessors")
    if tuple(record["holds"]) != HOLDS:
        raise ValidationError("ledger:holds")


def validate_repository() -> None:
    validate_authority(json.loads(AUTHORITY.read_text()))
    validate_findings(json.loads(FINDINGS.read_text()))
    validate_evidence(json.loads(EVIDENCE.read_text()), check_files=True)
    validate_plan(PLAN.read_text())
    validate_reproductions(json.loads(REPRODUCTIONS.read_text()))
    validate_ledger(json.loads(LEDGER.read_text()))
    if sha256(PLAN) != PLAN_SHA256:
        raise ValidationError("repository:plan_hash")
    if sha256(ROOT / "scripts/reproduce_remediation_v11.py") != HARNESS_SHA256:
        raise ValidationError("repository:harness_hash")
    if sha256(REPRODUCTIONS) != REPRODUCTIONS_SHA256:
        raise ValidationError("repository:reproductions_hash")
    tree = subprocess.run(["git", "rev-parse", f"{PUBLIC_CANDIDATE}^{{tree}}"], cwd=ROOT, check=True, capture_output=True, text=True).stdout.strip()
    if tree != PUBLIC_TREE:
        raise ValidationError("repository:tree")
    status = subprocess.run(["git", "status", "--porcelain=v1", "-z", "--untracked-files=all"], cwd=ROOT, check=True, capture_output=True).stdout.decode().split("\0")
    paths = tuple(sorted(entry[3:] for entry in status if entry))
    if len(paths) != len(set(paths)) or not set(paths).issubset(SCOPE):
        raise ValidationError(f"repository:scope:{paths}")


def mutation_self_test() -> int:
    findings = json.loads(FINDINGS.read_text())
    evidence = json.loads(EVIDENCE.read_text())
    reproductions = json.loads(REPRODUCTIONS.read_text())
    ledger = json.loads(LEDGER.read_text())
    mutations: list[tuple[str, object]] = []
    for mutate in (
        lambda value: value["findings"].pop(),
        lambda value: value["findings"].reverse(),
        lambda value: value["findings"][0].update(severity="medium"),
        lambda value: value["findings"][0].update(status="closed"),
        lambda value: value["findings"][4].update(status="closed"),
        lambda value: value["findings"][1]["requirements"].pop(),
        lambda value: value.update(extra=False),
    ):
        candidate = copy.deepcopy(findings)
        mutate(candidate)
        mutations.append(("findings", candidate))
    for mutate in (
        lambda value: value["historical_findings"].reverse(),
        lambda value: value["new_findings"].pop(),
        lambda value: value.update(retained_hold="FINDING_099"),
        lambda value: value["historical_evidence"].pop(),
        lambda value: value["historical_evidence"].reverse(),
        lambda value: value["historical_evidence"][0].update(sha256="0" * 64),
        lambda value: value["supersession"].update(historical_bytes_mutable=True),
        lambda value: value["supersession"].update(v11_closure_required=False),
        lambda value: value.update(extra=False),
    ):
        candidate = copy.deepcopy(evidence)
        mutate(candidate)
        mutations.append(("evidence", candidate))
    for mutate in (
        lambda value: value["cases"].pop(),
        lambda value: value["cases"].reverse(),
        lambda value: value["cases"][0].update(finding="FINDING_097"),
        lambda value: value["cases"][0].update(expected="fixed_pass"),
        lambda value: value["cases"][1].update(path="missing"),
        lambda value: value.update(extra=False),
    ):
        candidate = copy.deepcopy(reproductions)
        mutate(candidate)
        mutations.append(("reproductions", candidate))
    mutations.append(("plan", PLAN.read_text().replace("`step_1315`–`step_1320`", "`step_1315`–`step_1321`", 1)))
    for mutate in (
        lambda value: value["cursor"].update(active_step="step_1329"),
        lambda value: value["cursor"].update(remaining_checkpoint_count=35),
        lambda value: value["findings"]["open"].reverse(),
        lambda value: value["active_checkpoint_scope"].reverse(),
        lambda value: value["predecessors"][0].update(candidate="0" * 40),
        lambda value: value["holds"].pop(),
        lambda value: value.update(extra=False),
    ):
        candidate = copy.deepcopy(ledger)
        mutate(candidate)
        mutations.append(("ledger", candidate))
    validators = {
        "findings": validate_findings,
        "evidence": validate_evidence,
        "reproductions": validate_reproductions,
        "plan": validate_plan,
        "ledger": validate_ledger,
    }
    for kind, candidate in mutations:
        try:
            validators[kind](candidate)
        except ValidationError:
            continue
        raise ValidationError(f"mutation:{kind}")
    return len(mutations)


def main() -> None:
    validate_repository()
    mutations = mutation_self_test()
    print("PASS: remediation v11 findings and evidence transition")
    print(f"- mutations={mutations}")
    print("- findings=4_open+1_held")
    print("- historical_artifacts=6")


if __name__ == "__main__":
    main()
