//! Raw NIP-01 boundary conformance against committed adversarial fixtures.

use std::fs;
use std::path::{Path, PathBuf};

use nostr_automerge::{
    Nip01VerificationError, ProtocolRevision, RawEventBytes, RawEventError, VerifiedNip01Event,
    WireDiagnostic,
};

fn corpus() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/v1_draft/nip01")
}

#[allow(clippy::expect_used)]
fn read(name: &str) -> Vec<u8> {
    fs::read(corpus().join(name)).expect("committed raw fixture")
}

#[test]
#[allow(clippy::expect_used)]
fn valid_signed_event_is_accepted() {
    let bytes = read("valid_event.json");
    let raw = RawEventBytes::new(trim_newline(&bytes), ProtocolRevision::draft_v1())
        .expect("bounded fixture");
    let event = VerifiedNip01Event::verify(raw).expect("valid signed event");
    assert_eq!(event.kind(), 1);
    assert_eq!(event.content(), "test");
}

#[test]
#[allow(clippy::expect_used)]
fn invalid_raw_corpus_has_exact_stable_diagnostics() {
    let cases = [
        (
            "duplicate_member.json",
            Nip01VerificationError::DuplicateMember,
            "json.duplicate_member",
        ),
        (
            "event_id_mismatch.json",
            Nip01VerificationError::EventIdMismatch,
            "nip01.event_id",
        ),
        (
            "invalid_signature.json",
            Nip01VerificationError::InvalidSignature,
            "nip01.signature",
        ),
        (
            "uppercase_identifier.json",
            Nip01VerificationError::Identifier,
            "nip01.identifier",
        ),
        (
            "invalid_tags.json",
            Nip01VerificationError::Shape,
            "nip01.shape",
        ),
        (
            "trailing_value.json",
            Nip01VerificationError::JsonSyntax,
            "json.syntax",
        ),
    ];
    for (name, expected, code) in cases {
        let bytes = read(name);
        let raw = RawEventBytes::new(trim_newline(&bytes), ProtocolRevision::draft_v1())
            .expect("UTF-8 fixture");
        let actual = VerifiedNip01Event::verify(raw);
        assert_eq!(actual, Err(expected), "{name}");
        assert_eq!(
            WireDiagnostic::from_nip01(expected).code().as_str(),
            code,
            "{name}"
        );
    }
}

#[test]
#[allow(clippy::expect_used)]
fn invalid_utf8_fails_at_ingress() {
    let encoded = String::from_utf8(read("invalid_utf8.raw.hex")).expect("ASCII hex fixture");
    let bytes = decode_hex(encoded.trim()).expect("valid fixture encoding");
    assert_eq!(
        RawEventBytes::new(&bytes, ProtocolRevision::draft_v1()),
        Err(RawEventError::InvalidUtf8)
    );
}

fn trim_newline(bytes: &[u8]) -> &[u8] {
    bytes.strip_suffix(b"\n").unwrap_or(bytes)
}

fn decode_hex(input: &str) -> Result<Vec<u8>, ()> {
    if !input.len().is_multiple_of(2) {
        return Err(());
    }
    input
        .as_bytes()
        .chunks_exact(2)
        .map(|pair| {
            let text = core::str::from_utf8(pair).map_err(|_| ())?;
            u8::from_str_radix(text, 16).map_err(|_| ())
        })
        .collect()
}
