#!/usr/bin/env python3
"""Validate v18 causal-projection structure or committed source identity."""

from __future__ import annotations

import argparse
import json
import re
import subprocess
import sys
from pathlib import Path
from typing import Any

sys.dont_write_bytecode = True
SCRIPT_ROOT = Path(__file__).resolve().parents[1]
SOURCE_PATH = "crates/nostr_automerge/src/graph/actor_state.rs"
CONSUMER_PATH = "crates/nostr_automerge/src/reference/epoch_engine.rs"
INVENTORY_PATH = "reports/causal_projection_inventory_v18.json"
CONTRACT_PATH = "spec/causal_projection_contracts_v18.json"
SOURCE_CANDIDATE = "076221ad7f03e67d89ac4b2fcfc8f2586b97f182"
HELPERS = {
    "projection_construction": ("ProjectionBuildSite", "perform_projection_build_operation"),
    "actor_sequence": ("ActorDecisionSite", "perform_actor_decision_operation"),
    "causal_counter": ("CausalNextSite", "perform_causal_next_operation"),
    "frontier_comparison": ("FrontierComparisonSite", "metered_frontier_operation"),
}
DIRECT_SITES = [
    ("actor_sequence_decision_metered_observed", "perform_actor_decision_operation", "ActorDecisionSite", "ActorStateRead", "self.actor_states.get(&candidate.actor).copied()"),
    ("actor_sequence_decision_metered_observed", "perform_actor_decision_operation", "ActorDecisionSite", "PredecessorCandidateRead", "self.branch_membership.get(&state.highest_change)"),
    ("actor_sequence_decision_metered_observed", "perform_actor_decision_operation", "ActorDecisionSite", "ActorIdentityDecision", "match (actor_state, predecessor)"),
    ("actor_sequence_decision_metered_observed", "perform_actor_decision_operation", "ActorDecisionSite", "SequenceRelationDecision", "match (actor_relation, actor_state)"),
    ("causal_next_decision_metered_observed", "perform_causal_next_operation", "CausalNextSite", "StoredCounterRead", "self.causal_next_op"),
    ("causal_next_decision_metered_observed", "perform_causal_next_operation", "CausalNextSite", "ExpectedStartComparison", "candidate.start_op == causal_next_op"),
    ("causal_next_decision_metered_observed", "perform_causal_next_operation", "CausalNextSite", "CheckedAdvance", "causal_next_op.checked_add(candidate.operation_count)"),
]
PROVENANCE_MARKERS = {
    "_v18_typed_stop_collapsed": "TYPED_BUDGET_EXHAUSTED_IDENTITY",
    "_v18_cancellation_collapsed": "TYPED_CANCELLED_IDENTITY",
    "_v18_unexpected_error_replaced": "UNEXPECTED_WORK_ERROR_IDENTITY",
}

sys.path.insert(0, str(SCRIPT_ROOT / "scripts"))
from validate_causal_projection_inventory_v18 import derive_rows, production  # noqa: E402
from validate_causal_projection_source_v13 import function_body  # noqa: E402
from validate_report_contract_v9 import ReportSuiteError, rust_code_view  # noqa: E402


class PropertyError(RuntimeError):
    def __init__(self, code: str):
        super().__init__(code)
        self.code = code


def require(condition: bool, code: str) -> None:
    if not condition:
        raise PropertyError(code)


def code_view(source: str) -> str:
    try:
        return rust_code_view(source)
    except ReportSuiteError as error:
        raise PropertyError("SITE_ID_MISMATCH") from error


def matching_call_end(source: str, start: int) -> int:
    opening = source.find("(", start)
    require(opening >= 0, "ALTERNATE_CONSUMER_BYPASS")
    depth = 0
    for index in range(opening, len(source)):
        character = source[index]
        if character == "(":
            depth += 1
        elif character == ")":
            depth -= 1
            if depth == 0:
                return index + 1
    raise PropertyError("ALTERNATE_CONSUMER_BYPASS")


