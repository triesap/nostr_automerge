use core::str::FromStr;
use std::fs;
use std::path::Path;

use serde::Deserialize;
use serde_json::{Map, Value};
use sha2::{Digest, Sha256};

use nostr_automerge::{ControllerPublicKey, DevicePublicKey, DocumentId};

use crate::checksum::verify_fixture_files;
use crate::expected::{ExpectedReport, load_expected};
use crate::fixture::load_fixture;
use crate::report_json::write_canonical_report;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum RunError {
    Fixture,
    Checksum,
    Expected,
    Input,
    Mismatch,
}

impl RunError {
    pub(crate) const fn exit_code(self) -> u8 {
        match self {
            Self::Mismatch => 1,
            _ => 2,
        }
    }

    pub(crate) const fn message(self) -> &'static str {
        match self {
            Self::Fixture => "fixture metadata is invalid",
            Self::Checksum => "fixture checksum verification failed",
            Self::Expected => "expected report is invalid",
            Self::Input => "fixture input is invalid or unsupported",
            Self::Mismatch => "fixture result does not match expected report",
        }
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ActorDerivationInput {
    controller: String,
    device: String,
    document_id: String,
}

pub(crate) fn run_fixture(path: &Path) -> Result<Vec<u8>, RunError> {
    let fixture = load_fixture(path).map_err(|_| RunError::Fixture)?;
    let base = path.parent().ok_or(RunError::Fixture)?;
    verify_fixture_files(&fixture, base).map_err(|_| RunError::Checksum)?;
    let expected =
        load_expected(&base.join(&fixture.expected.report_path)).map_err(|_| RunError::Expected)?;
    if fixture.fixture_id != "actor_derivation_001" || fixture.inputs.len() != 1 {
        return Err(RunError::Input);
    }
    let input = fs::read(base.join(&fixture.inputs[0].path)).map_err(|_| RunError::Input)?;
    let input: ActorDerivationInput =
        serde_json::from_slice(&input).map_err(|_| RunError::Input)?;
    let actual = actor_derivation_report(expected.clone(), &input)?;
    compare_expected(&actual, &expected)?;
    write_canonical_report(&actual).map_err(|_| RunError::Expected)
}

fn actor_derivation_report(
    mut report: ExpectedReport,
    input: &ActorDerivationInput,
) -> Result<ExpectedReport, RunError> {
    let controller =
        ControllerPublicKey::from_str(&input.controller).map_err(|_| RunError::Input)?;
    let document = DocumentId::from_str(&input.document_id).map_err(|_| RunError::Input)?;
    let device = DevicePublicKey::from_str(&input.device).map_err(|_| RunError::Input)?;
    let mut hasher = Sha256::new();
    hasher.update(b"nostr-crdt/automerge/actor/v1\0");
    hasher.update(controller.as_bytes());
    hasher.update(document.as_bytes());
    hasher.update(device.as_bytes());
    let actor: [u8; 32] = hasher.finalize().into();
    let mut encoded = String::with_capacity(64);
    for byte in actor {
        use core::fmt::Write;
        write!(&mut encoded, "{byte:02x}").map_err(|_| RunError::Input)?;
    }
    let mut value = Map::new();
    value.insert("type".to_owned(), Value::String("bytes32".to_owned()));
    value.insert("value".to_owned(), Value::String(encoded));
    let Some(assertion) = report.state_assertions.first_mut() else {
        return Err(RunError::Expected);
    };
    assertion.expected = Value::Object(value);
    Ok(report)
}

pub(crate) fn compare_expected(
    actual: &ExpectedReport,
    expected: &ExpectedReport,
) -> Result<(), RunError> {
    (actual == expected).then_some(()).ok_or(RunError::Mismatch)
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::{RunError, compare_expected};
    use crate::expected::load_expected;

    #[test]
    fn expected_mismatch_has_stable_exit_code() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/examples/actor_derivation_001.expected.json");
        let expected = load_expected(&path);
        assert!(expected.is_ok());
        let Ok(expected) = expected else { return };
        let mut actual = expected.clone();
        actual.completion = "cancelled".to_owned();
        assert_eq!(
            compare_expected(&actual, &expected),
            Err(RunError::Mismatch)
        );
        assert_eq!(RunError::Mismatch.exit_code(), 1);
    }
}
