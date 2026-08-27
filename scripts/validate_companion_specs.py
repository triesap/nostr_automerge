#!/usr/bin/env python3
"""Validate imported specs and the staged v10 companion authority."""

from __future__ import annotations

import copy
import hashlib
import json
from pathlib import Path
from typing import Any


ROOT = Path(__file__).resolve().parents[1]
AUTHORITY_PATH = "spec/companion_authority_v10.json"
TRANSITION_PATH = "spec/authority_transition_v10.json"
STAGES = (
    "transition_installed",
    "companion_authority_installed",
    "requirements_appended",
    "checkpoint_expectations_corrected",
    "distribution_locked",
    "checkpoint_control_fixtures_added",
    "carrier_independence_fixtures_added",
    "interruption_fixtures_added",
    "target_work_fixtures_added",
    "distribution_complete",
)
COMPANION_STAGE = STAGES.index("companion_authority_installed")
NIP_SHA256 = "8262bf32cb70b7c0e46210441120652e52504fb73839641ac19dddfed840acf8"

REQUIRED = (
    "ACCEPTANCE_CRITERIA.md",
    "API_CONTRACTS.md",
    "ARCHITECTURE.md",
    "AUTOMERGE_PROFILE.md",
    "CHECKPOINT_PROFILE.md",
    "CONFORMANCE.md",
    "CONTROL_AND_AUTHORIZATION.md",
    "DATA_MODEL.md",
    "FUTURE_FARM_WORKSPACES_CONTEXT.md",
    "NIP_DRAFT.md",
    "NIP_PR_DESCRIPTION.md",
    "NORMATIVE_REQUIREMENTS.md",
    "NOSTR_AUTOMERGE_V1_SPEC.md",
    "OUT_OF_SCOPE_AND_FUTURE_WORK.md",
    "PRODUCT_SPEC.md",
    "SECURITY.md",
    "VERSIONING_AND_COMPATIBILITY.md",
    "WIRE_FORMAT.md",
)
ORIGINAL_ADAPTED = {
    "ACCEPTANCE_CRITERIA.md",
    "AUTOMERGE_PROFILE.md",
    "CONFORMANCE.md",
    "NIP_DRAFT.md",
    "NOSTR_AUTOMERGE_V1_SPEC.md",
}
DOCUMENT_ORDER = (
    "spec/NOSTR_AUTOMERGE_V1_SPEC.md",
    "spec/API_CONTRACTS.md",
    "spec/CONFORMANCE.md",
    "spec/CHECKPOINT_PROFILE.md",
    "spec/draft_limits.md",
    "spec/REPORT_CONTRACT.md",
)
BASELINE_HASHES: dict[str, str | None] = {
    "spec/NOSTR_AUTOMERGE_V1_SPEC.md": "58177c31eb06086d76297bbb0fc15343a8e34c15499d6e03636c63df7604bb10",
    "spec/API_CONTRACTS.md": "1114079b3f90a04895947e4b25a720a13fe8e28380cf824e2935a3fa373b8593",
    "spec/CONFORMANCE.md": "9c4118c3b67c6268ed484a5457270c0dc46a88223da571aa519beb655ec7908d",
    "spec/CHECKPOINT_PROFILE.md": "6e6c4228eba2cf90b2cf16f4bd84a384a531cbaf7fba462a59661666a9b8da76",
    "spec/draft_limits.md": "8294556e12fd1fb7f713f732be48a5812fca5cf422e6098554efdf5559425b70",
    "spec/REPORT_CONTRACT.md": None,
}
# This independent checkpoint identity prevents a document and its manifest
# declaration from being changed together. A later authorized authority edit
# must update the document, manifest, and this reviewed pin in one checkpoint.
INSTALLED_LIVE_HASHES = {
    "spec/NOSTR_AUTOMERGE_V1_SPEC.md": "a81ad7f3e5cc7e386a9313f6d5355afc1ec95757a5c9a4051ea94b79eafeceb0",
    "spec/API_CONTRACTS.md": "ce7f2992292b2f5159ff25dc555b29265fea0ec475d39fc65fc60344b76ca37a",
    "spec/CONFORMANCE.md": "d8439031d76caaeb2dd8a2af8ba2d2eed7843fb1634f90f74e5f5c6d85d8d32e",
    "spec/CHECKPOINT_PROFILE.md": "85bac3c3aea268fa0bfc559b93280a36c571ee6a79c2d588a30755a6a2588886",
    "spec/draft_limits.md": "c482c9906b0714e0b2f359703aa3b46a6ea5aea7f583adf67886a99b2cc135d9",
    "spec/REPORT_CONTRACT.md": "636bd1ff32673a00dc0f41440bde61f2b0f8d86f853a7feaaf119de1ff2ce189",
}
V11_LIVE_HASHES = {
    "spec/ARCHITECTURE.md": "5c01f373a939ed3b9f4d11a5b19988cec1abd65a61f61313bc37160acf41f878",
    "spec/SECURITY.md": "3010e3a8bd4597141f2926e3c16d64fb0623e6e5d6ea517ad9815ab7639ce056",
}
HEADING_BINDINGS = {
    "spec/CHECKPOINT_PROFILE.md": (
        "Checkpoint control resolution precedence",
        "Recoverable checkpoint control states",
    ),
    "spec/REPORT_CONTRACT.md": (
        "Independent change-carrier outcomes",
        "No-progress interruption reports",
        "Two-tier finalization reservation",
        "Target-local deterministic work",
        "Unsupported change identity",
    ),
    "spec/CONFORMANCE.md": (
        "Signed conformance v10",
        "Semantically exact proof catalog",
    ),
}
CLAUSE_BINDINGS = {
    "NCRDT-CPAUTH-001": (
        "spec/CHECKPOINT_PROFILE.md",
        "A checkpoint descriptor control reference MUST be resolved and authorized before chunk assembly, carrier-history coverage, accepted-at-control lookup, snapshot loading, or history verification is attempted.",
    ),
    "NCRDT-CPAUTH-002": (
        "spec/CHECKPOINT_PROFILE.md",
        "Only a missing or statefully pending referenced control may produce a pending checkpoint descriptor. A noncanonical, wrong-kind, wrong-coordinate, statically invalid, dynamically invalid, unsupported, or role-denied control MUST produce an invalid draft-v1 descriptor outcome.",
    ),
    "NCRDT-DISPOSITION-006": (
        "spec/REPORT_CONTRACT.md",
        "A change-carrier Event disposition MUST be derived from that carrier claim and its referenced control or branch. An aggregate ChangeHash disposition MUST NOT convert a carrier with a known-invalid reference into accepted, pending, or excluded.",
    ),
    "NCRDT-INTERRUPT-001": (
        "spec/REPORT_CONTRACT.md",
        "A public evaluation that ends in `budget_exhausted` or `cancelled` MUST return a constant-size no-progress report. It MUST NOT expose canonical controls, protocol dispositions, evidence, checkpoints, an available or resolved manifest, integrity alerts, heads, or materialized document state.",
    ),
    "NCRDT-RESOURCE-013": (
        "spec/REPORT_CONTRACT.md",
        "The evaluator MUST reserve fixed no-progress fallback capacity separately from complete-report capacity. Actual complete-report passes are consumed immediately before their work; on interruption, complete-report capacity is forfeited and only fixed fallback passes are consumed.",
    ),
    "NCRDT-RESOURCE-014": (
        "spec/REPORT_CONTRACT.md",
        "Every target-proportional preparation collection, raw-byte copy or shared-reference operation, branch memo traversal, canonical derivation pass, alert copy, and disposition copy MUST be bounded, charged, cancellation-aware, or eliminated.",
    ),
    "NCRDT-VERSION-002": (
        "spec/REPORT_CONTRACT.md",
        "An unsupported change carrier whose canonical Change Chunk and ChangeHash were not verified receives only an Event `unsupported_revision` outcome. Its unverified `x` tag MUST NOT create a semantic ChangeHash disposition in draft v1.",
    ),
    "NCRDT-CONF-010": (
        "spec/CONFORMANCE.md",
        "The checksum-bound signed v10 distribution MUST contain exactly 192 scenarios, including the corrected checkpoint expectations and new carrier, interruption, and work-boundary cases. Both implementations MUST execute all scenarios twice and under all eight delivery permutations with byte-identical canonical output and deliberate mismatch rejection.",
    ),
    "NCRDT-EVIDENCE-006": (
        "spec/CONFORMANCE.md",
        "Every passing requirement row MUST bind to a semantically matching exact signed fixture or named assertion through a validated proof catalog. Broad command-only proof, unrelated assertion categories, stale expectations, and missing opaque TypeScript evidence identifiers MUST be rejected.",
    ),
}
HELD_CLAIMS = (
    "candidate_closure",
    "nip_conformance",
    "publication",
    "release",
    "deployment",
    "production_qualification",
)
FORBIDDEN_CONTRADICTIONS = (
    "candidate contract overrides the nip",
    "incomplete report may preserve canonical",
    "noncanonical control may produce a pending checkpoint descriptor",
    "fallback and complete ledgers may borrow",
    "unverified `x` tag creates a semantic changehash",
    "all-unsupported claims are unsupported",
    "only unsupported claims is `unsupported_revision`",
    "only unsupported carriers is `unsupported_revision`",
)


