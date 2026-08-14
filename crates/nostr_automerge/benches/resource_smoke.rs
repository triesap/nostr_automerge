//! Sealed-limit resource benchmark smoke.

use nostr_automerge::checkpoint::{leaf_hash, merkle_root};
use nostr_automerge::{
    Completion, CorpusBuilder, DocumentCoordinate, NeverCancelled, ProtocolRevision,
    ReferenceEvaluator, WorkBudget, WorkCounter,
};

fn main() {
    for count in [2048, 4096] {
        let leaves = (0..count)
            .map(|index| leaf_hash(index, count, [index as u8; 32]))
            .collect::<Vec<_>>();
        assert!(merkle_root(&leaves).is_ok());
    }

    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../fixtures/v1_draft/scenarios/change_claims/pending_and_invalid_claims_same_hash.input.json"
    ))
    .expect("signed duplicate-claim fixture");
    let coordinate: DocumentCoordinate = fixture["coordinate"]
        .as_str()
        .expect("coordinate")
        .parse()
        .expect("valid coordinate");
    let mut builder = CorpusBuilder::new();
    for event in fixture["raw_events"].as_array().expect("raw events") {
        let data = event["data"].as_str().expect("event bytes");
        let _ = builder.ingest_bytes(data.as_bytes());
    }
    let mut budget = WorkBudget::new(1_000_000, 1_000_000);
    let report = ReferenceEvaluator::new(ProtocolRevision::draft_v1())
        .evaluate(&builder.finish(), coordinate, &mut budget, &NeverCancelled)
        .expect("resource benchmark report");
    assert_eq!(report.completion(), Completion::Complete);
    assert!(budget.consumed().get(WorkCounter::Carrier) >= 2);
}
