use std::fs;
use std::path::Path;

use sha2::{Digest, Sha256};

use crate::fixture::FixtureMetadata;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ChecksumError {
    Io,
    Mismatch,
}

pub(crate) fn verify_fixture_files(
    fixture: &FixtureMetadata,
    base: &Path,
) -> Result<(), ChecksumError> {
    for input in &fixture.inputs {
        verify_file(&base.join(&input.path), &input.sha256)?;
    }
    verify_file(
        &base.join(&fixture.expected.report_path),
        &fixture.expected.sha256,
    )
}

fn verify_file(path: &Path, expected: &str) -> Result<(), ChecksumError> {
    let bytes = fs::read(path).map_err(|_| ChecksumError::Io)?;
    let actual = Sha256::digest(bytes);
    let mut encoded = String::with_capacity(64);
    for byte in actual {
        use core::fmt::Write;
        write!(&mut encoded, "{byte:02x}").map_err(|_| ChecksumError::Mismatch)?;
    }
    (encoded == expected)
        .then_some(())
        .ok_or(ChecksumError::Mismatch)
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::{ChecksumError, verify_fixture_files};
    use crate::fixture::load_fixture;

    #[test]
    fn verify_fixture_file_checksums() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/examples/actor_derivation_001.fixture.json");
        let fixture = load_fixture(&path);
        assert!(fixture.is_ok());
        let Ok(mut fixture) = fixture else { return };
        let base = path.parent();
        assert!(base.is_some());
        let Some(base) = base else { return };
        assert_eq!(verify_fixture_files(&fixture, base), Ok(()));

        fixture.inputs[0].path = fixture.expected.report_path.clone();
        assert_eq!(
            verify_fixture_files(&fixture, base),
            Err(ChecksumError::Mismatch)
        );
    }
}
