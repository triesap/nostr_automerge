use std::collections::BTreeSet;
use std::fs;
use std::path::{Component, Path, PathBuf};

use serde::Deserialize;

const FIXTURE_SCHEMA: &str = "nostr_automerge.fixture.v1";
const REVISION: &str = "draft_2026_08";

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct FixtureMetadata {
    pub(crate) fixture_schema: String,
    pub(crate) fixture_id: String,
    pub(crate) revision: String,
    pub(crate) requirements: Vec<String>,
    pub(crate) seed: Option<u64>,
    pub(crate) provenance: Provenance,
    pub(crate) inputs: Vec<FixtureFile>,
    pub(crate) expected: ExpectedFile,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct Provenance {
    pub(crate) generator: String,
    pub(crate) generator_revision: String,
    pub(crate) created_at: String,
    pub(crate) source_versions: std::collections::BTreeMap<String, String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct FixtureFile {
    pub(crate) name: String,
    pub(crate) path: PathBuf,
    pub(crate) sha256: String,
    pub(crate) media_type: Option<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct ExpectedFile {
    pub(crate) report_path: PathBuf,
    pub(crate) sha256: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum FixtureError {
    Io,
    Json,
    Schema,
    Identifier,
    Revision,
    Requirement,
    Duplicate,
    Path,
    Missing,
    Checksum,
    NormativeSchema,
}

pub(crate) fn load_fixture(path: &Path) -> Result<FixtureMetadata, FixtureError> {
    let bytes = fs::read(path).map_err(|_| FixtureError::Io)?;
    let fixture: FixtureMetadata =
        serde_json::from_slice(&bytes).map_err(|_| FixtureError::Json)?;
    validate_fixture(&fixture, path.parent().ok_or(FixtureError::Path)?)?;
    Ok(fixture)
}

pub(crate) fn load_normative_fixture(path: &Path) -> Result<FixtureMetadata, FixtureError> {
    let fixture = load_fixture(path)?;
    if fixture
        .inputs
        .as_slice()
        .first()
        .map(|input| input.name.as_str())
        != Some("signed_scenario")
        || fixture.inputs.len() != 1
    {
        return Err(FixtureError::NormativeSchema);
    }
    let base = path.parent().ok_or(FixtureError::Path)?;
    let bytes = fs::read(base.join(&fixture.inputs[0].path)).map_err(|_| FixtureError::Io)?;
    let input: serde_json::Value =
        serde_json::from_slice(&bytes).map_err(|_| FixtureError::Json)?;
    validate_normative_input(&input)?;
    Ok(fixture)
}

fn validate_normative_input(input: &serde_json::Value) -> Result<(), FixtureError> {
    let object = input.as_object().ok_or(FixtureError::NormativeSchema)?;
    if object
        .get("scenario_schema")
        .and_then(serde_json::Value::as_str)
        != Some("nostr_automerge.signed_scenario.v2")
        || !object.contains_key("raw_events")
    {
        return Err(FixtureError::NormativeSchema);
    }
    for forbidden in [
        "operations",
        "valid",
        "selected",
        "accepted",
        "excluded",
        "controls",
        "changes",
    ] {
        if object.contains_key(forbidden) {
            return Err(FixtureError::NormativeSchema);
        }
    }
    Ok(())
}

pub(crate) fn load_fixtures(
    paths: impl IntoIterator<Item = PathBuf>,
) -> Result<Vec<FixtureMetadata>, FixtureError> {
    let mut ids = BTreeSet::new();
    let mut fixtures = Vec::new();
    for path in paths {
        let fixture = load_fixture(&path)?;
        if !ids.insert(fixture.fixture_id.clone()) {
            return Err(FixtureError::Duplicate);
        }
        fixtures.push(fixture);
    }
    fixtures.sort_by(|left, right| left.fixture_id.cmp(&right.fixture_id));
    Ok(fixtures)
}

fn validate_fixture(fixture: &FixtureMetadata, base: &Path) -> Result<(), FixtureError> {
    if fixture.fixture_schema != FIXTURE_SCHEMA {
        return Err(FixtureError::Schema);
    }
    if !is_snake_identifier(&fixture.fixture_id) {
        return Err(FixtureError::Identifier);
    }
    if fixture.revision != REVISION {
        return Err(FixtureError::Revision);
    }
    if fixture.requirements.is_empty()
        || fixture.requirements.iter().any(|id| !is_requirement(id))
        || fixture.requirements.iter().collect::<BTreeSet<_>>().len() != fixture.requirements.len()
    {
        return Err(FixtureError::Requirement);
    }
    if fixture.inputs.is_empty() {
        return Err(FixtureError::Missing);
    }
    let mut names = BTreeSet::new();
    for input in &fixture.inputs {
        if !is_snake_identifier(&input.name) || !names.insert(&input.name) {
            return Err(FixtureError::Duplicate);
        }
        validate_file(base, &input.path, &input.sha256)?;
    }
    validate_file(
        base,
        &fixture.expected.report_path,
        &fixture.expected.sha256,
    )
}

fn validate_file(base: &Path, relative: &Path, checksum: &str) -> Result<(), FixtureError> {
    if relative.as_os_str().is_empty()
        || relative
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(FixtureError::Path);
    }
    if checksum.len() != 64
        || !checksum
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(FixtureError::Checksum);
    }
    base.join(relative)
        .is_file()
        .then_some(())
        .ok_or(FixtureError::Missing)
}

fn is_snake_identifier(value: &str) -> bool {
    (3..=128).contains(&value.len())
        && value.bytes().enumerate().all(|(index, byte)| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || (index > 0 && byte == b'_')
        })
}

fn is_requirement(value: &str) -> bool {
    value.starts_with("NCRDT-")
        && value.len() > 6
        && value[6..]
            .bytes()
            .all(|byte| byte.is_ascii_uppercase() || byte.is_ascii_digit() || byte == b'-')
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use super::{
        FixtureError, FixtureMetadata, load_fixture, load_fixtures, load_normative_fixture,
        validate_fixture, validate_normative_input,
    };

    fn fixture_path() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/examples/actor_derivation_001.fixture.json")
    }

    fn valid() -> Option<FixtureMetadata> {
        load_fixture(&fixture_path()).ok()
    }

    #[test]
    fn implement_fixture_metadata_loader() {
        let fixture = load_fixture(&fixture_path());
        assert!(fixture.is_ok());
        assert_eq!(fixture.as_ref().map(|value| value.seed), Ok(None));
        assert_eq!(
            load_fixtures([fixture_path(), fixture_path()]),
            Err(FixtureError::Duplicate)
        );

        let Some(mut fixture) = valid() else { return };
        let base = fixture_path().parent().map(Path::to_path_buf);
        assert!(base.is_some());
        let Some(base) = base else { return };
        fixture.inputs[0].path = PathBuf::from("../escape.json");
        assert_eq!(validate_fixture(&fixture, &base), Err(FixtureError::Path));

        let Some(mut fixture) = valid() else { return };
        fixture.inputs[0].path = PathBuf::from("missing.json");
        assert_eq!(
            validate_fixture(&fixture, &base),
            Err(FixtureError::Missing)
        );

        let Some(mut fixture) = valid() else { return };
        fixture.revision = "draft_2026_09".to_owned();
        assert_eq!(
            validate_fixture(&fixture, &base),
            Err(FixtureError::Revision)
        );
    }

    #[test]
    fn normative_loader_rejects_abstract_inputs() {
        let signed = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/v1_draft/scenarios/manifest/manifest_valid.fixture.json");
        assert!(load_normative_fixture(&signed).is_ok());
        let abstract_input = serde_json::json!({
            "scenario_schema": "nostr_automerge.interop_core.v1",
            "operations": [{"valid": true, "selected": true}]
        });
        assert_eq!(
            validate_normative_input(&abstract_input),
            Err(FixtureError::NormativeSchema)
        );
    }
}
