use std::process::Command;

fn md_cmd() -> Command {
    Command::new(env!("CARGO_BIN_EXE_md"))
}

fn run_md(args: &[&str], stdin: Option<&str>) -> (String, String, i32) {
    let mut cmd = md_cmd();
    cmd.args(args);
    if stdin.is_some() {
        cmd.stdin(std::process::Stdio::piped());
    }
    let mut child = cmd
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("failed to spawn md");

    if let Some(input) = stdin {
        use std::io::Write;
        child
            .stdin
            .as_mut()
            .unwrap()
            .write_all(input.as_bytes())
            .unwrap();
        drop(child.stdin.take());
    }

    let output = child.wait_with_output().expect("failed to wait");
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let code = output.status.code().unwrap_or(-1);
    (stdout, stderr, code)
}

// ── CLI behavior tests (mirror cli_tests.sh) ──────────────

#[test]
fn test_help_exits_0() {
    let (stdout, _, code) = run_md(&["--help"], None);
    assert_eq!(code, 0);
    assert!(
        stdout.to_lowercase().contains("usage"),
        "expected usage text in: {stdout}"
    );
}

#[test]
fn test_width_constrains_output() {
    let input =
        "The quick brown fox jumps over the lazy dog and continues running through the forest.";
    let (stdout, _, code) = run_md(
        &["--no-color", "--no-pager", "--width", "40", "-"],
        Some(input),
    );
    assert_eq!(code, 0);
    for line in stdout.lines() {
        assert!(line.len() <= 40, "line exceeds 40 chars: {:?}", line);
    }
}

#[test]
fn test_nonexistent_file_exits_1() {
    let (_, _, code) = run_md(&["--no-pager", "/tmp/md_nonexistent_file_test"], None);
    assert_eq!(code, 1);
}

#[test]
fn test_empty_file_exits_0() {
    let tmp = std::env::temp_dir().join("md_empty_test.md");
    std::fs::write(&tmp, "").unwrap();
    let (_, _, code) = run_md(&["--no-color", "--no-pager", tmp.to_str().unwrap()], None);
    std::fs::remove_file(&tmp).ok();
    assert_eq!(code, 0);
}

#[test]
fn test_stdin_pipe() {
    let (stdout, _, code) = run_md(&["--no-color", "--no-pager", "-"], Some("# Hello\n"));
    assert_eq!(code, 0);
    assert!(stdout.contains("HELLO"), "H1 should uppercase: {}", stdout);
}

// ── Rendering fixture tests (mirror run_compat_tests.sh) ──

#[test]
fn test_rendering_fixtures() {
    let fixture_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures/rendering");
    let mut tested = 0;

    for entry in std::fs::read_dir(&fixture_dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.extension().map_or(true, |e| e != "md") {
            continue;
        }

        let stem = path.file_stem().unwrap().to_str().unwrap();
        let expected_path = fixture_dir.join(format!("{stem}.expected.txt"));
        if !expected_path.exists() {
            continue;
        }

        let input = std::fs::read_to_string(&path).unwrap();
        let expected = std::fs::read_to_string(&expected_path).unwrap();

        let (stdout, stderr, code) = run_md(
            &["--no-color", "--no-pager", "--width", "60", "-"],
            Some(&input),
        );

        assert_eq!(code, 0, "fixture {stem} exited with {code}: {stderr}");
        assert_eq!(
            stdout.trim_end(),
            expected,
            "fixture {stem} output mismatch"
        );

        tested += 1;
    }

    assert!(
        tested >= 15,
        "expected at least 15 rendering fixtures, found {tested}"
    );
}
