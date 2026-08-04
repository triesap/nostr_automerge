use crate::{ProtocolDisposition, ProtocolRevision};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum VersionAssessment {
    Supported,
    Unsupported {
        declared_version: Option<u64>,
        declared_profile: Option<String>,
    },
}

pub(crate) fn assess(
    declared_version: Option<u64>,
    declared_profile: Option<&str>,
) -> VersionAssessment {
    if declared_version.is_some_and(|value| value != 1)
        || declared_profile.is_some_and(|value| value != ProtocolRevision::draft_v1().format())
    {
        VersionAssessment::Unsupported {
            declared_version,
            declared_profile: declared_profile.map(str::to_owned),
        }
    } else {
        VersionAssessment::Supported
    }
}

pub(crate) const fn disposition_for_supported(valid: bool) -> ProtocolDisposition {
    ProtocolDisposition::for_revision(true, valid)
}

#[cfg(test)]
mod tests {
    use super::{VersionAssessment, assess, disposition_for_supported};
    use crate::ProtocolDisposition;
    use crate::carrier::manifest::{ManifestContentError, parse_content};

    #[test]
    fn enforce_protocol_revision_profile_semantics() {
        assert_eq!(
            assess(Some(1), Some("automerge-change-v1")),
            VersionAssessment::Supported
        );
        assert!(matches!(
            assess(Some(2), None),
            VersionAssessment::Unsupported { .. }
        ));
        assert!(matches!(
            assess(None, Some("automerge-change-v2")),
            VersionAssessment::Unsupported { .. }
        ));
        assert_eq!(
            disposition_for_supported(false),
            ProtocolDisposition::Invalid
        );
        assert_eq!(
            disposition_for_supported(true),
            ProtocolDisposition::Accepted
        );

        let known_v1_unknown_field = r#"{"application":null,"checkpoint":null,"control":"1111111111111111111111111111111111111111111111111111111111111111","description":null,"format":"automerge-change-v1","name":null,"relays":[],"status":"active","successor":null,"text_encoding":"utf16","unknown":null,"v":1}"#;
        assert_eq!(
            parse_content(known_v1_unknown_field),
            Err(ManifestContentError::Shape)
        );
        assert_eq!(
            disposition_for_supported(parse_content(known_v1_unknown_field).is_ok()),
            ProtocolDisposition::Invalid
        );
    }
}
