#!/usr/bin/env python3
"""Execute the remediation-v4 ordinary source mutation campaign."""

from __future__ import annotations

import argparse
import hashlib
import json
import subprocess
from dataclasses import dataclass
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
OUTPUT = ROOT / "reports/mutation_campaign_v4.json"


@dataclass(frozen=True)
class Mutation:
    name: str
    path: str
    search: str
    replacement: str
    command: tuple[str, ...]


PUBLIC_TEST = ("cargo", "test", "-p", "nostr_automerge", "--test", "public_engine_api")
LIB_TEST = ("cargo", "test", "-p", "nostr_automerge", "--lib")
CORPUS_TEST = (
    "cargo", "run", "--quiet", "-p", "nostr_automerge_conformance", "--locked",
    "--", "run_corpus", "fixtures/v1_draft/scenarios",
)

MUTATIONS = (
    Mutation(
        "silent_missing_control_claim_omission",
        "crates/nostr_automerge/src/engine/reference_evaluator.rs",
        "None => ClaimState::Pending,",
        "None => ClaimState::Invalid,",
        CORPUS_TEST,
    ),
    Mutation(
        "preferred_carrier_only",
        "crates/nostr_automerge/src/engine/reference_evaluator.rs",
        "let mut raw = None;\n    let mut carriers = Vec::new();\n    for event_id in event_ids {",
        "let mut raw = None;\n    let mut carriers = Vec::new();\n    for event_id in event_ids.iter().take(1) {",
        CORPUS_TEST,
    ),
    Mutation(
        "accepted_base_readmission",
        "crates/nostr_automerge/src/reference/evaluate.rs",
        "let epoch_changes = control\n        .changes\n        .iter()\n        .filter(|change| {\n            !accepted_base\n                .accepted_closure()\n                .contains(&change.candidate.change_hash)\n        })",
        "let epoch_changes = control\n        .changes\n        .iter()\n        .filter(|_| true)",
        LIB_TEST + ("reference::evaluate::tests::accepted_base_candidates_are_filtered_from_both_epoch_paths", "--locked"),
    ),
    Mutation(
        "invalid_claim_poisoning",
        "crates/nostr_automerge/src/engine/reference_evaluator.rs",
        "if final_accepted.contains(&hash) {\n            batch\n                .dispositions\n                .insert(hash, ProtocolDisposition::Accepted);\n            continue;\n        }",
        "if final_accepted.contains(&hash) {\n            batch\n                .dispositions\n                .insert(hash, ProtocolDisposition::Invalid);\n            continue;\n        }",
        CORPUS_TEST,
    ),
    Mutation(
        "pruned_dependency_unknown",
        "crates/nostr_automerge/src/reference/evaluate.rs",
        ".filter(|hash| !selected_base.contains(hash))",
        ".filter(|_| false)",
        LIB_TEST + ("reference::evaluate::tests::pruned_prior_dependency_is_invalid_not_pending", "--locked"),
    ),
    Mutation(
        "corpus_global_coordinate_scan",
        "crates/nostr_automerge/src/evidence/document_view.rs",
        "pub(crate) fn evaluation_event_count(&self) -> usize {\n        self.input_event_ids().count().saturating_add(\n            self.corpus\n                .duplicates\n                .iter()\n                .filter(|evidence| match evidence {\n                    EventEvidence::DuplicateEvent { event_id, .. } => self.contains_input(event_id),\n                    _ => false,\n                })\n                .count(),\n        )\n    }",
        "pub(crate) fn evaluation_event_count(&self) -> usize {\n        self.corpus.evaluation_event_count()\n    }",
        PUBLIC_TEST + ("unrelated_coordinate_evidence_is_report_and_budget_inert", "--locked"),
    ),
    Mutation(
        "unrelated_manifest_in_target_dispositions",
        "crates/nostr_automerge/src/evidence/document_view.rs",
        "pub(crate) fn contains_reportable(&self, event_id: &EventId) -> bool {\n        self.reportable_event_ids.contains(event_id)\n    }",
        "pub(crate) fn contains_reportable(&self, event_id: &EventId) -> bool {\n        self.corpus.events.contains_key(event_id)\n    }",
        PUBLIC_TEST + ("unrelated_coordinate_evidence_is_report_and_budget_inert", "--locked"),
    ),
    Mutation(
        "skip_finalization_reservation",
        "crates/nostr_automerge/src/engine/reference_evaluator.rs",
        ".and_then(|value| value.checked_add(8))",
        ".and_then(|_| Some(0))",
        PUBLIC_TEST + ("every_v3_work_counter_boundary", "--locked"),
    ),
    Mutation(
        "partial_finalization_reservation",
        "crates/nostr_automerge/src/engine/reference_evaluator.rs",
        "budget.charge(WorkCounter::Assertion, plan.items)?;",
        "for _ in 0..plan.items {\n            budget.charge(WorkCounter::Assertion, 1)?;\n        }",
        LIB_TEST + ("engine::reference_evaluator::tests::finalization_reservation_is_atomic_and_refundable", "--locked"),
    ),
    Mutation(
        "interrupted_report_permit_bypass",
        "crates/nostr_automerge/src/engine/reference_evaluator.rs",
        "fn reserved_interrupted_report(\n    revision: ProtocolRevision,\n    coordinate: DocumentCoordinate,\n    completion: Completion,\n    permit: &mut ReportFinalizationPermit,\n) -> Result<EvaluationReport, EvaluationError> {\n    permit.consume();",
        "fn reserved_interrupted_report(\n    revision: ProtocolRevision,\n    coordinate: DocumentCoordinate,\n    completion: Completion,\n    permit: &mut ReportFinalizationPermit,\n) -> Result<EvaluationReport, EvaluationError> {\n    let _ = permit;",
        LIB_TEST + ("engine::reference_evaluator::tests::reserved_report_wrappers_consume_without_optional_expansion", "--locked"),
    ),
    Mutation(
        "optional_evidence_expansion_after_stop",
        "crates/nostr_automerge/src/engine/reference_evaluator.rs",
        ") -> Result<EvaluationReport, EvaluationError> {\n    permit.consume();\n    compact_batch_report(revision, coordinate, batch, manifest, checkpoints)\n}",
        ") -> Result<EvaluationReport, EvaluationError> {\n    let _ = checkpoints.iter().count();\n    permit.consume();\n    compact_batch_report(revision, coordinate, batch, manifest, checkpoints)\n}",
        LIB_TEST + ("engine::reference_evaluator::tests::reserved_report_wrappers_consume_without_optional_expansion", "--locked"),
    ),
    Mutation(
        "first_d_manifest_selection",
        "crates/nostr_automerge/src/evidence/corpus_builder.rs",
        "let document_ids = event\n        .tags()\n        .iter()\n        .filter(|tag| tag.first().is_some_and(|value| value == \"d\"))\n        .filter_map(|tag| tag.get(1)?.parse().ok())\n        .collect::<BTreeSet<_>>();\n    if document_ids.len() != 1 {\n        return None;\n    }\n    let document_id = document_ids.into_iter().next()?;",
        "let document_id = event\n        .tags()\n        .iter()\n        .filter(|tag| tag.first().is_some_and(|value| value == \"d\"))\n        .find_map(|tag| tag.get(1)?.parse().ok())?;",
        CORPUS_TEST,
    ),
    Mutation(
        "duplicate_same_d_manifest_fallback",
        "crates/nostr_automerge/src/evidence/corpus_builder.rs",
        "EventEvidence::InvalidCarrier {\n                event, diagnostic, ..\n            } => manifest_coordinate(event).map(|coordinate| {\n                (\n                    coordinate,\n                    ManifestSelection {\n                        created_at: event.created_at(),\n                        event_id: event.event_id(),\n                        state: ManifestSelectionState::Unavailable(*diagnostic),\n                    },\n                )\n            }),",
        "EventEvidence::InvalidCarrier { .. } => None,",
        CORPUS_TEST,
    ),
)


