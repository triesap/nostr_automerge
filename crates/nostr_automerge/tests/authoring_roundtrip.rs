//! Test-only signing proof for externally signed authoring drafts.

mod support;

use nostr_automerge::authoring::UnsignedEventDraft;
use nostr_automerge::{Nip01VerificationError, VerifiedNip01Event};
use support::test_signer::TestSigner;

#[test]
#[allow(clippy::expect_used)]
fn add_test_only_signing_roundtrip() {
    let signer = TestSigner::from_byte(1);
    for (kind, tags, content) in [
        (
            31_624,
            vec![vec!["d".to_owned(), "document".to_owned()]],
            "manifest",
        ),
        (
            16_624,
            vec![vec!["a".to_owned(), "coordinate".to_owned()]],
            "control",
        ),
        (
            1_624,
            vec![vec!["x".to_owned(), "change".to_owned()]],
            "change",
        ),
    ] {
        let prepared = UnsignedEventDraft::new(1, kind, tags, content.to_owned())
            .expect("valid draft")
            .prepare(signer.public_key())
            .expect("canonical preimage");
        let raw = signer.sign(&prepared);
        let verified = VerifiedNip01Event::verify(raw).expect("signed event verifies");
        assert_eq!(verified.event_id(), prepared.event_id());
        assert_eq!(verified.content(), content);
    }

    let prepared = UnsignedEventDraft::new(1, 1_624, vec![], "wrong signer".to_owned())
        .expect("valid draft")
        .prepare(signer.public_key())
        .expect("canonical preimage");
    let wrong = TestSigner::from_byte(2).sign(&prepared);
    assert_eq!(
        VerifiedNip01Event::verify(wrong),
        Err(Nip01VerificationError::InvalidSignature)
    );
}
