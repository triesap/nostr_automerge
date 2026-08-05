//! Downstream-only compile and behavior checks for the public engine surface.

use nostr_automerge::{CorpusBuilder, IngestOutcome, ProtocolRevision, RawEventBytes};

#[test]
fn build_immutable_ingress_corpus_through_public_api() {
    let valid = RawEventBytes::new(
        include_bytes!("../../../fixtures/v1_draft/nip01/valid_event.json"),
        ProtocolRevision::draft_v1(),
    );
    assert!(valid.is_ok());
    let Ok(valid) = valid else { return };
    let invalid = RawEventBytes::new(b"{}", ProtocolRevision::draft_v1());
    assert!(invalid.is_ok());
    let Ok(invalid) = invalid else { return };

    let mut builder = CorpusBuilder::new();
    assert!(matches!(
        builder.ingest(valid.clone()),
        IngestOutcome::Irrelevant { .. }
    ));
    assert!(matches!(
        builder.ingest(valid),
        IngestOutcome::Duplicate { .. }
    ));
    assert!(matches!(
        builder.ingest(invalid),
        IngestOutcome::Invalid { .. }
    ));
    let corpus = builder.finish();
    assert_eq!(corpus.event_count(), 1);
    assert_eq!(corpus.invalid_count(), 1);
    assert_eq!(corpus.duplicate_count(), 1);
    assert!(!corpus.is_empty());
}
