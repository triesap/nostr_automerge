//! Private language-neutral conformance runner.

use std::path::Path;
use std::process::ExitCode;

#[allow(dead_code)]
mod checksum;
#[allow(dead_code)]
mod expected;
#[allow(dead_code)]
mod fixture;
mod fixture_generation;
#[allow(dead_code)]
mod permutations;
#[allow(dead_code)]
mod report_json;
mod runner;
mod scenario;

const HELP: &str = "nostr_automerge_conformance\n\nUSAGE:\n    nostr_automerge_conformance run_fixture <path>\n    nostr_automerge_conformance run_corpus <directory> [--family <name>] [--requirement <id>]\n    nostr_automerge_conformance generate_signed_profile <profile>\n    nostr_automerge_conformance --help";

#[derive(Clone, Debug, PartialEq, Eq)]
struct CliOutput {
    stdout: String,
    stderr: String,
    code: u8,
}

fn run(args: impl IntoIterator<Item = String>) -> CliOutput {
    let args = args.into_iter().collect::<Vec<_>>();
    match args.as_slice() {
        [argument] if matches!(argument.as_str(), "-h" | "--help") => CliOutput {
            stdout: format!("{HELP}\n"),
            stderr: String::new(),
            code: 0,
        },
        [command, path] if command == "run_fixture" => match runner::run_fixture(Path::new(path)) {
            Ok(bytes) => CliOutput {
                stdout: String::from_utf8(bytes).unwrap_or_default(),
                stderr: String::new(),
                code: 0,
            },
            Err(error) => CliOutput {
                stdout: String::new(),
                stderr: format!("{}\n", error.message()),
                code: error.exit_code(),
            },
        },
        [command, root, rest @ ..] if command == "run_corpus" => {
            let filters = parse_filters(rest);
            match filters.and_then(|(family, requirement)| {
                let paths = runner::discover_fixtures(Path::new(root)).map_err(|_| ())?;
                let summary = runner::run_corpus(paths, family.as_deref(), requirement.as_deref());
                let code = u8::from(summary.failed != 0);
                runner::write_corpus_summary(&summary)
                    .map(|bytes| (bytes, code))
                    .map_err(|_| ())
            }) {
                Ok((bytes, code)) => CliOutput {
                    stdout: String::from_utf8(bytes).unwrap_or_default(),
                    stderr: String::new(),
                    code,
                },
                Err(()) => CliOutput {
                    stdout: String::new(),
                    stderr: "corpus command failed\n".to_owned(),
                    code: 2,
                },
            }
        }
        [command, profile] if command == "generate_signed_profile" => {
            match fixture_generation::generate(profile) {
                Ok(()) => CliOutput {
                    stdout: format!("generated signed profile: {profile}\n"),
                    stderr: String::new(),
                    code: 0,
                },
                Err(error) => CliOutput {
                    stdout: String::new(),
                    stderr: format!("signed profile generation failed: {error}\n"),
                    code: 2,
                },
            }
        }
        _ => CliOutput {
            stdout: String::new(),
            stderr: format!("usage error\n{HELP}\n"),
            code: 2,
        },
    }
}

fn parse_filters(args: &[String]) -> Result<(Option<String>, Option<String>), ()> {
    let mut family = None;
    let mut requirement = None;
    let mut index = 0;
    while index < args.len() {
        let target = match args[index].as_str() {
            "--family" => &mut family,
            "--requirement" => &mut requirement,
            _ => return Err(()),
        };
        let Some(value) = args.get(index + 1) else {
            return Err(());
        };
        if target.replace(value.clone()).is_some() {
            return Err(());
        }
        index += 2;
    }
    Ok((family, requirement))
}

fn main() -> ExitCode {
    let output = run(std::env::args().skip(1));
    print!("{}", output.stdout);
    eprint!("{}", output.stderr);
    ExitCode::from(output.code)
}

#[cfg(test)]
mod tests {
    use super::run;

    #[test]
    fn no_parallel_normative_evaluator() {
        let source = include_str!("main.rs");
        let forbidden = ["mod ", "interop", ";"].concat();
        assert!(!source.contains(&forbidden));
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/interop.rs");
        assert!(!path.exists());
        for legacy in ["permutation.rs", "scenario_variants.rs"] {
            assert!(
                !std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                    .join("src")
                    .join(legacy)
                    .exists()
            );
        }
    }

    #[test]
    fn add_single_fixture_cli_command() {
        let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/examples/actor_derivation_001.fixture.json");
        let success = run(["run_fixture".to_owned(), fixture.display().to_string()]);
        assert_eq!(success.code, 0);
        assert!(success.stdout.contains("actor_derivation_001"));
        assert!(success.stderr.is_empty());

        let malformed = run(["run_fixture".to_owned(), "missing.fixture.json".to_owned()]);
        assert_eq!(malformed.code, 2);
        assert!(malformed.stdout.is_empty());
        assert!(!malformed.stderr.is_empty());

        let help = run(["--help".to_owned()]);
        assert_eq!(help.code, 0);
        assert!(help.stdout.contains("run_fixture"));
        assert_eq!(run(Vec::new()).code, 2);
    }
}