def helper_structure(source: str, name: str) -> None:
    body = code_view(function_body(source, name))
    descriptor = body.find("let descriptor = site.descriptor();")
    charge = body.find("charge(descriptor)")
    target_text = "let result = target();" if name == "metered_frontier_operation" else "let result = perform();"
    target = body.find(target_text)
    completion = body.find("TargetCompleted")
    returned = body.find("Ok(result)")
    require(min(descriptor, charge, returned) >= 0, "ALTERNATE_CONSUMER_BYPASS")
    require(target >= 0 and body.count(target_text) == 1, "TARGET_AFTER_STOP")
    require(completion >= 0, "OBSERVATION_AFTER_STOP")
    require(descriptor < charge, "CHARGE_AFTER_OPERATION")
    require(charge < target, "CHARGE_AFTER_OPERATION")
    require("?;" in body[charge:target], "TARGET_AFTER_STOP")
    require(target < completion < returned, "OBSERVATION_AFTER_STOP")
    signature_start = source.find(f"fn {name}")
    signature = code_view(source[signature_start:signature_start + 450])
    require("site:" in signature and "Descriptor) -> Result<(), E>" in signature, "SITE_ID_MISMATCH")


def direct_site_structure(source: str) -> None:
    for owner, helper, enum, site, target in DIRECT_SITES:
        body = code_view(function_body(source, owner))
        pattern = re.compile(rf"\b{helper}\s*\(\s*{enum}::{site}\b")
        matches = list(pattern.finditer(body))
        require(len(matches) == 1, "ALTERNATE_CONSUMER_BYPASS")
        call = body[matches[0].start():matching_call_end(body, matches[0].start())]
        closure = call.find("||")
        target_index = call.find(target)
        require(closure >= 0 and target_index > closure, "SITE_TARGET_BEFORE_CHARGE")


def hoist_direct_target(source: str, requested_site: str) -> str:
    matches = [row for row in DIRECT_SITES if row[3] == requested_site]
    require(len(matches) == 1, "SITE_ID_MISMATCH")
    _, helper, enum, site, _ = matches[0]
    production_source = production(source)
    match = re.search(rf"\b{helper}\s*\(\s*{enum}::{site}\b", production_source)
    require(match is not None, "ALTERNATE_CONSUMER_BYPASS")
    call_end = matching_call_end(production_source, match.start())
    call = production_source[match.start():call_end]
    closure = call.find("||")
    final_comma = call.rfind(",")
    require(closure >= 0 and final_comma > closure, "SITE_TARGET_BEFORE_CHARGE")
    target = call[closure + 2:final_comma].strip()
    cache = "_v18_hoisted_" + re.sub(r"(?<!^)(?=[A-Z])", "_", site).lower()
    changed_call = call[:closure + 2] + " " + cache + call[final_comma:]
    changed = source[:match.start()] + changed_call + source[call_end:]
    line_start = changed.rfind("\n", 0, match.start()) + 1
    prefix = changed[line_start:match.start()]
    indent = re.match(r"\s*", prefix).group()
    changed = changed[:line_start] + f"{indent}let {cache} = {target};\n" + changed[line_start:]
    for _, (_, helper_name) in HELPERS.items():
        require(function_body(changed, helper_name) == function_body(source, helper_name), "SITE_TARGET_BEFORE_CHARGE")
    return changed


