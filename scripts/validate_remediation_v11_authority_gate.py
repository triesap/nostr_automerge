#!/usr/bin/env python3
"""Validate the closed public specification-authority gate for remediation v11."""

from __future__ import annotations

import copy
import hashlib
import json
import pathlib
import subprocess
from typing import Any

import validate_distribution_v12 as distribution_v12


ROOT = pathlib.Path(__file__).resolve().parents[1]
REPORT = ROOT / "reports/remediation_v11_authority_gate.json"
SCHEMA = ROOT / "tools/validation/remediation_v11_authority_gate.schema.json"
REPORT_SHA256 = "53cbb6a26371001fcb0d2184f61194ce3244fb72fd91c8f9520943c336ec464f"
SCHEMA_SHA256 = "bb5e97c0d8c4992d2d23734306e2fadfdaac7e667dc2ee75ff2dde624fe75d77"
RESULT_IDENTITY = "b72dc33803f50cc0e7db89a595652cba1fdd79f6bb17a13e739278cf6ee99f14"
CANDIDATES = (
    ("step_1340", "83682d9c4e54c8ec7f98f7a1894b77bddbebde91"),
    ("step_1341", "3b0aed3d218cbc0fcec67676532b0860a06b3b13"),
    ("step_1342", "65acc5b80eb7264385be28bac62d8f94cf59f81a"),
    ("step_1343", "188e0ee2be355f62024e9cdb709ddece30424445"),
    ("step_1344", "18d2555d81a3d3c656c3fd81cfdbd1b213826914"),
)
SOURCES = (
    ("spec/NIP_DRAFT.md", "8262bf32cb70b7c0e46210441120652e52504fb73839641ac19dddfed840acf8"),
    ("spec/companion_authority_v10.json", "0e30fe422176b69b3cf7e3e68500e60bc4038823a84ba8757336ddc286788c94"),
    ("spec/API_CONTRACTS.md", "ce7f2992292b2f5159ff25dc555b29265fea0ec475d39fc65fc60344b76ca37a"),
    ("spec/requirements.json", "840822a1acf171c887b9a9aba79ddf159ffcd9c5d7a74bd74d7e0bac5c6161f4"),
    ("spec/requirements_applicability.json", "e473e8484ad0ccfb7d917485a0d7ab0d5bd2b98aa120ecaec588e7bacd19de28"),
    ("spec/authority_transition_v10.json", "fafee33c8d796ee8a9731cd507980dce937de23b9d1fe9d5cde6416fbc20af35"),
    ("docs/adr/adr_0072_metered_persistent_state.md", "7cbe4b80c950ac756147047d0c414f90eb8099779c225bbb3f46704b6b9f50ec"),
    ("docs/adr/adr_0073_no_post_stop_target_work.md", "a1e529d20f75254313e8c49bb5b5b4e575c118d4f878f3cf89aff3a5b3e8ec20"),
    ("docs/adr/adr_0074_unsupported_event_only_identity.md", "eacd506ed130d39b3c72ac61a0ea29b328209abc886b3c8d848723449398140c"),
    ("docs/adr/adr_0075_bounded_persistent_teardown.md", "7c4ae5365a3459f5d38574f1e9ebbe38c56b2d3b3b496f5d71b6c0d8cf1c2542"),
)
EVIDENCE = (
    ("persistent_state_core", "e540248bab985856d9aba407758ed1343c3c0e039f81347d29e4909abdecf695"),
    ("persistent_state_integration", "d5f7feb42dba21f079cbbcbf7b200cb84f2126dd851e51a0240de63b8eb0b55d"),
    ("target_work_accounting", "e15dca3958e9c9cf98da585c5a60135e4b3c9d8b59ddec9c0e3ef068615948ae"),
    ("persistent_ownership", "10235d1eac0b09a2b22ba70959a47a06478a08f595b31c9f843bb9fb41dcc67f"),
)
HOLDS = (
    "external_assurance",
    "event_kind_allocation",
    "nip_submission",
    "production_qualification",
    "publication",
    "release",
    "remote_mutation",
)


class AuthorityGateError(RuntimeError):
    """The v11 authority gate is stale or open shaped."""


def require(condition: bool, diagnostic: str) -> None:
    if not condition:
        raise AuthorityGateError(diagnostic)


