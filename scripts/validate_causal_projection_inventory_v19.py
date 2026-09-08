#!/usr/bin/env python3
"""Derive and validate the v19 Rust causal-projection source inventory."""

from __future__ import annotations

import argparse
import copy
import hashlib
import json
import re
import subprocess
import sys
import textwrap
from pathlib import Path
from typing import Any

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "scripts"))
import validate_causal_projection_inventory_v18 as discovery  # noqa: E402

REPORT = ROOT / "reports/causal_projection_inventory_v19.json"
SCHEMA = ROOT / "tools/validation/causal_projection_inventory_v19.schema.json"
SOURCE = ROOT / "crates/nostr_automerge/src/graph/actor_state.rs"
SOURCE_PATH = "crates/nostr_automerge/src/graph/actor_state.rs"
SOURCE_CANDIDATE = "a8f8651c26a0ac7822e1e5d61f3aafa3fec53d1f"
AUTHORITY = "spec/causal_projection_contracts_v19.json"
TOP_FIELDS = [
    "schema", "status", "authority", "source_candidate", "source_path",
    "source_production_sha256", "row_contract", "rows", "counts",
    "derivation", "proof_event_model", "result_identity_sha256", "result",
]


class InventoryError(RuntimeError):
    pass


def require(condition: bool, code: str) -> None:
    if not condition:
        raise InventoryError(code)


def canonical(value: object) -> bytes:
    return json.dumps(value, ensure_ascii=True, separators=(",", ":"), sort_keys=True).encode()


def candidate_source() -> str:
    completed = subprocess.run(
        ["git", "show", f"{SOURCE_CANDIDATE}:{SOURCE_PATH}"],
        cwd=ROOT,
        capture_output=True,
        text=True,
        check=False,
    )
    require(completed.returncode == 0, "SOURCE_CANDIDATE_MISSING")
    return completed.stdout


def assurance_normalized_current(source: str) -> str:
    """Remove only the v19 test-only direct-target assurance layer."""

    sites = [
        "ActorStateRead", "PredecessorCandidateRead", "ActorIdentityDecision",
        "SequenceRelationDecision", "StoredCounterRead", "ExpectedStartComparison",
        "CheckedAdvance", "CandidateKindComparison", "MemberCountRead",
    ]
    for site in sites:
        match = re.search(
            rf'#\[cfg\(test\)\]\n[ \t]+v19_record_direct_target\("{site}"\);',
            source,
        )
        require(match is not None, "DIRECT_PROBE_MISSING:" + site)
        marker = match.group()
        marker_start = match.start()
        opening = source.rfind("|| {", 0, marker_start)
        require(opening >= 0, "DIRECT_PROBE_CLOSURE:" + site)
        depth = 0
        closing = None
        for index in range(opening + 3, len(source)):
            if source[index] == "{":
                depth += 1
            elif source[index] == "}":
                depth -= 1
                if depth == 0:
                    closing = index
                    break
        require(closing is not None, "DIRECT_PROBE_CLOSE:" + site)
        inner = source[opening + 4:closing]
        require(inner.count(marker) == 1, "DIRECT_PROBE_SCOPE:" + site)
        inner = inner.replace(marker, "", 1)
        normalized = textwrap.dedent(
            "\n".join(line for line in inner.splitlines() if line.strip())
        )
        line_start = source.rfind("\n", 0, opening) + 1
        indent = source[line_start:opening]
        normalized = normalized.replace("\n", "\n" + indent)
        source = source[:opening] + "|| " + normalized + source[closing + 1:]

    guard_start = source.find("#[cfg(test)]\ntype V19DirectTargetSink")
    guard_end = source.find("/// Immutable accepted-closure facts", guard_start)
    require(guard_start >= 0 and guard_end > guard_start, "DIRECT_PROBE_GUARD")
    source = source[:guard_start] + source[guard_end:]
    return discovery.production(source)


def derive_rows(source: str) -> list[dict[str, Any]]:
    discovery.SOURCE_CANDIDATE = SOURCE_CANDIDATE
    rows = discovery.derive_rows(source)
    for row in rows:
        row["source_candidate"] = SOURCE_CANDIDATE
    return rows