class CompanionError(ValueError):
    """One companion-authority invariant failed."""


def require(condition: bool, diagnostic: str) -> None:
    """Raise a stable failure when *condition* is false."""

    if not condition:
        raise CompanionError(diagnostic)


def sha256_bytes(data: bytes) -> str:
    """Return the lowercase SHA-256 of *data*."""

    return hashlib.sha256(data).hexdigest()


def sha256(path: Path) -> str:
    """Return the lowercase SHA-256 of *path*."""

    return sha256_bytes(path.read_bytes())


def load_json(path: Path) -> dict[str, Any]:
    """Load a JSON object."""

    value = json.loads(path.read_text(encoding="utf-8"))
    if not isinstance(value, dict):
        raise CompanionError(f"expected_object:{path.relative_to(ROOT)}")
    return value


def normalized(text: str) -> str:
    """Collapse Markdown wrapping without weakening exact token content."""

    return " ".join(text.split())


def authority_documents(authority: dict[str, Any]) -> dict[str, dict[str, Any]]:
    """Return the closed ordered authority document inventory."""

    documents = authority.get("documents")
    require(isinstance(documents, list), "authority_documents")
    paths = [item.get("path") for item in documents if isinstance(item, dict)]
    require(len(paths) == len(documents), "authority_document_shape")
    require(paths == list(DOCUMENT_ORDER), "authority_document_order")
    require(len(paths) == len(set(paths)), "authority_document_duplicates")
    return {str(item["path"]): item for item in documents}


