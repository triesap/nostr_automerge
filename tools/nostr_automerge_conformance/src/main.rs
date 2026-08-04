//! Private language-neutral conformance runner.

use std::process::ExitCode;

#[allow(dead_code)]
mod fixture;

const HELP: &str = "nostr_automerge_conformance\n\nUSAGE:\n    nostr_automerge_conformance --help\n\nThe conformance runner is reserved but not implemented yet.";

fn run(args: impl IntoIterator<Item = String>) -> Result<String, String> {
    match args.into_iter().next().as_deref() {
        Some("-h" | "--help") => Ok(HELP.to_owned()),
        Some(_) | None => Err("not implemented: use --help".to_owned()),
    }
}

fn main() -> ExitCode {
    match run(std::env::args().skip(1)) {
        Ok(output) => {
            println!("{output}");
            ExitCode::SUCCESS
        }
        Err(message) => {
            eprintln!("{message}");
            ExitCode::from(2)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::run;

    #[test]
    fn help_is_available_before_semantics() {
        assert!(run(["--help".to_owned()]).is_ok());
        assert!(run(Vec::new()).is_err());
    }
}
