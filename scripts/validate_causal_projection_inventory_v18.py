#!/usr/bin/env python3
"""Derive and validate the v18 Rust causal-projection source inventory."""

from __future__ import annotations

import argparse
import copy
import hashlib
import json
import re
import subprocess
import sys
from pathlib import Path
from typing import Any

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parents[1]
REPORT = ROOT / "reports/causal_projection_inventory_v18.json"
SCHEMA = ROOT / "tools/validation/causal_projection_inventory_v18.schema.json"
SOURCE = ROOT / "crates/nostr_automerge/src/graph/actor_state.rs"
SOURCE_PATH = "crates/nostr_automerge/src/graph/actor_state.rs"
SOURCE_CANDIDATE = "076221ad7f03e67d89ac4b2fcfc8f2586b97f182"
AUTHORITY = "spec/causal_projection_contracts_v18.json"
ROW_FIELDS = [
    "id", "phase", "site_id", "family", "language", "applicability",
    "source_path", "source_symbol", "owner_mode", "counter",
    "abstract_owner_class", "reachability_sha256", "proof_test",
    "proof_command", "proof_status", "source_candidate", "result",
]
TOP_FIELDS = [
    "schema", "status", "authority", "source_candidate", "source_path",
    "source_production_sha256", "row_contract", "rows", "counts",
    "derivation", "result_identity_sha256", "result",
]
PHASES = [
    ("projection_build_sites", "ProjectionBuildSite", "projection_construction", "build_trusted_epoch_projection_observed", "source_operation", True),
    ("actor_decision_sites", "ActorDecisionSite", "actor_sequence", "actor_sequence_decision_metered_observed", "direct_operation", False),
    ("causal_next_sites", "CausalNextSite", "causal_counter", "causal_next_decision_metered_observed", "direct_operation", False),
    ("frontier_comparison_sites", "FrontierComparisonSite", "frontier_comparison", "empty_frontier_decision_metered_observed", "direct_operation", True),
]


class InventoryError(RuntimeError):
    pass


def require(condition: bool, code: str) -> None:
    if not condition:
        raise InventoryError(code)


def canonical(value: object) -> bytes:
    return json.dumps(value, ensure_ascii=True, separators=(",", ":"), sort_keys=True).encode()


def snake(value: str) -> str:
    return re.sub(r"(?<!^)(?=[A-Z])", "_", value).lower()


def production(source: str) -> str:
    marker = "\n#[cfg(test)]\npub(crate) mod tests {"
    require(source.count(marker) == 1, "SOURCE_TEST_BOUNDARY")
    return source.split(marker, 1)[0] + "\n"


def lexical_code(source: str) -> str:
    pattern = re.compile(
        r'//[^\n]*|/\*.*?\*/|r(?P<hashes>#+)".*?"(?P=hashes)|r".*?"|"(?:\\.|[^"\\])*"',
        re.DOTALL,
    )
    return pattern.sub(lambda match: "".join("\n" if char == "\n" else " " for char in match.group()), source)


def committed_source() -> str:
    completed = subprocess.run(
        ["git", "show", f"{SOURCE_CANDIDATE}:{SOURCE_PATH}"],
        cwd=ROOT,
        capture_output=True,
        text=True,
        check=False,
    )
    require(completed.returncode == 0, "SOURCE_CANDIDATE_MISSING")
    return completed.stdout


def registry(source: str, macro: str, with_counter: bool) -> tuple[list[tuple[str, str, str]], tuple[int, int]]:
    match = re.search(rf"(?m)^{macro}! \{{\n(?P<body>.*?)^\}}$", source, re.DOTALL)
    require(match is not None, "REGISTRY_MISSING:" + macro)
    pattern = (
        r"^\s*(\w+) => \((\w+), (GraphNode|GraphEdge)\),$"
        if with_counter
        else r"^\s*(\w+) => (\w+),$"
    )
    rows: list[tuple[str, str, str]] = []
    for line in match.group("body").splitlines():
        if not line.strip():
            continue
        item = re.match(pattern, line)
        require(item is not None, "REGISTRY_SHAPE:" + macro)
        rows.append((item.group(1), item.group(2), item.group(3) if with_counter else "GraphNode"))
    require(rows and len(rows) == len({row[0] for row in rows}), "REGISTRY_DUPLICATE:" + macro)
    return rows, match.span()


def exact_test(phase: str, site: str) -> str:
    test_phase = "frontier" if phase == "frontier_comparison" else phase
    site_suffix = snake(site)
    if test_phase == "frontier" and site_suffix.startswith("frontier_"):
        site_suffix = site_suffix.removeprefix("frontier_")
    return f"graph::actor_state::tests::causal_projection_v17_site_{test_phase}_{site_suffix}"