def source_commit() -> str:
    return subprocess.run(
        ("git", "log", "-1", "--format=%H", "--", "crates", "tools", "Cargo.toml", "Cargo.lock", "rust-toolchain.toml", "fixtures"),
        cwd=ROOT,
        check=True,
        capture_output=True,
        text=True,
    ).stdout.strip()


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--anchors-only", action="store_true")
    args = parser.parse_args()
    for mutation in MUTATIONS:
        source = (ROOT / mutation.path).read_text(encoding="utf-8")
        if source.count(mutation.search) != 1:
            raise AssertionError(f"stale mutation anchor: {mutation.name}")
    if args.anchors_only:
        print(f"PASS: {len(MUTATIONS)} remediation-v4 mutation anchors are exact")
        return 0

    results: list[dict[str, str]] = []
    for mutation in MUTATIONS:
        path = ROOT / mutation.path
        original = path.read_text(encoding="utf-8")
        mutated = original.replace(mutation.search, mutation.replacement, 1)
        try:
            path.write_text(mutated, encoding="utf-8")
            completed = subprocess.run(
                mutation.command,
                cwd=ROOT,
                capture_output=True,
                check=False,
                text=True,
            )
        finally:
            path.write_text(original, encoding="utf-8")
        corpus_failure = False
        if mutation.command == CORPUS_TEST and completed.returncode == 0:
            summary = json.loads(completed.stdout)
            corpus_failure = summary.get("failed", 0) > 0
        if completed.returncode == 0 and not corpus_failure:
            raise AssertionError(f"source mutation survived: {mutation.name}")
        if corpus_failure:
            outcome = "caught_by_signed_corpus"
        elif "test result: FAILED" in completed.stdout + completed.stderr:
            outcome = "caught_by_test"
        else:
            outcome = "rejected"
        results.append({"mutation": mutation.name, "result": outcome})

    for mutation in MUTATIONS:
        data = (ROOT / mutation.path).read_bytes()
        if hashlib.sha256(data).hexdigest() == hashlib.sha256(data.replace(b"\r\n", b"\n")).hexdigest():
            continue
        raise AssertionError(f"source restoration changed line endings: {mutation.path}")
    report = {
        "schema": "nostr_automerge.mutation_campaign.v4",
        "implementation_commit": source_commit(),
        "tool": "repository ordinary deterministic source mutator v4",
        "generated": len(results),
        "caught": len(results),
        "survived": 0,
        "status": "pass",
        "mutations": results,
    }
    OUTPUT.write_text(json.dumps(report, sort_keys=True, separators=(",", ":")) + "\n", encoding="utf-8")
    print(f"PASS: all {len(results)} required remediation-v4 source mutations were killed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