def sha256(path: pathlib.Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def canonical(value: Any) -> bytes:
    return json.dumps(value, ensure_ascii=False, sort_keys=True, separators=(",", ":")).encode()


def require_record(value: object, keys: tuple[str, ...], diagnostic: str) -> dict[str, Any]:
    require(type(value) is dict and tuple(value) == keys, f"{diagnostic}:keys")
    return value


def validate_report(value: object) -> None:
    record = require_record(
        value,
        (
            "schema", "status", "checkpoint", "revision", "candidates", "authority",
            "requirements", "distribution", "evidence", "holds", "release_claimed",
            "remote_actions_performed", "result", "result_identity_sha256",
        ),
        "report",
    )
    require(
        (record["schema"], record["status"], record["checkpoint"], record["revision"], record["result"])
        == ("nostr_automerge.remediation_v11_authority_gate.v1", "pass", "step_1345", "draft_2026_08", "pass"),
        "report:identity",
    )
    require(
        tuple((row.get("step"), row.get("candidate")) for row in record["candidates"])
        == CANDIDATES,
        "report:candidates",
    )
    require(
        tuple((row.get("path"), row.get("sha256")) for row in record["authority"])
        == SOURCES,
        "report:authority",
    )
    require(
        record["requirements"]
        == {
            "count": 152,
            "appended": [
                "NCRDT-RESOURCE-015",
                "NCRDT-RESOURCE-016",
                "NCRDT-VERSION-003",
                "NCRDT-OWNERSHIP-001",
            ],
        },
        "report:requirements",
    )
    require(
        record["distribution"]
        == {
            "base_manifest_sha256": "db247fa3e6891e850f32ed9b00fb08cfd78d30c9eb88ea36a00bd22dabb63f5a",
            "transition_sha256": "88026826cb34db3c0f68376cd29239243f407e2dee01b664d733bdce9165e705",
            "schema_sha256": "5e6d924ffd3cb1980e60698696d9c832d70523bf43934c857cd07054546cf37c",
            "manifest_sha256": "4f19b95fa3a3fc3a1606391eba4636734d8537d9fcdc78531868420b21e7bca5",
            "current_fixture_count": 193,
            "target_fixture_count": 198,
            "planned_fixture_count": 5,
            "complete": False,
        },
        "report:distribution",
    )
    require(
        tuple((row.get("class"), row.get("sha256")) for row in record["evidence"])
        == EVIDENCE,
        "report:evidence",
    )
    require(tuple(record["holds"]) == HOLDS, "report:holds")
    require(record["release_claimed"] is False and record["remote_actions_performed"] is False, "report:held")
    projection = dict(record)
    identity = projection.pop("result_identity_sha256")
    require(hashlib.sha256(canonical(projection)).hexdigest() == identity == RESULT_IDENTITY, "report:result_identity")


def validate_schema(value: object) -> None:
    schema = require_record(
        value,
        ("$schema", "$id", "title", "type", "additionalProperties", "required", "properties", "$defs"),
        "schema",
    )
    required = (
        "schema", "status", "checkpoint", "revision", "candidates", "authority",
        "requirements", "distribution", "evidence", "holds", "release_claimed",
        "remote_actions_performed", "result", "result_identity_sha256",
    )
    require(schema["type"] == "object" and schema["additionalProperties"] is False, "schema:closed")
    require(tuple(schema["required"]) == required and tuple(schema["properties"]) == required, "schema:shape")
    require(schema["properties"]["candidates"]["minItems"] == schema["properties"]["candidates"]["maxItems"] == 5, "schema:candidates")
    require(schema["properties"]["authority"]["minItems"] == schema["properties"]["authority"]["maxItems"] == 10, "schema:authority")
    require(schema["properties"]["holds"]["minItems"] == schema["properties"]["holds"]["maxItems"] == 7, "schema:holds")


def validate_repository() -> None:
    report = json.loads(REPORT.read_text())
    schema = json.loads(SCHEMA.read_text())
    validate_report(report)
    validate_schema(schema)
    require(sha256(REPORT) == REPORT_SHA256 and sha256(SCHEMA) == SCHEMA_SHA256, "repository:record_hash")
    for path, expected in SOURCES:
        require(sha256(ROOT / path) == expected, f"repository:source:{path}")
    for (_, parent), (_, child) in zip(CANDIDATES, CANDIDATES[1:]):
        actual = subprocess.run(
            ("git", "rev-parse", f"{child}^"), cwd=ROOT, check=True, capture_output=True, text=True
        ).stdout.strip()
        require(actual == parent, f"repository:parent:{child}")
    distribution_v12.main()


def mutation_self_test(report: dict[str, Any], schema: dict[str, Any]) -> int:
    report_mutations = []
    for mutate in (
        lambda value: value.update(extra=False),
        lambda value: value["candidates"].pop(),
        lambda value: value["candidates"].reverse(),
        lambda value: value["authority"].pop(),
        lambda value: value["authority"].reverse(),
        lambda value: value["requirements"].update(count=151),
        lambda value: value["requirements"]["appended"].reverse(),
        lambda value: value["distribution"].update(target_fixture_count=197),
        lambda value: value["distribution"].update(manifest_sha256="0" * 64),
        lambda value: value["evidence"].reverse(),
        lambda value: value["holds"].pop(),
        lambda value: value.update(release_claimed=True),
        lambda value: value.update(remote_actions_performed=True),
        lambda value: value.update(result_identity_sha256="0" * 64),
    ):
        candidate = copy.deepcopy(report)
        mutate(candidate)
        report_mutations.append(candidate)
    schema_mutations = []
    for mutate in (
        lambda value: value.update(extra=False),
        lambda value: value.update(additionalProperties=True),
        lambda value: value["properties"]["holds"].update(maxItems=8),
    ):
        candidate = copy.deepcopy(schema)
        mutate(candidate)
        schema_mutations.append(candidate)
    for candidate in report_mutations:
        try:
            validate_report(candidate)
        except AuthorityGateError:
            continue
        raise AuthorityGateError("mutation:report")
    for candidate in schema_mutations:
        try:
            validate_schema(candidate)
        except AuthorityGateError:
            continue
        raise AuthorityGateError("mutation:schema")
    return len(report_mutations) + len(schema_mutations)


def main() -> int:
    validate_repository()
    report = json.loads(REPORT.read_text())
    schema = json.loads(SCHEMA.read_text())
    mutations = mutation_self_test(report, schema)
    print("PASS: remediation v11 public authority gate")
    print(f"- candidates={len(CANDIDATES)}")
    print("- requirements=152")
    print("- distribution=193_of_198")
    print(f"- negative_mutations={mutations}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
