//! Raw Automerge framing rejection corpus.

use std::fs;
use std::path::{Path, PathBuf};

fn corpus() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/v1_draft/automerge_framing")
}

#[test]
#[allow(clippy::expect_used)]
fn forbidden_chunk_corpus_is_complete() {
    for name in [
        "document_chunk.hex",
        "compressed_change_chunk.hex",
        "bundle_chunk.hex",
        "unknown_chunk.hex",
    ] {
        let encoded = fs::read_to_string(corpus().join(name)).expect("committed fixture");
        let bytes = decode_hex(encoded.trim()).expect("fixture hex");
        assert_eq!(&bytes[..4], &[0x85, 0x6f, 0x4a, 0x83], "{name}");
        assert_ne!(bytes[8], 1, "{name}");
    }
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
