use serde_json::Value;

use crate::VerifiedNip01Event;
use crate::profile::kinds::CarrierKind;

use super::CarrierCandidate;

pub(crate) fn classify(event: VerifiedNip01Event) -> Option<CarrierCandidate> {
    let kind = crate::ProtocolRevision::draft_v1().classify_kind(event.kind())?;
    let declaration = declaration(kind, event.content());
    if declaration.unsupported {
        return Some(CarrierCandidate::UnsupportedRevision {
            event,
            declared_version: declaration.version,
            declared_profile: declaration.profile,
        });
    }
    Some(match kind {
        CarrierKind::Manifest => CarrierCandidate::Manifest(event),
        CarrierKind::Control => CarrierCandidate::Control(event),
        CarrierKind::Change => CarrierCandidate::Change(event),
        CarrierKind::CheckpointDescriptor => CarrierCandidate::CheckpointDescriptor(event),
        CarrierKind::CheckpointChunk => CarrierCandidate::CheckpointChunk(event),
    })
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct Declaration {
    version: Option<u64>,
    profile: Option<String>,
    unsupported: bool,
}

fn declaration(kind: CarrierKind, content: &str) -> Declaration {
    if kind == CarrierKind::Change {
        return Declaration::default();
    }
    let Ok(Value::Object(object)) = serde_json::from_str(content) else {
        return Declaration::default();
    };
    let version = object.get("v").and_then(Value::as_u64);
    let profile = object
        .get("format")
        .and_then(Value::as_str)
        .map(str::to_owned);
    let unsupported = version.is_some_and(|value| value != 1)
        || profile
            .as_deref()
            .is_some_and(|value| value != crate::ProtocolRevision::draft_v1().format());
    Declaration {
        version,
        profile,
        unsupported,
    }
}

#[cfg(test)]
mod tests {
    use super::{Declaration, declaration};
    use crate::profile::kinds::CarrierKind;

    #[test]
    fn add_carrier_classification() {
        for kind in [
            CarrierKind::Manifest,
            CarrierKind::Control,
            CarrierKind::CheckpointDescriptor,
            CarrierKind::CheckpointChunk,
        ] {
            assert_eq!(
                declaration(kind, r#"{"format":"automerge-change-v1","v":1}"#),
                Declaration {
                    version: Some(1),
                    profile: Some("automerge-change-v1".to_owned()),
                    unsupported: false,
                }
            );
            assert!(declaration(kind, r#"{"v":2}"#).unsupported);
            assert!(declaration(kind, r#"{"format":"automerge-change-v2"}"#).unsupported);
        }
        assert_eq!(
            declaration(CarrierKind::Change, "binary base64"),
            Declaration::default()
        );
        assert!(
            crate::ProtocolRevision::draft_v1()
                .classify_kind(1)
                .is_none()
        );
    }
}