def expected_report(committed_source: str) -> dict[str, Any]:
    production = discovery.production(committed_source)
    rows = derive_rows(production)
    phases = {
        phase: sum(row["phase"] == phase for row in rows)
        for _, _, phase, *_ in discovery.PHASES
    }
    report: dict[str, Any] = {
        "schema": "nostr_automerge.causal_projection_inventory.v19.v1",
        "status": "provisional_source_derived",
        "authority": AUTHORITY,
        "source_candidate": SOURCE_CANDIDATE,
        "source_path": SOURCE_PATH,
        "source_production_sha256": hashlib.sha256(production.encode()).hexdigest(),
        "row_contract": discovery.ROW_FIELDS,
        "rows": rows,
        "counts": {"rows": len(rows), "phases": phases},
        "derivation": {
            "method": "committed_lexical_descriptor_registry",
            "planned_counts": False,
            "comment_and_string_tokens_ignored": True,
            "proof_evidence_terminal": False,
        },
        "proof_event_model": {
            "events": [
                "ChargeAttempt", "ChargeAccepted", "TargetDispatched",
                "TargetReturned", "CompletionObserved", "PublicationCompleted",
            ],
            "target_source": "test_only_thread_local_probe",
            "completion_source": "production_completion_observer",
        },
        "result_identity_sha256": "",
        "result": "pass",
    }
    identity = {key: value for key, value in report.items() if key != "result_identity_sha256"}
    report["result_identity_sha256"] = hashlib.sha256(canonical(identity)).hexdigest()
    return report


def validate(report: Any, schema: Any, committed_source: str, current_source: str) -> None:
    expected = expected_report(committed_source)
    require(type(report) is dict and list(report) == TOP_FIELDS, "REPORT_SHAPE")
    require(report == expected, "REPORT_DERIVATION_MISMATCH")
    require(
        assurance_normalized_current(current_source) == discovery.production(committed_source),
        "SOURCE_CANDIDATE_DRIFT",
    )
    rows = report["rows"]
    require(len(rows) == len({row["id"] for row in rows}) == len({row["site_id"] for row in rows}), "ROW_UNIQUENESS")
    require(report["counts"]["rows"] == len(rows), "COUNT_DERIVATION")
    require(sum(report["counts"]["phases"].values()) == len(rows), "PHASE_COUNT_DERIVATION")
    require(all(list(row) == discovery.ROW_FIELDS for row in rows), "ROW_SHAPE")
    require(all(current_source.count(row["proof_test"].rsplit("::", 1)[-1]) == 1 for row in rows), "PROOF_TEST_MISSING")
    require(schema["additionalProperties"] is False and schema["required"] == TOP_FIELDS, "SCHEMA_CLOSED")
    require(schema["properties"]["rows"]["minItems"] == 1 and "maxItems" not in schema["properties"]["rows"], "SCHEMA_COUNT_NOT_DERIVED")
    row_schema = schema["properties"]["rows"]["items"]
    require(row_schema["additionalProperties"] is False and row_schema["required"] == discovery.ROW_FIELDS, "SCHEMA_ROW_CLOSED")


def self_test(report: Any, schema: Any, committed_source: str, current_source: str) -> int:
    cases = [
        lambda r, _s, _c: r["rows"].pop(),
        lambda r, _s, _c: r["rows"][0].update(counter="graph_edge"),
        lambda r, _s, _c: r["proof_event_model"]["events"].remove("TargetReturned"),
        lambda r, _s, _c: r["counts"].update(rows=1),
        lambda _r, s, _c: s.update(additionalProperties=True),
        lambda _r, _s, c: c.replace("ProjectionBuildSite::MemberCountRead", "/* ProjectionBuildSite::MemberCountRead */", 1),
    ]
    caught = 0
    for mutate in cases:
        changed_report = copy.deepcopy(report)
        changed_schema = copy.deepcopy(schema)
        changed_source = current_source
        result = mutate(changed_report, changed_schema, changed_source)
        if isinstance(result, str):
            changed_source = result
        try:
            validate(changed_report, changed_schema, committed_source, changed_source)
        except InventoryError:
            caught += 1
            continue
        raise InventoryError("MUTATION_SURVIVED")
    return caught


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--write-report", action="store_true")
    args = parser.parse_args()
    committed = candidate_source()
    expected = expected_report(committed)
    if args.write_report:
        REPORT.write_text(json.dumps(expected, ensure_ascii=True, indent=2) + "\n")
    report = json.loads(REPORT.read_text())
    schema = json.loads(SCHEMA.read_text())
    current = SOURCE.read_text()
    validate(report, schema, committed, current)
    attacks = self_test(report, schema, committed, current)
    print(f"PASS v19 causal projection inventory: rows={len(report['rows'])} attacks={attacks}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
