use std::path::Path;
use std::process::Command;

fn main() {
    // Only build frontend when the web feature is enabled
    if std::env::var("CARGO_FEATURE_WEB").is_err() {
        return;
    }

    let app_dir = Path::new("src/web/app");
    let dist_index = app_dir.join("dist/index.html");

    // Rerun if frontend sources change
    println!("cargo:rerun-if-changed=src/web/app/src");
    println!("cargo:rerun-if-changed=src/web/app/index.html");
    println!("cargo:rerun-if-changed=src/web/app/package.json");
    println!("cargo:rerun-if-changed=src/web/app/vite.config.ts");
    println!("cargo:rerun-if-changed=src/web/app/tsconfig.json");

    // Skip build if dist is already up to date and we're not in CI
    // (developer can always run `npx vite build` manually)
    if dist_index.exists() && std::env::var("CI").is_err() {
        return;
    }

    // Check if node_modules exist
    if !app_dir.join("node_modules").exists() {
        eprintln!("cargo:warning=Frontend node_modules missing — run `pnpm install` in src/web/app/");
        return;
    }

    let status = Command::new("npx")
        .args(["vite", "build"])
        .current_dir(app_dir)
        .status()
        .expect("failed to run npx vite build");

    assert!(status.success(), "Frontend build failed");
}