def derive_rows(source: str) -> list[dict[str, Any]]:
    rows: list[dict[str, Any]] = []
    for macro, enum, phase, symbol, owner, with_counter in PHASES:
        entries, span = registry(source, macro, with_counter)
        reachable = lexical_code(source[:span[0]] + (" " * (span[1] - span[0])) + source[span[1]:])
        for site, family, counter in entries:
            occurrences = len(re.findall(rf"\b{enum}::{site}\b", reachable))
            require(occurrences == 1, f"SITE_REACHABILITY:{enum}:{site}:{occurrences}")
            test = exact_test(phase, site)
            rows.append({
                "id": f"rust.{phase}.{snake(site)}",
                "phase": phase,
                "site_id": site,
                "family": family,
                "language": "rust",
                "applicability": "required",
                "source_path": SOURCE_PATH,
                "source_symbol": symbol,
                "owner_mode": "item_metered",
                "counter": snake(counter),
                "abstract_owner_class": owner,
                "reachability_sha256": hashlib.sha256(f"{SOURCE_CANDIDATE}:{enum}:{site}:{occurrences}".encode()).hexdigest(),
                "proof_test": test,
                "proof_command": f"cargo test -p nostr_automerge --lib {test} --locked -- --exact --nocapture",
                "proof_status": "pending_actual_execution",
                "source_candidate": SOURCE_CANDIDATE,
                "result": "pass",
            })
    require(len(rows) == len({row["id"] for row in rows}) == len({row["site_id"] for row in rows}), "SITE_ID_DUPLICATE")
    return rows


def expected_report(candidate_source: str) -> dict[str, Any]:
    source = production(candidate_source)
    rows = derive_rows(source)
    phases = {phase: sum(row["phase"] == phase for row in rows) for _, _, phase, *_ in PHASES}
    report: dict[str, Any] = {
        "schema": "nostr_automerge.causal_projection_inventory.v18.v1",
        "status": "provisional_source_derived",
        "authority": AUTHORITY,
        "source_candidate": SOURCE_CANDIDATE,
        "source_path": SOURCE_PATH,
        "source_production_sha256": hashlib.sha256(source.encode()).hexdigest(),
        "row_contract": ROW_FIELDS,
        "rows": rows,
        "counts": {"rows": len(rows), "phases": phases},
        "derivation": {"method": "committed_lexical_descriptor_registry", "planned_counts": False, "comment_and_string_tokens_ignored": True, "proof_evidence_terminal": False},
        "result_identity_sha256": "",
        "result": "pass",
    }
    identity = {key: value for key, value in report.items() if key != "result_identity_sha256"}
    report["result_identity_sha256"] = hashlib.sha256(canonical(identity)).hexdigest()
    return report


def validate(report: Any, schema: Any, candidate_source: str, current_source: str) -> None:
    expected = expected_report(candidate_source)
    require(type(report) is dict and list(report) == TOP_FIELDS, "REPORT_SHAPE")
    require(report == expected, "REPORT_DERIVATION_MISMATCH")
    require(production(current_source) == production(candidate_source), "SOURCE_CANDIDATE_DRIFT")
    require(report["counts"]["rows"] == len(report["rows"]), "COUNT_DERIVATION")
    require(sum(report["counts"]["phases"].values()) == len(report["rows"]), "PHASE_COUNT_DERIVATION")
    require(all(list(row) == ROW_FIELDS for row in report["rows"]), "ROW_SHAPE")
    require(all(current_source.count(row["proof_test"].rsplit("::", 1)[-1]) == 1 for row in report["rows"]), "PROOF_TEST_MISSING")
    require(schema["additionalProperties"] is False and schema["required"] == TOP_FIELDS, "SCHEMA_CLOSED")
    require(schema["properties"]["rows"]["minItems"] == 1 and "maxItems" not in schema["properties"]["rows"], "SCHEMA_COUNT_NOT_DERIVED")


def self_test(report: Any, schema: Any, candidate_source: str, current_source: str) -> int:
    cases = [
        lambda r, _s, _c: r["rows"].pop(),
        lambda r, _s, _c: r["rows"][0].update(counter="graph_edge"),
        lambda r, _s, _c: r["counts"].update(rows=1),
        lambda _r, s, _c: s.update(additionalProperties=True),
        lambda _r, _s, c: c.replace("ProjectionBuildSite::MemberCountRead", "/* ProjectionBuildSite::MemberCountRead */", 1),
    ]
    caught = 0
    for mutate in cases:
        changed_report, changed_schema, changed_source = copy.deepcopy(report), copy.deepcopy(schema), current_source
        result = mutate(changed_report, changed_schema, changed_source)
        if isinstance(result, str):
            changed_source = result
        try:
            validate(changed_report, changed_schema, candidate_source, changed_source)
        except InventoryError:
            caught += 1
            continue
        raise InventoryError("MUTATION_SURVIVED")
    return caught


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--write-report", action="store_true")
    args = parser.parse_args()
    candidate_source = committed_source()
    expected = expected_report(candidate_source)
    if args.write_report:
        REPORT.write_text(json.dumps(expected, ensure_ascii=True, indent=2) + "\n")
    report, schema = json.loads(REPORT.read_text()), json.loads(SCHEMA.read_text())
    validate(report, schema, candidate_source, SOURCE.read_text())
    print(f"PASS: causal projection inventory v18 rows={report['counts']['rows']} attacks={self_test(report, schema, candidate_source, SOURCE.read_text())}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