def validate_structure(
    source_text: str,
    consumer: str,
    inventory: dict[str, Any],
    contract: dict[str, Any],
) -> None:
    source = production(source_text)
    for marker, code in PROVENANCE_MARKERS.items():
        require(marker not in source, code)
    require("ChargeAttempt" not in source, "OPERATION_OBSERVATION_BEFORE_TARGET")
    try:
        current = derive_rows(source)
    except Exception as error:
        raise PropertyError("SITE_ID_MISMATCH") from error
    expected = [
        (row["phase"], row["site_id"], row["family"], row["counter"])
        for row in inventory["rows"]
    ]
    observed = [
        (row["phase"], row["site_id"], row["family"], row["counter"])
        for row in current
    ]
    require(observed == expected, "SITE_ID_MISMATCH")
    for phase, (enum, helper) in HELPERS.items():
        helper_structure(source, helper)
        for row in (row for row in inventory["rows"] if row["phase"] == phase):
            occurrences = len(re.findall(rf"\b{helper}\s*\(\s*{enum}::{row['site_id']}\b", code_view(source)))
            require(occurrences == 1, "ALTERNATE_CONSUMER_BYPASS")
    direct_site_structure(source)
    publish = code_view(function_body(source, "build_trusted_epoch_projection_observed"))
    site = publish.find("ProjectionBuildSite::ProjectionPublish")
    publication = publish.find("published(ProjectionPublicationOperation::Projection);")
    require(site >= 0 and publication > site, "PUBLICATION_AFTER_STOP")
    require(consumer.count(".candidate_semantics_decision_metered(") == 1, "ALTERNATE_CONSUMER_BYPASS")
    require(contract["mutation"]["site_local_property"] == "SITE_TARGET_BEFORE_CHARGE", "SITE_TARGET_BEFORE_CHARGE")
    require(contract["property_codes"] == [
        "TYPED_BUDGET_EXHAUSTED_IDENTITY", "TYPED_CANCELLED_IDENTITY",
        "UNEXPECTED_WORK_ERROR_IDENTITY", "CHARGE_AFTER_OPERATION",
        "OPERATION_OBSERVATION_BEFORE_TARGET", "SITE_TARGET_BEFORE_CHARGE",
        "TARGET_AFTER_STOP", "OBSERVATION_AFTER_STOP", "PUBLICATION_AFTER_STOP",
        "SITE_ID_MISMATCH", "COUNTER_MISMATCH", "ALTERNATE_CONSUMER_BYPASS",
    ], "TYPED_BUDGET_EXHAUSTED_IDENTITY")


def structural(root: Path) -> None:
    source = (root / SOURCE_PATH).read_text()
    consumer = (root / CONSUMER_PATH).read_text()
    inventory = json.loads((root / INVENTORY_PATH).read_text())
    contract = json.loads((root / CONTRACT_PATH).read_text())
    validate_structure(source, consumer, inventory, contract)
    for _, _, _, site, _ in DIRECT_SITES:
        changed = hoist_direct_target(source, site)
        try:
            validate_structure(changed, consumer, inventory, contract)
        except PropertyError as error:
            require(error.code == "SITE_TARGET_BEFORE_CHARGE", f"WRONG_PROPERTY:{site}:{error.code}")
            continue
        raise PropertyError(f"MUTATION_SURVIVED:{site}")


def identity(root: Path) -> None:
    completed = subprocess.run(
        ["git", "show", f"{SOURCE_CANDIDATE}:{SOURCE_PATH}"],
        cwd=root,
        capture_output=True,
        text=True,
        check=False,
    )
    require(completed.returncode == 0, "SOURCE_CANDIDATE_MISSING")
    require(production((root / SOURCE_PATH).read_text()) == production(completed.stdout), "SOURCE_IDENTITY_DRIFT")
    inventory = json.loads((root / INVENTORY_PATH).read_text())
    require(inventory["source_candidate"] == SOURCE_CANDIDATE, "INVENTORY_IDENTITY_DRIFT")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--root", type=Path, default=SCRIPT_ROOT)
    parser.add_argument("--mode", choices=("structural", "identity"), default="structural")
    args = parser.parse_args()
    root = args.root.resolve()
    try:
        structural(root) if args.mode == "structural" else identity(root)
    except PropertyError as error:
        print(f"FAIL: {error.code}")
        return 1
    print(f"PASS: causal projection properties v18 mode={args.mode} sites=source-derived direct-sites=7")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
