//! Dispatch the explicit alignment bootstrap without rewriting historical gates.

use std::path::Path;
use std::process::Command;

pub(crate) fn active(root: &Path) -> bool {
    root.join("spec/nostr_automerge_alignment_refactor_v1/authority.json")
        .is_file()
}

pub(crate) fn validate(root: &Path) -> Result<(), String> {
    let output = Command::new("python3")
        .current_dir(root)
        .env("PYTHONDONTWRITEBYTECODE", "1")
        .arg("scripts/alignment_v1/validation.py")
        .output()
        .map_err(|error| format!("alignment_validation: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "alignment_validation: {}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    print!("{}", String::from_utf8_lossy(&output.stdout));
    Ok(())
}
