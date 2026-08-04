use sha2::{Digest, Sha256};

use crate::EventId;

use super::raw::RawNip01Event;
use super::serialize::{SerializationError, canonical_preimage};

pub(crate) fn calculate_event_id(event: &RawNip01Event) -> Result<EventId, EventIdError> {
    let preimage = canonical_preimage(event).map_err(EventIdError::Serialization)?;
    Ok(EventId::from_bytes(Sha256::digest(preimage).into()))
}

pub(crate) fn verify_declared_event_id(event: &RawNip01Event) -> Result<(), EventIdError> {
    if calculate_event_id(event)? == event.id {
        Ok(())
    } else {
        Err(EventIdError::Mismatch)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum EventIdError {
    Serialization(SerializationError),
    Mismatch,
}

#[cfg(test)]
mod tests {
    use super::{EventIdError, calculate_event_id, verify_declared_event_id};
    use crate::wire::nip01::raw::parse;
    use crate::{ProtocolRevision, RawEventBytes};

    #[test]
    #[allow(clippy::expect_used)]
    fn matches_independent_sha256_vector_and_declared_id() {
        let expected = "ddeaafce75fa5021c2d3f7b71dfa1cd6eef21f14a896685fbc0ca531e06f62cd";
        let raw = format!(
            r#"{{"id":"{expected}","pubkey":"{}","created_at":0,"kind":1,"tags":[["e","x"]],"content":"line\n☃","sig":"{}"}}"#,
            "11".repeat(32),
            "22".repeat(64)
        );
        let bounded = RawEventBytes::new(raw.as_bytes(), ProtocolRevision::draft_v1())
            .expect("trusted fixture");
        let event = parse(&bounded).expect("trusted fixture shape");
        assert_eq!(
            calculate_event_id(&event).map(|id| id.to_hex()),
            Ok(expected.to_owned())
        );
        assert!(verify_declared_event_id(&event).is_ok());

        let changed = raw.replace(expected, &"00".repeat(32));
        let bounded = RawEventBytes::new(changed.as_bytes(), ProtocolRevision::draft_v1())
            .expect("trusted fixture");
        let event = parse(&bounded).expect("trusted fixture shape");
        assert_eq!(
            verify_declared_event_id(&event),
            Err(EventIdError::Mismatch)
        );
    }
}
