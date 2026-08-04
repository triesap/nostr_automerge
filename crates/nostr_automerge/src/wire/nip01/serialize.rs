use serde::Serialize;

use super::raw::RawNip01Event;

pub(crate) fn canonical_preimage(event: &RawNip01Event) -> Result<Vec<u8>, SerializationError> {
    let public_key = event.pubkey.to_hex();
    let tuple = (
        0_u8,
        public_key.as_str(),
        event.created_at,
        event.kind,
        event.tags.as_slice(),
        event.content.as_str(),
    );
    let mut output = Vec::new();
    tuple
        .serialize(&mut serde_json::Serializer::new(&mut output))
        .map_err(|_| SerializationError)?;
    Ok(output)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct SerializationError;

#[cfg(test)]
mod tests {
    use super::canonical_preimage;
    use crate::wire::nip01::raw::parse;
    use crate::{ProtocolRevision, RawEventBytes};

    #[test]
    #[allow(clippy::expect_used)]
    fn serializes_exact_nip01_array_and_escaping() {
        let raw = format!(
            r#"{{"id":"{}","pubkey":"{}","created_at":0,"kind":1,"tags":[["e","x"]],"content":"line\n☃","sig":"{}"}}"#,
            "00".repeat(32),
            "11".repeat(32),
            "22".repeat(64)
        );
        let bounded = RawEventBytes::new(raw.as_bytes(), ProtocolRevision::draft_v1())
            .expect("trusted fixture");
        let event = parse(&bounded).expect("trusted fixture shape");
        let encoded = canonical_preimage(&event).expect("in-memory serialization");
        let expected = format!(r#"[0,"{}",0,1,[["e","x"]],"line\n☃"]"#, "11".repeat(32));
        assert_eq!(encoded, expected.as_bytes());
    }
}
