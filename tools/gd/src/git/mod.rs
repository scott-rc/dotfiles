pub mod diff;

use std::path::{Path, PathBuf};
use std::process::Command;

/// Run a git command in the given repo, return stdout or None on failure.
pub fn run(repo: &Path, args: &[&str]) -> Option<String> {
    let out = Command::new("git")
        .args(args)
        .current_dir(repo)
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    Some(String::from_utf8_lossy(&out.stdout).into_owned())
}

/// Get the repo root from any path inside it.
pub fn repo_root(from: &Path) -> Option<PathBuf> {
    let out = run(from, &["rev-parse", "--show-toplevel"])?;
    Some(PathBuf::from(out.trim()))
}

/// List untracked files (respecting .gitignore).
pub fn untracked_files(repo: &Path) -> Vec<String> {
    run(repo, &["ls-files", "--others", "--exclude-standard"])
        .unwrap_or_default()
        .lines()
        .filter(|l| !l.is_empty())
        .map(String::from)
        .collect()
}
