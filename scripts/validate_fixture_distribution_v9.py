#!/usr/bin/env python3
"""Validate the signed-v9 distribution and its byte-preserving v8 transition."""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
MANIFEST = ROOT / "fixtures/distribution/manifest_v9.json"
BASE = ROOT / "fixtures/distribution/manifest_v8.json"
PROFILES = {"checkpoint", "core", "malformed", "property", "resource"}
BASE_SIGNED_EVENTS_SHA256 = "50313da01a212e25fcab49e27882d5e9ed11110cfe1ab1b69d6771f83f6e8844"
V9_REQUIREMENTS = {
    "invalid_change_under_valid_noncanonical_control": ["NCRDT-BRANCH-003", "NCRDT-BRANCH-004", "NCRDT-DISPOSITION-004"],
    "pending_change_under_valid_noncanonical_control": ["NCRDT-BRANCH-003", "NCRDT-BRANCH-004"],
    "equivocation_excluded_change_under_valid_noncanonical_control": ["NCRDT-BRANCH-003", "NCRDT-BRANCH-004"],
    "noncanonical_bad_start_op_is_invalid": ["NCRDT-BRANCH-004"],
    "same_hash_valid_and_noncanonical_invalid_carriers": ["NCRDT-BRANCH-004", "NCRDT-DISPOSITION-004", "NCRDT-DISPOSITION-005"],
    "unrelated_control_flood_exact_budget": ["NCRDT-RESOURCE-011", "NCRDT-SCOPE-007"],
    "unrelated_control_flood_does_not_change_digest": ["NCRDT-SCOPE-007"],
    "change_carrier_mixed_outcomes": ["NCRDT-DISPOSITION-004", "NCRDT-DISPOSITION-005"],
    "change_carrier_event_order_stability": ["NCRDT-CONF-009", "NCRDT-DISPOSITION-005"],
}
REQUIRED_SCHEMAS = {
    "fixtures/schema/distribution.schema.v9.json",
    "fixtures/schema/fixture.schema.v8.json",
    "fixtures/schema/interop_attestation_v3.schema.json",
}


def digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def signed_event_set_sha256(entries: list[dict[str, object]]) -> str:
    value = hashlib.sha256()
    for entry in entries:
        identifier = str(entry["fixture_id"]).encode()
        value.update(len(identifier).to_bytes(4, "big"))
        value.update(identifier)
        scenario = json.loads((ROOT / str(entry["input_paths"][0])).read_text())
        for raw in scenario["raw_events"]:
            encoding = raw["encoding"].encode()
            data = raw["data"].encode()
            value.update(len(encoding).to_bytes(4, "big"))
            value.update(encoding)
            value.update(len(data).to_bytes(8, "big"))
            value.update(data)
    return value.hexdigest()


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--allow-locked-transition", action="store_true")
    return parser.parse_args()


def fail(message: str) -> None:
    raise SystemExit(message)


def expected_report(entry: dict[str, object]) -> dict[str, object]:
    value = json.loads((ROOT / str(entry["expected_path"])).read_text(encoding="utf-8"))
    if not isinstance(value, dict):
        fail(f"expected report is not an object: {entry['fixture_id']}")
    return value


def disposition_count(report: dict[str, object], namespace: str, disposition: str) -> int:
    records = report.get("disposition_records", [])
    if not isinstance(records, list):
        return 0
    return sum(
        isinstance(record, dict)
        and record.get("namespace") == namespace
        and record.get("disposition") == disposition
        for record in records
    )