def validate_post_import_authority(
    stage: str,
    authority: dict[str, Any],
    documents: dict[str, bytes],
) -> None:
    """Validate stage, exact hashes, anchors, clauses, and held claims."""

    require(stage in STAGES, "transition_stage")
    require(STAGES.index(stage) >= COMPANION_STAGE, "authority_before_effective_stage")
    require(
        set(authority)
        == {
            "schema",
            "status",
            "effective_stage",
            "protocol_revision",
            "nip_authority",
            "documents",
            "held_claims",
        },
        "authority_keys",
    )
    require(
        authority.get("schema") == "nostr_automerge.companion_authority.v10",
        "authority_schema",
    )
    require(authority.get("status") == "staged_local_candidate", "authority_status")
    require(
        authority.get("effective_stage") == "companion_authority_installed",
        "authority_effective_stage",
    )
    require(authority.get("protocol_revision") == "draft_2026_08", "authority_revision")

    nip = authority.get("nip_authority")
    require(
        nip
        == {
            "path": "spec/NIP_DRAFT.md",
            "sha256": NIP_SHA256,
            "status": "controlling_normative_authority_reconciled",
        },
        "nip_authority",
    )
    require(
        sha256_bytes(documents.get("spec/NIP_DRAFT.md", b"")) == NIP_SHA256,
        "nip_changed",
    )
    require(authority.get("held_claims") == list(HELD_CLAIMS), "held_claims")

    by_path = authority_documents(authority)
    require(
        tuple(INSTALLED_LIVE_HASHES) == DOCUMENT_ORDER,
        "installed_identity_inventory",
    )
    for path in DOCUMENT_ORDER:
        item = by_path[path]
        require(set(item) == {"path", "baseline_sha256", "live_sha256"}, f"document_keys:{path}")
        require(item.get("baseline_sha256") == BASELINE_HASHES[path], f"baseline_hash:{path}")
        require(
            item.get("live_sha256") == INSTALLED_LIVE_HASHES[path],
            f"installed_identity_manifest:{path}",
        )
        data = documents.get(path)
        require(isinstance(data, bytes) and data, f"live_document:{path}")
        require(
            sha256_bytes(data) == INSTALLED_LIVE_HASHES[path],
            f"installed_identity_bytes:{path}",
        )
        if BASELINE_HASHES[path] is not None:
            require(item.get("live_sha256") != BASELINE_HASHES[path], f"document_not_advanced:{path}")

    decoded = {
        path: data.decode("utf-8", errors="strict") for path, data in documents.items()
    }
    for path, headings in HEADING_BINDINGS.items():
        text = decoded[path]
        for heading in headings:
            require(text.count(f"## {heading}\n") == 1, f"heading:{path}:{heading}")
    all_authority_text = "\n".join(decoded[path] for path in DOCUMENT_ORDER)
    for identifier, (path, clause) in CLAUSE_BINDINGS.items():
        require(all_authority_text.count(f"`{identifier}`") == 1, f"identifier:{identifier}")
        require(normalized(clause) in normalized(decoded[path]), f"clause:{identifier}")

    report = normalized(decoded["spec/REPORT_CONTRACT.md"])
    for required in (
        "canonical manifest availability is `missing`",
        "canonical empty-input history and dispositions digests",
        "Every canonical or target-sized collection is empty",
    ):
        require(normalized(required) in report, "no_progress_shape")
    companion = normalized(decoded["spec/NOSTR_AUTOMERGE_V1_SPEC.md"])
    for required in (
        "They do not edit or override the unchanged repository-local NIP draft.",
        "Candidate closure, NIP conformance, publication, release, deployment, and production qualification remain held",
        "does not become current until `distribution_complete`",
    ):
        require(normalized(required) in companion, "companion_boundary")
    api = normalized(decoded["spec/API_CONTRACTS.md"])
    require("Result<EvaluationReport, EvaluationError>" in api, "api_typed_error")
    require("constant-size, revision-bound no-progress shape" in api, "api_no_progress")
    for required in (
        "The report preserves two independent identity layers.",
        "Aggregate semantic reduction never rewrites a known-invalid carrier Event.",
        "Its unverified `x` tag does not create a semantic disposition",
    ):
        require(normalized(required) in api, "api_carrier_identity")
    for required in (
        "Every verified semantic change yields exactly one represented hash outcome.",
        "remains Event-only evidence and does not enter semantic hash reduction",
        "supplies no semantic reducer input",
    ):
        require(normalized(required) in companion, "companion_carrier_identity")
    limits = normalized(decoded["spec/draft_limits.md"])
    require("fixed no-progress fallback ledger" in limits, "resource_fallback")
    require("Unrelated-coordinate evidence cannot change target work consumption" in limits, "resource_scope")

    lowered = normalized(all_authority_text).lower()
    for contradiction in FORBIDDEN_CONTRADICTIONS:
        require(contradiction not in lowered, f"contradiction:{contradiction}")


