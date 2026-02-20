pub mod diff;

use std::path::{Path, PathBuf};

use tokio::process::Command;

/// Run a git command in the given repo, return stdout or None on failure.
pub async fn run(repo: &Path, args: &[&str]) -> Option<String> {
    let out = Command::new("git")
        .args(args)
        .current_dir(repo)
        .output()
        .await
        .ok()?;
    if !out.status.success() {
        return None;
    }
    Some(String::from_utf8_lossy(&out.stdout).into_owned())
}

/// Get the repo root from any path inside it.
pub async fn repo_root(from: &Path) -> Option<PathBuf> {
    let out = run(from, &["rev-parse", "--show-toplevel"]).await?;
    Some(PathBuf::from(out.trim()))
}
