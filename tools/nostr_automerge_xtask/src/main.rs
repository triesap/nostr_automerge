//! Private repository automation entry point.

use std::process::ExitCode;

mod checkpoint_report;
mod requirements;
mod sbom;
mod validate;

const HELP: &str = "nostr_automerge_xtask\n\nUSAGE:\n    nostr_automerge_xtask validate\n    nostr_automerge_xtask checkpoint-report\n    nostr_automerge_xtask sbom\n    nostr_automerge_xtask --help";

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
        Some("sbom") if args.next().is_none() => {
            println!("{}", sbom::generate());
            ExitCode::SUCCESS
        }
        Some("checkpoint-report") if args.next().is_none() => {
            let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
            match checkpoint_report::run(&root) {
                Ok(()) => {
                    println!("PASS: checkpoint conformance report generated");
                    ExitCode::SUCCESS
                }
                Err(error) => {
                    eprintln!("checkpoint conformance report failed: {error}");
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
