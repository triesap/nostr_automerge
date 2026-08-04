//! Private repository automation entry point.

use std::process::ExitCode;

mod requirements;
mod validate;

const HELP: &str = "nostr_automerge_xtask\n\nUSAGE:\n    nostr_automerge_xtask validate\n    nostr_automerge_xtask --help";

fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);
    match args.next().as_deref() {
        Some("-h" | "--help") => {
            println!("{HELP}");
            ExitCode::SUCCESS
        }
        Some("validate") if args.next().is_none() => {
            let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
            match validate::validate_repository(&root) {
                Ok(report) => {
                    println!("PASS: repository validation");
                    println!("- validators={}", report.validators.join(","));
                    println!("- covered_requirements={}", report.covered_requirements);
                    println!(
                        "- deferred_checkpoint_requirements={}",
                        report.deferred_checkpoint_requirements
                    );
                    ExitCode::SUCCESS
                }
                Err(error) => {
                    eprintln!("repository validation failed: {error}");
                    ExitCode::FAILURE
                }
            }
        }
        _ => {
            eprintln!("{HELP}");
            ExitCode::from(2)
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn help_lists_repository_validation() {
        assert!(super::HELP.contains("validate"));
    }
}
