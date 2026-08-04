use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct Registry {
    requirements: Vec<Requirement>,
}

#[derive(Deserialize)]
struct Requirement {
    id: String,
    section: String,
}

#[derive(Deserialize)]
struct FixtureRequirements {
    requirements: Vec<String>,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub(crate) struct CoverageReport {
    pub(crate) covered: Vec<String>,
    pub(crate) deferred_checkpoint: Vec<String>,
    pub(crate) missing: Vec<String>,
    pub(crate) schema: &'static str,
    pub(crate) unknown: Vec<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CoverageError {
    Io,
    Json,
    Duplicate,
}

pub(crate) fn generate_coverage_report(
    registry_path: &Path,
    fixture_paths: impl IntoIterator<Item = PathBuf>,
) -> Result<CoverageReport, CoverageError> {
    let registry: Registry =
        serde_json::from_slice(&fs::read(registry_path).map_err(|_| CoverageError::Io)?)
            .map_err(|_| CoverageError::Json)?;
    let mut requirements = BTreeMap::new();
    for requirement in registry.requirements {
        if requirements
            .insert(requirement.id, requirement.section)
            .is_some()
        {
            return Err(CoverageError::Duplicate);
        }
    }
    let mut referenced = BTreeSet::new();
    for path in fixture_paths {
        let fixture: FixtureRequirements =
            serde_json::from_slice(&fs::read(path).map_err(|_| CoverageError::Io)?)
                .map_err(|_| CoverageError::Json)?;
        referenced.extend(fixture.requirements);
    }
    Ok(classify_coverage(requirements, referenced))
}

fn classify_coverage(
    requirements: BTreeMap<String, String>,
    referenced: BTreeSet<String>,
) -> CoverageReport {
    let known = requirements.keys().cloned().collect::<BTreeSet<_>>();
    let covered = referenced.intersection(&known).cloned().collect::<Vec<_>>();
    let unknown = referenced.difference(&known).cloned().collect::<Vec<_>>();
    let deferred_checkpoint = requirements
        .iter()
        .filter(|(id, section)| {
            !referenced.contains(*id) && section.to_ascii_lowercase().contains("checkpoint")
        })
        .map(|(id, _)| id.clone())
        .collect::<Vec<_>>();
    let deferred = deferred_checkpoint.iter().cloned().collect::<BTreeSet<_>>();
    let missing = known
        .difference(&referenced)
        .filter(|id| !deferred.contains(*id))
        .cloned()
        .collect::<Vec<_>>();
    CoverageReport {
        covered,
        deferred_checkpoint,
        missing,
        schema: "nostr_automerge.requirement_coverage.v1",
        unknown,
    }
}

pub(crate) fn discover_fixture_metadata(root: &Path) -> Result<Vec<PathBuf>, CoverageError> {
    let mut pending = vec![root.to_path_buf()];
    let mut paths = Vec::new();
    while let Some(directory) = pending.pop() {
        for entry in fs::read_dir(directory).map_err(|_| CoverageError::Io)? {
            let path = entry.map_err(|_| CoverageError::Io)?.path();
            if path.is_dir() {
                pending.push(path);
            } else if path
                .file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.ends_with(".fixture.json"))
            {
                paths.push(path);
            }
        }
    }
    paths.sort();
    Ok(paths)
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::{discover_fixture_metadata, generate_coverage_report};

    #[test]
    fn generate_requirement_coverage_report() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let fixtures = discover_fixture_metadata(&root.join("fixtures"));
        assert!(fixtures.is_ok());
        let Ok(fixtures) = fixtures else { return };
        let report = generate_coverage_report(&root.join("spec/requirements.json"), fixtures);
        assert!(report.is_ok());
        let Ok(report) = report else { return };
        assert!(report.unknown.is_empty());
        assert!(report.covered.iter().any(|id| id == "NCRDT-ACTOR-001"));
        assert!(!report.deferred_checkpoint.is_empty());
        assert!(
            report
                .deferred_checkpoint
                .iter()
                .all(|id| id.starts_with("NCRDT-"))
        );
        assert!(!report.missing.is_empty());
    }
}
