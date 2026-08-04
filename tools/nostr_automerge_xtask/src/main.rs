use std::process::{Command, ExitCode};

const HELP: &str = "nostr_automerge_xtask\n\nUSAGE:\n    nostr_automerge_xtask validate\n    nostr_automerge_xtask --help";

fn validation_command(root: &std::path::Path) -> Command {
    let mut command = Command::new("python3");
    command.current_dir(root).arg("scripts/validate_spec.py");
    command
}

fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);
    match args.next().as_deref() {
        Some("-h" | "--help") => {
            println!("{HELP}");
            ExitCode::SUCCESS
        }
        Some("validate") if args.next().is_none() => {
            let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
            match validation_command(&root).status() {
                Ok(status) if status.success() => ExitCode::SUCCESS,
                Ok(status) => ExitCode::from(status.code().unwrap_or(1) as u8),
                Err(error) => {
                    eprintln!("failed to run specification validator: {error}");
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
    use super::validation_command;

    #[test]
    fn routes_validation_to_repository_script() {
        let command = validation_command(std::path::Path::new("repo"));
        assert_eq!(command.get_program(), "python3");
        assert_eq!(
            command.get_args().next().and_then(|arg| arg.to_str()),
            Some("scripts/validate_spec.py")
        );
    }
}
