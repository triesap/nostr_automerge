//! Private language-neutral conformance runner.

use std::path::Path;
use std::process::ExitCode;

#[allow(dead_code)]
mod checksum;
#[allow(dead_code)]
mod expected;
#[allow(dead_code)]
mod fixture;
#[allow(dead_code)]
mod report_json;
mod runner;

const HELP: &str = "nostr_automerge_conformance\n\nUSAGE:\n    nostr_automerge_conformance run_fixture <path>\n    nostr_automerge_conformance --help";

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
        _ => CliOutput {
            stdout: String::new(),
            stderr: format!("usage error\n{HELP}\n"),
            code: 2,
        },
    }
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