def mutation_self_test(
    stage: str, authority: dict[str, Any], documents: dict[str, bytes]
) -> int:
    """Prove coordinated weakening and authority drift fail closed."""

    require(STAGES.index(stage) >= COMPANION_STAGE, "mutation_stage")
    mutations: list[tuple[str, str, dict[str, Any], dict[str, bytes]]] = []
    mutations.append(("early_stage", "transition_installed", copy.deepcopy(authority), documents.copy()))

    def changed_document(name: str, old: str, new: str) -> tuple[dict[str, Any], dict[str, bytes]]:
        candidate_authority = copy.deepcopy(authority)
        candidate_documents = documents.copy()
        text = candidate_documents[name].decode("utf-8")
        require(old in text, f"mutation_source:{name}")
        candidate_documents[name] = text.replace(old, new, 1).encode("utf-8")
        for item in candidate_authority["documents"]:
            if item["path"] == name:
                item["live_sha256"] = sha256_bytes(candidate_documents[name])
        return candidate_authority, candidate_documents

    candidate, docs = changed_document(
        "spec/REPORT_CONTRACT.md",
        "Valid-carrier dominance applies only to the aggregate semantic outcome.",
        "Valid-carrier dominance applies only to the aggregate semantic outcome. A known-invalid carrier Event MAY be accepted when its aggregate ChangeHash is accepted.",
    )
    try:
        validate_post_import_authority(stage, candidate, docs)
    except CompanionError as error:
        require(
            str(error) == "installed_identity_manifest:spec/REPORT_CONTRACT.md",
            "coordinated_tamper_diagnostic",
        )
    else:
        raise CompanionError("mutation_survived:coordinated_semantic_tamper")
    caught = 1

    candidate, docs = changed_document(
        "spec/CHECKPOINT_PROFILE.md",
        "## Checkpoint control resolution precedence",
        "## Deferred checkpoint control resolution",
    )
    mutations.append(("missing_anchor", stage, candidate, docs))
    candidate, docs = changed_document(
        "spec/REPORT_CONTRACT.md",
        "MUST return a constant-size no-progress report",
        "MAY return a constant-size no-progress report",
    )
    mutations.append(("weakened_clause", stage, candidate, docs))
    candidate, docs = changed_document(
        "spec/REPORT_CONTRACT.md",
        "Internal partial\nevaluator state is never a public `EvaluationReport`.",
        "Internal partial\nevaluator state is never a public `EvaluationReport`. Incomplete report may preserve canonical state.",
    )
    mutations.append(("contradictory_clause", stage, candidate, docs))
    candidate, docs = changed_document(
        "spec/REPORT_CONTRACT.md",
        "canonical manifest availability is\n`missing`",
        "canonical manifest availability is\nimplementation-defined",
    )
    mutations.append(("missing_manifest_value", stage, candidate, docs))
    candidate, docs = changed_document(
        "spec/REPORT_CONTRACT.md",
        "canonical empty-input history and\ndispositions digests",
        "implementation-selected history and\ndispositions digests",
    )
    mutations.append(("weakened_empty_digests", stage, candidate, docs))
    candidate, docs = changed_document("spec/NIP_DRAFT.md", "NIP-XX", "NIP-YY")
    candidate["nip_authority"]["sha256"] = sha256_bytes(docs["spec/NIP_DRAFT.md"])
    mutations.append(("coordinated_nip_mutation", stage, candidate, docs))

    wrong_stage = copy.deepcopy(authority)
    wrong_stage["effective_stage"] = "requirements_appended"
    mutations.append(("authority_stage_drift", stage, wrong_stage, documents.copy()))
    wrong_hash = copy.deepcopy(authority)
    wrong_hash["documents"][0]["live_sha256"] = "0" * 64
    mutations.append(("authority_hash_drift", stage, wrong_hash, documents.copy()))
    missing = copy.deepcopy(authority)
    missing["documents"].pop()
    mutations.append(("authority_document_missing", stage, missing, documents.copy()))
    extra = copy.deepcopy(authority)
    extra["documents"].append(copy.deepcopy(extra["documents"][0]))
    mutations.append(("authority_document_extra", stage, extra, documents.copy()))
    held = copy.deepcopy(authority)
    held["held_claims"].pop()
    mutations.append(("held_claim_removed", stage, held, documents.copy()))
    no_report = documents.copy()
    no_report.pop("spec/REPORT_CONTRACT.md")
    mutations.append(("report_contract_missing", stage, copy.deepcopy(authority), no_report))

    for name, mutation_stage, candidate_authority, candidate_documents in mutations:
        try:
            validate_post_import_authority(
                mutation_stage, candidate_authority, candidate_documents
            )
        except (CompanionError, UnicodeDecodeError):
            caught += 1
            continue
        raise CompanionError(f"mutation_survived:{name}")
    return caught