def validate_v9_semantics(current: dict[str, dict[str, object]]) -> None:
    invalid = expected_report(current["invalid_change_under_valid_noncanonical_control"])
    if len(invalid.get("invalid_changes", [])) != 1 or disposition_count(
        invalid, "event", "invalid"
    ) != 1:
        fail("invalid noncanonical change outcome is not exact")
    pending = expected_report(current["pending_change_under_valid_noncanonical_control"])
    if len(pending.get("pending_changes", [])) != 1 or disposition_count(
        pending, "event", "pending"
    ) != 1:
        fail("pending noncanonical change outcome is not exact")
    start_op = expected_report(current["noncanonical_bad_start_op_is_invalid"])
    if len(start_op.get("invalid_changes", [])) != 1 or disposition_count(
        start_op, "event", "invalid"
    ) != 1:
        fail("noncanonical bad start_op is not invalid")
    equivocation = expected_report(
        current["equivocation_excluded_change_under_valid_noncanonical_control"]
    )
    alert_types = {
        alert.get("type")
        for alert in equivocation.get("integrity_alerts", [])
        if isinstance(alert, dict)
    }
    if len(equivocation.get("excluded_changes", [])) != 2 or "device_equivocation" not in alert_types:
        fail("noncanonical branch equivocation evidence is incomplete")

    for identifier in (
        "same_hash_valid_and_noncanonical_invalid_carriers",
        "change_carrier_mixed_outcomes",
        "change_carrier_event_order_stability",
    ):
        entry = current[identifier]
        report = expected_report(entry)
        accepted_hashes = set(report.get("accepted_changes", []))
        invalid_event_ids = {
            record.get("identifier")
            for record in report.get("disposition_records", [])
            if isinstance(record, dict)
            and record.get("namespace") == "event"
            and record.get("disposition") == "invalid"
        }
        scenario = json.loads((ROOT / str(entry["input_paths"][0])).read_text())
        invalid_claimed_hashes = set()
        for raw in scenario["raw_events"]:
            event = json.loads(raw["data"])
            if event.get("id") not in invalid_event_ids:
                continue
            invalid_claimed_hashes.update(
                tag[1]
                for tag in event.get("tags", [])
                if len(tag) == 2 and tag[0] == "x"
            )
        if not accepted_hashes or not invalid_event_ids:
            fail(f"mixed carrier records are incomplete: {identifier}")
        if identifier == "same_hash_valid_and_noncanonical_invalid_carriers" and not (
            accepted_hashes & invalid_claimed_hashes
        ):
            fail("invalid noncanonical carrier is not bound to an accepted semantic hash")

    mixed = expected_report(current["change_carrier_mixed_outcomes"])
    if {
        record.get("disposition")
        for record in mixed.get("disposition_records", [])
        if isinstance(record, dict) and record.get("namespace") == "event"
    } != {"accepted", "invalid", "pending"}:
        fail("mixed carrier Event outcomes are incomplete")
    stable = expected_report(current["change_carrier_event_order_stability"])
    for field in ("disposition_records", "dispositions_digest", "history_digest"):
        if stable.get(field) != mixed.get(field):
            fail(f"carrier order-stability fixture changed {field}")

    exact = expected_report(current["unrelated_control_flood_exact_budget"])
    digest = expected_report(current["unrelated_control_flood_does_not_change_digest"])
    if exact.get("completion") != "complete" or digest.get("completion") != "complete":
        fail("unrelated control flood did not complete")
    for field in ("canonical_controls", "disposition_records", "dispositions_digest", "history_digest"):
        if exact.get(field) != digest.get(field):
            fail(f"unrelated control flood changed target {field}")


