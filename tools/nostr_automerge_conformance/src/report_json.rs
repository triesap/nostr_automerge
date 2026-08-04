use crate::expected::{ExpectedError, ExpectedReport, validate_expected};

pub(crate) fn write_canonical_report(report: &ExpectedReport) -> Result<Vec<u8>, ExpectedError> {
    validate_expected(report)?;
    let value = serde_json::to_value(report).map_err(|_| ExpectedError::Json)?;
    let mut bytes = serde_json::to_vec(&value).map_err(|_| ExpectedError::Json)?;
    bytes.push(b'\n');
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::Path;

    use super::write_canonical_report;
    use crate::expected::load_expected;

    #[test]
    fn implement_canonical_report_json_writer() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/examples/actor_derivation_001.expected.json");
        let expected_bytes = fs::read(&path);
        let report = load_expected(&path);
        assert!(expected_bytes.is_ok() && report.is_ok());
        let (Ok(expected_bytes), Ok(report)) = (expected_bytes, report) else {
            return;
        };
        let first = write_canonical_report(&report);
        let second = write_canonical_report(&report);
        assert_eq!(first, second);
        assert_eq!(first, Ok(expected_bytes));
    }
}