def main() -> int:
    """Validate companion completeness, provenance, staged authority, and sources."""

    transition = load_json(ROOT / TRANSITION_PATH)
    stage = transition.get("current_stage")
    require(isinstance(stage, str) and stage in STAGES, "current_stage")

    package = load_json(ROOT / "docs/provenance/source_package_manifest.json")
    source_hashes = {
        item["path"]: item["sha256"]
        for item in package.get("files", [])
        if isinstance(item, dict) and "path" in item and "sha256" in item
    }
    adaptation = load_json(ROOT / "docs/import_adaptation.json")
    adapted_hashes = {
        Path(item["path"]).name: item["target_sha256"]
        for item in adaptation.get("imported_files", [])
        if isinstance(item, dict) and item.get("adapted") is True
    }

    post_import = STAGES.index(stage) >= COMPANION_STAGE
    authority = load_json(ROOT / AUTHORITY_PATH) if post_import else {}
    authorized = authority_documents(authority) if post_import else {}
    for name in REQUIRED:
        path = ROOT / "spec" / name
        require(path.is_file() and path.stat().st_size > 0, f"missing_spec:{name}")
        source_digest = source_hashes.get(f"specs/{name}")
        require(isinstance(source_digest, str), f"source_manifest:{name}")
        original = adapted_hashes.get(name) if name in ORIGINAL_ADAPTED else source_digest
        relative = f"spec/{name}"
        expected = (
            NIP_SHA256
            if relative == "spec/NIP_DRAFT.md" and post_import
            else V11_LIVE_HASHES[relative]
            if relative in V11_LIVE_HASHES and post_import
            else authorized[relative]["live_sha256"]
            if relative in authorized
            else original
        )
        require(sha256(path) == expected, f"companion_spec_digest:{name}")

    documents = {
        path: (ROOT / path).read_bytes()
        for path in (*DOCUMENT_ORDER, "spec/NIP_DRAFT.md")
        if (ROOT / path).is_file()
    }
    if post_import:
        validate_post_import_authority(stage, authority, documents)
        mutations = mutation_self_test(stage, authority, documents)
    else:
        require(not (ROOT / "spec/REPORT_CONTRACT.md").exists(), "early_report_contract")
        mutations = 0

    requirements = load_json(ROOT / "spec/requirements.json")
    rows = requirements.get("requirements")
    require(isinstance(rows, list), "requirement_rows")
    for requirement in rows:
        require(isinstance(requirement, dict), "requirement_entry")
        source = requirement.get("source")
        require(isinstance(source, str), f"requirement_source:{requirement.get('id')}")
        require((ROOT / source).is_file(), f"requirement_source_path:{requirement.get('id')}")

    print("PASS: companion specification set")
    print(f"- required_specs={len(REQUIRED) + int(post_import)}")
    print(f"- imported_adapted_specs={len(ORIGINAL_ADAPTED)}")
    print(f"- post_import_authority={'active' if post_import else 'inactive'}")
    print(f"- authority_documents={len(DOCUMENT_ORDER) if post_import else 0}")
    print(f"- requirement_sources={len(rows)}")
    print(f"- negative_mutations={mutations}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