def main() -> None:
    args = parse_args()
    manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
    base = json.loads(BASE.read_text(encoding="utf-8"))
    if manifest.get("distribution_schema") != "nostr_automerge.fixture_distribution.v9":
        fail("invalid fixture distribution schema")
    if manifest.get("distribution_id") != "draft_2026_08_signed_neutral_9":
        fail("invalid fixture distribution id")
    if manifest.get("supersedes") != "fixtures/distribution/manifest_v8.json":
        fail("fixture distribution does not supersede v8")
    if manifest.get("base_manifest_sha256") != digest(BASE):
        fail("v8 base manifest identity is stale")
    if manifest.get("target_fixture_count") != 180:
        fail("signed-v9 target count is not exact")
    if manifest.get("requirements_sha256") != digest(ROOT / "spec/requirements.json"):
        fail("fixture distribution requirements identity is stale")
    for field, relative in (
        ("authority_sha256", "spec/NIP_DRAFT.md"),
        ("companion_sha256", "spec/NOSTR_AUTOMERGE_V1_SPEC.md"),
        ("conformance_sha256", "spec/CONFORMANCE.md"),
    ):
        if manifest.get(field) != digest(ROOT / relative):
            fail(f"fixture distribution {field} is stale")

    base_entries = {item["fixture_id"]: item for item in base["fixtures"]}
    entries = manifest.get("fixtures")
    if not isinstance(entries, list) or entries != sorted(
        entries, key=lambda item: item["fixture_id"].encode()
    ):
        fail("fixture entries are not canonically ordered")
    current = {item["fixture_id"]: item for item in entries}
    if len(current) != len(entries):
        fail("duplicate fixture id")
    if manifest.get("preserved_v8_fixture_count") != len(base_entries) or len(base_entries) != 171:
        fail("v8 fixture count is not preserved")
    for identifier, entry in base_entries.items():
        if current.get(identifier) != entry:
            fail(f"v8 fixture entry changed: {identifier}")
    base_files = {
        item["path"]: item["sha256"]
        for item in base["files"]
    }
    current_files = {item["path"]: item["sha256"] for item in manifest.get("files", [])}
    changed_reports = []
    preserved_entries = [current[identifier] for identifier in sorted(base_entries, key=str.encode)]
    signed_events_sha256 = signed_event_set_sha256(preserved_entries)
    if (
        signed_events_sha256 != BASE_SIGNED_EVENTS_SHA256
        or manifest.get("preserved_v8_signed_events_sha256") != signed_events_sha256
    ):
        fail("v8 signed event set changed")
    for identifier, entry in base_entries.items():
        if current_files.get(entry["expected_path"]) != base_files.get(entry["expected_path"]):
            changed_reports.append(identifier)
    changed_reports.sort(key=str.encode)
    if manifest.get("intentional_v8_report_changes") != changed_reports:
        fail("intentional v8 report-change inventory is stale")
    if not REQUIRED_SCHEMAS.issubset(current_files):
        fail("signed-v9 schema set is incomplete")
    for relative, checksum in current_files.items():
        path = ROOT / relative
        if not path.is_file() or digest(path) != checksum:
            fail(f"missing or stale distribution file: {relative}")

    if manifest.get("v9_fixtures") != list(V9_REQUIREMENTS):
        fail("signed-v9 fixture inventory changed")
    missing = sorted(set(V9_REQUIREMENTS) - set(current), key=str.encode)
    if manifest.get("missing_v8_fixtures") != [] or manifest.get("missing_v9_fixtures") != missing:
        fail("signed-v9 missing fixture inventory is stale")
    complete = manifest.get("complete") is True
    if complete:
        if manifest.get("status") != "canonical_signed_neutral_corpus":
            fail("complete signed-v9 status is invalid")
        if len(entries) != 180 or missing:
            fail("signed-v9 distribution is incomplete")
    elif not args.allow_locked_transition:
        fail("signed-v9 distribution remains in a locked transition")
    elif (
        manifest.get("status") != "locked_transition"
        or len(entries) != 171
        or set(missing) != set(V9_REQUIREMENTS)
    ):
        fail("invalid signed-v9 locked transition")

    assigned = [identifier for values in manifest.get("profiles", {}).values() for identifier in values]
    if set(manifest.get("profiles", {})) != PROFILES or len(assigned) != len(set(assigned)):
        fail("signed-v9 profiles are invalid")
    if set(assigned) != set(current):
        fail("signed-v9 profiles do not cover the distribution")
    known_requirements = {
        row["id"]
        for row in json.loads((ROOT / "spec/requirements.json").read_text())["requirements"]
    }
    for identifier in set(V9_REQUIREMENTS) & set(current):
        entry = current[identifier]
        if entry.get("requirements") != V9_REQUIREMENTS[identifier]:
            fail(f"incorrect remediation-v8 requirements: {identifier}")
        if not set(entry["requirements"]).issubset(known_requirements):
            fail(f"unknown remediation-v8 requirement: {identifier}")
        scenario = json.loads((ROOT / entry["input_paths"][0]).read_text())
        if scenario.get("scenario_schema") != "nostr_automerge.signed_scenario.v2":
            fail(f"fixture is not a signed scenario: {identifier}")
        if scenario.get("requirements") != entry["requirements"]:
            fail(f"scenario requirements differ from metadata: {identifier}")
        if any(key in scenario for key in ("operations", "valid", "selected", "accepted")):
            fail(f"fixture contains abstract protocol truth: {identifier}")
    if complete:
        validate_v9_semantics(current)
    state = "complete" if complete else "locked transition"
    print(f"PASS: fixture distribution v9 {state} ({len(entries)} signed fixtures)")
    print("- preserved_v8_fixtures=171")
    print(f"- preserved_v8_signed_events_sha256={signed_events_sha256}")
    print(f"- intentional_v8_report_changes={len(changed_reports)}")
    print(f"- missing_v9_fixtures={len(missing)}")


if __name__ == "__main__":
    main()
